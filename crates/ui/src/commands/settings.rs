//! Settings management command handlers

use crate::app::{AppSettings, DamApp};
use crate::commands::CommandResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSettingsRequest {
    pub settings: AppSettings,
}

/// Get current application settings
#[tauri::command]
pub async fn get_settings(
    app_state: State<'_, Arc<Mutex<DamApp>>>,
) -> Result<CommandResponse<AppSettings>, String> {
    let app = app_state.lock().await;
    let settings = app.settings.clone();
    Ok(CommandResponse::success(settings))
}

/// Update application settings
#[tauri::command]
pub async fn update_settings(
    request: UpdateSettingsRequest,
    app_state: State<'_, Arc<Mutex<DamApp>>>,
) -> Result<CommandResponse<()>, String> {
    let mut app = app_state.lock().await;
    let result = app.update_settings(request.settings).await;
    Ok(result.into())
}
