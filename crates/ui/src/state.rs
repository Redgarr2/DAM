//! Application state management for Tauri

use crate::app::DamApp;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Type alias for the shared application state
pub type AppState = Arc<Mutex<DamApp>>;

/// Initialize the application state
pub async fn init_app_state() -> Result<AppState, Box<dyn std::error::Error>> {
    let app = DamApp::new().await?;
    Ok(Arc::new(Mutex::new(app)))
}
