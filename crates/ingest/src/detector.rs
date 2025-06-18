//! File format detection and validation
//! 
//! This module provides functionality to detect file formats based on:
//! - File extensions
//! - Magic bytes (file signatures)
//! - Content analysis
//! - MIME type detection

use schema::{FileFormat, DamResult};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tracing::{debug, warn};
use crate::error::IngestError;

/// Service for detecting file formats
pub struct FormatDetector {
    /// Magic byte patterns for format detection
    magic_patterns: Vec<MagicPattern>,
}

/// A magic byte pattern for file format detection
#[derive(Debug, Clone)]
struct MagicPattern {
    /// File extension this pattern matches
    extension: String,
    
    /// Magic bytes to look for
    signature: Vec<u8>,
    
    /// Offset where signature should be found (0 = start of file)
    offset: usize,
    
    /// MIME type for this format
    mime_type: String,
    
    /// Whether this format is fully supported
    supported: bool,
}

impl FormatDetector {
    /// Create a new format detector with built-in patterns
    pub fn new() -> DamResult<Self> {
        let mut detector = Self {
            magic_patterns: Vec::new(),
        };
        
        detector.add_builtin_patterns();
        Ok(detector)
    }
    
    /// Detect file format from path and content
    pub async fn detect_format<P: AsRef<Path>>(&self, path: P) -> DamResult<FileFormat> {
        let path = path.as_ref();
        
        // First try extension-based detection
        let mut format = self.detect_from_extension(path);
        
        // Then try magic byte detection for more accurate results
        if let Ok(magic_format) = self.detect_from_magic_bytes(path).await {
            // If magic bytes give us a different result, prefer that
            if magic_format.extension != format.extension {
                debug!(
                    "Magic byte detection overrides extension: {} -> {} for {}",
                    format.extension,
                    magic_format.extension,
                    path.display()
                );
                format = magic_format;
            }
        }
        
        // Try MIME type detection as fallback
        if format.mime_type.is_none() {
            if let Some(mime_type) = self.detect_mime_type(path).await {
                format.mime_type = Some(mime_type);
            }
        }
        
        Ok(format)
    }
    
    /// Detect format based on file extension
    fn detect_from_extension<P: AsRef<Path>>(&self, path: P) -> FileFormat {
        let path = path.as_ref();
        
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // Check against known extensions
        let supported = self.is_extension_supported(&extension);
        let mime_type = self.extension_to_mime(&extension);
        
        FileFormat {
            extension,
            mime_type,
            version: None,
            supported,
        }
    }
    
    /// Detect format from magic bytes at the start of the file
    async fn detect_from_magic_bytes<P: AsRef<Path>>(&self, path: P) -> DamResult<FileFormat> {
        let path = path.as_ref();
        
        // Read first 512 bytes for magic byte detection
        let mut file = fs::File::open(path).await?;
        let mut buffer = vec![0u8; 512];
        let bytes_read = file.read(&mut buffer).await?;
        buffer.truncate(bytes_read);
        
        // Check against all magic patterns
        for pattern in &self.magic_patterns {
            if self.matches_pattern(&buffer, pattern) {
                return Ok(FileFormat {
                    extension: pattern.extension.clone(),
                    mime_type: Some(pattern.mime_type.clone()),
                    version: None,
                    supported: pattern.supported,
                });
            }
        }
        
        Err(IngestError::UnknownFormat {
            path: path.to_path_buf(),
        }.into())
    }
    
    /// Detect MIME type using the infer crate
    async fn detect_mime_type<P: AsRef<Path>>(&self, path: P) -> Option<String> {
        let path = path.as_ref();
        
        // Read first 8KB for MIME detection
        if let Ok(mut file) = fs::File::open(path).await {
            let mut buffer = vec![0u8; 8192];
            if let Ok(bytes_read) = file.read(&mut buffer).await {
                buffer.truncate(bytes_read);
                
                if let Some(kind) = infer::get(&buffer) {
                    return Some(kind.mime_type().to_string());
                }
            }
        }
        
        None
    }
    
    /// Check if a magic pattern matches the given buffer
    fn matches_pattern(&self, buffer: &[u8], pattern: &MagicPattern) -> bool {
        if buffer.len() < pattern.offset + pattern.signature.len() {
            return false;
        }
        
        let start = pattern.offset;
        let end = start + pattern.signature.len();
        
        &buffer[start..end] == pattern.signature.as_slice()
    }
    
    /// Check if an extension is supported
    fn is_extension_supported(&self, extension: &str) -> bool {
        match extension {
            // Images
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "tga" | "webp" | "psd" | "psb" => true,
            
            // 3D formats
            "blend" | "fbx" | "obj" | "gltf" | "glb" | "dae" | "3ds" | "ply" | "stl" => true,
            
            // Audio
            "wav" | "mp3" | "flac" | "ogg" | "aac" | "m4a" => true,
            
            // Video
            "mp4" | "mov" | "avi" | "mkv" | "wmv" | "webm" => true,
            
            // Documents
            "txt" | "md" | "pdf" | "doc" | "docx" => true,
            
            // Archives
            "zip" | "rar" | "tar" | "gz" | "7z" => true,
            
            _ => false,
        }
    }
    
    /// Convert extension to MIME type
    fn extension_to_mime(&self, extension: &str) -> Option<String> {
        let mime_type = match extension {
            // Images
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "tiff" => "image/tiff",
            "webp" => "image/webp",
            "psd" => "image/vnd.adobe.photoshop",
            
            // 3D formats
            "gltf" => "model/gltf+json",
            "glb" => "model/gltf-binary",
            "obj" => "text/plain", // OBJ files are text-based
            
            // Audio
            "wav" => "audio/wav",
            "mp3" => "audio/mpeg",
            "flac" => "audio/flac",
            "ogg" => "audio/ogg",
            "aac" => "audio/aac",
            "m4a" => "audio/mp4",
            
            // Video
            "mp4" => "video/mp4",
            "mov" => "video/quicktime",
            "avi" => "video/x-msvideo",
            "mkv" => "video/x-matroska",
            "webm" => "video/webm",
            
            // Documents
            "txt" => "text/plain",
            "md" => "text/markdown",
            "pdf" => "application/pdf",
            
            // Archives
            "zip" => "application/zip",
            "tar" => "application/x-tar",
            "gz" => "application/gzip",
            
            _ => return None,
        };
        
        Some(mime_type.to_string())
    }
    
    /// Add built-in magic byte patterns
    fn add_builtin_patterns(&mut self) {
        // PNG
        self.add_pattern("png", vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], 0, "image/png", true);
        
        // JPEG
        self.add_pattern("jpg", vec![0xFF, 0xD8, 0xFF], 0, "image/jpeg", true);
        
        // GIF
        self.add_pattern("gif", vec![0x47, 0x49, 0x46, 0x38], 0, "image/gif", true);
        
        // BMP
        self.add_pattern("bmp", vec![0x42, 0x4D], 0, "image/bmp", true);
        
        // TIFF (little endian)
        self.add_pattern("tiff", vec![0x49, 0x49, 0x2A, 0x00], 0, "image/tiff", true);
        
        // TIFF (big endian)
        self.add_pattern("tiff", vec![0x4D, 0x4D, 0x00, 0x2A], 0, "image/tiff", true);
        
        // WebP
        self.add_pattern("webp", vec![0x57, 0x45, 0x42, 0x50], 8, "image/webp", true);
        
        // Photoshop PSD
        self.add_pattern("psd", vec![0x38, 0x42, 0x50, 0x53], 0, "image/vnd.adobe.photoshop", true);
        
        // ZIP (and formats based on ZIP like GLTF GLB)
        self.add_pattern("zip", vec![0x50, 0x4B, 0x03, 0x04], 0, "application/zip", true);
        self.add_pattern("zip", vec![0x50, 0x4B, 0x05, 0x06], 0, "application/zip", true);
        
        // glTF binary
        self.add_pattern("glb", vec![0x67, 0x6C, 0x54, 0x46], 0, "model/gltf-binary", true);
        
        // Blender files
        self.add_pattern("blend", vec![0x42, 0x4C, 0x45, 0x4E, 0x44, 0x45, 0x52], 0, "application/x-blender", true);
        
        // WAV
        self.add_pattern("wav", vec![0x52, 0x49, 0x46, 0x46], 0, "audio/wav", true);
        
        // MP3
        self.add_pattern("mp3", vec![0xFF, 0xFB], 0, "audio/mpeg", true);
        self.add_pattern("mp3", vec![0x49, 0x44, 0x33], 0, "audio/mpeg", true); // ID3 tag
        
        // FLAC
        self.add_pattern("flac", vec![0x66, 0x4C, 0x61, 0x43], 0, "audio/flac", true);
        
        // OGG
        self.add_pattern("ogg", vec![0x4F, 0x67, 0x67, 0x53], 0, "audio/ogg", true);
        
        // MP4/M4A
        self.add_pattern("mp4", vec![0x66, 0x74, 0x79, 0x70], 4, "video/mp4", true);
        
        // AVI
        self.add_pattern("avi", vec![0x52, 0x49, 0x46, 0x46], 0, "video/x-msvideo", true);
        
        // PDF
        self.add_pattern("pdf", vec![0x25, 0x50, 0x44, 0x46, 0x2D], 0, "application/pdf", true);
        
        // RAR
        self.add_pattern("rar", vec![0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00], 0, "application/vnd.rar", true);
        
        // 7-Zip
        self.add_pattern("7z", vec![0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C], 0, "application/x-7z-compressed", true);
        
        // GZIP
        self.add_pattern("gz", vec![0x1F, 0x8B], 0, "application/gzip", true);
    }
    
    /// Add a magic pattern to the detector
    fn add_pattern(&mut self, extension: &str, signature: Vec<u8>, offset: usize, mime_type: &str, supported: bool) {
        self.magic_patterns.push(MagicPattern {
            extension: extension.to_string(),
            signature,
            offset,
            mime_type: mime_type.to_string(),
            supported,
        });
    }
}

impl Default for FormatDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create FormatDetector")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    
    #[tokio::test]
    async fn test_format_detector_creation() {
        let detector = FormatDetector::new();
        assert!(detector.is_ok());
    }
    
    #[tokio::test]
    async fn test_extension_detection() {
        let detector = FormatDetector::new().unwrap();
        
        let format = detector.detect_from_extension("test.png");
        assert_eq!(format.extension, "png");
        assert!(format.supported);
        assert_eq!(format.mime_type, Some("image/png".to_string()));
        
        let format = detector.detect_from_extension("unknown.xyz");
        assert_eq!(format.extension, "xyz");
        assert!(!format.supported);
    }
    
    #[tokio::test]
    async fn test_magic_byte_detection() {
        let detector = FormatDetector::new().unwrap();
        let dir = tempdir().unwrap();
        
        // Create a fake PNG file with correct magic bytes
        let png_path = dir.path().join("test.dat");
        let mut file = File::create(&png_path).await.unwrap();
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).await.unwrap();
        file.write_all(b"fake png data").await.unwrap();
        file.flush().await.unwrap();
        
        let format = detector.detect_from_magic_bytes(&png_path).await.unwrap();
        assert_eq!(format.extension, "png");
        assert_eq!(format.mime_type, Some("image/png".to_string()));
        assert!(format.supported);
    }
    
    #[test]
    fn test_extension_support() {
        let detector = FormatDetector::new().unwrap();
        
        assert!(detector.is_extension_supported("png"));
        assert!(detector.is_extension_supported("blend"));
        assert!(detector.is_extension_supported("wav"));
        assert!(!detector.is_extension_supported("xyz"));
    }
    
    #[test]
    fn test_mime_type_conversion() {
        let detector = FormatDetector::new().unwrap();
        
        assert_eq!(detector.extension_to_mime("png"), Some("image/png".to_string()));
        assert_eq!(detector.extension_to_mime("mp4"), Some("video/mp4".to_string()));
        assert_eq!(detector.extension_to_mime("wav"), Some("audio/wav".to_string()));
        assert_eq!(detector.extension_to_mime("xyz"), None);
    }
}
