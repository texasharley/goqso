use serde::{Deserialize, Serialize};
use tauri::{command, Emitter};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::udp::{UdpListenerState, UdpMessage, start_listener, QsoLoggedMessage};
use crate::db::DbStats;
use sqlx::{Pool, Sqlite, Row};
use tokio::sync::Mutex as TokioMutex;

// ============================================================================
// Application State
// ============================================================================

/// Application state holding the database connection pool and UDP listener
pub struct AppState {
    pub db: Arc<TokioMutex<Option<Pool<Sqlite>>>>,
    pub udp_state: Arc<UdpListenerState>,
}

// ============================================================================
// UDP Listener Commands
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct UdpStatus {
    pub running: bool,
    pub port: u16,
    pub connected: bool,
    pub wsjtx_version: Option<String>,
}

#[command]
pub async fn start_udp_listener(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    port: u16,
) -> Result<(), String> {
    let udp_state = state.udp_state.clone();
    
    if udp_state.is_running() {
        return Err("UDP listener already running".to_string());
    }
    
    udp_state.set_port(port);
    
    // Create channel for receiving messages from the listener
    let (tx, mut rx) = mpsc::unbounded_channel::<UdpMessage>();
    
    // Start the listener
    start_listener(udp_state.clone(), tx)?;
    log::info!("Started UDP listener on port {}", port);
    
    // Spawn task to handle incoming messages and emit events to frontend
    let app_handle = app.clone();
    let db_arc = state.db.clone();
    
    tauri::async_runtime::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                UdpMessage::Decode(decode) => {
                    // Parse the FT8 message to extract callsign info
                    if let Some((call, grid, msg_type)) = crate::udp::parse_ft8_message(&decode.message) {
                        // Look up DXCC for the callsign
                        let (dxcc, country) = crate::reference::lookup_call(&call);
                        
                        // For US stations (DXCC 291), also look up the state from grid
                        let state = if dxcc == Some(291) {
                            if let Some(g) = &grid {
                                crate::reference::grid_to_state(g)
                                    .map(|(code, name)| serde_json::json!({
                                        "code": code,
                                        "name": name
                                    }))
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        
                        let _ = app_handle.emit("wsjtx-decode", serde_json::json!({
                            "time_ms": decode.time_ms,
                            "snr": decode.snr,
                            "delta_freq": decode.delta_freq,
                            "mode": decode.mode,
                            "message": decode.message,
                            "call": call,
                            "grid": grid,
                            "msg_type": format!("{:?}", msg_type),
                            "dxcc": dxcc,
                            "country": country,
                            "low_confidence": decode.low_confidence,
                            "state": state,
                        }));
                    }
                }
                UdpMessage::QsoLogged(qso) => {
                    log::info!("Received QSO from WSJT-X: {}", qso.call);
                    
                    // Insert into database
                    let db_guard = db_arc.lock().await;
                    if let Some(pool) = db_guard.as_ref() {
                        if let Err(e) = insert_qso_from_wsjtx(pool, &qso).await {
                            log::error!("Failed to insert QSO: {}", e);
                        }
                    }
                    drop(db_guard);
                    
                    // Emit event to frontend
                    let _ = app_handle.emit("qso-logged", QsoEvent::from_wsjtx(&qso));
                }
                UdpMessage::Heartbeat { id, version, .. } => {
                    let _ = app_handle.emit("wsjtx-heartbeat", serde_json::json!({
                        "id": id,
                        "version": version,
                    }));
                }
                UdpMessage::Status { id, dial_freq, mode, dx_call, tx_enabled, transmitting, .. } => {
                    let _ = app_handle.emit("wsjtx-status", serde_json::json!({
                        "id": id,
                        "dial_freq": dial_freq,
                        "mode": mode,
                        "dx_call": dx_call,
                        "tx_enabled": tx_enabled,
                        "transmitting": transmitting,
                    }));
                }
                UdpMessage::Connected => {
                    let _ = app_handle.emit("udp-connected", ());
                }
                UdpMessage::Disconnected => {
                    let _ = app_handle.emit("udp-disconnected", ());
                }
                UdpMessage::Error(e) => {
                    log::error!("UDP error: {}", e);
                    let _ = app_handle.emit("udp-error", e);
                }
            }
        }
    });
    
    Ok(())
}

#[command]
pub async fn stop_udp_listener(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.udp_state.set_running(false);
    log::info!("Stopping UDP listener");
    Ok(())
}

#[command]
pub async fn get_udp_status(state: tauri::State<'_, AppState>) -> Result<UdpStatus, String> {
    Ok(UdpStatus {
        running: state.udp_state.is_running(),
        port: state.udp_state.get_port(),
        connected: state.udp_state.is_running(), // For now, same as running
        wsjtx_version: None,
    })
}

// ============================================================================
// QSO Event for Frontend
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct QsoEvent {
    pub call: String,
    pub grid: String,
    pub freq_mhz: f64,
    pub mode: String,
    pub rst_sent: String,
    pub rst_rcvd: String,
    pub band: String,
}

impl QsoEvent {
    fn from_wsjtx(qso: &QsoLoggedMessage) -> Self {
        let freq_mhz = qso.freq_hz as f64 / 1_000_000.0;
        Self {
            call: qso.call.clone(),
            grid: qso.grid.clone(),
            freq_mhz,
            mode: qso.mode.clone(),
            rst_sent: qso.report_sent.clone(),
            rst_rcvd: qso.report_rcvd.clone(),
            band: freq_to_band(freq_mhz),
        }
    }
}

fn freq_to_band(freq_mhz: f64) -> String {
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

/// Insert a QSO from WSJT-X into the database
async fn insert_qso_from_wsjtx(pool: &Pool<Sqlite>, qso: &QsoLoggedMessage) -> Result<(), String> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let freq_mhz = qso.freq_hz as f64 / 1_000_000.0;
    let band = freq_to_band(freq_mhz);
    let now = chrono::Utc::now();
    let qso_date = now.format("%Y-%m-%d").to_string();
    let time_on = now.format("%H%M").to_string();
    
    // Look up DXCC entity for the callsign
    let (dxcc, country) = crate::reference::lookup_call(&qso.call);
    
    // Get continent from DXCC lookup
    let continent = dxcc.and_then(|d| crate::reference::get_continent(d));
    
    // Build adif_fields JSON for extended data
    let adif_fields = serde_json::json!({
        "name": if qso.name.is_empty() { None } else { Some(&qso.name) },
        "comments": if qso.comments.is_empty() { None } else { Some(&qso.comments) },
        "tx_pwr": if qso.tx_power.is_empty() { None } else { Some(&qso.tx_power) },
        "operator": if qso.operator_call.is_empty() { None } else { Some(&qso.operator_call) },
        "prop_mode": if qso.adif_propagation_mode.is_empty() { None } else { Some(&qso.adif_propagation_mode) },
        "exchange_sent": if qso.exchange_sent.is_empty() { None } else { Some(&qso.exchange_sent) },
        "exchange_rcvd": if qso.exchange_rcvd.is_empty() { None } else { Some(&qso.exchange_rcvd) },
    }).to_string();
    
    sqlx::query(
        r#"
        INSERT INTO qsos (
            uuid, call, qso_date, time_on, time_off, band, mode, freq,
            dxcc, country, continent, gridsquare,
            rst_sent, rst_rcvd, station_callsign, my_gridsquare,
            adif_fields, source, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'WSJT-X', datetime('now'), datetime('now'))
        "#
    )
    .bind(&uuid)
    .bind(&qso.call)
    .bind(&qso_date)
    .bind(&time_on)
    .bind(if qso.datetime_off.is_empty() { None } else { Some(&qso.datetime_off) })
    .bind(&band)
    .bind(&qso.mode)
    .bind(freq_mhz)
    .bind(dxcc)
    .bind(&country)
    .bind(&continent)
    .bind(&qso.grid)
    .bind(&qso.report_sent)
    .bind(&qso.report_rcvd)
    .bind(if qso.my_call.is_empty() { None } else { Some(&qso.my_call) })
    .bind(if qso.my_grid.is_empty() { None } else { Some(&qso.my_grid) })
    .bind(&adif_fields)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    log::info!("Inserted QSO: {} on {} ({} - {})", qso.call, band, dxcc.unwrap_or(0), country.unwrap_or_default());
    Ok(())
}

// ============================================================================
// QSO Operations
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Qso {
    pub id: i64,
    pub uuid: String,
    pub call: String,
    pub qso_date: String,
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
    pub my_gridsquare: Option<String>,
    pub tx_pwr: Option<f64>,
    pub adif_fields: Option<String>,
    pub user_data: Option<String>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
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

#[command]
pub async fn get_qsos(
    state: tauri::State<'_, AppState>,
    limit: i32, 
    offset: i32
) -> Result<Vec<Qso>, String> {
    log::info!("Getting QSOs: limit={}, offset={}", limit, offset);
    
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    let rows = sqlx::query(
        r#"
        SELECT 
            id, uuid, call, qso_date, time_on, time_off, band, mode, freq,
            dxcc, country, continent, state, gridsquare, cqz, ituz,
            rst_sent, rst_rcvd, station_callsign, my_gridsquare, tx_pwr,
            adif_fields, user_data, source, created_at, updated_at
        FROM qsos 
        ORDER BY qso_date DESC, time_on DESC 
        LIMIT ? OFFSET ?
        "#
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    let qsos: Vec<Qso> = rows.iter().map(|row| {
        use sqlx::Row;
        Qso {
            id: row.get("id"),
            uuid: row.get("uuid"),
            call: row.get("call"),
            qso_date: row.get("qso_date"),
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
            my_gridsquare: row.get("my_gridsquare"),
            tx_pwr: row.get("tx_pwr"),
            adif_fields: row.get("adif_fields"),
            user_data: row.get("user_data"),
            source: row.get("source"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }).collect();
    
    Ok(qsos)
}

#[command]
pub async fn add_qso(
    state: tauri::State<'_, AppState>,
    qso: NewQso
) -> Result<Qso, String> {
    log::info!("Adding QSO: {}", qso.call);
    
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    let uuid = uuid::Uuid::new_v4().to_string();
    let source = qso.source.unwrap_or_else(|| "manual".to_string());
    
    // Look up DXCC entity for the callsign
    let (dxcc, country) = crate::reference::lookup_call(&qso.call);
    let continent = dxcc.and_then(|d| crate::reference::get_continent(d));
    
    let result = sqlx::query(
        r#"
        INSERT INTO qsos (uuid, call, qso_date, time_on, band, mode, freq, dxcc, country, continent, gridsquare, rst_sent, rst_rcvd, source, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))
        "#
    )
    .bind(&uuid)
    .bind(&qso.call)
    .bind(&qso.qso_date)
    .bind(&qso.time_on)
    .bind(&qso.band)
    .bind(&qso.mode)
    .bind(qso.freq)
    .bind(dxcc)
    .bind(&country)
    .bind(&continent)
    .bind(&qso.gridsquare)
    .bind(&qso.rst_sent)
    .bind(&qso.rst_rcvd)
    .bind(&source)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    let id = result.last_insert_rowid();
    let now = chrono::Utc::now().to_rfc3339();
    
    Ok(Qso {
        id,
        uuid,
        call: qso.call,
        qso_date: qso.qso_date,
        time_on: qso.time_on,
        time_off: None,
        band: qso.band,
        mode: qso.mode,
        freq: qso.freq,
        dxcc,
        country,
        continent,
        state: None,
        gridsquare: qso.gridsquare,
        cqz: None,
        ituz: None,
        rst_sent: qso.rst_sent,
        rst_rcvd: qso.rst_rcvd,
        station_callsign: None,
        my_gridsquare: None,
        tx_pwr: None,
        adif_fields: None,
        user_data: None,
        source,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[command]
pub async fn delete_qso(
    state: tauri::State<'_, AppState>,
    id: i64
) -> Result<(), String> {
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

// =============================================================================
// Callsign History & Award Status Commands
// =============================================================================

/// Summary of previous QSOs with a callsign
#[derive(Debug, Serialize)]
pub struct CallsignHistory {
    pub call: String,
    pub total_qsos: i32,
    pub bands_worked: Vec<String>,
    pub modes_worked: Vec<String>,
    pub first_qso: Option<String>,  // date of first QSO
    pub last_qso: Option<String>,   // date of most recent QSO
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

/// Get history of all QSOs with a specific callsign
#[command]
pub async fn get_callsign_history(
    state: tauri::State<'_, AppState>,
    call: String,
    exclude_id: Option<i64>,  // Exclude current QSO from history
) -> Result<CallsignHistory, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    // Get all QSOs with this callsign
    let query = if let Some(excl_id) = exclude_id {
        sqlx::query(
            r#"SELECT id, qso_date, time_on, band, mode, rst_sent, rst_rcvd 
               FROM qsos WHERE call = ? AND id != ? 
               ORDER BY qso_date DESC, time_on DESC"#
        )
        .bind(&call)
        .bind(excl_id)
    } else {
        sqlx::query(
            r#"SELECT id, qso_date, time_on, band, mode, rst_sent, rst_rcvd 
               FROM qsos WHERE call = ? 
               ORDER BY qso_date DESC, time_on DESC"#
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

/// Status flags for a QSO (used for badge display)
#[derive(Debug, Serialize)]
pub struct QsoStatus {
    pub is_dupe: bool,              // Same call/band/mode within dupe window
    pub is_new_dxcc: bool,          // First QSO with this DXCC entity ever
    pub is_new_band_dxcc: bool,     // First QSO with this DXCC on this band
    pub is_new_mode_dxcc: bool,     // First QSO with this DXCC on this mode
    pub has_previous_qso: bool,     // Worked this callsign before
    pub previous_qso_count: i32,    // How many times worked before
}

/// Check the status of a QSO (dupe, new DXCC, etc.)
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
    
    // Check for dupe (same call/band/mode on same date)
    let dupe_query = if let Some(excl_id) = exclude_id {
        sqlx::query(
            "SELECT COUNT(*) as cnt FROM qsos WHERE call = ? AND band = ? AND mode = ? AND qso_date = ? AND id != ?"
        )
        .bind(&call).bind(&band).bind(&mode).bind(&qso_date).bind(excl_id)
    } else {
        sqlx::query(
            "SELECT COUNT(*) as cnt FROM qsos WHERE call = ? AND band = ? AND mode = ? AND qso_date = ?"
        )
        .bind(&call).bind(&band).bind(&mode).bind(&qso_date)
    };
    let is_dupe: i64 = dupe_query.fetch_one(pool).await.map(|r| r.get("cnt")).unwrap_or(0);
    
    // Check previous QSOs with this callsign
    let prev_count: i64 = if let Some(excl_id) = exclude_id {
        sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE call = ? AND id != ?")
            .bind(&call).bind(excl_id)
            .fetch_one(pool).await.map(|r| r.get("cnt")).unwrap_or(0)
    } else {
        sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE call = ?")
            .bind(&call)
            .fetch_one(pool).await.map(|r| r.get("cnt")).unwrap_or(0)
    };
    
    // DXCC status checks
    let (is_new_dxcc, is_new_band_dxcc, is_new_mode_dxcc) = if let Some(dxcc_id) = dxcc {
        let any_with_dxcc: i64 = if let Some(excl_id) = exclude_id {
            sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE dxcc = ? AND id != ?")
                .bind(dxcc_id).bind(excl_id)
                .fetch_one(pool).await.map(|r| r.get("cnt")).unwrap_or(0)
        } else {
            sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE dxcc = ?")
                .bind(dxcc_id)
                .fetch_one(pool).await.map(|r| r.get("cnt")).unwrap_or(0)
        };
        
        let any_on_band: i64 = if let Some(excl_id) = exclude_id {
            sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE dxcc = ? AND band = ? AND id != ?")
                .bind(dxcc_id).bind(&band).bind(excl_id)
                .fetch_one(pool).await.map(|r| r.get("cnt")).unwrap_or(0)
        } else {
            sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE dxcc = ? AND band = ?")
                .bind(dxcc_id).bind(&band)
                .fetch_one(pool).await.map(|r| r.get("cnt")).unwrap_or(0)
        };
        
        let any_on_mode: i64 = if let Some(excl_id) = exclude_id {
            sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE dxcc = ? AND mode = ? AND id != ?")
                .bind(dxcc_id).bind(&mode).bind(excl_id)
                .fetch_one(pool).await.map(|r| r.get("cnt")).unwrap_or(0)
        } else {
            sqlx::query("SELECT COUNT(*) as cnt FROM qsos WHERE dxcc = ? AND mode = ?")
                .bind(dxcc_id).bind(&mode)
                .fetch_one(pool).await.map(|r| r.get("cnt")).unwrap_or(0)
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

/// Update an existing QSO
#[command]
pub async fn update_qso(
    state: tauri::State<'_, AppState>,
    id: i64,
    updates: serde_json::Value,
) -> Result<(), String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    // Build dynamic UPDATE query from provided fields
    let obj = updates.as_object().ok_or("Updates must be an object")?;
    
    if obj.is_empty() {
        return Ok(());
    }
    
    let mut set_clauses: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();
    
    // Allowed fields to update
    let allowed = ["call", "qso_date", "time_on", "time_off", "band", "mode", "freq",
                   "rst_sent", "rst_rcvd", "gridsquare", "state", "name", "qth",
                   "station_callsign", "my_gridsquare", "tx_pwr", "adif_fields", "user_data"];
    
    for (key, value) in obj {
        if allowed.contains(&key.as_str()) {
            set_clauses.push(format!("{} = ?", key));
            values.push(value.as_str().unwrap_or("").to_string());
        }
    }
    
    if set_clauses.is_empty() {
        return Ok(());
    }
    
    // Always update updated_at
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

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub imported: i32,
    pub duplicates: i32,
    pub errors: i32,
}

#[command]
pub async fn import_adif(path: String) -> Result<ImportResult, String> {
    log::info!("Importing ADIF from: {}", path);
    // TODO: Parse ADIF file and import
    Ok(ImportResult {
        imported: 0,
        duplicates: 0,
        errors: 0,
    })
}

#[command]
pub async fn export_adif(path: String, qso_ids: Option<Vec<i64>>) -> Result<i32, String> {
    log::info!("Exporting ADIF to: {}", path);
    // TODO: Export QSOs to ADIF format
    Ok(0)
}

/// Add test QSOs for UI development - removes all previous test entries first
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
    
    // Sample test QSOs: (call, grid, country, dxcc, continent, band, mode, freq, rst_sent, rst_rcvd)
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
    
    for (i, (call, grid, country, dxcc, continent, band, mode, freq, rst_sent, rst_rcvd)) in test_qsos.iter().enumerate() {
        let uuid = uuid::Uuid::new_v4().to_string();
        // Spread QSOs over the past few hours
        let qso_time = now - chrono::Duration::minutes((i as i64) * 15);
        let qso_date = qso_time.format("%Y-%m-%d").to_string();
        let time_on = qso_time.format("%H%M").to_string();
        
        sqlx::query(
            r#"
            INSERT INTO qsos (uuid, call, qso_date, time_on, band, mode, freq, dxcc, country, continent, gridsquare, rst_sent, rst_rcvd, source, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'TEST', datetime('now'), datetime('now'))
            "#
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
    
    // Emit event to trigger UI refresh
    let _ = app.emit("test-qsos-added", count);
    
    Ok(count)
}

// ============================================================================
// LoTW Sync Commands
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SyncStatus {
    pub pending_uploads: i32,
    pub last_upload: Option<String>,
    pub last_download: Option<String>,
    pub is_syncing: bool,
}

#[command]
pub async fn sync_lotw_upload() -> Result<i32, String> {
    log::info!("Starting LoTW upload");
    // TODO: Export pending QSOs, sign with TQSL, upload
    Ok(0)
}

#[command]
pub async fn sync_lotw_download() -> Result<i32, String> {
    log::info!("Starting LoTW download");
    // TODO: Download confirmations from LoTW
    Ok(0)
}

#[command]
pub async fn get_sync_status() -> Result<SyncStatus, String> {
    Ok(SyncStatus {
        pending_uploads: 0,
        last_upload: None,
        last_download: None,
        is_syncing: false,
    })
}

#[command]
pub async fn detect_tqsl_path() -> Result<Option<String>, String> {
    let paths = if cfg!(windows) {
        vec![
            r"C:\Program Files\TrustedQSL\tqsl.exe",
            r"C:\Program Files (x86)\TrustedQSL\tqsl.exe",
        ]
    } else if cfg!(target_os = "macos") {
        vec!["/Applications/TrustedQSL/tqsl.app/Contents/MacOS/tqsl"]
    } else {
        vec!["/usr/bin/tqsl", "/usr/local/bin/tqsl"]
    };

    for path in paths {
        if std::path::Path::new(path).exists() {
            log::info!("Found TQSL at: {}", path);
            return Ok(Some(path.to_string()));
        }
    }
    
    log::warn!("TQSL not found in common locations");
    Ok(None)
}

// ============================================================================
// Awards Progress Commands
// ============================================================================

#[derive(Debug, Serialize)]
pub struct DxccProgress {
    pub worked: i32,
    pub confirmed: i32,
    pub total: i32,
}

#[derive(Debug, Serialize)]
pub struct WasProgress {
    pub worked: i32,
    pub confirmed: i32,
    pub total: i32,
    pub states: Vec<StateStatus>,
}

#[derive(Debug, Serialize)]
pub struct StateStatus {
    pub abbrev: String,
    pub name: String,
    pub worked: bool,
    pub confirmed: bool,
}

#[derive(Debug, Serialize)]
pub struct VuccProgress {
    pub worked: i32,
    pub confirmed: i32,
    pub target: i32,
    pub band: Option<String>,
}

#[command]
pub async fn get_dxcc_progress() -> Result<DxccProgress, String> {
    // TODO: Calculate from database
    Ok(DxccProgress {
        worked: 0,
        confirmed: 0,
        total: 340,
    })
}

#[command]
pub async fn get_was_progress() -> Result<WasProgress, String> {
    // TODO: Calculate from database
    Ok(WasProgress {
        worked: 0,
        confirmed: 0,
        total: 50,
        states: vec![],
    })
}

#[command]
pub async fn get_vucc_progress(band: Option<String>) -> Result<VuccProgress, String> {
    // TODO: Calculate from database
    Ok(VuccProgress {
        worked: 0,
        confirmed: 0,
        target: 100,
        band,
    })
}

// ============================================================================
// CTY Lookup Commands
// ============================================================================

#[derive(Debug, Serialize)]
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

#[command]
pub async fn lookup_callsign(call: String) -> Result<CallsignInfo, String> {
    log::info!("Looking up callsign: {}", call);
    // TODO: Look up in CTY database
    Ok(CallsignInfo {
        call,
        dxcc: None,
        entity_name: None,
        cq_zone: None,
        itu_zone: None,
        continent: None,
        latitude: None,
        longitude: None,
    })
}

// ============================================================================
// Settings Commands
// ============================================================================

#[command]
pub async fn get_setting(key: String) -> Result<Option<String>, String> {
    // TODO: Query settings table
    Ok(None)
}

#[command]
pub async fn set_setting(key: String, value: String) -> Result<(), String> {
    log::info!("Setting {} = {}", key, value);
    // TODO: Update settings table
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
