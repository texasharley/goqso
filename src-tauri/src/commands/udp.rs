//! UDP Listener Commands
//!
//! Commands for managing the WSJT-X UDP listener.

use serde::Serialize;
use tauri::{command, Emitter};
use tokio::sync::mpsc;

use super::state::AppState;
use super::time_utils::{format_time_from_ms, get_current_utc_time, normalize_time_to_hhmmss, is_valid_adif_date, is_valid_adif_time, time_to_seconds};
use super::qso::freq_to_band;
use super::band_activity::save_band_activity;
use crate::udp::{UdpMessage, start_listener, QsoLoggedMessage};
use crate::udp::wsjtx::{is_valid_grid, normalize_rst};

#[derive(Debug, Clone, Serialize)]
pub struct UdpStatus {
    pub running: bool,
    pub port: u16,
    pub connected: bool,
    pub wsjtx_version: Option<String>,
}

/// QSO Event for Frontend
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
    pub fn from_wsjtx(qso: &QsoLoggedMessage) -> Self {
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

/// Parse TX message to extract de_call (sender) and dx_call (target)
fn parse_tx_message_calls(message: &str) -> (Option<String>, Option<String>) {
    let parts: Vec<&str> = message.split_whitespace().collect();
    if parts.is_empty() {
        return (None, None);
    }
    
    if parts[0] == "CQ" {
        if parts.len() >= 2 {
            return (Some(parts[1].to_string()), None);
        }
        return (None, None);
    }
    
    if parts.len() >= 2 {
        let dx_call = parts[0].to_string();
        let de_call = parts[1].to_string();
        return (Some(de_call), Some(dx_call));
    }
    
    (None, None)
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
    
    let (tx, mut rx) = mpsc::unbounded_channel::<UdpMessage>();
    
    start_listener(udp_state.clone(), tx)?;
    log::info!("Started UDP listener on port {}", port);
    
    let app_handle = app.clone();
    let db_arc = state.db.clone();
    
    tauri::async_runtime::spawn(async move {
        let mut last_tx_msg = String::new();
        let mut recent_qso_keys: std::collections::VecDeque<String> = std::collections::VecDeque::new();
        const MAX_RECENT_QSOS: usize = 10;
        
        while let Some(msg) = rx.recv().await {
            match msg {
                UdpMessage::Decode(decode) => {
                    if let Some((de_call, dx_call, grid, msg_type)) = crate::udp::parse_ft8_message(&decode.message) {
                        let lookup = crate::reference::lookup_call_full(&de_call);
                        
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
                                None,
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
                            "call": de_call,
                            "grid": grid,
                            "msg_type": format!("{:?}", msg_type),
                            "dxcc": lookup.dxcc_as_i32(),
                            "country": lookup.country,
                            "continent": lookup.continent,
                            "cqz": lookup.cqz,
                            "ituz": lookup.ituz,
                            "low_confidence": decode.low_confidence,
                        }));
                    }
                }
                UdpMessage::QsoLogged(qso) => {
                    let source_type = if qso.id == "TYPE5" { "QsoLogged(5)" } else if qso.id == "TYPE12" { "LoggedADIF(12)" } else { "Unknown" };
                    
                    let freq_mhz = qso.freq_hz / 1_000_000;
                    let qso_key = format!("{}|{}", qso.call.to_uppercase(), freq_mhz);
                    
                    log::warn!("[QSO-HANDLER] Source={} call={} key={}", source_type, qso.call, qso_key);
                    
                    if recent_qso_keys.contains(&qso_key) {
                        log::warn!("[QSO-HANDLER] DUPLICATE BLOCKED: {}", qso.call);
                        continue;
                    }
                    
                    recent_qso_keys.push_back(qso_key.clone());
                    if recent_qso_keys.len() > MAX_RECENT_QSOS {
                        recent_qso_keys.pop_front();
                    }
                    
                    log::info!("Received QSO from WSJT-X: {}", qso.call);
                    
                    let db_guard = db_arc.lock().await;
                    if let Some(pool) = db_guard.as_ref() {
                        if let Err(e) = insert_qso_from_wsjtx(pool, &qso).await {
                            log::error!("Failed to insert QSO: {}", e);
                        } else {
                            log::info!("QSO inserted successfully: {}", qso.call);
                        }
                    }
                    drop(db_guard);
                    
                    let _ = app_handle.emit("qso-logged", QsoEvent::from_wsjtx(&qso));
                }
                UdpMessage::Heartbeat { id, version, .. } => {
                    let _ = app_handle.emit("wsjtx-heartbeat", serde_json::json!({
                        "id": id,
                        "version": version,
                    }));
                }
                UdpMessage::Status { id, dial_freq, mode, dx_call, de_call, report, tx_enabled, transmitting, tx_message, .. } => {
                    if transmitting && !tx_message.is_empty() && tx_message != last_tx_msg {
                        last_tx_msg = tx_message.clone();
                        
                        let db_guard = db_arc.lock().await;
                        if let Some(pool) = db_guard.as_ref() {
                            let (parsed_de_call, parsed_dx_call) = parse_tx_message_calls(&tx_message);
                            let time_utc = get_current_utc_time();
                            
                            let _ = save_band_activity(
                                pool,
                                &time_utc,
                                None,
                                "tx",
                                &tx_message,
                                None,
                                None,
                                parsed_de_call.as_deref(),
                                parsed_dx_call.as_deref(),
                                Some(dial_freq as f64),
                                Some(&mode),
                            ).await;
                        }
                        drop(db_guard);
                    }
                    
                    if !transmitting {
                        last_tx_msg.clear();
                    }
                    
                    let _ = app_handle.emit("wsjtx-status", serde_json::json!({
                        "id": id,
                        "dial_freq": dial_freq,
                        "mode": mode,
                        "dx_call": dx_call,
                        "de_call": de_call,
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
        modifiers: 0x00,
    };
    
    log::info!("Calling station from message: {}", message);
    
    send_reply(&state.udp_state, reply)
}

#[command]
pub async fn get_udp_status(state: tauri::State<'_, AppState>) -> Result<UdpStatus, String> {
    Ok(UdpStatus {
        running: state.udp_state.is_running(),
        port: state.udp_state.get_port(),
        connected: state.udp_state.is_running(),
        wsjtx_version: None,
    })
}

/// Insert a QSO from WSJT-X into the database
async fn insert_qso_from_wsjtx(pool: &sqlx::Pool<sqlx::Sqlite>, qso: &QsoLoggedMessage) -> Result<(), String> {
    use sqlx::Row;
    
    if qso.call.is_empty() {
        return Err("Empty callsign".to_string());
    }
    if qso.mode.is_empty() {
        return Err("Empty mode".to_string());
    }
    if qso.freq_hz == 0 {
        return Err("Zero frequency".to_string());
    }
    
    let uuid = uuid::Uuid::new_v4().to_string();
    let freq_mhz = qso.freq_hz as f64 / 1_000_000.0;
    let band = freq_to_band(freq_mhz);
    let now = chrono::Utc::now();
    
    let (qso_date, time_on) = if !qso.datetime_on.is_empty() {
        if qso.datetime_on.contains('-') && qso.datetime_on.contains(' ') {
            let parts: Vec<&str> = qso.datetime_on.split(' ').collect();
            if parts.len() >= 2 {
                let date_part = parts[0].replace('-', "");
                let time_part = normalize_time_to_hhmmss(parts[1]);
                (date_part, time_part)
            } else {
                (now.format("%Y%m%d").to_string(), normalize_time_to_hhmmss(&qso.datetime_on))
            }
        } else {
            (now.format("%Y%m%d").to_string(), normalize_time_to_hhmmss(&qso.datetime_on))
        }
    } else {
        (now.format("%Y%m%d").to_string(), now.format("%H%M%S").to_string())
    };
    
    if !is_valid_adif_date(&qso_date) {
        return Err(format!("Invalid date format: {}", qso_date));
    }
    if !is_valid_adif_time(&time_on) {
        return Err(format!("Invalid time format: {}", time_on));
    }
    
    let time_seconds = time_to_seconds(&time_on).unwrap_or(0) as i32;
    let exists: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(SELECT 1 FROM qsos 
           WHERE call = ? AND qso_date = ? AND LOWER(band) = LOWER(?) AND mode = ?
           AND ABS((CAST(SUBSTR(time_on, 1, 2) AS INTEGER) * 3600 
                    + CAST(SUBSTR(time_on, 3, 2) AS INTEGER) * 60
                    + CAST(SUBSTR(time_on, 5, 2) AS INTEGER)) - ?) <= 120)"#
    )
    .bind(&qso.call)
    .bind(&qso_date)
    .bind(&band)
    .bind(&qso.mode)
    .bind(time_seconds)
    .fetch_one(pool)
    .await
    .unwrap_or(false);
    
    if exists {
        log::info!("Skipping duplicate QSO: {} on {}", qso.call, band);
        return Ok(());
    }
    
    let lookup = crate::reference::lookup_call_full(&qso.call);
    // Convert DXCC from ARRL 3-digit string to integer for database storage
    let dxcc_int = lookup.dxcc_as_i32();
    
    let adif_fields = serde_json::json!({
        "name": if qso.name.is_empty() { None } else { Some(&qso.name) },
        "comments": if qso.comments.is_empty() { None } else { Some(&qso.comments) },
        "tx_pwr": if qso.tx_power.is_empty() { None } else { Some(&qso.tx_power) },
        "operator": if qso.operator_call.is_empty() { None } else { Some(&qso.operator_call) },
        "prop_mode": if qso.adif_propagation_mode.is_empty() { None } else { Some(&qso.adif_propagation_mode) },
    }).to_string();
    
    // Validate grid before storing
    let validated_grid = if is_valid_grid(&qso.grid) { Some(&qso.grid) } else { None };
    
    // Normalize RST values
    let rst_sent = normalize_rst(&qso.report_sent);
    let rst_rcvd = normalize_rst(&qso.report_rcvd);
    
    sqlx::query(
        r#"INSERT INTO qsos (
            uuid, call, qso_date, time_on, time_off, band, mode, freq,
            dxcc, country, continent, cqz, ituz, gridsquare,
            rst_sent, rst_rcvd, station_callsign, my_gridsquare,
            adif_fields, source, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'WSJT-X', datetime('now'), datetime('now'))"#
    )
    .bind(&uuid)
    .bind(&qso.call)
    .bind(&qso_date)
    .bind(&time_on)
    .bind(if qso.datetime_off.is_empty() { None } else { Some(&qso.datetime_off) })
    .bind(&band)
    .bind(&qso.mode)
    .bind(freq_mhz)
    .bind(dxcc_int)
    .bind(&lookup.country)
    .bind(&lookup.continent)
    .bind(lookup.cqz)
    .bind(lookup.ituz)
    .bind(validated_grid)
    .bind(&rst_sent)
    .bind(&rst_rcvd)
    .bind(if qso.my_call.is_empty() { None } else { Some(&qso.my_call) })
    .bind(if qso.my_grid.is_empty() { None } else { Some(&qso.my_grid) })
    .bind(&adif_fields)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    log::info!("Inserted QSO: {} on {}", qso.call, band);
    Ok(())
}
