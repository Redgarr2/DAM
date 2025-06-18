//! Search-related data structures
//! 
//! Defines types for search queries, results, and indexing operations.

use crate::{Asset, AssetType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A search query with multiple criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Text search terms
    pub text: Option<String>,
    
    /// Asset type filter
    pub asset_type: Option<AssetType>,
    
    /// Tag filters (all must match)
    pub tags: Vec<String>,
    
    /// File extension filter
    pub extensions: Vec<String>,
    
    /// Date range filter
    pub date_range: Option<DateRange>,
    
    /// File size range filter
    pub size_range: Option<SizeRange>,
    
    /// Semantic similarity search
    pub semantic_query: Option<String>,
    
    /// Maximum number of results
    pub limit: Option<usize>,
    
    /// Search result offset (for pagination)
    pub offset: Option<usize>,
    
    /// Sort criteria
    pub sort: Option<SortCriteria>,
}

/// Date range for filtering search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

/// File size range for filtering (in bytes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeRange {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

/// Sort criteria for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortCriteria {
    /// Sort by relevance score (default)
    Relevance,
    
    /// Sort by creation date
    CreatedDate { ascending: bool },
    
    /// Sort by modification date
    ModifiedDate { ascending: bool },
    
    /// Sort by file size
    FileSize { ascending: bool },
    
    /// Sort by filename
    Filename { ascending: bool },
    
    /// Sort by asset type
    AssetType { ascending: bool },
}

/// Search results container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Matching assets
    pub assets: Vec<AssetMatch>,
    
    /// Total number of matches (before pagination)
    pub total_count: usize,
    
    /// Time taken to execute search (in milliseconds)
    pub search_time_ms: u64,
    
    /// Search query that produced these results
    pub query: SearchQuery,
    
    /// Faceted search results (aggregations)
    pub facets: SearchFacets,
}

/// A single asset match with relevance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMatch {
    /// The matched asset
    pub asset: Asset,
    
    /// Relevance score (0.0 - 1.0)
    pub score: f32,
    
    /// Which fields matched the search
    pub matched_fields: Vec<MatchedField>,
    
    /// Highlighted snippets from matched content
    pub highlights: Vec<String>,
}

/// Information about which field matched the search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedField {
    /// Name of the field that matched
    pub field: String,
    
    /// Match score for this field
    pub score: f32,
    
    /// Matched text snippet
    pub snippet: Option<String>,
}

/// Faceted search aggregations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacets {
    /// Asset type counts
    pub asset_types: HashMap<AssetType, usize>,
    
    /// File extension counts
    pub extensions: HashMap<String, usize>,
    
    /// Tag counts
    pub tags: HashMap<String, usize>,
    
    /// Size distribution
    pub size_distribution: SizeDistribution,
    
    /// Date distribution
    pub date_distribution: DateDistribution,
}

/// File size distribution buckets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeDistribution {
    pub small: usize,    // < 1MB
    pub medium: usize,   // 1MB - 100MB
    pub large: usize,    // 100MB - 1GB
    pub xlarge: usize,   // > 1GB
}

/// Creation date distribution buckets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateDistribution {
    pub last_day: usize,
    pub last_week: usize,
    pub last_month: usize,
    pub last_year: usize,
    pub older: usize,
}

/// Indexing operations for search engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexOperation {
    /// Add or update an asset in the index
    Upsert { asset: Asset },
    
    /// Remove an asset from the index
    Delete { asset_id: Uuid },
    
    /// Batch operation containing multiple operations
    Batch { operations: Vec<IndexOperation> },
    
    /// Rebuild the entire index
    Rebuild,
    
    /// Optimize the index for better performance
    Optimize,
}

/// Indexing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResult {
    /// Whether the operation succeeded
    pub success: bool,
    
    /// Number of documents affected
    pub documents_affected: usize,
    
    /// Time taken for the operation (in milliseconds)
    pub operation_time_ms: u64,
    
    /// Error message if operation failed
    pub error: Option<String>,
}

/// Vector embedding for semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingVector {
    /// Asset ID this embedding belongs to
    pub asset_id: Uuid,
    
    /// The embedding vector
    pub vector: Vec<f32>,
    
    /// Dimension of the vector
    pub dimension: usize,
    
    /// Model used to generate this embedding
    pub model: String,
    
    /// When this embedding was generated
    pub generated_at: DateTime<Utc>,
}

/// Similarity search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResult {
    /// Similar assets found
    pub similar_assets: Vec<SimilarAsset>,
    
    /// Query vector used for search
    pub query_vector: Vec<f32>,
    
    /// Search parameters used
    pub search_params: SimilaritySearchParams,
}

/// A similar asset with distance/similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarAsset {
    /// The similar asset
    pub asset: Asset,
    
    /// Similarity score (higher = more similar)
    pub similarity: f32,
    
    /// Distance metric used
    pub distance: f32,
}

/// Parameters for similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilaritySearchParams {
    /// Number of similar assets to return
    pub limit: usize,
    
    /// Minimum similarity threshold
    pub min_similarity: f32,
    
    /// Distance metric to use
    pub distance_metric: DistanceMetric,
}

/// Distance metrics for vector similarity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistanceMetric {
    /// Cosine similarity
    Cosine,
    
    /// Euclidean distance
    Euclidean,
    
    /// Dot product
    DotProduct,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: None,
            asset_type: None,
            tags: Vec::new(),
            extensions: Vec::new(),
            date_range: None,
            size_range: None,
            semantic_query: None,
            limit: Some(50),
            offset: Some(0),
            sort: Some(SortCriteria::Relevance),
        }
    }
}

impl SearchQuery {
    /// Create a simple text search query
    pub fn text_search(query: &str) -> Self {
        Self {
            text: Some(query.to_string()),
            ..Default::default()
        }
    }
    
    /// Create a semantic similarity search query
    pub fn semantic_search(query: &str) -> Self {
        Self {
            semantic_query: Some(query.to_string()),
            ..Default::default()
        }
    }
    
    /// Filter by asset type
    pub fn with_asset_type(mut self, asset_type: AssetType) -> Self {
        self.asset_type = Some(asset_type);
        self
    }
    
    /// Add tag filter
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
    
    /// Set result limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl Default for SimilaritySearchParams {
    fn default() -> Self {
        Self {
            limit: 10,
            min_similarity: 0.5,
            distance_metric: DistanceMetric::Cosine,
        }
    }
}
