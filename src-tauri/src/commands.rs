use serde::{Deserialize, Serialize};
use tauri::{command, Emitter};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::udp::{UdpListenerState, UdpMessage, start_listener, QsoLoggedMessage};
use crate::db::DbStats;
use sqlx::{Pool, Sqlite, Row};
use tokio::sync::Mutex as TokioMutex;

// ============================================================================
// Helper Functions
// ============================================================================

/// Format time from milliseconds since midnight UTC to HHMMSS
fn format_time_from_ms(time_ms: u32) -> String {
    let total_seconds = time_ms / 1000;
    let hours = (total_seconds / 3600) % 24;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{:02}{:02}{:02}", hours, minutes, seconds)
}

/// Get current UTC time as HHMMSS
fn get_current_utc_time() -> String {
    use chrono::Utc;
    Utc::now().format("%H%M%S").to_string()
}

/// Parse TX message to extract de_call (sender) and dx_call (target)
/// FT8 format: "TARGET SENDER REPORT" or "CQ SENDER GRID"
fn parse_tx_message_calls(message: &str) -> (Option<String>, Option<String>) {
    let parts: Vec<&str> = message.split_whitespace().collect();
    if parts.is_empty() {
        return (None, None);
    }
    
    // CQ message: "CQ N5JKK EM26" - de=N5JKK, dx=None
    if parts[0] == "CQ" {
        if parts.len() >= 2 {
            return (Some(parts[1].to_string()), None);
        }
        return (None, None);
    }
    
    // Normal message: "W1ABC N5JKK -05" - we (N5JKK) are calling W1ABC
    // de=N5JKK (us), dx=W1ABC (target)
    if parts.len() >= 2 {
        let dx_call = parts[0].to_string();
        let de_call = parts[1].to_string();
        return (Some(de_call), Some(dx_call));
    }
    
    (None, None)
}

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
        // Track last TX message to avoid duplicates
        let mut last_tx_msg = String::new();
        
        while let Some(msg) = rx.recv().await {
            match msg {
                UdpMessage::Decode(decode) => {
                    // Parse the FT8 message to extract callsign info
                    // de_call = sender, dx_call = station being called (None for CQ)
                    if let Some((de_call, dx_call, grid, msg_type)) = crate::udp::parse_ft8_message(&decode.message) {
                        // Look up DXCC entity info for the DE (sending) callsign
                        // Per ADIF spec: Callsign prefix gives DXCC, COUNTRY, CQZ, ITUZ, CONT
                        // STATE is NOT derived from prefix (portable operators may be elsewhere)
                        let lookup = crate::reference::lookup_call_full(&de_call);
                        
                        // Save RX message to band_activity
                        let db_guard = db_arc.lock().await;
                        if let Some(pool) = db_guard.as_ref() {
                            let time_utc = format_time_from_ms(decode.time_ms);
                            let _ = save_band_activity(
                                pool,
                                &time_utc,
                                Some(decode.time_ms as i64),
                                "rx",
                                &decode.message,
                                Some(decode.snr),
                                Some(decode.delta_freq as i32),
                                Some(&de_call),
                                dx_call.as_deref(),
                                None, // dial_freq not in decode
                                Some(&decode.mode),
                            ).await;
                        }
                        drop(db_guard);
                        
                        let _ = app_handle.emit("wsjtx-decode", serde_json::json!({
                            "time_ms": decode.time_ms,
                            "snr": decode.snr,
                            "delta_time": decode.delta_time,
                            "delta_freq": decode.delta_freq,
                            "mode": decode.mode,
                            "message": decode.message,
                            "de_call": de_call,
                            "dx_call": dx_call,
                            "call": de_call, // backwards compat - the station sending
                            "grid": grid,
                            "msg_type": format!("{:?}", msg_type),
                            "dxcc": lookup.dxcc,
                            "country": lookup.country,
                            "continent": lookup.continent,
                            "cqz": lookup.cqz,
                            "ituz": lookup.ituz,
                            "low_confidence": decode.low_confidence,
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
                        } else {
                            log::info!("QSO inserted successfully: {}", qso.call);
                        }
                    } else {
                        log::warn!("Database not ready, QSO not saved: {}", qso.call);
                    }
                    drop(db_guard);
                    
                    // Emit event to frontend (always, even if db insert failed)
                    let _ = app_handle.emit("qso-logged", QsoEvent::from_wsjtx(&qso));
                }
                UdpMessage::Heartbeat { id, version, .. } => {
                    let _ = app_handle.emit("wsjtx-heartbeat", serde_json::json!({
                        "id": id,
                        "version": version,
                    }));
                }
                UdpMessage::Status { id, dial_freq, mode, dx_call, report, tx_enabled, transmitting, tx_message, .. } => {
                    // Save TX message to band_activity when transmitting starts
                    if transmitting && !tx_message.is_empty() && tx_message != last_tx_msg {
                        last_tx_msg = tx_message.clone();
                        
                        let db_guard = db_arc.lock().await;
                        if let Some(pool) = db_guard.as_ref() {
                            // Extract de_call (us) and dx_call from tx_message
                            let (de_call, parsed_dx_call) = parse_tx_message_calls(&tx_message);
                            let time_utc = get_current_utc_time();
                            
                            let _ = save_band_activity(
                                pool,
                                &time_utc,
                                None,
                                "tx",
                                &tx_message,
                                None, // No SNR for TX
                                None, // No delta_freq for TX
                                de_call.as_deref(),
                                parsed_dx_call.as_deref(),
                                Some(dial_freq as f64),
                                Some(&mode),
                            ).await;
                        }
                        drop(db_guard);
                    }
                    
                    // Clear last_tx_msg when not transmitting (prepare for next TX)
                    if !transmitting {
                        last_tx_msg.clear();
                    }
                    
                    let _ = app_handle.emit("wsjtx-status", serde_json::json!({
                        "id": id,
                        "dial_freq": dial_freq,
                        "mode": mode,
                        "dx_call": dx_call,
                        "report": report,
                        "tx_enabled": tx_enabled,
                        "transmitting": transmitting,
                        "tx_message": tx_message,
                    }));
                }
                UdpMessage::Connected => {
                    let _ = app_handle.emit("udp-connected", ());
                }
                UdpMessage::Clear { id, window } => {
                    // Clear message from WSJT-X - clear band activity
                    log::debug!("Clear window {} from {}", window, id);
                    let _ = app_handle.emit("wsjtx-clear", serde_json::json!({
                        "id": id,
                        "window": window,
                    }));
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

/// Send a Reply message to WSJT-X to call a station
/// This is equivalent to double-clicking on a decode in WSJT-X
#[command]
pub async fn call_station(
    state: tauri::State<'_, AppState>,
    time_ms: u32,
    snr: i32,
    delta_time: f64,
    delta_freq: u32,
    mode: String,
    message: String,
    low_confidence: bool,
) -> Result<(), String> {
    use crate::udp::listener::send_reply;
    use crate::udp::wsjtx::ReplyMessage;
    
    // Get the WSJT-X instance ID
    let id = state.udp_state.get_wsjtx_id()
        .ok_or("WSJT-X not connected - no heartbeat received yet")?;
    
    let reply = ReplyMessage {
        id,
        time_ms,
        snr,
        delta_time,
        delta_freq,
        mode,
        message: message.clone(),
        low_confidence,
        modifiers: 0x00, // No modifiers
    };
    
    log::info!("Calling station from message: {}", message);
    
    send_reply(&state.udp_state, reply)
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
/// Per data population strategy (CLAUDE.md):
/// - Tier 1: Callsign prefix → DXCC, COUNTRY, CQZ, ITUZ, CONT
/// - STATE is NOT derived from prefix (portable operators may be elsewhere)
/// - STATE will be filled later by LoTW sync (Tier 2)
async fn insert_qso_from_wsjtx(pool: &Pool<Sqlite>, qso: &QsoLoggedMessage) -> Result<(), String> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let freq_mhz = qso.freq_hz as f64 / 1_000_000.0;
    let band = freq_to_band(freq_mhz);
    let now = chrono::Utc::now();
    let qso_date = now.format("%Y-%m-%d").to_string();
    let time_on = now.format("%H%M").to_string();
    
    // Look up full DXCC entity info for the callsign
    // This gives us DXCC, COUNTRY, CQZ, ITUZ, CONT from prefix
    let lookup = crate::reference::lookup_call_full(&qso.call);
    
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
            dxcc, country, continent, cqz, ituz, gridsquare,
            rst_sent, rst_rcvd, station_callsign, my_gridsquare,
            adif_fields, source, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'WSJT-X', datetime('now'), datetime('now'))
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
    .bind(lookup.dxcc)
    .bind(&lookup.country)
    .bind(&lookup.continent)
    .bind(lookup.cqz)
    .bind(lookup.ituz)
    .bind(&qso.grid)
    .bind(&qso.report_sent)
    .bind(&qso.report_rcvd)
    .bind(if qso.my_call.is_empty() { None } else { Some(&qso.my_call) })
    .bind(if qso.my_grid.is_empty() { None } else { Some(&qso.my_grid) })
    .bind(&adif_fields)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    log::info!("Inserted QSO: {} on {} ({} - {})", qso.call, band, lookup.dxcc.unwrap_or(0), lookup.country.clone().unwrap_or_default());
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
    pub lotw_rcvd: Option<String>,     // Y if confirmed via LoTW
    #[serde(default)]
    pub eqsl_rcvd: Option<String>,     // Y if confirmed via eQSL
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
    
    // Use LEFT JOINs to get confirmation status in a single query
    // This is more performant than N+1 queries
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
    
    // Look up DXCC entity for the callsign (full lookup includes continent, cqz, ituz)
    let lookup = crate::reference::lookup_call_full(&qso.call);
    
    let result = sqlx::query(
        r#"
        INSERT INTO qsos (uuid, call, qso_date, time_on, band, mode, freq, dxcc, country, continent, cqz, ituz, gridsquare, rst_sent, rst_rcvd, source, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))
        "#
    )
    .bind(&uuid)
    .bind(&qso.call)
    .bind(&qso.qso_date)
    .bind(&qso.time_on)
    .bind(&qso.band)
    .bind(&qso.mode)
    .bind(qso.freq)
    .bind(lookup.dxcc)
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
        "INSERT INTO sync_queue (qso_id, target, status, created_at) VALUES (?, 'LOTW', 'pending', datetime('now'))"
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
        dxcc: lookup.dxcc,
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

#[command]
pub async fn clear_all_qsos(
    state: tauri::State<'_, AppState>,
) -> Result<i64, String> {
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
    pub total_qsos: i32,
    pub qsls_received: i32,
    pub last_upload: Option<String>,
    pub last_download: Option<String>,
    pub is_syncing: bool,
    pub lotw_configured: bool,
}

// NOTE: Upload functionality is intentionally NOT implemented.
// We must ensure no test data is ever submitted to LoTW.
// Only GET operations (downloading confirmations) are enabled.

#[command]
pub async fn sync_lotw_download(
    state: tauri::State<'_, AppState>,
    username: String,
    password: String,
    since_date: Option<String>,
) -> Result<LotwDownloadResult, String> {
    log::info!("Starting LoTW confirmation download");
    
    use crate::lotw::{LotwClient, LotwQueryOptions};
    
    // Create client with credentials
    let client = LotwClient::new(username, password);
    
    // Build query options
    let options = LotwQueryOptions {
        qso_qslsince: since_date,
        qso_qsldetail: true,
        qso_withown: true,
        ..Default::default()
    };
    
    // Download confirmations from LoTW
    let result = client.download_confirmations(&options).await
        .map_err(|e| e.to_string())?;
    
    log::info!("Downloaded {} bytes from LoTW", result.adif_content.len());
    
    // Parse the ADIF content
    use crate::adif::parse_adif;
    let adif_file = parse_adif(&result.adif_content)
        .map_err(|e| format!("Failed to parse LoTW response: {}", e))?;
    
    log::info!("Parsed {} QSL records from LoTW", adif_file.records.len());
    
    // Now match confirmations to local QSOs
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    let mut matched = 0;
    let mut unmatched = 0;
    let mut errors: Vec<String> = Vec::new();
    
    for record in &adif_file.records {
        let call = match record.call() {
            Some(c) => c.to_string(),
            None => {
                errors.push("Record missing CALL field".to_string());
                continue;
            }
        };
        
        let band = record.band().map(|s| s.to_uppercase()).unwrap_or_default();
        let mode = record.mode().map(|s| s.to_uppercase()).unwrap_or_default();
        let qso_date = record.qso_date().map(|s| s.to_string()).unwrap_or_default();
        let time_on = record.time_on().map(|s| s.to_string()).unwrap_or_default();
        
        // Match by call, band, date, and time_on prefix
        // LoTW may send 4-char time (HHMM) while we store 6-char (HHMMSS)
        // Use prefix matching: our time_on LIKE 'HHMM%'
        let match_result = sqlx::query(
            r#"SELECT id FROM qsos 
               WHERE UPPER(call) = ? AND UPPER(band) = ? AND qso_date = ?
                 AND time_on LIKE ? || '%'
               LIMIT 1"#
        )
        .bind(call.to_uppercase())
        .bind(&band)
        .bind(&qso_date)
        .bind(&time_on)
        .fetch_optional(pool)
        .await;
        
        match match_result {
            Ok(Some(row)) => {
                let qso_id: i64 = row.get("id");
                
                // Extract QSL details from LoTW record
                let qsl_date = record.get("QSLRDATE").map(|s| s.to_string());
                let dxcc: Option<i32> = record.get("DXCC").and_then(|s| s.parse().ok());
                let state = record.get("STATE").map(|s| s.to_string());
                let cnty = record.get("CNTY").map(|s| s.to_string());
                let gridsquare = record.get("GRIDSQUARE").map(|s| s.to_string());
                let cqz: Option<i32> = record.get("CQZ").and_then(|s| s.parse().ok());
                let ituz: Option<i32> = record.get("ITUZ").and_then(|s| s.parse().ok());
                let country = record.get("COUNTRY").map(|s| s.to_string());
                let credit_granted = record.get("APP_LOTW_CREDIT_GRANTED").map(|s| s.to_string());
                
                // Insert or update confirmation record (using confirmations table)
                sqlx::query(
                    r#"INSERT INTO confirmations (qso_id, source, qsl_rcvd, qsl_rcvd_date, credit_granted, verified_at)
                       VALUES (?, 'LOTW', 'Y', ?, ?, datetime('now'))
                       ON CONFLICT(qso_id, source) DO UPDATE SET
                         qsl_rcvd = 'Y',
                         qsl_rcvd_date = COALESCE(excluded.qsl_rcvd_date, qsl_rcvd_date),
                         credit_granted = COALESCE(excluded.credit_granted, credit_granted),
                         verified_at = datetime('now')"#
                )
                .bind(qso_id)
                .bind(&qsl_date)
                .bind(&credit_granted)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to insert confirmation: {}", e))?;
                
                // Update the QSO with LoTW-provided location data (if available)
                sqlx::query(
                    r#"UPDATE qsos SET 
                       dxcc = COALESCE(?, dxcc),
                       country = COALESCE(?, country),
                       state = COALESCE(?, state),
                       gridsquare = COALESCE(?, gridsquare),
                       cqz = COALESCE(?, cqz),
                       ituz = COALESCE(?, ituz),
                       updated_at = datetime('now')
                       WHERE id = ?"#
                )
                .bind(dxcc)
                .bind(&country)
                .bind(&state)
                .bind(&gridsquare)
                .bind(cqz)
                .bind(ituz)
                .bind(qso_id)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to update QSO {}: {}", qso_id, e))?;
                
                matched += 1;
                log::debug!("Matched LoTW QSL: {} on {} {}", call, band, qso_date);
            }
            Ok(None) => {
                unmatched += 1;
                log::debug!("No local QSO for LoTW QSL: {} on {} {}", call, band, qso_date);
            }
            Err(e) => {
                errors.push(format!("DB error matching {}: {}", call, e));
            }
        }
    }
    
    log::info!("LoTW sync complete: {} matched, {} unmatched", matched, unmatched);
    
    Ok(LotwDownloadResult {
        total_records: adif_file.records.len() as i32,
        matched,
        unmatched,
        errors,
        last_qsl: result.last_qsl,
    })
}

#[derive(Debug, Serialize)]
pub struct LotwDownloadResult {
    pub total_records: i32,
    pub matched: i32,
    pub unmatched: i32,
    pub errors: Vec<String>,
    pub last_qsl: Option<String>,
}

#[command]
pub async fn get_sync_status(
    state: tauri::State<'_, AppState>,
) -> Result<SyncStatus, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    // Count QSOs not yet UPLOADED to LoTW (no confirmation record with source='LOTW' and qsl_sent='Y')
    // This is distinct from "pending confirmation" (sent but not yet confirmed)
    let row = sqlx::query(
        "SELECT 
            (SELECT COUNT(*) FROM qsos) as total_qsos,
            (SELECT COUNT(*) FROM qsos q 
             WHERE NOT EXISTS (
                SELECT 1 FROM confirmations c 
                WHERE c.qso_id = q.id AND c.source = 'LOTW' AND c.qsl_sent = 'Y'
             )) as pending,
            (SELECT COUNT(DISTINCT qso_id) FROM confirmations WHERE source = 'LOTW' AND qsl_rcvd = 'Y') as qsls_received,
            (SELECT value FROM settings WHERE key = 'lotw_last_download') as last_download,
            (SELECT value FROM settings WHERE key = 'lotw_last_upload') as last_upload,
            EXISTS(SELECT 1 FROM settings WHERE key = 'lotw_username' AND value IS NOT NULL AND value != '') as has_creds"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(SyncStatus {
        pending_uploads: row.get::<i64, _>("pending") as i32,
        total_qsos: row.get::<i64, _>("total_qsos") as i32,
        qsls_received: row.get::<i64, _>("qsls_received") as i32,
        last_upload: row.try_get::<String, _>("last_upload").ok(),
        last_download: row.try_get::<String, _>("last_download").ok(),
        is_syncing: false,
        lotw_configured: row.get::<bool, _>("has_creds"),
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
// LoTW Upload via TQSL
// ============================================================================

#[derive(Debug, Serialize)]
pub struct LotwUploadResult {
    pub qsos_exported: usize,
    pub success: bool,
    pub message: String,
}

/// Upload pending QSOs to LoTW using TQSL command-line interface.
/// This exports unconfirmed QSOs as ADIF and calls TQSL to sign and upload.
#[command]
pub async fn upload_to_lotw(
    state: tauri::State<'_, AppState>,
    tqsl_path: String,
) -> Result<LotwUploadResult, String> {
    use std::io::Write;
    
    log::info!("Starting LoTW upload via TQSL");
    
    // Get database connection
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    // Query for QSOs that haven't been sent to LoTW yet
    // A QSO is "pending upload" if it has no LOTW confirmation record with qsl_sent = 'Y'
    // Note: source is stored as uppercase 'LOTW' in the database
    let rows = sqlx::query(
        r#"
        SELECT q.* FROM qsos q
        LEFT JOIN confirmations c ON q.id = c.qso_id AND c.source = 'LOTW'
        WHERE c.id IS NULL OR c.qsl_sent IS NULL OR c.qsl_sent != 'Y'
        ORDER BY q.qso_date DESC, q.time_on DESC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to query pending QSOs: {}", e))?;
    
    if rows.is_empty() {
        return Ok(LotwUploadResult {
            qsos_exported: 0,
            success: true,
            message: "No pending QSOs to upload".to_string(),
        });
    }
    
    // Collect QSO IDs for marking as sent after upload
    use sqlx::Row;
    let qso_ids: Vec<i64> = rows.iter().map(|r| r.get::<i64, _>("id")).collect();
    
    // Convert rows to JSON for ADIF export
    let qsos: Vec<serde_json::Value> = rows.iter().map(|r| row_to_json(r)).collect();
    let qso_count = qsos.len();
    
    log::info!("Exporting {} QSOs for LoTW upload", qso_count);
    
    // Generate ADIF content
    let records: Vec<std::collections::HashMap<String, String>> = qsos
        .iter()
        .map(|q| crate::adif::writer::qso_to_adif(q))
        .collect();
    
    let adif_content = crate::adif::write_adif(&records, "GoQSO");
    
    // Write to temp file
    let temp_dir = std::env::temp_dir();
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let temp_file = temp_dir.join(format!("goqso_upload_{}.adi", timestamp));
    
    let mut file = std::fs::File::create(&temp_file)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    file.write_all(adif_content.as_bytes())
        .map_err(|e| format!("Failed to write ADIF file: {}", e))?;
    
    log::info!("Wrote ADIF to: {}", temp_file.display());
    
    // Call TQSL to sign and upload
    // Per https://lotw.arrl.org/lotw-help/cmdline/
    // -d = suppress date-range dialog
    // -u = upload to LoTW after signing
    // -a compliant = skip duplicates/out-of-range, sign all valid QSOs
    // -x = batch mode, exit when done (status to stderr)
    // Note: If no -l specified, TQSL will prompt for Station Location
    let output = std::process::Command::new(&tqsl_path)
        .args(["-d", "-u", "-a", "compliant", "-x"])
        .arg(&temp_file)
        .output()
        .map_err(|e| format!("Failed to execute TQSL: {}", e))?;
    
    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    log::info!("TQSL exit code: {:?}", output.status.code());
    log::info!("TQSL stdout: {}", stdout);
    log::info!("TQSL stderr: {}", stderr);
    
    // TQSL exit codes per https://lotw.arrl.org/lotw-help/cmdline/
    // 0 = success: all QSOs submitted were signed and uploaded
    // 1 = cancelled by user
    // 2 = rejected by LoTW
    // 3 = unexpected response from TQSL server
    // 4 = TQSL error
    // 5 = TQSLlib error
    // 6 = unable to open input file
    // 7 = unable to open output file
    // 8 = no QSOs processed (all duplicates or out of date range)
    // 9 = some QSOs processed, some ignored (duplicates/out of range)
    // 10 = command syntax error
    // 11 = LoTW connection error
    
    let success = matches!(output.status.code(), Some(0) | Some(9));
    
    // Mark QSOs as sent to LoTW if upload succeeded (code 0, 8, or 9)
    // Code 8 means all were duplicates - they were already sent before
    if matches!(output.status.code(), Some(0) | Some(8) | Some(9)) {
        let today = chrono::Utc::now().format("%Y%m%d").to_string();
        for qso_id in &qso_ids {
            // Insert or update confirmation record with qsl_sent = 'Y'
            sqlx::query(
                r#"
                INSERT INTO confirmations (qso_id, source, qsl_sent, qsl_sent_date)
                VALUES (?, 'LOTW', 'Y', ?)
                ON CONFLICT(qso_id, source) DO UPDATE SET
                    qsl_sent = 'Y',
                    qsl_sent_date = COALESCE(confirmations.qsl_sent_date, excluded.qsl_sent_date)
                "#
            )
            .bind(qso_id)
            .bind(&today)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to mark QSO {} as sent: {}", qso_id, e))?;
        }
        log::info!("Marked {} QSOs as sent to LoTW", qso_ids.len());
        
        // Save last upload date
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES ('lotw_last_upload', ?, ?)")
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to save last upload date: {}", e))?;
    }
    
    let message = match output.status.code() {
        Some(0) => format!("Successfully uploaded {} QSO(s) to LoTW", qso_count),
        Some(9) => format!("Uploaded QSOs to LoTW - some duplicates skipped"),
        Some(1) => "Upload cancelled by user".to_string(),
        Some(2) => format!("Rejected by LoTW: {}", stderr),
        Some(3) => format!("Unexpected response from LoTW server: {}", stderr),
        Some(4) => format!("TQSL error: {}", stderr),
        Some(5) => format!("TQSLlib error: {}", stderr),
        Some(6) => "Unable to open input file".to_string(),
        Some(7) => "Unable to open output file".to_string(),
        Some(8) => "No QSOs uploaded (all were duplicates or out of date range)".to_string(),
        Some(10) => format!("Command syntax error: {}", stderr),
        Some(11) => "LoTW connection error - check your internet connection".to_string(),
        Some(code) => format!("TQSL error (code {}): {}", code, stderr),
        None => format!("TQSL process terminated unexpectedly: {}", stderr),
    };
    
    Ok(LotwUploadResult {
        qsos_exported: qso_count,
        success,
        message,
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
pub async fn get_setting(
    state: tauri::State<'_, AppState>,
    key: String
) -> Result<Option<String>, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    let result = sqlx::query_scalar::<_, String>(
        "SELECT value FROM settings WHERE key = ?"
    )
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
    value: String
) -> Result<(), String> {
    log::info!("Setting {} = {}", key, if key.contains("password") { "***" } else { &value });
    
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    sqlx::query(
        r#"INSERT INTO settings (key, value, updated_at) 
           VALUES (?, ?, datetime('now'))
           ON CONFLICT(key) DO UPDATE SET 
             value = excluded.value,
             updated_at = datetime('now')"#
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
// ADIF Import/Export Commands
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub total_records: usize,
    pub imported: usize,
    pub skipped: usize,
    pub errors: usize,
    pub error_messages: Vec<String>,
}

#[command]
pub async fn import_adif(
    state: tauri::State<'_, AppState>,
    content: String,
    skip_duplicates: bool,
) -> Result<ImportResult, String> {
    use crate::adif::parse_adif;
    
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    let adif_file = parse_adif(&content)?;
    
    let mut result = ImportResult {
        total_records: adif_file.records.len(),
        imported: 0,
        skipped: 0,
        errors: 0,
        error_messages: Vec::new(),
    };
    
    for record in &adif_file.records {
        let call = match record.call() {
            Some(c) => c.to_uppercase(),
            None => {
                result.errors += 1;
                result.error_messages.push("Record missing CALL field".to_string());
                continue;
            }
        };
        
        let band = record.get_or("BAND", "").to_uppercase();
        let mode = record.get_or("MODE", "").to_uppercase();
        let qso_date = record.get_or("QSO_DATE", "");
        let time_on = record.get_or("TIME_ON", "");
        
        if band.is_empty() || mode.is_empty() || qso_date.is_empty() {
            result.errors += 1;
            result.error_messages.push(format!("Record for {} missing required fields", call));
            continue;
        }
        
        // Check for duplicate (same call, date, time, band, mode)
        // Note: time_on is included because you can work the same station
        // multiple times on the same day/band/mode (e.g., contest exchanges)
        if skip_duplicates {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM qsos WHERE call = ? AND qso_date = ? AND time_on = ? AND band = ? AND mode = ?)"
            )
            .bind(&call)
            .bind(&qso_date)
            .bind(&time_on)
            .bind(&band)
            .bind(&mode)
            .fetch_one(pool)
            .await
            .unwrap_or(false);
            
            if exists {
                result.skipped += 1;
                continue;
            }
        }
        
        // Build adif_fields JSON for extended fields
        let mut adif_fields = serde_json::Map::new();
        for (key, value) in &record.fields {
            // Skip core fields we store in columns
            let core_fields = ["CALL", "QSO_DATE", "QSO_DATE_OFF", "TIME_ON", "TIME_OFF", "BAND", "MODE", "FREQ",
                              "DXCC", "COUNTRY", "STATE", "CNTY", "GRIDSQUARE", "CQZ", "ITUZ", "CONT",
                              "RST_SENT", "RST_RCVD", "STATION_CALLSIGN", "MY_GRIDSQUARE", "TX_PWR", "OPERATOR"];
            if !core_fields.contains(&key.as_str()) && !key.starts_with("APP_") {
                adif_fields.insert(key.to_lowercase(), serde_json::Value::String(value.clone()));
            }
        }
        
        let uuid = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // Insert the QSO with all available columns
        let insert_result = sqlx::query(
            r#"INSERT INTO qsos (
                uuid, call, qso_date, qso_date_off, time_on, time_off, band, mode, submode, freq,
                dxcc, country, state, cnty, gridsquare, continent, cqz, ituz,
                rst_sent, rst_rcvd, station_callsign, operator, my_gridsquare, tx_pwr,
                prop_mode, sat_name, iota, pota_ref, sota_ref, wwff_ref, pfx,
                name, qth, comment, arrl_sect,
                my_cnty, my_arrl_sect, my_sota_ref, my_pota_ref,
                adif_fields, source, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&uuid)
        .bind(&call)
        .bind(&qso_date)
        .bind(record.get("QSO_DATE_OFF"))
        .bind(&time_on)
        .bind(record.get("TIME_OFF"))
        .bind(&band)
        .bind(&mode)
        .bind(record.get("SUBMODE"))
        .bind(record.freq())
        .bind(record.dxcc())
        .bind(record.country())
        .bind(record.state())
        .bind(record.cnty())
        .bind(record.gridsquare())
        .bind(record.get("CONT"))
        .bind(record.cqz())
        .bind(record.ituz())
        .bind(record.get("RST_SENT"))
        .bind(record.get("RST_RCVD"))
        .bind(record.get("STATION_CALLSIGN"))
        .bind(record.get("OPERATOR"))
        .bind(record.get("MY_GRIDSQUARE"))
        .bind(record.get("TX_PWR").and_then(|s| s.parse::<f64>().ok()))
        .bind(record.get("PROP_MODE"))
        .bind(record.get("SAT_NAME"))
        .bind(record.get("IOTA"))
        .bind(record.get("POTA_REF"))
        .bind(record.get("SOTA_REF"))
        .bind(record.get("WWFF_REF"))
        .bind(record.get("PFX"))
        .bind(record.get("NAME"))
        .bind(record.get("QTH"))
        .bind(record.get("COMMENT"))
        .bind(record.get("ARRL_SECT"))
        .bind(record.get("MY_CNTY"))
        .bind(record.get("MY_ARRL_SECT"))
        .bind(record.get("MY_SOTA_REF"))
        .bind(record.get("MY_POTA_REF"))
        .bind(serde_json::to_string(&adif_fields).unwrap_or_default())
        .bind("ADIF")
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await;
        
        match insert_result {
            Ok(_) => result.imported += 1,
            Err(e) => {
                result.errors += 1;
                if result.error_messages.len() < 10 {
                    result.error_messages.push(format!("{}: {}", call, e));
                }
            }
        }
    }
    
    log::info!("ADIF import: {} imported, {} skipped, {} errors", 
              result.imported, result.skipped, result.errors);
    
    Ok(result)
}

#[command]
pub async fn export_adif(
    state: tauri::State<'_, AppState>,
    qso_ids: Option<Vec<i64>>,
) -> Result<String, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    let qsos: Vec<serde_json::Value> = if let Some(ids) = qso_ids {
        // Export specific QSOs
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!("SELECT * FROM qsos WHERE id IN ({}) ORDER BY qso_date DESC, time_on DESC", placeholders);
        let mut q = sqlx::query(&query);
        for id in &ids {
            q = q.bind(id);
        }
        let rows = q.fetch_all(pool).await.map_err(|e| e.to_string())?;
        rows.iter().map(|r| row_to_json(r)).collect()
    } else {
        // Export all QSOs
        let rows = sqlx::query("SELECT * FROM qsos ORDER BY qso_date DESC, time_on DESC")
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        rows.iter().map(|r| row_to_json(r)).collect()
    };
    
    let records: Vec<std::collections::HashMap<String, String>> = qsos
        .iter()
        .map(|q| crate::adif::writer::qso_to_adif(q))
        .collect();
    
    Ok(crate::adif::write_adif(&records, "GoQSO"))
}

fn row_to_json(row: &sqlx::sqlite::SqliteRow) -> serde_json::Value {
    use sqlx::Row;
    
    serde_json::json!({
        "id": row.get::<i64, _>("id"),
        "uuid": row.get::<String, _>("uuid"),
        "call": row.get::<String, _>("call"),
        "qso_date": row.get::<String, _>("qso_date"),
        "time_on": row.get::<String, _>("time_on"),
        "time_off": row.try_get::<String, _>("time_off").ok(),
        "band": row.get::<String, _>("band"),
        "mode": row.get::<String, _>("mode"),
        "freq": row.try_get::<f64, _>("freq").ok(),
        "dxcc": row.try_get::<i64, _>("dxcc").ok(),
        "country": row.try_get::<String, _>("country").ok(),
        "state": row.try_get::<String, _>("state").ok(),
        "cnty": row.try_get::<String, _>("cnty").ok(),
        "gridsquare": row.try_get::<String, _>("gridsquare").ok(),
        "continent": row.try_get::<String, _>("continent").ok(),
        "cqz": row.try_get::<i64, _>("cqz").ok(),
        "ituz": row.try_get::<i64, _>("ituz").ok(),
        "rst_sent": row.try_get::<String, _>("rst_sent").ok(),
        "rst_rcvd": row.try_get::<String, _>("rst_rcvd").ok(),
        "station_callsign": row.try_get::<String, _>("station_callsign").ok(),
        "my_gridsquare": row.try_get::<String, _>("my_gridsquare").ok(),
        "tx_pwr": row.try_get::<f64, _>("tx_pwr").ok(),
        "adif_fields": row.try_get::<String, _>("adif_fields").ok(),
        "source": row.try_get::<String, _>("source").ok(),
    })
}

// ============================================================================
// LoTW Confirmation Import
// ============================================================================

#[derive(Debug, Serialize)]
pub struct LotwImportResult {
    pub total_records: usize,
    pub matched: usize,
    pub not_found: usize,
    pub already_confirmed: usize,
    pub errors: usize,
}

#[command]
pub async fn import_lotw_confirmations(
    state: tauri::State<'_, AppState>,
    content: String,
) -> Result<LotwImportResult, String> {
    use crate::adif::parse_adif;
    
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    let adif_file = parse_adif(&content)?;
    
    let mut result = LotwImportResult {
        total_records: adif_file.records.len(),
        matched: 0,
        not_found: 0,
        already_confirmed: 0,
        errors: 0,
    };
    
    for record in &adif_file.records {
        // Only process confirmed records
        if !record.is_lotw_confirmed() {
            continue;
        }
        
        let call = match record.call() {
            Some(c) => c.to_uppercase(),
            None => continue,
        };
        
        let band = record.get_or("BAND", "").to_uppercase();
        let mode = record.get_or("MODE", "").to_uppercase();
        let qso_date = record.get_or("QSO_DATE", "");
        let time_on = record.get_or("TIME_ON", "");
        
        // Find matching QSO in our database
        // Match by call, band, mode, date, and time (within 5 minutes)
        let matching_qso: Option<(i64,)> = sqlx::query_as(
            r#"SELECT id FROM qsos 
               WHERE call = ? AND band = ? AND mode = ? AND qso_date = ?
               ORDER BY ABS(
                   CAST(SUBSTR(time_on, 1, 2) AS INTEGER) * 60 + CAST(SUBSTR(time_on, 3, 2) AS INTEGER) -
                   CAST(SUBSTR(?, 1, 2) AS INTEGER) * 60 - CAST(SUBSTR(?, 3, 2) AS INTEGER)
               )
               LIMIT 1"#
        )
        .bind(&call)
        .bind(&band)
        .bind(&mode)
        .bind(&qso_date)
        .bind(&time_on)
        .bind(&time_on)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        
        let qso_id = match matching_qso {
            Some((id,)) => id,
            None => {
                result.not_found += 1;
                continue;
            }
        };
        
        // Check if already confirmed
        let already_confirmed: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM confirmations WHERE qso_id = ? AND source = 'LOTW' AND qsl_rcvd = 'Y')"
        )
        .bind(qso_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);
        
        if already_confirmed {
            result.already_confirmed += 1;
            continue;
        }
        
        // Insert or update confirmation
        let qslrdate = record.qslrdate().map(|s| s.as_str()).unwrap_or("");
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let insert_result = sqlx::query(
            r#"INSERT INTO confirmations (qso_id, source, qsl_rcvd, qsl_rcvd_date, verified_at)
               VALUES (?, 'LOTW', 'Y', ?, ?)
               ON CONFLICT(qso_id, source) DO UPDATE SET 
                   qsl_rcvd = 'Y', 
                   qsl_rcvd_date = excluded.qsl_rcvd_date,
                   verified_at = excluded.verified_at"#
        )
        .bind(qso_id)
        .bind(qslrdate)
        .bind(&now)
        .execute(pool)
        .await;
        
        match insert_result {
            Ok(_) => result.matched += 1,
            Err(_) => result.errors += 1,
        }
        
        // Also update QSO with any additional LoTW data (state, county, etc.)
        if let Some(state) = record.state() {
            let _ = sqlx::query("UPDATE qsos SET state = COALESCE(state, ?) WHERE id = ?")
                .bind(state)
                .bind(qso_id)
                .execute(pool)
                .await;
        }
        if let Some(cnty) = record.cnty() {
            let _ = sqlx::query("UPDATE qsos SET cnty = COALESCE(cnty, ?) WHERE id = ?")
                .bind(cnty)
                .bind(qso_id)
                .execute(pool)
                .await;
        }
        if let Some(grid) = record.gridsquare() {
            let _ = sqlx::query("UPDATE qsos SET gridsquare = COALESCE(gridsquare, ?) WHERE id = ?")
                .bind(grid)
                .bind(qso_id)
                .execute(pool)
                .await;
        }
    }
    
    log::info!("LoTW import: {} matched, {} not found, {} already confirmed", 
              result.matched, result.not_found, result.already_confirmed);
    
    Ok(result)
}

// ============================================================================
// Award Progress Commands
// ============================================================================

#[derive(Debug, Serialize)]
pub struct DxccProgress {
    pub worked: i64,
    pub confirmed: i64,
    pub total: i64,
}

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
        sqlx::query_scalar("SELECT COUNT(DISTINCT dxcc) FROM qsos WHERE dxcc IS NOT NULL AND band = ? AND mode = ?")
            .bind(b).bind(m)
            .fetch_one(pool).await.unwrap_or(0)
    } else if let Some(b) = &band {
        sqlx::query_scalar("SELECT COUNT(DISTINCT dxcc) FROM qsos WHERE dxcc IS NOT NULL AND band = ?")
            .bind(b)
            .fetch_one(pool).await.unwrap_or(0)
    } else if let Some(m) = &mode {
        sqlx::query_scalar("SELECT COUNT(DISTINCT dxcc) FROM qsos WHERE dxcc IS NOT NULL AND mode = ?")
            .bind(m)
            .fetch_one(pool).await.unwrap_or(0)
    } else {
        sqlx::query_scalar("SELECT COUNT(DISTINCT dxcc) FROM qsos WHERE dxcc IS NOT NULL")
            .fetch_one(pool).await.unwrap_or(0)
    };
    
    // Count confirmed DXCC entities
    let confirmed: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(DISTINCT q.dxcc) FROM qsos q
           JOIN confirmations c ON c.qso_id = q.id
           WHERE q.dxcc IS NOT NULL AND c.source = 'LOTW' AND c.qsl_rcvd = 'Y'"#
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

#[derive(Debug, Serialize)]
pub struct WasProgress {
    pub worked: i64,
    pub confirmed: i64,
    pub total: i64,
    pub worked_states: Vec<String>,
    pub confirmed_states: Vec<String>,
}

#[command]
pub async fn get_was_progress(
    state: tauri::State<'_, AppState>,
    band: Option<String>,
    mode: Option<String>,
) -> Result<WasProgress, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    // Get unique worked US states
    let worked_states: Vec<(String,)> = if let (Some(b), Some(m)) = (&band, &mode) {
        sqlx::query_as(
            "SELECT DISTINCT state FROM qsos WHERE dxcc = 291 AND state IS NOT NULL AND band = ? AND mode = ?"
        )
        .bind(b).bind(m)
        .fetch_all(pool).await.unwrap_or_default()
    } else {
        sqlx::query_as(
            "SELECT DISTINCT state FROM qsos WHERE dxcc = 291 AND state IS NOT NULL"
        )
        .fetch_all(pool).await.unwrap_or_default()
    };
    
    // Get confirmed states
    let confirmed_states: Vec<(String,)> = sqlx::query_as(
        r#"SELECT DISTINCT q.state FROM qsos q
           JOIN confirmations c ON c.qso_id = q.id
           WHERE q.dxcc = 291 AND q.state IS NOT NULL AND c.source = 'LOTW' AND c.qsl_rcvd = 'Y'"#
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

// ============================================================================
// Diagnostic Commands
// ============================================================================

#[derive(Debug, Serialize)]
pub struct QsoDiagnostic {
    pub call: String,
    pub qso_date: String,
    pub time_on: String,
    pub band: String,
    pub mode: String,
    pub source: Option<String>,
    pub has_lotw_confirmation: bool,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticReport {
    pub total_qsos: i64,
    pub confirmed_count: i64,
    pub pending_count: i64,
    pub by_source: Vec<(String, i64)>,
    pub duplicate_candidates: Vec<String>,
    pub qsos_not_in_lotw_window: Vec<QsoDiagnostic>,
}

/// Get diagnostic information about QSO data for troubleshooting
#[command]
pub async fn get_qso_diagnostics(
    state: tauri::State<'_, AppState>,
) -> Result<DiagnosticReport, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    // Total QSOs
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM qsos")
        .fetch_one(pool).await.map_err(|e| e.to_string())?;
    
    // Confirmed count (has LoTW confirmation with qsl_rcvd='Y')
    let confirmed: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(DISTINCT q.id) FROM qsos q
           JOIN confirmations c ON c.qso_id = q.id
           WHERE c.source = 'LOTW' AND c.qsl_rcvd = 'Y'"#
    ).fetch_one(pool).await.map_err(|e| e.to_string())?;
    
    // QSOs by source
    let by_source: Vec<(String, i64)> = sqlx::query_as(
        "SELECT COALESCE(source, 'unknown') as src, COUNT(*) as cnt FROM qsos GROUP BY source ORDER BY cnt DESC"
    ).fetch_all(pool).await.unwrap_or_default();
    
    // Find potential duplicates (same call+date+band but different times within 5 min)
    let dupe_candidates: Vec<(String,)> = sqlx::query_as(
        r#"SELECT DISTINCT a.call || ' on ' || a.qso_date || ' ' || a.band 
           FROM qsos a 
           JOIN qsos b ON a.call = b.call AND a.qso_date = b.qso_date AND a.band = b.band AND a.id != b.id
           WHERE ABS(CAST(SUBSTR(a.time_on, 1, 2) * 60 + SUBSTR(a.time_on, 3, 2) AS INTEGER) - 
                     CAST(SUBSTR(b.time_on, 1, 2) * 60 + SUBSTR(b.time_on, 3, 2) AS INTEGER)) < 5
           LIMIT 20"#
    ).fetch_all(pool).await.unwrap_or_default();
    
    // QSOs that might not be in LoTW (before Feb 2023 or different characteristics)
    // LoTW shows oldest from 2023-02-04, so check for older QSOs
    let not_in_lotw: Vec<QsoDiagnostic> = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, bool)>(
        r#"SELECT q.call, q.qso_date, q.time_on, q.band, q.mode, q.source,
           EXISTS(SELECT 1 FROM confirmations c WHERE c.qso_id = q.id AND c.source = 'LOTW') as has_lotw
           FROM qsos q
           WHERE q.qso_date < '20230204' OR q.source = 'ADIF'
           ORDER BY q.qso_date DESC
           LIMIT 20"#
    ).fetch_all(pool).await.unwrap_or_default()
     .into_iter()
     .map(|(call, date, time, band, mode, source, has_lotw)| QsoDiagnostic {
         call, qso_date: date, time_on: time, band, mode, source, has_lotw_confirmation: has_lotw
     })
     .collect();
    
    Ok(DiagnosticReport {
        total_qsos: total.0,
        confirmed_count: confirmed.0,
        pending_count: total.0 - confirmed.0,
        by_source: by_source,
        duplicate_candidates: dupe_candidates.into_iter().map(|(s,)| s).collect(),
        qsos_not_in_lotw_window: not_in_lotw,
    })
}

// ============================================================================
// Band Activity Commands
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

/// Save a band activity message (TX or RX)
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
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
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

/// Get recent band activity messages
#[command]
pub async fn get_recent_activity(
    state: tauri::State<'_, AppState>,
    minutes: Option<i32>,
) -> Result<Vec<BandActivityMessage>, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;
    
    let mins = minutes.unwrap_or(60);
    
    let rows: Vec<(i64, String, Option<i64>, String, String, Option<i32>, Option<i32>, Option<String>, Option<String>, Option<f64>, Option<String>)> = sqlx::query_as(
        r#"SELECT id, time_utc, time_ms, direction, message, snr, delta_freq, de_call, dx_call, dial_freq, mode
           FROM band_activity
           WHERE created_at > datetime('now', ? || ' minutes')
           ORDER BY created_at ASC"#
    )
    .bind(format!("-{}", mins))
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get band activity: {}", e))?;
    
    Ok(rows.into_iter().map(|(id, time_utc, time_ms, direction, message, snr, delta_freq, de_call, dx_call, dial_freq, mode)| {
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
    }).collect())
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
        r#"DELETE FROM band_activity WHERE created_at < datetime('now', ? || ' minutes')"#
    )
    .bind(format!("-{}", mins))
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to prune band activity: {}", e))?;
    
    Ok(result.rows_affected() as i64)
}
