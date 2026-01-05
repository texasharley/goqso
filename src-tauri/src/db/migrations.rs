/// SQL migration for initial database schema
/// 
/// Design principles:
/// - Columns for frequently queried/indexed fields
/// - JSON blobs for flexible/rarely-queried fields
/// - snake_case naming (map to ADIF on import/export)
/// - Normalized confirmations (one row per source)
pub const MIGRATION_001: &str = r#"
-- =============================================================================
-- QSO Log - Core table with hybrid column/JSON approach
-- =============================================================================
CREATE TABLE IF NOT EXISTS qsos (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid            TEXT NOT NULL UNIQUE,
    
    -- CORE FIELDS (indexed, frequently queried)
    -- These drive dupe checking, award tracking, filtering
    call            TEXT NOT NULL,
    qso_date        TEXT NOT NULL,          -- YYYY-MM-DD
    time_on         TEXT NOT NULL,          -- HHMM or HHMMSS
    time_off        TEXT,                   -- HHMM or HHMMSS
    band            TEXT NOT NULL,          -- e.g., "20m"
    mode            TEXT NOT NULL,          -- e.g., "FT8"
    freq            REAL,                   -- MHz
    
    -- LOCATION (for award tracking)
    dxcc            INTEGER,                -- DXCC entity number
    country         TEXT,                   -- Entity name
    state           TEXT,                   -- US state/CA province
    cnty            TEXT,                   -- County (ARRL format: ST,County)
    gridsquare      TEXT,                   -- Maidenhead grid
    continent       TEXT,                   -- e.g., "NA", "EU"
    cqz             INTEGER,                -- CQ Zone
    ituz            INTEGER,                -- ITU Zone
    
    -- SIGNAL REPORTS
    rst_sent        TEXT,
    rst_rcvd        TEXT,
    
    -- MY STATION (for multi-station/portable ops)
    station_callsign TEXT,                  -- My callsign used
    my_gridsquare   TEXT,                   -- My grid
    tx_pwr          REAL,                   -- Watts
    
    -- EXTENDED ADIF FIELDS (JSON - flexible, rarely queried individually)
    -- Stores: name, qth, comments, prop_mode, sota_ref, pota_ref, iota, 
    --         wwff_ref, rig, antenna, operator, contest_id, srx, stx, etc.
    adif_fields     TEXT DEFAULT '{}',
    
    -- USER-DEFINED DATA (JSON - completely flexible)
    user_data       TEXT DEFAULT '{}',
    
    -- METADATA
    source          TEXT DEFAULT 'manual',  -- WSJT-X, ADIF, manual
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    
    UNIQUE(call, qso_date, time_on, band, mode)
);

-- Performance indexes for common operations
CREATE INDEX IF NOT EXISTS idx_qsos_call ON qsos(call);
CREATE INDEX IF NOT EXISTS idx_qsos_date ON qsos(qso_date DESC);
CREATE INDEX IF NOT EXISTS idx_qsos_dupe ON qsos(call, band, mode, qso_date);
CREATE INDEX IF NOT EXISTS idx_qsos_dxcc ON qsos(dxcc, band, mode);
CREATE INDEX IF NOT EXISTS idx_qsos_state ON qsos(state, band, mode);
CREATE INDEX IF NOT EXISTS idx_qsos_grid ON qsos(gridsquare, band);

-- =============================================================================
-- Confirmations - Normalized (one row per QSO per source)
-- =============================================================================
CREATE TABLE IF NOT EXISTS confirmations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    qso_id          INTEGER NOT NULL REFERENCES qsos(id) ON DELETE CASCADE,
    source          TEXT NOT NULL,          -- LOTW, EQSL, QRZ, CLUBLOG, CARD
    
    -- QSL tracking
    qsl_sent        TEXT,                   -- Y, N, R (requested), Q (queued)
    qsl_sent_date   TEXT,
    qsl_rcvd        TEXT,                   -- Y, N, R, etc.
    qsl_rcvd_date   TEXT,
    
    -- LoTW specific
    credit_granted  TEXT,                   -- DXCC, WAS, etc.
    credit_submitted TEXT,
    
    -- Metadata
    verified_at     TEXT,
    raw_data        TEXT,                   -- Original confirmation data (JSON)
    
    UNIQUE(qso_id, source)
);

CREATE INDEX IF NOT EXISTS idx_confirmations_qso ON confirmations(qso_id);
CREATE INDEX IF NOT EXISTS idx_confirmations_source ON confirmations(source, qsl_rcvd);

-- =============================================================================
-- Sync Queue - For offline operation
-- =============================================================================
CREATE TABLE IF NOT EXISTS sync_queue (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    qso_id          INTEGER NOT NULL REFERENCES qsos(id) ON DELETE CASCADE,
    target          TEXT NOT NULL,          -- LOTW, EQSL, etc.
    action          TEXT NOT NULL,          -- upload, delete
    status          TEXT DEFAULT 'pending', -- pending, processing, completed, failed
    attempts        INTEGER DEFAULT 0,
    last_error      TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sync_queue_status ON sync_queue(status, target);

-- =============================================================================
-- Award Progress - Denormalized for fast lookups
-- =============================================================================
CREATE TABLE IF NOT EXISTS award_progress (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    award_type      TEXT NOT NULL,          -- DXCC, WAS, VUCC, WAZ
    target_id       TEXT NOT NULL,          -- Entity code, state abbrev, grid, zone
    band            TEXT,                   -- NULL = mixed
    mode            TEXT,                   -- NULL = mixed
    
    -- First QSO that worked/confirmed this target
    worked_qso_id   INTEGER REFERENCES qsos(id) ON DELETE SET NULL,
    confirmed_qso_id INTEGER REFERENCES qsos(id) ON DELETE SET NULL,
    
    -- Tracking
    worked_date     TEXT,
    confirmed_date  TEXT,
    credited        INTEGER DEFAULT 0,      -- Credit applied to award
    
    updated_at      TEXT NOT NULL,
    
    UNIQUE(award_type, target_id, band, mode)
);

CREATE INDEX IF NOT EXISTS idx_award_progress_type ON award_progress(award_type, credited);

-- =============================================================================
-- App Settings
-- =============================================================================
CREATE TABLE IF NOT EXISTS settings (
    key             TEXT PRIMARY KEY,
    value           TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

-- =============================================================================
-- LoTW Sync State
-- =============================================================================
CREATE TABLE IF NOT EXISTS lotw_sync_state (
    id              INTEGER PRIMARY KEY CHECK (id = 1),
    last_qsl_date   TEXT,                   -- Last QSL date downloaded
    last_qso_rx     TEXT,                   -- Last QSO received timestamp
    last_upload_at  TEXT,
    last_download_at TEXT,
    updated_at      TEXT NOT NULL
);

INSERT OR IGNORE INTO lotw_sync_state (id, updated_at) VALUES (1, datetime('now'));

-- =============================================================================
-- Reference Data - DXCC Entities
-- =============================================================================
CREATE TABLE IF NOT EXISTS dxcc_entities (
    entity_code     INTEGER PRIMARY KEY,
    entity_name     TEXT NOT NULL,
    prefix          TEXT,                   -- Primary prefix
    continent       TEXT,
    cq_zone         INTEGER,
    itu_zone        INTEGER,
    latitude        REAL,
    longitude       REAL,
    utc_offset      REAL,
    is_deleted      INTEGER DEFAULT 0,
    notes           TEXT
);

-- =============================================================================
-- Reference Data - Callsign Prefixes
-- =============================================================================
CREATE TABLE IF NOT EXISTS callsign_prefixes (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    prefix          TEXT NOT NULL,
    entity_code     INTEGER NOT NULL REFERENCES dxcc_entities(entity_code),
    cq_zone         INTEGER,
    itu_zone        INTEGER,
    continent       TEXT,
    is_exact        INTEGER DEFAULT 0,      -- Exact match vs prefix match
    
    UNIQUE(prefix, entity_code)
);

CREATE INDEX IF NOT EXISTS idx_prefixes_prefix ON callsign_prefixes(prefix);

-- =============================================================================
-- Reference Data - US States
-- =============================================================================
CREATE TABLE IF NOT EXISTS us_states (
    abbrev          TEXT PRIMARY KEY,
    name            TEXT NOT NULL
);

INSERT OR IGNORE INTO us_states VALUES 
    ('AL', 'Alabama'), ('AK', 'Alaska'), ('AZ', 'Arizona'), ('AR', 'Arkansas'),
    ('CA', 'California'), ('CO', 'Colorado'), ('CT', 'Connecticut'), ('DE', 'Delaware'),
    ('FL', 'Florida'), ('GA', 'Georgia'), ('HI', 'Hawaii'), ('ID', 'Idaho'),
    ('IL', 'Illinois'), ('IN', 'Indiana'), ('IA', 'Iowa'), ('KS', 'Kansas'),
    ('KY', 'Kentucky'), ('LA', 'Louisiana'), ('ME', 'Maine'), ('MD', 'Maryland'),
    ('MA', 'Massachusetts'), ('MI', 'Michigan'), ('MN', 'Minnesota'), ('MS', 'Mississippi'),
    ('MO', 'Missouri'), ('MT', 'Montana'), ('NE', 'Nebraska'), ('NV', 'Nevada'),
    ('NH', 'New Hampshire'), ('NJ', 'New Jersey'), ('NM', 'New Mexico'), ('NY', 'New York'),
    ('NC', 'North Carolina'), ('ND', 'North Dakota'), ('OH', 'Ohio'), ('OK', 'Oklahoma'),
    ('OR', 'Oregon'), ('PA', 'Pennsylvania'), ('RI', 'Rhode Island'), ('SC', 'South Carolina'),
    ('SD', 'South Dakota'), ('TN', 'Tennessee'), ('TX', 'Texas'), ('UT', 'Utah'),
    ('VT', 'Vermont'), ('VA', 'Virginia'), ('WA', 'Washington'), ('WV', 'West Virginia'),
    ('WI', 'Wisconsin'), ('WY', 'Wyoming');

-- =============================================================================
-- Reference Data Version Tracking
-- =============================================================================
CREATE TABLE IF NOT EXISTS reference_versions (
    source          TEXT PRIMARY KEY,       -- dxcc, prefixes, etc.
    version         TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);
"#;

/// Migration 002: Add missing columns to qsos table
/// 
/// The original schema was deployed without these columns that are needed for:
/// - County tracking (WAS county hunting, LoTW sync)
/// - POTA/SOTA/WWFF activations
/// - Submode tracking (FT4 vs FT8, etc.)
/// - Propagation mode (EME, SAT, etc.)
/// - Operator name and QTH (common searches)
/// - Comments
/// 
/// Design rationale:
/// - cnty: Column (required for WAS county awards, LoTW returns it)
/// - submode: Column (needed to distinguish FT4/FT8, LSB/USB)
/// - prop_mode: Column (EME/SAT filtering for VUCC, WAZ EME endorsements)
/// - pota_ref, sota_ref, wwff_ref: Columns (activation logging, becoming very popular)
/// - iota: Column (IOTA awards program)
/// - name, qth, comment: Columns (frequently searched/displayed)
/// - pfx: Column (WPX prefix for WPX award)
/// - sat_name: Column (satellite ops)
pub const MIGRATION_002: &str = r#"
-- Add missing county column (critical for WAS county hunting and LoTW)
ALTER TABLE qsos ADD COLUMN cnty TEXT;

-- Add submode (e.g., FT4 vs FT8, LSB vs USB)
ALTER TABLE qsos ADD COLUMN submode TEXT;

-- Add propagation mode (EME, SAT, ES, etc.)
ALTER TABLE qsos ADD COLUMN prop_mode TEXT;

-- Add satellite name
ALTER TABLE qsos ADD COLUMN sat_name TEXT;

-- Add special activity references (increasingly popular)
ALTER TABLE qsos ADD COLUMN pota_ref TEXT;
ALTER TABLE qsos ADD COLUMN sota_ref TEXT;
ALTER TABLE qsos ADD COLUMN wwff_ref TEXT;
ALTER TABLE qsos ADD COLUMN iota TEXT;

-- Add WPX prefix
ALTER TABLE qsos ADD COLUMN pfx TEXT;

-- Add frequently searched/displayed fields
ALTER TABLE qsos ADD COLUMN name TEXT;
ALTER TABLE qsos ADD COLUMN qth TEXT;
ALTER TABLE qsos ADD COLUMN comment TEXT;

-- Add ARRL section (for Sweepstakes, Field Day)
ALTER TABLE qsos ADD COLUMN arrl_sect TEXT;

-- Add my county and my ARRL section for portable/rover ops
ALTER TABLE qsos ADD COLUMN my_cnty TEXT;
ALTER TABLE qsos ADD COLUMN my_arrl_sect TEXT;
ALTER TABLE qsos ADD COLUMN my_sota_ref TEXT;
ALTER TABLE qsos ADD COLUMN my_pota_ref TEXT;

-- Index for county award hunting
CREATE INDEX IF NOT EXISTS idx_qsos_cnty ON qsos(cnty);

-- Index for activity references
CREATE INDEX IF NOT EXISTS idx_qsos_pota ON qsos(pota_ref);
CREATE INDEX IF NOT EXISTS idx_qsos_sota ON qsos(sota_ref);

-- Index for propagation mode (EME/SAT filtering)
CREATE INDEX IF NOT EXISTS idx_qsos_prop ON qsos(prop_mode);
"#;

/// Migration 003: Add qso_date_off and operator columns
/// 
/// qso_date_off: Some QSOs span midnight (especially EME, contest DXpeditions)
///               Also returned by LoTW and expected in ADIF exports
/// 
/// operator: The person operating (distinct from station_callsign)
///           Used for multi-op stations, club stations, guest ops
///           Required for accurate LoTW signing
pub const MIGRATION_003: &str = r#"
-- Add QSO end date (for QSOs spanning midnight)
ALTER TABLE qsos ADD COLUMN qso_date_off TEXT;

-- Add operator callsign (distinct from station callsign)
-- This is who is operating, not what callsign is being used
ALTER TABLE qsos ADD COLUMN operator TEXT;
"#;

/// Run all migrations
pub fn get_migrations() -> Vec<&'static str> {
    vec![MIGRATION_001, MIGRATION_002, MIGRATION_003]
}
