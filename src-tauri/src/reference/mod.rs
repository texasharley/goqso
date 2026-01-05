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
use dxcc::DXCC_ENTITIES;
use prefixes::PREFIX_RULES;

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
pub fn lookup_call_full(call: &str) -> CallsignLookup {
    let call_upper = call.to_uppercase();
    
    // Find the best matching prefix rule
    let mut best_match: Option<&prefixes::PrefixRule> = None;
    let mut best_priority = 0u8;
    let mut best_len = 0usize;
    
    for rule in PREFIX_RULES {
        if rule.exact {
            if call_upper == rule.prefix {
                best_match = Some(rule);
                break;
            }
        } else if call_upper.starts_with(rule.prefix) {
            let len = rule.prefix.len();
            if len > best_len || (len == best_len && rule.priority > best_priority) {
                best_match = Some(rule);
                best_priority = rule.priority;
                best_len = len;
            }
        }
    }
    
    // If we found a prefix match, look up the full DXCC entity
    if let Some(rule) = best_match {
        for entity in DXCC_ENTITIES {
            if entity.entity_id == rule.entity_id {
                return CallsignLookup {
                    dxcc: Some(entity.entity_id as i32),
                    country: Some(entity.name.to_string()),
                    continent: Some(entity.continent.to_string()),
                    cqz: Some(entity.cq_zone as i32),
                    ituz: Some(entity.itu_zone as i32),
                };
            }
        }
        // Entity ID found but no entity data (shouldn't happen)
        return CallsignLookup {
            dxcc: Some(rule.entity_id as i32),
            ..Default::default()
        };
    }
    
    CallsignLookup::default()
}

/// Look up a callsign and return (DXCC entity ID, Country name)
/// Legacy function - prefer lookup_call_full for complete data
pub fn lookup_call(call: &str) -> (Option<i32>, Option<String>) {
    let call_upper = call.to_uppercase();
    
    // Find the best matching prefix rule
    let mut best_match: Option<&prefixes::PrefixRule> = None;
    let mut best_priority = 0u8;
    let mut best_len = 0usize;
    
    for rule in PREFIX_RULES {
        if rule.exact {
            if call_upper == rule.prefix {
                best_match = Some(rule);
                break;
            }
        } else if call_upper.starts_with(rule.prefix) {
            // Prefer longer prefix matches and higher priority
            let len = rule.prefix.len();
            if len > best_len || (len == best_len && rule.priority > best_priority) {
                best_match = Some(rule);
                best_priority = rule.priority;
                best_len = len;
            }
        }
    }
    
    // If we found a prefix match, look up the DXCC entity
    if let Some(rule) = best_match {
        let entity_id = rule.entity_id as i32;
        
        // Find the entity name
        for entity in DXCC_ENTITIES {
            if entity.entity_id == rule.entity_id {
                return (Some(entity_id), Some(entity.name.to_string()));
            }
        }
        
        // Entity ID found but no name (shouldn't happen)
        return (Some(entity_id), None);
    }
    
    (None, None)
}

/// Get continent for a DXCC entity
pub fn get_continent(dxcc: i32) -> Option<String> {
    for entity in DXCC_ENTITIES {
        if entity.entity_id == dxcc as u16 {
            return Some(entity.continent.to_string());
        }
    }
    None
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