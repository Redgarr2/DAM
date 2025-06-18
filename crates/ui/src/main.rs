//! Digital Asset Manager - Desktop Application
//! 
//! A powerful offline digital asset management system built with Tauri.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;
use tracing::{info, error};
use std::sync::Arc;
use tokio::sync::Mutex;

mod app;
mod commands;
mod error;
mod state;

use app::DamApp;
use error::UiError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Digital Asset Manager");
    
    // Create application state
    let app_state = Arc::new(Mutex::new(DamApp::new().await?));
    
    // Build Tauri application
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::search::search_assets,
            commands::search::search_similar,
            commands::assets::get_asset_details,
            commands::assets::import_file,
            commands::assets::import_directory,
            commands::library::get_library_stats,
            commands::library::scan_library,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .setup(|_app| {
            info!("Tauri application setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
    Ok(())
}
