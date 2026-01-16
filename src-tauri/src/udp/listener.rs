// UDP Listener for WSJT-X
// Listens on configurable port (default 2237) and parses WSJT-X messages

use std::net::{UdpSocket, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use super::wsjtx::{parse_message, parse_qso_logged, parse_logged_adif, parse_decode, WsjtxMessageType, QsoLoggedMessage, DecodeMessage, read_qt_string, ReplyMessage, is_valid_grid, normalize_rst};

/// Parse an ADIF record string into a QsoLoggedMessage
/// ADIF format: <TAG:LENGTH>VALUE or <TAG:LENGTH:TYPE>VALUE
fn parse_adif_to_qso(adif: &str) -> Option<QsoLoggedMessage> {
    let mut fields = std::collections::HashMap::new();
    let mut pos = 0;
    let bytes = adif.as_bytes();
    
    while pos < bytes.len() {
        // Find next '<'
        while pos < bytes.len() && bytes[pos] != b'<' {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }
        pos += 1; // Skip '<'
        
        // Find field name (until ':' or '>')
        let name_start = pos;
        while pos < bytes.len() && bytes[pos] != b':' && bytes[pos] != b'>' {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }
        
        let field_name = std::str::from_utf8(&bytes[name_start..pos]).ok()?.to_uppercase();
        
        if bytes[pos] == b'>' {
            // No length specified (e.g., <EOR>)
            pos += 1;
            continue;
        }
        
        pos += 1; // Skip ':'
        
        // Parse length
        let len_start = pos;
        while pos < bytes.len() && bytes[pos].is_ascii_digit() {
            pos += 1;
        }
        let length: usize = std::str::from_utf8(&bytes[len_start..pos]).ok()?.parse().ok()?;
        
        // Skip optional type specifier and '>'
        while pos < bytes.len() && bytes[pos] != b'>' {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }
        pos += 1; // Skip '>'
        
        // Extract value
        if pos + length <= bytes.len() {
            let value = std::str::from_utf8(&bytes[pos..pos + length]).ok()?;
            fields.insert(field_name, value.to_string());
            pos += length;
        }
    }
    
    log::debug!("Parsed ADIF fields: {:?}", fields);
    
    // Extract required fields
    let call = fields.get("CALL")?.clone();
    let raw_grid = fields.get("GRIDSQUARE").cloned().unwrap_or_default();
    let mode = fields.get("MODE").cloned().unwrap_or_default();
    
    // Validate grid - WSJT-X sometimes puts "RR73", "RRR", "73" in grid field
    let grid = if is_valid_grid(&raw_grid) { 
        raw_grid 
    } else { 
        log::warn!("Invalid grid '{}' for {} in LoggedADIF, clearing it", raw_grid, call);
        String::new() 
    };
    
    // Frequency in Hz - ADIF FREQ is in MHz
    let freq_hz = fields.get("FREQ")
        .and_then(|f| f.parse::<f64>().ok())
        .map(|f| (f * 1_000_000.0) as u64)
        .unwrap_or(0);
    
    // Normalize RST values for consistent storage
    let report_sent = normalize_rst(&fields.get("RST_SENT").cloned().unwrap_or_default());
    let report_rcvd = normalize_rst(&fields.get("RST_RCVD").cloned().unwrap_or_default());
    let my_call = fields.get("STATION_CALLSIGN").or(fields.get("OPERATOR")).cloned().unwrap_or_default();
    let my_grid = fields.get("MY_GRIDSQUARE").cloned().unwrap_or_default();
    
    log::info!("Parsed ADIF QSO: call={} grid={} freq={} mode={}", call, grid, freq_hz, mode);
    
    Some(QsoLoggedMessage {
        id: String::new(),
        datetime_off: fields.get("TIME_OFF").cloned().unwrap_or_default(),
        call,
        grid,
        freq_hz,
        mode,
        report_sent,
        report_rcvd,
        tx_power: fields.get("TX_PWR").cloned().unwrap_or_default(),
        comments: fields.get("COMMENT").cloned().unwrap_or_default(),
        name: fields.get("NAME").cloned().unwrap_or_default(),
        datetime_on: fields.get("TIME_ON").cloned().unwrap_or_default(),
        operator_call: fields.get("OPERATOR").cloned().unwrap_or_default(),
        my_call,
        my_grid,
        exchange_sent: String::new(),
        exchange_rcvd: String::new(),
        adif_propagation_mode: fields.get("PROP_MODE").cloned().unwrap_or_default(),
    })
}

/// Listener state that can be shared across threads
pub struct UdpListenerState {
    running: AtomicBool,
    port: std::sync::Mutex<u16>,
    wsjtx_addr: std::sync::Mutex<Option<SocketAddr>>,
    wsjtx_id: std::sync::Mutex<Option<String>>,
}

impl UdpListenerState {
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            port: std::sync::Mutex::new(2237),
            wsjtx_addr: std::sync::Mutex::new(None),
            wsjtx_id: std::sync::Mutex::new(None),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn set_running(&self, value: bool) {
        self.running.store(value, Ordering::SeqCst);
    }

    pub fn get_port(&self) -> u16 {
        *self.port.lock().unwrap()
    }

    pub fn set_port(&self, port: u16) {
        *self.port.lock().unwrap() = port;
    }
    
    pub fn set_wsjtx_addr(&self, addr: SocketAddr, id: String) {
        *self.wsjtx_addr.lock().unwrap() = Some(addr);
        *self.wsjtx_id.lock().unwrap() = Some(id);
    }
    
    pub fn get_wsjtx_addr(&self) -> Option<SocketAddr> {
        *self.wsjtx_addr.lock().unwrap()
    }
    
    pub fn get_wsjtx_id(&self) -> Option<String> {
        self.wsjtx_id.lock().unwrap().clone()
    }
}

impl Default for UdpListenerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Message types sent from the listener to the main thread
/// NOTE: Some variant fields (max_schema, revision, tx_mode, decoding) are parsed
/// from the WSJT-X protocol for completeness but not currently used in the UI.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum UdpMessage {
    QsoLogged(QsoLoggedMessage),
    Decode(DecodeMessage),
    Clear { id: String, window: u8 },
    Heartbeat { id: String, max_schema: u32, version: String, revision: String },
    Status { id: String, dial_freq: u64, mode: String, dx_call: String, de_call: String, report: String, tx_mode: String, tx_enabled: bool, transmitting: bool, decoding: bool, tx_message: String },
    Connected,
    Disconnected,
    Error(String),
}

/// Start the UDP listener in a background thread
pub fn start_listener(
    state: Arc<UdpListenerState>,
    sender: mpsc::UnboundedSender<UdpMessage>,
) -> Result<(), String> {
    if state.is_running() {
        return Err("UDP listener already running".to_string());
    }

    let port = state.get_port();
    state.set_running(true);

    std::thread::spawn(move || {
        let addr = format!("0.0.0.0:{}", port);
        
        let socket = match UdpSocket::bind(&addr) {
            Ok(s) => {
                log::info!("UDP listener bound to {}", addr);
                let _ = sender.send(UdpMessage::Connected);
                s
            }
            Err(e) => {
                log::error!("Failed to bind UDP socket: {}", e);
                let _ = sender.send(UdpMessage::Error(format!("Failed to bind: {}", e)));
                state.set_running(false);
                return;
            }
        };

        // Set a read timeout so we can check the running flag periodically
        socket.set_read_timeout(Some(Duration::from_millis(500))).ok();

        let mut buf = [0u8; 2048];
        
        while state.is_running() {
            match socket.recv_from(&mut buf) {
                Ok((len, src)) => {
                    log::trace!("Received {} bytes from {}", len, src);
                    
                    if let Some(msg_type) = parse_message(&buf[..len]) {
                        log::debug!("UDP message type: {:?} ({} bytes)", msg_type, len);
                        match msg_type {
                            WsjtxMessageType::Decode => {
                                if let Some(decode) = parse_decode(&buf[..len]) {
                                    if decode.is_new && !decode.off_air {
                                        log::debug!("Decode: {} dB: {}", decode.snr, decode.message);
                                        let _ = sender.send(UdpMessage::Decode(decode));
                                    }
                                }
                            }
                            WsjtxMessageType::QsoLogged => {
                                log::warn!("[QSO-SOURCE] QsoLogged (type 5) received from WSJT-X ({} bytes)", len);
                                log::debug!("QsoLogged raw bytes: {:02x?}", &buf[..len.min(200)]);
                                if let Some(mut qso) = parse_qso_logged(&buf[..len]) {
                                    log::warn!("[QSO-SOURCE] Type5: call={} mode={} freq={} datetime_on={} grid={}", 
                                        qso.call, qso.mode, qso.freq_hz, qso.datetime_on, qso.grid);
                                    // Tag source for debugging
                                    qso.id = "TYPE5".to_string();
                                    let _ = sender.send(UdpMessage::QsoLogged(qso));
                                } else {
                                    log::error!("Failed to parse QsoLogged message");
                                }
                            }
                            WsjtxMessageType::Heartbeat => {
                                if let Some(hb) = parse_heartbeat(&buf[..len]) {
                                    log::debug!("Heartbeat from WSJT-X: {} at {}", hb.id, src);
                                    // Store the WSJT-X address for sending replies
                                    state.set_wsjtx_addr(src, hb.id.clone());
                                    let _ = sender.send(UdpMessage::Heartbeat {
                                        id: hb.id,
                                        max_schema: hb.max_schema,
                                        version: hb.version,
                                        revision: hb.revision,
                                    });
                                }
                            }
                            WsjtxMessageType::Status => {
                                if let Some(status) = parse_status(&buf[..len]) {
                                    log::debug!("Status: {} de_call={} mode={} freq={} tx_msg='{}'", 
                                        status.id, status.de_call, status.mode, status.dial_freq, status.tx_message);
                                    let _ = sender.send(UdpMessage::Status {
                                        id: status.id,
                                        dial_freq: status.dial_freq,
                                        mode: status.mode,
                                        dx_call: status.dx_call,
                                        de_call: status.de_call,
                                        report: status.report,
                                        tx_mode: status.tx_mode,
                                        tx_enabled: status.tx_enabled,
                                        transmitting: status.transmitting,
                                        decoding: status.decoding,
                                        tx_message: status.tx_message,
                                    });
                                }
                            }
                            WsjtxMessageType::LoggedADIF => {
                                log::warn!("[QSO-SOURCE] LoggedADIF (type 12) received from WSJT-X ({} bytes)", len);
                                if let Some(adif_msg) = parse_logged_adif(&buf[..len]) {
                                    log::warn!("[QSO-SOURCE] Type12 ADIF: {}", adif_msg.adif);
                                    // Convert ADIF string to QsoLoggedMessage
                                    if let Some(mut qso) = parse_adif_to_qso(&adif_msg.adif) {
                                        log::warn!("[QSO-SOURCE] Type12: call={} mode={} freq={} datetime_on={} grid={}", 
                                            qso.call, qso.mode, qso.freq_hz, qso.datetime_on, qso.grid);
                                        // Tag source for debugging
                                        qso.id = "TYPE12".to_string();
                                        let _ = sender.send(UdpMessage::QsoLogged(qso));
                                    } else {
                                        log::error!("Failed to parse ADIF content: {}", adif_msg.adif);
                                    }
                                } else {
                                    log::error!("Failed to parse LoggedADIF message");
                                }
                            }
                            WsjtxMessageType::Clear => {
                                // Clear message sent at start of new decode period
                                // Window: 0 = Band Activity, 1 = Rx Frequency
                                if let Some(clear) = parse_clear(&buf[..len]) {
                                    log::debug!("Clear window {} from {}", clear.window, clear.id);
                                    let _ = sender.send(UdpMessage::Clear {
                                        id: clear.id,
                                        window: clear.window,
                                    });
                                }
                            }
                            _ => {
                                log::trace!("Received {:?} message", msg_type);
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    // Normal timeout on Windows - just continue waiting
                    continue;
                }
                Err(ref e) if e.raw_os_error() == Some(10060) => {
                    // Windows WSAETIMEDOUT - normal timeout, continue waiting
                    continue;
                }
                Err(e) => {
                    log::error!("UDP receive error: {}", e);
                    let _ = sender.send(UdpMessage::Error(format!("Receive error: {}", e)));
                }
            }
        }

        log::info!("UDP listener stopped");
        let _ = sender.send(UdpMessage::Disconnected);
        state.set_running(false);
    });

    Ok(())
}

// ============================================================================
// Additional Message Parsers
// ============================================================================

#[derive(Debug)]
struct HeartbeatMessage {
    id: String,
    max_schema: u32,
    version: String,
    revision: String,
}

fn parse_heartbeat(data: &[u8]) -> Option<HeartbeatMessage> {
    let mut offset = 12; // Skip magic, schema, type
    
    let id = read_qt_string(data, &mut offset)?;
    
    if offset + 4 > data.len() {
        return None;
    }
    let max_schema = u32::from_be_bytes([
        data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
    ]);
    offset += 4;
    
    let version = read_qt_string(data, &mut offset).unwrap_or_default();
    let revision = read_qt_string(data, &mut offset).unwrap_or_default();
    
    Some(HeartbeatMessage {
        id,
        max_schema,
        version,
        revision,
    })
}

#[derive(Debug)]
struct StatusMessage {
    id: String,
    dial_freq: u64,
    mode: String,
    dx_call: String,
    de_call: String,
    report: String,
    tx_mode: String,
    tx_enabled: bool,
    transmitting: bool,
    decoding: bool,
    tx_message: String,
}

fn parse_status(data: &[u8]) -> Option<StatusMessage> {
    let mut offset = 12; // Skip magic, schema, type
    
    let id = read_qt_string(data, &mut offset)?;
    
    // Dial frequency (u64)
    if offset + 8 > data.len() {
        return None;
    }
    let dial_freq = u64::from_be_bytes([
        data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
    ]);
    offset += 8;
    
    let mode = read_qt_string(data, &mut offset)?;
    let dx_call = read_qt_string(data, &mut offset).unwrap_or_default();
    let report = read_qt_string(data, &mut offset).unwrap_or_default();
    let tx_mode = read_qt_string(data, &mut offset).unwrap_or_default();
    
    // Boolean fields
    if offset + 3 > data.len() {
        return Some(StatusMessage {
            id,
            dial_freq,
            mode,
            dx_call,
            de_call: String::new(),
            report,
            tx_mode,
            tx_enabled: false,
            transmitting: false,
            decoding: false,
            tx_message: String::new(),
        });
    }
    
    let tx_enabled = data[offset] != 0;
    let transmitting = data[offset + 1] != 0;
    let decoding = data[offset + 2] != 0;
    offset += 3;
    
    // Skip rx_df (u32), tx_df (u32)
    offset += 8;
    
    // Read de_call (our callsign!), skip de_grid and dx_grid
    let de_call = read_qt_string(data, &mut offset).unwrap_or_default();
    let _ = read_qt_string(data, &mut offset); // de_grid
    let _ = read_qt_string(data, &mut offset); // dx_grid
    
    // Skip tx_watchdog (bool), sub_mode (string), fast_mode (bool)
    offset += 1;
    let _ = read_qt_string(data, &mut offset);
    offset += 1;
    
    // Skip special_op_mode (u8), freq_tolerance (u32), tr_period (u32), config_name (string)
    offset += 1 + 4 + 4;
    let _ = read_qt_string(data, &mut offset);
    
    // Now we get tx_message!
    let tx_message = read_qt_string(data, &mut offset).unwrap_or_default();
    
    Some(StatusMessage {
        id,
        dial_freq,
        mode,
        dx_call,
        de_call,
        report,
        tx_mode,
        tx_enabled,
        transmitting,
        decoding,
        tx_message,
    })
}

#[derive(Debug)]
struct ClearMessage {
    id: String,
    window: u8,  // 0 = Band Activity, 1 = Rx Frequency
}

fn parse_clear(data: &[u8]) -> Option<ClearMessage> {
    let mut offset = 12; // Skip magic, schema, type
    
    let id = read_qt_string(data, &mut offset)?;
    
    // Window (u8): 0 = Band Activity, 1 = Rx Frequency, 2 = Both (optional)
    let window = if offset < data.len() {
        data[offset]
    } else {
        0 // Default to Band Activity
    };
    
    Some(ClearMessage { id, window })
}

/// Send a Reply message to WSJT-X to initiate a QSO
pub fn send_reply(
    state: &Arc<UdpListenerState>,
    reply: ReplyMessage,
) -> Result<(), String> {
    let addr = state.get_wsjtx_addr()
        .ok_or("WSJT-X address not known - wait for heartbeat")?;
    
    let _port = state.get_port(); // Retrieved for future use
    
    // Create a socket to send from
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("Failed to bind send socket: {}", e))?;
    
    let data = reply.encode();
    
    log::info!("Sending Reply message to {} for: {}", addr, reply.message);
    
    socket.send_to(&data, addr)
        .map_err(|e| format!("Failed to send Reply: {}", e))?;
    
    Ok(())
}