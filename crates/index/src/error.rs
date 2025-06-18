//! Index-specific error types

use schema::DamError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Index not found: {0}")]
    IndexNotFound(String),
    
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    
    #[error("Search failed: {0}")]
    SearchFailed(String),
    
    #[error("Vector operation failed: {0}")]
    VectorError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Index corrupted: {0}")]
    CorruptedIndex(String),
}

impl From<IndexError> for DamError {
    fn from(err: IndexError) -> Self {
        DamError::search(err.to_string())
    }
}

impl From<sled::Error> for IndexError {
    fn from(err: sled::Error) -> Self {
        IndexError::DatabaseError(err.to_string())
    }
}

impl From<serde_json::Error> for IndexError {
    fn from(err: serde_json::Error) -> Self {
        IndexError::SerializationError(err.to_string())
    }
}
