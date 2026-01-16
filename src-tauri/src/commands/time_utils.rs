//! Time Utilities
//!
//! Functions for parsing, normalizing, and validating ADIF time formats.

/// Format time from milliseconds since midnight UTC to HHMMSS
pub fn format_time_from_ms(time_ms: u32) -> String {
    let total_seconds = time_ms / 1000;
    let hours = (total_seconds / 3600) % 24;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{:02}{:02}{:02}", hours, minutes, seconds)
}

/// Get current UTC time as HHMMSS
pub fn get_current_utc_time() -> String {
    use chrono::Utc;
    Utc::now().format("%H%M%S").to_string()
}

/// Normalize time string to 6-character HHMMSS format (ADIF standard)
/// 
/// # Arguments
/// * `time_str` - Time string in various formats:
///   - HHMM (4 chars) → append "00" for seconds
///   - HHMMSS (6 chars) → use as-is
///   - "YYYY-MM-DD HH:MM:SS" (datetime) → extract time, remove colons
///   - "HH:MM:SS" (with colons) → remove colons
/// 
/// # Returns
/// * 6-character string in HHMMSS format
pub fn normalize_time_to_hhmmss(time_str: &str) -> String {
    let clean = time_str.trim();
    
    // Handle datetime format "YYYY-MM-DD HH:MM:SS" from QsoLogged message
    if clean.contains('-') && clean.contains(':') && clean.contains(' ') {
        if let Some(time_part) = clean.split(' ').last() {
            let no_colons = time_part.replace(':', "");
            if no_colons.len() >= 6 {
                return no_colons[..6].to_string();
            } else if no_colons.len() >= 4 {
                return format!("{:0<6}", no_colons);
            }
        }
    }
    
    // Handle time with colons "HH:MM:SS" or "HH:MM"
    if clean.contains(':') {
        let no_colons = clean.replace(':', "");
        if no_colons.len() >= 6 {
            return no_colons[..6].to_string();
        } else if no_colons.len() >= 4 {
            return format!("{:0<6}", no_colons);
        }
    }
    
    // Standard HHMMSS or HHMM format
    if clean.len() >= 6 {
        clean[..6].to_string()
    } else if clean.len() == 4 {
        format!("{}00", clean)
    } else if clean.len() == 5 {
        format!("{}0", clean)
    } else if clean.is_empty() {
        "000000".to_string()
    } else {
        format!("{:0<6}", clean)
    }
}

/// Extract HHMM (first 4 chars) from time string for duplicate comparison
pub fn extract_hhmm(time_str: &str) -> String {
    let clean = time_str.trim();
    if clean.len() >= 4 {
        clean[..4].to_string()
    } else {
        format!("{:0<4}", clean)
    }
}

/// Convert time string to minutes since midnight for time difference calculations
#[allow(dead_code)]
pub fn time_to_minutes(time_str: &str) -> Option<u32> {
    let clean = time_str.trim();
    if clean.len() < 4 {
        return None;
    }
    let hours: u32 = clean[..2].parse().ok()?;
    let minutes: u32 = clean[2..4].parse().ok()?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(hours * 60 + minutes)
}

/// Convert time string (HHMMSS or HHMM) to seconds since midnight
pub fn time_to_seconds(time_str: &str) -> Option<u32> {
    let clean = time_str.trim();
    if clean.len() < 4 {
        return None;
    }
    let hours: u32 = clean[..2].parse().ok()?;
    let minutes: u32 = clean[2..4].parse().ok()?;
    let seconds: u32 = if clean.len() >= 6 {
        clean[4..6].parse().ok()?
    } else {
        0
    };
    if hours > 23 || minutes > 59 || seconds > 59 {
        return None;
    }
    Some(hours * 3600 + minutes * 60 + seconds)
}

/// Calculate absolute time difference in minutes between two time strings
#[allow(dead_code)]
pub fn time_difference_minutes(time1: &str, time2: &str) -> Option<u32> {
    let m1 = time_to_minutes(time1)?;
    let m2 = time_to_minutes(time2)?;
    let diff = if m1 > m2 { m1 - m2 } else { m2 - m1 };
    Some(if diff > 720 { 1440 - diff } else { diff })
}

/// Normalize date string to 8-character YYYYMMDD format (ADIF standard)
#[allow(dead_code)]
pub fn normalize_date_to_yyyymmdd(date_str: &str) -> String {
    let digits: String = date_str.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 8 {
        digits[..8].to_string()
    } else {
        format!("{:0<8}", digits)
    }
}

/// Validate ADIF date format (YYYYMMDD)
pub fn is_valid_adif_date(date_str: &str) -> bool {
    if date_str.len() != 8 {
        return false;
    }
    if !date_str.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let year: u32 = match date_str[..4].parse() {
        Ok(y) => y,
        Err(_) => return false,
    };
    let month: u32 = match date_str[4..6].parse() {
        Ok(m) => m,
        Err(_) => return false,
    };
    let day: u32 = match date_str[6..8].parse() {
        Ok(d) => d,
        Err(_) => return false,
    };
    year >= 1900 && year <= 2100 && month >= 1 && month <= 12 && day >= 1 && day <= 31
}

/// Validate ADIF time format (HHMM or HHMMSS)
pub fn is_valid_adif_time(time_str: &str) -> bool {
    let len = time_str.len();
    if len != 4 && len != 6 {
        return false;
    }
    if !time_str.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let hours: u32 = match time_str[..2].parse() {
        Ok(h) => h,
        Err(_) => return false,
    };
    let minutes: u32 = match time_str[2..4].parse() {
        Ok(m) => m,
        Err(_) => return false,
    };
    if hours > 23 || minutes > 59 {
        return false;
    }
    if len == 6 {
        let seconds: u32 = match time_str[4..6].parse() {
            Ok(s) => s,
            Err(_) => return false,
        };
        if seconds > 59 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_time_to_hhmmss_4_chars() {
        assert_eq!(normalize_time_to_hhmmss("1234"), "123400");
        assert_eq!(normalize_time_to_hhmmss("0000"), "000000");
        assert_eq!(normalize_time_to_hhmmss("2359"), "235900");
    }

    #[test]
    fn test_normalize_time_to_hhmmss_6_chars() {
        assert_eq!(normalize_time_to_hhmmss("123456"), "123456");
        assert_eq!(normalize_time_to_hhmmss("000000"), "000000");
    }

    #[test]
    fn test_normalize_time_to_hhmmss_datetime_format() {
        assert_eq!(normalize_time_to_hhmmss("2026-01-08 23:24:45"), "232445");
        assert_eq!(normalize_time_to_hhmmss("2026-01-08 00:00:00"), "000000");
    }

    #[test]
    fn test_normalize_time_to_hhmmss_time_with_colons() {
        assert_eq!(normalize_time_to_hhmmss("23:24:45"), "232445");
        assert_eq!(normalize_time_to_hhmmss("00:00:00"), "000000");
    }

    #[test]
    fn test_extract_hhmm() {
        assert_eq!(extract_hhmm("123456"), "1234");
        assert_eq!(extract_hhmm("1234"), "1234");
    }

    #[test]
    fn test_time_to_seconds() {
        assert_eq!(time_to_seconds("000000"), Some(0));
        assert_eq!(time_to_seconds("120000"), Some(43200));
        assert_eq!(time_to_seconds("235959"), Some(86399));
    }

    #[test]
    fn test_is_valid_adif_date() {
        assert!(is_valid_adif_date("20260108"));
        assert!(!is_valid_adif_date("2026-01-08"));
        assert!(!is_valid_adif_date("20261301"));
    }

    #[test]
    fn test_is_valid_adif_time() {
        assert!(is_valid_adif_time("1234"));
        assert!(is_valid_adif_time("123456"));
        assert!(!is_valid_adif_time("2400"));
    }
}
