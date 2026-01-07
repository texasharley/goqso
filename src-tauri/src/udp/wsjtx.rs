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

/// Parse QSO Logged message (type 5)
pub fn parse_qso_logged(data: &[u8]) -> Option<QsoLoggedMessage> {
    let mut offset = 12; // Skip magic, schema, type
    
    let id = read_qt_string(data, &mut offset)?;
    
    // DateTime off (QDateTime - 8 bytes: Julian day + milliseconds)
    if offset + 8 > data.len() {
        return None;
    }
    let _julian_day = u64::from_be_bytes([
        data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
    ]);
    offset += 8;
    // Skip time spec byte
    if offset + 1 > data.len() {
        return None;
    }
    offset += 1;
    
    let call = read_qt_string(data, &mut offset)?;
    let grid = read_qt_string(data, &mut offset)?;
    
    // Frequency (u64)
    if offset + 8 > data.len() {
        return None;
    }
    let freq_hz = u64::from_be_bytes([
        data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
    ]);
    offset += 8;
    
    let mode = read_qt_string(data, &mut offset)?;
    let report_sent = read_qt_string(data, &mut offset)?;
    let report_rcvd = read_qt_string(data, &mut offset)?;
    let tx_power = read_qt_string(data, &mut offset)?;
    let comments = read_qt_string(data, &mut offset)?;
    let name = read_qt_string(data, &mut offset)?;
    
    // DateTime on
    if offset + 9 > data.len() {
        return None;
    }
    offset += 9;
    
    let operator_call = read_qt_string(data, &mut offset)?;
    let my_call = read_qt_string(data, &mut offset)?;
    let my_grid = read_qt_string(data, &mut offset)?;
    let exchange_sent = read_qt_string(data, &mut offset).unwrap_or_default();
    let exchange_rcvd = read_qt_string(data, &mut offset).unwrap_or_default();
    let adif_propagation_mode = read_qt_string(data, &mut offset).unwrap_or_default();
    
    Some(QsoLoggedMessage {
        id,
        datetime_off: String::new(), // TODO: Parse properly
        call,
        grid,
        freq_hz,
        mode,
        report_sent,
        report_rcvd,
        tx_power,
        comments,
        name,
        datetime_on: String::new(),
        operator_call,
        my_call,
        my_grid,
        exchange_sent,
        exchange_rcvd,
        adif_propagation_mode,
    })
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
