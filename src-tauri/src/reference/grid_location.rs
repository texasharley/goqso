// Grid Square Location Lookup
// Converts Maidenhead grid squares to DXCC entity data
//
// NOTE: This module is intentionally not fully used yet - planned for future
// grid-based DXCC lookup when we need to determine DXCC from operating location
// rather than callsign prefix (important for portable/expedition operators).
//
#![allow(dead_code)]

// This is the PRIMARY method for determining station DXCC entity.
// Per FT8 protocol, grid is exchanged at QSO start and represents 
// WHERE THE STATION IS (not where they live).
//
// Use cases:
// - DXCC award: Determine country/entity from grid
// - Accurate DXCC for portable operators at their actual operating location
// - DXpeditions at their operating location
//
// For US STATE (WAS award): Use FCC database lookup by callsign.
// Grid-to-state is unreliable due to irregular state boundaries.
// Industry standard (CQRLOG, LoTW) uses callsign→state mapping.
//
// NO API calls - pure offline computation from grid coordinates.

use super::dxcc::{DxccEntity, DXCC_ENTITIES};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Lazily-initialized HashMap for O(1) DXCC entity lookup by entity_id
/// Uses ARRL 3-digit string format (e.g., "001" for Canada)
static DXCC_MAP: OnceLock<HashMap<&'static str, &'static DxccEntity>> = OnceLock::new();

/// Get or initialize the DXCC entity HashMap
fn get_dxcc_map() -> &'static HashMap<&'static str, &'static DxccEntity> {
    DXCC_MAP.get_or_init(|| {
        let mut map = HashMap::with_capacity(DXCC_ENTITIES.len());
        for entity in DXCC_ENTITIES {
            map.insert(entity.entity_id, entity);
        }
        map
    })
}

/// Result of grid-based location lookup
/// Contains DXCC entity info for DXCC award tracking
/// STATE field is populated from FCC database, not from grid lookup
#[derive(Debug, Clone, Default)]
pub struct GridLocation {
    /// DXCC entity ID (ARRL 3-digit format: "291" for USA, "223" for England)
    pub dxcc: Option<String>,
    /// Country/entity name (e.g., "United States of America")
    pub country: Option<String>,
    /// Continent code (e.g., "NA", "EU", "AS")
    pub continent: Option<String>,
    /// US state code for WAS award - populated from FCC database, not grid
    pub state: Option<String>,
    /// Latitude of grid center
    pub latitude: Option<f64>,
    /// Longitude of grid center
    pub longitude: Option<f64>,
}

impl GridLocation {
    /// Get DXCC entity ID as integer for database/ADIF compatibility
    /// Converts from ARRL 3-digit string format ("001") to integer (1)
    pub fn dxcc_as_i32(&self) -> Option<i32> {
        self.dxcc.as_ref().and_then(|s| s.parse::<i32>().ok())
    }
}

/// Convert a 4 or 6 character grid square to lat/lon coordinates
/// Returns (latitude, longitude) of the grid square center
pub fn grid_to_latlon(grid: &str) -> Option<(f64, f64)> {
    let grid = grid.to_uppercase();
    let len = grid.len();
    
    if len < 4 || len % 2 != 0 {
        return None;
    }
    
    let bytes = grid.as_bytes();
    
    // Validate format: Letter, Letter, Digit, Digit, [Letter, Letter]
    if bytes[0] < b'A' || bytes[0] > b'R' { return None; }
    if bytes[1] < b'A' || bytes[1] > b'R' { return None; }
    if bytes[2] < b'0' || bytes[2] > b'9' { return None; }
    if bytes[3] < b'0' || bytes[3] > b'9' { return None; }
    
    let lon_field = (bytes[0] - b'A') as f64;
    let lat_field = (bytes[1] - b'A') as f64;
    let lon_square = (bytes[2] - b'0') as f64;
    let lat_square = (bytes[3] - b'0') as f64;
    
    // Maidenhead origin is at -180°, -90°
    // Each field is 20° longitude x 10° latitude
    // Each square is 2° longitude x 1° latitude
    let mut lon = -180.0 + lon_field * 20.0 + lon_square * 2.0 + 1.0; // +1 for center
    let mut lat = -90.0 + lat_field * 10.0 + lat_square * 1.0 + 0.5;  // +0.5 for center
    
    // 6-character grid adds subsquare precision
    if len >= 6 {
        if bytes[4] < b'A' || bytes[4] > b'X' { return None; }
        if bytes[5] < b'A' || bytes[5] > b'X' { return None; }
        
        let lon_subsq = (bytes[4].to_ascii_lowercase() - b'a') as f64;
        let lat_subsq = (bytes[5].to_ascii_lowercase() - b'a') as f64;
        
        // Each subsquare is 5' longitude x 2.5' latitude
        // = 1/12 degree x 1/24 degree
        lon = -180.0 + lon_field * 20.0 + lon_square * 2.0 + lon_subsq * (2.0/24.0) + (1.0/24.0);
        lat = -90.0 + lat_field * 10.0 + lat_square * 1.0 + lat_subsq * (1.0/24.0) + (0.5/24.0);
    }
    
    Some((lat, lon))
}

/// Look up DXCC entity from Maidenhead grid square
/// 
/// This is the PRIMARY method for DXCC entity lookup.
/// Grid represents where the station IS operating from.
/// 
/// NOTE: State is NOT populated by this function.
/// For US state, use FCC database lookup by callsign.
/// 
/// Returns None if:
/// - Grid is invalid or empty
/// - Location is in international waters with no DXCC assignment
pub fn lookup_grid(grid: &str) -> GridLocation {
    if grid.is_empty() || grid.len() < 4 {
        return GridLocation::default();
    }
    
    let Some((lat, lon)) = grid_to_latlon(grid) else {
        return GridLocation::default();
    };
    
    // Find DXCC entity from coordinates
    let entity_id = coords_to_dxcc(lat, lon);
    
    if let Some(id) = entity_id {
        let dxcc_map = get_dxcc_map();
        if let Some(entity) = dxcc_map.get(id) {
            return GridLocation {
                dxcc: Some(entity.entity_id.to_string()),
                country: Some(entity.name.to_string()),
                continent: Some(entity.continent.to_string()),
                state: None, // State comes from FCC database, not grid
                latitude: Some(lat),
                longitude: Some(lon),
            };
        }
    }
    
    // No entity found - return coordinates only
    GridLocation {
        latitude: Some(lat),
        longitude: Some(lon),
        ..Default::default()
    }
}

/// Map coordinates to DXCC entity ID (ARRL 3-digit string format)
/// Uses geographic boundaries for major DXCC entities
/// 
/// This handles the critical cases where prefix lookup fails:
/// - KG4 calls in continental US (not Guantanamo)
/// - Portable operators
/// - Grid squares that span entity boundaries
fn coords_to_dxcc(lat: f64, lon: f64) -> Option<&'static str> {
    // =========================================================================
    // NORTH AMERICA - Handle the KG4 problem and US territories
    // =========================================================================
    
    // Guantanamo Bay, Cuba (entity 105) - VERY specific location
    // Naval Station Guantanamo Bay: 19.9°N, 75.1°W
    // Only grids FK29 area are actually Guantanamo
    if lat >= 19.5 && lat <= 20.5 && lon >= -75.5 && lon <= -74.5 {
        return Some("105"); // Guantanamo Bay
    }
    
    // Continental United States (entity 291)
    // Excludes Alaska (entity 6) and Hawaii (entity 110)
    if lat >= 24.5 && lat <= 49.5 && lon >= -125.0 && lon <= -66.5 {
        return Some("291"); // USA
    }
    
    // Alaska (entity 006)
    if lat >= 51.0 && lat <= 72.0 && lon >= -180.0 && lon <= -130.0 {
        return Some("006"); // Alaska
    }
    
    // Hawaii (entity 110)
    if lat >= 18.5 && lat <= 23.0 && lon >= -161.0 && lon <= -154.0 {
        return Some("110"); // Hawaii
    }
    
    // Puerto Rico (entity 202)
    if lat >= 17.5 && lat <= 18.6 && lon >= -67.5 && lon <= -65.2 {
        return Some("202"); // Puerto Rico
    }
    
    // US Virgin Islands (entity 285)
    if lat >= 17.5 && lat <= 18.5 && lon >= -65.2 && lon <= -64.5 {
        return Some("285"); // US Virgin Islands
    }
    
    // =========================================================================
    // CANADA (entity 001)
    // =========================================================================
    if lat >= 41.5 && lat <= 84.0 && lon >= -141.0 && lon <= -52.0 {
        // Exclude areas that are US
        if !(lat >= 24.5 && lat <= 49.5 && lon >= -125.0 && lon <= -66.5) {
            return Some("001"); // Canada
        }
    }
    
    // =========================================================================
    // CARIBBEAN
    // =========================================================================
    
    // Cuba (entity 070)
    if lat >= 19.5 && lat <= 23.5 && lon >= -85.0 && lon <= -74.0 {
        // Exclude Guantanamo
        if !(lat >= 19.5 && lat <= 20.5 && lon >= -75.5 && lon <= -74.5) {
            return Some("070"); // Cuba
        }
    }
    
    // Jamaica (entity 082)
    if lat >= 17.5 && lat <= 18.6 && lon >= -78.5 && lon <= -76.0 {
        return Some("082"); // Jamaica
    }
    
    // Dominican Republic (entity 072)
    if lat >= 17.5 && lat <= 20.0 && lon >= -72.0 && lon <= -68.0 {
        return Some("072"); // Dominican Republic
    }
    
    // Haiti (entity 078)
    if lat >= 18.0 && lat <= 20.0 && lon >= -74.5 && lon <= -71.5 {
        return Some("078"); // Haiti
    }
    
    // Bahamas (entity 060)
    if lat >= 20.5 && lat <= 27.5 && lon >= -80.5 && lon <= -72.5 {
        return Some("060"); // Bahamas
    }
    
    // =========================================================================
    // MEXICO & CENTRAL AMERICA
    // =========================================================================
    
    // Mexico (entity 050)
    if lat >= 14.0 && lat <= 33.0 && lon >= -118.0 && lon <= -86.0 {
        // Exclude US
        if !(lat >= 24.5 && lat <= 49.5 && lon >= -125.0 && lon <= -66.5) {
            return Some("050"); // Mexico
        }
    }
    
    // =========================================================================
    // SOUTH AMERICA
    // =========================================================================
    
    // Brazil (entity 108)
    if lat >= -34.0 && lat <= 5.5 && lon >= -74.0 && lon <= -34.0 {
        return Some("108"); // Brazil
    }
    
    // Argentina (entity 100)
    if lat >= -55.0 && lat <= -21.0 && lon >= -74.0 && lon <= -53.0 {
        return Some("100"); // Argentina
    }
    
    // Chile (entity 112)
    if lat >= -56.0 && lat <= -17.5 && lon >= -76.0 && lon <= -66.5 {
        return Some("112"); // Chile
    }
    
    // =========================================================================
    // EUROPE
    // =========================================================================
    
    // United Kingdom (entity 223) - England
    if lat >= 49.5 && lat <= 55.5 && lon >= -6.0 && lon <= 2.0 {
        return Some("223"); // England
    }
    
    // Germany (entity 230)
    if lat >= 47.0 && lat <= 55.5 && lon >= 5.5 && lon <= 15.5 {
        return Some("230"); // Germany
    }
    
    // France (entity 227)
    if lat >= 42.0 && lat <= 51.5 && lon >= -5.0 && lon <= 8.5 {
        return Some("227"); // France
    }
    
    // Spain (entity 281)
    if lat >= 36.0 && lat <= 43.8 && lon >= -9.5 && lon <= 3.5 {
        return Some("281"); // Spain
    }
    
    // Italy (entity 248)
    if lat >= 36.5 && lat <= 47.5 && lon >= 6.5 && lon <= 18.5 {
        return Some("248"); // Italy
    }
    
    // Poland (entity 269)
    if lat >= 49.0 && lat <= 55.0 && lon >= 14.0 && lon <= 24.5 {
        return Some("269"); // Poland
    }
    
    // Netherlands (entity 263)
    if lat >= 50.5 && lat <= 53.7 && lon >= 3.3 && lon <= 7.3 {
        return Some("263"); // Netherlands
    }
    
    // Belgium (entity 209)
    if lat >= 49.5 && lat <= 51.5 && lon >= 2.5 && lon <= 6.5 {
        return Some("209"); // Belgium
    }
    
    // European Russia (entity 054)
    if lat >= 41.0 && lat <= 82.0 && lon >= 19.0 && lon <= 60.0 {
        return Some("054"); // European Russia
    }
    
    // =========================================================================
    // ASIA
    // =========================================================================
    
    // Japan (entity 339)
    if lat >= 24.0 && lat <= 46.0 && lon >= 122.0 && lon <= 154.0 {
        return Some("339"); // Japan
    }
    
    // China (entity 318)
    if lat >= 18.0 && lat <= 54.0 && lon >= 73.0 && lon <= 135.0 {
        return Some("318"); // China
    }
    
    // India (entity 324)
    if lat >= 6.0 && lat <= 36.0 && lon >= 68.0 && lon <= 97.0 {
        return Some("324"); // India
    }
    
    // Asiatic Russia (entity 015)
    if lat >= 41.0 && lat <= 82.0 && lon >= 60.0 && lon <= 180.0 {
        return Some("015"); // Asiatic Russia
    }
    
    // =========================================================================
    // OCEANIA
    // =========================================================================
    
    // Australia (entity 150)
    if lat >= -44.0 && lat <= -10.0 && lon >= 113.0 && lon <= 154.0 {
        return Some("150"); // Australia
    }
    
    // New Zealand (entity 170)
    if lat >= -47.5 && lat <= -34.0 && lon >= 166.0 && lon <= 179.0 {
        return Some("170"); // New Zealand
    }
    
    // Indonesia (entity 327)
    if lat >= -11.0 && lat <= 6.0 && lon >= 95.0 && lon <= 141.0 {
        return Some("327"); // Indonesia
    }
    
    // =========================================================================
    // AFRICA
    // =========================================================================
    
    // South Africa (entity 462)
    if lat >= -35.0 && lat <= -22.0 && lon >= 16.0 && lon <= 33.0 {
        return Some("462"); // South Africa
    }
    
    // =========================================================================
    // ANTARCTICA (entity 013)
    // =========================================================================
    if lat < -60.0 {
        return Some("013"); // Antarctica
    }
    
    // No match - could be international waters or unmapped territory
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_to_latlon_4char() {
        // FN42 - around Massachusetts/Vermont area
        // FN: F=5, N=13 -> base: -80°, 40°
        // 42: 4*2°=-8° from base, 2*1°=+2° -> center: -72° to -70°, 42° to 43°
        let result = grid_to_latlon("FN42");
        assert!(result.is_some());
        let (lat, lon) = result.unwrap();
        assert!(lat > 42.0 && lat < 43.0);
        assert!(lon > -72.0 && lon < -70.0);
    }

    #[test]
    fn test_grid_to_latlon_6char() {
        // FN42ab - more precise location
        let result = grid_to_latlon("FN42ab");
        assert!(result.is_some());
    }

    #[test]
    fn test_guantanamo_vs_alabama() {
        // FK29 is Guantanamo Bay
        let fk29 = lookup_grid("FK29");
        assert_eq!(fk29.dxcc, Some("105".to_string())); // Guantanamo Bay
        
        // EM62 is Alabama (where KG4BHR actually is) - DXCC is USA
        let em62 = lookup_grid("EM62");
        assert_eq!(em62.dxcc, Some("291".to_string())); // USA
        assert_eq!(em62.country.as_deref(), Some("United States of America"));
        // NOTE: State is NOT populated from grid - use FCC database for state lookup
        assert!(em62.state.is_none());
    }

    #[test]
    fn test_continental_us() {
        // Various continental US grids
        assert_eq!(lookup_grid("FN42").dxcc, Some("291".to_string())); // Massachusetts
        assert_eq!(lookup_grid("DM79").dxcc, Some("291".to_string())); // California
        assert_eq!(lookup_grid("EM12").dxcc, Some("291".to_string())); // Texas
    }

    #[test]
    fn test_alaska_hawaii() {
        // Alaska grids (BP, BO, etc.)
        let alaska = lookup_grid("BP51");
        assert_eq!(alaska.dxcc, Some("006".to_string())); // Alaska
        
        // Hawaii grids (BL, BK)
        let hawaii = lookup_grid("BL11");
        assert_eq!(hawaii.dxcc, Some("110".to_string())); // Hawaii
    }

    #[test]
    fn test_invalid_grid() {
        assert!(lookup_grid("").dxcc.is_none());
        assert!(lookup_grid("RR73").dxcc.is_none()); // FT8 message, not grid
        assert!(lookup_grid("XX").dxcc.is_none()); // Too short
    }

    #[test]
    fn test_japan() {
        // PM95 - Tokyo area
        let result = lookup_grid("PM95");
        assert_eq!(result.dxcc, Some("339".to_string())); // Japan
    }

    #[test]
    fn test_germany() {
        // JO62 - Berlin area
        let result = lookup_grid("JO62");
        assert_eq!(result.dxcc, Some("230".to_string())); // Germany
    }

    #[test]
    fn test_uk() {
        // IO91 - London area
        let result = lookup_grid("IO91");
        assert_eq!(result.dxcc, Some("223".to_string())); // England
    }
}
