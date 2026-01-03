// US States for Worked All States (WAS) award
// Source: ARRL WAS rules
//
// All 50 US states are required for WAS award
// State abbreviations follow USPS standards

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

/// Get a state by its two-letter code
pub fn get_state(code: &str) -> Option<&'static UsState> {
    let code_upper = code.to_uppercase();
    US_STATES.iter().find(|s| s.code == code_upper)
}

/// Get a state by name (case-insensitive)
pub fn get_state_by_name(name: &str) -> Option<&'static UsState> {
    let name_lower = name.to_lowercase();
    US_STATES.iter().find(|s| s.name.to_lowercase() == name_lower)
}

/// Total number of states for WAS
pub const WAS_TOTAL: usize = 50;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_count() {
        assert_eq!(US_STATES.len(), 50);
    }

    #[test]
    fn test_get_state() {
        assert!(get_state("TX").is_some());
        assert!(get_state("tx").is_some()); // case insensitive
        assert_eq!(get_state("TX").unwrap().name, "Texas");
    }

    #[test]
    fn test_get_state_by_name() {
        assert!(get_state_by_name("California").is_some());
        assert!(get_state_by_name("california").is_some()); // case insensitive
        assert_eq!(get_state_by_name("California").unwrap().code, "CA");
    }
}
