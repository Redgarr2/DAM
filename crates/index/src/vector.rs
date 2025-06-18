//! Vector similarity search for embeddings

use crate::error::IndexError;
use crate::document::AssetDocument;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Vector similarity search result
#[derive(Debug, Clone)]
pub struct VectorMatch {
    pub document_id: Uuid,
    pub similarity: f32,
    pub embedding_type: EmbeddingType,
}

/// Type of embedding used for search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingType {
    Visual,
    Text,
}

/// In-memory vector store for similarity search
#[derive(Debug, Clone)]
pub struct VectorStore {
    /// Visual embeddings indexed by document ID
    visual_embeddings: HashMap<Uuid, Vec<f32>>,
    /// Text embeddings indexed by document ID
    text_embeddings: HashMap<Uuid, Vec<f32>>,
    /// Dimension of visual embeddings
    visual_dim: Option<usize>,
    /// Dimension of text embeddings
    text_dim: Option<usize>,
}

impl VectorStore {
    /// Create a new vector store
    pub fn new() -> Self {
        Self {
            visual_embeddings: HashMap::new(),
            text_embeddings: HashMap::new(),
            visual_dim: None,
            text_dim: None,
        }
    }
    
    /// Add or update visual embedding for a document
    pub fn add_visual_embedding(&mut self, doc_id: Uuid, embedding: Vec<f32>) -> Result<(), IndexError> {
        // Validate dimension consistency
        if let Some(expected_dim) = self.visual_dim {
            if embedding.len() != expected_dim {
                return Err(IndexError::VectorError(format!(
                    "Visual embedding dimension mismatch: expected {}, got {}",
                    expected_dim, embedding.len()
                )));
            }
        } else {
            self.visual_dim = Some(embedding.len());
        }
        
        // Normalize the embedding
        let normalized = normalize_vector(&embedding);
        self.visual_embeddings.insert(doc_id, normalized);
        Ok(())
    }
    
    /// Add or update text embedding for a document
    pub fn add_text_embedding(&mut self, doc_id: Uuid, embedding: Vec<f32>) -> Result<(), IndexError> {
        // Validate dimension consistency
        if let Some(expected_dim) = self.text_dim {
            if embedding.len() != expected_dim {
                return Err(IndexError::VectorError(format!(
                    "Text embedding dimension mismatch: expected {}, got {}",
                    expected_dim, embedding.len()
                )));
            }
        } else {
            self.text_dim = Some(embedding.len());
        }
        
        // Normalize the embedding
        let normalized = normalize_vector(&embedding);
        self.text_embeddings.insert(doc_id, normalized);
        Ok(())
    }
    
    /// Remove embeddings for a document
    pub fn remove_document(&mut self, doc_id: &Uuid) {
        self.visual_embeddings.remove(doc_id);
        self.text_embeddings.remove(doc_id);
    }
    
    /// Find similar documents using visual embedding
    pub fn find_visual_similar(&self, query_embedding: &[f32], top_k: usize, min_similarity: f32) -> Result<Vec<VectorMatch>, IndexError> {
        if self.visual_embeddings.is_empty() {
            return Ok(Vec::new());
        }
        
        // Normalize query embedding
        let normalized_query = normalize_vector(query_embedding);
        
        // Calculate similarities
        let mut similarities: Vec<VectorMatch> = self.visual_embeddings
            .iter()
            .map(|(doc_id, embedding)| {
                let similarity = cosine_similarity(&normalized_query, embedding);
                VectorMatch {
                    document_id: *doc_id,
                    similarity,
                    embedding_type: EmbeddingType::Visual,
                }
            })
            .filter(|m| m.similarity >= min_similarity)
            .collect();
        
        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        
        // Take top k
        similarities.truncate(top_k);
        
        Ok(similarities)
    }
    
    /// Find similar documents using text embedding
    pub fn find_text_similar(&self, query_embedding: &[f32], top_k: usize, min_similarity: f32) -> Result<Vec<VectorMatch>, IndexError> {
        if self.text_embeddings.is_empty() {
            return Ok(Vec::new());
        }
        
        // Normalize query embedding
        let normalized_query = normalize_vector(query_embedding);
        
        // Calculate similarities
        let mut similarities: Vec<VectorMatch> = self.text_embeddings
            .iter()
            .map(|(doc_id, embedding)| {
                let similarity = cosine_similarity(&normalized_query, embedding);
                VectorMatch {
                    document_id: *doc_id,
                    similarity,
                    embedding_type: EmbeddingType::Text,
                }
            })
            .filter(|m| m.similarity >= min_similarity)
            .collect();
        
        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        
        // Take top k
        similarities.truncate(top_k);
        
        Ok(similarities)
    }
    
    /// Find similar documents to a given document
    pub fn find_similar_to_document(&self, doc_id: &Uuid, embedding_type: EmbeddingType, top_k: usize, min_similarity: f32) -> Result<Vec<VectorMatch>, IndexError> {
        match embedding_type {
            EmbeddingType::Visual => {
                if let Some(query_embedding) = self.visual_embeddings.get(doc_id) {
                    let mut results = self.find_visual_similar(query_embedding, top_k + 1, min_similarity)?;
                    // Remove the query document itself
                    results.retain(|m| m.document_id != *doc_id);
                    results.truncate(top_k);
                    Ok(results)
                } else {
                    Err(IndexError::DocumentNotFound(format!("No visual embedding found for document: {}", doc_id)))
                }
            }
            EmbeddingType::Text => {
                if let Some(query_embedding) = self.text_embeddings.get(doc_id) {
                    let mut results = self.find_text_similar(query_embedding, top_k + 1, min_similarity)?;
                    // Remove the query document itself
                    results.retain(|m| m.document_id != *doc_id);
                    results.truncate(top_k);
                    Ok(results)
                } else {
                    Err(IndexError::DocumentNotFound(format!("No text embedding found for document: {}", doc_id)))
                }
            }
        }
    }
    
    /// Get statistics about the vector store
    pub fn get_stats(&self) -> VectorStoreStats {
        VectorStoreStats {
            visual_embeddings_count: self.visual_embeddings.len(),
            text_embeddings_count: self.text_embeddings.len(),
            visual_dimension: self.visual_dim,
            text_dimension: self.text_dim,
        }
    }
    
    /// Clear all embeddings
    pub fn clear(&mut self) {
        self.visual_embeddings.clear();
        self.text_embeddings.clear();
        self.visual_dim = None;
        self.text_dim = None;
    }
    
    /// Load embeddings from documents
    pub fn load_from_documents(&mut self, documents: &[AssetDocument]) -> Result<(), IndexError> {
        for doc in documents {
            if let Some(ref visual_emb) = doc.visual_embedding {
                self.add_visual_embedding(doc.id, visual_emb.clone())?;
            }
            if let Some(ref text_emb) = doc.text_embedding {
                self.add_text_embedding(doc.id, text_emb.clone())?;
            }
        }
        Ok(())
    }
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreStats {
    pub visual_embeddings_count: usize,
    pub text_embeddings_count: usize,
    pub visual_dimension: Option<usize>,
    pub text_dimension: Option<usize>,
}

/// Calculate cosine similarity between two normalized vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vector dimensions must match");
    
    // Since vectors are normalized, cosine similarity is just dot product
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Normalize a vector to unit length
fn normalize_vector(vector: &[f32]) -> Vec<f32> {
    let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if magnitude == 0.0 {
        // Return zero vector if input is zero
        return vector.to_vec();
    }
    
    vector.iter().map(|x| x / magnitude).collect()
}

/// Calculate Euclidean distance between two vectors
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vector dimensions must match");
    
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Calculate Manhattan distance between two vectors
pub fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vector dimensions must match");
    
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vector_normalization() {
        let vector = vec![3.0, 4.0, 0.0];
        let normalized = normalize_vector(&vector);
        
        // Should have unit length
        let magnitude: f32 = normalized.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 1e-6);
        
        // Components should be 0.6, 0.8, 0.0
        assert!((normalized[0] - 0.6).abs() < 1e-6);
        assert!((normalized[1] - 0.8).abs() < 1e-6);
        assert!((normalized[2] - 0.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
        
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);
        
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - (-1.0)).abs() < 1e-6);
    }
    
    #[test]
    fn test_vector_store_operations() {
        let mut store = VectorStore::new();
        let doc_id = Uuid::new_v4();
        let embedding = vec![0.1, 0.2, 0.3, 0.4];
        
        // Add embedding
        store.add_visual_embedding(doc_id, embedding.clone()).unwrap();
        
        // Search for similar
        let results = store.find_visual_similar(&embedding, 5, 0.5).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, doc_id);
        assert!(results[0].similarity > 0.99); // Should be very similar to itself
        
        // Remove document
        store.remove_document(&doc_id);
        let results = store.find_visual_similar(&embedding, 5, 0.5).unwrap();
        assert_eq!(results.len(), 0);
    }
    
    #[test]
    fn test_dimension_validation() {
        let mut store = VectorStore::new();
        let doc_id1 = Uuid::new_v4();
        let doc_id2 = Uuid::new_v4();
        
        // Add first embedding
        store.add_visual_embedding(doc_id1, vec![0.1, 0.2, 0.3]).unwrap();
        
        // Try to add embedding with different dimension
        let result = store.add_visual_embedding(doc_id2, vec![0.1, 0.2]);
        assert!(result.is_err());
    }
}
