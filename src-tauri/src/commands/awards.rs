//! Award progress tracking commands
//!
//! This module handles:
//! - get_dxcc_progress: DXCC worked/confirmed counts
//! - get_was_progress: WAS (Worked All States) progress

use serde::Serialize;
use tauri::command;

use super::state::AppState;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct DxccProgress {
    pub worked: i64,
    pub confirmed: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct WasProgress {
    pub worked: i64,
    pub confirmed: i64,
    pub total: i64,
    pub worked_states: Vec<String>,
    pub confirmed_states: Vec<String>,
}

// ============================================================================
// Award Commands
// ============================================================================

#[command]
pub async fn get_dxcc_progress(
    state: tauri::State<'_, AppState>,
    band: Option<String>,
    mode: Option<String>,
) -> Result<DxccProgress, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    // Count unique worked DXCC entities
    let worked: i64 = if let (Some(b), Some(m)) = (&band, &mode) {
        sqlx::query_scalar(
            "SELECT COUNT(DISTINCT dxcc) FROM qsos WHERE dxcc IS NOT NULL AND band = ? AND mode = ?",
        )
        .bind(b)
        .bind(m)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
    } else if let Some(b) = &band {
        sqlx::query_scalar(
            "SELECT COUNT(DISTINCT dxcc) FROM qsos WHERE dxcc IS NOT NULL AND band = ?",
        )
        .bind(b)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
    } else if let Some(m) = &mode {
        sqlx::query_scalar(
            "SELECT COUNT(DISTINCT dxcc) FROM qsos WHERE dxcc IS NOT NULL AND mode = ?",
        )
        .bind(m)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
    } else {
        sqlx::query_scalar("SELECT COUNT(DISTINCT dxcc) FROM qsos WHERE dxcc IS NOT NULL")
            .fetch_one(pool)
            .await
            .unwrap_or(0)
    };

    // Count confirmed DXCC entities
    let confirmed: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(DISTINCT q.dxcc) FROM qsos q
           JOIN confirmations c ON c.qso_id = q.id
           WHERE q.dxcc IS NOT NULL AND c.source = 'LOTW' AND c.qsl_rcvd = 'Y'"#,
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    Ok(DxccProgress {
        worked,
        confirmed,
        total: 340, // Current active DXCC entities
    })
}

#[command]
pub async fn get_was_progress(
    state: tauri::State<'_, AppState>,
    band: Option<String>,
    mode: Option<String>,
) -> Result<WasProgress, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    // DXCC entities that count for WAS (Worked All States):
    // - 291: United States of America (continental)
    // - 6: Alaska (separate DXCC entity, but state AK counts for WAS)
    // - 110: Hawaii (separate DXCC entity, but state HI counts for WAS)
    
    // Get unique worked US states (including Alaska and Hawaii)
    let worked_states: Vec<(String,)> = if let (Some(b), Some(m)) = (&band, &mode) {
        sqlx::query_as(
            "SELECT DISTINCT state FROM qsos WHERE dxcc IN (291, 6, 110) AND state IS NOT NULL AND band = ? AND mode = ?",
        )
        .bind(b)
        .bind(m)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as("SELECT DISTINCT state FROM qsos WHERE dxcc IN (291, 6, 110) AND state IS NOT NULL")
            .fetch_all(pool)
            .await
            .unwrap_or_default()
    };

    // Get confirmed states (including Alaska and Hawaii)
    let confirmed_states: Vec<(String,)> = sqlx::query_as(
        r#"SELECT DISTINCT q.state FROM qsos q
           JOIN confirmations c ON c.qso_id = q.id
           WHERE q.dxcc IN (291, 6, 110) AND q.state IS NOT NULL AND c.source = 'LOTW' AND c.qsl_rcvd = 'Y'"#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Ok(WasProgress {
        worked: worked_states.len() as i64,
        confirmed: confirmed_states.len() as i64,
        total: 50,
        worked_states: worked_states.into_iter().map(|(s,)| s).collect(),
        confirmed_states: confirmed_states.into_iter().map(|(s,)| s).collect(),
    })
}
