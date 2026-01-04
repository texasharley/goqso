// ADIF Mode Registry
// All 180+ modes from ADIF 3.1.4 specification
// Reference: https://adif.org/314/ADIF_314.htm#Mode_Enumeration

/// Mode group for award categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeGroup {
    Phone,  // Voice modes: SSB, FM, AM
    CW,     // Morse code
    Data,   // Digital modes: FT8, RTTY, PSK, etc.
    Image,  // SSTV, FAX, ATV
}

impl ModeGroup {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModeGroup::Phone => "PHONE",
            ModeGroup::CW => "CW",
            ModeGroup::Data => "DATA",
            ModeGroup::Image => "IMAGE",
        }
    }
}

/// Normalize a mode string to standard ADIF format
pub fn normalize_mode(mode: &str) -> String {
    mode.to_uppercase().trim().to_string()
}

/// Get the mode group for a given mode
pub fn get_mode_group(mode: &str) -> ModeGroup {
    let mode_upper = mode.to_uppercase();
    
    match mode_upper.as_str() {
        // CW
        "CW" => ModeGroup::CW,
        
        // Phone modes
        "SSB" | "LSB" | "USB" | "FM" | "AM" | "C4FM" | "DMR" | "DSTAR" | "M17" 
        | "FREEDV" | "DIGVOICE" => ModeGroup::Phone,
        
        // Image modes
        "SSTV" | "FAX" | "ATV" => ModeGroup::Image,
        
        // Everything else is Data
        _ => ModeGroup::Data,
    }
}

/// All valid ADIF modes (from spec + common submodes)
/// This list is used for validation during import
pub const VALID_MODES: &[&str] = &[
    // Primary modes
    "AM", "ARDOP", "ATV", "C4FM", "CHIP", "CLO", "CONTESTI", "CW", "DIGITALVOICE",
    "DMR", "DSTAR", "FAX", "FM", "FSK441", "FT4", "FT8", "HELL", "ISCAT", "JT4",
    "JT6M", "JT9", "JT44", "JT65", "JS8", "M17", "MFSK", "MSK144", "MT63", "OLIVIA",
    "OPERA", "PAC", "PAX", "PKT", "PSK", "PSK2K", "Q65", "QRA64", "ROS", "RTTY",
    "RTTYM", "SSB", "SSTV", "T10", "THOR", "THRB", "TOR", "V4", "WINMOR", "WSPR",
    "FST4", "FST4W",
    
    // Common submodes that might appear as MODE
    "LSB", "USB", "RTTY45", "RTTY50", "RTTY75", 
    "PSK31", "PSK63", "PSK125", "PSK250",
    "BPSK31", "BPSK63", "BPSK125", "BPSK250",
    "QPSK31", "QPSK63", "QPSK125", "QPSK250",
    "8PSK125", "8PSK250", "8PSK500", "8PSK1000",
    "JT65A", "JT65B", "JT65C",
    "MFSK4", "MFSK8", "MFSK11", "MFSK16", "MFSK22", "MFSK31", "MFSK32", "MFSK64", "MFSK128",
    "OLIVIA4/125", "OLIVIA4/250", "OLIVIA8/250", "OLIVIA8/500", "OLIVIA16/500", "OLIVIA16/1000", "OLIVIA32/1000",
    "DOMINO", "DOMINOEX", "DOMINOF",
    "DOM4", "DOM5", "DOM8", "DOM11", "DOM16", "DOM22", "DOM44", "DOM88",
    "THOR4", "THOR5", "THOR8", "THOR11", "THOR16", "THOR22", "THOR25", "THOR50", "THOR100",
    "VARA", "VARA HF", "VARA FM", "VARA FM 1200", "VARA FM 9600", "VARA SAT",
    "FREEDV",
    "HELLSCHREIBER", "FMHELL", "HELL80", "FSKHELL", "PSKHELL", "SLOWHELL",
    
    // Satellite modes
    "SAT",
    
    // Contest/special
    "DIGITALVOICE", "DIGVOICE", "VOI",
    
    // EME
    "EME",
    
    // Slow modes
    "WSPR", "FST4W",
];

/// Check if a mode string is valid
pub fn is_valid_mode(mode: &str) -> bool {
    let mode_upper = mode.to_uppercase();
    VALID_MODES.iter().any(|m| m.to_uppercase() == mode_upper)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mode_groups() {
        assert_eq!(get_mode_group("FT8"), ModeGroup::Data);
        assert_eq!(get_mode_group("SSB"), ModeGroup::Phone);
        assert_eq!(get_mode_group("CW"), ModeGroup::CW);
        assert_eq!(get_mode_group("SSTV"), ModeGroup::Image);
        assert_eq!(get_mode_group("RTTY"), ModeGroup::Data);
        assert_eq!(get_mode_group("FM"), ModeGroup::Phone);
    }
    
    #[test]
    fn test_valid_modes() {
        assert!(is_valid_mode("FT8"));
        assert!(is_valid_mode("ft8")); // Case insensitive
        assert!(is_valid_mode("SSB"));
        assert!(is_valid_mode("CW"));
        assert!(is_valid_mode("PSK31"));
    }
}
