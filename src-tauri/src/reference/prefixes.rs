// Prefix to DXCC Entity mapping
// Source: ITU Radio Regulations and ARRL prefix assignments
// 
// This maps callsign prefixes to DXCC entities.
// ITU allocates prefix blocks internationally.
// ARRL assigns DXCC entity numbers.

use super::dxcc::DxccEntity;

/// A prefix rule for matching callsigns to DXCC entities
#[derive(Debug, Clone)]
pub struct PrefixRule {
    /// The prefix pattern (e.g., "W", "VE", "JA")
    pub prefix: &'static str,
    /// The DXCC entity ID this prefix maps to
    pub entity_id: u16,
    /// Optional: exact match only (for special calls)
    pub exact: bool,
    /// Priority for overlapping prefixes (higher = more specific)
    pub priority: u8,
}

/// Core prefix mappings from ITU allocations
/// These are the official prefix blocks allocated by ITU
pub const PREFIX_RULES: &[PrefixRule] = &[
    // =========================================================================
    // NORTH AMERICA
    // =========================================================================
    // United States (ITU: A, K, N, W)
    PrefixRule { prefix: "K", entity_id: 291, exact: false, priority: 10 },
    PrefixRule { prefix: "W", entity_id: 291, exact: false, priority: 10 },
    PrefixRule { prefix: "N", entity_id: 291, exact: false, priority: 10 },
    PrefixRule { prefix: "AA", entity_id: 291, exact: false, priority: 20 },
    PrefixRule { prefix: "AB", entity_id: 291, exact: false, priority: 20 },
    PrefixRule { prefix: "AC", entity_id: 291, exact: false, priority: 20 },
    PrefixRule { prefix: "AD", entity_id: 291, exact: false, priority: 20 },
    PrefixRule { prefix: "AE", entity_id: 291, exact: false, priority: 20 },
    PrefixRule { prefix: "AF", entity_id: 291, exact: false, priority: 20 },
    PrefixRule { prefix: "AG", entity_id: 291, exact: false, priority: 20 },
    PrefixRule { prefix: "AH0", entity_id: 166, exact: false, priority: 30 }, // Mariana Islands
    PrefixRule { prefix: "AH1", entity_id: 20, exact: false, priority: 30 },  // Baker & Howland
    PrefixRule { prefix: "AH2", entity_id: 103, exact: false, priority: 30 }, // Guam
    PrefixRule { prefix: "AH3", entity_id: 297, exact: false, priority: 30 }, // Johnston Island
    PrefixRule { prefix: "AH4", entity_id: 174, exact: false, priority: 30 }, // Midway
    PrefixRule { prefix: "AH5", entity_id: 138, exact: false, priority: 30 }, // Palmyra
    PrefixRule { prefix: "AH6", entity_id: 110, exact: false, priority: 30 }, // Hawaii
    PrefixRule { prefix: "AH7", entity_id: 110, exact: false, priority: 30 }, // Hawaii
    PrefixRule { prefix: "AH8", entity_id: 197, exact: false, priority: 30 }, // American Samoa
    PrefixRule { prefix: "AL", entity_id: 6, exact: false, priority: 30 },    // Alaska
    PrefixRule { prefix: "KL", entity_id: 6, exact: false, priority: 30 },    // Alaska
    PrefixRule { prefix: "NL", entity_id: 6, exact: false, priority: 30 },    // Alaska
    PrefixRule { prefix: "WL", entity_id: 6, exact: false, priority: 30 },    // Alaska
    PrefixRule { prefix: "KH6", entity_id: 110, exact: false, priority: 30 }, // Hawaii
    PrefixRule { prefix: "WH6", entity_id: 110, exact: false, priority: 30 }, // Hawaii
    PrefixRule { prefix: "NH6", entity_id: 110, exact: false, priority: 30 }, // Hawaii
    PrefixRule { prefix: "KP1", entity_id: 182, exact: false, priority: 30 }, // Navassa
    PrefixRule { prefix: "KP2", entity_id: 285, exact: false, priority: 30 }, // US Virgin Islands
    PrefixRule { prefix: "KP3", entity_id: 202, exact: false, priority: 30 }, // Puerto Rico
    PrefixRule { prefix: "KP4", entity_id: 202, exact: false, priority: 30 }, // Puerto Rico
    PrefixRule { prefix: "NP3", entity_id: 202, exact: false, priority: 30 }, // Puerto Rico
    PrefixRule { prefix: "NP4", entity_id: 202, exact: false, priority: 30 }, // Puerto Rico
    PrefixRule { prefix: "WP3", entity_id: 202, exact: false, priority: 30 }, // Puerto Rico
    PrefixRule { prefix: "WP4", entity_id: 202, exact: false, priority: 30 }, // Puerto Rico
    PrefixRule { prefix: "KG4", entity_id: 105, exact: false, priority: 30 }, // Guantanamo Bay
    
    // British Caribbean (ITU: VP-VQ)
    PrefixRule { prefix: "VP2E", entity_id: 8, exact: false, priority: 40 },   // Anguilla
    PrefixRule { prefix: "VP2M", entity_id: 177, exact: false, priority: 40 }, // Montserrat
    PrefixRule { prefix: "VP2V", entity_id: 65, exact: false, priority: 40 },  // British Virgin Islands
    PrefixRule { prefix: "VP5", entity_id: 84, exact: false, priority: 30 },   // Turks & Caicos Islands
    PrefixRule { prefix: "VP9", entity_id: 51, exact: false, priority: 30 },   // Bermuda
    PrefixRule { prefix: "ZF", entity_id: 96, exact: false, priority: 30 },    // Cayman Islands
    
    // Caribbean Nations (ITU: V2-V8)
    PrefixRule { prefix: "V2", entity_id: 12, exact: false, priority: 20 },    // Antigua & Barbuda
    PrefixRule { prefix: "V3", entity_id: 66, exact: false, priority: 20 },    // Belize
    PrefixRule { prefix: "V4", entity_id: 246, exact: false, priority: 20 },   // St. Kitts & Nevis
    
    // Caribbean Nations (ITU: C6, 6Y, 8P, 9Y)
    PrefixRule { prefix: "C6", entity_id: 211, exact: false, priority: 20 },   // Bahamas
    PrefixRule { prefix: "6Y", entity_id: 82, exact: false, priority: 20 },    // Jamaica
    PrefixRule { prefix: "8P", entity_id: 62, exact: false, priority: 20 },    // Barbados
    PrefixRule { prefix: "9Y", entity_id: 249, exact: false, priority: 20 },   // Trinidad & Tobago
    PrefixRule { prefix: "9Z", entity_id: 249, exact: false, priority: 20 },   // Trinidad & Tobago
    
    // Caribbean Nations (ITU: J3, J6, J7, J8)
    PrefixRule { prefix: "J3", entity_id: 97, exact: false, priority: 20 },    // Grenada
    PrefixRule { prefix: "J6", entity_id: 97, exact: false, priority: 20 },    // St. Lucia (ARRL entity 097)
    PrefixRule { prefix: "J7", entity_id: 95, exact: false, priority: 20 },    // Dominica
    PrefixRule { prefix: "J8", entity_id: 98, exact: false, priority: 20 },    // St. Vincent
    
    // Caribbean - Cuba, Dominican Republic, Haiti (ITU: CL-CM, CO, HI, HH)
    PrefixRule { prefix: "CM", entity_id: 70, exact: false, priority: 20 },    // Cuba
    PrefixRule { prefix: "CL", entity_id: 70, exact: false, priority: 20 },    // Cuba
    PrefixRule { prefix: "CO", entity_id: 70, exact: false, priority: 20 },    // Cuba
    PrefixRule { prefix: "T4", entity_id: 70, exact: false, priority: 20 },    // Cuba
    PrefixRule { prefix: "HI", entity_id: 72, exact: false, priority: 20 },    // Dominican Republic
    PrefixRule { prefix: "HH", entity_id: 78, exact: false, priority: 20 },    // Haiti
    PrefixRule { prefix: "4V", entity_id: 78, exact: false, priority: 20 },    // Haiti
    
    // French Caribbean (ITU: F - French overseas)
    PrefixRule { prefix: "FM", entity_id: 64, exact: false, priority: 30 },    // Martinique
    PrefixRule { prefix: "FG", entity_id: 79, exact: false, priority: 30 },    // Guadeloupe
    PrefixRule { prefix: "FS", entity_id: 213, exact: false, priority: 30 },   // French St. Martin
    PrefixRule { prefix: "FJ", entity_id: 214, exact: false, priority: 30 },   // St. Barthelemy
    PrefixRule { prefix: "FP", entity_id: 277, exact: false, priority: 30 },   // St. Pierre & Miquelon
    
    // Netherlands Caribbean (ITU: PJ)
    PrefixRule { prefix: "PJ2", entity_id: 517, exact: false, priority: 30 },  // Curacao
    PrefixRule { prefix: "PJ4", entity_id: 52, exact: false, priority: 30 },   // Bonaire
    PrefixRule { prefix: "PJ5", entity_id: 516, exact: false, priority: 30 },  // Saba & St. Eustatius
    PrefixRule { prefix: "PJ6", entity_id: 516, exact: false, priority: 30 },  // Saba & St. Eustatius
    PrefixRule { prefix: "PJ7", entity_id: 520, exact: false, priority: 30 },  // Sint Maarten
    PrefixRule { prefix: "P4", entity_id: 9, exact: false, priority: 20 },     // Aruba
    
    // Central America (ITU: TI, TG, HR/HQ, YN/HT, YS/HU, HP)
    PrefixRule { prefix: "TI", entity_id: 308, exact: false, priority: 20 },   // Costa Rica
    PrefixRule { prefix: "TE", entity_id: 308, exact: false, priority: 20 },   // Costa Rica
    PrefixRule { prefix: "TI9", entity_id: 37, exact: false, priority: 30 },   // Cocos Island
    PrefixRule { prefix: "TG", entity_id: 76, exact: false, priority: 20 },    // Guatemala (ARRL entity 076)
    PrefixRule { prefix: "TD", entity_id: 76, exact: false, priority: 20 },    // Guatemala (ARRL entity 076)
    PrefixRule { prefix: "HR", entity_id: 80, exact: false, priority: 20 },    // Honduras
    PrefixRule { prefix: "HQ", entity_id: 80, exact: false, priority: 20 },    // Honduras
    PrefixRule { prefix: "YN", entity_id: 86, exact: false, priority: 20 },    // Nicaragua
    PrefixRule { prefix: "HT", entity_id: 86, exact: false, priority: 20 },    // Nicaragua
    PrefixRule { prefix: "H6", entity_id: 86, exact: false, priority: 20 },    // Nicaragua
    PrefixRule { prefix: "H7", entity_id: 86, exact: false, priority: 20 },    // Nicaragua
    PrefixRule { prefix: "YS", entity_id: 74, exact: false, priority: 20 },    // El Salvador (ARRL entity 074)
    PrefixRule { prefix: "HU", entity_id: 74, exact: false, priority: 20 },    // El Salvador (ARRL entity 074)
    PrefixRule { prefix: "HP", entity_id: 88, exact: false, priority: 20 },    // Panama
    PrefixRule { prefix: "HO", entity_id: 88, exact: false, priority: 20 },    // Panama
    PrefixRule { prefix: "H3", entity_id: 88, exact: false, priority: 20 },    // Panama
    PrefixRule { prefix: "H8", entity_id: 88, exact: false, priority: 20 },    // Panama
    PrefixRule { prefix: "H9", entity_id: 88, exact: false, priority: 20 },    // Panama
    PrefixRule { prefix: "3E", entity_id: 88, exact: false, priority: 20 },    // Panama
    PrefixRule { prefix: "3F", entity_id: 88, exact: false, priority: 20 },    // Panama
    
    // South America (ITU: various)
    PrefixRule { prefix: "YV", entity_id: 148, exact: false, priority: 20 },   // Venezuela
    PrefixRule { prefix: "YW", entity_id: 148, exact: false, priority: 20 },   // Venezuela
    PrefixRule { prefix: "YX", entity_id: 148, exact: false, priority: 20 },   // Venezuela
    PrefixRule { prefix: "YY", entity_id: 148, exact: false, priority: 20 },   // Venezuela
    PrefixRule { prefix: "4M", entity_id: 148, exact: false, priority: 20 },   // Venezuela
    PrefixRule { prefix: "YV0", entity_id: 17, exact: false, priority: 30 },   // Aves Island
    PrefixRule { prefix: "HK", entity_id: 116, exact: false, priority: 20 },   // Colombia
    PrefixRule { prefix: "HJ", entity_id: 116, exact: false, priority: 20 },   // Colombia
    PrefixRule { prefix: "5J", entity_id: 116, exact: false, priority: 20 },   // Colombia
    PrefixRule { prefix: "5K", entity_id: 116, exact: false, priority: 20 },   // Colombia
    PrefixRule { prefix: "HK0", entity_id: 216, exact: false, priority: 30 },  // San Andres & Providencia
    PrefixRule { prefix: "HK0M", entity_id: 161, exact: false, priority: 40 }, // Malpelo Island
    PrefixRule { prefix: "HC", entity_id: 120, exact: false, priority: 20 },   // Ecuador
    PrefixRule { prefix: "HD", entity_id: 120, exact: false, priority: 20 },   // Ecuador
    PrefixRule { prefix: "HC8", entity_id: 71, exact: false, priority: 30 },   // Galapagos Islands
    PrefixRule { prefix: "HD8", entity_id: 71, exact: false, priority: 30 },   // Galapagos Islands
    PrefixRule { prefix: "OA", entity_id: 136, exact: false, priority: 20 },   // Peru
    PrefixRule { prefix: "OB", entity_id: 136, exact: false, priority: 20 },   // Peru
    PrefixRule { prefix: "OC", entity_id: 136, exact: false, priority: 20 },   // Peru
    PrefixRule { prefix: "4T", entity_id: 136, exact: false, priority: 20 },   // Peru
    PrefixRule { prefix: "CP", entity_id: 117, exact: false, priority: 20 },   // Bolivia
    PrefixRule { prefix: "CE", entity_id: 108, exact: false, priority: 20 },   // Chile
    PrefixRule { prefix: "CA", entity_id: 108, exact: false, priority: 20 },   // Chile
    PrefixRule { prefix: "CB", entity_id: 108, exact: false, priority: 20 },   // Chile
    PrefixRule { prefix: "CC", entity_id: 108, exact: false, priority: 20 },   // Chile
    PrefixRule { prefix: "CD", entity_id: 108, exact: false, priority: 20 },   // Chile
    PrefixRule { prefix: "XQ", entity_id: 108, exact: false, priority: 20 },   // Chile
    PrefixRule { prefix: "XR", entity_id: 108, exact: false, priority: 20 },   // Chile
    PrefixRule { prefix: "3G", entity_id: 108, exact: false, priority: 20 },   // Chile
    PrefixRule { prefix: "CE0Y", entity_id: 47, exact: false, priority: 40 },  // Easter Island
    PrefixRule { prefix: "CE0X", entity_id: 125, exact: false, priority: 40 }, // San Felix & San Ambrosio
    PrefixRule { prefix: "CE0Z", entity_id: 126, exact: false, priority: 40 }, // Juan Fernandez
    PrefixRule { prefix: "CE0", entity_id: 47, exact: false, priority: 30 },   // Easter Island (default)
    PrefixRule { prefix: "ZP", entity_id: 132, exact: false, priority: 20 },   // Paraguay
    PrefixRule { prefix: "CX", entity_id: 144, exact: false, priority: 20 },   // Uruguay
    PrefixRule { prefix: "CV", entity_id: 144, exact: false, priority: 20 },   // Uruguay
    PrefixRule { prefix: "CW", entity_id: 144, exact: false, priority: 20 },   // Uruguay
    PrefixRule { prefix: "PZ", entity_id: 140, exact: false, priority: 20 },   // Suriname
    PrefixRule { prefix: "8R", entity_id: 129, exact: false, priority: 20 },   // Guyana
    PrefixRule { prefix: "FY", entity_id: 76, exact: false, priority: 30 },    // French Guiana
    
    // Falklands & South Atlantic (ITU: VP8)
    PrefixRule { prefix: "VP8", entity_id: 56, exact: false, priority: 30 },   // Falkland Islands (default)
    PrefixRule { prefix: "VP8/G", entity_id: 241, exact: false, priority: 40 }, // South Georgia
    PrefixRule { prefix: "VP8/S", entity_id: 240, exact: false, priority: 40 }, // South Sandwich Islands
    PrefixRule { prefix: "VP8/O", entity_id: 235, exact: false, priority: 40 }, // South Orkney Islands
    PrefixRule { prefix: "VP8/H", entity_id: 239, exact: false, priority: 40 }, // South Shetland Islands
    
    // Canada (ITU: CF-CK, CY-CZ, VA-VG, VO, VX-VY, XJ-XO)
    PrefixRule { prefix: "VE", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "VA", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "VB", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "VC", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "VD", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "VG", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "VO", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "VX", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "VY", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "CF", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "CG", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "CH", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "CI", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "CJ", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "CK", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "CY", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "CZ", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "XJ", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "XK", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "XL", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "XM", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "XN", entity_id: 1, exact: false, priority: 10 },
    PrefixRule { prefix: "XO", entity_id: 1, exact: false, priority: 10 },
    
    // Mexico (ITU: 4A-4C, 6D-6J, XA-XI)
    PrefixRule { prefix: "XE", entity_id: 50, exact: false, priority: 10 },
    PrefixRule { prefix: "XF", entity_id: 50, exact: false, priority: 10 },
    PrefixRule { prefix: "4A", entity_id: 50, exact: false, priority: 10 },
    PrefixRule { prefix: "4B", entity_id: 50, exact: false, priority: 10 },
    PrefixRule { prefix: "4C", entity_id: 50, exact: false, priority: 10 },
    PrefixRule { prefix: "6D", entity_id: 50, exact: false, priority: 10 },
    PrefixRule { prefix: "6E", entity_id: 50, exact: false, priority: 10 },
    PrefixRule { prefix: "6F", entity_id: 50, exact: false, priority: 10 },
    PrefixRule { prefix: "6G", entity_id: 50, exact: false, priority: 10 },
    PrefixRule { prefix: "6H", entity_id: 50, exact: false, priority: 10 },
    PrefixRule { prefix: "6I", entity_id: 50, exact: false, priority: 10 },
    PrefixRule { prefix: "6J", entity_id: 50, exact: false, priority: 10 },
    
    // =========================================================================
    // EUROPE
    // =========================================================================
    // United Kingdom (ITU: G, M, 2E)
    PrefixRule { prefix: "G", entity_id: 223, exact: false, priority: 10 },   // England
    PrefixRule { prefix: "M", entity_id: 223, exact: false, priority: 10 },   // England
    PrefixRule { prefix: "2E", entity_id: 223, exact: false, priority: 10 },  // England
    PrefixRule { prefix: "GM", entity_id: 265, exact: false, priority: 20 },  // Scotland
    PrefixRule { prefix: "MM", entity_id: 265, exact: false, priority: 20 },  // Scotland
    PrefixRule { prefix: "2M", entity_id: 265, exact: false, priority: 20 },  // Scotland
    PrefixRule { prefix: "GW", entity_id: 294, exact: false, priority: 20 },  // Wales
    PrefixRule { prefix: "MW", entity_id: 294, exact: false, priority: 20 },  // Wales
    PrefixRule { prefix: "2W", entity_id: 294, exact: false, priority: 20 },  // Wales
    PrefixRule { prefix: "GI", entity_id: 279, exact: false, priority: 20 },  // Northern Ireland
    PrefixRule { prefix: "MI", entity_id: 279, exact: false, priority: 20 },  // Northern Ireland
    PrefixRule { prefix: "GD", entity_id: 114, exact: false, priority: 20 },  // Isle of Man
    PrefixRule { prefix: "MD", entity_id: 114, exact: false, priority: 20 },  // Isle of Man
    PrefixRule { prefix: "GJ", entity_id: 106, exact: false, priority: 20 },  // Jersey
    PrefixRule { prefix: "MJ", entity_id: 106, exact: false, priority: 20 },  // Jersey
    PrefixRule { prefix: "GU", entity_id: 104, exact: false, priority: 20 },  // Guernsey
    PrefixRule { prefix: "MU", entity_id: 104, exact: false, priority: 20 },  // Guernsey
    
    // Germany (ITU: DA-DR)
    PrefixRule { prefix: "DA", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DB", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DC", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DD", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DE", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DF", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DG", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DH", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DI", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DJ", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DK", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DL", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DM", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DN", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DO", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DP", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DQ", entity_id: 230, exact: false, priority: 10 },
    PrefixRule { prefix: "DR", entity_id: 230, exact: false, priority: 10 },
    
    // France (ITU: F)
    PrefixRule { prefix: "F", entity_id: 227, exact: false, priority: 10 },
    
    // Italy (ITU: I)
    PrefixRule { prefix: "I", entity_id: 236, exact: false, priority: 10 },
    
    // Spain (ITU: EA-EH)
    PrefixRule { prefix: "EA", entity_id: 281, exact: false, priority: 10 },
    PrefixRule { prefix: "EB", entity_id: 281, exact: false, priority: 10 },
    PrefixRule { prefix: "EC", entity_id: 281, exact: false, priority: 10 },
    PrefixRule { prefix: "ED", entity_id: 281, exact: false, priority: 10 },
    PrefixRule { prefix: "EE", entity_id: 281, exact: false, priority: 10 },
    PrefixRule { prefix: "EF", entity_id: 281, exact: false, priority: 10 },
    PrefixRule { prefix: "EG", entity_id: 281, exact: false, priority: 10 },
    PrefixRule { prefix: "EH", entity_id: 281, exact: false, priority: 10 },
    PrefixRule { prefix: "EA6", entity_id: 21, exact: false, priority: 30 },  // Balearic Islands (counted as Spain for DXCC)
    PrefixRule { prefix: "EA8", entity_id: 29, exact: false, priority: 30 },  // Canary Islands
    PrefixRule { prefix: "EA9", entity_id: 256, exact: false, priority: 30 }, // Ceuta & Melilla
    
    // Portugal (ITU: CR-CS, CT)
    PrefixRule { prefix: "CT", entity_id: 272, exact: false, priority: 10 },
    PrefixRule { prefix: "CT3", entity_id: 149, exact: false, priority: 30 }, // Madeira
    PrefixRule { prefix: "CU", entity_id: 21, exact: false, priority: 30 },   // Azores
    
    // Netherlands (ITU: PA-PI)
    PrefixRule { prefix: "PA", entity_id: 263, exact: false, priority: 10 },
    PrefixRule { prefix: "PB", entity_id: 263, exact: false, priority: 10 },
    PrefixRule { prefix: "PC", entity_id: 263, exact: false, priority: 10 },
    PrefixRule { prefix: "PD", entity_id: 263, exact: false, priority: 10 },
    PrefixRule { prefix: "PE", entity_id: 263, exact: false, priority: 10 },
    PrefixRule { prefix: "PF", entity_id: 263, exact: false, priority: 10 },
    PrefixRule { prefix: "PG", entity_id: 263, exact: false, priority: 10 },
    PrefixRule { prefix: "PH", entity_id: 263, exact: false, priority: 10 },
    PrefixRule { prefix: "PI", entity_id: 263, exact: false, priority: 10 },
    
    // Belgium (ITU: ON-OT)
    PrefixRule { prefix: "ON", entity_id: 209, exact: false, priority: 10 },
    PrefixRule { prefix: "OO", entity_id: 209, exact: false, priority: 10 },
    PrefixRule { prefix: "OP", entity_id: 209, exact: false, priority: 10 },
    PrefixRule { prefix: "OQ", entity_id: 209, exact: false, priority: 10 },
    PrefixRule { prefix: "OR", entity_id: 209, exact: false, priority: 10 },
    PrefixRule { prefix: "OS", entity_id: 209, exact: false, priority: 10 },
    PrefixRule { prefix: "OT", entity_id: 209, exact: false, priority: 10 },
    
    // Switzerland (ITU: HB, HE)
    PrefixRule { prefix: "HB", entity_id: 287, exact: false, priority: 10 },
    PrefixRule { prefix: "HE", entity_id: 287, exact: false, priority: 10 },
    PrefixRule { prefix: "HB0", entity_id: 239, exact: false, priority: 30 }, // Liechtenstein
    
    // Austria (ITU: OE)
    PrefixRule { prefix: "OE", entity_id: 206, exact: false, priority: 10 },
    
    // Poland (ITU: SN-SR, 3Z)
    PrefixRule { prefix: "SP", entity_id: 269, exact: false, priority: 10 },
    PrefixRule { prefix: "SQ", entity_id: 269, exact: false, priority: 10 },
    PrefixRule { prefix: "SN", entity_id: 269, exact: false, priority: 10 },
    PrefixRule { prefix: "SO", entity_id: 269, exact: false, priority: 10 },
    PrefixRule { prefix: "SR", entity_id: 269, exact: false, priority: 10 },
    PrefixRule { prefix: "3Z", entity_id: 269, exact: false, priority: 10 },
    
    // Czech Republic (ITU: OK-OL)
    PrefixRule { prefix: "OK", entity_id: 503, exact: false, priority: 10 },
    PrefixRule { prefix: "OL", entity_id: 503, exact: false, priority: 10 },
    
    // Slovakia (ITU: OM)
    PrefixRule { prefix: "OM", entity_id: 504, exact: false, priority: 10 },
    
    // Hungary (ITU: HA, HG)
    PrefixRule { prefix: "HA", entity_id: 239, exact: false, priority: 10 },
    PrefixRule { prefix: "HG", entity_id: 239, exact: false, priority: 10 },
    
    // Romania (ITU: YO-YR)
    PrefixRule { prefix: "YO", entity_id: 275, exact: false, priority: 10 },
    PrefixRule { prefix: "YP", entity_id: 275, exact: false, priority: 10 },
    PrefixRule { prefix: "YQ", entity_id: 275, exact: false, priority: 10 },
    PrefixRule { prefix: "YR", entity_id: 275, exact: false, priority: 10 },
    
    // Bulgaria (ITU: LZ)
    PrefixRule { prefix: "LZ", entity_id: 212, exact: false, priority: 10 },
    
    // Greece (ITU: SV-SZ)
    PrefixRule { prefix: "SV", entity_id: 245, exact: false, priority: 10 },
    PrefixRule { prefix: "SW", entity_id: 245, exact: false, priority: 10 },
    PrefixRule { prefix: "SX", entity_id: 245, exact: false, priority: 10 },
    PrefixRule { prefix: "SY", entity_id: 245, exact: false, priority: 10 },
    PrefixRule { prefix: "SZ", entity_id: 245, exact: false, priority: 10 },
    PrefixRule { prefix: "SV5", entity_id: 40, exact: false, priority: 30 },  // Dodecanese
    PrefixRule { prefix: "SV9", entity_id: 180, exact: false, priority: 30 }, // Crete
    
    // Scandinavia
    PrefixRule { prefix: "OZ", entity_id: 222, exact: false, priority: 10 },  // Denmark
    PrefixRule { prefix: "LA", entity_id: 266, exact: false, priority: 10 },  // Norway
    PrefixRule { prefix: "LB", entity_id: 266, exact: false, priority: 10 },
    PrefixRule { prefix: "LC", entity_id: 266, exact: false, priority: 10 },
    PrefixRule { prefix: "LD", entity_id: 266, exact: false, priority: 10 },
    PrefixRule { prefix: "LE", entity_id: 266, exact: false, priority: 10 },
    PrefixRule { prefix: "LF", entity_id: 266, exact: false, priority: 10 },
    PrefixRule { prefix: "LG", entity_id: 266, exact: false, priority: 10 },
    PrefixRule { prefix: "LH", entity_id: 266, exact: false, priority: 10 },
    PrefixRule { prefix: "LI", entity_id: 266, exact: false, priority: 10 },
    PrefixRule { prefix: "LJ", entity_id: 266, exact: false, priority: 10 },
    PrefixRule { prefix: "LK", entity_id: 266, exact: false, priority: 10 },
    PrefixRule { prefix: "LL", entity_id: 266, exact: false, priority: 10 },
    PrefixRule { prefix: "LM", entity_id: 266, exact: false, priority: 10 },
    PrefixRule { prefix: "LN", entity_id: 266, exact: false, priority: 10 },
    PrefixRule { prefix: "SM", entity_id: 284, exact: false, priority: 10 },  // Sweden
    PrefixRule { prefix: "SA", entity_id: 284, exact: false, priority: 10 },
    PrefixRule { prefix: "SB", entity_id: 284, exact: false, priority: 10 },
    PrefixRule { prefix: "SC", entity_id: 284, exact: false, priority: 10 },
    PrefixRule { prefix: "SD", entity_id: 284, exact: false, priority: 10 },
    PrefixRule { prefix: "SE", entity_id: 284, exact: false, priority: 10 },
    PrefixRule { prefix: "SF", entity_id: 284, exact: false, priority: 10 },
    PrefixRule { prefix: "SG", entity_id: 284, exact: false, priority: 10 },
    PrefixRule { prefix: "SH", entity_id: 284, exact: false, priority: 10 },
    PrefixRule { prefix: "SI", entity_id: 284, exact: false, priority: 10 },
    PrefixRule { prefix: "SJ", entity_id: 284, exact: false, priority: 10 },
    PrefixRule { prefix: "SK", entity_id: 284, exact: false, priority: 10 },
    PrefixRule { prefix: "SL", entity_id: 284, exact: false, priority: 10 },
    PrefixRule { prefix: "OH", entity_id: 224, exact: false, priority: 10 },  // Finland
    PrefixRule { prefix: "OG", entity_id: 224, exact: false, priority: 10 },
    PrefixRule { prefix: "OF", entity_id: 224, exact: false, priority: 10 },
    PrefixRule { prefix: "OI", entity_id: 224, exact: false, priority: 10 },
    PrefixRule { prefix: "OJ", entity_id: 224, exact: false, priority: 10 },
    PrefixRule { prefix: "OH0", entity_id: 167, exact: false, priority: 30 }, // Aland Islands
    PrefixRule { prefix: "TF", entity_id: 118, exact: false, priority: 10 },  // Iceland
    PrefixRule { prefix: "OY", entity_id: 221, exact: false, priority: 10 },  // Faroe Islands
    PrefixRule { prefix: "OX", entity_id: 222, exact: false, priority: 10 },  // Greenland
    PrefixRule { prefix: "JW", entity_id: 5, exact: false, priority: 10 },    // Svalbard
    PrefixRule { prefix: "JX", entity_id: 259, exact: false, priority: 10 },  // Jan Mayen
    
    // Ireland (ITU: EI-EJ)
    PrefixRule { prefix: "EI", entity_id: 122, exact: false, priority: 10 },
    PrefixRule { prefix: "EJ", entity_id: 122, exact: false, priority: 10 },
    
    // Russia (ITU: R, UA-UI)
    PrefixRule { prefix: "R", entity_id: 54, exact: false, priority: 10 },    // European Russia (default)
    PrefixRule { prefix: "UA", entity_id: 54, exact: false, priority: 10 },
    PrefixRule { prefix: "UB", entity_id: 54, exact: false, priority: 10 },
    PrefixRule { prefix: "UC", entity_id: 54, exact: false, priority: 10 },
    PrefixRule { prefix: "UD", entity_id: 54, exact: false, priority: 10 },
    PrefixRule { prefix: "UE", entity_id: 54, exact: false, priority: 10 },
    PrefixRule { prefix: "UF", entity_id: 54, exact: false, priority: 10 },
    PrefixRule { prefix: "UG", entity_id: 54, exact: false, priority: 10 },
    PrefixRule { prefix: "UH", entity_id: 54, exact: false, priority: 10 },
    PrefixRule { prefix: "UI", entity_id: 54, exact: false, priority: 10 },
    // Asiatic Russia prefixes (UA0, UA9, etc.) - these need number-based rules
    PrefixRule { prefix: "UA0", entity_id: 15, exact: false, priority: 20 },  // Asiatic Russia
    PrefixRule { prefix: "UA9", entity_id: 15, exact: false, priority: 20 },  // Asiatic Russia
    PrefixRule { prefix: "R0", entity_id: 15, exact: false, priority: 20 },
    PrefixRule { prefix: "R9", entity_id: 15, exact: false, priority: 20 },
    
    // Ukraine (ITU: UR-UZ, EM-EO)
    PrefixRule { prefix: "UR", entity_id: 288, exact: false, priority: 10 },
    PrefixRule { prefix: "US", entity_id: 288, exact: false, priority: 10 },
    PrefixRule { prefix: "UT", entity_id: 288, exact: false, priority: 10 },
    PrefixRule { prefix: "UU", entity_id: 288, exact: false, priority: 10 },
    PrefixRule { prefix: "UV", entity_id: 288, exact: false, priority: 10 },
    PrefixRule { prefix: "UW", entity_id: 288, exact: false, priority: 10 },
    PrefixRule { prefix: "UX", entity_id: 288, exact: false, priority: 10 },
    PrefixRule { prefix: "UY", entity_id: 288, exact: false, priority: 10 },
    PrefixRule { prefix: "UZ", entity_id: 288, exact: false, priority: 10 },
    PrefixRule { prefix: "EM", entity_id: 288, exact: false, priority: 10 },
    PrefixRule { prefix: "EN", entity_id: 288, exact: false, priority: 10 },
    PrefixRule { prefix: "EO", entity_id: 288, exact: false, priority: 10 },
    
    // Baltic States
    PrefixRule { prefix: "ES", entity_id: 52, exact: false, priority: 10 },   // Estonia
    PrefixRule { prefix: "YL", entity_id: 145, exact: false, priority: 10 },  // Latvia
    PrefixRule { prefix: "LY", entity_id: 146, exact: false, priority: 10 },  // Lithuania
    
    // =========================================================================
    // ASIA
    // =========================================================================
    // Japan (ITU: JA-JS, 7J-7N, 8J-8N)
    PrefixRule { prefix: "JA", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JE", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JF", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JG", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JH", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JI", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JJ", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JK", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JL", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JM", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JN", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JO", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JP", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JQ", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JR", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "JS", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "7J", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "7K", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "7L", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "7M", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "7N", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "8J", entity_id: 339, exact: false, priority: 10 },
    PrefixRule { prefix: "8N", entity_id: 339, exact: false, priority: 10 },
    
    // China (ITU: B, 3H-3U, XS)
    PrefixRule { prefix: "B", entity_id: 318, exact: false, priority: 10 },
    PrefixRule { prefix: "BV", entity_id: 386, exact: false, priority: 20 },  // Taiwan
    
    // South Korea (ITU: HL, 6K-6N, D7-D9, DS-DT)
    PrefixRule { prefix: "HL", entity_id: 137, exact: false, priority: 10 },
    PrefixRule { prefix: "DS", entity_id: 137, exact: false, priority: 10 },
    PrefixRule { prefix: "DT", entity_id: 137, exact: false, priority: 10 },
    PrefixRule { prefix: "6K", entity_id: 137, exact: false, priority: 10 },
    PrefixRule { prefix: "6L", entity_id: 137, exact: false, priority: 10 },
    PrefixRule { prefix: "6M", entity_id: 137, exact: false, priority: 10 },
    PrefixRule { prefix: "6N", entity_id: 137, exact: false, priority: 10 },
    
    // Hong Kong (ITU: VR)
    PrefixRule { prefix: "VR", entity_id: 324, exact: false, priority: 10 },
    
    // Philippines (ITU: DU-DZ, 4D-4I)
    PrefixRule { prefix: "DU", entity_id: 372, exact: false, priority: 10 },
    PrefixRule { prefix: "DV", entity_id: 372, exact: false, priority: 10 },
    PrefixRule { prefix: "DW", entity_id: 372, exact: false, priority: 10 },
    PrefixRule { prefix: "DX", entity_id: 372, exact: false, priority: 10 },
    PrefixRule { prefix: "DY", entity_id: 372, exact: false, priority: 10 },
    PrefixRule { prefix: "DZ", entity_id: 372, exact: false, priority: 10 },
    PrefixRule { prefix: "4D", entity_id: 372, exact: false, priority: 10 },
    PrefixRule { prefix: "4E", entity_id: 372, exact: false, priority: 10 },
    PrefixRule { prefix: "4F", entity_id: 372, exact: false, priority: 10 },
    PrefixRule { prefix: "4G", entity_id: 372, exact: false, priority: 10 },
    PrefixRule { prefix: "4H", entity_id: 372, exact: false, priority: 10 },
    PrefixRule { prefix: "4I", entity_id: 372, exact: false, priority: 10 },
    
    // Thailand (ITU: HS, E2)
    PrefixRule { prefix: "HS", entity_id: 387, exact: false, priority: 10 },
    PrefixRule { prefix: "E2", entity_id: 387, exact: false, priority: 10 },
    
    // India (ITU: AT-AW, VT-VW) - Entity 324
    PrefixRule { prefix: "VU", entity_id: 324, exact: false, priority: 10 },
    PrefixRule { prefix: "AT", entity_id: 324, exact: false, priority: 10 },
    PrefixRule { prefix: "VT", entity_id: 324, exact: false, priority: 10 },
    PrefixRule { prefix: "VW", entity_id: 324, exact: false, priority: 10 },
    PrefixRule { prefix: "VU4", entity_id: 11, exact: false, priority: 30 },  // Andaman & Nicobar Is.
    PrefixRule { prefix: "VU7", entity_id: 142, exact: false, priority: 30 }, // Lakshadweep Is.
    
    // Southeast Asia
    PrefixRule { prefix: "9M2", entity_id: 299, exact: false, priority: 20 }, // West Malaysia
    PrefixRule { prefix: "9M4", entity_id: 299, exact: false, priority: 20 }, // West Malaysia
    PrefixRule { prefix: "9M6", entity_id: 46, exact: false, priority: 20 },  // East Malaysia
    PrefixRule { prefix: "9M8", entity_id: 46, exact: false, priority: 20 },  // East Malaysia
    PrefixRule { prefix: "9V", entity_id: 381, exact: false, priority: 10 },  // Singapore
    PrefixRule { prefix: "YB", entity_id: 327, exact: false, priority: 10 },  // Indonesia
    PrefixRule { prefix: "YC", entity_id: 327, exact: false, priority: 10 },  // Indonesia
    PrefixRule { prefix: "YD", entity_id: 327, exact: false, priority: 10 },  // Indonesia
    PrefixRule { prefix: "YE", entity_id: 327, exact: false, priority: 10 },  // Indonesia
    PrefixRule { prefix: "YF", entity_id: 327, exact: false, priority: 10 },  // Indonesia
    PrefixRule { prefix: "YG", entity_id: 327, exact: false, priority: 10 },  // Indonesia
    PrefixRule { prefix: "YH", entity_id: 327, exact: false, priority: 10 },  // Indonesia
    PrefixRule { prefix: "V8", entity_id: 345, exact: false, priority: 10 },  // Brunei
    PrefixRule { prefix: "4W", entity_id: 511, exact: false, priority: 10 },  // Timor-Leste
    PrefixRule { prefix: "XV", entity_id: 293, exact: false, priority: 10 },  // Vietnam
    PrefixRule { prefix: "3W", entity_id: 293, exact: false, priority: 10 },  // Vietnam
    PrefixRule { prefix: "XU", entity_id: 312, exact: false, priority: 10 },  // Cambodia
    PrefixRule { prefix: "XW", entity_id: 143, exact: false, priority: 10 },  // Laos
    PrefixRule { prefix: "XY", entity_id: 309, exact: false, priority: 10 },  // Myanmar
    PrefixRule { prefix: "XZ", entity_id: 309, exact: false, priority: 10 },  // Myanmar
    
    // South Asia
    PrefixRule { prefix: "4S", entity_id: 315, exact: false, priority: 10 },  // Sri Lanka
    PrefixRule { prefix: "4P", entity_id: 315, exact: false, priority: 10 },  // Sri Lanka
    PrefixRule { prefix: "4Q", entity_id: 315, exact: false, priority: 10 },  // Sri Lanka
    PrefixRule { prefix: "4R", entity_id: 315, exact: false, priority: 10 },  // Sri Lanka
    PrefixRule { prefix: "8Q", entity_id: 159, exact: false, priority: 10 },  // Maldives
    PrefixRule { prefix: "9N", entity_id: 369, exact: false, priority: 10 },  // Nepal
    PrefixRule { prefix: "A5", entity_id: 306, exact: false, priority: 10 },  // Bhutan
    PrefixRule { prefix: "S2", entity_id: 305, exact: false, priority: 10 },  // Bangladesh
    PrefixRule { prefix: "AP", entity_id: 372, exact: false, priority: 10 },  // Pakistan
    PrefixRule { prefix: "AS", entity_id: 372, exact: false, priority: 10 },  // Pakistan
    PrefixRule { prefix: "YA", entity_id: 3, exact: false, priority: 10 },    // Afghanistan
    PrefixRule { prefix: "T6", entity_id: 3, exact: false, priority: 10 },    // Afghanistan
    
    // Middle East
    PrefixRule { prefix: "TA", entity_id: 390, exact: false, priority: 10 },  // Turkey
    PrefixRule { prefix: "TB", entity_id: 390, exact: false, priority: 10 },  // Turkey
    PrefixRule { prefix: "TC", entity_id: 390, exact: false, priority: 10 },  // Turkey
    PrefixRule { prefix: "4X", entity_id: 336, exact: false, priority: 10 },  // Israel
    PrefixRule { prefix: "4Z", entity_id: 336, exact: false, priority: 10 },  // Israel
    PrefixRule { prefix: "JY", entity_id: 342, exact: false, priority: 10 },  // Jordan
    PrefixRule { prefix: "OD", entity_id: 354, exact: false, priority: 10 },  // Lebanon
    PrefixRule { prefix: "YK", entity_id: 384, exact: false, priority: 10 },  // Syria
    PrefixRule { prefix: "YI", entity_id: 333, exact: false, priority: 10 },  // Iraq
    PrefixRule { prefix: "EP", entity_id: 330, exact: false, priority: 10 },  // Iran
    PrefixRule { prefix: "EQ", entity_id: 330, exact: false, priority: 10 },  // Iran
    PrefixRule { prefix: "9K", entity_id: 348, exact: false, priority: 10 },  // Kuwait
    PrefixRule { prefix: "HZ", entity_id: 378, exact: false, priority: 10 },  // Saudi Arabia
    PrefixRule { prefix: "A6", entity_id: 391, exact: false, priority: 10 },  // United Arab Emirates
    PrefixRule { prefix: "A4", entity_id: 370, exact: false, priority: 10 },  // Oman
    PrefixRule { prefix: "A7", entity_id: 376, exact: false, priority: 10 },  // Qatar
    PrefixRule { prefix: "A9", entity_id: 304, exact: false, priority: 10 },  // Bahrain
    PrefixRule { prefix: "7O", entity_id: 492, exact: false, priority: 10 },  // Yemen
    
    // Central Asia
    PrefixRule { prefix: "UN", entity_id: 130, exact: false, priority: 10 },  // Kazakhstan
    PrefixRule { prefix: "UO", entity_id: 130, exact: false, priority: 10 },  // Kazakhstan
    PrefixRule { prefix: "UP", entity_id: 130, exact: false, priority: 10 },  // Kazakhstan
    PrefixRule { prefix: "UQ", entity_id: 130, exact: false, priority: 10 },  // Kazakhstan
    PrefixRule { prefix: "UJ", entity_id: 292, exact: false, priority: 10 },  // Uzbekistan
    PrefixRule { prefix: "UK", entity_id: 292, exact: false, priority: 10 },  // Uzbekistan
    PrefixRule { prefix: "UL", entity_id: 292, exact: false, priority: 10 },  // Uzbekistan
    PrefixRule { prefix: "UM", entity_id: 292, exact: false, priority: 10 },  // Uzbekistan
    PrefixRule { prefix: "EY", entity_id: 262, exact: false, priority: 10 },  // Tajikistan
    PrefixRule { prefix: "EX", entity_id: 135, exact: false, priority: 10 },  // Kyrgyzstan
    PrefixRule { prefix: "EZ", entity_id: 280, exact: false, priority: 10 },  // Turkmenistan
    
    // Caucasus
    PrefixRule { prefix: "4L", entity_id: 75, exact: false, priority: 10 },   // Georgia
    PrefixRule { prefix: "EK", entity_id: 14, exact: false, priority: 10 },   // Armenia
    PrefixRule { prefix: "4J", entity_id: 18, exact: false, priority: 10 },   // Azerbaijan
    PrefixRule { prefix: "4K", entity_id: 18, exact: false, priority: 10 },   // Azerbaijan
    
    // East Asia extras
    PrefixRule { prefix: "VR2", entity_id: 321, exact: false, priority: 20 }, // Hong Kong (more specific)
    PrefixRule { prefix: "XX9", entity_id: 152, exact: false, priority: 10 }, // Macau
    PrefixRule { prefix: "JT", entity_id: 363, exact: false, priority: 10 },  // Mongolia
    PrefixRule { prefix: "JU", entity_id: 363, exact: false, priority: 10 },  // Mongolia
    PrefixRule { prefix: "JV", entity_id: 363, exact: false, priority: 10 },  // Mongolia
    PrefixRule { prefix: "P5", entity_id: 344, exact: false, priority: 10 },  // North Korea
    PrefixRule { prefix: "JD1", entity_id: 177, exact: false, priority: 30 }, // Minami Torishima (default)
    PrefixRule { prefix: "JD1/O", entity_id: 192, exact: false, priority: 40 }, // Ogasawara
    
    // =========================================================================
    // OCEANIA
    // =========================================================================
    // Australia (ITU: AX, VH-VN, VZ)
    PrefixRule { prefix: "VK", entity_id: 150, exact: false, priority: 10 },
    PrefixRule { prefix: "AX", entity_id: 150, exact: false, priority: 10 },
    PrefixRule { prefix: "VK9C", entity_id: 153, exact: false, priority: 30 }, // Cocos Islands
    PrefixRule { prefix: "VK9X", entity_id: 38, exact: false, priority: 30 },  // Christmas Island
    PrefixRule { prefix: "VK9L", entity_id: 147, exact: false, priority: 30 }, // Lord Howe Island
    PrefixRule { prefix: "VK9N", entity_id: 189, exact: false, priority: 30 }, // Norfolk Island
    PrefixRule { prefix: "VK9W", entity_id: 35, exact: false, priority: 30 },  // Willis Island
    PrefixRule { prefix: "VK0H", entity_id: 303, exact: false, priority: 30 }, // Heard Island
    PrefixRule { prefix: "VK0M", entity_id: 303, exact: false, priority: 30 }, // Macquarie Island
    
    // New Zealand (ITU: ZK, ZL-ZM)
    PrefixRule { prefix: "ZL", entity_id: 170, exact: false, priority: 10 },
    PrefixRule { prefix: "ZM", entity_id: 170, exact: false, priority: 10 },
    PrefixRule { prefix: "ZK", entity_id: 170, exact: false, priority: 10 },
    PrefixRule { prefix: "ZL7", entity_id: 34, exact: false, priority: 30 },  // Chatham Islands
    PrefixRule { prefix: "ZL8", entity_id: 133, exact: false, priority: 30 }, // Kermadec Islands
    PrefixRule { prefix: "ZL9", entity_id: 16, exact: false, priority: 30 },  // NZ Subantarctic Islands
    
    // Pacific Islands - Fiji, Tonga, Samoa
    PrefixRule { prefix: "3D2", entity_id: 176, exact: false, priority: 10 }, // Fiji
    PrefixRule { prefix: "3D2/R", entity_id: 460, exact: false, priority: 30 }, // Rotuma
    PrefixRule { prefix: "3D2/C", entity_id: 489, exact: false, priority: 30 }, // Conway Reef
    PrefixRule { prefix: "A3", entity_id: 160, exact: false, priority: 10 },  // Tonga
    PrefixRule { prefix: "5W", entity_id: 190, exact: false, priority: 10 },  // Samoa
    PrefixRule { prefix: "E6", entity_id: 188, exact: false, priority: 10 },  // Niue
    PrefixRule { prefix: "E5", entity_id: 191, exact: false, priority: 10 },  // North Cook Islands
    PrefixRule { prefix: "E51", entity_id: 234, exact: false, priority: 30 }, // South Cook Islands
    PrefixRule { prefix: "ZK3", entity_id: 270, exact: false, priority: 10 }, // Tokelau
    PrefixRule { prefix: "FW", entity_id: 298, exact: false, priority: 10 },  // Wallis & Futuna
    PrefixRule { prefix: "FO", entity_id: 175, exact: false, priority: 10 },  // French Polynesia
    PrefixRule { prefix: "VP6", entity_id: 172, exact: false, priority: 10 }, // Pitcairn Island
    PrefixRule { prefix: "VP6/D", entity_id: 513, exact: false, priority: 30 }, // Ducie Island
    
    // Pacific Islands - Micronesia
    PrefixRule { prefix: "T8", entity_id: 22, exact: false, priority: 10 },   // Palau
    PrefixRule { prefix: "V6", entity_id: 173, exact: false, priority: 10 },  // Micronesia
    PrefixRule { prefix: "V7", entity_id: 168, exact: false, priority: 10 },  // Marshall Islands
    PrefixRule { prefix: "C2", entity_id: 157, exact: false, priority: 10 },  // Nauru
    PrefixRule { prefix: "T30", entity_id: 301, exact: false, priority: 20 }, // West Kiribati
    PrefixRule { prefix: "T31", entity_id: 31, exact: false, priority: 20 },  // Central Kiribati
    PrefixRule { prefix: "T32", entity_id: 48, exact: false, priority: 20 },  // East Kiribati
    PrefixRule { prefix: "T33", entity_id: 490, exact: false, priority: 20 }, // Banaba Island
    PrefixRule { prefix: "T2", entity_id: 282, exact: false, priority: 10 },  // Tuvalu
    PrefixRule { prefix: "YJ", entity_id: 158, exact: false, priority: 10 },  // Vanuatu
    PrefixRule { prefix: "FK", entity_id: 162, exact: false, priority: 10 },  // New Caledonia
    PrefixRule { prefix: "TX", entity_id: 512, exact: false, priority: 30 },  // Chesterfield Islands (special)
    PrefixRule { prefix: "P2", entity_id: 163, exact: false, priority: 10 },  // Papua New Guinea
    PrefixRule { prefix: "H4", entity_id: 185, exact: false, priority: 10 },  // Solomon Islands
    PrefixRule { prefix: "H40", entity_id: 507, exact: false, priority: 30 }, // Temotu Province
    
    // =========================================================================
    // AFRICA
    // =========================================================================
    // South Africa (ITU: ZR-ZU) - Entity 462
    PrefixRule { prefix: "ZS", entity_id: 462, exact: false, priority: 10 },
    PrefixRule { prefix: "ZR", entity_id: 462, exact: false, priority: 10 },
    PrefixRule { prefix: "ZT", entity_id: 462, exact: false, priority: 10 },
    PrefixRule { prefix: "ZU", entity_id: 462, exact: false, priority: 10 },
    PrefixRule { prefix: "ZS8", entity_id: 201, exact: false, priority: 30 }, // Prince Edward & Marion Is.
    
    // East Africa
    PrefixRule { prefix: "5H", entity_id: 470, exact: false, priority: 10 },  // Tanzania
    PrefixRule { prefix: "5I", entity_id: 470, exact: false, priority: 10 },  // Tanzania
    PrefixRule { prefix: "5Z", entity_id: 430, exact: false, priority: 10 },  // Kenya
    PrefixRule { prefix: "5Y", entity_id: 430, exact: false, priority: 10 },  // Kenya
    PrefixRule { prefix: "5X", entity_id: 286, exact: false, priority: 10 },  // Uganda
    PrefixRule { prefix: "9U", entity_id: 404, exact: false, priority: 10 },  // Burundi
    PrefixRule { prefix: "9X", entity_id: 454, exact: false, priority: 10 },  // Rwanda
    PrefixRule { prefix: "ET", entity_id: 53, exact: false, priority: 10 },   // Ethiopia
    PrefixRule { prefix: "E3", entity_id: 51, exact: false, priority: 10 },   // Eritrea
    PrefixRule { prefix: "6O", entity_id: 232, exact: false, priority: 10 },  // Somalia
    PrefixRule { prefix: "T5", entity_id: 232, exact: false, priority: 10 },  // Somalia
    PrefixRule { prefix: "J2", entity_id: 382, exact: false, priority: 10 },  // Djibouti
    
    // Central Africa
    PrefixRule { prefix: "9J", entity_id: 482, exact: false, priority: 10 },  // Zambia
    PrefixRule { prefix: "9I", entity_id: 482, exact: false, priority: 10 },  // Zambia
    PrefixRule { prefix: "7Q", entity_id: 440, exact: false, priority: 10 },  // Malawi
    PrefixRule { prefix: "Z2", entity_id: 452, exact: false, priority: 10 },  // Zimbabwe
    PrefixRule { prefix: "C8", entity_id: 181, exact: false, priority: 10 },  // Mozambique
    PrefixRule { prefix: "C9", entity_id: 181, exact: false, priority: 10 },  // Mozambique
    PrefixRule { prefix: "D2", entity_id: 401, exact: false, priority: 10 },  // Angola
    PrefixRule { prefix: "D3", entity_id: 401, exact: false, priority: 10 },  // Angola
    PrefixRule { prefix: "9O", entity_id: 414, exact: false, priority: 10 },  // Democratic Republic of Congo
    PrefixRule { prefix: "9P", entity_id: 414, exact: false, priority: 10 },  // Democratic Republic of Congo
    PrefixRule { prefix: "9Q", entity_id: 414, exact: false, priority: 10 },  // Democratic Republic of Congo
    PrefixRule { prefix: "9R", entity_id: 414, exact: false, priority: 10 },  // Democratic Republic of Congo
    PrefixRule { prefix: "9S", entity_id: 414, exact: false, priority: 10 },  // Democratic Republic of Congo
    PrefixRule { prefix: "9T", entity_id: 414, exact: false, priority: 10 },  // Democratic Republic of Congo
    PrefixRule { prefix: "TN", entity_id: 412, exact: false, priority: 10 },  // Republic of the Congo
    PrefixRule { prefix: "TR", entity_id: 420, exact: false, priority: 10 },  // Gabon
    PrefixRule { prefix: "3C", entity_id: 49, exact: false, priority: 10 },   // Equatorial Guinea
    PrefixRule { prefix: "3C0", entity_id: 195, exact: false, priority: 30 }, // Annobon Island
    PrefixRule { prefix: "S9", entity_id: 219, exact: false, priority: 10 },  // Sao Tome & Principe
    PrefixRule { prefix: "TJ", entity_id: 406, exact: false, priority: 10 },  // Cameroon
    PrefixRule { prefix: "TL", entity_id: 408, exact: false, priority: 10 },  // Central African Republic
    PrefixRule { prefix: "TT", entity_id: 410, exact: false, priority: 10 },  // Chad
    
    // West Africa
    PrefixRule { prefix: "5U", entity_id: 187, exact: false, priority: 10 },  // Niger
    PrefixRule { prefix: "5N", entity_id: 450, exact: false, priority: 10 },  // Nigeria
    PrefixRule { prefix: "TY", entity_id: 416, exact: false, priority: 10 },  // Benin
    PrefixRule { prefix: "5V", entity_id: 483, exact: false, priority: 10 },  // Togo
    PrefixRule { prefix: "9G", entity_id: 424, exact: false, priority: 10 },  // Ghana
    PrefixRule { prefix: "TU", entity_id: 428, exact: false, priority: 10 },  // Cote d'Ivoire
    PrefixRule { prefix: "EL", entity_id: 434, exact: false, priority: 10 },  // Liberia
    PrefixRule { prefix: "9L", entity_id: 458, exact: false, priority: 10 },  // Sierra Leone
    PrefixRule { prefix: "3X", entity_id: 107, exact: false, priority: 10 },  // Guinea
    PrefixRule { prefix: "J5", entity_id: 109, exact: false, priority: 10 },  // Guinea-Bissau
    PrefixRule { prefix: "6V", entity_id: 456, exact: false, priority: 10 },  // Senegal
    PrefixRule { prefix: "6W", entity_id: 456, exact: false, priority: 10 },  // Senegal
    PrefixRule { prefix: "C5", entity_id: 422, exact: false, priority: 10 },  // The Gambia
    PrefixRule { prefix: "TZ", entity_id: 442, exact: false, priority: 10 },  // Mali
    PrefixRule { prefix: "5T", entity_id: 444, exact: false, priority: 10 },  // Mauritania
    PrefixRule { prefix: "XT", entity_id: 480, exact: false, priority: 10 },  // Burkina Faso
    PrefixRule { prefix: "D4", entity_id: 409, exact: false, priority: 10 },  // Cabo Verde
    
    // North Africa
    PrefixRule { prefix: "CN", entity_id: 446, exact: false, priority: 10 },  // Morocco
    PrefixRule { prefix: "5C", entity_id: 446, exact: false, priority: 10 },  // Morocco
    PrefixRule { prefix: "5D", entity_id: 446, exact: false, priority: 10 },  // Morocco
    PrefixRule { prefix: "5E", entity_id: 446, exact: false, priority: 10 },  // Morocco
    PrefixRule { prefix: "5F", entity_id: 446, exact: false, priority: 10 },  // Morocco
    PrefixRule { prefix: "5G", entity_id: 446, exact: false, priority: 10 },  // Morocco
    PrefixRule { prefix: "S0", entity_id: 302, exact: false, priority: 10 },  // Western Sahara
    PrefixRule { prefix: "7R", entity_id: 400, exact: false, priority: 10 },  // Algeria
    PrefixRule { prefix: "7T", entity_id: 400, exact: false, priority: 10 },  // Algeria
    PrefixRule { prefix: "7U", entity_id: 400, exact: false, priority: 10 },  // Algeria
    PrefixRule { prefix: "7V", entity_id: 400, exact: false, priority: 10 },  // Algeria
    PrefixRule { prefix: "7W", entity_id: 400, exact: false, priority: 10 },  // Algeria
    PrefixRule { prefix: "7X", entity_id: 400, exact: false, priority: 10 },  // Algeria
    PrefixRule { prefix: "7Y", entity_id: 400, exact: false, priority: 10 },  // Algeria
    PrefixRule { prefix: "3V", entity_id: 474, exact: false, priority: 10 },  // Tunisia
    PrefixRule { prefix: "5A", entity_id: 436, exact: false, priority: 10 },  // Libya
    PrefixRule { prefix: "SU", entity_id: 478, exact: false, priority: 10 },  // Egypt
    PrefixRule { prefix: "ST", entity_id: 466, exact: false, priority: 10 },  // Sudan
    PrefixRule { prefix: "Z8", entity_id: 521, exact: false, priority: 10 },  // South Sudan
    
    // Southern Africa
    PrefixRule { prefix: "7P", entity_id: 432, exact: false, priority: 10 },  // Lesotho
    PrefixRule { prefix: "3DA", entity_id: 468, exact: false, priority: 10 }, // Eswatini (Swaziland)
    PrefixRule { prefix: "V5", entity_id: 464, exact: false, priority: 10 },  // Namibia
    PrefixRule { prefix: "A2", entity_id: 402, exact: false, priority: 10 },  // Botswana
    PrefixRule { prefix: "8O", entity_id: 402, exact: false, priority: 10 },  // Botswana
    
    // Indian Ocean (African region)
    PrefixRule { prefix: "5R", entity_id: 438, exact: false, priority: 10 },  // Madagascar
    PrefixRule { prefix: "3B8", entity_id: 165, exact: false, priority: 10 }, // Mauritius
    PrefixRule { prefix: "3B9", entity_id: 207, exact: false, priority: 10 }, // Rodrigues Island
    PrefixRule { prefix: "3B6", entity_id: 4, exact: false, priority: 10 },   // Agalega & St. Brandon
    PrefixRule { prefix: "3B7", entity_id: 4, exact: false, priority: 10 },   // Agalega & St. Brandon
    PrefixRule { prefix: "FR", entity_id: 453, exact: false, priority: 10 },  // Reunion
    PrefixRule { prefix: "FH", entity_id: 169, exact: false, priority: 10 },  // Mayotte
    PrefixRule { prefix: "D6", entity_id: 411, exact: false, priority: 10 },  // Comoros
    PrefixRule { prefix: "S7", entity_id: 379, exact: false, priority: 10 },  // Seychelles
    PrefixRule { prefix: "VQ9", entity_id: 33, exact: false, priority: 10 },  // Chagos Islands
    
    // Atlantic Islands (African region)
    PrefixRule { prefix: "3Y", entity_id: 24, exact: false, priority: 10 },   // Bouvet Island
    PrefixRule { prefix: "ZD8", entity_id: 205, exact: false, priority: 10 }, // Ascension Island
    PrefixRule { prefix: "ZD7", entity_id: 250, exact: false, priority: 10 }, // St. Helena
    PrefixRule { prefix: "ZD9", entity_id: 274, exact: false, priority: 10 }, // Tristan da Cunha & Gough
    
    // =========================================================================
    // SOUTH AMERICA
    // =========================================================================
    // Argentina (ITU: AY-AZ, L2-L9, LO-LW)
    PrefixRule { prefix: "LU", entity_id: 100, exact: false, priority: 10 },
    PrefixRule { prefix: "LO", entity_id: 100, exact: false, priority: 10 },
    PrefixRule { prefix: "LP", entity_id: 100, exact: false, priority: 10 },
    PrefixRule { prefix: "LQ", entity_id: 100, exact: false, priority: 10 },
    PrefixRule { prefix: "LR", entity_id: 100, exact: false, priority: 10 },
    PrefixRule { prefix: "LS", entity_id: 100, exact: false, priority: 10 },
    PrefixRule { prefix: "LT", entity_id: 100, exact: false, priority: 10 },
    PrefixRule { prefix: "LV", entity_id: 100, exact: false, priority: 10 },
    PrefixRule { prefix: "LW", entity_id: 100, exact: false, priority: 10 },
    PrefixRule { prefix: "AY", entity_id: 100, exact: false, priority: 10 },
    PrefixRule { prefix: "AZ", entity_id: 100, exact: false, priority: 10 },
    
    // Brazil (ITU: PP-PY, ZV-ZZ)
    PrefixRule { prefix: "PY", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "PP", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "PQ", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "PR", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "PS", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "PT", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "PU", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "PV", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "PW", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "PX", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "ZV", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "ZW", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "ZX", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "ZY", entity_id: 112, exact: false, priority: 10 },
    PrefixRule { prefix: "ZZ", entity_id: 112, exact: false, priority: 10 },
];

/// Look up the DXCC entity for a callsign
/// Uses longest prefix match with priority ordering
pub fn lookup_callsign(callsign: &str) -> Option<u16> {
    let call_upper = callsign.to_uppercase();
    
    // Find all matching prefixes, sorted by prefix length (longest first) then priority
    let mut matches: Vec<&PrefixRule> = PREFIX_RULES
        .iter()
        .filter(|rule| {
            if rule.exact {
                call_upper == rule.prefix
            } else {
                call_upper.starts_with(rule.prefix)
            }
        })
        .collect();
    
    // Sort by prefix length (descending) then priority (descending)
    matches.sort_by(|a, b| {
        let len_cmp = b.prefix.len().cmp(&a.prefix.len());
        if len_cmp == std::cmp::Ordering::Equal {
            b.priority.cmp(&a.priority)
        } else {
            len_cmp
        }
    });
    
    matches.first().map(|rule| rule.entity_id)
}

/// Get all prefixes for a given entity
pub fn get_prefixes_for_entity(entity_id: u16) -> Vec<&'static str> {
    PREFIX_RULES
        .iter()
        .filter(|rule| rule.entity_id == entity_id)
        .map(|rule| rule.prefix)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_us_callsigns() {
        assert_eq!(lookup_callsign("W1AW"), Some(291));
        assert_eq!(lookup_callsign("K5ABC"), Some(291));
        assert_eq!(lookup_callsign("N7XYZ"), Some(291));
        assert_eq!(lookup_callsign("AA1BB"), Some(291));
        assert_eq!(lookup_callsign("KB9FIN"), Some(291)); // Verify KB prefix -> US
    }

    #[test]
    fn test_us_territories() {
        assert_eq!(lookup_callsign("KH6ABC"), Some(110)); // Hawaii
        assert_eq!(lookup_callsign("KL7ABC"), Some(6));   // Alaska
        assert_eq!(lookup_callsign("KP4ABC"), Some(202)); // Puerto Rico
        assert_eq!(lookup_callsign("KP2ABC"), Some(285)); // US Virgin Islands
    }

    #[test]
    fn test_caribbean_british() {
        assert_eq!(lookup_callsign("VP2MXY"), Some(177)); // Montserrat - CRITICAL TEST
        assert_eq!(lookup_callsign("VP2EAB"), Some(8));   // Anguilla
        assert_eq!(lookup_callsign("VP2VAB"), Some(65));  // British Virgin Islands
        assert_eq!(lookup_callsign("VP5ABC"), Some(84));  // Turks & Caicos
        assert_eq!(lookup_callsign("VP9ABC"), Some(51));  // Bermuda
        assert_eq!(lookup_callsign("ZF1ABC"), Some(96));  // Cayman Islands
    }

    #[test]
    fn test_caribbean_nations() {
        assert_eq!(lookup_callsign("V26ABC"), Some(12));  // Antigua & Barbuda
        assert_eq!(lookup_callsign("V31ABC"), Some(66));  // Belize
        assert_eq!(lookup_callsign("V47ABC"), Some(246)); // St. Kitts & Nevis
        assert_eq!(lookup_callsign("C6ABC"), Some(211));  // Bahamas
        assert_eq!(lookup_callsign("6Y5ABC"), Some(82));  // Jamaica
        assert_eq!(lookup_callsign("8P6ABC"), Some(62));  // Barbados
        assert_eq!(lookup_callsign("9Y4ABC"), Some(249)); // Trinidad & Tobago
        assert_eq!(lookup_callsign("J38ABC"), Some(97));  // Grenada
        assert_eq!(lookup_callsign("J68ABC"), Some(97));  // St. Lucia (ARRL entity 097)
        assert_eq!(lookup_callsign("J79ABC"), Some(95));  // Dominica
        assert_eq!(lookup_callsign("J88ABC"), Some(98));  // St. Vincent
    }

    #[test]
    fn test_caribbean_other() {
        assert_eq!(lookup_callsign("CO8LY"), Some(70));   // Cuba
        assert_eq!(lookup_callsign("HI3ABC"), Some(72));  // Dominican Republic
        assert_eq!(lookup_callsign("HH2ABC"), Some(78));  // Haiti
        assert_eq!(lookup_callsign("FM5ABC"), Some(64));  // Martinique
        assert_eq!(lookup_callsign("FG5ABC"), Some(79));  // Guadeloupe
        assert_eq!(lookup_callsign("PJ2ABC"), Some(517)); // Curacao
        assert_eq!(lookup_callsign("PJ4ABC"), Some(52));  // Bonaire
        assert_eq!(lookup_callsign("PJ7ABC"), Some(520)); // Sint Maarten
        assert_eq!(lookup_callsign("P40ABC"), Some(9));   // Aruba
    }

    #[test]
    fn test_central_america() {
        assert_eq!(lookup_callsign("TI2ABC"), Some(308)); // Costa Rica
        assert_eq!(lookup_callsign("TG9ABC"), Some(76));  // Guatemala (ARRL entity 076)
        assert_eq!(lookup_callsign("HR2ABC"), Some(80));  // Honduras
        assert_eq!(lookup_callsign("YN2ABC"), Some(86));  // Nicaragua
        assert_eq!(lookup_callsign("YS1ABC"), Some(74));  // El Salvador (ARRL entity 074)
        assert_eq!(lookup_callsign("HP1ABC"), Some(88));  // Panama
    }

    #[test]
    fn test_south_america() {
        assert_eq!(lookup_callsign("YV5ABC"), Some(148)); // Venezuela
        assert_eq!(lookup_callsign("HK3ABC"), Some(116)); // Colombia
        assert_eq!(lookup_callsign("HC2ABC"), Some(120)); // Ecuador
        assert_eq!(lookup_callsign("HC8ABC"), Some(71));  // Galapagos
        assert_eq!(lookup_callsign("OA4ABC"), Some(136)); // Peru
        assert_eq!(lookup_callsign("CP6ABC"), Some(117)); // Bolivia
        assert_eq!(lookup_callsign("CE3ABC"), Some(108)); // Chile
        assert_eq!(lookup_callsign("LU1ABC"), Some(100)); // Argentina
        assert_eq!(lookup_callsign("PY2ABC"), Some(112)); // Brazil
        assert_eq!(lookup_callsign("ZP5ABC"), Some(132)); // Paraguay
        assert_eq!(lookup_callsign("CX2ABC"), Some(144)); // Uruguay
    }

    #[test]
    fn test_japan() {
        assert_eq!(lookup_callsign("JA1ABC"), Some(339));
        assert_eq!(lookup_callsign("JH1NBN"), Some(339));
        assert_eq!(lookup_callsign("7K1ABC"), Some(339));
    }

    #[test]
    fn test_germany() {
        assert_eq!(lookup_callsign("DL1ABC"), Some(230));
        assert_eq!(lookup_callsign("DJ5XYZ"), Some(230));
    }

    #[test]
    fn test_uk() {
        assert_eq!(lookup_callsign("G3ABC"), Some(223));  // England
        assert_eq!(lookup_callsign("GM3ABC"), Some(265)); // Scotland
        assert_eq!(lookup_callsign("GW3ABC"), Some(294)); // Wales
    }

    #[test]
    fn test_hk0_san_andres() {
        // HK0 prefix should map to San Andres & Providencia (entity 216)
        assert_eq!(lookup_callsign("HK0"), Some(216));
        // Note: lookup_callsign doesn't handle compound callsigns like HK0/DF3TJ
        // That's handled by extract_dxcc_portion in mod.rs
    }
}
