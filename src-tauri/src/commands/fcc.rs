//! FCC database commands
//!
//! This module handles:
//! - get_fcc_sync_status: Check FCC database sync status
//! - sync_fcc_database: Download and import FCC database
//! - lookup_fcc_callsign: Single callsign lookup
//! - lookup_fcc_callsigns: Batch callsign lookup

use tauri::{command, Emitter, Manager};

use super::state::AppState;
use crate::fcc::{FccLicenseInfo, FccSyncStatus};

// ============================================================================
// FCC Commands
// ============================================================================

/// Get FCC database sync status
#[command]
pub async fn get_fcc_sync_status(
    state: tauri::State<'_, AppState>,
) -> Result<FccSyncStatus, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    crate::fcc::get_sync_status(pool).await
}

/// Sync (download and import) the FCC amateur license database
#[command]
pub async fn sync_fcc_database(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<FccSyncStatus, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    // Mark sync as in progress
    sqlx::query("UPDATE fcc_sync_status SET sync_in_progress = 1, error_message = NULL WHERE id = 1")
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update sync status: {}", e))?;

    // Emit progress event
    let _ = app.emit("fcc-sync-progress", "Starting FCC database download...");

    // Get app data directory
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    // Download the database
    let _ = app.emit("fcc-sync-progress", "Downloading FCC database (~25MB)...");

    let en_path = match crate::fcc::download_fcc_database(&data_dir).await {
        Ok(path) => path,
        Err(e) => {
            let _ = sqlx::query(
                "UPDATE fcc_sync_status SET sync_in_progress = 0, error_message = ? WHERE id = 1",
            )
            .bind(&e)
            .execute(pool)
            .await;

            return Err(e);
        }
    };

    // Parse and import
    let _ = app.emit(
        "fcc-sync-progress",
        "Importing FCC records into database...",
    );

    let record_count = match crate::fcc::parse_fcc_database(&en_path, pool).await {
        Ok(count) => count,
        Err(e) => {
            let _ = sqlx::query(
                "UPDATE fcc_sync_status SET sync_in_progress = 0, error_message = ? WHERE id = 1",
            )
            .bind(&e)
            .execute(pool)
            .await;

            return Err(e);
        }
    };

    // Update success status
    sqlx::query(
        r#"UPDATE fcc_sync_status SET 
           sync_in_progress = 0, 
           last_sync_at = datetime('now'),
           record_count = ?,
           error_message = NULL
           WHERE id = 1"#,
    )
    .bind(record_count as i64)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update sync status: {}", e))?;

    let _ = app.emit(
        "fcc-sync-progress",
        format!("Imported {} FCC records", record_count),
    );

    // Return updated status
    crate::fcc::get_sync_status(pool).await
}

/// Lookup a single callsign in the FCC database
#[command]
pub async fn lookup_fcc_callsign(
    state: tauri::State<'_, AppState>,
    callsign: String,
) -> Result<Option<FccLicenseInfo>, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    Ok(crate::fcc::lookup_callsign(pool, &callsign).await)
}

/// Lookup multiple callsigns in the FCC database (batch)
#[command]
pub async fn lookup_fcc_callsigns(
    state: tauri::State<'_, AppState>,
    callsigns: Vec<String>,
) -> Result<Vec<FccLicenseInfo>, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    Ok(crate::fcc::lookup_callsigns(pool, &callsigns).await)
}
