//! User interface and desktop application
//! 
//! This crate provides a Tauri-based desktop application for the Digital Asset Manager.
//! It integrates all the core functionality (ingestion, processing, indexing) into a
//! user-friendly interface.

pub mod app;
pub mod commands;
pub mod error;
pub mod state;

pub use app::{DamApp, AppSettings, LibraryStats};
pub use error::{UiError, UiResult};
pub use state::{AppState, init_app_state};
