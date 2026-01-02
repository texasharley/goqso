/// SQL migration for initial database schema
pub const MIGRATION_001: &str = r#"
-- QSO Log (ADIF 3.1.4 compatible)
CREATE TABLE IF NOT EXISTS qsos (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid            TEXT NOT NULL UNIQUE,
    
    -- Required fields
    call            TEXT NOT NULL,
    qso_date        TEXT NOT NULL,
    time_on         TEXT NOT NULL,
    band            TEXT NOT NULL,
    mode            TEXT NOT NULL,
    
    -- Frequency
    freq            REAL,
    freq_rx         REAL,
    
    -- Location data (from CTY lookup)
    dxcc            INTEGER,
    country         TEXT,
    state           TEXT,
    gridsquare      TEXT,
    cqz             INTEGER,
    ituz            INTEGER,
    
    -- Signal reports
    rst_sent        TEXT,
    rst_rcvd        TEXT,
    
    -- Station info
    station_callsign TEXT,
    my_gridsquare   TEXT,
    tx_pwr          REAL,
    
    -- Metadata
    source          TEXT DEFAULT 'manual',
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    
    UNIQUE(call, qso_date, time_on, band, mode)
);

CREATE INDEX IF NOT EXISTS idx_qsos_call ON qsos(call);
CREATE INDEX IF NOT EXISTS idx_qsos_date ON qsos(qso_date);
CREATE INDEX IF NOT EXISTS idx_qsos_dxcc ON qsos(dxcc);
CREATE INDEX IF NOT EXISTS idx_qsos_state ON qsos(state);
CREATE INDEX IF NOT EXISTS idx_qsos_grid ON qsos(gridsquare);

-- Confirmations (from LoTW, eQSL, etc.)
CREATE TABLE IF NOT EXISTS confirmations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    qso_id          INTEGER NOT NULL REFERENCES qsos(id) ON DELETE CASCADE,
    source          TEXT NOT NULL,
    confirmed_at    TEXT NOT NULL,
    lotw_qsl_date   TEXT,
    lotw_credit_granted TEXT,
    
    UNIQUE(qso_id, source)
);

-- Sync queue for offline operation
CREATE TABLE IF NOT EXISTS sync_queue (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    qso_id          INTEGER NOT NULL REFERENCES qsos(id) ON DELETE CASCADE,
    action          TEXT NOT NULL,
    status          TEXT DEFAULT 'pending',
    attempts        INTEGER DEFAULT 0,
    last_error      TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sync_queue_status ON sync_queue(status);

-- Award progress tracking
CREATE TABLE IF NOT EXISTS award_progress (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    award_type      TEXT NOT NULL,
    target_id       TEXT NOT NULL,
    band            TEXT,
    mode            TEXT,
    worked_qso_id   INTEGER REFERENCES qsos(id) ON DELETE SET NULL,
    confirmed_qso_id INTEGER REFERENCES qsos(id) ON DELETE SET NULL,
    credited        INTEGER DEFAULT 0,
    updated_at      TEXT NOT NULL,
    
    UNIQUE(award_type, target_id, band, mode)
);

CREATE INDEX IF NOT EXISTS idx_award_progress_type ON award_progress(award_type);

-- App settings
CREATE TABLE IF NOT EXISTS settings (
    key             TEXT PRIMARY KEY,
    value           TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

-- LoTW sync state
CREATE TABLE IF NOT EXISTS lotw_sync_state (
    id              INTEGER PRIMARY KEY CHECK (id = 1),
    last_qsl_date   TEXT,
    last_qso_rx     TEXT,
    last_sync_at    TEXT,
    updated_at      TEXT NOT NULL
);

-- Initialize LoTW sync state singleton
INSERT OR IGNORE INTO lotw_sync_state (id, updated_at) VALUES (1, datetime('now'));

-- DXCC entities reference table
CREATE TABLE IF NOT EXISTS dxcc_entities (
    entity_code     INTEGER PRIMARY KEY,
    entity_name     TEXT NOT NULL,
    cq_zone         INTEGER,
    itu_zone        INTEGER,
    continent       TEXT,
    latitude        REAL,
    longitude       REAL,
    utc_offset      REAL,
    is_deleted      INTEGER DEFAULT 0
);

-- Callsign prefix lookup
CREATE TABLE IF NOT EXISTS callsign_prefixes (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    prefix          TEXT NOT NULL,
    entity_code     INTEGER NOT NULL REFERENCES dxcc_entities(entity_code),
    cq_zone         INTEGER,
    itu_zone        INTEGER,
    is_exact        INTEGER DEFAULT 0,
    
    UNIQUE(prefix, entity_code)
);

CREATE INDEX IF NOT EXISTS idx_prefixes_prefix ON callsign_prefixes(prefix);

-- US States reference
CREATE TABLE IF NOT EXISTS us_states (
    abbrev          TEXT PRIMARY KEY,
    name            TEXT NOT NULL
);

-- Insert US states
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

-- Reference data version tracking
CREATE TABLE IF NOT EXISTS reference_data_version (
    source          TEXT PRIMARY KEY,
    version         TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);
"#;

/// Run all migrations
pub fn get_migrations() -> Vec<&'static str> {
    vec![MIGRATION_001]
}
