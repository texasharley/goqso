//! Tauri Command Handlers
//!
//! This module organizes all Tauri IPC commands into logical submodules.
//! Each submodule handles a specific domain of functionality.
//!
//! ## Module Organization
//! - `state` - Application state and shared types
//! - `time_utils` - Time parsing, normalization, and validation utilities
//! - `udp` - WSJT-X UDP listener commands
//! - `qso` - QSO CRUD operations and history
//! - `adif` - ADIF import/export
//! - `lotw` - LoTW sync (download/upload)
//! - `awards` - Award progress (DXCC, WAS, VUCC)
//! - `settings` - Application settings
//! - `band_activity` - Band activity storage and retrieval
//! - `fcc` - FCC database commands
//! - `diagnostics` - Debug and diagnostic commands

mod state;
pub mod time_utils;
pub mod udp;
pub mod qso;
pub mod adif;
pub mod lotw;
pub mod awards;
pub mod settings;
pub mod band_activity;
pub mod fcc;
pub mod diagnostics;

// Re-export AppState for use in main.rs
pub use state::AppState;
