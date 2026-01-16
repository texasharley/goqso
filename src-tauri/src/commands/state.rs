//! Application State
//!
//! Shared state managed by Tauri, accessible to all command handlers.

use std::sync::Arc;
use sqlx::{Pool, Sqlite};
use tokio::sync::Mutex as TokioMutex;

use crate::udp::UdpListenerState;

/// Application state holding the database connection pool and UDP listener
pub struct AppState {
    pub db: Arc<TokioMutex<Option<Pool<Sqlite>>>>,
    pub udp_state: Arc<UdpListenerState>,
}
