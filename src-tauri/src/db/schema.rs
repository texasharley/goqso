// Database schema definitions
// These are used as reference - actual tables created via SQL migrations
//
// NOTE: Schema structs defined here for documentation/reference purposes.
// Actual DB operations use dynamic queries. These may be used in future for ORM-like access.
//
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// QSO record - ADIF 3.1.4 compatible
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Qso {
    pub id: i64,
    pub uuid: String,
    
    // Required fields
    pub call: String,
    pub qso_date: String,      // YYYYMMDD
    pub time_on: String,       // HHMMSS
    pub band: String,          // e.g., "20m"
    pub mode: String,          // e.g., "FT8"
    
    // Frequency
    pub freq: Option<f64>,     // MHz
    pub freq_rx: Option<f64>,  // For split
    
    // Location data (from CTY lookup)
    pub dxcc: Option<i32>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub gridsquare: Option<String>,
    pub cqz: Option<i32>,
    pub ituz: Option<i32>,
    
    // Signal reports
    pub rst_sent: Option<String>,
    pub rst_rcvd: Option<String>,
    
    // Station info
    pub station_callsign: Option<String>,
    pub my_gridsquare: Option<String>,
    pub tx_pwr: Option<f64>,
    
    // Metadata
    pub source: String,        // 'wsjt-x', 'adif-import', 'manual'
    pub created_at: String,
    pub updated_at: String,
}

/// Confirmation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confirmation {
    pub id: i64,
    pub qso_id: i64,
    pub source: String,        // 'lotw', 'eqsl', 'qrz', 'card'
    pub confirmed_at: String,
    pub lotw_qsl_date: Option<String>,
    pub lotw_credit_granted: Option<String>,
}

/// Sync queue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncQueueEntry {
    pub id: i64,
    pub qso_id: i64,
    pub action: String,        // 'upload'
    pub status: String,        // 'pending', 'in_progress', 'completed', 'failed'
    pub attempts: i32,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Award progress entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwardProgress {
    pub id: i64,
    pub award_type: String,    // 'dxcc', 'was', 'vucc'
    pub target_id: String,     // Entity number, state abbrev, or grid
    pub band: Option<String>,
    pub mode: Option<String>,
    pub worked_qso_id: Option<i64>,
    pub confirmed_qso_id: Option<i64>,
    pub credited: bool,
    pub updated_at: String,
}

/// DXCC entity reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DxccEntity {
    pub entity_code: i32,
    pub entity_name: String,
    pub cq_zone: Option<i32>,
    pub itu_zone: Option<i32>,
    pub continent: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub utc_offset: Option<f64>,
    pub is_deleted: bool,
}

/// Callsign prefix lookup entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallsignPrefix {
    pub id: i64,
    pub prefix: String,
    pub entity_code: i32,
    pub cq_zone: Option<i32>,
    pub itu_zone: Option<i32>,
    pub is_exact: bool,
}
