// FCC Amateur License Database Module
//
// Downloads and imports the FCC ULS amateur license database for offline
// callsign lookups. Essential for POTA/portable operations without internet.
//
// Data source: https://data.fcc.gov/download/pub/uls/complete/l_amat.zip
// Update frequency: Weekly (automatic background sync on app startup)

mod download;
mod parser;

pub use download::download_fcc_database;
pub use parser::{parse_fcc_database, FccLicense};

use sqlx::SqlitePool;
use serde::Serialize;
use tauri::Manager;

/// Check if FCC sync is needed and run it silently in the background
/// Syncs if: never synced, or last sync > 7 days ago
pub async fn sync_fcc_if_needed(app: &tauri::AppHandle) {
    let state = app.state::<crate::commands::AppState>();
    let db_guard = state.db.lock().await;
    let pool = match db_guard.as_ref() {
        Some(p) => p,
        None => {
            log::warn!("FCC sync: database not ready");
            return;
        }
    };
    
    // Check if sync is needed
    let needs_sync = match get_sync_status(pool).await {
        Ok(status) => {
            if status.record_count == 0 {
                log::info!("FCC database empty, will sync");
                true
            } else if let Some(last_sync) = status.last_sync_at {
                // Parse the timestamp and check if > 7 days old
                match chrono::NaiveDateTime::parse_from_str(&last_sync, "%Y-%m-%d %H:%M:%S") {
                    Ok(dt) => {
                        let age = chrono::Utc::now().naive_utc() - dt;
                        if age.num_days() > 7 {
                            log::info!("FCC database is {} days old, will sync", age.num_days());
                            true
                        } else {
                            log::debug!("FCC database is {} days old, no sync needed", age.num_days());
                            false
                        }
                    }
                    Err(_) => {
                        log::warn!("Failed to parse FCC sync timestamp, will sync");
                        true
                    }
                }
            } else {
                log::info!("FCC database has no sync timestamp, will sync");
                true
            }
        }
        Err(e) => {
            log::warn!("Failed to get FCC sync status: {}, will sync", e);
            true
        }
    };
    
    if !needs_sync {
        return;
    }
    
    // Release the lock before the long-running sync
    drop(db_guard);
    
    log::info!("Starting background FCC database sync...");
    
    // Get app data directory
    let data_dir = match app.path().app_data_dir() {
        Ok(d) => d,
        Err(e) => {
            log::error!("FCC sync: failed to get app data dir: {}", e);
            return;
        }
    };
    
    // Download the database
    let en_path = match download_fcc_database(&data_dir).await {
        Ok(path) => path,
        Err(e) => {
            log::error!("FCC sync: download failed: {}", e);
            return;
        }
    };
    
    // Re-acquire the lock for import
    let db_guard = state.db.lock().await;
    let pool = match db_guard.as_ref() {
        Some(p) => p,
        None => {
            log::error!("FCC sync: database disappeared during download");
            return;
        }
    };
    
    // Parse and import
    let record_count = match parse_fcc_database(&en_path, pool).await {
        Ok(count) => count,
        Err(e) => {
            log::error!("FCC sync: import failed: {}", e);
            return;
        }
    };
    
    // Update sync status
    let _ = sqlx::query(
        r#"UPDATE fcc_sync_status SET 
           sync_in_progress = 0, 
           last_sync_at = datetime('now'),
           record_count = ?,
           error_message = NULL
           WHERE id = 1"#
    )
    .bind(record_count as i64)
    .execute(pool)
    .await;
    
    log::info!("FCC background sync complete: {} records imported", record_count);
}

/// FCC sync status
#[derive(Debug, Serialize, Clone)]
pub struct FccSyncStatus {
    pub last_sync_at: Option<String>,
    pub record_count: i64,
    pub file_date: Option<String>,
    pub sync_in_progress: bool,
    pub error_message: Option<String>,
}

/// Get current FCC sync status
pub async fn get_sync_status(pool: &SqlitePool) -> Result<FccSyncStatus, String> {
    let row: (Option<String>, i64, Option<String>, i64, Option<String>) = sqlx::query_as(
        r#"SELECT last_sync_at, record_count, file_date, sync_in_progress, error_message 
           FROM fcc_sync_status WHERE id = 1"#
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to get FCC sync status: {}", e))?;
    
    Ok(FccSyncStatus {
        last_sync_at: row.0,
        record_count: row.1,
        file_date: row.2,
        sync_in_progress: row.3 != 0,
        error_message: row.4,
    })
}

/// Lookup a callsign in the FCC database
pub async fn lookup_callsign(pool: &SqlitePool, call: &str) -> Option<FccLicenseInfo> {
    let call_upper = call.to_uppercase();
    
    let result: Result<(String, Option<String>, Option<String>, Option<String>, Option<String>), _> = 
        sqlx::query_as(
            "SELECT call, name, state, city, grid FROM fcc_licenses WHERE call = ?"
        )
        .bind(&call_upper)
        .fetch_one(pool)
        .await;
    
    match result {
        Ok((call, name, state, city, grid)) => Some(FccLicenseInfo {
            call,
            name,
            state,
            city,
            grid,
        }),
        Err(_) => None,
    }
}

/// Simplified license info for lookups
#[derive(Debug, Serialize, Clone)]
pub struct FccLicenseInfo {
    pub call: String,
    pub name: Option<String>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub grid: Option<String>,
}

/// Batch lookup multiple callsigns
pub async fn lookup_callsigns(pool: &SqlitePool, calls: &[String]) -> Vec<FccLicenseInfo> {
    if calls.is_empty() {
        return Vec::new();
    }
    
    // Build query with placeholders
    let placeholders: Vec<String> = calls.iter().map(|_| "?".to_string()).collect();
    let query = format!(
        "SELECT call, name, state, city, grid FROM fcc_licenses WHERE call IN ({})",
        placeholders.join(", ")
    );
    
    let mut q = sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>, Option<String>)>(&query);
    
    for call in calls {
        q = q.bind(call.to_uppercase());
    }
    
    match q.fetch_all(pool).await {
        Ok(rows) => rows.into_iter().map(|(call, name, state, city, grid)| {
            FccLicenseInfo { call, name, state, city, grid }
        }).collect(),
        Err(_) => Vec::new(),
    }
}
