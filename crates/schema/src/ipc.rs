//! Inter-process communication types
//! 
//! Defines message types for communication between different crates in the DAM system.

use crate::{Asset, SearchQuery, SearchResult, IndexOperation, IndexResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Messages sent between different components of the DAM system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DamMessage {
    /// Ingest-related messages
    Ingest(IngestMessage),
    
    /// Processing-related messages
    Process(ProcessMessage),
    
    /// Index-related messages
    Index(IndexMessage),
    
    /// UI-related messages
    Ui(UiMessage),
    
    /// Server-related messages
    Server(ServerMessage),
    
    /// Version control messages
    Version(VersionMessage),
    
    /// System-wide messages
    System(SystemMessage),
}

/// Messages related to asset ingestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IngestMessage {
    /// Request to ingest a file or directory
    IngestPath { path: PathBuf },
    
    /// Request to ingest multiple files
    IngestBatch { paths: Vec<PathBuf> },
    
    /// Report ingestion progress
    Progress { 
        processed: usize, 
        total: usize, 
        current_file: Option<PathBuf> 
    },
    
    /// Ingestion completed successfully
    Completed { 
        assets_created: Vec<Uuid>,
        duration_ms: u64,
    },
    
    /// Ingestion failed
    Failed { 
        path: PathBuf, 
        error: String 
    },
    
    /// File format detected
    FormatDetected { 
        path: PathBuf, 
        format: String, 
        supported: bool 
    },
}

/// Messages related to AI processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessMessage {
    /// Request to process an asset with AI
    ProcessAsset { asset_id: Uuid },
    
    /// Request to process multiple assets
    ProcessBatch { asset_ids: Vec<Uuid> },
    
    /// Request audio transcription
    TranscribeAudio { 
        asset_id: Uuid, 
        audio_path: PathBuf 
    },
    
    /// Request image tagging
    TagImage { 
        asset_id: Uuid, 
        image_path: PathBuf 
    },
    
    /// Request embedding generation
    GenerateEmbedding { 
        asset_id: Uuid, 
        content: String 
    },
    
    /// Request image-to-image editing
    EditImage {
        asset_id: Uuid,
        image_path: PathBuf,
        prompt: String,
        strength: f32,
    },
    
    /// Processing task started
    Started { 
        task_id: Uuid, 
        asset_id: Uuid, 
        task_type: ProcessingTaskType 
    },
    
    /// Processing progress update
    Progress { 
        task_id: Uuid, 
        progress: f32 
    },
    
    /// Processing completed
    Completed { 
        task_id: Uuid, 
        result: ProcessingResult 
    },
    
    /// Processing failed
    Failed { 
        task_id: Uuid, 
        error: String 
    },
}

/// Types of AI processing tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingTaskType {
    Transcription,
    ImageTagging,
    EmbeddingGeneration,
    ImageEditing,
    VideoAnalysis,
}

/// Results from AI processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingResult {
    /// Transcription result
    Transcription { text: String },
    
    /// Image tagging result
    Tags { tags: Vec<String> },
    
    /// Embedding generation result
    Embedding { vector: Vec<f32> },
    
    /// Image editing result
    EditedImage { output_path: PathBuf },
    
    /// Combined results
    Combined { 
        tags: Option<Vec<String>>,
        embedding: Option<Vec<f32>>,
        transcription: Option<String>,
    },
}

/// Messages related to search indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexMessage {
    /// Request to perform an index operation
    Operation { operation: IndexOperation },
    
    /// Search request
    Search { query: SearchQuery },
    
    /// Similarity search request
    SimilaritySearch { 
        query_vector: Vec<f32>, 
        limit: usize 
    },
    
    /// Index operation completed
    OperationCompleted { result: IndexResult },
    
    /// Search results
    SearchResults { result: SearchResult },
    
    /// Index statistics
    Stats { 
        document_count: usize, 
        index_size_bytes: u64,
        last_updated: DateTime<Utc>,
    },
}

/// Messages related to the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiMessage {
    /// Request to display an asset
    ShowAsset { asset_id: Uuid },
    
    /// Request to preview a 3D model
    Preview3D { asset_id: Uuid },
    
    /// Update UI with new asset
    AssetAdded { asset: Asset },
    
    /// Update UI with modified asset
    AssetUpdated { asset: Asset },
    
    /// Remove asset from UI
    AssetRemoved { asset_id: Uuid },
    
    /// Show processing progress
    ShowProgress { 
        title: String, 
        progress: f32, 
        message: Option<String> 
    },
    
    /// Hide progress dialog
    HideProgress,
    
    /// Show notification
    Notification { 
        level: NotificationLevel, 
        title: String, 
        message: String 
    },
}

/// Notification levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// Messages related to the LAN server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Start the server
    Start { port: u16, bind_address: String },
    
    /// Stop the server
    Stop,
    
    /// Server started successfully
    Started { address: String, port: u16 },
    
    /// Server stopped
    Stopped,
    
    /// Client connected
    ClientConnected { 
        client_id: String, 
        ip_address: String 
    },
    
    /// Client disconnected
    ClientDisconnected { client_id: String },
    
    /// Access log entry
    AccessLog { 
        client_id: String, 
        method: String, 
        path: String, 
        status: u16,
        timestamp: DateTime<Utc>,
    },
    
    /// Authentication request
    AuthRequest { 
        client_id: String, 
        token: String 
    },
    
    /// Authentication result
    AuthResult { 
        client_id: String, 
        success: bool, 
        user_id: Option<String> 
    },
}

/// Messages related to version control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionMessage {
    /// Create a snapshot of an asset
    CreateSnapshot { asset_id: Uuid },
    
    /// Get version history for an asset
    GetHistory { asset_id: Uuid },
    
    /// Compare two versions
    CompareVersions { 
        asset_id: Uuid, 
        version1: String, 
        version2: String 
    },
    
    /// Revert to a specific version
    RevertToVersion { 
        asset_id: Uuid, 
        version: String 
    },
    
    /// Snapshot created
    SnapshotCreated { 
        asset_id: Uuid, 
        version: String, 
        timestamp: DateTime<Utc> 
    },
    
    /// Version history response
    History { 
        asset_id: Uuid, 
        versions: Vec<VersionEntry> 
    },
    
    /// Version comparison result
    ComparisonResult { 
        asset_id: Uuid, 
        diff: VersionDiff 
    },
}

/// A single version entry in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub message: Option<String>,
    pub file_size: u64,
    pub changes_summary: Option<String>,
}

/// Difference between two versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    pub version1: String,
    pub version2: String,
    pub diff_type: DiffType,
    pub changes: Vec<Change>,
    pub visual_diff_path: Option<PathBuf>,
}

/// Type of version diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffType {
    /// Binary file comparison
    Binary,
    
    /// Text file comparison
    Text,
    
    /// PSD layer comparison
    PsdLayers,
    
    /// 3D model comparison
    ThreeD,
    
    /// Image comparison
    Image,
}

/// A single change in a version diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub change_type: ChangeType,
    pub description: String,
    pub location: Option<String>,
}

/// Type of change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    Moved,
}

/// System-wide messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemMessage {
    /// Shutdown request
    Shutdown,
    
    /// Status request
    StatusRequest,
    
    /// Status response
    Status { 
        uptime_ms: u64, 
        memory_usage_mb: u64,
        active_tasks: usize,
        indexed_assets: usize,
    },
    
    /// Configuration change
    ConfigChanged { 
        section: String, 
        key: String, 
        value: String 
    },
    
    /// Error occurred
    Error { 
        component: String, 
        error: String, 
        timestamp: DateTime<Utc> 
    },
    
    /// Health check
    HealthCheck,
    
    /// Health check response
    HealthCheckResponse { healthy: bool },
}

/// Message envelope with routing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// Unique message ID
    pub id: Uuid,
    
    /// Sender component
    pub sender: String,
    
    /// Target component (None for broadcast)
    pub target: Option<String>,
    
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    
    /// The actual message
    pub message: DamMessage,
    
    /// Request/response correlation ID
    pub correlation_id: Option<Uuid>,
    
    /// Message priority
    pub priority: MessagePriority,
}

/// Message priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

impl MessageEnvelope {
    /// Create a new message envelope
    pub fn new(sender: String, message: DamMessage) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender,
            target: None,
            timestamp: Utc::now(),
            message,
            correlation_id: None,
            priority: MessagePriority::Normal,
        }
    }
    
    /// Set the target component
    pub fn to(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }
    
    /// Set correlation ID for request/response
    pub fn correlate_with(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    /// Set message priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
}
