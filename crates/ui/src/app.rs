//! Main application state and initialization

use crate::error::{UiError, UiResult};
use index::IndexService;
use ingest::IngestService;
// use process::{TranscriptionService, TaggingService};  // Temporarily disabled
use schema::{Asset, DamResult, ModelTier};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Main application state
pub struct DamApp {
    /// Search and indexing service
    pub index_service: IndexService,
    
    /// File ingestion service
    pub ingest_service: IngestService,
    
    /// AI transcription service (temporarily disabled)
    // pub transcription_service: TranscriptionService,
    
    /// AI image tagging service (temporarily disabled)
    // pub tagging_service: TaggingService,
    
    /// Application settings
    pub settings: AppSettings,
    
    /// Current library path
    pub library_path: Option<PathBuf>,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Default library location
    pub default_library_path: Option<PathBuf>,
    
    /// AI processing settings
    pub ai_enabled: bool,
    pub ai_tier: ModelTier,
    
    /// UI preferences
    pub theme: ThemeMode,
    pub preview_size: PreviewSize,
    pub auto_tag: bool,
    pub auto_transcribe: bool,
    
    /// Search preferences
    pub search_results_limit: usize,
    pub enable_similarity_search: bool,
    pub similarity_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewSize {
    Small,
    Medium,
    Large,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_library_path: None,
            ai_enabled: true,
            ai_tier: ModelTier::Medium,
            theme: ThemeMode::System,
            preview_size: PreviewSize::Medium,
            auto_tag: true,
            auto_transcribe: true,
            search_results_limit: 50,
            enable_similarity_search: true,
            similarity_threshold: 0.7,
        }
    }
}

impl DamApp {
    /// Initialize the application
    pub async fn new() -> UiResult<Self> {
        info!("Initializing DAM application");
        
        // Load or create settings
        let settings = Self::load_settings().unwrap_or_default();
        
        // Initialize services
        let index_service = IndexService::new()
            .map_err(|e| UiError::InitializationFailed(format!("Failed to initialize search service: {}", e)))?;
        
        let ingest_service = IngestService::new()
            .map_err(|e| UiError::InitializationFailed(format!("Failed to initialize ingest service: {}", e)))?;
        
        // Temporarily disabled until whisper.lib is compiled
        // let transcription_service = TranscriptionService::new()
        //     .map_err(|e| UiError::InitializationFailed(format!("Failed to initialize transcription service: {}", e)))?;
        
        // let tagging_service = TaggingService::new()
        //     .map_err(|e| UiError::InitializationFailed(format!("Failed to initialize tagging service: {}", e)))?;
        
        let app = Self {
            index_service,
            ingest_service,
            // transcription_service,
            // tagging_service,
            settings,
            library_path: None,
        };
        
        // Temporarily disabled AI tier setting
        // Set AI tier from settings
        // if app.settings.ai_enabled {
        //     if let Err(e) = app.transcription_service.set_tier(app.settings.ai_tier.clone()).await {
        //         warn!("Failed to set transcription tier: {}", e);
        //     }
        //     if let Err(e) = app.tagging_service.set_tier(app.settings.ai_tier.clone()).await {
        //         warn!("Failed to set tagging tier: {}", e);
        //     }
        // }
        
        // Load default library if specified
        // if let Some(ref library_path) = app.settings.default_library_path {
        //     if library_path.exists() {
        //         app.library_path = Some(library_path.clone());
        //         info!("Loaded default library: {}", library_path.display());
        //     }
        // }
        
        info!("DAM application initialized successfully");
        Ok(app)
    }
    
    /// Import a single file
    pub async fn import_file(&mut self, file_path: PathBuf) -> UiResult<Asset> {
        info!("Importing file: {}", file_path.display());
        
        // Ingest the file
        let asset = self.ingest_service.ingest_file(&file_path).await?;
        
        // Add to search index
        self.index_service.index_asset(&asset).await?;
        
        // AI processing temporarily disabled
        // Process with AI if enabled
        // if self.settings.ai_enabled {
        //     self.process_asset_with_ai(&mut asset).await?;
        // }
        
        info!("Successfully imported: {}", file_path.display());
        Ok(asset)
    }
    
    /// Import all files in a directory
    pub async fn import_directory(&mut self, dir_path: PathBuf) -> UiResult<Vec<Asset>> {
        info!("Importing directory: {}", dir_path.display());
        
        let assets = self.ingest_service.ingest_directory(&dir_path).await?;
        let mut imported_assets = Vec::new();
        
        for asset in assets {
            // Add to search index
            if let Err(e) = self.index_service.index_asset(&asset).await {
                error!("Failed to index asset {}: {}", asset.id, e);
                continue;
            }
            
            // AI processing temporarily disabled
            // Process with AI if enabled
            // if self.settings.ai_enabled {
            //     if let Err(e) = self.process_asset_with_ai(&mut asset).await {
            //         warn!("Failed to process asset {} with AI: {}", asset.id, e);
            //     }
            // }
            
            imported_assets.push(asset);
        }
        
        info!("Successfully imported {} assets from directory", imported_assets.len());
        Ok(imported_assets)
    }
    
    /// Process an asset with AI services (temporarily disabled)
    // async fn process_asset_with_ai(&mut self, asset: &mut Asset) -> UiResult<()> {
    //     // Implementation temporarily disabled
    //     Ok(())
    // }
    
    /// Search for assets
    pub async fn search_assets(&self, query: &str, limit: usize) -> UiResult<Vec<index::SearchResult>> {
        let results = self.index_service.search_text(query, limit).await?;
        Ok(results)
    }
    
    /// Find similar assets (temporarily disabled)
    pub async fn find_similar(&self, asset_id: Uuid, limit: usize) -> UiResult<Vec<index::SearchResult>> {
        // Temporarily return empty results
        Ok(vec![])
        // let results = self.index_service.find_similar(
        //     asset_id,
        //     index::EmbeddingType::Visual,
        //     limit
        // ).await?;
        // Ok(results)
    }
    
    /// Get library statistics
    pub fn get_library_stats(&self) -> LibraryStats {
        let index_stats = self.index_service.get_stats();
        
        LibraryStats {
            total_assets: index_stats.total_documents,
            total_size: 0, // TODO: Calculate from assets
            asset_types: vec![], // TODO: Aggregate by type
            ai_processed: index_stats.visual_embeddings + index_stats.text_embeddings,
        }
    }
    
    /// Update application settings
    pub async fn update_settings(&mut self, new_settings: AppSettings) -> UiResult<()> {
        // AI tier updates temporarily disabled
        // Update AI tier if changed
        // if new_settings.ai_tier != self.settings.ai_tier && new_settings.ai_enabled {
        //     self.transcription_service.set_tier(new_settings.ai_tier.clone()).await?;
        //     self.tagging_service.set_tier(new_settings.ai_tier.clone()).await?;
        // }
        
        // Save settings
        self.settings = new_settings;
        self.save_settings()?;
        
        info!("Settings updated successfully");
        Ok(())
    }
    
    /// Load settings from disk
    fn load_settings() -> Option<AppSettings> {
        let settings_path = Self::settings_path();
        if settings_path.exists() {
            match std::fs::read_to_string(&settings_path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(settings) => Some(settings),
                        Err(e) => {
                            warn!("Failed to parse settings file: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read settings file: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }
    
    /// Save settings to disk
    fn save_settings(&self) -> UiResult<()> {
        let settings_path = Self::settings_path();
        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(&self.settings)?;
        std::fs::write(&settings_path, content)?;
        
        Ok(())
    }
    
    /// Get the path for settings file
    fn settings_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("dam").join("settings.json")
        } else {
            PathBuf::from("settings.json")
        }
    }
}

/// Library statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStats {
    pub total_assets: usize,
    pub total_size: u64,
    pub asset_types: Vec<AssetTypeCount>,
    pub ai_processed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTypeCount {
    pub asset_type: schema::AssetType,
    pub count: usize,
}
