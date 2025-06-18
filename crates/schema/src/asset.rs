//! Core asset data structures
//! 
//! Defines the fundamental types for representing digital assets in the DAM system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// A digital asset in the DAM system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// Unique identifier for this asset
    pub id: Uuid,
    
    /// Original file path when ingested
    pub original_path: PathBuf,
    
    /// Current file path (may change during organization)
    pub current_path: PathBuf,
    
    /// Type of asset (image, 3D model, audio, etc.)
    pub asset_type: AssetType,
    
    /// File size in bytes
    pub file_size: u64,
    
    /// File format information
    pub format: FileFormat,
    
    /// When this asset was first ingested
    pub created_at: DateTime<Utc>,
    
    /// Last modification time
    pub modified_at: DateTime<Utc>,
    
    /// AI-generated tags describing the asset
    pub tags: Vec<String>,
    
    /// Additional metadata extracted from the file
    pub metadata: AssetMetadata,
    
    /// Preview/thumbnail information
    pub preview: Option<PreviewInfo>,
    
    /// Vector embedding for semantic search
    pub embedding: Option<Vec<f32>>,
    
    /// Version control information
    pub version_info: VersionInfo,
}

/// Categories of digital assets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AssetType {
    /// 2D images (PNG, JPEG, PSD, etc.)
    Image,
    
    /// 3D models and scenes
    ThreeD,
    
    /// Audio files (WAV, MP3, etc.)
    Audio,
    
    /// Video files (MP4, MOV, etc.)
    Video,
    
    /// Text documents
    Document,
    
    /// Archive files (ZIP, TAR, etc.)
    Archive,
    
    /// Unknown or unsupported format
    Unknown,
}

/// File format details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFormat {
    /// File extension (e.g., "png", "blend")
    pub extension: String,
    
    /// MIME type if known
    pub mime_type: Option<String>,
    
    /// Format-specific version info
    pub version: Option<String>,
    
    /// Whether this format is fully supported
    pub supported: bool,
}

/// Asset-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    /// Image-specific metadata
    pub image: Option<ImageMetadata>,
    
    /// 3D model metadata
    pub three_d: Option<ThreeDMetadata>,
    
    /// Audio metadata
    pub audio: Option<AudioMetadata>,
    
    /// Video metadata
    pub video: Option<VideoMetadata>,
    
    /// Custom metadata fields
    pub custom: HashMap<String, String>,
}

/// Image-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    /// Image dimensions
    pub width: u32,
    pub height: u32,
    
    /// Color depth (bits per pixel)
    pub bit_depth: u8,
    
    /// Color space (RGB, CMYK, etc.)
    pub color_space: String,
    
    /// Whether image has transparency
    pub has_alpha: bool,
    
    /// PSD-specific layer information
    pub layers: Option<Vec<PsdLayer>>,
}

/// Photoshop layer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsdLayer {
    /// Layer name
    pub name: String,
    
    /// Layer opacity (0-255)
    pub opacity: u8,
    
    /// Layer blend mode
    pub blend_mode: String,
    
    /// Layer bounds
    pub bounds: (i32, i32, i32, i32), // left, top, right, bottom
    
    /// Whether layer is visible
    pub visible: bool,
}

/// 3D model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreeDMetadata {
    /// Number of vertices
    pub vertex_count: Option<u32>,
    
    /// Number of polygons/faces
    pub face_count: Option<u32>,
    
    /// Number of materials
    pub material_count: Option<u32>,
    
    /// Bounding box dimensions
    pub bounds: Option<BoundingBox>,
    
    /// Animation information
    pub animations: Vec<AnimationInfo>,
    
    /// Texture references
    pub textures: Vec<String>,
}

/// 3D bounding box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: (f32, f32, f32),
    pub max: (f32, f32, f32),
}

/// Animation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationInfo {
    pub name: String,
    pub duration: f32, // seconds
    pub frame_count: u32,
}

/// Audio metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    /// Duration in seconds
    pub duration: f32,
    
    /// Sample rate (Hz)
    pub sample_rate: u32,
    
    /// Number of channels
    pub channels: u8,
    
    /// Bit rate (kbps)
    pub bit_rate: Option<u32>,
    
    /// Audio format (WAV, MP3, etc.)
    pub format: String,
    
    /// Transcription if available
    pub transcription: Option<String>,
}

/// Video metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    /// Duration in seconds
    pub duration: f32,
    
    /// Video dimensions
    pub width: u32,
    pub height: u32,
    
    /// Frame rate
    pub fps: f32,
    
    /// Video codec
    pub video_codec: String,
    
    /// Audio codec
    pub audio_codec: Option<String>,
    
    /// Bit rate (kbps)
    pub bit_rate: Option<u32>,
}

/// Preview/thumbnail information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewInfo {
    /// Path to thumbnail image
    pub thumbnail_path: PathBuf,
    
    /// Thumbnail dimensions
    pub thumbnail_size: (u32, u32),
    
    /// For 3D models, path to rendered preview
    pub rendered_preview: Option<PathBuf>,
    
    /// Preview generation timestamp
    pub generated_at: DateTime<Utc>,
}

/// Version control information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Current version hash
    pub current_version: String,
    
    /// Number of versions
    pub version_count: u32,
    
    /// Last snapshot timestamp
    pub last_snapshot: DateTime<Utc>,
    
    /// Whether there are uncommitted changes
    pub has_changes: bool,
}

impl Asset {
    /// Create a new asset from a file path
    pub fn new(path: PathBuf, asset_type: AssetType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            original_path: path.clone(),
            current_path: path,
            asset_type,
            file_size: 0,
            format: FileFormat {
                extension: String::new(),
                mime_type: None,
                version: None,
                supported: false,
            },
            created_at: now,
            modified_at: now,
            tags: Vec::new(),
            metadata: AssetMetadata {
                image: None,
                three_d: None,
                audio: None,
                video: None,
                custom: HashMap::new(),
            },
            preview: None,
            embedding: None,
            version_info: VersionInfo {
                current_version: String::new(),
                version_count: 1,
                last_snapshot: now,
                has_changes: false,
            },
        }
    }
    
    /// Get the asset's filename
    pub fn filename(&self) -> Option<&str> {
        self.current_path.file_name()?.to_str()
    }
    
    /// Get the asset's file extension
    pub fn extension(&self) -> Option<&str> {
        self.current_path.extension()?.to_str()
    }
    
    /// Check if this asset has a preview available
    pub fn has_preview(&self) -> bool {
        self.preview.is_some()
    }
    
    /// Check if this asset has been processed by AI
    pub fn is_processed(&self) -> bool {
        !self.tags.is_empty() || self.embedding.is_some()
    }
}

impl Default for AssetMetadata {
    fn default() -> Self {
        Self {
            image: None,
            three_d: None,
            audio: None,
            video: None,
            custom: HashMap::new(),
        }
    }
}

impl AssetType {
    /// Determine asset type from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // Images
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "tga" | "webp" | "psd" => Self::Image,
            
            // 3D formats
            "blend" | "fbx" | "obj" | "gltf" | "glb" | "dae" | "3ds" | "max" | "c4d" => Self::ThreeD,
            
            // Audio
            "wav" | "mp3" | "flac" | "ogg" | "aac" | "m4a" | "wma" => Self::Audio,
            
            // Video
            "mp4" | "mov" | "avi" | "mkv" | "wmv" | "flv" | "webm" => Self::Video,
            
            // Documents
            "txt" | "md" | "pdf" | "doc" | "docx" | "rtf" => Self::Document,
            
            // Archives
            "zip" | "rar" | "tar" | "gz" | "7z" => Self::Archive,
            
            _ => Self::Unknown,
        }
    }
    
    /// Get human-readable name for this asset type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Image => "Image",
            Self::ThreeD => "3D Model",
            Self::Audio => "Audio",
            Self::Video => "Video", 
            Self::Document => "Document",
            Self::Archive => "Archive",
            Self::Unknown => "Unknown",
        }
    }
}
