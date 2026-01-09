// FCC Database Parser Module
//
// Parses the EN.dat file from the FCC ULS database.
// EN.dat contains Entity records with licensee information.
//
// Format: Pipe-delimited (|) with the following columns:
// 0:  Record Type (EN)
// 1:  Unique System Identifier
// 2:  ULS File Number
// 3:  EBF Number
// 4:  Call Sign
// 5:  Entity Type (L=Licensee, C=Contact, O=Owner, etc.)
// 6:  Licensee ID
// 7:  Entity Name (for clubs/organizations)
// 8:  First Name
// 9:  MI (Middle Initial)
// 10: Last Name
// 11: Suffix
// 12: Phone
// 13: Fax
// 14: Email
// 15: Street Address
// 16: City
// 17: State (2-letter code)
// 18: Zip Code
// 19: PO Box
// 20: Attention Line
// 21: SGIN
// 22: FCC Registration Number (FRN)
// 23: Applicant Type Code
// 24: Applicant Type Code Other
// 25: Status Code
// 26: Status Date

use std::path::PathBuf;
use std::io::{BufRead, BufReader};
use std::fs::File;
use sqlx::SqlitePool;

/// Represents an FCC amateur license record
#[derive(Debug, Clone)]
pub struct FccLicense {
    pub call: String,
    pub entity_type: String,
    pub entity_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub frn: Option<String>,
}

impl FccLicense {
    /// Get the licensee name (combines first/last for individuals, uses entity name for clubs)
    pub fn name(&self) -> Option<String> {
        if let (Some(first), Some(last)) = (&self.first_name, &self.last_name) {
            if !first.is_empty() && !last.is_empty() {
                return Some(format!("{} {}", first, last));
            }
        }
        self.entity_name.clone()
    }
}

/// Parse EN.dat and import records into the database
/// 
/// Returns the number of records imported
pub async fn parse_fcc_database(en_path: &PathBuf, pool: &SqlitePool) -> Result<usize, String> {
    log::info!("Parsing FCC EN.dat file: {:?}", en_path);
    
    let file = File::open(en_path)
        .map_err(|e| format!("Failed to open EN.dat: {}", e))?;
    
    let reader = BufReader::new(file);
    let mut records: Vec<FccLicense> = Vec::new();
    let mut line_count = 0;
    let mut skipped = 0;
    
    for line in reader.lines() {
        line_count += 1;
        
        let line = match line {
            Ok(l) => l,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }
        
        // Parse pipe-delimited fields
        let fields: Vec<&str> = line.split('|').collect();
        
        // Need at least 18 fields for state
        if fields.len() < 18 {
            skipped += 1;
            continue;
        }
        
        // Only process EN (Entity) records
        if fields[0] != "EN" {
            continue;
        }
        
        // Only process Licensee entities (L)
        let entity_type = fields.get(5).unwrap_or(&"").to_string();
        if entity_type != "L" {
            continue;
        }
        
        // Extract call sign (field 4)
        let call = fields.get(4).unwrap_or(&"").trim().to_uppercase();
        if call.is_empty() {
            skipped += 1;
            continue;
        }
        
        // Extract other fields
        let entity_name = non_empty_string(fields.get(7));
        let first_name = non_empty_string(fields.get(8));
        let last_name = non_empty_string(fields.get(10));
        let city = non_empty_string(fields.get(16));
        let state = non_empty_string(fields.get(17));
        let zip = non_empty_string(fields.get(18));
        let frn = non_empty_string(fields.get(22));
        
        // Only include records with a valid state (we care about US licensees)
        if state.is_none() {
            skipped += 1;
            continue;
        }
        
        records.push(FccLicense {
            call,
            entity_type,
            entity_name,
            first_name,
            last_name,
            city,
            state,
            zip,
            frn,
        });
        
        // Log progress every 100k records
        if records.len() % 100_000 == 0 {
            log::info!("Parsed {} records...", records.len());
        }
    }
    
    log::info!("Parsed {} total lines, {} valid records, {} skipped", 
               line_count, records.len(), skipped);
    
    // Import into database in batches
    let batch_size = 1000;
    let total = records.len();
    
    // Clear existing records first
    sqlx::query("DELETE FROM fcc_licenses")
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to clear existing FCC records: {}", e))?;
    
    log::info!("Importing {} records into database...", total);
    
    for (i, chunk) in records.chunks(batch_size).enumerate() {
        import_batch(pool, chunk).await?;
        
        if (i + 1) % 100 == 0 {
            log::info!("Imported {}/{} records...", (i + 1) * batch_size, total);
        }
    }
    
    log::info!("FCC database import complete: {} records", total);
    
    Ok(total)
}

/// Import a batch of records
async fn import_batch(pool: &SqlitePool, records: &[FccLicense]) -> Result<(), String> {
    // Build a multi-value INSERT
    let mut query = String::from(
        "INSERT OR REPLACE INTO fcc_licenses (call, name, city, state, zip, frn, updated_at) VALUES "
    );
    
    let values: Vec<String> = records.iter().map(|r| {
        let name = r.name()
            .map(|n| format!("'{}'", n.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string());
        let city = r.city.as_ref()
            .map(|c| format!("'{}'", c.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string());
        let state = r.state.as_ref()
            .map(|s| format!("'{}'", s.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string());
        let zip = r.zip.as_ref()
            .map(|z| format!("'{}'", z.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string());
        let frn = r.frn.as_ref()
            .map(|f| format!("'{}'", f.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string());
        
        format!("('{}', {}, {}, {}, {}, {}, datetime('now'))", 
                r.call.replace('\'', "''"), name, city, state, zip, frn)
    }).collect();
    
    query.push_str(&values.join(", "));
    
    sqlx::query(&query)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to insert FCC records: {}", e))?;
    
    Ok(())
}

/// Helper to convert empty strings to None
fn non_empty_string(s: Option<&&str>) -> Option<String> {
    s.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_non_empty_string() {
        assert_eq!(non_empty_string(Some(&"")), None);
        assert_eq!(non_empty_string(Some(&"  ")), None);
        assert_eq!(non_empty_string(Some(&"test")), Some("test".to_string()));
        assert_eq!(non_empty_string(None), None);
    }
    
    #[test]
    fn test_license_name() {
        let license = FccLicense {
            call: "W1AW".to_string(),
            entity_type: "L".to_string(),
            entity_name: None,
            first_name: Some("John".to_string()),
            last_name: Some("Smith".to_string()),
            city: Some("Newington".to_string()),
            state: Some("CT".to_string()),
            zip: Some("06111".to_string()),
            frn: Some("1234567890".to_string()),
        };
        
        assert_eq!(license.name(), Some("John Smith".to_string()));
    }
}
