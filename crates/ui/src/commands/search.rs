//! Search command handlers

use crate::app::DamApp;
use crate::commands::CommandResponse;
use index::SearchResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimilarSearchRequest {
    pub asset_id: String,
    pub limit: Option<usize>,
}

/// Search for assets by text query
#[tauri::command]
pub async fn search_assets(
    request: SearchRequest,
    app_state: State<'_, Arc<Mutex<DamApp>>>,
) -> Result<CommandResponse<Vec<SearchResult>>, String> {
    let app = app_state.lock().await;
    let limit = request.limit.unwrap_or(50);
    
    let result = app.search_assets(&request.query, limit).await;
    Ok(result.into())
}

/// Find assets similar to a given asset
#[tauri::command]
pub async fn search_similar(
    request: SimilarSearchRequest,
    app_state: State<'_, Arc<Mutex<DamApp>>>,
) -> Result<CommandResponse<Vec<SearchResult>>, String> {
    let app = app_state.lock().await;
    let limit = request.limit.unwrap_or(10);
    
    // Parse UUID
    let asset_id = match Uuid::parse_str(&request.asset_id) {
        Ok(id) => id,
        Err(_) => return Ok(CommandResponse::error("Invalid asset ID".to_string())),
    };
    
    let result = app.find_similar(asset_id, limit).await;
    Ok(result.into())
}
