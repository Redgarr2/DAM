//! Asset management command handlers

use crate::app::DamApp;
use crate::commands::CommandResponse;
use schema::Asset;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportFileRequest {
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportDirectoryRequest {
    pub directory_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetDetailsRequest {
    pub asset_id: String,
}

/// Get detailed information about an asset
#[tauri::command]
pub async fn get_asset_details(
    request: AssetDetailsRequest,
    app_state: State<'_, Arc<Mutex<DamApp>>>,
) -> Result<CommandResponse<Option<Asset>>, String> {
    let app = app_state.lock().await;
    
    // Parse UUID
    let asset_id = match Uuid::parse_str(&request.asset_id) {
        Ok(id) => id,
        Err(_) => return Ok(CommandResponse::error("Invalid asset ID".to_string())),
    };
    
    // For now, we'll need to search for the asset since we don't have direct lookup
    // This could be optimized later with a direct asset lookup method
    let search_results = match app.search_assets("", 1000).await {
        Ok(results) => results,
        Err(e) => return Ok(CommandResponse::error(e.to_string())),
    };
    
    let asset = search_results
        .into_iter()
        .find(|result| result.document.asset_id == asset_id)
        .map(|result| {
            // Convert AssetDocument back to Asset
            // This is a simplified conversion for now
            Asset {
                id: result.document.asset_id,
                original_path: result.document.file_path.clone(),
                current_path: result.document.file_path,
                asset_type: result.document.asset_type,
                file_size: result.document.file_size,
                format: schema::FileFormat {
                    extension: result.document.filename
                        .split('.')
                        .last()
                        .unwrap_or("unknown")
                        .to_string(),
                    mime_type: None,
                    version: None,
                    supported: true,
                },
                created_at: result.document.created_at,
                modified_at: result.document.modified_at,
                tags: result.document.tags,
                metadata: schema::AssetMetadata::default(), // TODO: Reconstruct from document
                preview: result.document.preview_path.map(|path| schema::PreviewInfo {
                    thumbnail_path: path.clone(),
                    thumbnail_size: (256, 256), // Default thumbnail size
                    rendered_preview: Some(path),
                    generated_at: result.document.indexed_at,
                }),
                embedding: result.document.visual_embedding,
                version_info: schema::VersionInfo {
                    current_version: "v1".to_string(),
                    version_count: 1,
                    last_snapshot: result.document.created_at,
                    has_changes: false,
                },
            }
        });
    
    Ok(CommandResponse::success(asset))
}

/// Import a single file
#[tauri::command]
pub async fn import_file(
    request: ImportFileRequest,
    app_state: State<'_, Arc<Mutex<DamApp>>>,
) -> Result<CommandResponse<Asset>, String> {
    let mut app = app_state.lock().await;
    let file_path = PathBuf::from(request.file_path);
    
    let result = app.import_file(file_path).await;
    Ok(result.into())
}

/// Import all files in a directory
#[tauri::command]
pub async fn import_directory(
    request: ImportDirectoryRequest,
    app_state: State<'_, Arc<Mutex<DamApp>>>,
) -> Result<CommandResponse<Vec<Asset>>, String> {
    let mut app = app_state.lock().await;
    let directory_path = PathBuf::from(request.directory_path);
    
    let result = app.import_directory(directory_path).await;
    Ok(result.into())
}
