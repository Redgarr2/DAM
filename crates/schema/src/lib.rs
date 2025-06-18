//! Shared data types and schemas for the DAM system
//! 
//! This crate defines all the core data structures used throughout the DAM system,
//! including asset metadata, search results, and inter-process communication messages.

pub mod asset;
pub mod search;
pub mod ipc;
pub mod error;
pub mod models;

pub use asset::*;
pub use search::*;
pub use ipc::*;
pub use error::*;
pub use models::*;
