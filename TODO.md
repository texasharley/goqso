# GoQSO - Development Roadmap

## Current Status: Phase 1 & 2 In Progress

Core architecture complete: Tauri 2.x + React + SQLite hybrid schema. WSJT-X UDP integration working with live FT8 decodes and auto-logging. Badge system and QSO history panel implemented.

**ADIF module complete** - Full parser, writer, and 180+ mode registry.
**LoTW client complete** - HTTP client for downloading confirmations from LoTW API.
**Data population strategy implemented** - Tiered approach per ADIF 3.1.4 spec (see CLAUDE.md).

---

## 📋 Changelog

### 2026-01-05: Data Population Strategy Implementation

**Problem Solved**: STATE field was being incorrectly derived from Maidenhead grid squares, which is unreliable for portable operators (e.g., POTA activators operating from a different state than their license).

**Changes Made**:

1. **Backend (Rust)**:
   - Added `lookup_call_full()` function returning DXCC, COUNTRY, CONT, CQZ, ITUZ from callsign prefix
   - Updated `insert_qso_from_wsjtx()` to populate CQZ/ITUZ at QSO time
   - Removed `grid_to_state()` call from decode events
   - Deprecated `grid_to_state()` function with documentation explaining why

2. **Frontend (React/TypeScript)**:
   - Updated `BandActivity.tsx` to show continent instead of (unreliable) state
   - Removed "NEW STATE" detection from live decodes (unreliable)
   - Added `continent`, `cqz`, `ituz` fields to decode events

3. **Data Strategy (Tiered)**:
   - **Tier 1 (At QSO time)**: Callsign prefix → DXCC, COUNTRY, CQZ, ITUZ, CONT
   - **Tier 2 (LoTW sync)**: STATE, CNTY filled from confirmations (authoritative)
   - **Tier 3 (Future)**: POTA_REF parsing, QRZ API for non-LoTW stations

**Why This Matters**:
- WAS award requires LoTW-confirmed STATE anyway
- Portable operators (POTA, SOTA) send grids from operating location, not license address
- 4-char grids are ~100×200km and can span multiple states
- LoTW is the authoritative source for award credit

See `CLAUDE.md` section "Data Population Strategy" for full documentation.

---

## ✅ Completed

### Core Architecture
- [x] Tauri 2.x project with React + TypeScript
- [x] SQLite hybrid schema (indexed columns + JSON blobs)
- [x] DXCC entities (340) and prefix rules
- [x] US state data with grid-to-state mapping
- [x] Dark theme with Tailwind CSS
- [x] Tiered data population strategy (DXCC/CQZ/ITUZ at QSO time, STATE from LoTW)

### WSJT-X Integration
- [x] UDP listener for WSJT-X (port 2237)
- [x] Parse Heartbeat, Status, Decode, QsoLogged messages
- [x] Band Activity display with live FT8 decodes
- [x] Priority Queue (NEW DXCC alerts)
- [x] Auto-log QSOs from WSJT-X with DXCC/CQZ/ITUZ enrichment
- [x] Toast notifications on QSO logged

### QSO Logging UX
- [x] QsoLog table with sortable columns
- [x] Badge system (new DXCC, band slot, dupe, previous QSO)
- [x] QSO detail modal with two-column layout
- [x] Callsign history panel (previous QSOs with station)
- [x] Award impact preview (shows what award credit)
- [x] Delete QSO with confirmation
- [x] Keyboard shortcuts (/, Escape)
- [x] UX design doc vs DXKeeper

### ADIF Support (Phase 1 - Complete)
- [x] ADIF parser (`<FIELD:length>value` format)
- [x] Header section handling (`<EOH>`)
- [x] QSO record parsing (`<EOR>`)
- [x] ADIF writer for export
- [x] Mode registry (180+ ADIF 3.1.4 modes)
- [x] Mode grouping (DATA, PHONE, CW, IMAGE)
- [x] Import command with duplicate detection
- [x] Export command
- [x] CNTY (county) field in schema

### LoTW Integration - Read Only (Phase 2a - Complete)
- [x] LoTW HTTP client module (`lotw/client.rs`)
- [x] Download confirmations endpoint (`lotwreport.adi`)
- [x] Download DXCC credits endpoint (`qslcards.php`)
- [x] LoTW user activity check (CSV)
- [x] Parse LoTW ADIF format
- [x] Match confirmations to local QSOs
- [x] Update lotw_qsl_rcvd/lotw_qsl_date fields
- [x] get_sync_status command
- [x] sync_lotw_download command (with credentials)

---

## 🔴 Critical Gaps (Remaining)

### 1. LoTW Upload (BLOCKED - Must Test First)
**IMPORTANT**: Upload functionality is intentionally NOT implemented yet.
We must never submit test data to LoTW. Only real QSO data can be uploaded.
- [ ] Queue QSOs for upload
- [ ] Integration with TQSL for signing
- [ ] Track upload status (pending/uploaded/failed)
- [ ] Batch upload support

### 2. Award Progress Dashboard
**Why Critical**: Visual motivation is the killer feature
- [x] get_dxcc_progress command (worked/confirmed counts)
- [x] get_was_progress command (state lists)
- [ ] DXCC progress UI: X/340 worked, Y confirmed (by band/mode)
- [ ] WAS progress UI: X/50 worked, Y confirmed
- [ ] VUCC progress: grid squares on 6m+
- [ ] USA-CA progress: counties
- [ ] Progress bars with targets

### 3. QSO Map Visualization
**Why Critical**: Visual gratification, better than QSOmap.org
- [ ] World map with QSO pins
- [ ] Color coding: worked vs confirmed
- [ ] Filter by band/mode/date range
- [ ] Grid square overlay for VUCC
- [ ] US state map for WAS
- [ ] Azimuthal projection centered on home QTH

---

## LoTW API Reference

### Download Confirmations (GET)
```
https://lotw.arrl.org/lotwuser/lotwreport.adi
  ?login=CALLSIGN
  &password=PASSWORD
  &qso_query=1
  &qso_qsl=yes              # Get confirmations (QSL_RCVD=Y)
  &qso_qslsince=YYYY-MM-DD  # Only new since date
  &qso_qsldetail=yes        # Include location details
  &qso_withown=yes          # Include station callsign
```

**Response fields:**
- Header: `APP_LoTW_LASTQSL`, `APP_LoTW_NUMREC`
- QSO: CALL, BAND, MODE, QSO_DATE, TIME_ON, QSL_RCVD, QSLRDATE
- Detail: DXCC, COUNTRY, STATE, CNTY, CQZ, ITUZ, GRIDSQUARE
- Award: CREDIT_GRANTED, APP_LoTW_2xQSL

### Download DXCC Credits (GET)
```
https://lotw.arrl.org/lotwuser/logbook/qslcards.php
  ?login=CALLSIGN
  &password=PASSWORD
```

### User Activity List (Public GET)
```
https://lotw.arrl.org/lotw-user-activity.csv
```
Format: `CALLSIGN,YYYY-MM-DD,HH:MM:SS` (last upload date)

---

## Phase 1: ADIF & Mode Support ✅ COMPLETE
- [ ] Format for LoTW upload compatibility

---

## Phase 2: LoTW Confirmation Sync

### 2.1 LoTW Download Import
LoTW provides confirmations as ADIF file with special fields:
- `<QSL_RCVD:1>Y` = Confirmed
- `<QSLRDATE:8>20260104` = Confirmation date
- `<APP_LoTW_2xQSL:1>Y` = Both parties confirmed

**Implementation:**
- [ ] Parse LoTW ADIF format
- [ ] Match by: CALL + BAND + MODE + QSO_DATE + TIME_ON (±5 min)
- [ ] Update confirmations table with:
  - service = 'lotw'
  - status = 'confirmed'
  - confirmed_at = QSLRDATE
- [ ] Handle new QSOs from LoTW (worked but not in log)

### 2.2 LoTW Upload
- [ ] Queue QSOs for upload
- [ ] Integration with TQSL for signing
- [ ] Track upload status (pending/uploaded/failed)
- [ ] Batch upload support

### 2.3 Confirmation UI
- [ ] Badge colors: ✓ green (confirmed), ○ yellow (uploaded), ✗ gray (not sent)
- [ ] Filter by confirmation status
- [ ] "Unconfirmed" quick filter
- [ ] Confirmation date in detail modal

---

## Phase 3: Award Progress Tracking

### 3.1 DXCC Award
- [ ] Worked/Confirmed counts by band and mode
- [ ] DXCC Challenge (band-slot counting)
- [ ] Progress toward:
  - DXCC (100 confirmed)
  - DXCC Honor Roll (326+)
  - DXCC #1 (340 current entities)
- [ ] Entity cards showing: worked bands, confirmed bands

### 3.2 WAS (Worked All States)
- [ ] 50 states tracker
- [ ] Triple Play: Phone + CW + Digital confirmed
- [ ] State cards with confirmation status

### 3.3 VUCC (VHF/UHF Century Club)
- [ ] Grid squares on 6m, 2m, 70cm, etc.
- [ ] 100/200/300+ level tracking
- [ ] Grid map visualization

### 3.4 USA-CA (Counties)
**Requires schema update: add CNTY column**
- [ ] 3,077 US counties
- [ ] Progress tracking
- [ ] County map

---

## Phase 4: QSO Map Visualization

### 4.1 Technology Choice
Options:
- **Leaflet.js** - Open source, free, flexible
- **MapLibre GL** - Modern, WebGL, beautiful
- **D3.js** - Full control, steep learning

Recommendation: **MapLibre GL** for professional look

### 4.2 World Map
- [ ] QSO pins at grid square centers
- [ ] Color: red=worked, green=confirmed
- [ ] Clustering for dense areas
- [ ] Popup on click: call, date, band, mode
- [ ] Filter controls: band, mode, date range

### 4.3 Projections
- [ ] Azimuthal equidistant (ham favorite)
- [ ] Mercator (standard)
- [ ] Centered on user's QTH

### 4.4 Overlays
- [ ] Grid square grid (for VUCC)
- [ ] CQ zones
- [ ] ITU zones
- [ ] DXCC entity boundaries

### 4.5 US-Specific
- [ ] State boundaries for WAS
- [ ] County boundaries for USA-CA
- [ ] State fill colors by status

---

## Phase 5: Transmission Control

### 2.1 Mock Transmission System
- [ ] Test decode data for simulation
- [ ] Mock Reply message (no actual TX)
- [ ] Visual TX indicator
- [ ] QSO state machine (Tx1-Tx6)

### 2.2 Safety Features
- [ ] TX enable toggle (default OFF)
- [ ] Confirmation dialog before TX
- [ ] TX timeout auto-disable
- [ ] Clear TX active indicator

### 2.3 Reply Integration
- [ ] Send Reply message to WSJT-X
- [ ] Track QSO state per callsign
- [ ] Suggest appropriate Tx message
- [ ] Manual Tx selection

### 2.4 Call Button / Double-Click
- [ ] Wire Priority Queue "Call" button
- [ ] Double-click row to call
- [ ] Call confirmation with preview

---

## Phase 3: LoTW Integration

### 3.1 Upload
- [ ] Queue QSOs for upload
- [ ] Sign with TQSL
- [ ] Upload to LoTW
- [ ] Mark as uploaded

### 3.2 Download
- [ ] Download confirmations
- [ ] Match to local QSOs  
- [ ] Update confirmation status

---

## Phase 4: Awards Tracking

### 4.1 DXCC Progress
- [ ] Worked/confirmed by band/mode
- [ ] Challenge tracking
- [ ] Visual progress display

### 4.2 WAS Progress
- [ ] 50 states worked/confirmed
- [ ] Triple Play support

### 4.3 VUCC Progress
- [ ] Grid squares by band

---

## UX Design Principles

### Minimal by Default
- Show only essential information
- Progressive disclosure - details on demand
- Clean, uncluttered interface

### Information Hierarchy
1. **Glanceable**: Status indicators, counts, colors
2. **Scannable**: Table with key columns  
3. **Detailed**: Modal for full information

### Responsive Feedback
- Immediate visual feedback
- Loading states
- Success/error notifications

---

## Technical Debt

- [ ] Fix 80+ Rust warnings (unused code)
- [ ] Unit tests for FT8 parsing
- [ ] Unit tests for DXCC lookup
- [ ] Error boundaries in React
- [ ] Proper error handling/display
- [ ] **VUCC Progress** - Grid squares on 6m+ bands
- [ ] **Awards Matrix UI** - Visual grid showing progress
- [ ] **Goal Notifications** - Alert when approaching award thresholds

---

## Phase 4: Polish & Release

### 📋 To Do
- [ ] **Error Handling** - User-friendly error messages
- [ ] **Settings Persistence** - Save/load user preferences
- [ ] **Offline Mode** - Queue operations when no internet
- [ ] **Performance Optimization** - Efficient queries for large logs
- [ ] **Installer Builds** - Windows MSI, macOS DMG, Linux AppImage
- [ ] **Documentation** - User guide and README

---

## Future Enhancements (Post-MVP)

### FT8 Direct Integration
> TODO: Research ft8_lib (kgoba/ft8_lib) for native FT8 encode/decode
> - Audio I/O integration with CAT control
> - Direct radio integration without WSJT-X dependency
> - Real-time waterfall display

### Additional Award Programs
> TODO: Research requirements and data sources
> - IOTA (Islands on the Air)
> - SOTA (Summits on the Air)
> - POTA (Parks on the Air) - API available at pota.app
> - CQ WAZ (Worked All Zones)
> - CQ WPX (Worked Prefixes)

### Contest Logging
> TODO: Evaluate Cabrillo format and contest-specific requirements
> - Cabrillo export
> - Dupe checking
> - Rate tracking
> - N+1 multiplier display

### Cloud Sync
> TODO: Design sync architecture for multi-device support
> - Secure cloud backup
> - Multi-device sync
> - Web interface for viewing logs remotely

### QSL Card Integration
> TODO: Research eQSL, QRZ logbook, and ClubLog APIs
> - eQSL.cc sync
> - QRZ.com logbook sync
> - ClubLog integration

### Mobile Support
> TODO: Leverage Tauri 2.x mobile capabilities
> - iOS build (Tauri 2.x supports this)
> - Android build
> - Responsive UI for tablet/phone

---

## Data Source Philosophy

### Why NOT CTY.DAT?
- Maintained by single person (Jim Reisert AD1C)
- No versioning, no API
- No guarantee of longevity
- Most ham software blindly trusts it

### Our Approach
1. **Curated DXCC entities** - Version-controlled JSON from ARRL official list
2. **ITU prefix allocations** - Official international prefix blocks
3. **LoTW as ground truth** - Confirmations include DXCC entity number
4. **Club Log API** (future) - Quality database with edge cases

### Reference Data Updates
- DXCC list changes ~1-2x per year with ARRL announcements
- We will version these changes in Git
- Users can update via app settings or manual download

---

## Technical Debt

### Cleanup Items
- [ ] Remove unused imports (Rust compiler warnings)
- [ ] Add proper error types instead of String errors
- [ ] Implement proper logging (tracing crate)
- [ ] Add unit tests for core logic
- [ ] Add integration tests for Tauri commands

### Performance Considerations
- [ ] Index optimization for QSO queries by date range
- [ ] Lazy loading for large QSO lists
- [ ] Background processing for LoTW sync

---

## Contributing

This project is currently in early development. See GitHub issues for current work items.

Repository: https://github.com/texasharley/goqso
