//! AI model configuration and tiering system
//! 
//! Defines the different quality tiers and model configurations
//! for the AI processing pipeline.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI processing quality tiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelTier {
    /// Low quality, fast processing (2-4GB VRAM)
    Low,
    /// Medium quality, balanced (8-12GB VRAM) 
    Medium,
    /// High quality, maximum performance (16+ GB VRAM)
    High,
}

impl ModelTier {
    /// Get human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Low => "Low (Fast)",
            Self::Medium => "Medium (Balanced)",
            Self::High => "High (Maximum Quality)",
        }
    }
    
    /// Get minimum VRAM requirement in MB
    pub fn min_vram_mb(&self) -> u32 {
        match self {
            Self::Low => 2048,
            Self::Medium => 8192,
            Self::High => 16384,
        }
    }
    
    /// Get recommended VRAM in MB
    pub fn recommended_vram_mb(&self) -> u32 {
        match self {
            Self::Low => 4096,
            Self::Medium => 12288,
            Self::High => 24576,
        }
    }
}

/// Configuration for audio transcription models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioModelConfig {
    pub model_name: String,
    pub model_size_mb: u32,
    pub languages: Vec<String>,
    pub speed_multiplier: f32, // How fast compared to real-time
    pub quality_score: u8, // 1-10
}

/// Configuration for image analysis models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionModelConfig {
    pub clip_model: String,
    pub blip_model: Option<String>,
    pub model_size_mb: u32,
    pub max_image_size: u32,
    pub tags_per_image: u32,
    pub quality_score: u8,
}

/// Configuration for image generation models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationModelConfig {
    pub base_model: String,
    pub controlnet_models: Vec<String>,
    pub refiner_model: Option<String>,
    pub model_size_mb: u32,
    pub max_resolution: (u32, u32),
    pub steps_per_image: u32,
    pub quality_score: u8,
}

/// Configuration for embedding models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModelConfig {
    pub model_name: String,
    pub model_size_mb: u32,
    pub embedding_dim: u32,
    pub max_text_length: u32,
    pub quality_score: u8,
}

/// Complete model configuration for a tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierModelConfig {
    pub tier: ModelTier,
    pub audio: AudioModelConfig,
    pub vision: VisionModelConfig,
    pub generation: GenerationModelConfig,
    pub embedding: EmbeddingModelConfig,
    pub total_size_mb: u32,
    pub cuda_required: bool,
}

/// AI model registry with all tier configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    pub tiers: HashMap<ModelTier, TierModelConfig>,
    pub current_tier: ModelTier,
    pub available_vram_mb: u32,
    pub cuda_available: bool,
}

impl ModelRegistry {
    /// Create a new model registry with default configurations
    pub fn new() -> Self {
        let mut tiers = HashMap::new();
        
        // Low tier configuration
        tiers.insert(ModelTier::Low, TierModelConfig {
            tier: ModelTier::Low,
            audio: AudioModelConfig {
                model_name: "whisper-tiny.en".to_string(),
                model_size_mb: 39,
                languages: vec!["en".to_string()],
                speed_multiplier: 4.0,
                quality_score: 6,
            },
            vision: VisionModelConfig {
                clip_model: "clip-vit-b-32".to_string(),
                blip_model: None,
                model_size_mb: 150,
                max_image_size: 224,
                tags_per_image: 5,
                quality_score: 6,
            },
            generation: GenerationModelConfig {
                base_model: "sd-1.5-lcm".to_string(),
                controlnet_models: vec![],
                refiner_model: None,
                model_size_mb: 1700,
                max_resolution: (512, 512),
                steps_per_image: 4,
                quality_score: 6,
            },
            embedding: EmbeddingModelConfig {
                model_name: "all-minilm-l6-v2".to_string(),
                model_size_mb: 23,
                embedding_dim: 384,
                max_text_length: 256,
                quality_score: 7,
            },
            total_size_mb: 800,
            cuda_required: false,
        });
        
        // Medium tier configuration
        tiers.insert(ModelTier::Medium, TierModelConfig {
            tier: ModelTier::Medium,
            audio: AudioModelConfig {
                model_name: "whisper-base".to_string(),
                model_size_mb: 244,
                languages: vec!["en".to_string(), "es".to_string(), "fr".to_string(), "de".to_string()],
                speed_multiplier: 2.0,
                quality_score: 8,
            },
            vision: VisionModelConfig {
                clip_model: "clip-vit-l-14".to_string(),
                blip_model: Some("blip-base".to_string()),
                model_size_mb: 1340,
                max_image_size: 336,
                tags_per_image: 10,
                quality_score: 8,
            },
            generation: GenerationModelConfig {
                base_model: "sd-1.5".to_string(),
                controlnet_models: vec!["controlnet-canny".to_string(), "controlnet-depth".to_string()],
                refiner_model: None,
                model_size_mb: 3500,
                max_resolution: (768, 768),
                steps_per_image: 20,
                quality_score: 8,
            },
            embedding: EmbeddingModelConfig {
                model_name: "all-mpnet-base-v2".to_string(),
                model_size_mb: 420,
                embedding_dim: 768,
                max_text_length: 384,
                quality_score: 9,
            },
            total_size_mb: 4000,
            cuda_required: true,
        });
        
        // High tier configuration  
        tiers.insert(ModelTier::High, TierModelConfig {
            tier: ModelTier::High,
            audio: AudioModelConfig {
                model_name: "whisper-large-v3".to_string(),
                model_size_mb: 1550,
                languages: vec!["multilingual".to_string()],
                speed_multiplier: 1.2,
                quality_score: 10,
            },
            vision: VisionModelConfig {
                clip_model: "openclip-vit-h-14".to_string(),
                blip_model: Some("blip2-flan-t5-xl".to_string()),
                model_size_mb: 6500,
                max_image_size: 518,
                tags_per_image: 20,
                quality_score: 10,
            },
            generation: GenerationModelConfig {
                base_model: "sdxl-base".to_string(),
                controlnet_models: vec![
                    "controlnet-xl-canny".to_string(),
                    "controlnet-xl-depth".to_string(),
                    "controlnet-xl-openpose".to_string(),
                ],
                refiner_model: Some("sdxl-refiner".to_string()),
                model_size_mb: 18000,
                max_resolution: (1024, 1024),
                steps_per_image: 40,
                quality_score: 10,
            },
            embedding: EmbeddingModelConfig {
                model_name: "e5-large-v2".to_string(),
                model_size_mb: 1200,
                embedding_dim: 1024,
                max_text_length: 512,
                quality_score: 10,
            },
            total_size_mb: 30000,
            cuda_required: true,
        });
        
        Self {
            tiers,
            current_tier: ModelTier::Medium, // Default to medium
            available_vram_mb: 0,
            cuda_available: false,
        }
    }
    
    /// Get configuration for current tier
    pub fn current_config(&self) -> Option<&TierModelConfig> {
        self.tiers.get(&self.current_tier)
    }
    
    /// Get configuration for specific tier
    pub fn get_config(&self, tier: &ModelTier) -> Option<&TierModelConfig> {
        self.tiers.get(tier)
    }
    
    /// Set current tier (validates VRAM requirements)
    pub fn set_tier(&mut self, tier: ModelTier) -> Result<(), String> {
        if let Some(config) = self.tiers.get(&tier) {
            if self.available_vram_mb < config.tier.min_vram_mb() {
                return Err(format!(
                    "Insufficient VRAM: {} MB available, {} MB required",
                    self.available_vram_mb,
                    config.tier.min_vram_mb()
                ));
            }
            
            if config.cuda_required && !self.cuda_available {
                return Err("CUDA required but not available".to_string());
            }
            
            self.current_tier = tier;
            Ok(())
        } else {
            Err("Invalid tier".to_string())
        }
    }
    
    /// Update system capabilities
    pub fn update_system_info(&mut self, vram_mb: u32, cuda_available: bool) {
        self.available_vram_mb = vram_mb;
        self.cuda_available = cuda_available;
    }
    
    /// Get recommended tier for current system
    pub fn recommended_tier(&self) -> ModelTier {
        if self.available_vram_mb >= ModelTier::High.recommended_vram_mb() && self.cuda_available {
            ModelTier::High
        } else if self.available_vram_mb >= ModelTier::Medium.recommended_vram_mb() && self.cuda_available {
            ModelTier::Medium
        } else {
            ModelTier::Low
        }
    }
    
    /// Get all available tiers for current system
    pub fn available_tiers(&self) -> Vec<ModelTier> {
        let mut available = Vec::new();
        
        for (tier, config) in &self.tiers {
            if self.available_vram_mb >= config.tier.min_vram_mb() && 
               (!config.cuda_required || self.cuda_available) {
                available.push(tier.clone());
            }
        }
        
        available.sort_by_key(|t| match t {
            ModelTier::Low => 0,
            ModelTier::Medium => 1,
            ModelTier::High => 2,
        });
        
        available
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Model loading status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStatus {
    /// Model not loaded
    NotLoaded,
    /// Currently loading
    Loading { progress: f32 },
    /// Successfully loaded
    Loaded { memory_usage_mb: u32 },
    /// Failed to load
    Failed { error: String },
}

/// Runtime model manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManager {
    pub registry: ModelRegistry,
    pub audio_status: ModelStatus,
    pub vision_status: ModelStatus,
    pub generation_status: ModelStatus,
    pub embedding_status: ModelStatus,
    pub total_vram_used_mb: u32,
}

impl ModelManager {
    /// Create new model manager
    pub fn new() -> Self {
        Self {
            registry: ModelRegistry::new(),
            audio_status: ModelStatus::NotLoaded,
            vision_status: ModelStatus::NotLoaded,
            generation_status: ModelStatus::NotLoaded,
            embedding_status: ModelStatus::NotLoaded,
            total_vram_used_mb: 0,
        }
    }
    
    /// Check if all models for current tier are loaded
    pub fn all_models_loaded(&self) -> bool {
        matches!(
            (&self.audio_status, &self.vision_status, &self.generation_status, &self.embedding_status),
            (ModelStatus::Loaded { .. }, ModelStatus::Loaded { .. }, ModelStatus::Loaded { .. }, ModelStatus::Loaded { .. })
        )
    }
    
    /// Get total memory usage across all loaded models
    pub fn total_memory_usage(&self) -> u32 {
        self.total_vram_used_mb
    }
}

impl Default for ModelManager {
    fn default() -> Self {
        Self::new()
    }
}
