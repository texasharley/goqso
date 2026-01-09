// Reference data module - authoritative DXCC and prefix data
// Source: ARRL DXCC List and ITU Radio Regulations
// NOT dependent on CTY.DAT
//
// Data Population Strategy (per ADIF 3.1.4 and CLAUDE.md):
// - Callsign prefix lookup provides: DXCC, COUNTRY, CQZ, ITUZ, CONT
// - STATE is NOT derived from prefix (portable operators may be elsewhere)
// - STATE comes from LoTW confirmations (authoritative for WAS award)

pub mod dxcc;
pub mod prefixes;
pub mod states;

use std::collections::HashMap;
use std::sync::OnceLock;
use dxcc::{DxccEntity, DXCC_ENTITIES};
use prefixes::PREFIX_RULES;

/// Lazily-initialized HashMap for O(1) DXCC entity lookup by entity_id
static DXCC_MAP: OnceLock<HashMap<u16, &'static DxccEntity>> = OnceLock::new();

/// Get or initialize the DXCC entity HashMap
fn get_dxcc_map() -> &'static HashMap<u16, &'static DxccEntity> {
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
#[derive(Debug, Clone, Default)]
pub struct CallsignLookup {
    pub dxcc: Option<i32>,
    pub country: Option<String>,
    pub continent: Option<String>,
    pub cqz: Option<i32>,
    pub ituz: Option<i32>,
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
        if let Some(entity) = dxcc_map.get(&rule.entity_id) {
            // Use first zone from arrays
            let cqz = entity.cq_zones.first().copied().unwrap_or(0) as i32;
            let ituz = entity.itu_zones.first().copied().unwrap_or(0) as i32;
            return CallsignLookup {
                dxcc: Some(entity.entity_id as i32),
                country: Some(entity.name.to_uppercase()),
                continent: Some(entity.continent.to_string()),
                cqz: Some(cqz),
                ituz: Some(ituz),
            };
        }
        // Entity ID found but no entity data (shouldn't happen)
        return CallsignLookup {
            dxcc: Some(rule.entity_id as i32),
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

/// Get all DXCC entities
pub fn get_all_entities() -> &'static [dxcc::DxccEntity] {
    DXCC_ENTITIES
}

/// Get all prefix rules
pub fn get_all_prefixes() -> &'static [prefixes::PrefixRule] {
    PREFIX_RULES
}

/// Get all US states for WAS tracking
pub fn get_all_states() -> &'static [states::UsState] {
    states::US_STATES
}

/// Get US state from Maidenhead grid square
/// 
/// **DEPRECATED**: Do not use for data population!
/// See documentation in states.rs for why grid→state conversion is unreliable.
/// STATE should come from LoTW confirmation (authoritative for WAS award).
#[deprecated(note = "Do not use for STATE field population - use LoTW confirmation instead")]
#[allow(deprecated)]
pub fn grid_to_state(grid: &str) -> Option<(&'static str, &'static str)> {
    states::grid_to_state(grid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compound_callsign_hk0() {
        // HK0/DF3TJ should resolve to San Andres & Providencia (entity 216)
        let result = lookup_call_full("HK0/DF3TJ");
        assert_eq!(result.dxcc, Some(216));
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
}