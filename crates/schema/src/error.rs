//! Error types for the DAM system
//! 
//! Defines standardized error types used throughout the system.

use thiserror::Error;
use std::path::PathBuf;
use uuid::Uuid;

/// Main error type for the DAM system
#[derive(Error, Debug)]
pub enum DamError {
    /// File system related errors
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    
    /// Asset ingestion errors
    #[error("Ingestion error: {message}")]
    Ingestion { message: String },
    
    /// Asset processing errors
    #[error("Processing error: {message}")]
    Processing { message: String },
    
    /// Search/indexing errors
    #[error("Search error: {message}")]
    Search { message: String },
    
    /// Version control errors
    #[error("Version control error: {message}")]
    VersionControl { message: String },
    
    /// Server/networking errors
    #[error("Server error: {message}")]
    Server { message: String },
    
    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    /// Unsupported file format
    #[error("Unsupported file format: {format} for file {path}")]
    UnsupportedFormat {
        format: String,
        path: PathBuf,
    },
    
    /// Asset not found
    #[error("Asset not found: {asset_id}")]
    AssetNotFound { asset_id: Uuid },
    
    /// Invalid asset data
    #[error("Invalid asset data: {message}")]
    InvalidAssetData { message: String },
    
    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// AI/ML processing errors
    #[error("AI processing error: {message}")]
    AiProcessing { message: String },
    
    /// Database/storage errors
    #[error("Storage error: {message}")]
    Storage { message: String },
    
    /// Authentication/authorization errors
    #[error("Authentication error: {message}")]
    Authentication { message: String },
    
    /// Permission denied
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },
    
    /// Resource not available
    #[error("Resource not available: {resource}")]
    ResourceNotAvailable { resource: String },
    
    /// Operation timeout
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },
    
    /// Invalid operation
    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },
    
    /// External dependency error
    #[error("External dependency error: {dependency} - {message}")]
    ExternalDependency {
        dependency: String,
        message: String,
    },
}

/// Result type alias for convenience
pub type DamResult<T> = Result<T, DamError>;

/// Error categories for easier error handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    /// File system and I/O errors
    FileSystem,
    
    /// Asset-related errors
    Asset,
    
    /// Processing and AI errors
    Processing,
    
    /// Search and indexing errors
    Search,
    
    /// Version control errors
    VersionControl,
    
    /// Network and server errors
    Network,
    
    /// Configuration errors
    Configuration,
    
    /// Authentication and permission errors
    Security,
    
    /// External dependency errors
    External,
    
    /// System-level errors
    System,
}

impl DamError {
    /// Get the error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            DamError::FileSystem(_) => ErrorCategory::FileSystem,
            DamError::Ingestion { .. } => ErrorCategory::Asset,
            DamError::Processing { .. } => ErrorCategory::Processing,
            DamError::Search { .. } => ErrorCategory::Search,
            DamError::VersionControl { .. } => ErrorCategory::VersionControl,
            DamError::Server { .. } => ErrorCategory::Network,
            DamError::Configuration { .. } => ErrorCategory::Configuration,
            DamError::UnsupportedFormat { .. } => ErrorCategory::Asset,
            DamError::AssetNotFound { .. } => ErrorCategory::Asset,
            DamError::InvalidAssetData { .. } => ErrorCategory::Asset,
            DamError::Serialization(_) => ErrorCategory::System,
            DamError::AiProcessing { .. } => ErrorCategory::Processing,
            DamError::Storage { .. } => ErrorCategory::System,
            DamError::Authentication { .. } => ErrorCategory::Security,
            DamError::PermissionDenied { .. } => ErrorCategory::Security,
            DamError::ResourceNotAvailable { .. } => ErrorCategory::System,
            DamError::Timeout { .. } => ErrorCategory::System,
            DamError::InvalidOperation { .. } => ErrorCategory::System,
            DamError::ExternalDependency { .. } => ErrorCategory::External,
        }
    }
    
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            DamError::FileSystem(_) => false,
            DamError::Ingestion { .. } => true,
            DamError::Processing { .. } => true,
            DamError::Search { .. } => true,
            DamError::VersionControl { .. } => true,
            DamError::Server { .. } => true,
            DamError::Configuration { .. } => false,
            DamError::UnsupportedFormat { .. } => false,
            DamError::AssetNotFound { .. } => false,
            DamError::InvalidAssetData { .. } => false,
            DamError::Serialization(_) => false,
            DamError::AiProcessing { .. } => true,
            DamError::Storage { .. } => true,
            DamError::Authentication { .. } => true,
            DamError::PermissionDenied { .. } => false,
            DamError::ResourceNotAvailable { .. } => true,
            DamError::Timeout { .. } => true,
            DamError::InvalidOperation { .. } => false,
            DamError::ExternalDependency { .. } => true,
        }
    }
    
    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            DamError::FileSystem(_) => "File system error occurred".to_string(),
            DamError::Ingestion { .. } => "Failed to import asset".to_string(),
            DamError::Processing { .. } => "Failed to process asset".to_string(),
            DamError::Search { .. } => "Search operation failed".to_string(),
            DamError::VersionControl { .. } => "Version control operation failed".to_string(),
            DamError::Server { .. } => "Server operation failed".to_string(),
            DamError::Configuration { .. } => "Configuration error".to_string(),
            DamError::UnsupportedFormat { format, .. } => {
                format!("Unsupported file format: {}", format)
            }
            DamError::AssetNotFound { .. } => "Asset not found".to_string(),
            DamError::InvalidAssetData { .. } => "Invalid asset data".to_string(),
            DamError::Serialization(_) => "Data serialization error".to_string(),
            DamError::AiProcessing { .. } => "AI processing failed".to_string(),
            DamError::Storage { .. } => "Storage operation failed".to_string(),
            DamError::Authentication { .. } => "Authentication failed".to_string(),
            DamError::PermissionDenied { .. } => "Permission denied".to_string(),
            DamError::ResourceNotAvailable { resource } => {
                format!("Resource not available: {}", resource)
            }
            DamError::Timeout { operation } => {
                format!("Operation timed out: {}", operation)
            }
            DamError::InvalidOperation { .. } => "Invalid operation".to_string(),
            DamError::ExternalDependency { dependency, .. } => {
                format!("External dependency error: {}", dependency)
            }
        }
    }
}

/// Convenience functions for creating specific error types
impl DamError {
    /// Create an ingestion error
    pub fn ingestion<S: Into<String>>(message: S) -> Self {
        Self::Ingestion {
            message: message.into(),
        }
    }
    
    /// Create a processing error
    pub fn processing<S: Into<String>>(message: S) -> Self {
        Self::Processing {
            message: message.into(),
        }
    }
    
    /// Create a search error
    pub fn search<S: Into<String>>(message: S) -> Self {
        Self::Search {
            message: message.into(),
        }
    }
    
    /// Create a version control error
    pub fn version_control<S: Into<String>>(message: S) -> Self {
        Self::VersionControl {
            message: message.into(),
        }
    }
    
    /// Create a server error
    pub fn server<S: Into<String>>(message: S) -> Self {
        Self::Server {
            message: message.into(),
        }
    }
    
    /// Create a configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }
    
    /// Create an unsupported format error
    pub fn unsupported_format<S: Into<String>>(format: S, path: PathBuf) -> Self {
        Self::UnsupportedFormat {
            format: format.into(),
            path,
        }
    }
    
    /// Create an asset not found error
    pub fn asset_not_found(asset_id: Uuid) -> Self {
        Self::AssetNotFound { asset_id }
    }
    
    /// Create an invalid asset data error
    pub fn invalid_asset_data<S: Into<String>>(message: S) -> Self {
        Self::InvalidAssetData {
            message: message.into(),
        }
    }
    
    /// Create an AI processing error
    pub fn ai_processing<S: Into<String>>(message: S) -> Self {
        Self::AiProcessing {
            message: message.into(),
        }
    }
    
    /// Create a storage error
    pub fn storage<S: Into<String>>(message: S) -> Self {
        Self::Storage {
            message: message.into(),
        }
    }
    
    /// Create an authentication error
    pub fn authentication<S: Into<String>>(message: S) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }
    
    /// Create a permission denied error
    pub fn permission_denied<S: Into<String>>(message: S) -> Self {
        Self::PermissionDenied {
            message: message.into(),
        }
    }
    
    /// Create a resource not available error
    pub fn resource_not_available<S: Into<String>>(resource: S) -> Self {
        Self::ResourceNotAvailable {
            resource: resource.into(),
        }
    }
    
    /// Create a timeout error
    pub fn timeout<S: Into<String>>(operation: S) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }
    
    /// Create an invalid operation error
    pub fn invalid_operation<S: Into<String>>(message: S) -> Self {
        Self::InvalidOperation {
            message: message.into(),
        }
    }
    
    /// Create an external dependency error
    pub fn external_dependency<S: Into<String>, T: Into<String>>(
        dependency: S,
        message: T,
    ) -> Self {
        Self::ExternalDependency {
            dependency: dependency.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_categories() {
        assert_eq!(
            DamError::ingestion("test").category(),
            ErrorCategory::Asset
        );
        assert_eq!(
            DamError::processing("test").category(),
            ErrorCategory::Processing
        );
        assert_eq!(
            DamError::search("test").category(),
            ErrorCategory::Search
        );
    }
    
    #[test]
    fn test_error_recoverability() {
        assert!(DamError::processing("test").is_recoverable());
        assert!(!DamError::unsupported_format("test", PathBuf::new()).is_recoverable());
        assert!(!DamError::configuration("test").is_recoverable());
    }
    
    #[test]
    fn test_user_messages() {
        let error = DamError::ingestion("test");
        assert_eq!(error.user_message(), "Failed to import asset");
        
        let error = DamError::unsupported_format("xyz", PathBuf::new());
        assert_eq!(error.user_message(), "Unsupported file format: xyz");
    }
}
