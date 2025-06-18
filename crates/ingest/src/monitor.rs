//! File system monitoring for automatic asset ingestion
//! 
//! This module watches directories for file changes and automatically
//! triggers ingestion of new or modified assets.

use schema::DamResult;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use crate::{IngestService, error::IngestError};

/// Events emitted by the file system monitor
#[derive(Debug, Clone)]
pub enum MonitorEvent {
    /// A new file was created
    FileCreated { path: PathBuf },
    
    /// An existing file was modified
    FileModified { path: PathBuf },
    
    /// A file was deleted
    FileDeleted { path: PathBuf },
    
    /// A file was moved/renamed
    FileMoved { from: PathBuf, to: PathBuf },
    
    /// Monitoring error occurred
    Error { message: String },
}

/// File system monitor service
pub struct FileSystemMonitor {
    /// The file system watcher
    watcher: Option<RecommendedWatcher>,
    
    /// Channel for receiving file system events
    event_receiver: Option<mpsc::Receiver<MonitorEvent>>,
    
    /// Ingestion service for processing detected files
    ingest_service: Arc<IngestService>,
    
    /// Paths being monitored
    monitored_paths: Vec<PathBuf>,
    
    /// Whether to automatically ingest detected files
    auto_ingest: bool,
}

impl FileSystemMonitor {
    /// Create a new file system monitor
    pub fn new(ingest_service: Arc<IngestService>) -> DamResult<Self> {
        Ok(Self {
            watcher: None,
            event_receiver: None,
            ingest_service,
            monitored_paths: Vec::new(),
            auto_ingest: true,
        })
    }
    
    /// Start monitoring a directory
    pub async fn start_monitoring<P: AsRef<Path>>(&mut self, path: P) -> DamResult<()> {
        let path = path.as_ref().to_path_buf();
        
        if !path.exists() {
            return Err(IngestError::file_not_found(path).into());
        }
        
        if !path.is_dir() {
            return Err(IngestError::not_a_directory(path).into());
        }
        
        info!("Starting file system monitoring for: {}", path.display());
        
        // Create event channel
        let (event_sender, event_receiver) = mpsc::channel(1000);
        
        // Create file system watcher
        let mut watcher = notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
            match result {
                Ok(event) => {
                    if let Some(monitor_event) = Self::convert_notify_event(event) {
                        if let Err(e) = event_sender.try_send(monitor_event) {
                            warn!("Failed to send monitor event: {}", e);
                        }
                    }
                }
                Err(e) => {
                    let error_event = MonitorEvent::Error {
                        message: format!("File system watch error: {}", e),
                    };
                    if let Err(send_err) = event_sender.try_send(error_event) {
                        error!("Failed to send error event: {}", send_err);
                    }
                }
            }
        }).map_err(|e| IngestError::monitoring_error(format!("Failed to create watcher: {}", e)))?;
        
        // Start watching the directory
        watcher.watch(&path, RecursiveMode::Recursive)
            .map_err(|e| IngestError::monitoring_error(format!("Failed to watch directory: {}", e)))?;
        
        self.watcher = Some(watcher);
        self.event_receiver = Some(event_receiver);
        self.monitored_paths.push(path.clone());
        
        info!("File system monitoring started for: {}", path.display());
        Ok(())
    }
    
    /// Stop monitoring all directories
    pub async fn stop_monitoring(&mut self) -> DamResult<()> {
        info!("Stopping file system monitoring");
        
        self.watcher = None;
        self.event_receiver = None;
        self.monitored_paths.clear();
        
        info!("File system monitoring stopped");
        Ok(())
    }
    
    /// Process file system events (call this in a loop)
    pub async fn process_events(&mut self) -> DamResult<Vec<MonitorEvent>> {
        let mut events = Vec::new();
        
        // Collect all events first to avoid borrow conflicts
        if let Some(receiver) = &mut self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                debug!("Received monitor event: {:?}", event);
                events.push(event);
            }
        }
        
        // Then handle each event
        for event in &events {
            if let Err(e) = self.handle_event(event).await {
                warn!("Failed to handle monitor event: {}", e);
            }
        }
        
        Ok(events)
    }
    
    /// Wait for and process a single event
    pub async fn wait_for_event(&mut self) -> DamResult<Option<MonitorEvent>> {
        if let Some(receiver) = &mut self.event_receiver {
            match receiver.recv().await {
                Some(event) => {
                    debug!("Received monitor event: {:?}", event);
                    
                    if let Err(e) = self.handle_event(&event).await {
                        warn!("Failed to handle monitor event: {}", e);
                    }
                    
                    Ok(Some(event))
                }
                None => Ok(None), // Channel closed
            }
        } else {
            Ok(None)
        }
    }
    
    /// Handle a monitor event
    async fn handle_event(&self, event: &MonitorEvent) -> DamResult<()> {
        match event {
            MonitorEvent::FileCreated { path } => {
                if self.auto_ingest && self.should_ingest_file(path) {
                    self.auto_ingest_file(path).await?;
                }
            }
            MonitorEvent::FileModified { path } => {
                if self.auto_ingest && self.should_ingest_file(path) {
                    // For modified files, we might want to update the existing asset
                    self.auto_ingest_file(path).await?;
                }
            }
            MonitorEvent::FileMoved { from: _, to } => {
                if self.auto_ingest && self.should_ingest_file(to) {
                    self.auto_ingest_file(to).await?;
                }
            }
            MonitorEvent::FileDeleted { path: _ } => {
                // File deletion would be handled by the main asset management system
                debug!("File deleted, asset cleanup should be handled externally");
            }
            MonitorEvent::Error { message } => {
                error!("Monitor error: {}", message);
            }
        }
        
        Ok(())
    }
    
    /// Automatically ingest a detected file
    async fn auto_ingest_file(&self, path: &Path) -> DamResult<()> {
        info!("Auto-ingesting detected file: {}", path.display());
        
        match self.ingest_service.ingest_file(path).await {
            Ok(asset) => {
                info!("Successfully auto-ingested: {} (ID: {})", 
                      path.display(), asset.id);
            }
            Err(e) => {
                warn!("Failed to auto-ingest {}: {}", path.display(), e);
            }
        }
        
        Ok(())
    }
    
    /// Check if a file should be automatically ingested
    fn should_ingest_file(&self, path: &Path) -> bool {
        // Skip directories
        if path.is_dir() {
            return false;
        }
        
        // Use the ingest service's filtering logic
        self.ingest_service.should_ingest(path)
    }
    
    /// Convert notify event to our monitor event
    fn convert_notify_event(event: Event) -> Option<MonitorEvent> {
        match event.kind {
            EventKind::Create(_) => {
                if let Some(path) = event.paths.first() {
                    Some(MonitorEvent::FileCreated {
                        path: path.clone(),
                    })
                } else {
                    None
                }
            }
            EventKind::Modify(_) => {
                if let Some(path) = event.paths.first() {
                    Some(MonitorEvent::FileModified {
                        path: path.clone(),
                    })
                } else {
                    None
                }
            }
            EventKind::Remove(_) => {
                if let Some(path) = event.paths.first() {
                    Some(MonitorEvent::FileDeleted {
                        path: path.clone(),
                    })
                } else {
                    None
                }
            }
            EventKind::Access(_) => {
                // Ignore access events to reduce noise
                None
            }
            EventKind::Other => None,
            _ => None,
        }
    }
    
    /// Set whether to automatically ingest detected files
    pub fn set_auto_ingest(&mut self, auto_ingest: bool) {
        self.auto_ingest = auto_ingest;
        info!("Auto-ingest set to: {}", auto_ingest);
    }
    
    /// Get the list of monitored paths
    pub fn monitored_paths(&self) -> &[PathBuf] {
        &self.monitored_paths
    }
    
    /// Check if monitoring is active
    pub fn is_monitoring(&self) -> bool {
        self.watcher.is_some()
    }
}

impl Drop for FileSystemMonitor {
    fn drop(&mut self) {
        if self.is_monitoring() {
            info!("Dropping FileSystemMonitor, stopping monitoring");
            // The watcher will be dropped automatically
        }
    }
}

/// Builder for configuring file system monitoring
pub struct MonitorBuilder {
    paths: Vec<PathBuf>,
    auto_ingest: bool,
    recursive: bool,
}

impl MonitorBuilder {
    /// Create a new monitor builder
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            auto_ingest: true,
            recursive: true,
        }
    }
    
    /// Add a path to monitor
    pub fn add_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.paths.push(path.into());
        self
    }
    
    /// Set whether to automatically ingest detected files
    pub fn auto_ingest(mut self, auto_ingest: bool) -> Self {
        self.auto_ingest = auto_ingest;
        self
    }
    
    /// Set whether to monitor recursively
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }
    
    /// Build the file system monitor
    pub fn build(self, ingest_service: Arc<IngestService>) -> DamResult<FileSystemMonitor> {
        let mut monitor = FileSystemMonitor::new(ingest_service)?;
        monitor.set_auto_ingest(self.auto_ingest);
        Ok(monitor)
    }
}

impl Default for MonitorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_monitor_creation() {
        let ingest_service = Arc::new(IngestService::new().unwrap());
        let monitor = FileSystemMonitor::new(ingest_service);
        assert!(monitor.is_ok());
    }
    
    #[tokio::test]
    async fn test_monitor_builder() {
        let ingest_service = Arc::new(IngestService::new().unwrap());
        let dir = tempdir().unwrap();
        
        let monitor = MonitorBuilder::new()
            .add_path(dir.path())
            .auto_ingest(false)
            .build(ingest_service);
        
        assert!(monitor.is_ok());
        let monitor = monitor.unwrap();
        assert!(!monitor.auto_ingest);
    }
    
    #[tokio::test]
    async fn test_monitor_start_stop() {
        let ingest_service = Arc::new(IngestService::new().unwrap());
        let dir = tempdir().unwrap();
        let mut monitor = FileSystemMonitor::new(ingest_service).unwrap();
        
        // Start monitoring
        let result = monitor.start_monitoring(dir.path()).await;
        assert!(result.is_ok());
        assert!(monitor.is_monitoring());
        assert_eq!(monitor.monitored_paths().len(), 1);
        
        // Stop monitoring
        let result = monitor.stop_monitoring().await;
        assert!(result.is_ok());
        assert!(!monitor.is_monitoring());
        assert_eq!(monitor.monitored_paths().len(), 0);
    }
    
    #[test]
    fn test_should_ingest_file() {
        let ingest_service = Arc::new(IngestService::new().unwrap());
        let monitor = FileSystemMonitor::new(ingest_service).unwrap();
        
        assert!(monitor.should_ingest_file(Path::new("test.png")));
        assert!(monitor.should_ingest_file(Path::new("model.blend")));
        assert!(!monitor.should_ingest_file(Path::new("temp.tmp")));
        assert!(!monitor.should_ingest_file(Path::new(".hidden")));
    }
    
    #[test]
    fn test_event_conversion() {
        use notify::{Event, EventKind};
        
        let create_event = Event {
            kind: EventKind::Create(notify::event::CreateKind::File),
            paths: vec![PathBuf::from("test.txt")],
            attrs: Default::default(),
        };
        
        let monitor_event = FileSystemMonitor::convert_notify_event(create_event);
        assert!(matches!(monitor_event, Some(MonitorEvent::FileCreated { .. })));
        
        let modify_event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content
            )),
            paths: vec![PathBuf::from("test.txt")],
            attrs: Default::default(),
        };
        
        let monitor_event = FileSystemMonitor::convert_notify_event(modify_event);
        assert!(matches!(monitor_event, Some(MonitorEvent::FileModified { .. })));
    }
}
