//! Searchable document types for asset indexing

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use schema::{Asset, AssetType};
use std::path::PathBuf;
use std::collections::HashMap;

/// A searchable document representing an indexed asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDocument {
    /// Unique document ID
    pub id: Uuid,
    
    /// Original asset information
    pub asset_id: Uuid,
    pub file_path: PathBuf,
    pub filename: String,
    pub asset_type: AssetType,
    
    /// File metadata
    pub file_size: u64,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub indexed_at: DateTime<Utc>,
    
    /// Searchable text content
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub transcription: Option<String>,
    pub extracted_text: Option<String>,
    
    /// Visual/audio analysis results
    pub ai_tags: Vec<String>,
    pub ai_caption: Option<String>,
    pub dominant_colors: Vec<String>,
    
    /// Technical metadata
    pub dimensions: Option<(u32, u32)>,
    pub duration: Option<f32>, // in seconds
    pub sample_rate: Option<u32>,
    pub frame_rate: Option<f32>,
    
    /// Preview information
    pub preview_path: Option<PathBuf>,
    pub thumbnail_path: Option<PathBuf>,
    
    /// Vector embeddings for similarity search
    pub visual_embedding: Option<Vec<f32>>,
    pub text_embedding: Option<Vec<f32>>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    
    /// Search optimization
    pub search_text: String, // Combined searchable text
    pub quality_score: f32,  // For ranking
}

impl AssetDocument {
    /// Create a new document from an asset
    pub fn from_asset(asset: &Asset) -> Self {
        let now = Utc::now();
        let filename = asset.current_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        let mut doc = Self {
            id: Uuid::new_v4(),
            asset_id: asset.id,
            file_path: asset.current_path.clone(),
            filename: filename.clone(),
            asset_type: asset.asset_type.clone(),
            file_size: asset.file_size,
            created_at: asset.created_at,
            modified_at: asset.modified_at,
            indexed_at: now,
            title: filename.clone(),
            description: None,
            tags: asset.tags.clone(),
            transcription: asset.metadata.audio.as_ref().and_then(|a| a.transcription.clone()),
            extracted_text: None,
            ai_tags: Vec::new(),
            ai_caption: None,
            dominant_colors: Vec::new(),
            dimensions: asset.metadata.image.as_ref().map(|img| (img.width, img.height)),
            duration: asset.metadata.audio.as_ref().map(|a| a.duration)
                .or_else(|| asset.metadata.video.as_ref().map(|v| v.duration)),
            sample_rate: asset.metadata.audio.as_ref().map(|a| a.sample_rate),
            frame_rate: asset.metadata.video.as_ref().map(|v| v.fps),
            preview_path: asset.preview.as_ref().map(|p| p.thumbnail_path.clone()),
            thumbnail_path: asset.preview.as_ref().map(|p| p.thumbnail_path.clone()),
            visual_embedding: asset.embedding.clone(),
            text_embedding: None,
            metadata: HashMap::new(),
            search_text: String::new(),
            quality_score: 1.0,
        };
        
        // Build search text from available fields
        doc.update_search_text();
        doc
    }
    
    /// Update the combined search text field
    pub fn update_search_text(&mut self) {
        let mut search_parts = Vec::new();
        
        // Core identifiers
        search_parts.push(self.filename.clone());
        search_parts.push(self.title.clone());
        
        // Descriptions and content
        if let Some(desc) = &self.description {
            search_parts.push(desc.clone());
        }
        if let Some(transcript) = &self.transcription {
            search_parts.push(transcript.clone());
        }
        if let Some(text) = &self.extracted_text {
            search_parts.push(text.clone());
        }
        if let Some(caption) = &self.ai_caption {
            search_parts.push(caption.clone());
        }
        
        // Tags
        search_parts.extend(self.tags.clone());
        search_parts.extend(self.ai_tags.clone());
        search_parts.extend(self.dominant_colors.clone());
        
        // Asset type
        search_parts.push(format!("{:?}", self.asset_type).to_lowercase());
        
        // Technical metadata
        if let Some((w, h)) = self.dimensions {
            search_parts.push(format!("{}x{}", w, h));
        }
        
        // Combine all parts
        self.search_text = search_parts.join(" ").to_lowercase();
    }
    
    /// Add tags to the document
    pub fn add_tags(&mut self, tags: Vec<String>) {
        self.tags.extend(tags);
        self.tags.sort();
        self.tags.dedup();
        self.update_search_text();
    }
    
    /// Add AI-generated tags
    pub fn add_ai_tags(&mut self, tags: Vec<String>) {
        self.ai_tags.extend(tags);
        self.ai_tags.sort();
        self.ai_tags.dedup();
        self.update_search_text();
    }
    
    /// Set transcription
    pub fn set_transcription(&mut self, transcription: String) {
        self.transcription = Some(transcription);
        self.update_search_text();
    }
    
    /// Set AI caption
    pub fn set_ai_caption(&mut self, caption: String) {
        self.ai_caption = Some(caption);
        self.update_search_text();
    }
    
    /// Set visual embedding
    pub fn set_visual_embedding(&mut self, embedding: Vec<f32>) {
        self.visual_embedding = Some(embedding);
    }
    
    /// Set text embedding
    pub fn set_text_embedding(&mut self, embedding: Vec<f32>) {
        self.text_embedding = Some(embedding);
    }
    
    /// Calculate quality score based on available metadata
    pub fn calculate_quality_score(&mut self) {
        let mut score = 1.0;
        
        // Bonus for having description
        if self.description.is_some() {
            score += 0.2;
        }
        
        // Bonus for having tags
        if !self.tags.is_empty() {
            score += 0.1 * (self.tags.len() as f32).min(5.0);
        }
        
        // Bonus for AI analysis
        if !self.ai_tags.is_empty() {
            score += 0.1 * (self.ai_tags.len() as f32).min(3.0);
        }
        
        // Bonus for transcription
        if self.transcription.is_some() {
            score += 0.3;
        }
        
        // Bonus for embeddings
        if self.visual_embedding.is_some() {
            score += 0.2;
        }
        if self.text_embedding.is_some() {
            score += 0.2;
        }
        
        // Recent files get slight boost
        let days_old = (Utc::now() - self.created_at).num_days();
        if days_old < 7 {
            score += 0.1;
        }
        
        self.quality_score = score;
    }
    
    /// Get all searchable text fields as a vector
    pub fn get_searchable_fields(&self) -> Vec<&str> {
        let mut fields = vec![self.filename.as_str(), self.title.as_str(), self.search_text.as_str()];
        
        if let Some(desc) = &self.description {
            fields.push(desc.as_str());
        }
        if let Some(transcript) = &self.transcription {
            fields.push(transcript.as_str());
        }
        if let Some(text) = &self.extracted_text {
            fields.push(text.as_str());
        }
        if let Some(caption) = &self.ai_caption {
            fields.push(caption.as_str());
        }
        
        fields
    }
}

/// Search index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Maximum number of results to return
    pub max_results: usize,
    
    /// Minimum similarity score for vector search
    pub min_similarity: f32,
    
    /// Weights for different search components
    pub text_weight: f32,
    pub tag_weight: f32,
    pub vector_weight: f32,
    
    /// Enable fuzzy matching
    pub fuzzy_matching: bool,
    
    /// Minimum query length
    pub min_query_length: usize,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            max_results: 100,
            min_similarity: 0.7,
            text_weight: 1.0,
            tag_weight: 1.5,
            vector_weight: 0.8,
            fuzzy_matching: true,
            min_query_length: 2,
        }
    }
}

/// Search result with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matching document
    pub document: AssetDocument,
    
    /// Overall relevance score
    pub score: f32,
    
    /// Individual component scores
    pub text_score: f32,
    pub tag_score: f32,
    pub vector_score: f32,
    
    /// Matching highlights
    pub highlights: Vec<String>,
    
    /// Reason for match
    pub match_reason: String,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(document: AssetDocument, score: f32) -> Self {
        Self {
            document,
            score,
            text_score: 0.0,
            tag_score: 0.0,
            vector_score: 0.0,
            highlights: Vec::new(),
            match_reason: String::new(),
        }
    }
    
    /// Calculate combined score using weights
    pub fn calculate_weighted_score(&mut self, config: &IndexConfig) {
        self.score = (self.text_score * config.text_weight)
            + (self.tag_score * config.tag_weight)
            + (self.vector_score * config.vector_weight);
        
        // Apply quality bonus
        self.score *= self.document.quality_score;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_document_search_text_generation() {
        let asset = Asset {
            id: Uuid::new_v4(),
            file_path: PathBuf::from("test_image.jpg"),
            asset_type: AssetType::Image,
            file_size: 1024,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            preview_path: None,
            thumbnail_path: None,
        };
        
        let mut doc = AssetDocument::from_asset(&asset);
        doc.add_tags(vec!["photo".to_string(), "vacation".to_string()]);
        doc.set_ai_caption("A beautiful sunset over the ocean".to_string());
        
        assert!(doc.search_text.contains("test_image.jpg"));
        assert!(doc.search_text.contains("photo"));
        assert!(doc.search_text.contains("vacation"));
        assert!(doc.search_text.contains("beautiful sunset"));
        assert!(doc.search_text.contains("image"));
    }
    
    #[test]
    fn test_quality_score_calculation() {
        let asset = Asset {
            id: Uuid::new_v4(),
            file_path: PathBuf::from("test.mp3"),
            asset_type: AssetType::Audio,
            file_size: 1024,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            preview_path: None,
            thumbnail_path: None,
        };
        
        let mut doc = AssetDocument::from_asset(&asset);
        let initial_score = doc.quality_score;
        
        doc.set_transcription("This is a test audio file".to_string());
        doc.add_tags(vec!["music".to_string(), "test".to_string()]);
        doc.calculate_quality_score();
        
        assert!(doc.quality_score > initial_score);
    }
}
