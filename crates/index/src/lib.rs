//! Search and indexing functionality
//! 
//! This crate provides comprehensive search capabilities including:
//! - Text search with TF-IDF scoring
//! - Vector similarity search for embeddings
//! - Hybrid search combining text and vector results
//! - Persistent storage using sled database

use schema::{DamResult, Asset};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use uuid::Uuid;
use tracing::{info, warn, debug};
use serde::{Serialize, Deserialize};

pub mod error;
pub mod document;
pub mod vector;
pub mod text_search;

pub use error::*;
pub use document::*;
pub use vector::*;
pub use text_search::*;

/// Main search and indexing service
pub struct IndexService {
    /// Text search index
    text_index: TextIndex,
    /// Vector similarity store
    vector_store: VectorStore,
    /// Document storage (sled database)
    doc_store: sled::Db,
    /// Configuration
    config: IndexConfig,
    /// Storage directory
    storage_dir: PathBuf,
}

impl IndexService {
    /// Create a new index service
    pub fn new() -> DamResult<Self> {
        let storage_dir = PathBuf::from("data/index");
        Self::with_storage_dir(storage_dir)
    }
    
    /// Create index service with custom storage directory
    pub fn with_storage_dir<P: AsRef<Path>>(storage_dir: P) -> DamResult<Self> {
        let storage_dir = storage_dir.as_ref().to_path_buf();
        
        info!("Initializing index service with storage: {}", storage_dir.display());
        
        // Create storage directory
        std::fs::create_dir_all(&storage_dir)?;
        
        // Open sled database
        let db_path = storage_dir.join("documents.db");
        let doc_store = sled::open(db_path)
            .map_err(|e| IndexError::DatabaseError(e.to_string()))?;
        
        let config = IndexConfig::default();
        let text_index = TextIndex::new(config.clone());
        let vector_store = VectorStore::new();
        
        let mut service = Self {
            text_index,
            vector_store,
            doc_store,
            config,
            storage_dir,
        };
        
        // Load existing documents
        service.reload_from_storage()?;
        
        info!("Index service initialized successfully");
        Ok(service)
    }
    
    /// Add or update an asset in the search index
    pub async fn index_asset(&mut self, asset: &Asset) -> DamResult<()> {
        debug!("Indexing asset: {}", asset.current_path.display());
        
        let mut document = AssetDocument::from_asset(asset);
        
        // Calculate quality score
        document.calculate_quality_score();
        
        // Add to text index
        self.text_index.add_document(&document)?;
        
        // Store document in database
        let doc_json = serde_json::to_vec(&document)?;
        self.doc_store.insert(document.id.as_bytes(), doc_json)
            .map_err(|e| IndexError::DatabaseError(e.to_string()))?;
        
        debug!("Successfully indexed asset: {}", asset.current_path.display());
        Ok(())
    }
    
    /// Update document with AI processing results
    pub async fn update_with_ai_results(
        &mut self, 
        asset_id: Uuid, 
        tags: Option<Vec<String>>, 
        caption: Option<String>,
        transcription: Option<String>,
        visual_embedding: Option<Vec<f32>>,
        text_embedding: Option<Vec<f32>>
    ) -> DamResult<()> {
        debug!("Updating AI results for asset: {}", asset_id);
        
        // Find document by asset ID
        let mut document = self.find_document_by_asset_id(&asset_id)?
            .ok_or_else(|| IndexError::DocumentNotFound(format!("Asset not found: {}", asset_id)))?;
        
        // Update with AI results
        if let Some(tags) = tags {
            document.add_ai_tags(tags);
        }
        
        if let Some(caption) = caption {
            document.set_ai_caption(caption);
        }
        
        if let Some(transcription) = transcription {
            document.set_transcription(transcription);
        }
        
        if let Some(embedding) = visual_embedding {
            document.set_visual_embedding(embedding.clone());
            self.vector_store.add_visual_embedding(document.id, embedding)?;
        }
        
        if let Some(embedding) = text_embedding {
            document.set_text_embedding(embedding.clone());
            self.vector_store.add_text_embedding(document.id, embedding)?;
        }
        
        // Recalculate quality score
        document.calculate_quality_score();
        
        // Update text index
        self.text_index.add_document(&document)?;
        
        // Update document storage
        let doc_json = serde_json::to_vec(&document)?;
        self.doc_store.insert(document.id.as_bytes(), doc_json)
            .map_err(|e| IndexError::DatabaseError(e.to_string()))?;
        
        debug!("Successfully updated AI results for asset: {}", asset_id);
        Ok(())
    }
    
    /// Remove an asset from the index
    pub async fn remove_asset(&mut self, asset_id: Uuid) -> DamResult<()> {
        debug!("Removing asset from index: {}", asset_id);
        
        // Find document
        if let Some(document) = self.find_document_by_asset_id(&asset_id)? {
            // Remove from text index
            self.text_index.remove_document(&document.id);
            
            // Remove from vector store
            self.vector_store.remove_document(&document.id);
            
            // Remove from document storage
            self.doc_store.remove(document.id.as_bytes())
                .map_err(|e| IndexError::DatabaseError(e.to_string()))?;
            
            debug!("Successfully removed asset from index: {}", asset_id);
        }
        
        Ok(())
    }
    
    /// Search for assets using text query
    pub async fn search_text(&self, query: &str, max_results: usize) -> DamResult<Vec<SearchResult>> {
        debug!("Text search query: '{}'", query);
        
        let text_matches = self.text_index.search(query, max_results)?;
        let mut results = Vec::new();
        
        for text_match in text_matches {
            if let Some(document) = self.get_document(&text_match.document_id)? {
                let mut result = SearchResult::new(document, text_match.score);
                result.text_score = text_match.score;
                result.match_reason = format!("Text match in: {}", 
                    text_match.matches.iter()
                        .map(|m| m.field_name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                result.highlights = text_match.matches.iter()
                    .map(|m| format!("{}: {}", m.field_name, m.match_text))
                    .collect();
                
                results.push(result);
            }
        }
        
        debug!("Text search returned {} results", results.len());
        Ok(results)
    }
    
    /// Search for visually similar assets
    pub async fn search_visual_similar(&self, query_embedding: &[f32], max_results: usize) -> DamResult<Vec<SearchResult>> {
        debug!("Visual similarity search with {} dimensional embedding", query_embedding.len());
        
        let vector_matches = self.vector_store.find_visual_similar(
            query_embedding, 
            max_results, 
            self.config.min_similarity
        )?;
        
        let mut results = Vec::new();
        
        for vector_match in vector_matches {
            if let Some(document) = self.get_document(&vector_match.document_id)? {
                let mut result = SearchResult::new(document, vector_match.similarity);
                result.vector_score = vector_match.similarity;
                result.match_reason = "Visual similarity".to_string();
                
                results.push(result);
            }
        }
        
        debug!("Visual similarity search returned {} results", results.len());
        Ok(results)
    }
    
    /// Find assets similar to a specific asset
    pub async fn find_similar(&self, asset_id: Uuid, embedding_type: EmbeddingType, max_results: usize) -> DamResult<Vec<SearchResult>> {
        debug!("Finding similar assets to: {}", asset_id);
        
        // Find document
        let document = self.find_document_by_asset_id(&asset_id)?
            .ok_or_else(|| IndexError::DocumentNotFound(format!("Asset not found: {}", asset_id)))?;
        
        let vector_matches = self.vector_store.find_similar_to_document(
            &document.id,
            embedding_type,
            max_results,
            self.config.min_similarity
        )?;
        
        let mut results = Vec::new();
        
        for vector_match in vector_matches {
            if let Some(document) = self.get_document(&vector_match.document_id)? {
                let mut result = SearchResult::new(document, vector_match.similarity);
                result.vector_score = vector_match.similarity;
                result.match_reason = format!("Similar to asset {}", asset_id);
                
                results.push(result);
            }
        }
        
        debug!("Similarity search returned {} results", results.len());
        Ok(results)
    }
    
    /// Hybrid search combining text and vector search
    pub async fn search_hybrid(&self, query: &str, query_embedding: Option<&[f32]>, max_results: usize) -> DamResult<Vec<SearchResult>> {
        debug!("Hybrid search: '{}' with embedding: {}", query, query_embedding.is_some());
        
        let mut all_results: HashMap<Uuid, SearchResult> = HashMap::new();
        
        // Text search
        if !query.trim().is_empty() {
            let text_results = self.search_text(query, max_results * 2).await?;
            for mut result in text_results {
                result.calculate_weighted_score(&self.config);
                all_results.insert(result.document.id, result);
            }
        }
        
        // Vector search
        if let Some(embedding) = query_embedding {
            let vector_results = self.search_visual_similar(embedding, max_results * 2).await?;
            for mut result in vector_results {
                result.calculate_weighted_score(&self.config);
                
                // Combine with existing text result if present
                if let Some(existing) = all_results.get_mut(&result.document.id) {
                    existing.vector_score = result.vector_score;
                    existing.score = (existing.text_score * self.config.text_weight) 
                        + (result.vector_score * self.config.vector_weight);
                    existing.match_reason = format!("{} + Visual similarity", existing.match_reason);
                } else {
                    all_results.insert(result.document.id, result);
                }
            }
        }
        
        // Sort and limit results
        let mut results: Vec<SearchResult> = all_results.into_values().collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(max_results);
        
        debug!("Hybrid search returned {} results", results.len());
        Ok(results)
    }
    
    /// Get search statistics
    pub fn get_stats(&self) -> IndexStats {
        let text_stats = self.text_index.get_stats();
        let vector_stats = self.vector_store.get_stats();
        
        IndexStats {
            total_documents: text_stats.total_documents,
            total_terms: text_stats.total_terms,
            avg_terms_per_doc: text_stats.avg_terms_per_doc,
            visual_embeddings: vector_stats.visual_embeddings_count,
            text_embeddings: vector_stats.text_embeddings_count,
            visual_dimension: vector_stats.visual_dimension,
            text_dimension: vector_stats.text_dimension,
        }
    }
    
    /// Clear all indexes
    pub async fn clear(&mut self) -> DamResult<()> {
        info!("Clearing all search indexes");
        
        self.text_index.clear();
        self.vector_store.clear();
        self.doc_store.clear()
            .map_err(|e| IndexError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Reload documents from storage
    fn reload_from_storage(&mut self) -> DamResult<()> {
        info!("Reloading documents from storage");
        
        let mut documents = Vec::new();
        
        // Load all documents from storage
        for result in self.doc_store.iter() {
            let (_, value) = result.map_err(|e| IndexError::DatabaseError(e.to_string()))?;
            if let Ok(document) = serde_json::from_slice::<AssetDocument>(&value) {
                documents.push(document);
            }
        }
        
        info!("Loaded {} documents from storage", documents.len());
        
        // Rebuild text index
        for doc in &documents {
            if let Err(e) = self.text_index.add_document(doc) {
                warn!("Failed to add document to text index: {}", e);
            }
        }
        
        // Rebuild vector store
        if let Err(e) = self.vector_store.load_from_documents(&documents) {
            warn!("Failed to load vector embeddings: {}", e);
        }
        
        info!("Successfully reloaded search indexes");
        Ok(())
    }
    
    /// Get document by ID
    fn get_document(&self, doc_id: &Uuid) -> DamResult<Option<AssetDocument>> {
        if let Some(data) = self.doc_store.get(doc_id.as_bytes())
            .map_err(|e| IndexError::DatabaseError(e.to_string()))? {
            let document: AssetDocument = serde_json::from_slice(&data)?;
            Ok(Some(document))
        } else {
            Ok(None)
        }
    }
    
    /// Find document by asset ID
    fn find_document_by_asset_id(&self, asset_id: &Uuid) -> DamResult<Option<AssetDocument>> {
        for result in self.doc_store.iter() {
            let (_, value) = result.map_err(|e| IndexError::DatabaseError(e.to_string()))?;
            if let Ok(document) = serde_json::from_slice::<AssetDocument>(&value) {
                if document.asset_id == *asset_id {
                    return Ok(Some(document));
                }
            }
        }
        Ok(None)
    }
}

impl Default for IndexService {
    fn default() -> Self {
        Self::new().expect("Failed to create IndexService")
    }
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_documents: usize,
    pub total_terms: usize,
    pub avg_terms_per_doc: f32,
    pub visual_embeddings: usize,
    pub text_embeddings: usize,
    pub visual_dimension: Option<usize>,
    pub text_dimension: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use schema::{AssetType, FileFormat, AssetMetadata, VersionInfo};
    use std::path::PathBuf;
    use chrono::Utc;
    use tempfile::TempDir;
    
    fn create_test_asset(filename: &str) -> Asset {
        let path = PathBuf::from(filename);
        let now = Utc::now();
        
        Asset {
            id: Uuid::new_v4(),
            original_path: path.clone(),
            current_path: path,
            asset_type: AssetType::Image,
            file_size: 1024,
            format: FileFormat {
                extension: "jpg".to_string(),
                mime_type: Some("image/jpeg".to_string()),
                version: None,
                supported: true,
            },
            created_at: now,
            modified_at: now,
            tags: Vec::new(),
            metadata: AssetMetadata::default(),
            preview: None,
            embedding: None,
            version_info: VersionInfo {
                current_version: "v1".to_string(),
                version_count: 1,
                last_snapshot: now,
                has_changes: false,
            },
        }
    }
    
    #[tokio::test]
    async fn test_index_service_creation() {
        let temp_dir = TempDir::new().unwrap();
        let service = IndexService::with_storage_dir(temp_dir.path());
        assert!(service.is_ok());
    }
    
    #[tokio::test]
    async fn test_asset_indexing_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let mut service = IndexService::with_storage_dir(temp_dir.path()).unwrap();
        
        // Create test asset
        let asset = create_test_asset("vacation_photo.jpg");
        let asset_id = asset.id;
        
        // Index asset
        service.index_asset(&asset).await.unwrap();
        
        // Search for it
        let results = service.search_text("vacation", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document.asset_id, asset_id);
        
        // Remove asset
        service.remove_asset(asset_id).await.unwrap();
        
        // Should not find it anymore
        let results = service.search_text("vacation", 10).await.unwrap();
        assert_eq!(results.len(), 0);
    }
    
    #[tokio::test]
    async fn test_ai_results_update() {
        let temp_dir = TempDir::new().unwrap();
        let mut service = IndexService::with_storage_dir(temp_dir.path()).unwrap();
        
        // Create and index asset
        let asset = create_test_asset("test.jpg");
        let asset_id = asset.id;
        service.index_asset(&asset).await.unwrap();
        
        // Update with AI results
        service.update_with_ai_results(
            asset_id,
            Some(vec!["cat".to_string(), "cute".to_string()]),
            Some("A cute cat sitting on a chair".to_string()),
            None,
            Some(vec![0.1, 0.2, 0.3, 0.4]),
            None
        ).await.unwrap();
        
        // Search should now find it by AI tags
        let results = service.search_text("cat", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document.asset_id, asset_id);
        
        // Visual similarity search should work
        let similar_results = service.search_visual_similar(&[0.1, 0.2, 0.3, 0.4], 5).await.unwrap();
        assert_eq!(similar_results.len(), 1);
    }
}
