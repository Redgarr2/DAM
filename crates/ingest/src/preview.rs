//! Preview and thumbnail generation
//! 
//! This module generates previews and thumbnails for various asset types.

use schema::{Asset, AssetType, PreviewInfo, DamResult};
use std::path::{Path, PathBuf};
use chrono::Utc;
use tracing::{debug, warn, error};
use crate::error::IngestError;
use image::GenericImageView;

/// Service for generating asset previews
pub struct PreviewGenerator {
    /// Directory where previews are stored
    preview_dir: PathBuf,
    
    /// Maximum preview dimensions
    max_preview_size: (u32, u32),
    
    /// JPEG quality for generated previews (0-100)
    jpeg_quality: u8,
}

impl PreviewGenerator {
    /// Create a new preview generator
    pub fn new() -> DamResult<Self> {
        let preview_dir = std::env::current_dir()
            .unwrap_or_default()
            .join("previews");
        
        Ok(Self {
            preview_dir,
            max_preview_size: (512, 512),
            jpeg_quality: 85,
        })
    }
    
    /// Create a preview generator with custom settings
    pub fn with_settings<P: Into<PathBuf>>(
        preview_dir: P,
        max_size: (u32, u32),
        jpeg_quality: u8,
    ) -> DamResult<Self> {
        Ok(Self {
            preview_dir: preview_dir.into(),
            max_preview_size: max_size,
            jpeg_quality,
        })
    }
    
    /// Generate preview for an asset
    pub async fn generate_preview(&self, asset: &Asset) -> DamResult<PreviewInfo> {
        debug!("Generating preview for: {}", asset.current_path.display());
        
        // Ensure preview directory exists
        tokio::fs::create_dir_all(&self.preview_dir).await?;
        
        match asset.asset_type {
            AssetType::Image => self.generate_image_preview(asset).await,
            AssetType::ThreeD => self.generate_3d_preview(asset).await,
            AssetType::Audio => self.generate_audio_preview(asset).await,
            AssetType::Video => self.generate_video_preview(asset).await,
            _ => {
                // For unsupported types, generate a generic icon
                self.generate_generic_preview(asset).await
            }
        }
    }
    
    /// Generate preview for image assets
    async fn generate_image_preview(&self, asset: &Asset) -> DamResult<PreviewInfo> {
        let input_path = &asset.current_path;
        let preview_filename = format!("{}.jpg", asset.id);
        let preview_path = self.preview_dir.join(&preview_filename);
        
        // Load and resize the image
        let img = image::open(input_path)
            .map_err(|e| IngestError::preview_generation_failed(
                input_path.clone(),
                format!("Failed to open image: {}", e)
            ))?;
        
        let (width, height) = img.dimensions();
        let (thumb_width, thumb_height) = self.calculate_thumbnail_size(width, height);
        
        // Resize image maintaining aspect ratio
        let thumbnail = img.resize(thumb_width, thumb_height, image::imageops::FilterType::Lanczos3);
        
        // Save as JPEG
        thumbnail.save_with_format(&preview_path, image::ImageFormat::Jpeg)
            .map_err(|e| IngestError::preview_generation_failed(
                input_path.clone(),
                format!("Failed to save thumbnail: {}", e)
            ))?;
        
        Ok(PreviewInfo {
            thumbnail_path: preview_path,
            thumbnail_size: (thumb_width, thumb_height),
            rendered_preview: None,
            generated_at: Utc::now(),
        })
    }
    
    /// Generate preview for 3D assets
    async fn generate_3d_preview(&self, asset: &Asset) -> DamResult<PreviewInfo> {
        let input_path = &asset.current_path;
        let preview_filename = format!("{}.jpg", asset.id);
        let preview_path = self.preview_dir.join(&preview_filename);
        
        // For now, generate a placeholder 3D preview
        // In a full implementation, this would:
        // 1. Load the 3D model
        // 2. Render it from multiple angles
        // 3. Create a composite preview image
        
        warn!("3D preview generation not fully implemented, creating placeholder for: {}", 
              input_path.display());
        
        self.create_placeholder_preview(&preview_path, "3D", (128, 128, 200)).await?;
        
        Ok(PreviewInfo {
            thumbnail_path: preview_path.clone(),
            thumbnail_size: self.max_preview_size,
            rendered_preview: Some(preview_path),
            generated_at: Utc::now(),
        })
    }
    
    /// Generate preview for audio assets
    async fn generate_audio_preview(&self, asset: &Asset) -> DamResult<PreviewInfo> {
        let input_path = &asset.current_path;
        let preview_filename = format!("{}.jpg", asset.id);
        let preview_path = self.preview_dir.join(&preview_filename);
        
        // For audio files, we could generate a waveform visualization
        // For now, create a placeholder with audio icon
        
        debug!("Generating audio waveform preview for: {}", input_path.display());
        
        self.create_placeholder_preview(&preview_path, "♪", (100, 150, 255)).await?;
        
        Ok(PreviewInfo {
            thumbnail_path: preview_path,
            thumbnail_size: self.max_preview_size,
            rendered_preview: None,
            generated_at: Utc::now(),
        })
    }
    
    /// Generate preview for video assets
    async fn generate_video_preview(&self, asset: &Asset) -> DamResult<PreviewInfo> {
        let input_path = &asset.current_path;
        let preview_filename = format!("{}.jpg", asset.id);
        let preview_path = self.preview_dir.join(&preview_filename);
        
        // For video files, we would extract a frame from the middle of the video
        // For now, create a placeholder
        
        debug!("Generating video frame preview for: {}", input_path.display());
        
        self.create_placeholder_preview(&preview_path, "▶", (255, 100, 100)).await?;
        
        Ok(PreviewInfo {
            thumbnail_path: preview_path,
            thumbnail_size: self.max_preview_size,
            rendered_preview: None,
            generated_at: Utc::now(),
        })
    }
    
    /// Generate generic preview for unsupported asset types
    async fn generate_generic_preview(&self, asset: &Asset) -> DamResult<PreviewInfo> {
        let preview_filename = format!("{}.jpg", asset.id);
        let preview_path = self.preview_dir.join(&preview_filename);
        
        self.create_placeholder_preview(&preview_path, "?", (128, 128, 128)).await?;
        
        Ok(PreviewInfo {
            thumbnail_path: preview_path,
            thumbnail_size: self.max_preview_size,
            rendered_preview: None,
            generated_at: Utc::now(),
        })
    }
    
    /// Create a placeholder preview image
    async fn create_placeholder_preview<P: AsRef<Path>>(
        &self,
        output_path: P,
        text: &str,
        color: (u8, u8, u8),
    ) -> DamResult<()> {
        let output_path = output_path.as_ref();
        let (width, height) = self.max_preview_size;
        
        // Create a simple colored rectangle as placeholder
        let mut img = image::RgbImage::new(width, height);
        
        // Fill with color
        for pixel in img.pixels_mut() {
            *pixel = image::Rgb([color.0, color.1, color.2]);
        }
        
        // Save the placeholder
        img.save_with_format(output_path, image::ImageFormat::Jpeg)
            .map_err(|e| IngestError::preview_generation_failed(
                output_path.to_path_buf(),
                format!("Failed to save placeholder: {}", e)
            ))?;
        
        Ok(())
    }
    
    /// Calculate thumbnail dimensions maintaining aspect ratio
    fn calculate_thumbnail_size(&self, original_width: u32, original_height: u32) -> (u32, u32) {
        let (max_width, max_height) = self.max_preview_size;
        
        if original_width <= max_width && original_height <= max_height {
            return (original_width, original_height);
        }
        
        let width_ratio = max_width as f32 / original_width as f32;
        let height_ratio = max_height as f32 / original_height as f32;
        let scale = width_ratio.min(height_ratio);
        
        let new_width = (original_width as f32 * scale) as u32;
        let new_height = (original_height as f32 * scale) as u32;
        
        (new_width.max(1), new_height.max(1))
    }
    
    /// Check if a preview already exists for an asset
    pub async fn preview_exists(&self, asset_id: &uuid::Uuid) -> bool {
        let preview_filename = format!("{}.jpg", asset_id);
        let preview_path = self.preview_dir.join(preview_filename);
        preview_path.exists()
    }
    
    /// Delete preview for an asset
    pub async fn delete_preview(&self, asset_id: &uuid::Uuid) -> DamResult<()> {
        let preview_filename = format!("{}.jpg", asset_id);
        let preview_path = self.preview_dir.join(preview_filename);
        
        if preview_path.exists() {
            tokio::fs::remove_file(&preview_path).await?;
            debug!("Deleted preview: {}", preview_path.display());
        }
        
        Ok(())
    }
    
    /// Get the path where a preview would be stored
    pub fn get_preview_path(&self, asset_id: &uuid::Uuid) -> PathBuf {
        let preview_filename = format!("{}.jpg", asset_id);
        self.preview_dir.join(preview_filename)
    }
    
    /// Clean up old previews that no longer have corresponding assets
    pub async fn cleanup_orphaned_previews(&self, valid_asset_ids: &[uuid::Uuid]) -> DamResult<usize> {
        let mut cleaned_count = 0;
        
        let mut dir_entries = tokio::fs::read_dir(&self.preview_dir).await?;
        
        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("jpg") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(asset_id) = uuid::Uuid::parse_str(filename) {
                        if !valid_asset_ids.contains(&asset_id) {
                            if let Err(e) = tokio::fs::remove_file(&path).await {
                                warn!("Failed to delete orphaned preview {}: {}", path.display(), e);
                            } else {
                                cleaned_count += 1;
                                debug!("Cleaned up orphaned preview: {}", path.display());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(cleaned_count)
    }
}

impl Default for PreviewGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create PreviewGenerator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use uuid::Uuid;
    
    #[test]
    fn test_preview_generator_creation() {
        let generator = PreviewGenerator::new();
        assert!(generator.is_ok());
    }
    
    #[test]
    fn test_thumbnail_size_calculation() {
        let generator = PreviewGenerator::new().unwrap();
        
        // Test image smaller than max size
        let (width, height) = generator.calculate_thumbnail_size(300, 200);
        assert_eq!((width, height), (300, 200));
        
        // Test image larger than max size (landscape)
        let (width, height) = generator.calculate_thumbnail_size(1920, 1080);
        assert!(width <= 512);
        assert!(height <= 512);
        assert_eq!(width as f32 / height as f32, 1920.0 / 1080.0); // Aspect ratio preserved
        
        // Test image larger than max size (portrait)
        let (width, height) = generator.calculate_thumbnail_size(1080, 1920);
        assert!(width <= 512);
        assert!(height <= 512);
        assert_eq!(width as f32 / height as f32, 1080.0 / 1920.0); // Aspect ratio preserved
        
        // Test square image
        let (width, height) = generator.calculate_thumbnail_size(1000, 1000);
        assert_eq!((width, height), (512, 512));
    }
    
    #[test]
    fn test_preview_path_generation() {
        let dir = tempdir().unwrap();
        let generator = PreviewGenerator::with_settings(
            dir.path(),
            (256, 256),
            80
        ).unwrap();
        
        let asset_id = Uuid::new_v4();
        let preview_path = generator.get_preview_path(&asset_id);
        
        assert!(preview_path.starts_with(dir.path()));
        assert!(preview_path.to_string_lossy().contains(&asset_id.to_string()));
        assert!(preview_path.extension().unwrap() == "jpg");
    }
    
    #[tokio::test]
    async fn test_placeholder_creation() {
        let dir = tempdir().unwrap();
        let generator = PreviewGenerator::with_settings(
            dir.path(),
            (128, 128),
            80
        ).unwrap();
        
        let placeholder_path = dir.path().join("test_placeholder.jpg");
        let result = generator.create_placeholder_preview(
            &placeholder_path,
            "TEST",
            (255, 0, 0)
        ).await;
        
        assert!(result.is_ok());
        assert!(placeholder_path.exists());
        
        // Verify the image can be loaded
        let img = image::open(&placeholder_path);
        assert!(img.is_ok());
        
        let img = img.unwrap();
        assert_eq!(img.dimensions(), (128, 128));
    }
}
