//! LoTW (Logbook of The World) sync commands
//!
//! This module handles:
//! - sync_lotw_download: Download confirmations from LoTW
//! - get_sync_status: Get upload/download status
//! - detect_tqsl_path: Find TQSL installation
//! - upload_to_lotw: Upload QSOs via TQSL

use serde::Serialize;
use sqlx::Row;
use std::io::Write;
use tauri::command;

use super::adif::row_to_json;
use super::state::AppState;
use super::time_utils::extract_hhmm;

// ============================================================================
// Data Types
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

#[derive(Debug, Serialize, Clone)]
pub struct UnmatchedQso {
    pub call: String,
    pub qso_date: String,
    pub time_on: String,
    pub band: String,
    pub mode: String,
}

#[derive(Debug, Serialize)]
pub struct LotwDownloadResult {
    pub total_records: i32,
    pub matched: i32,
    pub unmatched: i32,
    pub unmatched_qsos: Vec<UnmatchedQso>,
    pub errors: Vec<String>,
    pub last_qsl: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LotwUploadResult {
    pub qsos_exported: usize,
    pub success: bool,
    pub message: String,
}

// ============================================================================
// Commands
// ============================================================================

#[command]
pub async fn sync_lotw_download(
    state: tauri::State<'_, AppState>,
    username: String,
    password: String,
    since_date: Option<String>,
) -> Result<LotwDownloadResult, String> {
    log::info!("Starting LoTW confirmation download, since_date={:?}", since_date);

    use crate::lotw::{LotwClient, LotwQueryOptions};

    let client = LotwClient::new(username, password);

    let options = LotwQueryOptions {
        qso_qslsince: since_date.clone(),
        qso_qsldetail: true,
        qso_withown: true,
        ..Default::default()
    };

    log::info!("LoTW query options: qso_qslsince={:?}", since_date);

    let result = client
        .download_confirmations(&options)
        .await
        .map_err(|e| e.to_string())?;

    log::info!(
        "Downloaded {} bytes from LoTW, last_qsl={:?}",
        result.adif_content.len(),
        result.last_qsl
    );

    use crate::adif::parse_adif;
    let adif_file =
        parse_adif(&result.adif_content).map_err(|e| format!("Failed to parse LoTW response: {}", e))?;

    log::info!("Parsed {} QSL records from LoTW", adif_file.records.len());

    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    let mut matched = 0;
    let mut unmatched = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut unmatched_qsos: Vec<UnmatchedQso> = Vec::new();

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

        let time_prefix = extract_hhmm(&time_on);

        let match_result = sqlx::query(
            r#"SELECT id FROM qsos 
               WHERE UPPER(call) = ? AND UPPER(band) = ? AND qso_date = ?
                 AND SUBSTR(time_on, 1, 4) = ?
               LIMIT 1"#,
        )
        .bind(call.to_uppercase())
        .bind(&band)
        .bind(&qso_date)
        .bind(&time_prefix)
        .fetch_optional(pool)
        .await;

        match match_result {
            Ok(Some(row)) => {
                let qso_id: i64 = row.get("id");

                let qsl_date = record.get("QSLRDATE").map(|s| s.to_string());
                let dxcc: Option<i32> = record.get("DXCC").and_then(|s| s.parse().ok());
                let state = record.get("STATE").map(|s| s.to_string());
                let gridsquare = record.get("GRIDSQUARE").map(|s| s.to_string());
                let cqz: Option<i32> = record.get("CQZ").and_then(|s| s.parse().ok());
                let ituz: Option<i32> = record.get("ITUZ").and_then(|s| s.parse().ok());
                let country = record.get("COUNTRY").map(|s| s.to_string());
                let credit_granted = record.get("APP_LOTW_CREDIT_GRANTED").map(|s| s.to_string());

                sqlx::query(
                    r#"INSERT INTO confirmations (qso_id, source, qsl_rcvd, qsl_rcvd_date, credit_granted, verified_at)
                       VALUES (?, 'LOTW', 'Y', ?, ?, datetime('now'))
                       ON CONFLICT(qso_id, source) DO UPDATE SET
                         qsl_rcvd = 'Y',
                         qsl_rcvd_date = COALESCE(excluded.qsl_rcvd_date, qsl_rcvd_date),
                         credit_granted = COALESCE(excluded.credit_granted, credit_granted),
                         verified_at = datetime('now')"#,
                )
                .bind(qso_id)
                .bind(&qsl_date)
                .bind(&credit_granted)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to insert confirmation: {}", e))?;

                sqlx::query(
                    r#"UPDATE qsos SET 
                       dxcc = COALESCE(?, dxcc),
                       country = COALESCE(?, country),
                       state = COALESCE(?, state),
                       gridsquare = COALESCE(?, gridsquare),
                       cqz = COALESCE(?, cqz),
                       ituz = COALESCE(?, ituz),
                       updated_at = datetime('now')
                       WHERE id = ?"#,
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
                unmatched_qsos.push(UnmatchedQso {
                    call: call.clone(),
                    qso_date: qso_date.clone(),
                    time_on: time_on.clone(),
                    band: band.clone(),
                    mode: mode.clone(),
                });
                log::warn!(
                    "No local QSO for LoTW QSL: {} on {} {} at {} ({})",
                    call,
                    band,
                    qso_date,
                    time_on,
                    mode
                );
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
        unmatched_qsos,
        errors,
        last_qsl: result.last_qsl,
    })
}

#[command]
pub async fn get_sync_status(state: tauri::State<'_, AppState>) -> Result<SyncStatus, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

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
            EXISTS(SELECT 1 FROM settings WHERE key = 'lotw_username' AND value IS NOT NULL AND value != '') as has_creds",
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

#[command]
pub async fn upload_to_lotw(
    state: tauri::State<'_, AppState>,
    tqsl_path: String,
) -> Result<LotwUploadResult, String> {
    log::info!("Starting LoTW upload via TQSL");

    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    let rows = sqlx::query(
        r#"
        SELECT q.* FROM qsos q
        LEFT JOIN confirmations c ON q.id = c.qso_id AND c.source = 'LOTW'
        WHERE c.id IS NULL OR c.qsl_sent IS NULL OR c.qsl_sent != 'Y'
        ORDER BY q.qso_date DESC, q.time_on DESC
        "#,
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

    let qso_ids: Vec<i64> = rows.iter().map(|r| r.get::<i64, _>("id")).collect();
    let qsos: Vec<serde_json::Value> = rows.iter().map(|r| row_to_json(r)).collect();
    let qso_count = qsos.len();

    log::info!("Exporting {} QSOs for LoTW upload", qso_count);

    let records: Vec<std::collections::HashMap<String, String>> = qsos
        .iter()
        .map(|q| crate::adif::writer::qso_to_adif(q))
        .collect();

    let adif_content = crate::adif::write_adif(&records, "GoQSO");

    let temp_dir = std::env::temp_dir();
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let temp_file = temp_dir.join(format!("goqso_upload_{}.adi", timestamp));

    let mut file =
        std::fs::File::create(&temp_file).map_err(|e| format!("Failed to create temp file: {}", e))?;
    file.write_all(adif_content.as_bytes())
        .map_err(|e| format!("Failed to write ADIF file: {}", e))?;

    log::info!("Wrote ADIF to: {}", temp_file.display());

    let output = std::process::Command::new(&tqsl_path)
        .args(["-d", "-u", "-a", "compliant", "-x"])
        .arg(&temp_file)
        .output()
        .map_err(|e| format!("Failed to execute TQSL: {}", e))?;

    let _ = std::fs::remove_file(&temp_file);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    log::info!("TQSL exit code: {:?}", output.status.code());
    log::info!("TQSL stdout: {}", stdout);
    log::info!("TQSL stderr: {}", stderr);

    let success = matches!(output.status.code(), Some(0) | Some(9));

    if matches!(output.status.code(), Some(0) | Some(8) | Some(9)) {
        let today = chrono::Utc::now().format("%Y%m%d").to_string();
        for qso_id in &qso_ids {
            sqlx::query(
                r#"
                INSERT INTO confirmations (qso_id, source, qsl_sent, qsl_sent_date)
                VALUES (?, 'LOTW', 'Y', ?)
                ON CONFLICT(qso_id, source) DO UPDATE SET
                    qsl_sent = 'Y',
                    qsl_sent_date = COALESCE(confirmations.qsl_sent_date, excluded.qsl_sent_date)
                "#,
            )
            .bind(qso_id)
            .bind(&today)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to mark QSO {} as sent: {}", qso_id, e))?;
        }
        log::info!("Marked {} QSOs as sent to LoTW", qso_ids.len());

        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES ('lotw_last_upload', ?, ?)",
        )
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to save last upload date: {}", e))?;
    }

    let message = match output.status.code() {
        Some(0) => format!("Successfully uploaded {} QSO(s) to LoTW", qso_count),
        Some(9) => "Uploaded QSOs to LoTW - some duplicates skipped".to_string(),
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
