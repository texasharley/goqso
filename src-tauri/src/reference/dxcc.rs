// DXCC Entity List - Current Entities
// Source: ARRL DXCC List (https://www.arrl.org/country-lists-prefixes)
// Last updated: 2025-01-01
// 
// This is the authoritative list of DXCC entities as defined by ARRL.
// Changes to this list are announced by ARRL and occur ~1-2 times per year.
//
// Fields:
// - entity_id: ARRL DXCC entity number (1-340+)
// - name: Official ARRL entity name
// - continent: Two-letter continent code (NA, SA, EU, AF, AS, OC, AN)
// - cq_zone: CQ zone number (1-40)
// - itu_zone: ITU zone number (1-90)
// - deleted: Whether this entity is deleted (no longer valid for new contacts)
// - notes: Any special notes about the entity

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DxccEntity {
    pub entity_id: u16,
    pub name: &'static str,
    pub continent: &'static str,
    pub cq_zone: u8,
    pub itu_zone: u8,
    pub deleted: bool,
}

/// Current DXCC entities list
/// This list is curated from the official ARRL DXCC list
pub const DXCC_ENTITIES: &[DxccEntity] = &[
    // =========================================================================
    // NORTH AMERICA (NA)
    // =========================================================================
    DxccEntity { entity_id: 1, name: "Canada", continent: "NA", cq_zone: 1, itu_zone: 2, deleted: false },
    DxccEntity { entity_id: 6, name: "Alaska", continent: "NA", cq_zone: 1, itu_zone: 1, deleted: false },
    DxccEntity { entity_id: 110, name: "Hawaii", continent: "OC", cq_zone: 31, itu_zone: 61, deleted: false },
    DxccEntity { entity_id: 291, name: "United States", continent: "NA", cq_zone: 3, itu_zone: 6, deleted: false },
    DxccEntity { entity_id: 202, name: "Puerto Rico", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 285, name: "US Virgin Islands", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 105, name: "Guantanamo Bay", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 182, name: "Navassa Island", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    
    // Mexico and Central America
    DxccEntity { entity_id: 50, name: "Mexico", continent: "NA", cq_zone: 6, itu_zone: 10, deleted: false },
    DxccEntity { entity_id: 78, name: "Guatemala", continent: "NA", cq_zone: 7, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 80, name: "Honduras", continent: "NA", cq_zone: 7, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 86, name: "Nicaragua", continent: "NA", cq_zone: 7, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 62, name: "El Salvador", continent: "NA", cq_zone: 7, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 308, name: "Costa Rica", continent: "NA", cq_zone: 7, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 88, name: "Panama", continent: "NA", cq_zone: 7, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 66, name: "Belize", continent: "NA", cq_zone: 7, itu_zone: 11, deleted: false },
    
    // Caribbean
    DxccEntity { entity_id: 70, name: "Cuba", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 72, name: "Dominican Republic", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 78, name: "Haiti", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 82, name: "Jamaica", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 249, name: "Trinidad & Tobago", continent: "SA", cq_zone: 9, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 9, name: "Aruba", continent: "SA", cq_zone: 9, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 52, name: "Bonaire", continent: "SA", cq_zone: 9, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 517, name: "Curacao", continent: "SA", cq_zone: 9, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 520, name: "Sint Maarten", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 516, name: "Saba & St. Eustatius", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 8, name: "Anguilla", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 12, name: "Antigua & Barbuda", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 62, name: "Barbados", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 95, name: "Dominica", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 97, name: "Grenada", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 177, name: "Montserrat", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 246, name: "St. Kitts & Nevis", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 65, name: "St. Lucia", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 98, name: "St. Vincent", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 64, name: "Martinique", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 79, name: "Guadeloupe", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 213, name: "St. Martin", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 214, name: "St. Barthelemy", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 211, name: "Bahamas", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 84, name: "Turks & Caicos Islands", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 65, name: "British Virgin Islands", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 96, name: "Cayman Islands", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 51, name: "Bermuda", continent: "NA", cq_zone: 5, itu_zone: 11, deleted: false },
    
    // =========================================================================
    // EUROPE (EU)
    // =========================================================================
    DxccEntity { entity_id: 223, name: "England", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 265, name: "Scotland", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 294, name: "Wales", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 279, name: "Northern Ireland", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 114, name: "Isle of Man", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 106, name: "Jersey", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 104, name: "Guernsey", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 122, name: "Ireland", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 227, name: "France", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 230, name: "Germany", continent: "EU", cq_zone: 14, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 236, name: "Italy", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 281, name: "Spain", continent: "EU", cq_zone: 14, itu_zone: 37, deleted: false },
    DxccEntity { entity_id: 272, name: "Portugal", continent: "EU", cq_zone: 14, itu_zone: 37, deleted: false },
    DxccEntity { entity_id: 263, name: "Netherlands", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 209, name: "Belgium", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 254, name: "Luxembourg", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 287, name: "Switzerland", continent: "EU", cq_zone: 14, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 206, name: "Austria", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 239, name: "Liechtenstein", continent: "EU", cq_zone: 14, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 260, name: "Monaco", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 27, name: "Andorra", continent: "EU", cq_zone: 14, itu_zone: 27, deleted: false },
    DxccEntity { entity_id: 278, name: "San Marino", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 295, name: "Vatican City", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 259, name: "Malta", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 225, name: "Cyprus", continent: "AS", cq_zone: 20, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 5, name: "Sovereign Base Areas", continent: "AS", cq_zone: 20, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 237, name: "Gibraltar", continent: "EU", cq_zone: 14, itu_zone: 37, deleted: false },
    
    // Scandinavia
    DxccEntity { entity_id: 222, name: "Denmark", continent: "EU", cq_zone: 14, itu_zone: 18, deleted: false },
    DxccEntity { entity_id: 266, name: "Norway", continent: "EU", cq_zone: 14, itu_zone: 18, deleted: false },
    DxccEntity { entity_id: 284, name: "Sweden", continent: "EU", cq_zone: 14, itu_zone: 18, deleted: false },
    DxccEntity { entity_id: 224, name: "Finland", continent: "EU", cq_zone: 15, itu_zone: 18, deleted: false },
    DxccEntity { entity_id: 167, name: "Aland Islands", continent: "EU", cq_zone: 15, itu_zone: 18, deleted: false },
    DxccEntity { entity_id: 118, name: "Iceland", continent: "EU", cq_zone: 40, itu_zone: 17, deleted: false },
    DxccEntity { entity_id: 221, name: "Faroe Islands", continent: "EU", cq_zone: 14, itu_zone: 18, deleted: false },
    DxccEntity { entity_id: 5, name: "Svalbard", continent: "EU", cq_zone: 40, itu_zone: 18, deleted: false },
    DxccEntity { entity_id: 259, name: "Jan Mayen", continent: "EU", cq_zone: 40, itu_zone: 18, deleted: false },
    DxccEntity { entity_id: 222, name: "Greenland", continent: "NA", cq_zone: 40, itu_zone: 5, deleted: false },
    
    // Eastern Europe
    DxccEntity { entity_id: 269, name: "Poland", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 503, name: "Czech Republic", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 504, name: "Slovakia", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 239, name: "Hungary", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 275, name: "Romania", continent: "EU", cq_zone: 20, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 212, name: "Bulgaria", continent: "EU", cq_zone: 20, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 245, name: "Greece", continent: "EU", cq_zone: 20, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 180, name: "Crete", continent: "EU", cq_zone: 20, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 40, name: "Dodecanese", continent: "EU", cq_zone: 20, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 219, name: "Albania", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 502, name: "North Macedonia", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 501, name: "Bosnia-Herzegovina", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 497, name: "Croatia", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 499, name: "Slovenia", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 296, name: "Serbia", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 514, name: "Kosovo", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    DxccEntity { entity_id: 513, name: "Montenegro", continent: "EU", cq_zone: 15, itu_zone: 28, deleted: false },
    
    // Baltic States
    DxccEntity { entity_id: 52, name: "Estonia", continent: "EU", cq_zone: 15, itu_zone: 29, deleted: false },
    DxccEntity { entity_id: 145, name: "Latvia", continent: "EU", cq_zone: 15, itu_zone: 29, deleted: false },
    DxccEntity { entity_id: 146, name: "Lithuania", continent: "EU", cq_zone: 15, itu_zone: 29, deleted: false },
    DxccEntity { entity_id: 126, name: "Kaliningrad", continent: "EU", cq_zone: 15, itu_zone: 29, deleted: false },
    
    // Former Soviet
    DxccEntity { entity_id: 54, name: "European Russia", continent: "EU", cq_zone: 16, itu_zone: 29, deleted: false },
    DxccEntity { entity_id: 15, name: "Asiatic Russia", continent: "AS", cq_zone: 17, itu_zone: 30, deleted: false },
    DxccEntity { entity_id: 288, name: "Ukraine", continent: "EU", cq_zone: 16, itu_zone: 29, deleted: false },
    DxccEntity { entity_id: 61, name: "Belarus", continent: "EU", cq_zone: 16, itu_zone: 29, deleted: false },
    DxccEntity { entity_id: 179, name: "Moldova", continent: "EU", cq_zone: 16, itu_zone: 29, deleted: false },
    
    // =========================================================================
    // ASIA (AS)
    // =========================================================================
    DxccEntity { entity_id: 339, name: "Japan", continent: "AS", cq_zone: 25, itu_zone: 45, deleted: false },
    DxccEntity { entity_id: 318, name: "China", continent: "AS", cq_zone: 24, itu_zone: 44, deleted: false },
    DxccEntity { entity_id: 386, name: "Taiwan", continent: "AS", cq_zone: 24, itu_zone: 44, deleted: false },
    DxccEntity { entity_id: 137, name: "South Korea", continent: "AS", cq_zone: 25, itu_zone: 44, deleted: false },
    DxccEntity { entity_id: 344, name: "North Korea", continent: "AS", cq_zone: 25, itu_zone: 44, deleted: false },
    DxccEntity { entity_id: 324, name: "Hong Kong", continent: "AS", cq_zone: 24, itu_zone: 44, deleted: false },
    DxccEntity { entity_id: 152, name: "Macau", continent: "AS", cq_zone: 24, itu_zone: 44, deleted: false },
    DxccEntity { entity_id: 363, name: "Mongolia", continent: "AS", cq_zone: 23, itu_zone: 32, deleted: false },
    DxccEntity { entity_id: 372, name: "Philippines", continent: "OC", cq_zone: 27, itu_zone: 50, deleted: false },
    DxccEntity { entity_id: 247, name: "Vietnam", continent: "AS", cq_zone: 26, itu_zone: 49, deleted: false },
    DxccEntity { entity_id: 387, name: "Thailand", continent: "AS", cq_zone: 26, itu_zone: 49, deleted: false },
    DxccEntity { entity_id: 312, name: "Cambodia", continent: "AS", cq_zone: 26, itu_zone: 49, deleted: false },
    DxccEntity { entity_id: 345, name: "Laos", continent: "AS", cq_zone: 26, itu_zone: 49, deleted: false },
    DxccEntity { entity_id: 309, name: "Myanmar", continent: "AS", cq_zone: 26, itu_zone: 49, deleted: false },
    DxccEntity { entity_id: 506, name: "East Timor", continent: "OC", cq_zone: 28, itu_zone: 54, deleted: false },
    DxccEntity { entity_id: 305, name: "Bangladesh", continent: "AS", cq_zone: 22, itu_zone: 41, deleted: false },
    DxccEntity { entity_id: 324, name: "India", continent: "AS", cq_zone: 22, itu_zone: 41, deleted: false },
    DxccEntity { entity_id: 315, name: "Sri Lanka", continent: "AS", cq_zone: 22, itu_zone: 41, deleted: false },
    DxccEntity { entity_id: 378, name: "Maldives", continent: "AS", cq_zone: 22, itu_zone: 41, deleted: false },
    DxccEntity { entity_id: 369, name: "Nepal", continent: "AS", cq_zone: 22, itu_zone: 42, deleted: false },
    DxccEntity { entity_id: 306, name: "Bhutan", continent: "AS", cq_zone: 22, itu_zone: 41, deleted: false },
    DxccEntity { entity_id: 372, name: "Pakistan", continent: "AS", cq_zone: 21, itu_zone: 41, deleted: false },
    DxccEntity { entity_id: 3, name: "Afghanistan", continent: "AS", cq_zone: 21, itu_zone: 40, deleted: false },
    
    // Middle East
    DxccEntity { entity_id: 391, name: "Turkey", continent: "AS", cq_zone: 20, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 336, name: "Israel", continent: "AS", cq_zone: 20, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 354, name: "Jordan", continent: "AS", cq_zone: 20, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 333, name: "Iraq", continent: "AS", cq_zone: 21, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 330, name: "Iran", continent: "AS", cq_zone: 21, itu_zone: 40, deleted: false },
    DxccEntity { entity_id: 348, name: "Kuwait", continent: "AS", cq_zone: 21, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 378, name: "Saudi Arabia", continent: "AS", cq_zone: 21, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 301, name: "United Arab Emirates", continent: "AS", cq_zone: 21, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 370, name: "Oman", continent: "AS", cq_zone: 21, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 376, name: "Qatar", continent: "AS", cq_zone: 21, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 304, name: "Bahrain", continent: "AS", cq_zone: 21, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 400, name: "Yemen", continent: "AS", cq_zone: 21, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 354, name: "Lebanon", continent: "AS", cq_zone: 20, itu_zone: 39, deleted: false },
    DxccEntity { entity_id: 384, name: "Syria", continent: "AS", cq_zone: 20, itu_zone: 39, deleted: false },
    
    // Central Asia
    DxccEntity { entity_id: 130, name: "Kazakhstan", continent: "AS", cq_zone: 17, itu_zone: 30, deleted: false },
    DxccEntity { entity_id: 292, name: "Uzbekistan", continent: "AS", cq_zone: 17, itu_zone: 30, deleted: false },
    DxccEntity { entity_id: 262, name: "Tajikistan", continent: "AS", cq_zone: 17, itu_zone: 30, deleted: false },
    DxccEntity { entity_id: 135, name: "Kyrgyzstan", continent: "AS", cq_zone: 17, itu_zone: 30, deleted: false },
    DxccEntity { entity_id: 280, name: "Turkmenistan", continent: "AS", cq_zone: 17, itu_zone: 30, deleted: false },
    
    // Caucasus
    DxccEntity { entity_id: 75, name: "Georgia", continent: "AS", cq_zone: 21, itu_zone: 29, deleted: false },
    DxccEntity { entity_id: 18, name: "Armenia", continent: "AS", cq_zone: 21, itu_zone: 29, deleted: false },
    DxccEntity { entity_id: 18, name: "Azerbaijan", continent: "AS", cq_zone: 21, itu_zone: 29, deleted: false },
    
    // Southeast Asia
    DxccEntity { entity_id: 248, name: "Malaysia (West)", continent: "AS", cq_zone: 28, itu_zone: 54, deleted: false },
    DxccEntity { entity_id: 50, name: "Malaysia (East)", continent: "OC", cq_zone: 28, itu_zone: 54, deleted: false },
    DxccEntity { entity_id: 381, name: "Singapore", continent: "AS", cq_zone: 28, itu_zone: 54, deleted: false },
    DxccEntity { entity_id: 327, name: "Indonesia", continent: "OC", cq_zone: 28, itu_zone: 54, deleted: false },
    DxccEntity { entity_id: 88, name: "Brunei", continent: "OC", cq_zone: 28, itu_zone: 54, deleted: false },
    
    // =========================================================================
    // OCEANIA (OC)
    // =========================================================================
    DxccEntity { entity_id: 150, name: "Australia", continent: "OC", cq_zone: 30, itu_zone: 59, deleted: false },
    DxccEntity { entity_id: 153, name: "Cocos (Keeling) Islands", continent: "OC", cq_zone: 29, itu_zone: 54, deleted: false },
    DxccEntity { entity_id: 38, name: "Christmas Island", continent: "OC", cq_zone: 29, itu_zone: 54, deleted: false },
    DxccEntity { entity_id: 303, name: "Heard Island", continent: "AF", cq_zone: 39, itu_zone: 68, deleted: false },
    DxccEntity { entity_id: 147, name: "Lord Howe Island", continent: "OC", cq_zone: 30, itu_zone: 60, deleted: false },
    DxccEntity { entity_id: 303, name: "Macquarie Island", continent: "OC", cq_zone: 30, itu_zone: 60, deleted: false },
    DxccEntity { entity_id: 189, name: "Norfolk Island", continent: "OC", cq_zone: 32, itu_zone: 60, deleted: false },
    DxccEntity { entity_id: 35, name: "Willis Island", continent: "OC", cq_zone: 30, itu_zone: 55, deleted: false },
    DxccEntity { entity_id: 170, name: "New Zealand", continent: "OC", cq_zone: 32, itu_zone: 60, deleted: false },
    DxccEntity { entity_id: 34, name: "Chatham Islands", continent: "OC", cq_zone: 32, itu_zone: 60, deleted: false },
    DxccEntity { entity_id: 133, name: "Kermadec Islands", continent: "OC", cq_zone: 32, itu_zone: 60, deleted: false },
    DxccEntity { entity_id: 16, name: "Auckland & Campbell", continent: "OC", cq_zone: 32, itu_zone: 60, deleted: false },
    
    // Pacific Islands
    DxccEntity { entity_id: 32, name: "Fiji", continent: "OC", cq_zone: 32, itu_zone: 56, deleted: false },
    DxccEntity { entity_id: 168, name: "Rotuma", continent: "OC", cq_zone: 32, itu_zone: 56, deleted: false },
    DxccEntity { entity_id: 169, name: "Conway Reef", continent: "OC", cq_zone: 32, itu_zone: 56, deleted: false },
    DxccEntity { entity_id: 190, name: "Tonga", continent: "OC", cq_zone: 32, itu_zone: 62, deleted: false },
    DxccEntity { entity_id: 191, name: "Samoa", continent: "OC", cq_zone: 32, itu_zone: 62, deleted: false },
    DxccEntity { entity_id: 197, name: "American Samoa", continent: "OC", cq_zone: 32, itu_zone: 62, deleted: false },
    DxccEntity { entity_id: 160, name: "Niue", continent: "OC", cq_zone: 32, itu_zone: 62, deleted: false },
    DxccEntity { entity_id: 181, name: "Cook Islands (North)", continent: "OC", cq_zone: 32, itu_zone: 62, deleted: false },
    DxccEntity { entity_id: 234, name: "Cook Islands (South)", continent: "OC", cq_zone: 32, itu_zone: 62, deleted: false },
    DxccEntity { entity_id: 508, name: "Tokelau Islands", continent: "OC", cq_zone: 31, itu_zone: 62, deleted: false },
    DxccEntity { entity_id: 123, name: "Wallis & Futuna", continent: "OC", cq_zone: 32, itu_zone: 62, deleted: false },
    DxccEntity { entity_id: 509, name: "French Polynesia", continent: "OC", cq_zone: 32, itu_zone: 63, deleted: false },
    DxccEntity { entity_id: 175, name: "Marquesas Islands", continent: "OC", cq_zone: 31, itu_zone: 63, deleted: false },
    DxccEntity { entity_id: 4, name: "Austral Islands", continent: "OC", cq_zone: 32, itu_zone: 63, deleted: false },
    DxccEntity { entity_id: 298, name: "Pitcairn Island", continent: "OC", cq_zone: 32, itu_zone: 63, deleted: false },
    DxccEntity { entity_id: 47, name: "Ducie Island", continent: "OC", cq_zone: 32, itu_zone: 63, deleted: false },
    DxccEntity { entity_id: 20, name: "Baker & Howland Islands", continent: "OC", cq_zone: 31, itu_zone: 61, deleted: false },
    DxccEntity { entity_id: 197, name: "Jarvis Island", continent: "OC", cq_zone: 31, itu_zone: 61, deleted: false },
    DxccEntity { entity_id: 134, name: "Kingman Reef", continent: "OC", cq_zone: 31, itu_zone: 61, deleted: false },
    DxccEntity { entity_id: 138, name: "Palmyra & Jarvis", continent: "OC", cq_zone: 31, itu_zone: 61, deleted: false },
    DxccEntity { entity_id: 297, name: "Johnston Island", continent: "OC", cq_zone: 31, itu_zone: 61, deleted: false },
    DxccEntity { entity_id: 174, name: "Midway Island", continent: "OC", cq_zone: 31, itu_zone: 61, deleted: false },
    DxccEntity { entity_id: 297, name: "Wake Island", continent: "OC", cq_zone: 31, itu_zone: 65, deleted: false },
    DxccEntity { entity_id: 103, name: "Guam", continent: "OC", cq_zone: 27, itu_zone: 64, deleted: false },
    DxccEntity { entity_id: 166, name: "Mariana Islands", continent: "OC", cq_zone: 27, itu_zone: 64, deleted: false },
    DxccEntity { entity_id: 22, name: "Palau", continent: "OC", cq_zone: 27, itu_zone: 64, deleted: false },
    DxccEntity { entity_id: 27, name: "Micronesia", continent: "OC", cq_zone: 27, itu_zone: 65, deleted: false },
    DxccEntity { entity_id: 158, name: "Marshall Islands", continent: "OC", cq_zone: 31, itu_zone: 65, deleted: false },
    DxccEntity { entity_id: 157, name: "Nauru", continent: "OC", cq_zone: 31, itu_zone: 65, deleted: false },
    DxccEntity { entity_id: 31, name: "Kiribati", continent: "OC", cq_zone: 31, itu_zone: 65, deleted: false },
    DxccEntity { entity_id: 490, name: "Banaba Island", continent: "OC", cq_zone: 31, itu_zone: 65, deleted: false },
    DxccEntity { entity_id: 282, name: "Tuvalu", continent: "OC", cq_zone: 31, itu_zone: 65, deleted: false },
    DxccEntity { entity_id: 185, name: "Vanuatu", continent: "OC", cq_zone: 32, itu_zone: 56, deleted: false },
    DxccEntity { entity_id: 163, name: "New Caledonia", continent: "OC", cq_zone: 32, itu_zone: 56, deleted: false },
    DxccEntity { entity_id: 162, name: "Chesterfield Islands", continent: "OC", cq_zone: 30, itu_zone: 56, deleted: false },
    DxccEntity { entity_id: 98, name: "Papua New Guinea", continent: "OC", cq_zone: 28, itu_zone: 51, deleted: false },
    DxccEntity { entity_id: 28, name: "Solomon Islands", continent: "OC", cq_zone: 28, itu_zone: 51, deleted: false },
    DxccEntity { entity_id: 185, name: "Temotu Province", continent: "OC", cq_zone: 32, itu_zone: 51, deleted: false },
    
    // =========================================================================
    // AFRICA (AF)
    // =========================================================================
    DxccEntity { entity_id: 400, name: "South Africa", continent: "AF", cq_zone: 38, itu_zone: 57, deleted: false },
    DxccEntity { entity_id: 462, name: "Marion Island", continent: "AF", cq_zone: 38, itu_zone: 57, deleted: false },
    DxccEntity { entity_id: 4, name: "Crozet Islands", continent: "AF", cq_zone: 39, itu_zone: 68, deleted: false },
    DxccEntity { entity_id: 131, name: "Kerguelen Islands", continent: "AF", cq_zone: 39, itu_zone: 68, deleted: false },
    DxccEntity { entity_id: 10, name: "Amsterdam & St. Paul", continent: "AF", cq_zone: 39, itu_zone: 68, deleted: false },
    DxccEntity { entity_id: 391, name: "Lesotho", continent: "AF", cq_zone: 38, itu_zone: 57, deleted: false },
    DxccEntity { entity_id: 468, name: "Swaziland", continent: "AF", cq_zone: 38, itu_zone: 57, deleted: false },
    DxccEntity { entity_id: 411, name: "Namibia", continent: "AF", cq_zone: 38, itu_zone: 57, deleted: false },
    DxccEntity { entity_id: 402, name: "Botswana", continent: "AF", cq_zone: 38, itu_zone: 57, deleted: false },
    DxccEntity { entity_id: 452, name: "Zimbabwe", continent: "AF", cq_zone: 38, itu_zone: 53, deleted: false },
    DxccEntity { entity_id: 483, name: "Zambia", continent: "AF", cq_zone: 36, itu_zone: 53, deleted: false },
    DxccEntity { entity_id: 505, name: "Malawi", continent: "AF", cq_zone: 37, itu_zone: 53, deleted: false },
    DxccEntity { entity_id: 440, name: "Mozambique", continent: "AF", cq_zone: 37, itu_zone: 53, deleted: false },
    DxccEntity { entity_id: 430, name: "Angola", continent: "AF", cq_zone: 36, itu_zone: 52, deleted: false },
    DxccEntity { entity_id: 428, name: "Democratic Republic of Congo", continent: "AF", cq_zone: 36, itu_zone: 52, deleted: false },
    DxccEntity { entity_id: 412, name: "Republic of the Congo", continent: "AF", cq_zone: 36, itu_zone: 52, deleted: false },
    DxccEntity { entity_id: 420, name: "Gabon", continent: "AF", cq_zone: 36, itu_zone: 52, deleted: false },
    DxccEntity { entity_id: 424, name: "Equatorial Guinea", continent: "AF", cq_zone: 36, itu_zone: 47, deleted: false },
    DxccEntity { entity_id: 456, name: "Sao Tome & Principe", continent: "AF", cq_zone: 36, itu_zone: 47, deleted: false },
    DxccEntity { entity_id: 408, name: "Cameroon", continent: "AF", cq_zone: 36, itu_zone: 47, deleted: false },
    DxccEntity { entity_id: 410, name: "Central African Republic", continent: "AF", cq_zone: 36, itu_zone: 47, deleted: false },
    DxccEntity { entity_id: 406, name: "Chad", continent: "AF", cq_zone: 36, itu_zone: 47, deleted: false },
    DxccEntity { entity_id: 446, name: "Niger", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 450, name: "Nigeria", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 416, name: "Benin", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 470, name: "Togo", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 424, name: "Ghana", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 418, name: "Cote d'Ivoire", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 434, name: "Liberia", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 466, name: "Sierra Leone", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 460, name: "Guinea", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 422, name: "Guinea-Bissau", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 458, name: "Senegal", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 422, name: "The Gambia", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 436, name: "Mali", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 474, name: "Mauritania", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 444, name: "Morocco", continent: "AF", cq_zone: 33, itu_zone: 37, deleted: false },
    DxccEntity { entity_id: 478, name: "Western Sahara", continent: "AF", cq_zone: 33, itu_zone: 46, deleted: false },
    DxccEntity { entity_id: 400, name: "Algeria", continent: "AF", cq_zone: 33, itu_zone: 37, deleted: false },
    DxccEntity { entity_id: 438, name: "Tunisia", continent: "AF", cq_zone: 33, itu_zone: 37, deleted: false },
    DxccEntity { entity_id: 436, name: "Libya", continent: "AF", cq_zone: 34, itu_zone: 38, deleted: false },
    DxccEntity { entity_id: 478, name: "Egypt", continent: "AF", cq_zone: 34, itu_zone: 38, deleted: false },
    DxccEntity { entity_id: 466, name: "Sudan", continent: "AF", cq_zone: 34, itu_zone: 48, deleted: false },
    DxccEntity { entity_id: 521, name: "South Sudan", continent: "AF", cq_zone: 34, itu_zone: 48, deleted: false },
    DxccEntity { entity_id: 436, name: "Ethiopia", continent: "AF", cq_zone: 37, itu_zone: 48, deleted: false },
    DxccEntity { entity_id: 414, name: "Eritrea", continent: "AF", cq_zone: 37, itu_zone: 48, deleted: false },
    DxccEntity { entity_id: 474, name: "Djibouti", continent: "AF", cq_zone: 37, itu_zone: 48, deleted: false },
    DxccEntity { entity_id: 232, name: "Somalia", continent: "AF", cq_zone: 37, itu_zone: 48, deleted: false },
    DxccEntity { entity_id: 404, name: "Kenya", continent: "AF", cq_zone: 37, itu_zone: 48, deleted: false },
    DxccEntity { entity_id: 286, name: "Uganda", continent: "AF", cq_zone: 37, itu_zone: 48, deleted: false },
    DxccEntity { entity_id: 439, name: "Tanzania", continent: "AF", cq_zone: 37, itu_zone: 53, deleted: false },
    DxccEntity { entity_id: 454, name: "Rwanda", continent: "AF", cq_zone: 36, itu_zone: 52, deleted: false },
    DxccEntity { entity_id: 404, name: "Burundi", continent: "AF", cq_zone: 36, itu_zone: 52, deleted: false },
    
    // Indian Ocean
    DxccEntity { entity_id: 384, name: "Madagascar", continent: "AF", cq_zone: 39, itu_zone: 53, deleted: false },
    DxccEntity { entity_id: 453, name: "Mauritius", continent: "AF", cq_zone: 39, itu_zone: 53, deleted: false },
    DxccEntity { entity_id: 41, name: "Rodriguez Island", continent: "AF", cq_zone: 39, itu_zone: 53, deleted: false },
    DxccEntity { entity_id: 1, name: "Agalega & St. Brandon", continent: "AF", cq_zone: 39, itu_zone: 53, deleted: false },
    DxccEntity { entity_id: 165, name: "Reunion", continent: "AF", cq_zone: 39, itu_zone: 53, deleted: false },
    DxccEntity { entity_id: 38, name: "Mayotte", continent: "AF", cq_zone: 39, itu_zone: 53, deleted: false },
    DxccEntity { entity_id: 21, name: "Comoros", continent: "AF", cq_zone: 39, itu_zone: 53, deleted: false },
    DxccEntity { entity_id: 379, name: "Seychelles", continent: "AF", cq_zone: 39, itu_zone: 53, deleted: false },
    DxccEntity { entity_id: 37, name: "Chagos Islands", continent: "AF", cq_zone: 39, itu_zone: 41, deleted: false },
    
    // Atlantic Islands (African region)
    DxccEntity { entity_id: 36, name: "Bouvet Island", continent: "AF", cq_zone: 38, itu_zone: 67, deleted: false },
    DxccEntity { entity_id: 250, name: "Ascension Island", continent: "AF", cq_zone: 36, itu_zone: 66, deleted: false },
    DxccEntity { entity_id: 183, name: "St. Helena", continent: "AF", cq_zone: 36, itu_zone: 66, deleted: false },
    DxccEntity { entity_id: 251, name: "Tristan da Cunha & Gough", continent: "AF", cq_zone: 38, itu_zone: 66, deleted: false },
    DxccEntity { entity_id: 29, name: "Canary Islands", continent: "AF", cq_zone: 33, itu_zone: 36, deleted: false },
    DxccEntity { entity_id: 256, name: "Ceuta & Melilla", continent: "AF", cq_zone: 33, itu_zone: 37, deleted: false },
    DxccEntity { entity_id: 149, name: "Madeira", continent: "AF", cq_zone: 33, itu_zone: 36, deleted: false },
    DxccEntity { entity_id: 21, name: "Azores", continent: "EU", cq_zone: 14, itu_zone: 36, deleted: false },
    DxccEntity { entity_id: 255, name: "Cape Verde", continent: "AF", cq_zone: 35, itu_zone: 46, deleted: false },
    
    // =========================================================================
    // SOUTH AMERICA (SA)
    // =========================================================================
    DxccEntity { entity_id: 100, name: "Argentina", continent: "SA", cq_zone: 13, itu_zone: 14, deleted: false },
    DxccEntity { entity_id: 112, name: "Brazil", continent: "SA", cq_zone: 11, itu_zone: 12, deleted: false },
    DxccEntity { entity_id: 108, name: "Chile", continent: "SA", cq_zone: 12, itu_zone: 14, deleted: false },
    DxccEntity { entity_id: 47, name: "Easter Island", continent: "SA", cq_zone: 12, itu_zone: 63, deleted: false },
    DxccEntity { entity_id: 125, name: "San Felix & San Ambrosio", continent: "SA", cq_zone: 12, itu_zone: 14, deleted: false },
    DxccEntity { entity_id: 126, name: "Juan Fernandez", continent: "SA", cq_zone: 12, itu_zone: 14, deleted: false },
    DxccEntity { entity_id: 117, name: "Bolivia", continent: "SA", cq_zone: 10, itu_zone: 12, deleted: false },
    DxccEntity { entity_id: 136, name: "Peru", continent: "SA", cq_zone: 10, itu_zone: 12, deleted: false },
    DxccEntity { entity_id: 120, name: "Ecuador", continent: "SA", cq_zone: 10, itu_zone: 12, deleted: false },
    DxccEntity { entity_id: 71, name: "Galapagos Islands", continent: "SA", cq_zone: 10, itu_zone: 12, deleted: false },
    DxccEntity { entity_id: 116, name: "Colombia", continent: "SA", cq_zone: 9, itu_zone: 12, deleted: false },
    DxccEntity { entity_id: 140, name: "San Andres & Providencia", continent: "NA", cq_zone: 7, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 144, name: "Malpelo Island", continent: "SA", cq_zone: 9, itu_zone: 12, deleted: false },
    DxccEntity { entity_id: 148, name: "Venezuela", continent: "SA", cq_zone: 9, itu_zone: 12, deleted: false },
    DxccEntity { entity_id: 17, name: "Aves Island", continent: "NA", cq_zone: 8, itu_zone: 11, deleted: false },
    DxccEntity { entity_id: 76, name: "French Guiana", continent: "SA", cq_zone: 9, itu_zone: 12, deleted: false },
    DxccEntity { entity_id: 129, name: "Suriname", continent: "SA", cq_zone: 9, itu_zone: 12, deleted: false },
    DxccEntity { entity_id: 77, name: "Guyana", continent: "SA", cq_zone: 9, itu_zone: 12, deleted: false },
    DxccEntity { entity_id: 132, name: "Paraguay", continent: "SA", cq_zone: 11, itu_zone: 14, deleted: false },
    DxccEntity { entity_id: 144, name: "Uruguay", continent: "SA", cq_zone: 13, itu_zone: 14, deleted: false },
    DxccEntity { entity_id: 56, name: "Falkland Islands", continent: "SA", cq_zone: 13, itu_zone: 16, deleted: false },
    DxccEntity { entity_id: 241, name: "South Georgia", continent: "SA", cq_zone: 13, itu_zone: 73, deleted: false },
    DxccEntity { entity_id: 240, name: "South Sandwich Islands", continent: "SA", cq_zone: 13, itu_zone: 73, deleted: false },
    DxccEntity { entity_id: 235, name: "South Orkney Islands", continent: "SA", cq_zone: 13, itu_zone: 73, deleted: false },
    DxccEntity { entity_id: 239, name: "South Shetland Islands", continent: "AN", cq_zone: 13, itu_zone: 73, deleted: false },
    
    // =========================================================================
    // ANTARCTICA (AN)
    // =========================================================================
    DxccEntity { entity_id: 13, name: "Antarctica", continent: "AN", cq_zone: 39, itu_zone: 74, deleted: false },
    DxccEntity { entity_id: 199, name: "Peter I Island", continent: "AN", cq_zone: 12, itu_zone: 72, deleted: false },
];

/// Get a DXCC entity by entity ID
pub fn get_entity(entity_id: u16) -> Option<&'static DxccEntity> {
    DXCC_ENTITIES.iter().find(|e| e.entity_id == entity_id)
}

/// Get a DXCC entity by name (case-insensitive)
pub fn get_entity_by_name(name: &str) -> Option<&'static DxccEntity> {
    let name_lower = name.to_lowercase();
    DXCC_ENTITIES.iter().find(|e| e.name.to_lowercase() == name_lower)
}

/// Get all current (non-deleted) entities
pub fn get_current_entities() -> Vec<&'static DxccEntity> {
    DXCC_ENTITIES.iter().filter(|e| !e.deleted).collect()
}

/// Get total count of current entities
pub fn current_entity_count() -> usize {
    DXCC_ENTITIES.iter().filter(|e| !e.deleted).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_major_entities() {
        assert!(get_entity(291).is_some()); // United States
        assert!(get_entity(339).is_some()); // Japan
        assert!(get_entity(227).is_some()); // France
    }

    #[test]
    fn test_entity_by_name() {
        assert!(get_entity_by_name("United States").is_some());
        assert!(get_entity_by_name("united states").is_some()); // case insensitive
    }

    #[test]
    fn test_current_entity_count() {
        // DXCC has approximately 340 current entities
        let count = current_entity_count();
        assert!(count > 300, "Expected at least 300 entities, got {}", count);
    }
}
