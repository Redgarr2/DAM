//! AI processing and machine learning
//! 
//! This crate handles all AI-powered processing including:
//! - Audio transcription via whisper.cpp
//! - Image tagging via CLIP/BLIP
//! - Generative image editing via Stable Diffusion
//! - Vector embedding generation for semantic search

pub mod transcription;
pub mod tagging;
pub mod generation;
pub mod embedding;
pub mod error;
pub mod whisper_ffi;

use schema::DamResult;
use std::path::Path;
use tracing::info;

pub use transcription::*;
pub use tagging::*;
pub use generation::*;
pub use embedding::*;
pub use error::*;

/// Main AI processing service
pub struct ProcessingService {
    transcription: TranscriptionService,
    tagging: TaggingService,
    generation: GenerationService,
    embedding: EmbeddingService,
}

impl ProcessingService {
    /// Create a new processing service
    pub fn new() -> DamResult<Self> {
        info!("Initializing AI processing service");
        
        Ok(Self {
            transcription: TranscriptionService::new()?,
            tagging: TaggingService::new()?,
            generation: GenerationService::new()?,
            embedding: EmbeddingService::new()?,
        })
    }
    
    /// Get reference to transcription service
    pub fn transcription(&self) -> &TranscriptionService {
        &self.transcription
    }
    
    /// Get reference to tagging service
    pub fn tagging(&self) -> &TaggingService {
        &self.tagging
    }
    
    /// Get reference to generation service
    pub fn generation(&self) -> &GenerationService {
        &self.generation
    }
    
    /// Get reference to embedding service
    pub fn embedding(&self) -> &EmbeddingService {
        &self.embedding
    }
}

impl Default for ProcessingService {
    fn default() -> Self {
        Self::new().expect("Failed to create ProcessingService")
    }
}
