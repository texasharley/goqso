// =============================================================================
// QSO Tracker - State Machine for Auto-Logging FT8/FT4 QSOs
// =============================================================================
//
// NOTE: This module is intentionally not used yet - planned for future auto-logging
// improvements where we track QSO state machine to log only complete exchanges.
//
#![allow(dead_code)]

// This module tracks the progress of QSOs in progress and determines when a 
// valid, complete QSO has occurred. A QSO is only logged when ALL required
// elements have been exchanged.
//
// By longstanding tradition, a minimally valid QSO requires:
// 1. Exchange of callsigns (both stations know who they're talking to)
// 2. Exchange of signal reports (or other information like grids)
// 3. Acknowledgments (RRR, RR73, or 73)
//
// Standard FT8 QSO Flow:
// ======================
// 
// Step  | TX Station | Message           | What Happened
// ------|------------|-------------------|--------------------------------
//   1   | K1ABC      | CQ K1ABC FN42     | K1ABC calls CQ
//   2   | G0XYZ      | K1ABC G0XYZ IO91  | G0XYZ answers with grid
//   3   | K1ABC      | G0XYZ K1ABC –19   | K1ABC sends signal report
//   4   | G0XYZ      | K1ABC G0XYZ R-22  | G0XYZ acknowledges + sends report
//   5   | K1ABC      | G0XYZ K1ABC RRR   | K1ABC acknowledges
//   6   | G0XYZ      | K1ABC G0XYZ 73    | G0XYZ sends 73 (optional)
//
// The QSO is considered COMPLETE at step 5 (RRR received/sent).
// Step 6 (73) is courtesy but not required.
//
// Edge Cases to Handle:
// =====================
// - Multiple transmissions of same message (retries due to no response)
// - RR73 instead of RRR + 73 (common shortcut)
// - Contest exchanges (different format)
// - Free text messages
// - QSOs that timeout without completion
// - Partial QSOs (should NOT be logged)
// - Late responses (someone answers a CQ we already gave up on)
//
// Validation Requirements (similar to JTAlert):
// =============================================
// From MY perspective (I am the logging station), a valid QSO requires:
//
// If I called CQ:
//   ✓ I received their callsign
//   ✓ I received their grid (or R+report)
//   ✓ I sent my report
//   ✓ I received acknowledgment (R-xx, RRR, RR73)
//   ✓ I sent acknowledgment (RRR, RR73)
//
// If I answered their CQ:
//   ✓ I sent my grid
//   ✓ I received their report
//   ✓ I sent R+report
//   ✓ I received acknowledgment (RRR, RR73)
//
// The key insight: We track both TX and RX messages for our QSO.

use std::time::{Duration, Instant};

mod state;
mod tracker;

pub use state::{QsoPhase, QsoRole};

/// A message we observed (either transmitted or received)
#[derive(Debug, Clone)]
pub struct ObservedMessage {
    /// UTC timestamp
    pub timestamp: Instant,
    /// The raw message text
    pub message: String,
    /// Was this transmitted by us (TX) or received (RX)?
    pub is_tx: bool,
    /// Signal-to-noise ratio (dB)
    pub snr: Option<i32>,
    /// Frequency offset (Hz)
    pub freq_offset: Option<u32>,
}

/// Represents a QSO in progress
#[derive(Debug, Clone)]
pub struct QsoInProgress {
    /// Their callsign
    pub their_call: String,
    /// My callsign (from WSJT-X status)
    pub my_call: String,
    /// Current phase of the QSO
    pub phase: QsoPhase,
    /// Role: did we initiate (CQ) or respond?
    pub role: QsoRole,
    /// Their grid square (if received)
    pub their_grid: Option<String>,
    /// Report we sent them
    pub report_sent: Option<String>,
    /// Report they sent us  
    pub report_rcvd: Option<String>,
    /// When this QSO started
    pub started_at: Instant,
    /// Last activity time
    pub last_activity: Instant,
    /// All messages exchanged in this QSO
    pub messages: Vec<ObservedMessage>,
    /// Has this QSO been completed and logged?
    pub completed: bool,
    /// Frequency (Hz)
    pub freq_hz: u64,
    /// Mode (FT8, FT4, etc.)
    pub mode: String,
}

impl QsoInProgress {
    /// Check if this QSO has timed out (no activity for a while)
    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }
    
    /// Check if this QSO meets minimum requirements for logging
    pub fn is_valid_for_logging(&self) -> bool {
        // Must have their call
        if self.their_call.is_empty() {
            return false;
        }
        
        // Must have exchanged reports
        if self.report_sent.is_none() || self.report_rcvd.is_none() {
            return false;
        }
        
        // Must have reached at least the Confirmed phase
        matches!(self.phase, QsoPhase::Confirmed | QsoPhase::Complete)
    }
}

/// Result of processing a message
#[derive(Debug, Clone)]
pub enum QsoEvent {
    /// A new QSO has started
    Started { their_call: String },
    /// QSO progressed to a new phase
    Progressed { their_call: String, phase: QsoPhase },
    /// QSO is complete and ready to log
    Complete(QsoInProgress),
    /// QSO was abandoned (timed out or they QSYed)
    Abandoned { their_call: String },
    /// No change
    None,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_qso_flow() {
        // TODO: Add tests for QSO state machine
    }
}
