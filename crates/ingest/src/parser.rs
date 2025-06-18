//! Asset metadata parsing
//! 
//! This module extracts metadata from various file formats to populate
//! the AssetMetadata structure with format-specific information.

use schema::{
    Asset, AssetMetadata, AssetType, DamResult,
    ImageMetadata, PsdLayer, ThreeDMetadata, BoundingBox, AnimationInfo,
    AudioMetadata, VideoMetadata,
};
use std::path::Path;
use tokio::fs;
use tracing::{debug, warn, error};
use crate::error::IngestError;
use image::{io::Reader as ImageReader, GenericImageView};
// use obj_rs as obj; // TODO: Fix obj-rs dependency issue

/// Service for parsing asset metadata
pub struct AssetParser {
    /// Maximum file size to read into memory for parsing (128MB)
    max_file_size: u64,
}

impl AssetParser {
    /// Create a new asset parser
    pub fn new() -> DamResult<Self> {
        Ok(Self {
            max_file_size: 128 * 1024 * 1024, // 128MB
        })
    }
    
    /// Parse metadata from an asset
    pub async fn parse_metadata(&self, asset: &Asset) -> DamResult<AssetMetadata> {
        let path = &asset.current_path;
        
        // Check file size before attempting to parse
        if asset.file_size > self.max_file_size {
            warn!("File too large for metadata parsing: {} ({} bytes)", 
                  path.display(), asset.file_size);
            return Ok(AssetMetadata::default());
        }
        
        debug!("Parsing metadata for: {}", path.display());
        
        let mut metadata = AssetMetadata::default();
        
        match asset.asset_type {
            AssetType::Image => {
                metadata.image = self.parse_image_metadata(path).await.ok();
            }
            AssetType::ThreeD => {
                metadata.three_d = self.parse_3d_metadata(path).await.ok();
            }
            AssetType::Audio => {
                metadata.audio = self.parse_audio_metadata(path).await.ok();
            }
            AssetType::Video => {
                metadata.video = self.parse_video_metadata(path).await.ok();
            }
            _ => {
                debug!("No specific metadata parser for asset type: {:?}", asset.asset_type);
            }
        }
        
        Ok(metadata)
    }
    
    /// Parse image metadata
    async fn parse_image_metadata<P: AsRef<Path>>(&self, path: P) -> DamResult<ImageMetadata> {
        let path = path.as_ref();
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "psd" | "psb" => self.parse_psd_metadata(path).await,
            _ => self.parse_standard_image_metadata(path).await,
        }
    }
    
    /// Parse standard image formats (PNG, JPEG, etc.)
    async fn parse_standard_image_metadata<P: AsRef<Path>>(&self, path: P) -> DamResult<ImageMetadata> {
        let path = path.as_ref();
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // Use the image crate to read basic metadata
        let img_reader = ImageReader::open(path)
            .map_err(|e| IngestError::metadata_extraction_failed(
                path.to_path_buf(), 
                format!("Failed to open image: {}", e)
            ))?;
        
        let (width, height) = img_reader.into_dimensions()
            .map_err(|e| IngestError::metadata_extraction_failed(
                path.to_path_buf(),
                format!("Failed to read dimensions: {}", e)
            ))?;
        
        // Try to determine color information from file format
        let (bit_depth, color_space, has_alpha) = self.detect_color_info(&extension);
        
        Ok(ImageMetadata {
            width,
            height,
            bit_depth,
            color_space,
            has_alpha,
            layers: None,
        })
    }
    
    /// Parse Photoshop PSD metadata including layers
    async fn parse_psd_metadata<P: AsRef<Path>>(&self, path: P) -> DamResult<ImageMetadata> {
        let path = path.as_ref();
        
        // Read the PSD file
        let psd_data = fs::read(path).await?;
        
        let psd = psd::Psd::from_bytes(&psd_data)
            .map_err(|e| IngestError::metadata_extraction_failed(
                path.to_path_buf(),
                format!("Failed to parse PSD: {}", e)
            ))?;
        
        // Extract basic image info
        let width = psd.width();
        let height = psd.height();
        let bit_depth = psd.depth() as u8;
        let color_space = format!("{:?}", psd.color_mode());
        let has_alpha = psd.color_mode() == psd::ColorMode::Rgb; // Simplified check
        
        // Extract layer information
        let mut layers = Vec::new();
        for layer in psd.layers() {
            layers.push(PsdLayer {
                name: layer.name().to_string(),
                opacity: layer.opacity(),
                blend_mode: format!("{:?}", layer.blend_mode()),
                bounds: (
                    layer.layer_left(),
                    layer.layer_top(),
                    layer.layer_right(),
                    layer.layer_bottom(),
                ),
                visible: layer.visible(),
            });
        }
        
        Ok(ImageMetadata {
            width,
            height,
            bit_depth,
            color_space,
            has_alpha,
            layers: if layers.is_empty() { None } else { Some(layers) },
        })
    }
    
    /// Parse 3D model metadata
    async fn parse_3d_metadata<P: AsRef<Path>>(&self, path: P) -> DamResult<ThreeDMetadata> {
        let path = path.as_ref();
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "gltf" | "glb" => self.parse_gltf_metadata(path).await,
            "obj" => self.parse_obj_metadata(path).await,
            "blend" => self.parse_blend_metadata(path).await,
            _ => {
                // For unsupported 3D formats, return basic metadata
                Ok(ThreeDMetadata {
                    vertex_count: None,
                    face_count: None,
                    material_count: None,
                    bounds: None,
                    animations: Vec::new(),
                    textures: Vec::new(),
                })
            }
        }
    }
    
    /// Parse glTF/GLB metadata
    async fn parse_gltf_metadata<P: AsRef<Path>>(&self, path: P) -> DamResult<ThreeDMetadata> {
        let path = path.as_ref();
        
        let (gltf, _buffers, _images) = gltf::import(path)
            .map_err(|e| IngestError::metadata_extraction_failed(
                path.to_path_buf(),
                format!("Failed to parse glTF: {}", e)
            ))?;
        
        let mut vertex_count = 0u32;
        let mut face_count = 0u32;
        let mut min_bounds = [f32::INFINITY; 3];
        let mut max_bounds = [f32::NEG_INFINITY; 3];
        let mut textures = Vec::new();
        let mut animations = Vec::new();
        
        // Count vertices and faces from meshes
        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                if let Some(accessor) = primitive.get(&gltf::Semantic::Positions) {
                    vertex_count += accessor.count() as u32;
                    
                    // Update bounding box - simplified without bounds check
                    // Note: accessor.bounds() may not be available in all gltf versions
                }
                
                if let Some(indices) = primitive.indices() {
                    face_count += (indices.count() / 3) as u32;
                }
            }
        }
        
        // Collect texture information
        for texture in gltf.textures() {
            let source = texture.source();
            match source.source() {
                gltf::image::Source::Uri { uri, .. } => {
                    textures.push(uri.to_string());
                }
                _ => {}
            }
        }
        
        // Collect animation information
        for animation in gltf.animations() {
            let name = animation.name().unwrap_or("Unnamed").to_string();
            let duration = 0.0f32; // Simplified - would need proper time calculation
            
            animations.push(AnimationInfo {
                name,
                duration,
                frame_count: (duration * 30.0) as u32, // Assume 30 FPS
            });
        }
        
        let bounds = if min_bounds[0].is_finite() {
            Some(BoundingBox {
                min: (min_bounds[0], min_bounds[1], min_bounds[2]),
                max: (max_bounds[0], max_bounds[1], max_bounds[2]),
            })
        } else {
            None
        };
        
        Ok(ThreeDMetadata {
            vertex_count: Some(vertex_count),
            face_count: Some(face_count),
            material_count: Some(gltf.materials().count() as u32),
            bounds,
            animations,
            textures,
        })
    }
    
    /// Parse OBJ metadata
    async fn parse_obj_metadata<P: AsRef<Path>>(&self, path: P) -> DamResult<ThreeDMetadata> {
        let path = path.as_ref();
        
        // TODO: Implement OBJ parsing once obj-rs dependency is fixed
        warn!("OBJ parsing not fully implemented, returning basic metadata for: {}", path.display());
        
        Ok(ThreeDMetadata {
            vertex_count: None,
            face_count: None,
            material_count: None,
            bounds: None,
            animations: Vec::new(),
            textures: Vec::new(),
        })
    }
    
    /// Parse Blender file metadata (basic)
    async fn parse_blend_metadata<P: AsRef<Path>>(&self, path: P) -> DamResult<ThreeDMetadata> {
        let _path = path.as_ref();
        
        // Blender files are complex binary formats
        // For now, return basic metadata and suggest using Blender CLI for detailed extraction
        warn!("Blender file parsing not fully implemented, returning basic metadata");
        
        Ok(ThreeDMetadata {
            vertex_count: None,
            face_count: None,
            material_count: None,
            bounds: None,
            animations: Vec::new(),
            textures: Vec::new(),
        })
    }
    
    /// Parse audio metadata
    async fn parse_audio_metadata<P: AsRef<Path>>(&self, path: P) -> DamResult<AudioMetadata> {
        let path = path.as_ref();
        
        // Use symphonia to read audio metadata
        let src = std::fs::File::open(path)
            .map_err(|e| IngestError::metadata_extraction_failed(
                path.to_path_buf(),
                format!("Failed to open audio file: {}", e)
            ))?;
        
        let mss = symphonia::core::io::MediaSourceStream::new(Box::new(src), Default::default());
        let mut hint = symphonia::core::probe::Hint::new();
        
        if let Some(extension) = path.extension() {
            hint.with_extension(&extension.to_string_lossy());
        }
        
        let meta_opts = symphonia::core::meta::MetadataOptions::default();
        let fmt_opts = symphonia::core::formats::FormatOptions::default();
        
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| IngestError::metadata_extraction_failed(
                path.to_path_buf(),
                format!("Failed to probe audio format: {}", e)
            ))?;
        
        let mut format = probed.format;
        let track = format.tracks()
            .iter()
            .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or_else(|| IngestError::metadata_extraction_failed(
                path.to_path_buf(),
                "No audio tracks found".to_string()
            ))?;
        
        let codec_params = &track.codec_params;
        
        let duration = if let Some(n_frames) = codec_params.n_frames {
            if let Some(sample_rate) = codec_params.sample_rate {
                n_frames as f32 / sample_rate as f32
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        Ok(AudioMetadata {
            duration,
            sample_rate: codec_params.sample_rate.unwrap_or(0),
            channels: codec_params.channels.map(|ch| ch.count() as u8).unwrap_or(0),
            bit_rate: codec_params.bits_per_sample.map(|bps| bps as u32),
            format: format!("{:?}", codec_params.codec),
            transcription: None, // Will be filled by AI processing
        })
    }
    
    /// Parse video metadata
    async fn parse_video_metadata<P: AsRef<Path>>(&self, path: P) -> DamResult<VideoMetadata> {
        let path = path.as_ref();
        
        // For now, return basic video metadata
        // A full implementation would use ffmpeg or similar
        warn!("Video metadata parsing not fully implemented for: {}", path.display());
        
        Ok(VideoMetadata {
            duration: 0.0,
            width: 0,
            height: 0,
            fps: 0.0,
            video_codec: "unknown".to_string(),
            audio_codec: None,
            bit_rate: None,
        })
    }
    
    /// Detect color information from file extension
    fn detect_color_info(&self, extension: &str) -> (u8, String, bool) {
        match extension {
            "png" => (8, "RGB".to_string(), true),
            "jpg" | "jpeg" => (8, "RGB".to_string(), false),
            "gif" => (8, "Indexed".to_string(), true),
            "bmp" => (8, "RGB".to_string(), false),
            "tiff" => (8, "RGB".to_string(), true),
            "webp" => (8, "RGB".to_string(), true),
            _ => (8, "RGB".to_string(), false),
        }
    }
}

impl Default for AssetParser {
    fn default() -> Self {
        Self::new().expect("Failed to create AssetParser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    
    #[tokio::test]
    async fn test_parser_creation() {
        let parser = AssetParser::new();
        assert!(parser.is_ok());
    }
    
    #[test]
    fn test_color_info_detection() {
        let parser = AssetParser::new().unwrap();
        
        let (bit_depth, color_space, has_alpha) = parser.detect_color_info("png");
        assert_eq!(bit_depth, 8);
        assert_eq!(color_space, "RGB");
        assert!(has_alpha);
        
        let (bit_depth, color_space, has_alpha) = parser.detect_color_info("jpg");
        assert_eq!(bit_depth, 8);
        assert_eq!(color_space, "RGB");
        assert!(!has_alpha);
    }
    
    #[tokio::test]
    async fn test_metadata_default() {
        let metadata = AssetMetadata::default();
        assert!(metadata.image.is_none());
        assert!(metadata.three_d.is_none());
        assert!(metadata.audio.is_none());
        assert!(metadata.video.is_none());
        assert!(metadata.custom.is_empty());
    }
}
