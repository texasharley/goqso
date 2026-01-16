// US States and Canadian Provinces for award tracking
// Source: USPS (US), Canada Post (CA), ARRL WAS/RAC rules
// Generated: 2026-01-16T00:07:11Z from us_states.json and canadian_provinces.json
//
// DO NOT MANUALLY EDIT - regenerate using: python scripts/generate_reference.py
//
#![allow(dead_code)]

// =========================================================================
// US STATES (WAS Award - 50 states required)
// =========================================================================

/// US State information for WAS tracking
#[derive(Debug, Clone)]
pub struct UsState {
    /// Two-letter USPS abbreviation
    pub code: &'static str,
    /// Full state name
    pub name: &'static str,
}

/// All 50 US states
pub const US_STATES: &[UsState] = &[
    UsState { code: "AL", name: "Alabama" },
    UsState { code: "AK", name: "Alaska" },
    UsState { code: "AZ", name: "Arizona" },
    UsState { code: "AR", name: "Arkansas" },
    UsState { code: "CA", name: "California" },
    UsState { code: "CO", name: "Colorado" },
    UsState { code: "CT", name: "Connecticut" },
    UsState { code: "DE", name: "Delaware" },
    UsState { code: "FL", name: "Florida" },
    UsState { code: "GA", name: "Georgia" },
    UsState { code: "HI", name: "Hawaii" },
    UsState { code: "ID", name: "Idaho" },
    UsState { code: "IL", name: "Illinois" },
    UsState { code: "IN", name: "Indiana" },
    UsState { code: "IA", name: "Iowa" },
    UsState { code: "KS", name: "Kansas" },
    UsState { code: "KY", name: "Kentucky" },
    UsState { code: "LA", name: "Louisiana" },
    UsState { code: "ME", name: "Maine" },
    UsState { code: "MD", name: "Maryland" },
    UsState { code: "MA", name: "Massachusetts" },
    UsState { code: "MI", name: "Michigan" },
    UsState { code: "MN", name: "Minnesota" },
    UsState { code: "MS", name: "Mississippi" },
    UsState { code: "MO", name: "Missouri" },
    UsState { code: "MT", name: "Montana" },
    UsState { code: "NE", name: "Nebraska" },
    UsState { code: "NV", name: "Nevada" },
    UsState { code: "NH", name: "New Hampshire" },
    UsState { code: "NJ", name: "New Jersey" },
    UsState { code: "NM", name: "New Mexico" },
    UsState { code: "NY", name: "New York" },
    UsState { code: "NC", name: "North Carolina" },
    UsState { code: "ND", name: "North Dakota" },
    UsState { code: "OH", name: "Ohio" },
    UsState { code: "OK", name: "Oklahoma" },
    UsState { code: "OR", name: "Oregon" },
    UsState { code: "PA", name: "Pennsylvania" },
    UsState { code: "RI", name: "Rhode Island" },
    UsState { code: "SC", name: "South Carolina" },
    UsState { code: "SD", name: "South Dakota" },
    UsState { code: "TN", name: "Tennessee" },
    UsState { code: "TX", name: "Texas" },
    UsState { code: "UT", name: "Utah" },
    UsState { code: "VT", name: "Vermont" },
    UsState { code: "VA", name: "Virginia" },
    UsState { code: "WA", name: "Washington" },
    UsState { code: "WV", name: "West Virginia" },
    UsState { code: "WI", name: "Wisconsin" },
    UsState { code: "WY", name: "Wyoming" },
];

// =========================================================================
// CANADIAN PROVINCES AND TERRITORIES
// =========================================================================

/// Canadian Province/Territory information for RAC award tracking
#[derive(Debug, Clone)]
pub struct CanadianProvince {
    /// Two-letter Canada Post abbreviation
    pub code: &'static str,
    /// Full province/territory name
    pub name: &'static str,
}

/// All 13 Canadian provinces and territories
pub const CANADIAN_PROVINCES: &[CanadianProvince] = &[
    // Provinces (10)
    CanadianProvince { code: "AB", name: "Alberta" },
    CanadianProvince { code: "BC", name: "British Columbia" },
    CanadianProvince { code: "MB", name: "Manitoba" },
    CanadianProvince { code: "NB", name: "New Brunswick" },
    CanadianProvince { code: "NL", name: "Newfoundland and Labrador" },
    CanadianProvince { code: "NS", name: "Nova Scotia" },
    CanadianProvince { code: "ON", name: "Ontario" },
    CanadianProvince { code: "PE", name: "Prince Edward Island" },
    CanadianProvince { code: "QC", name: "Quebec" },
    CanadianProvince { code: "SK", name: "Saskatchewan" },
    // Territories (3)
    CanadianProvince { code: "NT", name: "Northwest Territories" },
    CanadianProvince { code: "NU", name: "Nunavut" },
    CanadianProvince { code: "YT", name: "Yukon" },
];

/// Get US state by code
pub fn get_us_state(code: &str) -> Option<&'static UsState> {
    let code_upper = code.to_uppercase();
    US_STATES.iter().find(|s| s.code == code_upper)
}

/// Get Canadian province by code
pub fn get_canadian_province(code: &str) -> Option<&'static CanadianProvince> {
    let code_upper = code.to_uppercase();
    CANADIAN_PROVINCES.iter().find(|p| p.code == code_upper)
}

/// Check if a state code is a valid US state
pub fn is_valid_us_state(code: &str) -> bool {
    get_us_state(code).is_some()
}

/// Check if a code is a valid Canadian province/territory
pub fn is_valid_canadian_province(code: &str) -> bool {
    get_canadian_province(code).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_us_states_count() {
        assert_eq!(US_STATES.len(), 50);
    }

    #[test]
    fn test_canadian_provinces_count() {
        assert_eq!(CANADIAN_PROVINCES.len(), 13);
    }

    #[test]
    fn test_us_state_lookup() {
        assert!(get_us_state("MN").is_some());
        assert!(get_us_state("mn").is_some()); // case insensitive
        assert!(get_us_state("XX").is_none());
    }

    #[test]
    fn test_canadian_province_lookup() {
        assert!(get_canadian_province("ON").is_some());
        assert!(get_canadian_province("on").is_some()); // case insensitive
        assert!(get_canadian_province("XX").is_none());
    }
}
