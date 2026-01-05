# GoQSO Architecture Guide

> **Purpose**: This file documents the complete architecture of GoQSO to maintain context across sessions.

## Project Overview

## Instructions for Claude:
Always read .github/copilot-instructions.md before doing anything!!!

**GoQSO** is an offline-first FT8 auto-logger with LoTW sync and ARRL awards tracking. It automates logging FT8 QSOs from WSJT-X with automatic DXCC/state lookup and LoTW integration.

## Tech Stack

- **Frontend**: React 18 + TypeScript + Tailwind CSS + Vite
- **Backend**: Tauri 2.x with Rust
- **Database**: SQLite via sqlx (NOT tauri-plugin-sql for data operations)
- **HTTP**: reqwest for LoTW API calls

## Directory Structure

```
goqso/
├── src/                      # React frontend
│   ├── components/           # UI components
│   ├── hooks/                # React hooks
│   ├── lib/                  # Tauri API wrappers
│   ├── stores/               # Zustand stores
│   └── types/                # TypeScript types
├── src-tauri/
│   └── src/
│       ├── adif/             # ADIF parser/writer + mode registry
│       ├── awards/           # Award progress calculations
│       ├── db/               # Database init, migrations, schema
│       ├── lotw/             # LoTW HTTP client
│       ├── reference/        # DXCC entities, prefixes, US states
│       ├── udp/              # WSJT-X UDP listener
│       ├── commands.rs       # All Tauri commands
│       └── main.rs           # App entry point
└── .github/
    └── copilot-instructions.md  # Dev environment instructions
```

## Database Location

```
Windows: %APPDATA%\com.goqso.app\goqso.db
macOS:   ~/Library/Application Support/com.goqso.app/goqso.db
Linux:   ~/.local/share/com.goqso.app/goqso.db
```

## Database Schema

### Design Philosophy

The schema follows a **hybrid column/JSON approach**:
- **Columns**: Frequently queried/filtered fields, award tracking fields, indexed lookups
- **JSON (adif_fields)**: Rarely queried ADIF fields, contest data, equipment info
- **JSON (user_data)**: User-defined custom fields

Based on:
- [ADIF 3.1.4 Specification](https://www.adif.org/314/ADIF_314.htm)
- [LoTW Developer API](https://lotw.arrl.org/lotw-help/developer-query-qsos-qsls/)
- DXKeeper field comparison

### Core Tables

1. **qsos** - Main QSO log (Migration 001 + 002)
   
   **Core fields (always columns, indexed):**
   - `call`, `qso_date`, `time_on`, `time_off`, `band`, `mode`, `submode`, `freq`
   
   **Location fields (for award tracking):**
   - `dxcc` - DXCC entity code (INTEGER)
   - `country` - Entity name
   - `state` - US state/CA province (WAS award)
   - `cnty` - US County (WAS county hunting, LoTW sync)
   - `gridsquare` - Maidenhead grid (VUCC)
   - `continent` - NA, EU, etc.
   - `cqz` - CQ Zone (WAZ award)
   - `ituz` - ITU Zone
   - `iota` - Islands on the Air reference
   - `pfx` - WPX prefix (WPX award)
   - `arrl_sect` - ARRL section (contests)
   
   **Special activity references (popular awards):**
   - `pota_ref` - Parks on the Air
   - `sota_ref` - Summits on the Air
   - `wwff_ref` - World Wide Flora Fauna
   
   **Propagation:**
   - `prop_mode` - EME, SAT, ES, etc.
   - `sat_name` - Satellite name
   
   **Operator info (frequently searched):**
   - `name` - Operator name
   - `qth` - Location/city
   - `comment` - Comments
   
   **Signal reports:**
   - `rst_sent`, `rst_rcvd`
   
   **My station (for portable/rover ops):**
   - `station_callsign`, `my_gridsquare`, `tx_pwr`
   - `my_cnty`, `my_arrl_sect`, `my_sota_ref`, `my_pota_ref`
   
   **Flexible JSON:**
   - `adif_fields` - Extended ADIF fields (rig, antenna, contest_id, srx, stx, etc.)
   - `user_data` - User-defined custom fields
   
   **Metadata:**
   - `source` - WSJT-X, ADIF, manual
   - `created_at`, `updated_at`
   
   **Unique constraint**: (call, qso_date, time_on, band, mode)
   
   **Indexes**: call, date DESC, dupe check, dxcc, state, grid, county, pota, sota, prop_mode

2. **confirmations** - QSL confirmations (normalized, one row per QSO per source)
   - Links to qso_id
   - Sources: LOTW, EQSL, QRZ, CLUBLOG, CARD
   - Tracks: qsl_sent/rcvd, dates, credit_granted

3. **sync_queue** - Offline sync queue
   - Tracks pending uploads to LoTW/eQSL
   - Status: pending, processing, completed, failed

4. **award_progress** - Denormalized award tracking
   - Award types: DXCC, WAS, VUCC, WAZ
   - Tracks worked/confirmed status per band/mode

5. **settings** - Key/value app settings

6. **lotw_sync_state** - LoTW sync metadata

7. **dxcc_entities** - Reference data (340 entities)

8. **callsign_prefixes** - Prefix to DXCC lookup (326 rules)

9. **reference_data_version** - Tracks reference data updates

### Migrations

- **MIGRATION_001**: Initial schema (qsos, confirmations, sync_queue, etc.)
- **MIGRATION_002**: Adds missing columns (cnty, submode, prop_mode, sat_name, pota_ref, sota_ref, wwff_ref, iota, pfx, name, qth, comment, arrl_sect, my_cnty, my_arrl_sect, my_sota_ref, my_pota_ref)

## Key Tauri Commands

### QSO Operations
- `get_qsos` - Fetch QSOs with pagination/filtering
- `add_qso` - Add new QSO
- `update_qso` - Update existing QSO
- `delete_qso` - Delete single QSO
- `clear_all_qsos` - Delete ALL QSOs (testing)
- `add_test_qsos` - Add synthetic test data

### ADIF Import/Export
- `import_adif(content, skip_duplicates)` - Parse ADIF string and import
- `export_adif` - Export QSOs to ADIF format

### LoTW Integration
- `sync_lotw_download(username, password, since_date)` - Download confirmations
- `get_sync_status` - Get pending uploads, last sync dates
- `detect_tqsl_path` - Find TQSL installation
- `import_lotw_confirmations` - Process downloaded confirmations

### Awards
- `get_dxcc_progress` - DXCC worked/confirmed counts
- `get_was_progress` - WAS state progress

### Reference Data
- `lookup_callsign` - Get DXCC entity from callsign prefix

### Settings
- `get_setting(key)` / `set_setting(key, value)`

## LoTW API Integration

### Endpoints (GET only - no uploads yet)
- `lotwreport.adi` - Download QSL confirmations
- `qslcards.php` - Download DXCC credits
- `lotw-user-activity.csv` - Check if callsign is LoTW user

### Client Location
`src-tauri/src/lotw/client.rs` - LotwClient with reqwest

## Reference Data Philosophy

- **Do NOT use CTY.DAT** from country-files.com
- Use curated DXCC data in `src-tauri/src/reference/`
- Source from ARRL official lists and ITU allocations
- LoTW confirmations are ground truth for DXCC credit

## ADIF Parser

Location: `src-tauri/src/adif/parser.rs`

- Parses ADIF files (handles header and records)
- Extracts fields via `<FIELD:length>value` format
- Case-insensitive field names

## Development Commands

```powershell
# Start dev server (use external window per copilot-instructions.md)
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd c:\dev\qso-logger\goqso; npm run tauri dev"

# Build for production
npm run tauri build

# Rust tests
cd src-tauri; cargo test

# Frontend build check
npm run build
```

## Important Notes

1. **Database initialization happens async** on app startup
2. **Reference data is populated once** on first run (~340 DXCC + 326 prefixes)
3. **WAL mode** is enabled for better SQLite performance
4. **Duplicate detection** uses: call + qso_date + time_on + band + mode

## Current State (as of last session)

- ✅ Core QSO logging working
- ✅ WSJT-X UDP listener working
- ✅ DXCC/State lookup working
- ✅ ADIF import/export working
- ✅ LoTW download client implemented
- ✅ Import UI (AdifImport.tsx) created
- ✅ LoTW Sync UI (LotwSync.tsx) created
- ✅ Lazy loading for QsoLog and AwardsMatrix tabs
- ✅ Combined single-query for get_sync_status
- ⏳ LoTW upload not implemented (need TQSL signing)
- ⏳ Awards matrix needs real data integration

## Performance Architecture

**See [PERFORMANCE.md](PERFORMANCE.md) for complete performance guide.**

Key principles:
1. No blocking startup operations
2. Single source of truth for DB state
3. Lazy load heavy components
4. Combine backend queries (reduce round trips)
5. Virtual scroll for large lists
6. Measure before optimizing

Dev mode is slow (15-25s) due to Rust recompilation. Production builds are fast (1-2s).

## User's Real QSO Data

- Location: `C:\DXLab\DXKeeper\LotWUpload2.ADI`
- Contains ~50+ real FT8 QSOs from DXKeeper
- Already submitted to LoTW

## GitHub Repository

https://github.com/texasharley/goqso

## Official Documentation References

### ADIF Specification
- **ADIF 3.1.4** (current): https://www.adif.org/314/ADIF_314.htm
- Field definitions, data types, enumerations
- Our schema maps directly to ADIF field names for seamless import/export

### LoTW (Logbook of The World)
- **Developer API**: https://lotw.arrl.org/lotw-help/developer-query-qsos-qsls/
- QSO/QSL query parameters and response format
- Confirmation download fields match our schema columns

### ARRL Awards Programs
- **DXCC**: https://www.arrl.org/dxcc
- **WAS (Worked All States)**: https://www.arrl.org/was
- **VUCC (VHF/UHF Century Club)**: https://www.arrl.org/vucc

### Portable/Field Operations
- **POTA**: https://pota.app/
- **SOTA**: https://www.sota.org.uk/
- **WWFF**: https://wwff.co/

### Other Resources
- **IOTA (Islands on the Air)**: https://www.iota-world.org/
- **eQSL**: https://www.eqsl.cc/
- **QRZ Logbook API**: https://www.qrz.com/docs/logbook/QRZLogbookAPI.html

## ADIF Field Data Model (Per ADIF 3.1.4)

### Key Field Relationships

Per the ADIF 3.1.4 specification, certain fields have dependencies:

> "STATE depends on DXCC. So, DXCC can be exported without exporting STATE. However, if STATE is exported, DXCC should be exported too."

### Location Fields (Contacted Station)

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| **DXCC** | Integer | DXCC Entity Code (enumerated) | 291 (United States) |
| **COUNTRY** | String | DXCC Entity Name | "UNITED STATES OF AMERICA" |
| **STATE** | Enumeration | Primary Administrative Subdivision code (depends on DXCC) | "MN" (Minnesota) |
| **CNTY** | Enumeration | Secondary Administrative Subdivision (e.g., US county) | "MN,Hennepin" |
| **GRIDSQUARE** | GridSquare | Maidenhead locator (2/4/6/8 chars) | "EN34" |
| **CQZ** | Integer | CQ Zone (1-40) | 4 |
| **ITUZ** | Integer | ITU Zone (1-90) | 7 |
| **CONT** | Enumeration | Continent (NA, SA, EU, AF, OC, AS, AN) | "NA" |

### US States (DXCC 291) - Primary Administrative Subdivisions

All 50 states plus DC use 2-letter codes matching postal abbreviations:
AL, AK, AZ, AR, CA, CO, CT, DE, DC, FL, GA, HI, ID, IL, IN, IA, KS, KY, LA, ME, MD, MA, MI, MN, MS, MO, MT, NE, NV, NH, NJ, NM, NY, NC, ND, OH, OK, OR, PA, RI, SC, SD, TN, TX, UT, VT, VA, WA, WV, WI, WY

### Canada (DXCC 1) - Primary Administrative Subdivisions

NS, QC, ON, MB, SK, AB, BC, NT, NB, NL, YT, PE, NU

### Display Approach

**Entity Column**: Shows COUNTRY field value only (the DXCC entity name)
- "UNITED STATES OF AMERICA", "CANADA", "GERMANY", etc.

**State/Province Column**: Shows STATE field value only (the subdivision code)
- "MN", "ON", "BY" (Bayern), etc.

**Grid Column**: Shows GRIDSQUARE field value
- "EN34", "FN31", etc.

### Data Population Sources

1. **WSJT-X UDP**: Provides only GRIDSQUARE (4 chars) and CALL
2. **ADIF Import**: May contain all fields (DXCC, COUNTRY, STATE, etc.)
3. **LoTW Download**: Provides DXCC, COUNTRY, STATE, GRIDSQUARE from confirmations
4. **Callsign Prefix Lookup**: Provides DXCC, COUNTRY, CQZ, ITUZ, CONT from prefix rules

### Data Population Strategy (Tiered Approach)

**Tier 1: At QSO Time (Immediate)**
- Callsign prefix lookup → DXCC, COUNTRY, CQZ, ITUZ, CONT
- Store GRIDSQUARE from WSJT-X (operator's claimed current location)
- **Do NOT derive STATE from callsign prefix** (see Portable Operations below)

**Tier 2: Authoritative Sync (Background)**
- LoTW sync → Fills/overwrites STATE, CNTY, confirms DXCC/GRID
- eQSL sync → Secondary source if LoTW doesn't have it

**Tier 3: For Non-LoTW/eQSL Stations**
- STATE field remains blank (honest - we don't know it)
- Optional future: QRZ XML API or HamQTH lookup
- Manual entry by user

### Portable/POTA Operations Handling

**The Problem**: W9ABC (licensed WI) operates POTA from California, sending grid CM87.

**Critical Rule**: All location fields refer to **where the station operated**, not their license address.

| Data Source | Gives You | Portable Accuracy |
|-------------|-----------|-------------------|
| Callsign Prefix (W9) | License DXCC (291) | ✅ DXCC correct |
| Callsign Prefix (W9) | License STATE (WI?) | ❌ **WRONG** for portable |
| WSJT-X Grid (CM87) | Operating location | ✅ Correct (if operator set it) |
| POTA_REF (K-1234@US-CA) | Explicit state | ✅ Correct (parse @US-XX) |
| LoTW Confirmation | Certificate station location | ✅ Correct (if portable cert used) |

**Our Strategy**:
1. DXCC from prefix is always valid (K, W, N, AA-AL = 291 regardless of state)
2. Never derive STATE from callsign prefix alone
3. Trust GRIDSQUARE from WSJT-X (operator's responsibility)
4. LoTW confirmation fills in authoritative STATE when received
5. Parse POTA_REF `@XX-YY` suffix for explicit state (future enhancement)

**For WAS Tracking**: You need LoTW-confirmed STATE anyway to claim the award, so incomplete STATE data for unconfirmed QSOs is acceptable.

**Why Not Grid→State Conversion?**: 
- 4-char grid is ~100×200km, can span multiple states
- DXCC boundaries ≠ political boundaries (Alaska vs Continental US)
- LoTW is the authoritative source for awards - trust it
