//! Asset ingestion and file parsing
//! 
//! This crate handles importing digital assets into the DAM system, including:
//! - File format detection and validation
//! - Metadata extraction from various file types
//! - Preview/thumbnail generation
//! - File system monitoring for automatic import

pub mod detector;
pub mod parser;
pub mod preview;
pub mod monitor;
pub mod error;

use schema::{Asset, AssetType, DamResult};
use std::path::Path;
use tokio::fs;
use tracing::{info, warn, error};
use uuid::Uuid;
use chrono::Utc;

pub use detector::*;
pub use parser::AssetParser;
pub use preview::*;
pub use monitor::*;
pub use error::*;

/// Main ingestion service
pub struct IngestService {
    detector: FormatDetector,
    parser: AssetParser,
    preview_generator: PreviewGenerator,
}

impl IngestService {
    /// Create a new ingestion service
    pub fn new() -> DamResult<Self> {
        Ok(Self {
            detector: FormatDetector::new()?,
            parser: AssetParser::new()?,
            preview_generator: PreviewGenerator::new()?,
        })
    }
    
    /// Ingest a single file
    pub async fn ingest_file<P: AsRef<Path>>(&self, path: P) -> DamResult<Asset> {
        let path = path.as_ref();
        info!("Ingesting file: {}", path.display());
        
        // Check if file exists and is readable
        if !path.exists() {
            return Err(IngestError::FileNotFound {
                path: path.to_path_buf(),
            }.into());
        }
        
        if !path.is_file() {
            return Err(IngestError::NotAFile {
                path: path.to_path_buf(),
            }.into());
        }
        
        // Get file metadata
        let metadata = fs::metadata(path).await?;
        let file_size = metadata.len();
        let modified = metadata.modified()?;
        
        // Detect file format
        let format_info = self.detector.detect_format(path).await?;
        info!("Detected format: {} for {}", format_info.extension, path.display());
        
        if !format_info.supported {
            warn!("Unsupported format {} for file {}", format_info.extension, path.display());
        }
        
        // Determine asset type
        let asset_type = AssetType::from_extension(&format_info.extension);
        
        // Create base asset
        let mut asset = Asset::new(path.to_path_buf(), asset_type);
        asset.file_size = file_size;
        asset.format = format_info;
        asset.modified_at = modified.into();
        
        // Parse file-specific metadata
        match self.parser.parse_metadata(&asset).await {
            Ok(metadata) => {
                asset.metadata = metadata;
                info!("Extracted metadata for {}", path.display());
            }
            Err(e) => {
                warn!("Failed to extract metadata for {}: {}", path.display(), e);
            }
        }
        
        // Generate preview/thumbnail
        match self.preview_generator.generate_preview(&asset).await {
            Ok(preview_info) => {
                asset.preview = Some(preview_info);
                info!("Generated preview for {}", path.display());
            }
            Err(e) => {
                warn!("Failed to generate preview for {}: {}", path.display(), e);
            }
        }
        
        info!("Successfully ingested: {}", path.display());
        Ok(asset)
    }
    
    /// Ingest multiple files in parallel
    pub async fn ingest_batch<P: AsRef<Path>>(&self, paths: Vec<P>) -> Vec<DamResult<Asset>> {
        info!("Ingesting batch of {} files", paths.len());
        
        let tasks = paths.into_iter().map(|path| {
            let service = self;
            async move {
                service.ingest_file(path).await
            }
        });
        
        futures::future::join_all(tasks).await
    }
    
    /// Ingest all files in a directory recursively
    pub async fn ingest_directory<P: AsRef<Path>>(&self, dir_path: P) -> DamResult<Vec<Asset>> {
        let dir_path = dir_path.as_ref();
        info!("Ingesting directory: {}", dir_path.display());
        
        if !dir_path.exists() {
            return Err(IngestError::FileNotFound {
                path: dir_path.to_path_buf(),
            }.into());
        }
        
        if !dir_path.is_dir() {
            return Err(IngestError::NotADirectory {
                path: dir_path.to_path_buf(),
            }.into());
        }
        
        // Collect all files recursively
        let mut file_paths = Vec::new();
        let mut walker = walkdir::WalkDir::new(dir_path).into_iter();
        
        while let Some(entry) = walker.next() {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        file_paths.push(entry.path().to_path_buf());
                    }
                }
                Err(e) => {
                    warn!("Error walking directory: {}", e);
                }
            }
        }
        
        info!("Found {} files in directory", file_paths.len());
        
        // Process files in batches to avoid overwhelming the system
        const BATCH_SIZE: usize = 10;
        let mut all_assets = Vec::new();
        
        for chunk in file_paths.chunks(BATCH_SIZE) {
            let results = self.ingest_batch(chunk.to_vec()).await;
            
            for result in results {
                match result {
                    Ok(asset) => all_assets.push(asset),
                    Err(e) => error!("Failed to ingest file: {}", e),
                }
            }
        }
        
        info!("Successfully ingested {} assets from directory", all_assets.len());
        Ok(all_assets)
    }
    
    /// Check if a file should be ingested (based on extension and other criteria)
    pub fn should_ingest<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        
        // Skip hidden files and directories
        if let Some(filename) = path.file_name() {
            if filename.to_string_lossy().starts_with('.') {
                return false;
            }
        }
        
        // Skip common non-asset files
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            match ext.as_str() {
                "tmp" | "temp" | "log" | "bak" | "cache" => return false,
                _ => {}
            }
        }
        
        // Check if we support this format
        if let Ok(format_info) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.detector.detect_format(path).await
            })
        }) {
            format_info.supported
        } else {
            false
        }
    }
}

impl Default for IngestService {
    fn default() -> Self {
        Self::new().expect("Failed to create IngestService")
    }
}

/// Utility function to compute file hash for deduplication
pub async fn compute_file_hash<P: AsRef<Path>>(path: P) -> DamResult<String> {
    use sha2::{Sha256, Digest};
    use tokio::io::AsyncReadExt;
    
    let mut file = fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    
    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

/// Check if a path represents a supported asset type
pub fn is_supported_asset<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        !matches!(AssetType::from_extension(&ext), AssetType::Unknown)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    
    #[tokio::test]
    async fn test_ingest_service_creation() {
        let service = IngestService::new();
        assert!(service.is_ok());
    }
    
    #[tokio::test]
    async fn test_compute_file_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        
        let mut file = File::create(&file_path).await.unwrap();
        file.write_all(b"Hello, world!").await.unwrap();
        file.flush().await.unwrap();
        
        let hash = compute_file_hash(&file_path).await.unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex characters
    }
    
    #[tokio::test]
    async fn test_is_supported_asset() {
        assert!(is_supported_asset("test.png"));
        assert!(is_supported_asset("model.blend"));
        assert!(is_supported_asset("audio.wav"));
        assert!(!is_supported_asset("document.xyz"));
        assert!(!is_supported_asset("file_without_extension"));
    }
}
