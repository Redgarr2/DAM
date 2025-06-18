//! Ingestion-specific error types

use schema::DamError;
use std::path::PathBuf;
use thiserror::Error;

/// Errors specific to the ingestion process
#[derive(Error, Debug)]
pub enum IngestError {
    /// File not found
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    
    /// Path is not a file
    #[error("Not a file: {path}")]
    NotAFile { path: PathBuf },
    
    /// Path is not a directory
    #[error("Not a directory: {path}")]
    NotADirectory { path: PathBuf },
    
    /// Unknown file format
    #[error("Unknown file format: {path}")]
    UnknownFormat { path: PathBuf },
    
    /// Unsupported file format
    #[error("Unsupported file format: {format} for {path}")]
    UnsupportedFormat { format: String, path: PathBuf },
    
    /// Metadata extraction failed
    #[error("Failed to extract metadata from {path}: {reason}")]
    MetadataExtractionFailed { path: PathBuf, reason: String },
    
    /// Preview generation failed
    #[error("Failed to generate preview for {path}: {reason}")]
    PreviewGenerationFailed { path: PathBuf, reason: String },
    
    /// File system monitoring error
    #[error("File system monitoring error: {reason}")]
    MonitoringError { reason: String },
    
    /// File access permission denied
    #[error("Permission denied accessing {path}")]
    PermissionDenied { path: PathBuf },
    
    /// File is too large to process
    #[error("File too large: {path} ({size} bytes)")]
    FileTooLarge { path: PathBuf, size: u64 },
    
    /// File is corrupted or invalid
    #[error("Corrupted file: {path}")]
    CorruptedFile { path: PathBuf },
    
    /// External tool dependency error
    #[error("External tool error: {tool} - {reason}")]
    ExternalToolError { tool: String, reason: String },
}

impl From<IngestError> for DamError {
    fn from(err: IngestError) -> Self {
        match err {
            IngestError::FileNotFound { path } => {
                DamError::ingestion(format!("File not found: {}", path.display()))
            }
            IngestError::NotAFile { path } => {
                DamError::ingestion(format!("Not a file: {}", path.display()))
            }
            IngestError::NotADirectory { path } => {
                DamError::ingestion(format!("Not a directory: {}", path.display()))
            }
            IngestError::UnknownFormat { path } => {
                DamError::ingestion(format!("Unknown file format: {}", path.display()))
            }
            IngestError::UnsupportedFormat { format, path } => {
                DamError::unsupported_format(format, path)
            }
            IngestError::MetadataExtractionFailed { path, reason } => {
                DamError::ingestion(format!("Failed to extract metadata from {}: {}", path.display(), reason))
            }
            IngestError::PreviewGenerationFailed { path, reason } => {
                DamError::ingestion(format!("Failed to generate preview for {}: {}", path.display(), reason))
            }
            IngestError::MonitoringError { reason } => {
                DamError::ingestion(format!("File system monitoring error: {}", reason))
            }
            IngestError::PermissionDenied { path } => {
                DamError::permission_denied(format!("Cannot access {}", path.display()))
            }
            IngestError::FileTooLarge { path, size } => {
                DamError::ingestion(format!("File too large: {} ({} bytes)", path.display(), size))
            }
            IngestError::CorruptedFile { path } => {
                DamError::ingestion(format!("Corrupted file: {}", path.display()))
            }
            IngestError::ExternalToolError { tool, reason } => {
                DamError::external_dependency(tool, reason)
            }
        }
    }
}

/// Result type for ingestion operations
pub type IngestResult<T> = Result<T, IngestError>;

impl IngestError {
    /// Create a file not found error
    pub fn file_not_found(path: PathBuf) -> Self {
        Self::FileNotFound { path }
    }
    
    /// Create a not a file error
    pub fn not_a_file(path: PathBuf) -> Self {
        Self::NotAFile { path }
    }
    
    /// Create a not a directory error
    pub fn not_a_directory(path: PathBuf) -> Self {
        Self::NotADirectory { path }
    }
    
    /// Create an unknown format error
    pub fn unknown_format(path: PathBuf) -> Self {
        Self::UnknownFormat { path }
    }
    
    /// Create an unsupported format error
    pub fn unsupported_format<S: Into<String>>(format: S, path: PathBuf) -> Self {
        Self::UnsupportedFormat {
            format: format.into(),
            path,
        }
    }
    
    /// Create a metadata extraction error
    pub fn metadata_extraction_failed<S: Into<String>>(path: PathBuf, reason: S) -> Self {
        Self::MetadataExtractionFailed {
            path,
            reason: reason.into(),
        }
    }
    
    /// Create a preview generation error
    pub fn preview_generation_failed<S: Into<String>>(path: PathBuf, reason: S) -> Self {
        Self::PreviewGenerationFailed {
            path,
            reason: reason.into(),
        }
    }
    
    /// Create a monitoring error
    pub fn monitoring_error<S: Into<String>>(reason: S) -> Self {
        Self::MonitoringError {
            reason: reason.into(),
        }
    }
    
    /// Create a permission denied error
    pub fn permission_denied(path: PathBuf) -> Self {
        Self::PermissionDenied { path }
    }
    
    /// Create a file too large error
    pub fn file_too_large(path: PathBuf, size: u64) -> Self {
        Self::FileTooLarge { path, size }
    }
    
    /// Create a corrupted file error
    pub fn corrupted_file(path: PathBuf) -> Self {
        Self::CorruptedFile { path }
    }
    
    /// Create an external tool error
    pub fn external_tool_error<S: Into<String>, T: Into<String>>(tool: S, reason: T) -> Self {
        Self::ExternalToolError {
            tool: tool.into(),
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_error_creation() {
        let path = PathBuf::from("test.txt");
        
        let err = IngestError::file_not_found(path.clone());
        assert!(matches!(err, IngestError::FileNotFound { .. }));
        
        let err = IngestError::unsupported_format("xyz", path.clone());
        assert!(matches!(err, IngestError::UnsupportedFormat { .. }));
        
        let err = IngestError::metadata_extraction_failed(path.clone(), "test reason");
        assert!(matches!(err, IngestError::MetadataExtractionFailed { .. }));
    }
    
    #[test]
    fn test_error_conversion() {
        let path = PathBuf::from("test.txt");
        let ingest_err = IngestError::file_not_found(path);
        let dam_err: DamError = ingest_err.into();
        
        assert!(matches!(dam_err, DamError::Ingestion { .. }));
    }
    
    #[test]
    fn test_error_display() {
        let path = PathBuf::from("test.txt");
        let err = IngestError::file_not_found(path);
        let msg = format!("{}", err);
        assert!(msg.contains("File not found"));
        assert!(msg.contains("test.txt"));
    }
}
