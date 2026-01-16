//! ADIF import/export commands
//!
//! This module handles:
//! - import_adif: Import QSOs from ADIF file content
//! - export_adif: Export QSOs to ADIF format
//! - import_lotw_confirmations: Import LoTW confirmation data

use serde::Serialize;
use sqlx::Row;
use tauri::command;

use super::state::AppState;
use super::time_utils::{extract_hhmm, normalize_time_to_hhmmss, time_to_seconds};
use crate::udp::wsjtx::{is_valid_grid, normalize_rst};

// ============================================================================
// Data Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub total_records: usize,
    pub imported: usize,
    pub skipped: usize,
    pub errors: usize,
    pub error_messages: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct LotwImportResult {
    pub total_records: usize,
    pub matched: usize,
    pub not_found: usize,
    pub already_confirmed: usize,
    pub errors: usize,
}

// ============================================================================
// Helper Functions
// ============================================================================

pub fn row_to_json(row: &sqlx::sqlite::SqliteRow) -> serde_json::Value {
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
// Commands
// ============================================================================

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
            result
                .error_messages
                .push(format!("Record for {} missing required fields", call));
            continue;
        }

        let time_on_normalized = normalize_time_to_hhmmss(&time_on);

        // Validate grid (reject FT8 messages like "RR73")
        let gridsquare = record
            .gridsquare()
            .filter(|g| is_valid_grid(g))
            .map(|g| g.to_uppercase());

        // Normalize RST values
        let rst_sent = record.get("RST_SENT").map(|r| normalize_rst(r));
        let rst_rcvd = record.get("RST_RCVD").map(|r| normalize_rst(r));

        // Check for duplicate using ±120 second window
        if skip_duplicates {
            let incoming_seconds = time_to_seconds(&time_on_normalized);

            let existing_times: Vec<String> = sqlx::query_scalar(
                "SELECT time_on FROM qsos WHERE call = ? AND qso_date = ? AND LOWER(band) = LOWER(?) AND UPPER(mode) = UPPER(?)",
            )
            .bind(&call)
            .bind(&qso_date)
            .bind(&band)
            .bind(&mode)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            let is_duplicate = existing_times.iter().any(|existing_time| {
                if let (Some(incoming), Some(existing)) =
                    (incoming_seconds, time_to_seconds(existing_time))
                {
                    let diff = (incoming as i32 - existing as i32).abs();
                    let diff = diff.min(86400 - diff);
                    diff <= 120
                } else {
                    extract_hhmm(&time_on) == extract_hhmm(existing_time)
                }
            });

            if is_duplicate {
                result.skipped += 1;
                continue;
            }
        }

        // Build adif_fields JSON for extended fields
        let mut adif_fields = serde_json::Map::new();
        for (key, value) in &record.fields {
            let core_fields = [
                "CALL",
                "QSO_DATE",
                "QSO_DATE_OFF",
                "TIME_ON",
                "TIME_OFF",
                "BAND",
                "MODE",
                "FREQ",
                "DXCC",
                "COUNTRY",
                "STATE",
                "CNTY",
                "GRIDSQUARE",
                "CQZ",
                "ITUZ",
                "CONT",
                "RST_SENT",
                "RST_RCVD",
                "STATION_CALLSIGN",
                "MY_GRIDSQUARE",
                "TX_PWR",
                "OPERATOR",
            ];
            if !core_fields.contains(&key.as_str()) && !key.starts_with("APP_") {
                adif_fields.insert(key.to_lowercase(), serde_json::Value::String(value.clone()));
            }
        }

        let uuid = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let insert_result = sqlx::query(
            r#"INSERT INTO qsos (
                uuid, call, qso_date, qso_date_off, time_on, time_off, band, mode, submode, freq,
                dxcc, country, state, cnty, gridsquare, continent, cqz, ituz,
                rst_sent, rst_rcvd, station_callsign, operator, my_gridsquare, tx_pwr,
                prop_mode, sat_name, iota, pota_ref, sota_ref, wwff_ref, pfx,
                name, qth, comment, arrl_sect,
                my_cnty, my_arrl_sect, my_sota_ref, my_pota_ref,
                adif_fields, source, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&uuid)
        .bind(&call)
        .bind(qso_date)
        .bind(record.get("QSO_DATE_OFF"))
        .bind(&time_on_normalized)
        .bind(record.get("TIME_OFF"))
        .bind(&band)
        .bind(&mode)
        .bind(record.get("SUBMODE"))
        .bind(record.freq())
        .bind(record.dxcc())
        .bind(record.country())
        .bind(record.state())
        .bind(record.cnty())
        .bind(&gridsquare)
        .bind(record.get("CONT"))
        .bind(record.cqz())
        .bind(record.ituz())
        .bind(&rst_sent)
        .bind(&rst_rcvd)
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

    log::info!(
        "ADIF import: {} imported, {} skipped, {} errors",
        result.imported,
        result.skipped,
        result.errors
    );

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
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT * FROM qsos WHERE id IN ({}) ORDER BY qso_date DESC, time_on DESC",
            placeholders
        );
        let mut q = sqlx::query(&query);
        for id in &ids {
            q = q.bind(id);
        }
        let rows = q.fetch_all(pool).await.map_err(|e| e.to_string())?;
        rows.iter().map(|r| row_to_json(r)).collect()
    } else {
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

        let matching_qso: Option<(i64,)> = sqlx::query_as(
            r#"SELECT id FROM qsos 
               WHERE call = ? AND band = ? AND mode = ? AND qso_date = ?
               ORDER BY ABS(
                   CAST(SUBSTR(time_on, 1, 2) AS INTEGER) * 60 + CAST(SUBSTR(time_on, 3, 2) AS INTEGER) -
                   CAST(SUBSTR(?, 1, 2) AS INTEGER) * 60 - CAST(SUBSTR(?, 3, 2) AS INTEGER)
               )
               LIMIT 1"#,
        )
        .bind(&call)
        .bind(&band)
        .bind(&mode)
        .bind(qso_date)
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

        let already_confirmed: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM confirmations WHERE qso_id = ? AND source = 'LOTW' AND qsl_rcvd = 'Y')",
        )
        .bind(qso_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if already_confirmed {
            result.already_confirmed += 1;
            continue;
        }

        let qslrdate = record.qslrdate().map(|s| s.as_str()).unwrap_or("");
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let insert_result = sqlx::query(
            r#"INSERT INTO confirmations (qso_id, source, qsl_rcvd, qsl_rcvd_date, verified_at)
               VALUES (?, 'LOTW', 'Y', ?, ?)
               ON CONFLICT(qso_id, source) DO UPDATE SET 
                   qsl_rcvd = 'Y', 
                   qsl_rcvd_date = excluded.qsl_rcvd_date,
                   verified_at = excluded.verified_at"#,
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

        // Update QSO with LoTW data
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

    log::info!(
        "LoTW import: {} matched, {} not found, {} already confirmed",
        result.matched,
        result.not_found,
        result.already_confirmed
    );

    Ok(result)
}
