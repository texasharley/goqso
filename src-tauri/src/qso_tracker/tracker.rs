// =============================================================================
// QSO Tracker - Tracks Multiple QSOs in Progress
// =============================================================================

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::state::{QsoState, QsoPhase, QsoRole};
use super::{QsoInProgress, QsoEvent, ObservedMessage};

/// Timeout for QSOs - if no activity for this long, abandon the QSO
const QSO_TIMEOUT: Duration = Duration::from_secs(120); // 2 minutes

/// Tracker that manages all QSOs in progress
#[derive(Debug)]
pub struct QsoTracker {
    /// QSOs in progress, keyed by their callsign (uppercased)
    qsos: HashMap<String, QsoInProgress>,
    /// My callsign (from WSJT-X status)
    my_call: String,
    /// My grid (from WSJT-X status)
    my_grid: String,
    /// Current frequency (from WSJT-X status)
    current_freq: u64,
    /// Current mode (from WSJT-X status)
    current_mode: String,
    /// Currently selected DX call in WSJT-X
    current_dx_call: String,
}

impl Default for QsoTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl QsoTracker {
    pub fn new() -> Self {
        Self {
            qsos: HashMap::new(),
            my_call: String::new(),
            my_grid: String::new(),
            current_freq: 0,
            current_mode: String::new(),
            current_dx_call: String::new(),
        }
    }
    
    /// Update status from WSJT-X Status message
    pub fn update_status(
        &mut self,
        my_call: &str,
        my_grid: &str,
        freq: u64,
        mode: &str,
        dx_call: &str,
    ) {
        self.my_call = my_call.to_uppercase();
        self.my_grid = my_grid.to_uppercase();
        self.current_freq = freq;
        self.current_mode = mode.to_string();
        self.current_dx_call = dx_call.to_uppercase();
    }
    
    /// Process a transmitted message (we sent this)
    pub fn process_tx(&mut self, message: &str, snr: Option<i32>) -> QsoEvent {
        let message = message.trim().to_uppercase();
        log::debug!("QsoTracker TX: {}", message);
        
        // Parse the message to extract callsigns
        let parts: Vec<&str> = message.split_whitespace().collect();
        if parts.len() < 2 {
            return QsoEvent::None;
        }
        
        // Handle CQ - we're calling CQ
        if parts[0] == "CQ" {
            // Find our callsign in the message
            // CQ K1ABC FN42 or CQ DX K1ABC FN42
            for i in 1..parts.len() {
                if is_valid_callsign(parts[i]) && parts[i] == self.my_call {
                    // We're calling CQ - no specific QSO yet
                    log::debug!("We're calling CQ");
                    return QsoEvent::None;
                }
            }
            return QsoEvent::None;
        }
        
        // Standard message format: THEIR_CALL MY_CALL INFO
        if parts.len() >= 2 {
            let their_call = clean_callsign(parts[0]);
            let sender = clean_callsign(parts[1]);
            
            // Verify this is from us
            if sender != self.my_call {
                return QsoEvent::None;
            }
            
            // Get or create QSO entry
            let qso = self.qsos.entry(their_call.clone()).or_insert_with(|| {
                log::info!("New QSO started with {} (we transmitted first)", their_call);
                QsoInProgress {
                    their_call: their_call.clone(),
                    my_call: self.my_call.clone(),
                    phase: QsoPhase::Started,
                    role: QsoRole::Unknown,
                    their_grid: None,
                    report_sent: None,
                    report_rcvd: None,
                    started_at: Instant::now(),
                    last_activity: Instant::now(),
                    messages: Vec::new(),
                    completed: false,
                    freq_hz: self.current_freq,
                    mode: self.current_mode.clone(),
                }
            });
            
            // Record this message
            qso.messages.push(ObservedMessage {
                timestamp: Instant::now(),
                message: message.clone(),
                is_tx: true,
                snr,
                freq_offset: None,
            });
            qso.last_activity = Instant::now();
            
            // Create state by replaying all previous messages (not including this one)
            let mut state = build_state_from_messages(&qso.messages[..qso.messages.len()-1], qso.role);
            let old_phase = state.phase;
            
            // Process this new message
            state.process_tx(&message);
            
            // Update QSO from state
            qso.phase = state.phase;
            if state.report_sent.is_some() {
                qso.report_sent = state.report_sent.clone();
            }
            
            // Check if newly completed
            if state.is_loggable() && !qso.completed {
                qso.completed = true;
                log::info!("QSO with {} is complete and ready to log!", their_call);
                return QsoEvent::Complete(qso.clone());
            }
            
            // Check for phase change
            if state.phase != old_phase {
                return QsoEvent::Progressed {
                    their_call,
                    phase: state.phase,
                };
            }
        }
        
        QsoEvent::None
    }
    
    /// Process a received (decoded) message from the waterfall
    pub fn process_rx(&mut self, message: &str, snr: i32, freq_offset: u32) -> QsoEvent {
        let message = message.trim().to_uppercase();
        log::trace!("QsoTracker RX: {} (SNR: {})", message, snr);
        
        let parts: Vec<&str> = message.split_whitespace().collect();
        if parts.len() < 2 {
            return QsoEvent::None;
        }
        
        // Handle CQ from someone else
        if parts[0] == "CQ" {
            // Someone else is calling CQ - we might answer
            return QsoEvent::None;
        }
        
        // Standard message format: THEIR_CALL SENDER_CALL INFO
        // For us to care, either:
        // - THEIR_CALL is us (they're calling us)
        // - SENDER_CALL is someone we're in QSO with (we're watching their response)
        
        let their_call = clean_callsign(parts[0]); // Who is being called
        let sender_call = clean_callsign(parts[1]); // Who is sending
        
        // Case 1: They're calling us
        if their_call == self.my_call {
            // Get or create QSO entry
            let is_new = !self.qsos.contains_key(&sender_call);
            let qso = self.qsos.entry(sender_call.clone()).or_insert_with(|| {
                log::info!("New QSO started with {} (they called us)", sender_call);
                QsoInProgress {
                    their_call: sender_call.clone(),
                    my_call: self.my_call.clone(),
                    phase: QsoPhase::CallReceived,
                    role: QsoRole::Initiator, // They answered our CQ
                    their_grid: None,
                    report_sent: None,
                    report_rcvd: None,
                    started_at: Instant::now(),
                    last_activity: Instant::now(),
                    messages: Vec::new(),
                    completed: false,
                    freq_hz: self.current_freq,
                    mode: self.current_mode.clone(),
                }
            });
            
            // Record this message
            qso.messages.push(ObservedMessage {
                timestamp: Instant::now(),
                message: message.clone(),
                is_tx: false,
                snr: Some(snr),
                freq_offset: Some(freq_offset),
            });
            qso.last_activity = Instant::now();
            
            // Create state by replaying all previous messages (not including this one)
            let mut state = build_state_from_messages(&qso.messages[..qso.messages.len()-1], qso.role);
            let old_phase = state.phase;
            
            // Process this new message
            state.process_rx(&message);
            
            // Update QSO from state
            qso.phase = state.phase;
            if state.their_grid.is_some() {
                qso.their_grid = state.their_grid.clone();
            }
            if state.report_rcvd.is_some() {
                qso.report_rcvd = state.report_rcvd.clone();
            }
            
            // Check if newly completed
            if state.is_loggable() && !qso.completed {
                qso.completed = true;
                log::info!("QSO with {} is complete and ready to log!", sender_call);
                return QsoEvent::Complete(qso.clone());
            }
            
            // Check for new QSO or phase change
            if is_new {
                return QsoEvent::Started { their_call: sender_call };
            }
            if state.phase != old_phase {
                return QsoEvent::Progressed {
                    their_call: sender_call,
                    phase: state.phase,
                };
            }
        }
        
        QsoEvent::None
    }
    
    /// Clean up timed-out QSOs and return abandoned ones
    pub fn cleanup_stale(&mut self) -> Vec<String> {
        let mut abandoned = Vec::new();
        
        self.qsos.retain(|call, qso| {
            if qso.is_timed_out(QSO_TIMEOUT) {
                if !qso.completed {
                    log::info!("QSO with {} timed out (incomplete)", call);
                    abandoned.push(call.clone());
                }
                false // Remove from map
            } else {
                true // Keep in map
            }
        });
        
        abandoned
    }
    
    /// Get a QSO in progress by callsign
    pub fn get_qso(&self, call: &str) -> Option<&QsoInProgress> {
        self.qsos.get(&call.to_uppercase())
    }
    
    /// Get all QSOs currently in progress
    pub fn active_qsos(&self) -> Vec<&QsoInProgress> {
        self.qsos.values().filter(|q| !q.completed).collect()
    }
    
    /// Get count of active QSOs
    pub fn active_count(&self) -> usize {
        self.qsos.values().filter(|q| !q.completed).count()
    }
}

/// Build a QsoState by replaying a slice of messages
fn build_state_from_messages(messages: &[ObservedMessage], role: QsoRole) -> QsoState {
    let mut state = QsoState::new();
    state.role = role;
    
    // Replay messages to rebuild state
    for msg in messages {
        if msg.is_tx {
            state.process_tx(&msg.message);
        } else {
            state.process_rx(&msg.message);
        }
    }
    
    state
}

/// Clean angle brackets and other decorations from callsigns
fn clean_callsign(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('<') && s.ends_with('>') {
        s[1..s.len()-1].to_uppercase()
    } else {
        s.to_uppercase()
    }
}

/// Basic validation that a string looks like a callsign
fn is_valid_callsign(s: &str) -> bool {
    let s = clean_callsign(s);
    let len = s.len();
    if len < 3 || len > 10 {
        return false;
    }
    
    let has_digit = s.chars().any(|c| c.is_ascii_digit());
    let has_letter = s.chars().any(|c| c.is_ascii_alphabetic());
    let all_valid = s.chars().all(|c| c.is_ascii_alphanumeric() || c == '/');
    
    has_digit && has_letter && all_valid
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn setup_tracker() -> QsoTracker {
        let mut tracker = QsoTracker::new();
        tracker.update_status("K1ABC", "FN42", 14074000, "FT8", "");
        tracker
    }
    
    #[test]
    fn test_complete_qso_as_cq_caller() {
        let mut tracker = setup_tracker();
        
        // We called CQ, someone answered
        let event = tracker.process_rx("K1ABC G0XYZ IO91", -15, 1500);
        assert!(matches!(event, QsoEvent::Started { .. }));
        
        // We send our report
        let event = tracker.process_tx("G0XYZ K1ABC -19", None);
        assert!(matches!(event, QsoEvent::Progressed { phase: QsoPhase::ReportSent, .. }));
        
        // They send R+report
        let event = tracker.process_rx("K1ABC G0XYZ R-22", -14, 1500);
        assert!(matches!(event, QsoEvent::Progressed { phase: QsoPhase::ReportReceived, .. }));
        
        // We send RRR
        let event = tracker.process_tx("G0XYZ K1ABC RRR", None);
        assert!(matches!(event, QsoEvent::Complete(_)));
    }
    
    #[test]
    fn test_incomplete_qso_not_logged() {
        let mut tracker = setup_tracker();
        
        // Someone answers our CQ
        tracker.process_rx("K1ABC G0XYZ IO91", -15, 1500);
        
        // We send report
        tracker.process_tx("G0XYZ K1ABC -19", None);
        
        // They never respond... QSO should NOT be complete
        let qso = tracker.get_qso("G0XYZ").unwrap();
        assert!(!qso.completed);
        assert!(!qso.phase.is_loggable());
    }
}
