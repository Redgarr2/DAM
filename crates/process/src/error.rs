//! Processing-specific error types

use schema::DamError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),
    
    #[error("Tagging failed: {0}")]
    TaggingFailed(String),
    
    #[error("Generation failed: {0}")]
    GenerationFailed(String),
    
    #[error("Embedding failed: {0}")]
    EmbeddingFailed(String),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Model load failed: {0}")]
    ModelLoadFailed(String),
    
    #[error("Audio load failed: {0}")]
    AudioLoadFailed(String),
    
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),
    
    #[error("Invalid tier: {0}")]
    InvalidTier(String),
    
    #[error("Image load failed: {0}")]
    ImageLoadFailed(String),
    
    #[error("Image processing failed: {0}")]
    ImageProcessingFailed(String),
    
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
}

impl From<ProcessError> for DamError {
    fn from(err: ProcessError) -> Self {
        DamError::processing(err.to_string())
    }
}
