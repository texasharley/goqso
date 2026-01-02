use serde::{Deserialize, Serialize};
use tauri::command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Shared state for UDP listener
static UDP_RUNNING: AtomicBool = AtomicBool::new(false);

// ============================================================================
// UDP Listener Commands
// ============================================================================

#[command]
pub async fn start_udp_listener(port: u16) -> Result<(), String> {
    if UDP_RUNNING.load(Ordering::SeqCst) {
        return Err("UDP listener already running".to_string());
    }
    
    UDP_RUNNING.store(true, Ordering::SeqCst);
    log::info!("Starting UDP listener on port {}", port);
    
    // TODO: Spawn actual UDP listener thread
    Ok(())
}

#[command]
pub async fn stop_udp_listener() -> Result<(), String> {
    UDP_RUNNING.store(false, Ordering::SeqCst);
    log::info!("Stopping UDP listener");
    Ok(())
}

#[command]
pub async fn get_udp_status() -> Result<bool, String> {
    Ok(UDP_RUNNING.load(Ordering::SeqCst))
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
    pub band: String,
    pub mode: String,
    pub freq: Option<f64>,
    pub dxcc: Option<i32>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub gridsquare: Option<String>,
    pub rst_sent: Option<String>,
    pub rst_rcvd: Option<String>,
    pub source: String,
    pub created_at: String,
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
pub async fn get_qsos(limit: i32, offset: i32) -> Result<Vec<Qso>, String> {
    // TODO: Query SQLite via tauri-plugin-sql
    log::info!("Getting QSOs: limit={}, offset={}", limit, offset);
    Ok(vec![])
}

#[command]
pub async fn add_qso(qso: NewQso) -> Result<Qso, String> {
    log::info!("Adding QSO: {}", qso.call);
    // TODO: Insert into database, enrich with CTY lookup
    Err("Not implemented".to_string())
}

#[command]
pub async fn delete_qso(id: i64) -> Result<(), String> {
    log::info!("Deleting QSO: {}", id);
    // TODO: Delete from database
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
