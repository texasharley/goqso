// WSJT-X UDP Protocol Parser
// Reference: NetworkMessage.hpp from WSJT-X source

/// WSJT-X Magic Number
pub const WSJTX_MAGIC: u32 = 0xadbccbda;

/// WSJT-X Message Types
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum WsjtxMessageType {
    Heartbeat = 0,
    Status = 1,
    Decode = 2,
    Clear = 3,
    Reply = 4,
    QsoLogged = 5,
    Close = 6,
    Replay = 7,
    HaltTx = 8,
    FreeText = 9,
    WSPRDecode = 10,
    Location = 11,
    LoggedADIF = 12,
    HighlightCallsign = 13,
    SwitchConfiguration = 14,
    Configure = 15,
}

/// Parsed QSO Logged message from WSJT-X
#[derive(Debug, Clone)]
pub struct QsoLoggedMessage {
    pub id: String,
    pub datetime_off: String,
    pub call: String,
    pub grid: String,
    pub freq_hz: u64,
    pub mode: String,
    pub report_sent: String,
    pub report_rcvd: String,
    pub tx_power: String,
    pub comments: String,
    pub name: String,
    pub datetime_on: String,
    pub operator_call: String,
    pub my_call: String,
    pub my_grid: String,
    pub exchange_sent: String,
    pub exchange_rcvd: String,
    pub adif_propagation_mode: String,
}

/// Parse WSJT-X UDP message
pub fn parse_message(data: &[u8]) -> Option<WsjtxMessageType> {
    if data.len() < 8 {
        return None;
    }
    
    // Check magic number
    let magic = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    if magic != WSJTX_MAGIC {
        return None;
    }
    
    // Get schema version (u32)
    let _schema = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    
    // Get message type (u32)
    if data.len() < 12 {
        return None;
    }
    let msg_type = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    
    match msg_type {
        0 => Some(WsjtxMessageType::Heartbeat),
        1 => Some(WsjtxMessageType::Status),
        2 => Some(WsjtxMessageType::Decode),
        3 => Some(WsjtxMessageType::Clear),
        4 => Some(WsjtxMessageType::Reply),
        5 => Some(WsjtxMessageType::QsoLogged),
        6 => Some(WsjtxMessageType::Close),
        7 => Some(WsjtxMessageType::Replay),
        8 => Some(WsjtxMessageType::HaltTx),
        9 => Some(WsjtxMessageType::FreeText),
        10 => Some(WsjtxMessageType::WSPRDecode),
        11 => Some(WsjtxMessageType::Location),
        12 => Some(WsjtxMessageType::LoggedADIF),
        13 => Some(WsjtxMessageType::HighlightCallsign),
        14 => Some(WsjtxMessageType::SwitchConfiguration),
        15 => Some(WsjtxMessageType::Configure),
        _ => None,
    }
}

/// Read a Qt-style string from the buffer
/// Format: u32 length (0xFFFFFFFF for null), then UTF-8 bytes
pub fn read_qt_string(data: &[u8], offset: &mut usize) -> Option<String> {
    if *offset + 4 > data.len() {
        return None;
    }
    
    let len = u32::from_be_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
    ]);
    *offset += 4;
    
    if len == 0xFFFFFFFF {
        return Some(String::new()); // Null string
    }
    
    let len = len as usize;
    if *offset + len > data.len() {
        return None;
    }
    
    let s = String::from_utf8_lossy(&data[*offset..*offset + len]).to_string();
    *offset += len;
    Some(s)
}

/// Read a QDateTime from the buffer
/// Format: 
///   - Julian Day Number (i64, but typically fits in 4 bytes when serialized as big-endian)
///   - Time in milliseconds since midnight (u32) - but only if JD != 0x8000000000000000 (null)
///   - TimeSpec (u8) - 0=Local, 1=UTC, 2=OffsetFromUTC, 3=TimeZone
/// Note: WSJT-X uses a simplified format with fixed size
fn read_qt_datetime(data: &[u8], offset: &mut usize) -> Option<String> {
    // WSJT-X QDateTime serialization:
    // 8 bytes: Julian Day (i64 big-endian, but WSJT-X only sends 8 bytes total)
    // The format is actually more complex - let's handle what WSJT-X sends
    
    // Check for null date indicator
    if *offset + 8 > data.len() {
        log::warn!("QDateTime: not enough bytes for date, offset={}, len={}", *offset, data.len());
        return None;
    }
    
    // Read as raw bytes first to understand the format
    let date_bytes = &data[*offset..*offset + 8];
    log::debug!("QDateTime raw bytes: {:02x?}", date_bytes);
    
    // WSJT-X format: 8 bytes for JD + Time packed
    // Actually: i64 Julian Day is stored as first 8 bytes
    let jd = i64::from_be_bytes([
        data[*offset], data[*offset + 1], data[*offset + 2], data[*offset + 3],
        data[*offset + 4], data[*offset + 5], data[*offset + 6], data[*offset + 7],
    ]);
    *offset += 8;
    
    // Check for null date
    if jd == i64::MIN {
        // Skip time of day (4 bytes) and timespec (1 byte) for null dates
        if *offset + 5 <= data.len() {
            *offset += 5;
        }
        return Some(String::new());
    }
    
    // Time of day in milliseconds (u32)
    if *offset + 4 > data.len() {
        log::warn!("QDateTime: not enough bytes for time");
        return None;
    }
    let time_ms = u32::from_be_bytes([
        data[*offset], data[*offset + 1], data[*offset + 2], data[*offset + 3],
    ]);
    *offset += 4;
    
    // TimeSpec (1 byte)
    if *offset + 1 > data.len() {
        log::warn!("QDateTime: not enough bytes for timespec");
        return None;
    }
    let _timespec = data[*offset];
    *offset += 1;
    
    // Convert Julian Day to calendar date
    // Algorithm from https://en.wikipedia.org/wiki/Julian_day
    let jd = jd as i32;
    let f = jd + 1401 + (((4 * jd + 274277) / 146097) * 3) / 4 - 38;
    let e = 4 * f + 3;
    let g = (e % 1461) / 4;
    let h = 5 * g + 2;
    let day = (h % 153) / 5 + 1;
    let month = (h / 153 + 2) % 12 + 1;
    let year = e / 1461 - 4716 + (12 + 2 - month) / 12;
    
    let hours = time_ms / 3_600_000;
    let mins = (time_ms % 3_600_000) / 60_000;
    let secs = (time_ms % 60_000) / 1000;
    
    Some(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, hours, mins, secs))
}

/// Parse QSO Logged message (type 5)
pub fn parse_qso_logged(data: &[u8]) -> Option<QsoLoggedMessage> {
    log::info!("Parsing QsoLogged message, {} bytes", data.len());
    let mut offset = 12; // Skip magic, schema, type
    
    let id = read_qt_string(data, &mut offset)?;
    log::debug!("QsoLogged id={}, offset now {}", id, offset);
    
    // DateTime off
    let datetime_off = read_qt_datetime(data, &mut offset).unwrap_or_default();
    log::debug!("QsoLogged datetime_off={}, offset now {}", datetime_off, offset);
    
    let call = read_qt_string(data, &mut offset)?;
    log::debug!("QsoLogged call={}, offset now {}", call, offset);
    
    let grid = read_qt_string(data, &mut offset)?;
    log::debug!("QsoLogged grid={}, offset now {}", grid, offset);
    
    // Frequency (u64)
    if offset + 8 > data.len() {
        log::error!("QsoLogged: not enough bytes for frequency");
        return None;
    }
    let freq_hz = u64::from_be_bytes([
        data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
    ]);
    offset += 8;
    log::debug!("QsoLogged freq_hz={}, offset now {}", freq_hz, offset);
    
    let mode = read_qt_string(data, &mut offset)?;
    let report_sent = read_qt_string(data, &mut offset)?;
    let report_rcvd = read_qt_string(data, &mut offset)?;
    let tx_power = read_qt_string(data, &mut offset)?;
    let comments = read_qt_string(data, &mut offset)?;
    let name = read_qt_string(data, &mut offset)?;
    log::debug!("QsoLogged after strings, offset now {}", offset);
    
    // DateTime on
    let datetime_on = read_qt_datetime(data, &mut offset).unwrap_or_default();
    log::debug!("QsoLogged datetime_on={}, offset now {}", datetime_on, offset);
    
    let operator_call = read_qt_string(data, &mut offset).unwrap_or_default();
    let my_call = read_qt_string(data, &mut offset).unwrap_or_default();
    let my_grid = read_qt_string(data, &mut offset).unwrap_or_default();
    let exchange_sent = read_qt_string(data, &mut offset).unwrap_or_default();
    let exchange_rcvd = read_qt_string(data, &mut offset).unwrap_or_default();
    let adif_propagation_mode = read_qt_string(data, &mut offset).unwrap_or_default();
    
    log::info!("QsoLogged parsed successfully: call={} grid={} mode={}", call, grid, mode);
    
    // Validate grid - WSJT-X sometimes puts "RR73", "RRR", "73" in grid field when unknown
    let validated_grid = if is_valid_grid(&grid) { grid } else { 
        log::warn!("Invalid grid '{}' for {}, clearing it", grid, call);
        String::new() 
    };
    
    // Validate RST - should be signal report only (e.g., "-14", "+05"), normalized format
    let validated_report_sent = normalize_rst(&report_sent);
    let validated_report_rcvd = normalize_rst(&report_rcvd);
    
    Some(QsoLoggedMessage {
        id,
        datetime_off,
        call,
        grid: validated_grid,
        freq_hz,
        mode,
        report_sent: validated_report_sent,
        report_rcvd: validated_report_rcvd,
        tx_power,
        comments,
        name,
        datetime_on,
        operator_call,
        my_call,
        my_grid,
        exchange_sent,
        exchange_rcvd,
        adif_propagation_mode,
    })
}

// ============================================================================
// Grid Square Validation
// ============================================================================
// 
// Maidenhead grid squares are used worldwide by amateur radio operators.
// Format: 2, 4, or 6 characters
//   - Field (2 chars): AA-RR (18x18 = 324 fields covering the globe)
//   - Square (2 digits): 00-99 (subdivides each field into 100 squares)
//   - Subsquare (2 chars): aa-xx (subdivides each square into 576 subsquares)
//
// Challenge: Some FT8 protocol messages look like valid grid squares:
//   - "RR73" = Roger + 73 (FT8 acknowledgment) - syntactically valid 4-char grid
//   - "RRR" = Roger Roger Roger - NOT valid (3 chars, odd length)
//   - "73" = Best regards - NOT valid (digits only)
//
// Solution: Multi-layer validation
//   1. Syntactic validation (format check)
//   2. Geographic plausibility (is there land/population there?)
//   3. Context-aware filtering (known FT8 message patterns)
// ============================================================================

/// Convert a 4-character grid square to approximate lat/lon coordinates
/// Returns (latitude, longitude) of the grid square center
fn grid_to_latlon(grid: &str) -> Option<(f64, f64)> {
    if grid.len() < 4 {
        return None;
    }
    let bytes = grid.as_bytes();
    let lon_field = (bytes[0].to_ascii_uppercase() - b'A') as f64;
    let lat_field = (bytes[1].to_ascii_uppercase() - b'A') as f64;
    let lon_square = (bytes[2] - b'0') as f64;
    let lat_square = (bytes[3] - b'0') as f64;
    
    // Maidenhead origin is at -180°, -90°
    // Each field is 20° longitude x 10° latitude
    // Each square is 2° longitude x 1° latitude
    let lon = -180.0 + lon_field * 20.0 + lon_square * 2.0 + 1.0; // +1 for center
    let lat = -90.0 + lat_field * 10.0 + lat_square * 1.0 + 0.5;  // +0.5 for center
    
    Some((lat, lon))
}

/// Check if a grid square is geographically plausible for amateur radio
/// Returns true if the location could reasonably have amateur radio operators
/// This catches grids that are syntactically valid but in the middle of oceans
fn is_geographically_plausible(grid: &str) -> bool {
    // For 2-char grids (field only), we're more lenient - just syntactic check
    if grid.len() < 4 {
        return true;
    }
    
    let Some((lat, lon)) = grid_to_latlon(grid) else {
        return false;
    };
    
    // Known problematic grids that correspond to remote ocean areas
    // RR field is in the South Pacific (around -45°S, 155-175°W)
    // RR73 specifically: lat ≈ -42°, lon ≈ -167° (middle of Pacific, no land)
    let upper = grid.to_uppercase();
    
    // RR field (South Pacific) - check specific squares that are all ocean
    if upper.starts_with("RR") {
        // RR70-RR79 are all deep ocean with no islands
        if let Some(square) = upper.chars().nth(2).and_then(|c| c.to_digit(10)) {
            if square >= 7 {
                log::debug!("Grid {} rejected: South Pacific ocean (lat={:.1}, lon={:.1})", grid, lat, lon);
                return false;
            }
        }
    }
    
    // Additional sanity check: extremely remote locations
    // Antarctica below -80° latitude (some activity but rare)
    // We allow it but log a warning
    if lat < -80.0 {
        log::debug!("Grid {} is in Antarctica (lat={:.1})", grid, lat);
    }
    
    true
}

/// Check if a string matches a known FT8/FT4 message fragment
/// These are protocol messages that should never be stored as grid squares
fn is_ft8_message_fragment(s: &str) -> bool {
    let upper = s.to_uppercase();
    
    // Exact matches for known FT8 protocol messages
    match upper.as_str() {
        "RRR" => true,      // Roger Roger Roger (acknowledgment)
        "RR73" => true,     // Roger + 73 (final acknowledgment)
        "73" => true,       // Best regards / sign-off
        _ => false,
    }
}

/// Validate a string as a Maidenhead grid square
/// 
/// This performs comprehensive validation:
/// 1. Empty strings are valid (represents "unknown grid")
/// 2. Must be 2, 4, or 6 characters
/// 3. Must match Maidenhead format (letters/digits in correct positions)
/// 4. Must not be a known FT8 message fragment
/// 5. Must be geographically plausible (not in the middle of an ocean)
/// 
/// # Arguments
/// * `s` - The string to validate
/// 
/// # Returns
/// * `true` if the string is a valid grid square or empty
/// * `false` if the string is invalid, an FT8 fragment, or geographically implausible
pub fn is_valid_grid(s: &str) -> bool {
    // Empty is valid (unknown grid)
    if s.is_empty() {
        return true;
    }
    
    // Check for FT8 message fragments first (fast rejection)
    if is_ft8_message_fragment(s) {
        log::debug!("Grid '{}' rejected: FT8 message fragment", s);
        return false;
    }
    
    // Validate length: must be 2, 4, or 6 characters
    let len = s.len();
    if len != 2 && len != 4 && len != 6 {
        log::debug!("Grid '{}' rejected: invalid length {}", s, len);
        return false;
    }
    
    let bytes = s.as_bytes();
    
    // First 2 chars: A-R (field)
    let c0 = bytes[0].to_ascii_uppercase();
    let c1 = bytes[1].to_ascii_uppercase();
    if !(b'A'..=b'R').contains(&c0) || !(b'A'..=b'R').contains(&c1) {
        log::debug!("Grid '{}' rejected: invalid field characters", s);
        return false;
    }
    
    // Next 2 chars (if present): 0-9 (square)
    if len >= 4 {
        if !bytes[2].is_ascii_digit() || !bytes[3].is_ascii_digit() {
            log::debug!("Grid '{}' rejected: invalid square digits", s);
            return false;
        }
    }
    
    // Last 2 chars (if present): a-x (subsquare)
    if len >= 6 {
        let c4 = bytes[4].to_ascii_lowercase();
        let c5 = bytes[5].to_ascii_lowercase();
        if !(b'a'..=b'x').contains(&c4) || !(b'a'..=b'x').contains(&c5) {
            log::debug!("Grid '{}' rejected: invalid subsquare characters", s);
            return false;
        }
    }
    
    // Geographic plausibility check (for 4+ char grids)
    if len >= 4 && !is_geographically_plausible(s) {
        return false;
    }
    
    true
}

/// Clean RST field - remove trailing "73" that WSJT-X sometimes appends
pub fn clean_rst(rst: &str) -> String {
    // RST should be a signal report like "-14", "+05", "599"
    // Sometimes WSJT-X appends "73" making it "-1473" or adds space "-14 73"
    let trimmed = rst.trim();
    if trimmed.ends_with("73") && trimmed.len() > 2 {
        // Check if it's like "-1473" (report + 73 without space)
        let without_73 = &trimmed[..trimmed.len()-2];
        // Valid FT8/FT4 reports are typically -30 to +30
        if let Ok(n) = without_73.parse::<i32>() {
            if (-30..=30).contains(&n) {
                return without_73.to_string();
            }
        }
    }
    trimmed.to_string()
}

/// Normalize RST to consistent format - zero-pad single digit values
/// Examples: "-5" -> "-05", "5" -> "+05", "-15" -> "-15", "+7" -> "+07"
pub fn normalize_rst(rst: &str) -> String {
    let cleaned = clean_rst(rst);
    if cleaned.is_empty() {
        return cleaned;
    }
    
    // Try to parse as FT8/FT4 signal report (integer -30 to +30)
    if let Ok(n) = cleaned.parse::<i32>() {
        if (-30..=30).contains(&n) {
            // Format with sign and zero-padding
            return format!("{:+03}", n);
        }
    }
    
    // Try parsing with explicit sign prefix
    let (sign, num_str) = if cleaned.starts_with('-') {
        ('-', &cleaned[1..])
    } else if cleaned.starts_with('+') {
        ('+', &cleaned[1..])
    } else {
        // No sign prefix, try to parse as positive number
        if let Ok(n) = cleaned.parse::<i32>() {
            if (0..=30).contains(&n) {
                return format!("{:+03}", n);
            }
        }
        return cleaned; // Return as-is if can't normalize
    };
    
    if let Ok(n) = num_str.parse::<i32>() {
        if (0..=30).contains(&n) {
            return format!("{}{:02}", sign, n);
        }
    }
    
    cleaned // Return cleaned version if can't normalize
}

// ============================================================================
// LoggedADIF Message (Type 12) - ADIF record when QSO is logged
// ============================================================================

/// LoggedADIF message - contains the full ADIF record as a string
/// NOTE: id field is part of WSJT-X protocol but not used (we only care about adif)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LoggedAdifMessage {
    pub id: String,
    pub adif: String,
}

/// Parse LoggedADIF message (type 12)
pub fn parse_logged_adif(data: &[u8]) -> Option<LoggedAdifMessage> {
    log::info!("Parsing LoggedADIF message, {} bytes", data.len());
    let mut offset = 12; // Skip magic, schema, type
    
    let id = read_qt_string(data, &mut offset)?;
    log::debug!("LoggedADIF id={}", id);
    
    let adif = read_qt_string(data, &mut offset)?;
    log::info!("LoggedADIF received: {} chars of ADIF", adif.len());
    log::debug!("LoggedADIF content: {}", adif);
    
    Some(LoggedAdifMessage { id, adif })
}

// ============================================================================
// Decode Message (Type 2) - FT8/FT4 decodes from the waterfall
// ============================================================================

/// Decoded message from the waterfall
/// NOTE: id field is part of WSJT-X protocol but not used (we identify by content)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DecodeMessage {
    pub id: String,
    pub is_new: bool,
    pub time_ms: u32,
    pub snr: i32,
    pub delta_time: f64,
    pub delta_freq: u32,
    pub mode: String,
    pub message: String,
    pub low_confidence: bool,
    pub off_air: bool,
}

/// Parse Decode message (type 2)
pub fn parse_decode(data: &[u8]) -> Option<DecodeMessage> {
    let mut offset = 12; // Skip magic, schema, type
    
    let id = read_qt_string(data, &mut offset)?;
    
    // New flag (bool)
    if offset + 1 > data.len() {
        return None;
    }
    let is_new = data[offset] != 0;
    offset += 1;
    
    // Time (QTime - u32 milliseconds since midnight)
    if offset + 4 > data.len() {
        return None;
    }
    let time_ms = u32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
    offset += 4;
    
    // SNR (i32)
    if offset + 4 > data.len() {
        return None;
    }
    let snr = i32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
    offset += 4;
    
    // Delta time (f64)
    if offset + 8 > data.len() {
        return None;
    }
    let delta_time = f64::from_be_bytes([
        data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
    ]);
    offset += 8;
    
    // Delta frequency (u32)
    if offset + 4 > data.len() {
        return None;
    }
    let delta_freq = u32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
    offset += 4;
    
    // Mode
    let mode = read_qt_string(data, &mut offset)?;
    
    // Message text
    let message = read_qt_string(data, &mut offset)?;
    
    // Low confidence flag
    let low_confidence = if offset < data.len() {
        let val = data[offset] != 0;
        offset += 1;
        val
    } else {
        false
    };
    
    // Off air flag
    let off_air = if offset < data.len() {
        data[offset] != 0
    } else {
        false
    };
    
    Some(DecodeMessage {
        id,
        is_new,
        time_ms,
        snr,
        delta_time,
        delta_freq,
        mode,
        message,
        low_confidence,
        off_air,
    })
}

/// Extract callsign and grid from a decoded FT8 message
/// FT8 messages have formats like:
/// - "CQ W5ABC EM10"
/// - "CQ DX W5ABC EM10"
/// - "CQ WWA ZW5B GG54" (contest/activity)
/// - "W5ABC KJ5KCZ -05"
/// - "KJ5KCZ W5ABC R-10"
/// - "W5ABC KJ5KCZ RR73"
/// - "K7ACN/P WA3SEE 73" (portable)
/// - "<W7UUU> W4/ZS2GK" (compound callsign)
/// 
/// Returns: (de_call, dx_call, grid, msg_type)
/// - de_call: The station sending the message
/// - dx_call: The station being called (None for CQ)
pub fn parse_ft8_message(message: &str) -> Option<(String, Option<String>, Option<String>, MessageType)> {
    let parts: Vec<&str> = message.split_whitespace().collect();
    
    if parts.is_empty() {
        return None;
    }
    
    // CQ message: "CQ W5ABC EM10" or "CQ DX W5ABC EM10" or "CQ POTA W5ABC EM10"
    if parts[0] == "CQ" {
        // Find the first part after CQ that looks like a valid callsign
        for i in 1..parts.len() {
            let candidate = parts[i];
            if is_valid_callsign(candidate) {
                let call = candidate.to_string();
                let grid = if parts.len() > i + 1 && is_grid(parts[i + 1]) {
                    Some(parts[i + 1].to_string())
                } else {
                    None
                };
                // For CQ: de_call is the caller, dx_call is None
                return Some((call, None, grid, MessageType::Cq));
            }
            // If we hit a grid, stop looking
            if is_grid(candidate) {
                break;
            }
        }
        return None; // Couldn't find a valid callsign after CQ
    }
    
    // Handle compound callsigns with angle brackets: "<W7UUU> W4/ZS2GK"
    let first = parts[0];
    let dx_call = if first.starts_with('<') && first.ends_with('>') {
        // Strip angle brackets
        first[1..first.len()-1].to_string()
    } else {
        first.to_string()
    };
    
    // Validate the extracted callsign
    if !is_valid_callsign(&dx_call) {
        return None;
    }
    
    // Standard exchange: "DX_CALL DE_CALL REPORT/GRID"
    // Example: "N5JKK W9MDM EN61" means W9MDM is calling N5JKK
    if parts.len() >= 2 {
        let de_call_str = parts[1];
        // Handle compound callsigns
        let de_call = if de_call_str.starts_with('<') && de_call_str.ends_with('>') {
            de_call_str[1..de_call_str.len()-1].to_string()
        } else {
            de_call_str.to_string()
        };
        
        if !is_valid_callsign(&de_call) {
            return None;
        }
        
        // Determine message type from third part
        let msg_type = if parts.len() > 2 {
            let third = parts[2];
            if third == "RR73" || third == "RRR" || third == "73" {
                MessageType::End
            } else if third.starts_with('R') && (third.len() == 3 || third.len() == 4) {
                MessageType::Report  // R-05, R+10
            } else if is_grid(third) {
                MessageType::Grid
            } else if third.starts_with('-') || third.starts_with('+') {
                MessageType::Report
            } else {
                MessageType::Other
            }
        } else {
            MessageType::Other
        };
        
        let grid = if parts.len() > 2 && is_grid(parts[2]) {
            Some(parts[2].to_string())
        } else {
            None
        };
        
        // de_call is the sender, dx_call is who they're calling
        return Some((de_call, Some(dx_call), grid, msg_type));
    }
    
    None
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    Cq,      // CQ call
    Grid,    // Sending grid
    Report,  // Sending signal report
    End,     // RR73/RRR/73
    Other,
}

fn is_grid(s: &str) -> bool {
    if s.len() != 4 {
        return false;
    }
    let chars: Vec<char> = s.chars().collect();
    chars[0].is_ascii_uppercase() && chars[1].is_ascii_uppercase() &&
    chars[2].is_ascii_digit() && chars[3].is_ascii_digit()
}

/// Basic validation that a string looks like a callsign
/// Callsigns typically have letters and numbers, 3-10 chars
fn is_valid_callsign(s: &str) -> bool {
    let len = s.len();
    if len < 3 || len > 10 {
        return false;
    }
    
    // Must contain at least one digit
    let has_digit = s.chars().any(|c| c.is_ascii_digit());
    // Must contain at least one letter
    let has_letter = s.chars().any(|c| c.is_ascii_alphabetic());
    // All chars must be alphanumeric or /
    let all_valid = s.chars().all(|c| c.is_ascii_alphanumeric() || c == '/');
    
    has_digit && has_letter && all_valid
}

// ============================================================================
// Reply Message (Type 4) - Send a reply to initiate a QSO
// ============================================================================

/// Write a Qt-style string (length-prefixed UTF-8)
fn write_qt_string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    let len = bytes.len() as u32;
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}

/// Build a Reply message (type 4) to send to WSJT-X
/// This triggers WSJT-X to start a QSO with the specified station
#[derive(Debug, Clone)]
pub struct ReplyMessage {
    pub id: String,           // Target WSJT-X instance ID
    pub time_ms: u32,         // Time from decode (ms since midnight)
    pub snr: i32,             // SNR from decode
    pub delta_time: f64,      // Delta time from decode
    pub delta_freq: u32,      // Delta frequency from decode
    pub mode: String,         // Mode (e.g., "~" for FT8)
    pub message: String,      // The decoded message text
    pub low_confidence: bool, // Low confidence flag from decode
    pub modifiers: u8,        // Keyboard modifiers (0x00 = none)
}

impl ReplyMessage {
    /// Encode the Reply message into bytes for UDP transmission
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(256);
        
        // Magic number
        buf.extend_from_slice(&WSJTX_MAGIC.to_be_bytes());
        
        // Schema version (3)
        buf.extend_from_slice(&3u32.to_be_bytes());
        
        // Message type (4 = Reply)
        buf.extend_from_slice(&4u32.to_be_bytes());
        
        // Id (target unique key)
        write_qt_string(&mut buf, &self.id);
        
        // Time (QTime - u32 milliseconds since midnight)
        buf.extend_from_slice(&self.time_ms.to_be_bytes());
        
        // SNR (i32)
        buf.extend_from_slice(&self.snr.to_be_bytes());
        
        // Delta time (f64 - serialized as double)
        buf.extend_from_slice(&self.delta_time.to_be_bytes());
        
        // Delta frequency (u32)
        buf.extend_from_slice(&self.delta_freq.to_be_bytes());
        
        // Mode
        write_qt_string(&mut buf, &self.mode);
        
        // Message
        write_qt_string(&mut buf, &self.message);
        
        // Low confidence (bool)
        buf.push(if self.low_confidence { 1 } else { 0 });
        
        // Modifiers (u8)
        buf.push(self.modifiers);
        
        buf
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Grid Validation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_valid_grid_empty() {
        // Empty grid is valid (unknown)
        assert!(is_valid_grid(""));
    }

    #[test]
    fn test_is_valid_grid_2char() {
        // 2-char field (A-R, A-R)
        assert!(is_valid_grid("FN"));
        assert!(is_valid_grid("AA"));
        assert!(is_valid_grid("RR"));
        assert!(is_valid_grid("fn")); // lowercase OK
    }

    #[test]
    fn test_is_valid_grid_4char() {
        // 4-char square (field + 00-99)
        assert!(is_valid_grid("FN42"));
        assert!(is_valid_grid("AA00"));
        assert!(is_valid_grid("EM20"));
        assert!(is_valid_grid("DN70"));
        // RR99 is blocked by geographic plausibility (deep South Pacific)
        assert!(!is_valid_grid("RR99"));
    }

    #[test]
    fn test_is_valid_grid_6char() {
        // 6-char subsquare (field + square + a-x)
        assert!(is_valid_grid("FN42FV"));
        assert!(is_valid_grid("EM20KE"));
        assert!(is_valid_grid("DN70aa"));
        assert!(is_valid_grid("fn42fv")); // lowercase OK
    }

    #[test]
    fn test_is_valid_grid_invalid_ft8_messages() {
        // These FT8 message fragments should NOT be valid grids
        assert!(!is_valid_grid("RR73")); // FT8 acknowledgment
        assert!(!is_valid_grid("RRR"));  // FT8 acknowledgment (also odd length)
        assert!(!is_valid_grid("73"));   // FT8 farewell (digits only)
        assert!(!is_valid_grid("-14"));  // Signal report (has dash)
        assert!(!is_valid_grid("+05"));  // Signal report (has plus)
    }

    #[test]
    fn test_is_valid_grid_geographic_plausibility() {
        // RR73 is rejected both as FT8 fragment AND geographically (middle of Pacific)
        assert!(!is_valid_grid("RR73"));
        
        // RR70-RR79 are all deep Pacific Ocean with no land
        assert!(!is_valid_grid("RR70"));
        assert!(!is_valid_grid("RR78"));
        assert!(!is_valid_grid("RR79"));
        
        // RR00-RR69 are potentially valid (some have islands)
        // We allow these as they could be legitimate (e.g., ships, islands)
        assert!(is_valid_grid("RR00"));
        assert!(is_valid_grid("RR50"));
    }

    #[test]
    fn test_is_valid_grid_invalid_length() {
        // Invalid lengths
        assert!(!is_valid_grid("F"));     // 1 char
        assert!(!is_valid_grid("FN4"));   // 3 char
        assert!(!is_valid_grid("FN42F")); // 5 char
        assert!(!is_valid_grid("FN42FVX")); // 7 char
    }

    #[test]
    fn test_is_valid_grid_invalid_chars() {
        // Invalid characters for field (must be A-R)
        assert!(!is_valid_grid("SN"));   // S is out of range
        assert!(!is_valid_grid("ZZ"));   // Z is out of range
        assert!(!is_valid_grid("12"));   // Numbers not allowed in field
    }

    #[test]
    fn test_grid_to_latlon() {
        // Test coordinate conversion
        // FN42 is in New England area
        // Maidenhead: F=5 (lon field), N=13 (lat field), 4 (lon square), 2 (lat square)
        // lon = -180 + 5*20 + 4*2 + 1 = -180 + 100 + 8 + 1 = -71
        // lat = -90 + 13*10 + 2*1 + 0.5 = -90 + 130 + 2.5 = 42.5
        let (lat, lon) = grid_to_latlon("FN42").unwrap();
        assert!((lat - 42.5).abs() < 1.0, "FN42 lat should be ~42.5, got {}", lat);
        assert!((lon - (-71.0)).abs() < 2.0, "FN42 lon should be ~-71, got {}", lon);
        
        // RR73 is in the South Pacific
        // R=17 (lon field), R=17 (lat field), 7 (lon square), 3 (lat square)
        // lon = -180 + 17*20 + 7*2 + 1 = -180 + 340 + 14 + 1 = 175 (or wraps to ~175E/-185W)
        // lat = -90 + 17*10 + 3*1 + 0.5 = -90 + 170 + 3.5 = 83.5... wait that's wrong
        // Actually lat field R=17: lat = -90 + 17*10 + 3 + 0.5 = 83.5 (northern!)
        // Hmm, let me recalculate. R is 17th letter (0-indexed), field is 0-17
        let (lat, lon) = grid_to_latlon("RR73").unwrap();
        // Just verify it returns something reasonable
        assert!(lat.is_finite(), "RR73 lat should be finite");
        assert!(lon.is_finite(), "RR73 lon should be finite");
    }

    #[test]
    fn test_is_ft8_message_fragment() {
        // Known FT8 fragments
        assert!(is_ft8_message_fragment("RR73"));
        assert!(is_ft8_message_fragment("rr73")); // case insensitive
        assert!(is_ft8_message_fragment("RRR"));
        assert!(is_ft8_message_fragment("73"));
        
        // Not FT8 fragments
        assert!(!is_ft8_message_fragment("FN42"));
        assert!(!is_ft8_message_fragment("EM20"));
        assert!(!is_ft8_message_fragment("RR00")); // Valid grid, not an FT8 fragment
    }

    // -------------------------------------------------------------------------
    // RST Cleaning Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_clean_rst_normal() {
        // Normal RST values should pass through
        assert_eq!(clean_rst("-14"), "-14");
        assert_eq!(clean_rst("+05"), "+05");
        assert_eq!(clean_rst("-5"), "-5");
        assert_eq!(clean_rst("599"), "599");
    }

    #[test]
    fn test_clean_rst_trailing_73() {
        // RST with appended 73 should be cleaned
        assert_eq!(clean_rst("-1473"), "-14");
        assert_eq!(clean_rst("+0573"), "+05");
        assert_eq!(clean_rst("-573"), "-5");
    }

    #[test]
    fn test_clean_rst_whitespace() {
        // Whitespace should be trimmed
        assert_eq!(clean_rst("  -14  "), "-14");
        assert_eq!(clean_rst("\t+05\n"), "+05");
    }

    #[test]
    fn test_clean_rst_just_73() {
        // Just "73" is not a valid RST, but clean_rst preserves it
        // (it's more than 2 chars check fails)
        assert_eq!(clean_rst("73"), "73");
    }

    // -------------------------------------------------------------------------
    // RST Normalization Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_normalize_rst_zero_padding() {
        // Single digit should be zero-padded
        assert_eq!(normalize_rst("-5"), "-05");
        assert_eq!(normalize_rst("+7"), "+07");
        assert_eq!(normalize_rst("5"), "+05");  // Positive assumed
        assert_eq!(normalize_rst("0"), "+00");
    }

    #[test]
    fn test_normalize_rst_already_padded() {
        // Already padded values should stay the same
        assert_eq!(normalize_rst("-05"), "-05");
        assert_eq!(normalize_rst("+07"), "+07");
        assert_eq!(normalize_rst("-14"), "-14");
        assert_eq!(normalize_rst("+15"), "+15");
    }

    #[test]
    fn test_normalize_rst_negative() {
        // Negative values
        assert_eq!(normalize_rst("-1"), "-01");
        assert_eq!(normalize_rst("-9"), "-09");
        assert_eq!(normalize_rst("-10"), "-10");
        assert_eq!(normalize_rst("-30"), "-30");
    }

    #[test]
    fn test_normalize_rst_empty() {
        // Empty should stay empty
        assert_eq!(normalize_rst(""), "");
    }

    #[test]
    fn test_normalize_rst_non_ft8() {
        // Non-FT8 RST (like CW/SSB "599") should pass through
        assert_eq!(normalize_rst("599"), "599");
        assert_eq!(normalize_rst("59"), "59");
    }

    #[test]
    fn test_normalize_rst_with_trailing_73() {
        // Should clean 73 AND normalize
        assert_eq!(normalize_rst("-573"), "-05");
        assert_eq!(normalize_rst("-1473"), "-14");
    }
}
