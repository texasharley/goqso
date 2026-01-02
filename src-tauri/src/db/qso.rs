// QSO database operations

use super::schema::Qso;

/// Mode groups for LoTW matching
pub fn mode_group(mode: &str) -> &'static str {
    match mode.to_uppercase().as_str() {
        "CW" => "CW",
        "SSB" | "USB" | "LSB" | "AM" | "FM" => "PHONE",
        _ => "DATA", // FT8, FT4, RTTY, PSK31, JT65, etc.
    }
}

/// Convert frequency to band
pub fn freq_to_band(freq_mhz: f64) -> Option<&'static str> {
    match freq_mhz {
        f if (1.8..=2.0).contains(&f) => Some("160m"),
        f if (3.5..=4.0).contains(&f) => Some("80m"),
        f if (5.3..=5.4).contains(&f) => Some("60m"),
        f if (7.0..=7.3).contains(&f) => Some("40m"),
        f if (10.1..=10.15).contains(&f) => Some("30m"),
        f if (14.0..=14.35).contains(&f) => Some("20m"),
        f if (18.068..=18.168).contains(&f) => Some("17m"),
        f if (21.0..=21.45).contains(&f) => Some("15m"),
        f if (24.89..=24.99).contains(&f) => Some("12m"),
        f if (28.0..=29.7).contains(&f) => Some("10m"),
        f if (50.0..=54.0).contains(&f) => Some("6m"),
        f if (144.0..=148.0).contains(&f) => Some("2m"),
        f if (420.0..=450.0).contains(&f) => Some("70cm"),
        _ => None,
    }
}

/// Check if two QSOs are duplicates (same call, band, mode within time window)
pub fn is_duplicate(
    qso1_call: &str,
    qso1_band: &str,
    qso1_mode: &str,
    qso1_datetime: &str,
    qso2_call: &str,
    qso2_band: &str,
    qso2_mode: &str,
    qso2_datetime: &str,
    window_minutes: i32,
) -> bool {
    if !qso1_call.eq_ignore_ascii_case(qso2_call) {
        return false;
    }
    if qso1_band != qso2_band {
        return false;
    }
    if qso1_mode != qso2_mode {
        return false;
    }
    
    // TODO: Parse datetimes and check window
    // For now, exact match
    qso1_datetime == qso2_datetime
}
