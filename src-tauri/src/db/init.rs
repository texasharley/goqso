// Database initialization and migration handling
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Row};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use crate::db::migrations::MIGRATION_001;
use crate::reference::{dxcc, prefixes};

/// Get the database path in the app data directory
pub fn get_db_path(app: &AppHandle) -> PathBuf {
    let app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
    std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");
    app_dir.join("goqso.db")
}

/// Initialize the database connection pool
pub async fn init_db(app: &AppHandle) -> Result<Pool<Sqlite>, String> {
    let db_path = get_db_path(app);
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
    
    log::info!("Initializing database at: {}", db_path.display());
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    
    // Enable WAL mode for better performance
    sqlx::query("PRAGMA journal_mode=WAL;")
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to enable WAL mode: {}", e))?;
    
    // Run migrations
    run_migrations(&pool).await?;
    
    // Populate reference data if needed
    populate_reference_data(&pool).await?;
    
    log::info!("Database initialization complete");
    
    Ok(pool)
}

/// Run all pending migrations
async fn run_migrations(pool: &Pool<Sqlite>) -> Result<(), String> {
    // Create migrations table if not exists
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL
        )"
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create migrations table: {}", e))?;
    
    // Check if migration 001 has been applied
    let applied: bool = sqlx::query("SELECT COUNT(*) as count FROM _migrations WHERE name = 'migration_001'")
        .fetch_one(pool)
        .await
        .map(|row| row.get::<i64, _>("count") > 0)
        .unwrap_or(false);
    
    if !applied {
        log::info!("Applying migration_001...");
        
        // Split migration into individual statements (SQLite doesn't support multiple statements)
        for statement in MIGRATION_001.split(';') {
            // Strip leading comments and whitespace
            let mut stmt = statement.trim();
            while stmt.starts_with("--") {
                // Skip to next line
                if let Some(idx) = stmt.find('\n') {
                    stmt = stmt[idx + 1..].trim();
                } else {
                    stmt = "";
                    break;
                }
            }
            
            if !stmt.is_empty() {
                sqlx::query(stmt)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("Migration failed on statement: {}\nError: {}", stmt, e))?;
            }
        }
        
        // Mark migration as applied
        sqlx::query("INSERT INTO _migrations (name, applied_at) VALUES ('migration_001', datetime('now'))")
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to record migration: {}", e))?;
        
        log::info!("Migration 001 applied successfully");
    }
    
    Ok(())
}

/// Populate DXCC entities and prefixes from our reference data
async fn populate_reference_data(pool: &Pool<Sqlite>) -> Result<(), String> {
    // Check if reference data is already populated
    let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM dxcc_entities")
        .fetch_one(pool)
        .await
        .map(|row| row.get("count"))
        .unwrap_or(0);
    
    if count > 0 {
        log::info!("Reference data already populated ({} DXCC entities)", count);
        return Ok(());
    }
    
    log::info!("Populating reference data...");
    
    // Insert DXCC entities
    for entity in dxcc::DXCC_ENTITIES.iter() {
        sqlx::query(
            "INSERT OR IGNORE INTO dxcc_entities (entity_code, entity_name, cq_zone, itu_zone, continent, is_deleted) 
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(entity.entity_id as i64)
        .bind(entity.name)
        .bind(entity.cq_zone as i64)
        .bind(entity.itu_zone as i64)
        .bind(entity.continent)
        .bind(if entity.deleted { 1 } else { 0 })
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to insert DXCC entity {}: {}", entity.name, e))?;
    }
    
    // Insert prefix rules
    for rule in prefixes::PREFIX_RULES.iter() {
        sqlx::query(
            "INSERT OR IGNORE INTO callsign_prefixes (prefix, entity_code, is_exact) VALUES (?, ?, ?)"
        )
        .bind(rule.prefix)
        .bind(rule.entity_id as i64)
        .bind(if rule.exact { 1 } else { 0 })
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to insert prefix {}: {}", rule.prefix, e))?;
    }
    
    // Update reference data version
    sqlx::query(
        "INSERT OR REPLACE INTO reference_data_version (source, version, updated_at) 
         VALUES ('goqso_internal', '2025.01', datetime('now'))"
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update reference version: {}", e))?;
    
    log::info!("Reference data populated: {} entities, {} prefixes", 
        dxcc::DXCC_ENTITIES.len(), 
        prefixes::PREFIX_RULES.len()
    );
    
    Ok(())
}

/// Get database stats for debugging
pub async fn get_db_stats(pool: &Pool<Sqlite>) -> Result<DbStats, String> {
    let qso_count: i64 = sqlx::query("SELECT COUNT(*) as count FROM qsos")
        .fetch_one(pool)
        .await
        .map(|row| row.get("count"))
        .unwrap_or(0);
    
    let entity_count: i64 = sqlx::query("SELECT COUNT(*) as count FROM dxcc_entities")
        .fetch_one(pool)
        .await
        .map(|row| row.get("count"))
        .unwrap_or(0);
    
    let prefix_count: i64 = sqlx::query("SELECT COUNT(*) as count FROM callsign_prefixes")
        .fetch_one(pool)
        .await
        .map(|row| row.get("count"))
        .unwrap_or(0);
    
    Ok(DbStats {
        qso_count,
        entity_count,
        prefix_count,
    })
}

#[derive(Debug, serde::Serialize)]
pub struct DbStats {
    pub qso_count: i64,
    pub entity_count: i64,
    pub prefix_count: i64,
}
