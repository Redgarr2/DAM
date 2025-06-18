//! Library management command handlers

use crate::app::{DamApp, LibraryStats};
use crate::commands::CommandResponse;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanLibraryRequest {
    pub library_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryStatsResponse {
    pub stats: LibraryStats,
    pub library_path: Option<String>,
}

/// Get current library statistics
#[tauri::command]
pub async fn get_library_stats(
    app_state: State<'_, Arc<Mutex<DamApp>>>,
) -> Result<CommandResponse<LibraryStatsResponse>, String> {
    let app = app_state.lock().await;
    
    let stats = app.get_library_stats();
    let library_path = app.library_path.as_ref().map(|p| p.to_string_lossy().to_string());
    
    let response = LibraryStatsResponse {
        stats,
        library_path,
    };
    
    Ok(CommandResponse::success(response))
}

/// Scan and import all assets from a library directory
#[tauri::command]
pub async fn scan_library(
    request: ScanLibraryRequest,
    app_state: State<'_, Arc<Mutex<DamApp>>>,
) -> Result<CommandResponse<usize>, String> {
    let mut app = app_state.lock().await;
    let library_path = PathBuf::from(request.library_path);
    
    // Set as current library
    app.library_path = Some(library_path.clone());
    
    // Import all assets from the directory
    let result = app.import_directory(library_path).await;
    
    match result {
        Ok(assets) => Ok(CommandResponse::success(assets.len())),
        Err(e) => Ok(CommandResponse::error(e.to_string())),
    }
}
