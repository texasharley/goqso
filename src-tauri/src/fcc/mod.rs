// FCC Amateur License Database Module
//
// Downloads and imports the FCC ULS amateur license database for offline
// callsign lookups. Essential for POTA/portable operations without internet.
//
// Data source: https://data.fcc.gov/download/pub/uls/complete/l_amat.zip
// Update frequency: Weekly (automatic background sync on app startup)

mod download;
mod parser;

pub use download::download_fcc_database;
pub use parser::parse_fcc_database;

use sqlx::SqlitePool;
use serde::Serialize;
use tauri::Manager;

/// Check if FCC sync is needed and run it silently in the background
/// Syncs if: never synced, or last sync > 7 days ago
pub async fn sync_fcc_if_needed(app: &tauri::AppHandle) {
    let state = app.state::<crate::commands::AppState>();
    let db_guard = state.db.lock().await;
    let pool = match db_guard.as_ref() {
        Some(p) => p,
        None => {
            log::warn!("FCC sync: database not ready");
            return;
        }
    };
    
    // Check if sync is needed
    let needs_sync = match get_sync_status(pool).await {
        Ok(status) => {
            if status.record_count == 0 {
                log::info!("FCC database empty, will sync");
                true
            } else if let Some(last_sync) = status.last_sync_at {
                // Parse the timestamp and check if > 7 days old
                match chrono::NaiveDateTime::parse_from_str(&last_sync, "%Y-%m-%d %H:%M:%S") {
                    Ok(dt) => {
                        let age = chrono::Utc::now().naive_utc() - dt;
                        if age.num_days() > 7 {
                            log::info!("FCC database is {} days old, will sync", age.num_days());
                            true
                        } else {
                            log::debug!("FCC database is {} days old, no sync needed", age.num_days());
                            false
                        }
                    }
                    Err(_) => {
                        log::warn!("Failed to parse FCC sync timestamp, will sync");
                        true
                    }
                }
            } else {
                log::info!("FCC database has no sync timestamp, will sync");
                true
            }
        }
        Err(e) => {
            log::warn!("Failed to get FCC sync status: {}, will sync", e);
            true
        }
    };
    
    if !needs_sync {
        return;
    }
    
    // Release the lock before the long-running sync
    drop(db_guard);
    
    log::info!("Starting background FCC database sync...");
    
    // Get app data directory
    let data_dir = match app.path().app_data_dir() {
        Ok(d) => d,
        Err(e) => {
            log::error!("FCC sync: failed to get app data dir: {}", e);
            return;
        }
    };
    
    // Download the database
    let en_path = match download_fcc_database(&data_dir).await {
        Ok(path) => path,
        Err(e) => {
            log::error!("FCC sync: download failed: {}", e);
            return;
        }
    };
    
    // Re-acquire the lock for import
    let db_guard = state.db.lock().await;
    let pool = match db_guard.as_ref() {
        Some(p) => p,
        None => {
            log::error!("FCC sync: database disappeared during download");
            return;
        }
    };
    
    // Parse and import
    let record_count = match parse_fcc_database(&en_path, pool).await {
        Ok(count) => count,
        Err(e) => {
            log::error!("FCC sync: import failed: {}", e);
            return;
        }
    };
    
    // Update sync status
    let _ = sqlx::query(
        r#"UPDATE fcc_sync_status SET 
           sync_in_progress = 0, 
           last_sync_at = datetime('now'),
           record_count = ?,
           error_message = NULL
           WHERE id = 1"#
    )
    .bind(record_count as i64)
    .execute(pool)
    .await;
    
    log::info!("FCC background sync complete: {} records imported", record_count);
}

/// FCC sync status
#[derive(Debug, Serialize, Clone)]
pub struct FccSyncStatus {
    pub last_sync_at: Option<String>,
    pub record_count: i64,
    pub file_date: Option<String>,
    pub sync_in_progress: bool,
    pub error_message: Option<String>,
}

/// Get current FCC sync status
pub async fn get_sync_status(pool: &SqlitePool) -> Result<FccSyncStatus, String> {
    let row: (Option<String>, i64, Option<String>, i64, Option<String>) = sqlx::query_as(
        r#"SELECT last_sync_at, record_count, file_date, sync_in_progress, error_message 
           FROM fcc_sync_status WHERE id = 1"#
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to get FCC sync status: {}", e))?;
    
    Ok(FccSyncStatus {
        last_sync_at: row.0,
        record_count: row.1,
        file_date: row.2,
        sync_in_progress: row.3 != 0,
        error_message: row.4,
    })
}

/// Lookup a callsign in the FCC database
pub async fn lookup_callsign(pool: &SqlitePool, call: &str) -> Option<FccLicenseInfo> {
    let call_upper = call.to_uppercase();
    
    let result: Result<(String, Option<String>, Option<String>, Option<String>, Option<String>), _> = 
        sqlx::query_as(
            "SELECT call, name, state, city, grid FROM fcc_licenses WHERE call = ?"
        )
        .bind(&call_upper)
        .fetch_one(pool)
        .await;
    
    match result {
        Ok((call, name, state, city, grid)) => Some(FccLicenseInfo {
            call,
            name,
            state,
            city,
            grid,
        }),
        Err(_) => None,
    }
}

/// Simplified license info for lookups
#[derive(Debug, Serialize, Clone)]
pub struct FccLicenseInfo {
    pub call: String,
    pub name: Option<String>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub grid: Option<String>,
}

/// Batch lookup multiple callsigns
pub async fn lookup_callsigns(pool: &SqlitePool, calls: &[String]) -> Vec<FccLicenseInfo> {
    if calls.is_empty() {
        return Vec::new();
    }
    
    // Build query with placeholders
    let placeholders: Vec<String> = calls.iter().map(|_| "?".to_string()).collect();
    let query = format!(
        "SELECT call, name, state, city, grid FROM fcc_licenses WHERE call IN ({})",
        placeholders.join(", ")
    );
    
    let mut q = sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>, Option<String>)>(&query);
    
    for call in calls {
        q = q.bind(call.to_uppercase());
    }
    
    match q.fetch_all(pool).await {
        Ok(rows) => rows.into_iter().map(|(call, name, state, city, grid)| {
            FccLicenseInfo { call, name, state, city, grid }
        }).collect(),
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    
    /// Helper to create an in-memory database with FCC schema and test data
    async fn setup_test_db_with_callsigns(callsigns: &[(&str, &str)]) -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");
        
        // Create the fcc_licenses table
        sqlx::query(r#"
            CREATE TABLE fcc_licenses (
                call TEXT PRIMARY KEY,
                name TEXT,
                city TEXT,
                state TEXT,
                zip TEXT,
                grid TEXT,
                license_class TEXT,
                grant_date TEXT,
                expire_date TEXT,
                frn TEXT,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#)
        .execute(&pool)
        .await
        .expect("Failed to create fcc_licenses table");
        
        // Create fcc_sync_status table
        sqlx::query(r#"
            CREATE TABLE fcc_sync_status (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                last_sync_at TEXT,
                record_count INTEGER DEFAULT 0,
                file_date TEXT,
                sync_in_progress INTEGER DEFAULT 0,
                error_message TEXT,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#)
        .execute(&pool)
        .await
        .expect("Failed to create fcc_sync_status table");
        
        sqlx::query("INSERT INTO fcc_sync_status (id) VALUES (1)")
            .execute(&pool)
            .await
            .expect("Failed to insert sync status");
        
        // Insert test callsigns
        for (call, state) in callsigns {
            sqlx::query("INSERT INTO fcc_licenses (call, state, name, city) VALUES (?, ?, 'Test Op', 'Test City')")
                .bind(*call)
                .bind(*state)
                .execute(&pool)
                .await
                .expect(&format!("Failed to insert callsign {}", call));
        }
        
        pool
    }
    
    /// Test: FCC lookup returns correct state for a single callsign
    #[tokio::test]
    async fn test_lookup_single_callsign() {
        let pool = setup_test_db_with_callsigns(&[("W1AW", "CT")]).await;
        
        let result = lookup_callsign(&pool, "W1AW").await;
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.call, "W1AW");
        assert_eq!(info.state, Some("CT".to_string()));
    }
    
    /// Test: FCC lookup is case-insensitive
    #[tokio::test]
    async fn test_lookup_case_insensitive() {
        let pool = setup_test_db_with_callsigns(&[("W1AW", "CT")]).await;
        
        let result = lookup_callsign(&pool, "w1aw").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().call, "W1AW");
    }
    
    /// Test: FCC lookup returns None for unknown callsign
    #[tokio::test]
    async fn test_lookup_unknown_callsign() {
        let pool = setup_test_db_with_callsigns(&[("W1AW", "CT")]).await;
        
        let result = lookup_callsign(&pool, "NOTACALL").await;
        assert!(result.is_none());
    }
    
    /// Test: Batch lookup returns all matching callsigns
    #[tokio::test]
    async fn test_batch_lookup() {
        let pool = setup_test_db_with_callsigns(&[
            ("W1AW", "CT"),
            ("K5TXT", "TX"),
            ("N6AA", "CA"),
        ]).await;
        
        let calls = vec!["W1AW".to_string(), "K5TXT".to_string(), "N6AA".to_string()];
        let results = lookup_callsigns(&pool, &calls).await;
        
        assert_eq!(results.len(), 3);
    }
    
    /// Test: Get sync status
    #[tokio::test]
    async fn test_get_sync_status() {
        let pool = setup_test_db_with_callsigns(&[]).await;
        
        let status = get_sync_status(&pool).await;
        assert!(status.is_ok());
        let status = status.unwrap();
        assert_eq!(status.record_count, 0);
        assert!(!status.sync_in_progress);
    }
    
    /// Comprehensive test: 100 callsigns across all 50 states (2 per state)
    /// This validates state lookup accuracy for a diverse set of callsigns
    #[tokio::test]
    async fn test_state_lookup_100_callsigns() {
        // 100 test callsigns - 2 per state covering all 50 US states
        // These are representative callsign patterns for each call area
        let test_callsigns: Vec<(&str, &str)> = vec![
            // Alabama (AL)
            ("K4AB", "AL"), ("W4ABC", "AL"),
            // Alaska (AK)
            ("KL7AA", "AK"), ("AL7AB", "AK"),
            // Arizona (AZ)
            ("K7AZ", "AZ"), ("W7AZA", "AZ"),
            // Arkansas (AR)
            ("K5AR", "AR"), ("W5ARK", "AR"),
            // California (CA)
            ("K6CA", "CA"), ("W6CAL", "CA"),
            // Colorado (CO)
            ("K0CO", "CO"), ("W0COL", "CO"),
            // Connecticut (CT)
            ("K1CT", "CT"), ("W1AW", "CT"),
            // Delaware (DE)
            ("K3DE", "DE"), ("W3DEL", "DE"),
            // Florida (FL)
            ("K4FL", "FL"), ("W4FLA", "FL"),
            // Georgia (GA)
            ("K4GA", "GA"), ("W4ATL", "GA"),
            // Hawaii (HI)
            ("KH6HI", "HI"), ("WH6AB", "HI"),
            // Idaho (ID)
            ("K7ID", "ID"), ("W7IDA", "ID"),
            // Illinois (IL)
            ("K9IL", "IL"), ("W9CHI", "IL"),
            // Indiana (IN)
            ("K9IN", "IN"), ("W9IND", "IN"),
            // Iowa (IA)
            ("K0IA", "IA"), ("W0IOW", "IA"),
            // Kansas (KS)
            ("K0KS", "KS"), ("W0KAN", "KS"),
            // Kentucky (KY)
            ("K4KY", "KY"), ("W4KEN", "KY"),
            // Louisiana (LA)
            ("K5LA", "LA"), ("W5NOR", "LA"),
            // Maine (ME)
            ("K1ME", "ME"), ("W1MAI", "ME"),
            // Maryland (MD)
            ("K3MD", "MD"), ("W3BAL", "MD"),
            // Massachusetts (MA)
            ("K1MA", "MA"), ("W1BOS", "MA"),
            // Michigan (MI)
            ("K8MI", "MI"), ("W8DET", "MI"),
            // Minnesota (MN)
            ("K0MN", "MN"), ("W0MIN", "MN"),
            // Mississippi (MS)
            ("K5MS", "MS"), ("W5MIS", "MS"),
            // Missouri (MO)
            ("K0MO", "MO"), ("W0STL", "MO"),
            // Montana (MT)
            ("K7MT", "MT"), ("W7MON", "MT"),
            // Nebraska (NE)
            ("K0NE", "NE"), ("W0NEB", "NE"),
            // Nevada (NV)
            ("K7NV", "NV"), ("W7VEG", "NV"),
            // New Hampshire (NH)
            ("K1NH", "NH"), ("W1HAM", "NH"),
            // New Jersey (NJ)
            ("K2NJ", "NJ"), ("W2JER", "NJ"),
            // New Mexico (NM)
            ("K5NM", "NM"), ("W5ABQ", "NM"),
            // New York (NY)
            ("K2NY", "NY"), ("W2NYC", "NY"),
            // North Carolina (NC)
            ("K4NC", "NC"), ("W4CAR", "NC"),
            // North Dakota (ND)
            ("K0ND", "ND"), ("W0NDA", "ND"),
            // Ohio (OH)
            ("K8OH", "OH"), ("W8OHI", "OH"),
            // Oklahoma (OK)
            ("K5OK", "OK"), ("W5OKL", "OK"),
            // Oregon (OR)
            ("K7OR", "OR"), ("W7ORE", "OR"),
            // Pennsylvania (PA)
            ("K3PA", "PA"), ("W3PHI", "PA"),
            // Rhode Island (RI)
            ("K1RI", "RI"), ("W1RHO", "RI"),
            // South Carolina (SC)
            ("K4SC", "SC"), ("W4SCA", "SC"),
            // South Dakota (SD)
            ("K0SD", "SD"), ("W0SDA", "SD"),
            // Tennessee (TN)
            ("K4TN", "TN"), ("W4NAS", "TN"),
            // Texas (TX)
            ("K5TX", "TX"), ("W5TEX", "TX"),
            // Utah (UT)
            ("K7UT", "UT"), ("W7SLC", "UT"),
            // Vermont (VT)
            ("K1VT", "VT"), ("W1VER", "VT"),
            // Virginia (VA)
            ("K4VA", "VA"), ("W4VIR", "VA"),
            // Washington (WA)
            ("K7WA", "WA"), ("W7SEA", "WA"),
            // West Virginia (WV)
            ("K8WV", "WV"), ("W8WVA", "WV"),
            // Wisconsin (WI)
            ("K9WI", "WI"), ("W9WIS", "WI"),
            // Wyoming (WY)
            ("K7WY", "WY"), ("W7WYO", "WY"),
        ];
        
        // Verify we have exactly 100 callsigns
        assert_eq!(test_callsigns.len(), 100, "Should have exactly 100 test callsigns");
        
        // Setup database with all test callsigns
        let pool = setup_test_db_with_callsigns(&test_callsigns).await;
        
        // Test each callsign individually
        let mut passed = 0;
        let mut failed = 0;
        let mut failures: Vec<String> = Vec::new();
        
        for (call, expected_state) in &test_callsigns {
            let result = lookup_callsign(&pool, call).await;
            
            match result {
                Some(info) => {
                    if info.state.as_deref() == Some(*expected_state) {
                        passed += 1;
                    } else {
                        failed += 1;
                        failures.push(format!(
                            "{}: expected {}, got {:?}",
                            call, expected_state, info.state
                        ));
                    }
                }
                None => {
                    failed += 1;
                    failures.push(format!("{}: lookup returned None", call));
                }
            }
        }
        
        // Print summary
        println!("\n=== FCC State Lookup Test Results ===");
        println!("Passed: {}/100", passed);
        println!("Failed: {}/100", failed);
        
        if !failures.is_empty() {
            println!("\nFailures:");
            for f in &failures {
                println!("  {}", f);
            }
        }
        
        // All 100 should pass
        assert_eq!(passed, 100, "All 100 callsigns should have correct state lookups");
        assert_eq!(failed, 0, "No failures expected");
    }
    
    /// Test: Batch lookup with 100 callsigns at once
    #[tokio::test]
    async fn test_batch_lookup_100_callsigns() {
        let test_callsigns: Vec<(&str, &str)> = vec![
            ("K4AB", "AL"), ("W4ABC", "AL"),
            ("KL7AA", "AK"), ("AL7AB", "AK"),
            ("K7AZ", "AZ"), ("W7AZA", "AZ"),
            ("K5AR", "AR"), ("W5ARK", "AR"),
            ("K6CA", "CA"), ("W6CAL", "CA"),
            ("K0CO", "CO"), ("W0COL", "CO"),
            ("K1CT", "CT"), ("W1AW", "CT"),
            ("K3DE", "DE"), ("W3DEL", "DE"),
            ("K4FL", "FL"), ("W4FLA", "FL"),
            ("K4GA", "GA"), ("W4ATL", "GA"),
            ("KH6HI", "HI"), ("WH6AB", "HI"),
            ("K7ID", "ID"), ("W7IDA", "ID"),
            ("K9IL", "IL"), ("W9CHI", "IL"),
            ("K9IN", "IN"), ("W9IND", "IN"),
            ("K0IA", "IA"), ("W0IOW", "IA"),
            ("K0KS", "KS"), ("W0KAN", "KS"),
            ("K4KY", "KY"), ("W4KEN", "KY"),
            ("K5LA", "LA"), ("W5NOR", "LA"),
            ("K1ME", "ME"), ("W1MAI", "ME"),
            ("K3MD", "MD"), ("W3BAL", "MD"),
            ("K1MA", "MA"), ("W1BOS", "MA"),
            ("K8MI", "MI"), ("W8DET", "MI"),
            ("K0MN", "MN"), ("W0MIN", "MN"),
            ("K5MS", "MS"), ("W5MIS", "MS"),
            ("K0MO", "MO"), ("W0STL", "MO"),
            ("K7MT", "MT"), ("W7MON", "MT"),
            ("K0NE", "NE"), ("W0NEB", "NE"),
            ("K7NV", "NV"), ("W7VEG", "NV"),
            ("K1NH", "NH"), ("W1HAM", "NH"),
            ("K2NJ", "NJ"), ("W2JER", "NJ"),
            ("K5NM", "NM"), ("W5ABQ", "NM"),
            ("K2NY", "NY"), ("W2NYC", "NY"),
            ("K4NC", "NC"), ("W4CAR", "NC"),
            ("K0ND", "ND"), ("W0NDA", "ND"),
            ("K8OH", "OH"), ("W8OHI", "OH"),
            ("K5OK", "OK"), ("W5OKL", "OK"),
            ("K7OR", "OR"), ("W7ORE", "OR"),
            ("K3PA", "PA"), ("W3PHI", "PA"),
            ("K1RI", "RI"), ("W1RHO", "RI"),
            ("K4SC", "SC"), ("W4SCA", "SC"),
            ("K0SD", "SD"), ("W0SDA", "SD"),
            ("K4TN", "TN"), ("W4NAS", "TN"),
            ("K5TX", "TX"), ("W5TEX", "TX"),
            ("K7UT", "UT"), ("W7SLC", "UT"),
            ("K1VT", "VT"), ("W1VER", "VT"),
            ("K4VA", "VA"), ("W4VIR", "VA"),
            ("K7WA", "WA"), ("W7SEA", "WA"),
            ("K8WV", "WV"), ("W8WVA", "WV"),
            ("K9WI", "WI"), ("W9WIS", "WI"),
            ("K7WY", "WY"), ("W7WYO", "WY"),
        ];
        
        let pool = setup_test_db_with_callsigns(&test_callsigns).await;
        
        // Batch lookup all 100 callsigns at once
        let calls: Vec<String> = test_callsigns.iter().map(|(c, _)| c.to_string()).collect();
        let results = lookup_callsigns(&pool, &calls).await;
        
        assert_eq!(results.len(), 100, "Batch lookup should return all 100 callsigns");
        
        // Verify each result has the correct state
        let expected_map: std::collections::HashMap<&str, &str> = 
            test_callsigns.iter().cloned().collect();
        
        for info in &results {
            let expected_state = expected_map.get(info.call.as_str());
            assert!(expected_state.is_some(), "Callsign {} not in expected map", info.call);
            assert_eq!(
                info.state.as_deref(), 
                Some(*expected_state.unwrap()),
                "State mismatch for {}", info.call
            );
        }
        
        println!("\n=== Batch Lookup Test Results ===");
        println!("All 100 callsigns returned with correct states");
    }
    
    /// Test: State code validation - all 50 states covered
    #[tokio::test]
    async fn test_all_50_states_covered() {
        use std::collections::HashSet;
        
        let test_callsigns: Vec<(&str, &str)> = vec![
            ("K4AB", "AL"), ("KL7AA", "AK"), ("K7AZ", "AZ"), ("K5AR", "AR"),
            ("K6CA", "CA"), ("K0CO", "CO"), ("K1CT", "CT"), ("K3DE", "DE"),
            ("K4FL", "FL"), ("K4GA", "GA"), ("KH6HI", "HI"), ("K7ID", "ID"),
            ("K9IL", "IL"), ("K9IN", "IN"), ("K0IA", "IA"), ("K0KS", "KS"),
            ("K4KY", "KY"), ("K5LA", "LA"), ("K1ME", "ME"), ("K3MD", "MD"),
            ("K1MA", "MA"), ("K8MI", "MI"), ("K0MN", "MN"), ("K5MS", "MS"),
            ("K0MO", "MO"), ("K7MT", "MT"), ("K0NE", "NE"), ("K7NV", "NV"),
            ("K1NH", "NH"), ("K2NJ", "NJ"), ("K5NM", "NM"), ("K2NY", "NY"),
            ("K4NC", "NC"), ("K0ND", "ND"), ("K8OH", "OH"), ("K5OK", "OK"),
            ("K7OR", "OR"), ("K3PA", "PA"), ("K1RI", "RI"), ("K4SC", "SC"),
            ("K0SD", "SD"), ("K4TN", "TN"), ("K5TX", "TX"), ("K7UT", "UT"),
            ("K1VT", "VT"), ("K4VA", "VA"), ("K7WA", "WA"), ("K8WV", "WV"),
            ("K9WI", "WI"), ("K7WY", "WY"),
        ];
        
        // Collect all unique states
        let states: HashSet<&str> = test_callsigns.iter().map(|(_, s)| *s).collect();
        
        assert_eq!(states.len(), 50, "Should cover all 50 US states");
        
        // Verify we have all state codes
        let expected_states: HashSet<&str> = [
            "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA",
            "HI", "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD",
            "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
            "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
            "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY",
        ].iter().cloned().collect();
        
        assert_eq!(states, expected_states, "All 50 US states should be covered");
        
        println!("\n=== State Coverage Test ===");
        println!("All 50 US states are covered in test data");
    }
}
