// Reference data module - authoritative DXCC and prefix data
// Source: ARRL DXCC List and ITU Radio Regulations
// NOT dependent on CTY.DAT

pub mod dxcc;
pub mod prefixes;
pub mod states;
use dxcc::DXCC_ENTITIES;
use prefixes::PREFIX_RULES;

/// Look up a callsign and return (DXCC entity ID, Country name)
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
pub fn grid_to_state(grid: &str) -> Option<(&'static str, &'static str)> {
    states::grid_to_state(grid)
}