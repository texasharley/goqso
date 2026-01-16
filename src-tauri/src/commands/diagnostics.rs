//! Diagnostic commands for troubleshooting QSO data
//!
//! This module provides:
//! - get_qso_diagnostics: Detailed statistics and potential issues

use serde::Serialize;
use tauri::command;

use super::state::AppState;

// ============================================================================
// Data Types
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

// ============================================================================
// Diagnostic Commands
// ============================================================================

/// Get diagnostic information about QSO data for troubleshooting
#[command]
pub async fn get_qso_diagnostics(
    state: tauri::State<'_, AppState>,
) -> Result<DiagnosticReport, String> {
    let db_guard = state.db.lock().await;
    let pool = db_guard.as_ref().ok_or("Database not initialized")?;

    // Total QSOs
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM qsos")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    // Confirmed count (has LoTW confirmation with qsl_rcvd='Y')
    let confirmed: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(DISTINCT q.id) FROM qsos q
           JOIN confirmations c ON c.qso_id = q.id
           WHERE c.source = 'LOTW' AND c.qsl_rcvd = 'Y'"#,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    // QSOs by source
    let by_source: Vec<(String, i64)> = sqlx::query_as(
        "SELECT COALESCE(source, 'unknown') as src, COUNT(*) as cnt FROM qsos GROUP BY source ORDER BY cnt DESC",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Find potential duplicates (same call+date+band but different times within 5 min)
    let dupe_candidates: Vec<(String,)> = sqlx::query_as(
        r#"SELECT DISTINCT a.call || ' on ' || a.qso_date || ' ' || a.band 
           FROM qsos a 
           JOIN qsos b ON a.call = b.call AND a.qso_date = b.qso_date AND a.band = b.band AND a.id != b.id
           WHERE ABS(CAST(SUBSTR(a.time_on, 1, 2) * 60 + SUBSTR(a.time_on, 3, 2) AS INTEGER) - 
                     CAST(SUBSTR(b.time_on, 1, 2) * 60 + SUBSTR(b.time_on, 3, 2) AS INTEGER)) < 5
           LIMIT 20"#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // QSOs that might not be in LoTW (before Feb 2023 or different characteristics)
    let not_in_lotw: Vec<QsoDiagnostic> = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, bool)>(
        r#"SELECT q.call, q.qso_date, q.time_on, q.band, q.mode, q.source,
           EXISTS(SELECT 1 FROM confirmations c WHERE c.qso_id = q.id AND c.source = 'LOTW') as has_lotw
           FROM qsos q
           WHERE q.qso_date < '20230204' OR q.source = 'ADIF'
           ORDER BY q.qso_date DESC
           LIMIT 20"#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(call, date, time, band, mode, source, has_lotw)| QsoDiagnostic {
        call,
        qso_date: date,
        time_on: time,
        band,
        mode,
        source,
        has_lotw_confirmation: has_lotw,
    })
    .collect();

    Ok(DiagnosticReport {
        total_qsos: total.0,
        confirmed_count: confirmed.0,
        pending_count: total.0 - confirmed.0,
        by_source,
        duplicate_candidates: dupe_candidates.into_iter().map(|(s,)| s).collect(),
        qsos_not_in_lotw_window: not_in_lotw,
    })
}
