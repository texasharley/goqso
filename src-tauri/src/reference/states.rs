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

/// Approximate state from Maidenhead grid square
/// This is an approximation based on grid center points
/// Returns (state_code, state_name) if determinable
pub fn grid_to_state(grid: &str) -> Option<(&'static str, &'static str)> {
    if grid.len() < 4 {
        return None;
    }
    
    let grid_upper = grid.to_uppercase();
    let chars: Vec<char> = grid_upper.chars().collect();
    
    // Parse grid to approximate coordinates
    // Grids: AA-RR for field (18x18), 00-99 for square
    let field_lon = (chars[0] as i32) - ('A' as i32);
    let field_lat = (chars[1] as i32) - ('A' as i32);
    let square_lon = (chars[2] as i32) - ('0' as i32);
    let square_lat = (chars[3] as i32) - ('0' as i32);
    
    // Approximate center of grid square
    let lon = -180.0 + (field_lon as f64 * 20.0) + (square_lon as f64 * 2.0) + 1.0;
    let lat = -90.0 + (field_lat as f64 * 10.0) + (square_lat as f64 * 1.0) + 0.5;
    
    // Simple bounding box lookup for continental US states
    // This is approximate but works for most cases
    approximate_state_from_coords(lat, lon)
}

/// Approximate state from lat/lon coordinates
/// Uses simple bounding boxes - not precise but good enough for ham radio
fn approximate_state_from_coords(lat: f64, lon: f64) -> Option<(&'static str, &'static str)> {
    // Check if in continental US range
    if lat < 24.0 || lat > 50.0 || lon < -125.0 || lon > -66.0 {
        // Check Alaska
        if lat > 51.0 && lat < 72.0 && lon > -180.0 && lon < -130.0 {
            return Some(("AK", "Alaska"));
        }
        // Check Hawaii  
        if lat > 18.0 && lat < 23.0 && lon > -161.0 && lon < -154.0 {
            return Some(("HI", "Hawaii"));
        }
        return None;
    }
    
    // Rough state approximations based on grid patterns
    // These are commonly seen grid prefixes mapped to states
    match (lat as i32, lon as i32) {
        // Texas
        (26..=36, -106..=-93) => Some(("TX", "Texas")),
        // California
        (32..=42, -125..=-114) => Some(("CA", "California")),
        // Florida
        (24..=31, -88..=-80) => Some(("FL", "Florida")),
        // New York
        (40..=45, -80..=-72) => Some(("NY", "New York")),
        // Pennsylvania
        (39..=42, -81..=-75) => Some(("PA", "Pennsylvania")),
        // Illinois
        (37..=43, -92..=-87) => Some(("IL", "Illinois")),
        // Ohio
        (38..=42, -85..=-80) => Some(("OH", "Ohio")),
        // Georgia
        (30..=35, -86..=-81) => Some(("GA", "Georgia")),
        // Michigan
        (41..=48, -91..=-82) => Some(("MI", "Michigan")),
        // North Carolina
        (33..=37, -85..=-75) => Some(("NC", "North Carolina")),
        // Virginia
        (36..=40, -84..=-75) => Some(("VA", "Virginia")),
        // Washington
        (45..=49, -125..=-117) => Some(("WA", "Washington")),
        // Arizona
        (31..=37, -115..=-109) => Some(("AZ", "Arizona")),
        // Colorado
        (36..=41, -109..=-102) => Some(("CO", "Colorado")),
        // Minnesota
        (43..=49, -97..=-89) => Some(("MN", "Minnesota")),
        // Wisconsin
        (42..=47, -93..=-87) => Some(("WI", "Wisconsin")),
        // Tennessee
        (35..=37, -90..=-82) => Some(("TN", "Tennessee")),
        // Missouri
        (36..=41, -96..=-89) => Some(("MO", "Missouri")),
        // Indiana
        (37..=42, -88..=-84) => Some(("IN", "Indiana")),
        // Oklahoma
        (33..=37, -103..=-94) => Some(("OK", "Oklahoma")),
        // Oregon
        (42..=46, -125..=-117) => Some(("OR", "Oregon")),
        // Kansas
        (36..=40, -102..=-94) => Some(("KS", "Kansas")),
        // Nevada
        (35..=42, -120..=-114) => Some(("NV", "Nevada")),
        // New Mexico
        (31..=37, -109..=-103) => Some(("NM", "New Mexico")),
        // Louisiana
        (29..=33, -94..=-89) => Some(("LA", "Louisiana")),
        // Kentucky
        (36..=39, -90..=-82) => Some(("KY", "Kentucky")),
        // Alabama
        (30..=35, -89..=-85) => Some(("AL", "Alabama")),
        // Arkansas
        (33..=37, -95..=-89) => Some(("AR", "Arkansas")),
        // Iowa
        (40..=44, -97..=-90) => Some(("IA", "Iowa")),
        // Mississippi
        (30..=35, -92..=-88) => Some(("MS", "Mississippi")),
        // Utah
        (37..=42, -114..=-109) => Some(("UT", "Utah")),
        // Nebraska
        (40..=43, -104..=-96) => Some(("NE", "Nebraska")),
        // South Dakota
        (43..=46, -104..=-96) => Some(("SD", "South Dakota")),
        // North Dakota
        (46..=49, -104..=-96) => Some(("ND", "North Dakota")),
        // Montana
        (45..=49, -117..=-104) => Some(("MT", "Montana")),
        // Idaho
        (42..=49, -117..=-111) => Some(("ID", "Idaho")),
        // Wyoming
        (41..=45, -111..=-104) => Some(("WY", "Wyoming")),
        // West Virginia
        (37..=40, -83..=-77) => Some(("WV", "West Virginia")),
        // South Carolina
        (32..=35, -84..=-79) => Some(("SC", "South Carolina")),
        // Maryland
        (38..=40, -79..=-75) => Some(("MD", "Maryland")),
        // Massachusetts
        (41..=43, -74..=-70) => Some(("MA", "Massachusetts")),
        // Connecticut
        (41..=42, -74..=-72) => Some(("CT", "Connecticut")),
        // New Jersey
        (39..=42, -76..=-74) => Some(("NJ", "New Jersey")),
        // Maine
        (43..=48, -71..=-67) => Some(("ME", "Maine")),
        // New Hampshire
        (42..=45, -73..=-70) => Some(("NH", "New Hampshire")),
        // Vermont
        (42..=45, -74..=-71) => Some(("VT", "Vermont")),
        // Rhode Island
        (41..=42, -72..=-71) => Some(("RI", "Rhode Island")),
        // Delaware
        (38..=40, -76..=-75) => Some(("DE", "Delaware")),
        _ => None,
    }
}

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
