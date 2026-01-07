// Amateur Radio Band Definitions
// Reference: ARRL Band Plan and ADIF 3.1.4 Specification
//
// This module provides frequency-to-band conversion and band metadata.

/// HF and VHF/UHF amateur radio bands
/// Returns the ADIF band name for a given frequency in MHz
pub fn freq_to_band(freq_mhz: f64) -> Option<&'static str> {
    match freq_mhz {
        // HF Bands
        f if (1.8..=2.0).contains(&f) => Some("160m"),
        f if (3.5..=4.0).contains(&f) => Some("80m"),
        f if (5.0..=5.5).contains(&f) => Some("60m"),   // Channelized in US
        f if (7.0..=7.3).contains(&f) => Some("40m"),
        f if (10.1..=10.15).contains(&f) => Some("30m"),
        f if (14.0..=14.35).contains(&f) => Some("20m"),
        f if (18.068..=18.168).contains(&f) => Some("17m"),
        f if (21.0..=21.45).contains(&f) => Some("15m"),
        f if (24.89..=24.99).contains(&f) => Some("12m"),
        f if (28.0..=29.7).contains(&f) => Some("10m"),
        // VHF/UHF Bands
        f if (50.0..=54.0).contains(&f) => Some("6m"),
        f if (144.0..=148.0).contains(&f) => Some("2m"),
        f if (222.0..=225.0).contains(&f) => Some("1.25m"),
        f if (420.0..=450.0).contains(&f) => Some("70cm"),
        f if (902.0..=928.0).contains(&f) => Some("33cm"),
        f if (1240.0..=1300.0).contains(&f) => Some("23cm"),
        _ => None,
    }
}

/// Convert frequency in Hz to band
pub fn freq_hz_to_band(freq_hz: u64) -> Option<&'static str> {
    freq_to_band(freq_hz as f64 / 1_000_000.0)
}

/// Get the typical FT8 frequency for a band (in Hz)
/// These are the most common FT8 frequencies per band
pub fn get_ft8_freq(band: &str) -> Option<u64> {
    match band.to_lowercase().as_str() {
        "160m" => Some(1_840_000),
        "80m" => Some(3_573_000),
        "60m" => Some(5_357_000),
        "40m" => Some(7_074_000),
        "30m" => Some(10_136_000),
        "20m" => Some(14_074_000),
        "17m" => Some(18_100_000),
        "15m" => Some(21_074_000),
        "12m" => Some(24_915_000),
        "10m" => Some(28_074_000),
        "6m" => Some(50_313_000),
        "2m" => Some(144_174_000),
        "70cm" => Some(432_065_000),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freq_to_band() {
        assert_eq!(freq_to_band(7.074), Some("40m"));
        assert_eq!(freq_to_band(14.074), Some("20m"));
        assert_eq!(freq_to_band(3.573), Some("80m"));
        assert_eq!(freq_to_band(50.313), Some("6m"));
        assert_eq!(freq_to_band(144.174), Some("2m"));
        assert_eq!(freq_to_band(999.0), None);
    }

    #[test]
    fn test_freq_hz_to_band() {
        assert_eq!(freq_hz_to_band(7_074_000), Some("40m"));
        assert_eq!(freq_hz_to_band(14_074_000), Some("20m"));
    }

    #[test]
    fn test_get_ft8_freq() {
        assert_eq!(get_ft8_freq("40m"), Some(7_074_000));
        assert_eq!(get_ft8_freq("20M"), Some(14_074_000)); // case insensitive
        assert_eq!(get_ft8_freq("unknown"), None);
    }
}
