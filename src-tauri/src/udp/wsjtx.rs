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
    
    // Validate RST - should be signal report only (e.g., "-14", "+05"), not "73" suffix
    let validated_report_sent = clean_rst(&report_sent);
    let validated_report_rcvd = clean_rst(&report_rcvd);
    
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

/// Check if a string is a valid Maidenhead grid square (2, 4, or 6 chars)
fn is_valid_grid(s: &str) -> bool {
    if s.is_empty() {
        return true; // Empty is OK (unknown)
    }
    let len = s.len();
    if len != 2 && len != 4 && len != 6 {
        return false;
    }
    let bytes = s.as_bytes();
    // First 2 chars: A-R (field)
    if len >= 2 {
        let c0 = bytes[0].to_ascii_uppercase();
        let c1 = bytes[1].to_ascii_uppercase();
        if !(b'A'..=b'R').contains(&c0) || !(b'A'..=b'R').contains(&c1) {
            return false;
        }
    }
    // Next 2 chars: 0-9 (square)
    if len >= 4 {
        if !bytes[2].is_ascii_digit() || !bytes[3].is_ascii_digit() {
            return false;
        }
    }
    // Last 2 chars: a-x (subsquare)
    if len >= 6 {
        let c4 = bytes[4].to_ascii_lowercase();
        let c5 = bytes[5].to_ascii_lowercase();
        if !(b'a'..=b'x').contains(&c4) || !(b'a'..=b'x').contains(&c5) {
            return false;
        }
    }
    true
}

/// Clean RST field - remove trailing "73" that WSJT-X sometimes appends
fn clean_rst(rst: &str) -> String {
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

// ============================================================================
// LoggedADIF Message (Type 12) - ADIF record when QSO is logged
// ============================================================================

/// LoggedADIF message - contains the full ADIF record as a string
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
