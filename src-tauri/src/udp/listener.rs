// UDP Listener for WSJT-X
// Listens on configurable port (default 2237) and parses WSJT-X messages

use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use super::wsjtx::{parse_message, parse_qso_logged, parse_decode, WsjtxMessageType, QsoLoggedMessage, DecodeMessage, read_qt_string};

/// Listener state that can be shared across threads
pub struct UdpListenerState {
    running: AtomicBool,
    port: std::sync::Mutex<u16>,
}

impl UdpListenerState {
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            port: std::sync::Mutex::new(2237),
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
}

impl Default for UdpListenerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Message types sent from the listener to the main thread
#[derive(Debug, Clone)]
pub enum UdpMessage {
    QsoLogged(QsoLoggedMessage),
    Decode(DecodeMessage),
    Heartbeat { id: String, max_schema: u32, version: String, revision: String },
    Status { id: String, dial_freq: u64, mode: String, dx_call: String, report: String, tx_mode: String, tx_enabled: bool, transmitting: bool, decoding: bool },
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
                                if let Some(qso) = parse_qso_logged(&buf[..len]) {
                                    log::info!("QSO Logged: {} on {} @ {} Hz", 
                                        qso.call, qso.mode, qso.freq_hz);
                                    let _ = sender.send(UdpMessage::QsoLogged(qso));
                                }
                            }
                            WsjtxMessageType::Heartbeat => {
                                if let Some(hb) = parse_heartbeat(&buf[..len]) {
                                    log::debug!("Heartbeat from WSJT-X: {}", hb.id);
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
                                    log::debug!("Status: {} mode={} freq={}", 
                                        status.id, status.mode, status.dial_freq);
                                    let _ = sender.send(UdpMessage::Status {
                                        id: status.id,
                                        dial_freq: status.dial_freq,
                                        mode: status.mode,
                                        dx_call: status.dx_call,
                                        report: status.report,
                                        tx_mode: status.tx_mode,
                                        tx_enabled: status.tx_enabled,
                                        transmitting: status.transmitting,
                                        decoding: status.decoding,
                                    });
                                }
                            }
                            WsjtxMessageType::LoggedADIF => {
                                log::info!("Received LoggedADIF message");
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
    report: String,
    tx_mode: String,
    tx_enabled: bool,
    transmitting: bool,
    decoding: bool,
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
            report,
            tx_mode,
            tx_enabled: false,
            transmitting: false,
            decoding: false,
        });
    }
    
    let tx_enabled = data[offset] != 0;
    let transmitting = data[offset + 1] != 0;
    let decoding = data[offset + 2] != 0;
    
    Some(StatusMessage {
        id,
        dial_freq,
        mode,
        dx_call,
        report,
        tx_mode,
        tx_enabled,
        transmitting,
        decoding,
    })
}
