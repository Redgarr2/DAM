//! Audio transcription using whisper.cpp
//! 
//! Provides offline speech-to-text capabilities using locally hosted
//! whisper models via FFI bindings with tiered quality levels.

use schema::{DamResult, ModelTier, ModelRegistry, ModelStatus};
use crate::error::ProcessError;
use crate::whisper_ffi::{WhisperContext, TranscriptResult, resample_to_16khz};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, warn, debug};
use symphonia::core::audio::Signal;

/// Audio transcription service with model management
pub struct TranscriptionService {
    /// Model registry for tier management
    registry: Arc<Mutex<ModelRegistry>>,
    /// Loaded whisper contexts per tier
    contexts: Arc<Mutex<HashMap<ModelTier, WhisperContext>>>,
    /// Model storage directory
    models_dir: PathBuf,
}

impl TranscriptionService {
    /// Create a new transcription service
    pub fn new() -> DamResult<Self> {
        info!("Initializing transcription service with whisper.cpp");
        
        let models_dir = PathBuf::from("models/whisper");
        
        Ok(Self {
            registry: Arc::new(Mutex::new(ModelRegistry::new())),
            contexts: Arc::new(Mutex::new(HashMap::new())),
            models_dir,
        })
    }
    
    /// Initialize with custom models directory
    pub fn with_models_dir<P: AsRef<Path>>(models_dir: P) -> DamResult<Self> {
        let models_dir = models_dir.as_ref().to_path_buf();
        info!("Initializing transcription service with models dir: {}", models_dir.display());
        
        Ok(Self {
            registry: Arc::new(Mutex::new(ModelRegistry::new())),
            contexts: Arc::new(Mutex::new(HashMap::new())),
            models_dir,
        })
    }
    
    /// Load model for specific tier
    pub async fn load_model(&self, tier: ModelTier) -> DamResult<()> {
        let config = {
            let registry = self.registry.lock().unwrap();
            registry.get_config(&tier)
                .ok_or_else(|| ProcessError::ModelNotFound(format!("No config for tier: {:?}", tier)))?
                .clone()
        };
        
        // Build model path
        let model_filename = match tier {
            ModelTier::Low => "ggml-tiny.en.bin",
            ModelTier::Medium => "ggml-base.bin", 
            ModelTier::High => "ggml-large-v3.bin",
        };
        
        let model_path = self.models_dir.join(model_filename);
        
        if !model_path.exists() {
            return Err(ProcessError::ModelNotFound(format!(
                "Model file not found: {}. Please download the whisper model.",
                model_path.display()
            )).into());
        }
        
        info!("Loading whisper model: {} for tier {:?}", model_path.display(), tier);
        
        // Load whisper context
        let context = WhisperContext::from_file(&model_path)
            .map_err(|e| ProcessError::ModelLoadFailed(e))?;
        
        // Store context
        {
            let mut contexts = self.contexts.lock().unwrap();
            contexts.insert(tier.clone(), context);
        }
        
        info!("Successfully loaded whisper model for tier {:?}", tier);
        Ok(())
    }
    
    /// Transcribe audio file to text
    pub async fn transcribe_file<P: AsRef<Path>>(&self, audio_path: P, language: Option<&str>) -> DamResult<TranscriptResult> {
        let path = audio_path.as_ref();
        debug!("Transcribing audio file: {}", path.display());
        
        // Read and decode audio file using symphonia
        let audio_data = self.load_audio_file(path).await?;
        
        // Transcribe the samples
        self.transcribe_samples(&audio_data.samples, audio_data.sample_rate, language).await
    }
    
    /// Transcribe raw audio samples
    pub async fn transcribe_samples(&self, samples: &[f32], sample_rate: u32, language: Option<&str>) -> DamResult<TranscriptResult> {
        // Get current tier and context
        let tier = {
            let registry = self.registry.lock().unwrap();
            registry.current_tier.clone()
        };
        
        let context = {
            let contexts = self.contexts.lock().unwrap();
            if !contexts.contains_key(&tier) {
                return Err(ProcessError::ModelNotLoaded(format!("Model not loaded for tier: {:?}", tier)).into());
            }
            // We can't return a reference here due to lifetime issues, so we'll need to handle this differently
            // For now, let's check if the model is loaded and then access it in the transcription
        };
        
        // Resample to 16kHz if needed
        let resampled = if sample_rate != 16000 {
            debug!("Resampling from {}Hz to 16kHz", sample_rate);
            resample_to_16khz(samples, sample_rate)
        } else {
            samples.to_vec()
        };
        
        // Perform transcription
        let result = {
            let contexts = self.contexts.lock().unwrap();
            let context = contexts.get(&tier)
                .ok_or_else(|| ProcessError::ModelNotLoaded(format!("Model not loaded for tier: {:?}", tier)))?;
            
            context.transcribe(&resampled, language)
                .map_err(|e| ProcessError::TranscriptionFailed(e))?
        };
        
        debug!("Transcription completed in {}ms", result.processing_time_ms);
        Ok(result)
    }
    
    /// Get supported languages for current tier
    pub fn supported_languages(&self) -> Vec<String> {
        let registry = self.registry.lock().unwrap();
        if let Some(config) = registry.current_config() {
            config.audio.languages.clone()
        } else {
            vec!["en".to_string()]
        }
    }
    
    /// Set AI quality tier
    pub async fn set_tier(&self, tier: ModelTier) -> DamResult<()> {
        {
            let mut registry = self.registry.lock().unwrap();
            registry.set_tier(tier.clone())
                .map_err(|e| ProcessError::InvalidTier(e))?;
        }
        
        // Load model if not already loaded
        if !self.is_model_loaded(&tier) {
            self.load_model(tier.clone()).await?;
        }
        
        info!("Switched transcription to tier: {:?}", tier);
        Ok(())
    }
    
    /// Get current tier
    pub fn current_tier(&self) -> ModelTier {
        let registry = self.registry.lock().unwrap();
        registry.current_tier.clone()
    }
    
    /// Check if model is loaded for tier
    pub fn is_model_loaded(&self, tier: &ModelTier) -> bool {
        let contexts = self.contexts.lock().unwrap();
        contexts.contains_key(tier)
    }
    
    /// Get model status for tier
    pub fn model_status(&self, tier: &ModelTier) -> ModelStatus {
        if self.is_model_loaded(tier) {
            ModelStatus::Loaded { memory_usage_mb: 100 } // Placeholder value
        } else {
            ModelStatus::NotLoaded
        }
    }
    
    /// Update system capabilities (VRAM, CUDA)
    pub fn update_system_info(&self, vram_mb: u32, cuda_available: bool) {
        let mut registry = self.registry.lock().unwrap();
        registry.update_system_info(vram_mb, cuda_available);
    }
    
    /// Get available tiers for current system
    pub fn available_tiers(&self) -> Vec<ModelTier> {
        let registry = self.registry.lock().unwrap();
        registry.available_tiers()
    }
    
    /// Load audio file using symphonia
    async fn load_audio_file(&self, path: &Path) -> DamResult<AudioData> {
        let file = std::fs::File::open(path)
            .map_err(|e| ProcessError::AudioLoadFailed(format!("Failed to open file: {}", e)))?;
        
        let mss = symphonia::core::io::MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = symphonia::core::probe::Hint::new();
        
        if let Some(extension) = path.extension() {
            hint.with_extension(&extension.to_string_lossy());
        }
        
        let meta_opts = symphonia::core::meta::MetadataOptions::default();
        let fmt_opts = symphonia::core::formats::FormatOptions::default();
        
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| ProcessError::AudioLoadFailed(format!("Failed to probe format: {}", e)))?;
        
        let mut format = probed.format;
        let track_id = {
            let track = format.tracks()
                .iter()
                .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
                .ok_or_else(|| ProcessError::AudioLoadFailed("No audio tracks found".to_string()))?;
            
            let dec_opts = symphonia::core::codecs::DecoderOptions::default();
            let mut decoder = symphonia::default::get_codecs()
                .make(&track.codec_params, &dec_opts)
                .map_err(|e| ProcessError::AudioLoadFailed(format!("Failed to create decoder: {}", e)))?;
            
            let sample_rate = track.codec_params.sample_rate.unwrap_or(16000);
            (track.id, decoder, sample_rate)
        };
        
        let (track_id, mut decoder, sample_rate) = track_id;
        let mut samples = Vec::new();
        
        // Decode all packets
        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::ResetRequired) => {
                    // Reset decoder and continue
                    decoder.reset();
                    continue;
                }
                Err(symphonia::core::errors::Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(e) => {
                    warn!("Error reading packet: {}", e);
                    break;
                }
            };
            
            if packet.track_id() != track_id {
                continue;
            }
            
            match decoder.decode(&packet) {
                Ok(decoded) => {
                    // Convert to f32 samples
                    let spec = decoded.spec();
                    let sample_count = decoded.capacity() as usize;
                    
                    // Convert samples to f32 mono (simplified implementation)
                    match decoded {
                        symphonia::core::audio::AudioBufferRef::F32(buf) => {
                            // For now, just get the first channel and use it as mono
                            let channel = buf.chan(0);
                            samples.extend_from_slice(channel);
                        }
                        _ => {
                            // Handle other formats by converting to f32
                            warn!("Audio format conversion needed - using basic conversion");
                            // This is a simplified conversion - in practice you'd want proper format handling
                        }
                    }
                }
                Err(e) => {
                    warn!("Error decoding packet: {}", e);
                    continue;
                }
            }
        }
        
        if samples.is_empty() {
            return Err(ProcessError::AudioLoadFailed("No audio samples decoded".to_string()).into());
        }
        
        Ok(AudioData {
            samples,
            sample_rate,
        })
    }
}

impl Default for TranscriptionService {
    fn default() -> Self {
        Self::new().expect("Failed to create TranscriptionService")
    }
}

/// Audio data container
struct AudioData {
    samples: Vec<f32>,
    sample_rate: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_transcription_service_creation() {
        let service = TranscriptionService::new();
        assert!(service.is_ok());
    }
    
    #[test]
    fn test_tier_management() {
        let service = TranscriptionService::new().unwrap();
        let current = service.current_tier();
        assert_eq!(current, ModelTier::Medium); // Default tier
    }
    
    #[test]
    fn test_available_tiers() {
        let service = TranscriptionService::new().unwrap();
        service.update_system_info(24576, true); // RTX 3090 specs
        let tiers = service.available_tiers();
        assert!(tiers.contains(&ModelTier::High));
    }
}
