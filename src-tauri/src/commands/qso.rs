//! QSO CRUD operations and history commands
//!
//! This module handles all QSO (contact) database operations:
//! - get_qsos: Fetch QSOs with pagination and confirmation status
//! - add_qso: Create new QSO with DXCC lookup
//! - update_qso: Update existing QSO fields
//! - delete_qso: Remove single QSO
//! - remove_duplicate_qsos: Clean up duplicate entries
//! - clear_all_qsos: Delete all QSOs (testing)
//! - add_test_qsos: Insert sample data (testing)
//! - get_callsign_history: Previous QSOs with a callsign
//! - check_qso_status: Check dupe/new DXCC status

use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::{command, Emitter};

use super::state::AppState;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Qso {
    pub id: i64,
    pub uuid: String,
    pub call: String,
    pub qso_date: String,
    pub qso_date_off: Option<String>,
    pub time_on: String,
    pub time_off: Option<String>,
    pub band: String,
    pub mode: String,
    pub freq: Option<f64>,
    pub dxcc: Option<i32>,
    pub country: Option<String>,
    pub continent: Option<String>,
    pub state: Option<String>,
    pub gridsquare: Option<String>,
    pub cqz: Option<i32>,
    pub ituz: Option<i32>,
    pub rst_sent: Option<String>,
    pub rst_rcvd: Option<String>,
    pub station_callsign: Option<String>,
    pub operator: Option<String>,
    pub my_gridsquare: Option<String>,
    pub tx_pwr: Option<f64>,
    pub adif_fields: Option<String>,
    pub user_data: Option<String>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
    // Confirmation status (from confirmations table)
    #[serde(default)]
    pub lotw_rcvd: Option<String>,
    #[serde(default)]
    pub eqsl_rcvd: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewQso {
    pub call: String,
    pub qso_date: String,
    pub time_on: String,
    pub band: String,
    pub mode: String,
    pub freq: Option<f64>,
    pub gridsquare: Option<String>,
    pub rst_sent: Option<String>,
    pub rst_rcvd: Option<String>,
    pub source: Option<String>,
}

/// Summary of previous QSOs with a callsign
#[derive(Debug, Serialize)]
pub struct CallsignHistory {
    pub call: String,
    pub total_qsos: i32,
    pub bands_worked: Vec<String>,
    pub modes_worked: Vec<String>,
    pub first_qso: Option<String>,
    pub last_qso: Option<String>,
    pub previous_qsos: Vec<PreviousQso>,
}

/// Compact representation of a previous QSO
#[derive(Debug, Serialize)]
pub struct PreviousQso {
    pub id: i64,
    pub qso_date: String,
    pub time_on: String,
    pub band: String,
    pub mode: String,
    pub rst_sent: Option<String>,
    pub rst_rcvd: Option<String>,
}

/// Status flags for a QSO (used for badge display)
#[derive(Debug, Serialize)]
pub struct QsoStatus {
    pub is_dupe: bool,
    pub is_new_dxcc: bool,
    pub is_new_band_dxcc: bool,
    pub is_new_mode_dxcc: bool,
    pub has_previous_qso: bool,
    pub previous_qso_count: i32,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert frequency in MHz to band string
pub fn freq_to_band(freq_mhz: f64) -> String {
    match freq_mhz {
        f if f >= 1.8 && f < 2.0 => "160m".to_string(),
        f if f >= 3.5 && f < 4.0 => "80m".to_string(),
        f if f >= 5.0 && f < 5.5 => "60m".to_string(),
        f if f >= 7.0 && f < 7.3 => "40m".to_string(),
        f if f >= 10.1 && f < 10.15 => "30m".to_string(),
        f if f >= 14.0 && f < 14.35 => "20m".to_string(),
        f if f >= 18.068 && f < 18.168 => "17m".to_string(),
        f if f >= 21.0 && f < 21.45 => "15m".to_string(),
        f if f >= 24.89 && f < 24.99 => "12m".to_string(),
        f if f >= 28.0 && f < 29.7 => "10m".to_string(),
        f if f >= 50.0 && f < 54.0 => "6m".to_string(),
        f if f >= 144.0 && f < 148.0 => "2m".to_string(),
        f if f >= 420.0 && f < 450.0 => "70cm".to_string(),
        _ => format!("{:.3}MHz", freq_mhz),
    }
}

// ============================================================================
// QSO Commands
// ============================================================================

#[command]
pub async fn get_qsos(
    state: tauri::State<'_, AppState>,
    limit: i32,
    offset: i32,
) -> Result<Vec<Qso>, String> {
    log::info!("Getting QSOs: limit={}, offset={}", limit, offset);

    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    let rows = sqlx::query(
        r#"
        SELECT 
            q.id, q.uuid, q.call, q.qso_date, q.qso_date_off, q.time_on, q.time_off, 
            q.band, q.mode, q.freq,
            q.dxcc, q.country, q.continent, q.state, q.gridsquare, q.cqz, q.ituz,
            q.rst_sent, q.rst_rcvd, q.station_callsign, q.operator, q.my_gridsquare, q.tx_pwr,
            q.adif_fields, q.user_data, q.source, q.created_at, q.updated_at,
            lotw.qsl_rcvd as lotw_rcvd,
            eqsl.qsl_rcvd as eqsl_rcvd
        FROM qsos q
        LEFT JOIN confirmations lotw ON q.id = lotw.qso_id AND lotw.source = 'LOTW'
        LEFT JOIN confirmations eqsl ON q.id = eqsl.qso_id AND eqsl.source = 'EQSL'
        ORDER BY q.qso_date DESC, q.time_on DESC 
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let qsos: Vec<Qso> = rows
        .iter()
        .map(|row| Qso {
            id: row.get("id"),
            uuid: row.get("uuid"),
            call: row.get("call"),
            qso_date: row.get("qso_date"),
            qso_date_off: row.get("qso_date_off"),
            time_on: row.get("time_on"),
            time_off: row.get("time_off"),
            band: row.get("band"),
            mode: row.get("mode"),
            freq: row.get("freq"),
            dxcc: row.get("dxcc"),
            country: row.get("country"),
            continent: row.get("continent"),
            state: row.get("state"),
            gridsquare: row.get("gridsquare"),
            cqz: row.get("cqz"),
            ituz: row.get("ituz"),
            rst_sent: row.get("rst_sent"),
            rst_rcvd: row.get("rst_rcvd"),
            station_callsign: row.get("station_callsign"),
            operator: row.get("operator"),
            my_gridsquare: row.get("my_gridsquare"),
            tx_pwr: row.get("tx_pwr"),
            adif_fields: row.get("adif_fields"),
            user_data: row.get("user_data"),
            source: row.get("source"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            lotw_rcvd: row.get("lotw_rcvd"),
            eqsl_rcvd: row.get("eqsl_rcvd"),
        })
        .collect();

    Ok(qsos)
}

#[command]
pub async fn add_qso(state: tauri::State<'_, AppState>, qso: NewQso) -> Result<Qso, String> {
    log::info!("Adding QSO: {}", qso.call);

    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    let uuid = uuid::Uuid::new_v4().to_string();
    let source = qso.source.unwrap_or_else(|| "manual".to_string());

    // Look up DXCC entity for the callsign
    let lookup = crate::reference::lookup_call_full(&qso.call);
    // Convert DXCC from ARRL 3-digit string to integer for database storage
    let dxcc_int = lookup.dxcc_as_i32();

    let result = sqlx::query(
        r#"
        INSERT INTO qsos (uuid, call, qso_date, time_on, band, mode, freq, dxcc, country, continent, cqz, ituz, gridsquare, rst_sent, rst_rcvd, source, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))
        "#,
    )
    .bind(&uuid)
    .bind(&qso.call)
    .bind(&qso.qso_date)
    .bind(&qso.time_on)
    .bind(&qso.band)
    .bind(&qso.mode)
    .bind(qso.freq)
    .bind(dxcc_int)
    .bind(&lookup.country)
    .bind(&lookup.continent)
    .bind(lookup.cqz)
    .bind(lookup.ituz)
    .bind(&qso.gridsquare)
    .bind(&qso.rst_sent)
    .bind(&qso.rst_rcvd)
    .bind(&source)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    let id = result.last_insert_rowid();

    // Add to sync queue for LoTW upload
    let _ = sqlx::query(
        "INSERT INTO sync_queue (qso_id, target, status, created_at) VALUES (?, 'LOTW', 'pending', datetime('now'))",
    )
    .bind(id)
    .execute(pool)
    .await;

    let now = chrono::Utc::now().to_rfc3339();

    Ok(Qso {
        id,
        uuid,
        call: qso.call,
        qso_date: qso.qso_date,
        qso_date_off: None,
        time_on: qso.time_on,
        time_off: None,
        band: qso.band,
        mode: qso.mode,
        freq: qso.freq,
        dxcc: lookup.dxcc_as_i32(),
        country: lookup.country,
        continent: lookup.continent,
        state: None,
        gridsquare: qso.gridsquare,
        cqz: lookup.cqz,
        ituz: lookup.ituz,
        rst_sent: qso.rst_sent,
        rst_rcvd: qso.rst_rcvd,
        station_callsign: None,
        operator: None,
        my_gridsquare: None,
        tx_pwr: None,
        adif_fields: None,
        user_data: None,
        source,
        created_at: now.clone(),
        updated_at: now,
        lotw_rcvd: None,
        eqsl_rcvd: None,
    })
}

#[command]
pub async fn update_qso(
    state: tauri::State<'_, AppState>,
    id: i64,
    updates: serde_json::Value,
) -> Result<(), String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    let obj = updates.as_object().ok_or("Updates must be an object")?;

    if obj.is_empty() {
        return Ok(());
    }

    let mut set_clauses: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    let allowed = [
        "call",
        "qso_date",
        "time_on",
        "time_off",
        "band",
        "mode",
        "freq",
        "rst_sent",
        "rst_rcvd",
        "gridsquare",
        "state",
        "name",
        "qth",
        "station_callsign",
        "my_gridsquare",
        "tx_pwr",
        "adif_fields",
        "user_data",
    ];

    for (key, value) in obj {
        if allowed.contains(&key.as_str()) {
            set_clauses.push(format!("{} = ?", key));
            values.push(value.as_str().unwrap_or("").to_string());
        }
    }

    if set_clauses.is_empty() {
        return Ok(());
    }

    set_clauses.push("updated_at = datetime('now')".to_string());

    let sql = format!("UPDATE qsos SET {} WHERE id = ?", set_clauses.join(", "));

    let mut query = sqlx::query(&sql);
    for v in &values {
        query = query.bind(v);
    }
    query = query.bind(id);

    query.execute(pool).await.map_err(|e| e.to_string())?;

    log::info!("Updated QSO {}: {:?}", id, obj.keys().collect::<Vec<_>>());
    Ok(())
}

#[command]
pub async fn delete_qso(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    log::info!("Deleting QSO: {}", id);

    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    sqlx::query("DELETE FROM qsos WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Find and remove duplicate QSOs (same call, date, time, band, mode)
/// Keeps the record with the BEST data (prefers: has grid, has entity, lowest id)
#[command]
pub async fn remove_duplicate_qsos(state: tauri::State<'_, AppState>) -> Result<i64, String> {
    log::info!("Finding and removing duplicate QSOs");

    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    let result = sqlx::query(
        r#"
        DELETE FROM qsos 
        WHERE id NOT IN (
            SELECT id FROM (
                SELECT id,
                    ROW_NUMBER() OVER (
                        PARTITION BY call, qso_date, SUBSTR(time_on, 1, 4), LOWER(band), mode
                        ORDER BY 
                            CASE WHEN gridsquare IS NOT NULL 
                                 AND LENGTH(gridsquare) >= 4 
                                 AND gridsquare NOT IN ('RR73', 'RRR', '73')
                                 THEN 0 ELSE 1 END,
                            CASE WHEN dxcc IS NOT NULL THEN 0 ELSE 1 END,
                            CASE WHEN country IS NOT NULL AND country != '' THEN 0 ELSE 1 END,
                            CASE WHEN rst_sent LIKE '%73%' OR rst_rcvd LIKE '%73%' THEN 1 ELSE 0 END,
                            id
                    ) as rn
                FROM qsos
            )
            WHERE rn = 1
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    let deleted = result.rows_affected() as i64;
    log::info!("Removed {} duplicate QSOs", deleted);

    Ok(deleted)
}

#[command]
pub async fn clear_all_qsos(state: tauri::State<'_, AppState>) -> Result<i64, String> {
    log::warn!("Clearing ALL QSOs from database");

    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    let result = sqlx::query("DELETE FROM qsos")
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    let deleted = result.rows_affected() as i64;
    log::info!("Deleted {} QSOs", deleted);

    Ok(deleted)
}

/// Add test QSOs for UI development
#[command]
pub async fn add_test_qsos(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<i32, String> {
    log::info!("Adding test QSOs...");

    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    // Delete previous test data
    sqlx::query("DELETE FROM qsos WHERE source = 'TEST'")
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    let test_qsos = vec![
        ("JA1ABC", "PM95", "Japan", 339, "AS", "20m", "FT8", 14.074, "-12", "-10"),
        ("DL1XYZ", "JO31", "Germany", 230, "EU", "40m", "FT8", 7.074, "-08", "-06"),
        ("VK2ABC", "QF56", "Australia", 150, "OC", "15m", "FT8", 21.074, "-15", "-11"),
        ("G4TEST", "IO91", "England", 223, "EU", "20m", "FT8", 14.074, "-05", "-09"),
        ("W5XYZ", "EM12", "United States", 291, "NA", "40m", "FT8", 7.074, "-11", "-13"),
        ("ZL1ZZZ", "RF72", "New Zealand", 170, "OC", "17m", "FT8", 18.100, "-18", "-14"),
        ("PY2AA", "GG66", "Brazil", 108, "SA", "10m", "FT8", 28.074, "-04", "-07"),
        ("UA3ABC", "KO85", "Russia", 15, "EU", "30m", "FT8", 10.136, "-14", "-16"),
    ];

    let now = chrono::Utc::now();
    let mut count = 0;

    for (i, (call, grid, country, dxcc, continent, band, mode, freq, rst_sent, rst_rcvd)) in
        test_qsos.iter().enumerate()
    {
        let uuid = uuid::Uuid::new_v4().to_string();
        let qso_time = now - chrono::Duration::minutes((i as i64) * 15);
        let qso_date = qso_time.format("%Y%m%d").to_string();
        let time_on = qso_time.format("%H%M%S").to_string();

        sqlx::query(
            r#"
            INSERT INTO qsos (uuid, call, qso_date, time_on, band, mode, freq, dxcc, country, continent, gridsquare, rst_sent, rst_rcvd, source, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'TEST', datetime('now'), datetime('now'))
            "#,
        )
        .bind(&uuid)
        .bind(call)
        .bind(&qso_date)
        .bind(&time_on)
        .bind(band)
        .bind(mode)
        .bind(freq)
        .bind(dxcc)
        .bind(country)
        .bind(continent)
        .bind(grid)
        .bind(rst_sent)
        .bind(rst_rcvd)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        count += 1;
    }

    log::info!("Added {} test QSOs", count);
    let _ = app.emit("test-qsos-added", count);

    Ok(count)
}

// ============================================================================
// Callsign History & Status Commands
// ============================================================================

#[command]
pub async fn get_callsign_history(
    state: tauri::State<'_, AppState>,
    call: String,
    exclude_id: Option<i64>,
) -> Result<CallsignHistory, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    let query = if let Some(excl_id) = exclude_id {
        sqlx::query(
            r#"SELECT id, qso_date, time_on, band, mode, rst_sent, rst_rcvd 
               FROM qsos WHERE call = ? AND id != ? 
               ORDER BY qso_date DESC, time_on DESC"#,
        )
        .bind(&call)
        .bind(excl_id)
    } else {
        sqlx::query(
            r#"SELECT id, qso_date, time_on, band, mode, rst_sent, rst_rcvd 
               FROM qsos WHERE call = ? 
               ORDER BY qso_date DESC, time_on DESC"#,
        )
        .bind(&call)
    };

    let rows = query.fetch_all(pool).await.map_err(|e| e.to_string())?;

    let mut bands_worked: Vec<String> = Vec::new();
    let mut modes_worked: Vec<String> = Vec::new();
    let mut previous_qsos: Vec<PreviousQso> = Vec::new();

    for row in &rows {
        let band: String = row.get("band");
        let mode: String = row.get("mode");

        if !bands_worked.contains(&band) {
            bands_worked.push(band.clone());
        }
        if !modes_worked.contains(&mode) {
            modes_worked.push(mode.clone());
        }

        previous_qsos.push(PreviousQso {
            id: row.get("id"),
            qso_date: row.get("qso_date"),
            time_on: row.get("time_on"),
            band,
            mode,
            rst_sent: row.get("rst_sent"),
            rst_rcvd: row.get("rst_rcvd"),
        });
    }

    let first_qso = previous_qsos.last().map(|q| q.qso_date.clone());
    let last_qso = previous_qsos.first().map(|q| q.qso_date.clone());

    Ok(CallsignHistory {
        call,
        total_qsos: previous_qsos.len() as i32,
        bands_worked,
        modes_worked,
        first_qso,
        last_qso,
        previous_qsos,
    })
}

#[command]
pub async fn check_qso_status(
    state: tauri::State<'_, AppState>,
    call: String,
    band: String,
    mode: String,
    dxcc: Option<i32>,
    qso_date: String,
    exclude_id: Option<i64>,
) -> Result<QsoStatus, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    // Check for dupe
    let dupe_query = if let Some(excl_id) = exclude_id {
        sqlx::query(
            "SELECT COUNT(*) as cnt FROM qsos WHERE call = ? AND band = ? AND mode = ? AND qso_date = ? AND id != ?",
        )
        .bind(&call)
        .bind(&band)
        .bind(&mode)
        .bind(&qso_date)
        .bind(excl_id)
    } else {
        sqlx::query(
            "SELECT COUNT(*) as cnt FROM qsos WHERE call = ? AND band = ? AND mode = ? AND qso_date = ?",
        )
        .bind(&call)
        .bind(&band)
        .bind(&mode)
        .bind(&qso_date)
    };
    let is_dupe: i64 = dupe_query
        .fetch_one(pool)
        .await
        .map(|r| r.get("cnt"))
        .unwrap_or(0);

    // Check previous QSOs
    let prev_count: i64 = if let Some(excl_id) = exclude_id {
        sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE call = ? AND id != ?")
            .bind(&call)
            .bind(excl_id)
            .fetch_one(pool)
            .await
            .map(|r| r.get("cnt"))
            .unwrap_or(0)
    } else {
        sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE call = ?")
            .bind(&call)
            .fetch_one(pool)
            .await
            .map(|r| r.get("cnt"))
            .unwrap_or(0)
    };

    // DXCC status checks
    let (is_new_dxcc, is_new_band_dxcc, is_new_mode_dxcc) = if let Some(dxcc_id) = dxcc {
        let any_with_dxcc: i64 = if let Some(excl_id) = exclude_id {
            sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE dxcc = ? AND id != ?")
                .bind(dxcc_id)
                .bind(excl_id)
                .fetch_one(pool)
                .await
                .map(|r| r.get("cnt"))
                .unwrap_or(0)
        } else {
            sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE dxcc = ?")
                .bind(dxcc_id)
                .fetch_one(pool)
                .await
                .map(|r| r.get("cnt"))
                .unwrap_or(0)
        };

        let any_on_band: i64 = if let Some(excl_id) = exclude_id {
            sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE dxcc = ? AND band = ? AND id != ?")
                .bind(dxcc_id)
                .bind(&band)
                .bind(excl_id)
                .fetch_one(pool)
                .await
                .map(|r| r.get("cnt"))
                .unwrap_or(0)
        } else {
            sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE dxcc = ? AND band = ?")
                .bind(dxcc_id)
                .bind(&band)
                .fetch_one(pool)
                .await
                .map(|r| r.get("cnt"))
                .unwrap_or(0)
        };

        let any_on_mode: i64 = if let Some(excl_id) = exclude_id {
            sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE dxcc = ? AND mode = ? AND id != ?")
                .bind(dxcc_id)
                .bind(&mode)
                .bind(excl_id)
                .fetch_one(pool)
                .await
                .map(|r| r.get("cnt"))
                .unwrap_or(0)
        } else {
            sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE dxcc = ? AND mode = ?")
                .bind(dxcc_id)
                .bind(&mode)
                .fetch_one(pool)
                .await
                .map(|r| r.get("cnt"))
                .unwrap_or(0)
        };

        (any_with_dxcc == 0, any_on_band == 0, any_on_mode == 0)
    } else {
        (false, false, false)
    };

    Ok(QsoStatus {
        is_dupe: is_dupe > 0,
        is_new_dxcc,
        is_new_band_dxcc,
        is_new_mode_dxcc,
        has_previous_qso: prev_count > 0,
        previous_qso_count: prev_count as i32,
    })
}
/// Result of the repair operation
#[derive(Debug, Serialize)]
pub struct RepairResult {
    pub qsos_checked: i32,
    pub qsos_repaired: i32,
    pub grids_cleared: i32,
    pub errors: Vec<String>,
}

/// Repair QSO data issues:
/// 1. Re-lookup DXCC for QSOs with NULL dxcc field
/// 2. Clear invalid grids (FT8 messages like RR73, RRR, 73)
/// 
/// This is a one-time repair command to fix data quality issues.
#[command]
pub async fn repair_qso_data(state: tauri::State<'_, AppState>) -> Result<RepairResult, String> {
    use crate::udp::wsjtx::is_valid_grid;
    
    log::info!("Starting QSO data repair...");
    
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    let mut qsos_checked = 0;
    let mut qsos_repaired = 0;
    let mut grids_cleared = 0;
    let mut errors: Vec<String> = Vec::new();
    
    // Step 1: Find all QSOs with NULL DXCC
    let rows = sqlx::query(
        "SELECT id, call, gridsquare FROM qsos WHERE dxcc IS NULL"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    log::info!("Found {} QSOs with NULL DXCC", rows.len());
    
    for row in &rows {
        qsos_checked += 1;
        let id: i64 = row.get("id");
        let call: String = row.get("call");
        let grid: Option<String> = row.get("gridsquare");
        
        // Look up DXCC from callsign
        let lookup = crate::reference::lookup_call_full(&call);
        let dxcc_int = lookup.dxcc_as_i32();
        
        if dxcc_int.is_none() {
            errors.push(format!("Could not lookup DXCC for: {}", call));
            continue;
        }
        
        // Check if grid is valid, clear if not
        let valid_grid = match &grid {
            Some(g) if is_valid_grid(g) => Some(g.clone()),
            Some(g) => {
                log::warn!("Clearing invalid grid '{}' for {}", g, call);
                grids_cleared += 1;
                None
            }
            None => None,
        };
        
        // Update the QSO
        let result = sqlx::query(
            "UPDATE qsos SET dxcc = ?, country = ?, continent = ?, cqz = ?, ituz = ?, gridsquare = ?, updated_at = datetime('now') WHERE id = ?"
        )
        .bind(dxcc_int)
        .bind(&lookup.country)
        .bind(&lookup.continent)
        .bind(lookup.cqz)
        .bind(lookup.ituz)
        .bind(&valid_grid)
        .bind(id)
        .execute(pool)
        .await;
        
        match result {
            Ok(_) => {
                qsos_repaired += 1;
                log::info!("Repaired QSO {}: {} -> DXCC {}", id, call, dxcc_int.unwrap());
            }
            Err(e) => {
                errors.push(format!("Failed to update {}: {}", call, e));
            }
        }
    }
    
    // Step 2: Clear any remaining invalid grids (even on QSOs with valid DXCC)
    let invalid_grid_rows = sqlx::query(
        "SELECT id, call, gridsquare FROM qsos WHERE gridsquare IS NOT NULL AND gridsquare != ''"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    for row in &invalid_grid_rows {
        let id: i64 = row.get("id");
        let call: String = row.get("call");
        let grid: String = row.get("gridsquare");
        
        if !is_valid_grid(&grid) {
            log::warn!("Clearing invalid grid '{}' for {} (id={})", grid, call, id);
            
            let result = sqlx::query(
                "UPDATE qsos SET gridsquare = NULL, updated_at = datetime('now') WHERE id = ?"
            )
            .bind(id)
            .execute(pool)
            .await;
            
            if let Err(e) = result {
                errors.push(format!("Failed to clear grid for {}: {}", call, e));
            } else {
                grids_cleared += 1;
            }
        }
    }
    
    log::info!("QSO repair complete: {} checked, {} repaired, {} grids cleared, {} errors",
               qsos_checked, qsos_repaired, grids_cleared, errors.len());
    
    Ok(RepairResult {
        qsos_checked,
        qsos_repaired,
        grids_cleared,
        errors,
    })
}