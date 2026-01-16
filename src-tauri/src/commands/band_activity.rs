//! Band activity storage and retrieval
//!
//! This module handles:
//! - save_band_activity: Store TX/RX messages from WSJT-X
//! - get_recent_activity: Retrieve recent band activity
//! - prune_band_activity: Clean up old messages

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tauri::command;

use super::state::AppState;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandActivityMessage {
    pub id: i64,
    pub time_utc: String,
    pub time_ms: Option<i64>,
    pub direction: String,
    pub message: String,
    pub snr: Option<i32>,
    pub delta_freq: Option<i32>,
    pub de_call: Option<String>,
    pub dx_call: Option<String>,
    pub dial_freq: Option<f64>,
    pub mode: Option<String>,
}

// ============================================================================
// Internal Functions
// ============================================================================

/// Save a band activity message (TX or RX)
/// This is called internally by the UDP listener, not exposed as a command
pub async fn save_band_activity(
    pool: &Pool<Sqlite>,
    time_utc: &str,
    time_ms: Option<i64>,
    direction: &str,
    message: &str,
    snr: Option<i32>,
    delta_freq: Option<i32>,
    de_call: Option<&str>,
    dx_call: Option<&str>,
    dial_freq: Option<f64>,
    mode: Option<&str>,
) -> Result<(), String> {
    sqlx::query(
        r#"INSERT INTO band_activity 
           (time_utc, time_ms, direction, message, snr, delta_freq, de_call, dx_call, dial_freq, mode)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(time_utc)
    .bind(time_ms)
    .bind(direction)
    .bind(message)
    .bind(snr)
    .bind(delta_freq)
    .bind(de_call)
    .bind(dx_call)
    .bind(dial_freq)
    .bind(mode)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to save band activity: {}", e))?;

    Ok(())
}

// ============================================================================
// Commands
// ============================================================================

/// Get recent band activity messages
#[command]
pub async fn get_recent_activity(
    state: tauri::State<'_, AppState>,
    minutes: Option<i32>,
) -> Result<Vec<BandActivityMessage>, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    let mins = minutes.unwrap_or(60);

    let rows: Vec<(
        i64,
        String,
        Option<i64>,
        String,
        String,
        Option<i32>,
        Option<i32>,
        Option<String>,
        Option<String>,
        Option<f64>,
        Option<String>,
    )> = sqlx::query_as(
        r#"SELECT id, time_utc, time_ms, direction, message, snr, delta_freq, de_call, dx_call, dial_freq, mode
           FROM band_activity
           WHERE created_at > datetime('now', ? || ' minutes')
           ORDER BY created_at ASC"#,
    )
    .bind(format!("-{}", mins))
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get band activity: {}", e))?;

    Ok(rows
        .into_iter()
        .map(
            |(
                id,
                time_utc,
                time_ms,
                direction,
                message,
                snr,
                delta_freq,
                de_call,
                dx_call,
                dial_freq,
                mode,
            )| {
                BandActivityMessage {
                    id,
                    time_utc,
                    time_ms,
                    direction,
                    message,
                    snr,
                    delta_freq,
                    de_call,
                    dx_call,
                    dial_freq,
                    mode,
                }
            },
        )
        .collect())
}

/// Clear old band activity messages (older than specified minutes)
#[command]
pub async fn prune_band_activity(
    state: tauri::State<'_, AppState>,
    older_than_minutes: Option<i32>,
) -> Result<i64, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    let mins = older_than_minutes.unwrap_or(60);

    let result = sqlx::query(
        r#"DELETE FROM band_activity WHERE created_at < datetime('now', ? || ' minutes')"#,
    )
    .bind(format!("-{}", mins))
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to prune band activity: {}", e))?;

    Ok(result.rows_affected() as i64)
}
