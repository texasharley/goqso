//! Settings and utility commands
//!
//! This module handles:
//! - get_setting / set_setting: Key-value settings storage
//! - is_db_ready: Check if database is initialized
//! - get_db_stats: Database statistics
//! - lookup_callsign: Callsign information lookup

use serde::{Deserialize, Serialize};
use tauri::command;

use super::state::AppState;
use crate::db::DbStats;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CallsignInfo {
    pub call: String,
    pub dxcc: Option<i32>,
    pub entity_name: Option<String>,
    pub cq_zone: Option<i32>,
    pub itu_zone: Option<i32>,
    pub continent: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

// ============================================================================
// Settings Commands
// ============================================================================

#[command]
pub async fn get_setting(
    state: tauri::State<'_, AppState>,
    key: String,
) -> Result<Option<String>, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    let result = sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ?")
        .bind(&key)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result)
}

#[command]
pub async fn set_setting(
    state: tauri::State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    log::info!(
        "Setting {} = {}",
        key,
        if key.contains("password") {
            "***"
        } else {
            &value
        }
    );

    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    sqlx::query(
        r#"INSERT INTO settings (key, value, updated_at) 
           VALUES (?, ?, datetime('now'))
           ON CONFLICT(key) DO UPDATE SET 
             value = excluded.value,
             updated_at = datetime('now')"#,
    )
    .bind(&key)
    .bind(&value)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// Database Commands
// ============================================================================

#[command]
pub async fn is_db_ready(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let db_guard = state.db.lock().await;
    Ok(db_guard.is_some())
}

#[command]
pub async fn get_db_stats(state: tauri::State<'_, AppState>) -> Result<DbStats, String> {
    let db_guard = state.db.lock().await;
    match db_guard.as_ref() {
        Some(pool) => crate::db::get_db_stats(pool).await,
        None => Err("Database not initialized".to_string()),
    }
}

// ============================================================================
// Callsign Lookup
// ============================================================================

#[command]
pub async fn lookup_callsign(call: String) -> Result<CallsignInfo, String> {
    log::info!("Looking up callsign: {}", call);

    // Use our reference module to look up DXCC info
    let lookup = crate::reference::lookup_call_full(&call);

    Ok(CallsignInfo {
        call,
        dxcc: lookup.dxcc_as_i32(),
        entity_name: lookup.country,
        cq_zone: lookup.cqz,
        itu_zone: lookup.ituz,
        continent: lookup.continent,
        latitude: None,
        longitude: None,
    })
}
