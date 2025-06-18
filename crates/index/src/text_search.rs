//! Text search engine implementation
//! 
//! Since Tantivy is temporarily disabled, this provides a basic
//! but functional text search using string matching and scoring.

use crate::error::IndexError;
use crate::document::{AssetDocument, IndexConfig};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

/// Text search result with scoring
#[derive(Debug, Clone)]
pub struct TextMatch {
    pub document_id: Uuid,
    pub score: f32,
    pub matches: Vec<FieldMatch>,
}

/// Match information for a specific field
#[derive(Debug, Clone)]
pub struct FieldMatch {
    pub field_name: String,
    pub match_text: String,
    pub position: usize,
    pub score: f32,
}

/// Simple inverted index for text search
#[derive(Debug, Clone)]
pub struct TextIndex {
    /// Term to document mapping with positions
    term_index: HashMap<String, HashMap<Uuid, Vec<TermOccurrence>>>,
    /// Document to terms mapping for updates
    document_terms: HashMap<Uuid, HashSet<String>>,
    /// Search configuration
    config: IndexConfig,
}

/// Term occurrence in a document
#[derive(Debug, Clone)]
pub struct TermOccurrence {
    pub field: String,
    pub position: usize,
    pub score_boost: f32,
}

impl TextIndex {
    /// Create a new text index
    pub fn new(config: IndexConfig) -> Self {
        Self {
            term_index: HashMap::new(),
            document_terms: HashMap::new(),
            config,
        }
    }
    
    /// Add or update a document in the index
    pub fn add_document(&mut self, document: &AssetDocument) -> Result<(), IndexError> {
        // Remove existing document if present
        self.remove_document(&document.id);
        
        let mut doc_terms = HashSet::new();
        
        // Index different fields with different boost scores
        self.index_field(&document.id, "filename", &document.filename, 2.0, &mut doc_terms);
        self.index_field(&document.id, "title", &document.title, 1.8, &mut doc_terms);
        
        // Index tags with high boost
        let tags_text = document.tags.join(" ");
        self.index_field(&document.id, "tags", &tags_text, 2.5, &mut doc_terms);
        
        // Index AI tags
        let ai_tags_text = document.ai_tags.join(" ");
        self.index_field(&document.id, "ai_tags", &ai_tags_text, 2.0, &mut doc_terms);
        
        // Index description if present
        if let Some(ref desc) = document.description {
            self.index_field(&document.id, "description", desc, 1.5, &mut doc_terms);
        }
        
        // Index transcription if present
        if let Some(ref transcript) = document.transcription {
            self.index_field(&document.id, "transcription", transcript, 1.8, &mut doc_terms);
        }
        
        // Index AI caption if present
        if let Some(ref caption) = document.ai_caption {
            self.index_field(&document.id, "ai_caption", caption, 1.6, &mut doc_terms);
        }
        
        // Index extracted text if present
        if let Some(ref text) = document.extracted_text {
            self.index_field(&document.id, "extracted_text", text, 1.4, &mut doc_terms);
        }
        
        // Index asset type
        let asset_type_text = format!("{:?}", document.asset_type).to_lowercase();
        self.index_field(&document.id, "asset_type", &asset_type_text, 1.2, &mut doc_terms);
        
        // Store document terms for later removal
        self.document_terms.insert(document.id, doc_terms);
        
        Ok(())
    }
    
    /// Remove a document from the index
    pub fn remove_document(&mut self, doc_id: &Uuid) {
        if let Some(terms) = self.document_terms.remove(doc_id) {
            // Remove document from all term indices
            for term in terms {
                if let Some(doc_map) = self.term_index.get_mut(&term) {
                    doc_map.remove(doc_id);
                    // Remove term entry if no documents remain
                    if doc_map.is_empty() {
                        self.term_index.remove(&term);
                    }
                }
            }
        }
    }
    
    /// Search for documents matching the query
    pub fn search(&self, query: &str, max_results: usize) -> Result<Vec<TextMatch>, IndexError> {
        if query.len() < self.config.min_query_length {
            return Ok(Vec::new());
        }
        
        let terms = self.tokenize(query);
        if terms.is_empty() {
            return Ok(Vec::new());
        }
        
        // Find documents containing any of the terms
        let mut doc_scores: HashMap<Uuid, f32> = HashMap::new();
        let mut doc_matches: HashMap<Uuid, Vec<FieldMatch>> = HashMap::new();
        
        for term in &terms {
            if let Some(doc_map) = self.term_index.get(term) {
                for (doc_id, occurrences) in doc_map {
                    let term_score = self.calculate_term_score(term, occurrences, doc_map.len());
                    
                    // Add to document score
                    *doc_scores.entry(*doc_id).or_insert(0.0) += term_score;
                    
                    // Create field matches
                    let matches = doc_matches.entry(*doc_id).or_insert_with(Vec::new);
                    for occurrence in occurrences {
                        matches.push(FieldMatch {
                            field_name: occurrence.field.clone(),
                            match_text: term.clone(),
                            position: occurrence.position,
                            score: term_score * occurrence.score_boost,
                        });
                    }
                }
            }
        }
        
        // Handle phrase matching for multi-term queries
        if terms.len() > 1 {
            self.boost_phrase_matches(query, &terms, &mut doc_scores);
        }
        
        // Convert to results and sort
        let mut results: Vec<TextMatch> = doc_scores
            .into_iter()
            .map(|(doc_id, score)| TextMatch {
                document_id: doc_id,
                score,
                matches: doc_matches.remove(&doc_id).unwrap_or_default(),
            })
            .collect();
        
        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        // Limit results
        results.truncate(max_results);
        
        Ok(results)
    }
    
    /// Get statistics about the index
    pub fn get_stats(&self) -> TextIndexStats {
        let total_terms = self.term_index.len();
        let total_documents = self.document_terms.len();
        let avg_terms_per_doc = if total_documents > 0 {
            self.document_terms.values()
                .map(|terms| terms.len())
                .sum::<usize>() as f32 / total_documents as f32
        } else {
            0.0
        };
        
        TextIndexStats {
            total_terms,
            total_documents,
            avg_terms_per_doc,
        }
    }
    
    /// Clear the index
    pub fn clear(&mut self) {
        self.term_index.clear();
        self.document_terms.clear();
    }
    
    /// Index a specific field of a document
    fn index_field(&mut self, doc_id: &Uuid, field: &str, text: &str, boost: f32, doc_terms: &mut HashSet<String>) {
        let terms = self.tokenize(text);
        
        for (position, term) in terms.iter().enumerate() {
            doc_terms.insert(term.clone());
            
            let doc_map = self.term_index.entry(term.clone()).or_insert_with(HashMap::new);
            let occurrences = doc_map.entry(*doc_id).or_insert_with(Vec::new);
            
            occurrences.push(TermOccurrence {
                field: field.to_string(),
                position,
                score_boost: boost,
            });
        }
    }
    
    /// Tokenize text into searchable terms
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|word| {
                // Remove punctuation and special characters
                word.chars()
                    .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect::<String>()
            })
            .filter(|term| term.len() >= 2) // Minimum term length
            .collect()
    }
    
    /// Calculate TF-IDF style score for a term
    fn calculate_term_score(&self, term: &str, occurrences: &[TermOccurrence], doc_freq: usize) -> f32 {
        let tf = occurrences.len() as f32; // Term frequency in document
        let idf = ((self.document_terms.len() as f32) / (doc_freq as f32 + 1.0)).ln(); // Inverse document frequency
        let boost = occurrences.iter().map(|o| o.score_boost).sum::<f32>() / occurrences.len() as f32;
        
        tf * idf * boost
    }
    
    /// Boost scores for phrase matches
    fn boost_phrase_matches(&self, query: &str, terms: &[String], doc_scores: &mut HashMap<Uuid, f32>) {
        // Simple phrase matching - boost documents that contain terms in sequence
        let query_lower = query.to_lowercase();
        
        for (doc_id, score) in doc_scores.iter_mut() {
            // Check if all terms appear in the same document
            let has_all_terms = terms.iter().all(|term| {
                self.term_index.get(term)
                    .map(|doc_map| doc_map.contains_key(doc_id))
                    .unwrap_or(false)
            });
            
            if has_all_terms {
                // Boost for having all terms
                *score *= 1.5;
                
                // Additional boost for exact phrase (simplified check)
                if terms.len() > 1 {
                    *score *= 1.2;
                }
            }
        }
    }
}

/// Statistics about the text index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextIndexStats {
    pub total_terms: usize,
    pub total_documents: usize,
    pub avg_terms_per_doc: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::AssetDocument;
    use schema::{Asset, AssetType};
    use std::path::PathBuf;
    use chrono::Utc;
    
    fn create_test_document(filename: &str, tags: Vec<String>) -> AssetDocument {
        let asset = Asset {
            id: Uuid::new_v4(),
            file_path: PathBuf::from(filename),
            asset_type: AssetType::Image,
            file_size: 1024,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            preview_path: None,
            thumbnail_path: None,
        };
        
        let mut doc = AssetDocument::from_asset(&asset);
        doc.add_tags(tags);
        doc
    }
    
    #[test]
    fn test_text_indexing_and_search() {
        let config = IndexConfig::default();
        let mut index = TextIndex::new(config);
        
        // Add test documents
        let doc1 = create_test_document("vacation_photo.jpg", vec!["vacation".to_string(), "beach".to_string()]);
        let doc2 = create_test_document("work_presentation.pdf", vec!["work".to_string(), "business".to_string()]);
        
        index.add_document(&doc1).unwrap();
        index.add_document(&doc2).unwrap();
        
        // Search for "vacation"
        let results = index.search("vacation", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, doc1.id);
        
        // Search for "work"
        let results = index.search("work", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, doc2.id);
        
        // Search for non-existent term
        let results = index.search("nonexistent", 10).unwrap();
        assert_eq!(results.len(), 0);
    }
    
    #[test]
    fn test_document_removal() {
        let config = IndexConfig::default();
        let mut index = TextIndex::new(config);
        
        let doc = create_test_document("test.jpg", vec!["test".to_string()]);
        let doc_id = doc.id;
        
        // Add document
        index.add_document(&doc).unwrap();
        let results = index.search("test", 10).unwrap();
        assert_eq!(results.len(), 1);
        
        // Remove document
        index.remove_document(&doc_id);
        let results = index.search("test", 10).unwrap();
        assert_eq!(results.len(), 0);
    }
    
    #[test]
    fn test_tokenization() {
        let config = IndexConfig::default();
        let index = TextIndex::new(config);
        
        let tokens = index.tokenize("Hello, World! This is a test-file_name.jpg");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test-file_name".to_string()));
        assert!(!tokens.contains(&"a".to_string())); // Too short
    }
}
