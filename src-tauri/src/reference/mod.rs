// Reference data module - authoritative DXCC and prefix data
// Source: ARRL DXCC List and ITU Radio Regulations
// NOT dependent on CTY.DAT
//
// Data Population Strategy (per ADIF 3.1.4 and CLAUDE.md):
// - PRIMARY: Grid square lookup provides: DXCC, COUNTRY, CONT (where they ARE)
// - FALLBACK: Callsign prefix lookup provides: DXCC, COUNTRY, CQZ, ITUZ, CONT
// - For US STATE (WAS award): Use FCC database lookup by callsign
//   Grid-to-state is unreliable due to irregular state boundaries.
//   Industry standard (CQRLOG, LoTW) uses callsign→state mapping.
// - Non-US stations: STATE is left blank (not applicable)
//
// Location Lookup Priority:
// 1. Grid square (if valid) → Grid-based DXCC lookup
// 2. Callsign prefix → Prefix-based DXCC lookup (fallback only)
// 3. For US STATE → FCC database lookup by callsign
//
// Special Cases (Grid-Based Override):
// - KG4 + Guantanamo grids (FK19-FK39, FL19-FL30) → Guantanamo Bay (105)
// - KG4 + any other grid OR no grid → USA (291)
//   Rationale: FCC database shows ALL 16,499 KG4 callsigns are US hams

pub mod dxcc;
pub mod grid_location;
pub mod prefixes;
pub mod states;

use std::collections::HashMap;
use std::sync::OnceLock;
use dxcc::{DxccEntity, DXCC_ENTITIES};
use prefixes::PREFIX_RULES;

// Re-export grid location types
pub use grid_location::lookup_grid;

/// Grid squares covering Guantanamo Bay Naval Base area
/// Ref: https://www.karhukoti.com/maidenhead-grid-square-locator/?grid=FK29
const GUANTANAMO_GRIDS: &[&str] = &[
    "FK19", "FK28", "FK29", "FK39",
    "FL19", "FL20", "FL29", "FL30",
];

/// Check if a grid square is in the Guantanamo Bay area
fn is_guantanamo_grid(grid: &str) -> bool {
    let grid_upper = grid.to_uppercase();
    // Check 4-char prefix of grid
    if grid_upper.len() >= 4 {
        let grid_4 = &grid_upper[..4];
        return GUANTANAMO_GRIDS.contains(&grid_4);
    }
    false
}

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

/// Complete callsign lookup result
/// Per ADIF 3.1.4: These fields can be derived from callsign prefix
/// Note: STATE is explicitly NOT included - it comes from LoTW confirmation
/// dxcc uses ARRL 3-digit string format (e.g., "001" for Canada)
#[derive(Debug, Clone, Default)]
pub struct CallsignLookup {
    pub dxcc: Option<String>,  // ARRL 3-digit format: "001", "291", etc.
    pub country: Option<String>,
    pub continent: Option<String>,
    pub cqz: Option<i32>,
    pub ituz: Option<i32>,
}

impl CallsignLookup {
    /// Get DXCC entity ID as integer for database/ADIF compatibility
    /// Converts from ARRL 3-digit string format ("001") to integer (1)
    pub fn dxcc_as_i32(&self) -> Option<i32> {
        self.dxcc.as_ref().and_then(|s| s.parse::<i32>().ok())
    }
}

/// PRIMARY lookup: Use grid first, fall back to prefix
/// 
/// This is the main entry point for location lookup.
/// Grid-based lookup is preferred because:
/// - Grid represents WHERE THE STATION IS (not where they live)
/// - Solves the KG4 problem (KG4BHR in Alabama vs Guantanamo)
/// - Handles portable operators correctly
/// 
/// Special case handling:
/// - KG4 + Guantanamo grid (FK/FL area) → Guantanamo Bay (105)
/// - KG4 + other grid or no grid → USA (291)
/// 
/// Fallback to prefix when:
/// - Grid is empty or invalid
/// - Grid-based lookup returns no entity (international waters, etc.)
/// 
/// Returns None for fields that cannot be determined.
/// STATE is NEVER populated here - use LoTW confirmation.
#[allow(dead_code)] // Grid-based lookup for future portable operator handling
pub fn lookup_location(call: &str, grid: &str) -> CallsignLookup {
    let call_upper = call.to_uppercase();
    let dxcc_map = get_dxcc_map();
    
    // Special case: KG4 callsigns need grid-based disambiguation
    // FCC shows all 16,499 KG4 are US hams, but actual Guantanamo ops
    // would send a grid in the FK29/FL20 area
    if call_upper.starts_with("KG4") {
        if !grid.is_empty() && is_guantanamo_grid(grid) {
            // KG4 + Guantanamo grid = Guantanamo Bay (105)
            if let Some(entity) = dxcc_map.get("105") {
                log::info!("KG4 {} with Guantanamo grid {} → entity 105", call, grid);
                return CallsignLookup {
                    dxcc: Some("105".to_string()),
                    country: Some(entity.name.to_uppercase()),
                    continent: Some(entity.continent.to_string()),
                    cqz: entity.cq_zones.first().map(|&z| z as i32),
                    ituz: entity.itu_zones.first().map(|&z| z as i32),
                };
            }
        }
        // KG4 + non-Guantanamo grid OR no grid = USA (291)
        // Fall through to normal prefix lookup (KG4 rule now returns 291)
    }
    
    // Try grid-based lookup first (PRIMARY)
    if !grid.is_empty() && grid.len() >= 4 {
        let grid_result = lookup_grid(grid);
        if grid_result.dxcc.is_some() {
            log::debug!("Location from grid {}: dxcc={:?} country={:?}", 
                       grid, grid_result.dxcc, grid_result.country);
            return CallsignLookup {
                dxcc: grid_result.dxcc,
                country: grid_result.country,
                continent: grid_result.continent,
                cqz: None,  // Grid doesn't give us zone info
                ituz: None,
            };
        }
    }
    
    // Fallback to prefix-based lookup
    if !call.is_empty() {
        let prefix_result = lookup_call_full(call);
        log::debug!("Location from prefix {}: dxcc={:?} country={:?}", 
                   call, prefix_result.dxcc, prefix_result.country);
        return prefix_result;
    }
    
    // Nothing to look up
    CallsignLookup::default()
}

/// Look up a callsign and return full entity information
/// Per our data population strategy (CLAUDE.md):
/// - Tier 1 (At QSO time): Callsign prefix → DXCC, COUNTRY, CQZ, ITUZ, CONT
/// - STATE is NOT derived from prefix (portable ops may be elsewhere)
/// 
/// Handles compound callsigns like:
/// - "HK0/DF3TJ" -> HK0 prefix (San Andrés)
/// - "W1AW/KH6" -> KH6 suffix (Hawaii)  
/// - "W1AW/P" or "W1AW/M" -> base call W1AW (portable/mobile markers ignored)
/// 
/// Uses O(1) HashMap lookup for DXCC entity after prefix match.
pub fn lookup_call_full(call: &str) -> CallsignLookup {
    let call_upper = call.to_uppercase();
    let dxcc_map = get_dxcc_map();
    
    // Handle compound callsigns with /
    let lookup_call = if call_upper.contains('/') {
        extract_dxcc_portion(&call_upper)
    } else {
        call_upper.clone()
    };
    
    // Find the best matching prefix rule
    let mut best_match: Option<&prefixes::PrefixRule> = None;
    let mut best_priority = 0u8;
    let mut best_len = 0usize;
    
    for rule in PREFIX_RULES {
        if rule.exact {
            if lookup_call == rule.prefix {
                best_match = Some(rule);
                break;
            }
        } else if lookup_call.starts_with(rule.prefix) {
            let len = rule.prefix.len();
            if len > best_len || (len == best_len && rule.priority > best_priority) {
                best_match = Some(rule);
                best_priority = rule.priority;
                best_len = len;
            }
        }
    }
    
    // If we found a prefix match, look up the full DXCC entity via HashMap (O(1))
    if let Some(rule) = best_match {
        if let Some(entity) = dxcc_map.get(rule.entity_id) {
            // Use first zone from arrays
            let cqz = entity.cq_zones.first().copied().unwrap_or(0) as i32;
            let ituz = entity.itu_zones.first().copied().unwrap_or(0) as i32;
            return CallsignLookup {
                dxcc: Some(entity.entity_id.to_string()),
                country: Some(entity.name.to_uppercase()),
                continent: Some(entity.continent.to_string()),
                cqz: Some(cqz),
                ituz: Some(ituz),
            };
        }
        // Entity ID found but no entity data (shouldn't happen)
        return CallsignLookup {
            dxcc: Some(rule.entity_id.to_string()),
            ..Default::default()
        };
    }
    
    CallsignLookup::default()
}

/// Extract the DXCC-determining portion of a compound callsign
/// Rules:
/// - Single char suffix like /P, /M, /A are ignored (portable, mobile, etc)
/// - Prefix/Call like HK0/DF3TJ -> use HK0 (prefix determines DXCC)
/// - Call/Suffix like W1AW/KH6 -> use KH6 (suffix determines DXCC)
fn extract_dxcc_portion(call: &str) -> String {
    let parts: Vec<&str> = call.split('/').collect();
    
    if parts.len() != 2 {
        // More than one slash or no slash, just use as-is
        return call.to_string();
    }
    
    let part0 = parts[0];
    let part1 = parts[1];
    
    // Single char suffixes are modifiers, not DXCC indicators
    // /P = portable, /M = mobile, /A = alternative, /MM = maritime mobile, etc.
    if part1.len() <= 2 && part1.chars().all(|c| c.is_ascii_alphabetic()) {
        return part0.to_string();
    }
    
    // If suffix looks like a country prefix (short, starts with letter, has digit)
    // e.g., W1AW/KH6 -> KH6 is the DXCC
    if part1.len() <= 4 && part1.chars().any(|c| c.is_ascii_digit()) {
        return part1.to_string();
    }
    
    // If prefix is short and looks like a country prefix (e.g., HK0/DF3TJ)
    // The prefix determines DXCC
    if part0.len() <= 4 && part0.chars().any(|c| c.is_ascii_digit()) {
        return part0.to_string();
    }
    
    // Default: use the longer part as it's more likely the base callsign
    if part0.len() >= part1.len() {
        part0.to_string()
    } else {
        part1.to_string()
    }
}

// These accessor functions are for future features (award matrix, data export)
#[allow(dead_code)]
/// Get all DXCC entities
pub fn get_all_entities() -> &'static [dxcc::DxccEntity] {
    DXCC_ENTITIES
}

#[allow(dead_code)]
/// Get all prefix rules
pub fn get_all_prefixes() -> &'static [prefixes::PrefixRule] {
    PREFIX_RULES
}

#[allow(dead_code)]
/// Get all US states for WAS tracking
pub fn get_all_states() -> &'static [states::UsState] {
    states::US_STATES
}

// NOTE: grid_to_state() REMOVED (2026-01-13) - see states.rs for rationale

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compound_callsign_hk0() {
        // HK0/DF3TJ should resolve to San Andres & Providencia (entity 216)
        let result = lookup_call_full("HK0/DF3TJ");
        assert_eq!(result.dxcc.as_deref(), Some("216"));
        assert_eq!(result.country.as_deref(), Some("SAN ANDRES & PROVIDENCIA"));
        assert_eq!(result.continent.as_deref(), Some("NA"));
    }

    #[test]
    fn test_extract_dxcc_portion() {
        // HK0/DF3TJ -> HK0 (prefix determines DXCC)
        assert_eq!(extract_dxcc_portion("HK0/DF3TJ"), "HK0");
        // W1AW/KH6 -> KH6 (suffix determines DXCC)
        assert_eq!(extract_dxcc_portion("W1AW/KH6"), "KH6");
        // W1AW/P -> W1AW (portable modifier ignored)
        assert_eq!(extract_dxcc_portion("W1AW/P"), "W1AW");
        // W1AW/MM -> W1AW (maritime mobile ignored)
        assert_eq!(extract_dxcc_portion("W1AW/MM"), "W1AW");
    }

    /// Regression test for BUG-001: DXCC must convert to integer for database
    /// The root cause was binding Option<String> "291" to INTEGER column.
    /// This test ensures dxcc_as_i32() correctly converts the ARRL 3-digit
    /// string format to integers suitable for SQLite binding.
    #[test]
    fn test_dxcc_as_i32_conversion() {
        // US callsign - verify string "291" converts to integer 291
        let result = lookup_call_full("W1AW");
        assert_eq!(result.dxcc.as_deref(), Some("291"));
        assert_eq!(result.dxcc_as_i32(), Some(291));
        
        // Canada - verify leading zeros handled ("001" -> 1)
        let result = lookup_call_full("VE3ABC");
        assert_eq!(result.dxcc.as_deref(), Some("001"));
        assert_eq!(result.dxcc_as_i32(), Some(1));
        
        // Alaska - verify "006" -> 6
        let result = lookup_call_full("KL7ABC");
        assert_eq!(result.dxcc.as_deref(), Some("006"));
        assert_eq!(result.dxcc_as_i32(), Some(6));
        
        // Unknown callsign should return None for both
        let result = lookup_call_full("");
        assert_eq!(result.dxcc, None);
        assert_eq!(result.dxcc_as_i32(), None);
    }
    
    /// Test for BUG-003 (KG4BHR) and BUG-004 (9Y4DG)
    /// These callsigns must map to the correct DXCC entities.
    #[test]
    fn test_bug_004_9y4dg() {
        // BUG-004: 9Y4DG must be Trinidad (090), not St. Kitts (249)
        let result = lookup_call_full("9Y4DG");
        assert_eq!(result.dxcc.as_deref(), Some("090"), "9Y4DG should be Trinidad");
        assert_eq!(result.dxcc_as_i32(), Some(90));
        assert_eq!(result.country.as_deref(), Some("TRINIDAD & TOBAGO"));
    }
    
    #[test]
    fn test_bug_003_kg4bhr() {
        // BUG-003: KG4BHR must be USA (291), not Guantanamo (105)
        // Per FCC database: KG4BHR is Shawn Bolton in Albertville, Alabama
        // All 16,499 KG4 callsigns in FCC are registered to US addresses
        
        // Using lookup_call_full (prefix only) - KG4 now defaults to USA
        let result = lookup_call_full("KG4BHR");
        assert_eq!(result.dxcc.as_deref(), Some("291"), "KG4BHR should be USA");
        assert_eq!(result.dxcc_as_i32(), Some(291));
        assert_eq!(result.country.as_deref(), Some("UNITED STATES OF AMERICA"));
        
        // KG4A also defaults to USA without grid info
        // (The old suffix-based rule was incorrect per FCC data)
        let result = lookup_call_full("KG4A");
        assert_eq!(result.dxcc.as_deref(), Some("291"), "KG4A without grid should be USA");
    }
    
    #[test]
    fn test_kg4_grid_based_disambiguation() {
        // KG4 + US grid = USA (291)
        let result = lookup_location("KG4BHR", "EM64");  // Alabama area
        assert_eq!(result.dxcc.as_deref(), Some("291"), "KG4 + US grid = USA");
        
        // KG4 + Guantanamo grid = Guantanamo Bay (105)
        let result = lookup_location("KG4AA", "FK29");  // Guantanamo area
        assert_eq!(result.dxcc.as_deref(), Some("105"), "KG4 + FK29 = Guantanamo");
        
        let result = lookup_location("KG4A", "FL20");  // Guantanamo area  
        assert_eq!(result.dxcc.as_deref(), Some("105"), "KG4 + FL20 = Guantanamo");
        
        // KG4 + no grid = USA (291) - safe default
        let result = lookup_location("KG4XYZ", "");
        assert_eq!(result.dxcc.as_deref(), Some("291"), "KG4 + no grid = USA");
        
        // KG4 + invalid grid = USA (291)
        let result = lookup_location("KG4ABC", "RR73");  // Invalid grid (FT8 message)
        assert_eq!(result.dxcc.as_deref(), Some("291"), "KG4 + invalid grid = USA");
    }
}