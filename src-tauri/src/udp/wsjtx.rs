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
