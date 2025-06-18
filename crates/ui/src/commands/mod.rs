//! Tauri command handlers
//! 
//! These functions are called from the frontend JavaScript/TypeScript code
//! and provide the interface between the UI and the Rust backend.

pub mod search;
pub mod assets;
pub mod library;
pub mod settings;

use crate::error::UiResult;
use serde::{Deserialize, Serialize};

/// Standard response wrapper for commands
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> CommandResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

impl<T> From<UiResult<T>> for CommandResponse<T> {
    fn from(result: UiResult<T>) -> Self {
        match result {
            Ok(data) => CommandResponse::success(data),
            Err(error) => CommandResponse::error(error.to_string()),
        }
    }
}
