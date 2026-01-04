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
    
    // India (ITU: AT-AW, VT-VW)
    PrefixRule { prefix: "VU", entity_id: 324, exact: false, priority: 10 },
    PrefixRule { prefix: "AT", entity_id: 324, exact: false, priority: 10 },
    
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
    
    // =========================================================================
    // AFRICA
    // =========================================================================
    // South Africa (ITU: ZR-ZU)
    PrefixRule { prefix: "ZS", entity_id: 400, exact: false, priority: 10 },
    PrefixRule { prefix: "ZR", entity_id: 400, exact: false, priority: 10 },
    PrefixRule { prefix: "ZT", entity_id: 400, exact: false, priority: 10 },
    PrefixRule { prefix: "ZU", entity_id: 400, exact: false, priority: 10 },
    
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
    }

    #[test]
    fn test_us_territories() {
        assert_eq!(lookup_callsign("KH6ABC"), Some(110)); // Hawaii
        assert_eq!(lookup_callsign("KL7ABC"), Some(6));   // Alaska
        assert_eq!(lookup_callsign("KP4ABC"), Some(202)); // Puerto Rico
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
}
