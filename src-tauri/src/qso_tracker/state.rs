// =============================================================================
// QSO State Machine - Phases and Roles
// =============================================================================

/// The role we play in the QSO
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QsoRole {
    /// We called CQ and they answered
    Initiator,
    /// They called CQ and we answered
    Responder,
    /// Unknown (haven't determined yet)
    Unknown,
}

/// The current phase of the QSO
/// 
/// QSO flow from initiator (CQ caller) perspective:
/// ```text
/// Phase:      Calling -> Received -> ReportSent -> ReportReceived -> Confirmed -> Complete
/// Our TX:     CQ            -           -05              -             RRR         (73)
/// Their RX:      -      grid/call         -            R-10             -          (73)
/// ```
///
/// QSO flow from responder (answering CQ) perspective:
/// ```text
/// Phase:      Calling -> Received -> ReportSent -> ReportReceived -> Confirmed -> Complete  
/// Our TX:         -       grid          R-10            -              -          (73)
/// Their RX:    (CQ)       -05            -             RRR             -          (73)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QsoPhase {
    /// Initial state - QSO started but nothing exchanged yet
    Started = 0,
    /// We've received their callsign (either from CQ or response)
    CallReceived = 1,
    /// We've sent our grid/response
    GridSent = 2,
    /// We've received their grid
    GridReceived = 3,
    /// We've sent our signal report
    ReportSent = 4,
    /// We've received their signal report
    ReportReceived = 5,
    /// We've received acknowledgment (R+report or RRR/RR73)
    Confirmed = 6,
    /// QSO is fully complete (73 exchanged or RR73 seen)
    Complete = 7,
}

impl QsoPhase {
    /// Returns true if this phase represents a valid, loggable QSO
    pub fn is_loggable(&self) -> bool {
        matches!(self, QsoPhase::Confirmed | QsoPhase::Complete)
    }
    
    /// Returns true if we're past the initial handshake
    pub fn is_in_progress(&self) -> bool {
        *self >= QsoPhase::ReportSent
    }
}

/// State machine for a single QSO
#[derive(Debug, Clone)]
pub struct QsoState {
    pub phase: QsoPhase,
    pub role: QsoRole,
    
    // Flags for what we've sent
    pub sent_grid: bool,
    pub sent_report: bool,
    pub sent_roger: bool,  // R+report
    pub sent_rrr: bool,    // RRR or RR73
    pub sent_73: bool,
    
    // Flags for what we've received
    pub rcvd_grid: bool,
    pub rcvd_report: bool,
    pub rcvd_roger: bool,
    pub rcvd_rrr: bool,
    pub rcvd_73: bool,
    
    // The actual values
    pub their_grid: Option<String>,
    pub report_sent: Option<String>,
    pub report_rcvd: Option<String>,
}

impl Default for QsoState {
    fn default() -> Self {
        Self {
            phase: QsoPhase::Started,
            role: QsoRole::Unknown,
            sent_grid: false,
            sent_report: false,
            sent_roger: false,
            sent_rrr: false,
            sent_73: false,
            rcvd_grid: false,
            rcvd_report: false,
            rcvd_roger: false,
            rcvd_rrr: false,
            rcvd_73: false,
            their_grid: None,
            report_sent: None,
            report_rcvd: None,
        }
    }
}

impl QsoState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Update state based on a transmitted message
    pub fn process_tx(&mut self, msg_text: &str) {
        let parts: Vec<&str> = msg_text.split_whitespace().collect();
        if parts.len() < 2 {
            return;
        }
        
        // Check for grid (4 char maidenhead)
        for part in &parts {
            if is_grid(part) {
                self.sent_grid = true;
            }
        }
        
        // Check last part for special tokens
        if let Some(last) = parts.last() {
            if *last == "RRR" || *last == "RR73" {
                self.sent_rrr = true;
                if *last == "RR73" {
                    self.sent_73 = true;
                }
            } else if *last == "73" {
                self.sent_73 = true;
            } else if last.starts_with('R') && is_report(&last[1..]) {
                // R-05, R+10, etc.
                self.sent_roger = true;
                self.sent_report = true;
                self.report_sent = Some(last[1..].to_string());
            } else if is_report(last) {
                // -05, +10, etc.
                self.sent_report = true;
                self.report_sent = Some(last.to_string());
            }
        }
        
        self.update_phase();
    }
    
    /// Update state based on a received message
    pub fn process_rx(&mut self, msg_text: &str) {
        let parts: Vec<&str> = msg_text.split_whitespace().collect();
        if parts.len() < 2 {
            return;
        }
        
        // Check for grid (4 char maidenhead)
        for part in &parts {
            if is_grid(part) {
                self.rcvd_grid = true;
                self.their_grid = Some(part.to_string());
            }
        }
        
        // Check last part for special tokens
        if let Some(last) = parts.last() {
            if *last == "RRR" || *last == "RR73" {
                self.rcvd_rrr = true;
                if *last == "RR73" {
                    self.rcvd_73 = true;
                }
            } else if *last == "73" {
                self.rcvd_73 = true;
            } else if last.starts_with('R') && is_report(&last[1..]) {
                // R-05, R+10, etc.
                self.rcvd_roger = true;
                self.rcvd_report = true;
                self.report_rcvd = Some(last[1..].to_string());
            } else if is_report(last) {
                // -05, +10, etc.
                self.rcvd_report = true;
                self.report_rcvd = Some(last.to_string());
            }
        }
        
        self.update_phase();
    }
    
    /// Recalculate the current phase based on flags
    fn update_phase(&mut self) {
        // Work backwards from most complete to least complete
        
        // Complete: 73 exchanged (either direction) or RR73 seen
        if (self.sent_73 || self.rcvd_73) && self.is_confirmed() {
            self.phase = QsoPhase::Complete;
            return;
        }
        
        // Confirmed: RRR received OR we sent RRR (after receiving R+report)
        if self.is_confirmed() {
            self.phase = QsoPhase::Confirmed;
            return;
        }
        
        // Report received: we got their signal report (with or without R)
        if self.rcvd_report {
            self.phase = QsoPhase::ReportReceived;
            return;
        }
        
        // Report sent: we sent our signal report
        if self.sent_report {
            self.phase = QsoPhase::ReportSent;
            return;
        }
        
        // Grid received
        if self.rcvd_grid {
            self.phase = QsoPhase::GridReceived;
            return;
        }
        
        // Grid sent
        if self.sent_grid {
            self.phase = QsoPhase::GridSent;
            return;
        }
        
        // Default: started but nothing meaningful yet
        self.phase = QsoPhase::Started;
    }
    
    /// Check if the QSO is confirmed (all required elements exchanged)
    fn is_confirmed(&self) -> bool {
        // Valid confirmation scenarios:
        // 1. I called CQ: I got R+report, I sent RRR/RR73
        // 2. I answered CQ: I sent R+report, I got RRR/RR73
        
        // Case 1: I received R+report and sent RRR
        if self.rcvd_roger && self.sent_rrr {
            return true;
        }
        
        // Case 2: I sent R+report and received RRR
        if self.sent_roger && self.rcvd_rrr {
            return true;
        }
        
        // Special case: Both sides sent RR73 (unlikely but valid)
        if self.sent_rrr && self.rcvd_rrr {
            return true;
        }
        
        false
    }
    
    /// Is this QSO valid for logging?
    pub fn is_loggable(&self) -> bool {
        // Must have exchanged reports
        if self.report_sent.is_none() || self.report_rcvd.is_none() {
            return false;
        }
        
        // Must be confirmed
        self.phase.is_loggable()
    }
}

/// Check if a string looks like a grid square
fn is_grid(s: &str) -> bool {
    if s.len() != 4 {
        return false;
    }
    let chars: Vec<char> = s.chars().collect();
    chars[0].is_ascii_uppercase() 
        && chars[1].is_ascii_uppercase()
        && chars[2].is_ascii_digit() 
        && chars[3].is_ascii_digit()
}

/// Check if a string looks like a signal report (-05, +10, etc.)
fn is_report(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    let s = if s.starts_with('-') || s.starts_with('+') {
        &s[1..]
    } else {
        s
    };
    
    // Should be 1-2 digits
    s.len() <= 2 && s.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cq_initiator_flow() {
        let mut state = QsoState::new();
        state.role = QsoRole::Initiator;
        
        // I called CQ, they answered with grid
        state.process_rx("K1ABC G0XYZ IO91");
        assert!(state.rcvd_grid);
        assert_eq!(state.their_grid, Some("IO91".to_string()));
        
        // I sent my report
        state.process_tx("G0XYZ K1ABC -19");
        assert!(state.sent_report);
        assert_eq!(state.report_sent, Some("-19".to_string()));
        
        // They sent R+report
        state.process_rx("K1ABC G0XYZ R-22");
        assert!(state.rcvd_roger);
        assert!(state.rcvd_report);
        
        // I sent RRR
        state.process_tx("G0XYZ K1ABC RRR");
        assert!(state.sent_rrr);
        assert!(state.is_loggable());
        assert_eq!(state.phase, QsoPhase::Confirmed);
        
        // They sent 73
        state.process_rx("K1ABC G0XYZ 73");
        assert!(state.rcvd_73);
        assert_eq!(state.phase, QsoPhase::Complete);
    }
    
    #[test]
    fn test_responder_flow() {
        let mut state = QsoState::new();
        state.role = QsoRole::Responder;
        
        // I answered their CQ with my grid
        state.process_tx("K1ABC G0XYZ IO91");
        assert!(state.sent_grid);
        
        // They sent their report
        state.process_rx("G0XYZ K1ABC -19");
        assert!(state.rcvd_report);
        
        // I sent R+report
        state.process_tx("K1ABC G0XYZ R-22");
        assert!(state.sent_roger);
        assert!(state.sent_report);
        
        // They sent RRR
        state.process_rx("G0XYZ K1ABC RRR");
        assert!(state.rcvd_rrr);
        assert!(state.is_loggable());
    }
    
    #[test]
    fn test_rr73_shortcut() {
        let mut state = QsoState::new();
        
        state.process_tx("K1ABC G0XYZ IO91");
        state.process_rx("G0XYZ K1ABC -19");
        state.process_tx("K1ABC G0XYZ R-22");
        
        // They send RR73 (combines RRR + 73)
        state.process_rx("G0XYZ K1ABC RR73");
        assert!(state.rcvd_rrr);
        assert!(state.rcvd_73);
        assert!(state.is_loggable());
        assert_eq!(state.phase, QsoPhase::Complete);
    }
    
    #[test]
    fn test_partial_qso_not_loggable() {
        let mut state = QsoState::new();
        
        // Only exchanged grids, no reports
        state.process_tx("K1ABC G0XYZ IO91");
        state.process_rx("G0XYZ K1ABC EM10");
        
        assert!(!state.is_loggable());
    }
}
