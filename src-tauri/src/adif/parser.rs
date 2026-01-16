// ADIF Parser
// Handles both standard ADIF and LoTW confirmation files
//
// NOTE: Some accessor methods (qsl_rcvd) are defined for completeness
// but not currently used. Keeping for future QSL verification features.
//
#![allow(dead_code)]

use std::collections::HashMap;

/// A single ADIF record (one QSO)
#[derive(Debug, Clone, Default)]
pub struct AdifRecord {
    /// All fields as key-value pairs (uppercase keys)
    pub fields: HashMap<String, String>,
}

impl AdifRecord {
    pub fn new() -> Self {
        Self { fields: HashMap::new() }
    }

    /// Get a field value (case-insensitive lookup)
    pub fn get(&self, key: &str) -> Option<&String> {
        self.fields.get(&key.to_uppercase())
    }

    /// Get field or default
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.fields.get(&key.to_uppercase())
            .map(|s| s.clone())
            .unwrap_or_else(|| default.to_string())
    }

    /// Check if a field exists
    pub fn has(&self, key: &str) -> bool {
        self.fields.contains_key(&key.to_uppercase())
    }

    /// Get standard fields for matching
    pub fn call(&self) -> Option<&String> { self.get("CALL") }
    pub fn band(&self) -> Option<&String> { self.get("BAND") }
    pub fn mode(&self) -> Option<&String> { self.get("MODE") }
    pub fn qso_date(&self) -> Option<&String> { self.get("QSO_DATE") }
    pub fn time_on(&self) -> Option<&String> { self.get("TIME_ON") }
    pub fn freq(&self) -> Option<f64> {
        self.get("FREQ").and_then(|s| s.parse().ok())
    }
    pub fn dxcc(&self) -> Option<i64> {
        self.get("DXCC").and_then(|s| s.parse().ok())
    }
    pub fn state(&self) -> Option<&String> { self.get("STATE") }
    pub fn cnty(&self) -> Option<&String> { self.get("CNTY") }
    pub fn gridsquare(&self) -> Option<&String> { self.get("GRIDSQUARE") }
    pub fn country(&self) -> Option<&String> { self.get("COUNTRY") }
    pub fn cqz(&self) -> Option<i64> {
        self.get("CQZ").and_then(|s| s.parse().ok())
    }
    pub fn ituz(&self) -> Option<i64> {
        self.get("ITUZ").and_then(|s| s.parse().ok())
    }
    
    // LoTW specific
    pub fn qsl_rcvd(&self) -> Option<&String> { self.get("QSL_RCVD") }
    pub fn qslrdate(&self) -> Option<&String> { self.get("QSLRDATE") }
    pub fn is_lotw_confirmed(&self) -> bool {
        self.get("QSL_RCVD").map(|s| s == "Y").unwrap_or(false)
    }
}

/// Parsed ADIF file
#[derive(Debug, Clone)]
pub struct AdifFile {
    /// Header fields (before <EOH>)
    pub header: HashMap<String, String>,
    /// QSO records
    pub records: Vec<AdifRecord>,
}

impl AdifFile {
    pub fn new() -> Self {
        Self {
            header: HashMap::new(),
            records: Vec::new(),
        }
    }
}

/// Parse an ADIF string into records
pub fn parse_adif(content: &str) -> Result<AdifFile, String> {
    let mut file = AdifFile::new();
    
    // Find end of header
    let content_upper = content.to_uppercase();
    let body_start = if let Some(eoh_pos) = content_upper.find("<EOH>") {
        // Parse header fields
        let header_section = &content[..eoh_pos];
        parse_fields_into(header_section, &mut file.header);
        eoh_pos + 5 // Skip past <EOH>
    } else {
        // No header, start from beginning
        0
    };
    
    let body = &content[body_start..];
    
    // Split by <EOR> case-insensitively
    let mut current_pos = 0;
    let body_upper = body.to_uppercase();
    
    while let Some(eor_offset) = body_upper[current_pos..].find("<EOR>") {
        let record_end = current_pos + eor_offset;
        let record_str = &body[current_pos..record_end];
        
        if !record_str.trim().is_empty() {
            let mut record = AdifRecord::new();
            parse_fields_into(record_str, &mut record.fields);
            
            // Only add if it has at least a CALL field
            if record.has("CALL") {
                file.records.push(record);
            }
        }
        
        current_pos = record_end + 5; // Skip past <EOR>
    }
    
    Ok(file)
}

/// Parse ADIF fields from a string section into a HashMap
fn parse_fields_into(content: &str, map: &mut HashMap<String, String>) {
    let mut pos = 0;
    let bytes = content.as_bytes();
    
    while pos < bytes.len() {
        // Find next '<'
        match bytes[pos..].iter().position(|&b| b == b'<') {
            Some(offset) => {
                pos += offset + 1; // Move past '<'
            }
            None => break,
        }
        
        // Find matching '>'
        let field_end = match bytes[pos..].iter().position(|&b| b == b'>') {
            Some(offset) => pos + offset,
            None => break,
        };
        
        let field_spec = &content[pos..field_end];
        pos = field_end + 1; // Move past '>'
        
        // Parse field spec: NAME:LENGTH or NAME:LENGTH:TYPE
        // Also handle NAME only (for <EOH>, <EOR>)
        let parts: Vec<&str> = field_spec.split(':').collect();
        if parts.is_empty() {
            continue;
        }
        
        let field_name = parts[0].to_uppercase();
        
        // Skip control tags
        if field_name == "EOH" || field_name == "EOR" {
            continue;
        }
        
        // Get length if specified
        let length: usize = if parts.len() > 1 {
            parts[1].parse().unwrap_or(0)
        } else {
            0
        };
        
        // Extract value
        if length > 0 && pos + length <= bytes.len() {
            let value = &content[pos..pos + length];
            // Strip comments (text after //)
            let clean_value = value.split("//").next().unwrap_or(value).trim();
            map.insert(field_name, clean_value.to_string());
            pos += length;
        } else if length == 0 {
            // Boolean/empty field
            map.insert(field_name, String::new());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_record() {
        let adif = r#"
<CALL:5>W1ABC
<BAND:3>20M
<MODE:3>FT8
<QSO_DATE:8>20260103
<TIME_ON:6>152600
<RST_SENT:3>-12
<RST_RCVD:3>-08
<GRIDSQUARE:4>FN31
<EOR>
"#;
        
        let file = parse_adif(adif).unwrap();
        assert_eq!(file.records.len(), 1);
        
        let rec = &file.records[0];
        assert_eq!(rec.call(), Some(&"W1ABC".to_string()));
        assert_eq!(rec.band(), Some(&"20M".to_string()));
        assert_eq!(rec.mode(), Some(&"FT8".to_string()));
        assert_eq!(rec.gridsquare(), Some(&"FN31".to_string()));
    }
    
    #[test]
    fn test_parse_lotw_format() {
        let adif = r#"
<PROGRAMID:4>LoTW
<EOH>

<CALL:5>W6ELI
<BAND:3>20M
<MODE:3>FT8
<QSO_DATE:8>20260103
<TIME_ON:6>150800
<QSL_RCVD:1>Y
<QSLRDATE:8>20260104
<DXCC:3>291
<COUNTRY:24>UNITED STATES OF AMERICA
<STATE:2>CA // California
<CNTY:14>CA,LOS ANGELES // Los Angeles
<CQZ:2>03
<GRIDSQUARE:4>DM04
<EOR>
"#;
        
        let file = parse_adif(adif).unwrap();
        assert_eq!(file.records.len(), 1);
        
        let rec = &file.records[0];
        assert!(rec.is_lotw_confirmed());
        assert_eq!(rec.state(), Some(&"CA".to_string()));
        assert_eq!(rec.cnty(), Some(&"CA,LOS ANGELES".to_string()));
        assert_eq!(rec.dxcc(), Some(291));
    }
    
    #[test]
    fn test_parse_multiple_records() {
        let adif = r#"
<CALL:5>W1ABC<BAND:3>20M<MODE:3>FT8<QSO_DATE:8>20260103<TIME_ON:4>1526<EOR>
<CALL:5>N2XYZ<BAND:3>40M<MODE:2>CW<QSO_DATE:8>20260103<TIME_ON:4>1630<EOR>
<CALL:4>K3AB<BAND:3>15M<MODE:3>SSB<QSO_DATE:8>20260103<TIME_ON:4>1745<EOR>
"#;
        
        let file = parse_adif(adif).unwrap();
        assert_eq!(file.records.len(), 3);
        assert_eq!(file.records[0].call(), Some(&"W1ABC".to_string()));
        assert_eq!(file.records[1].call(), Some(&"N2XYZ".to_string()));
        assert_eq!(file.records[2].call(), Some(&"K3AB".to_string()));
    }
}
