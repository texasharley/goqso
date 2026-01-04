# GoQSO - Development Roadmap

## Current Status: Phase 1 Core Complete

Core architecture complete: Tauri 2.x + React + SQLite hybrid schema. WSJT-X UDP integration working with live FT8 decodes and auto-logging. Badge system and QSO history panel implemented.

---

## ✅ Completed

### Core Architecture
- [x] Tauri 2.x project with React + TypeScript
- [x] SQLite hybrid schema (indexed columns + JSON blobs)
- [x] DXCC entities (340) and prefix rules
- [x] US state data with grid-to-state mapping
- [x] Dark theme with Tailwind CSS

### WSJT-X Integration
- [x] UDP listener for WSJT-X (port 2237)
- [x] Parse Heartbeat, Status, Decode, QsoLogged messages
- [x] Band Activity display with live FT8 decodes
- [x] Priority Queue (NEW COUNTRY / NEW STATE alerts)
- [x] Auto-log QSOs from WSJT-X with DXCC enrichment
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

---

## 🔴 Critical Gaps (vs DXKeeper)

### 1. ADIF Import/Export
**Why Critical**: Can't migrate from other loggers, can't backup data
- [ ] Parse ADIF file format
- [ ] Import command with field mapping
- [ ] Export to ADIF (full log or filtered)
- [ ] Handle all 180+ modes (see Modes.txt)

### 2. LoTW Integration
**Why Critical**: DXCC/WAS confirmation is the whole point
- [ ] Import LoTW confirmations from .adi file (like lotwreport.adi)
- [ ] Match to local QSOs by call/band/mode/date/time
- [ ] Update confirmation status in database
- [ ] Show confirmation badges in log (L=LoTW confirmed)
- [ ] Upload QSOs to LoTW (requires tqsl.exe integration)
- [ ] Track upload queue status

### 3. Award Progress Dashboard
**Why Critical**: Visual motivation is the killer feature
- [ ] DXCC progress: X/340 worked, Y confirmed (by band/mode)
- [ ] WAS progress: X/50 worked, Y confirmed
- [ ] VUCC progress: grid squares on 6m+
- [ ] USA-CA progress: counties (requires CNTY field)
- [ ] Progress bars with targets

### 4. QSO Map Visualization
**Why Critical**: Visual gratification, better than QSOmap.org
- [ ] World map with QSO pins
- [ ] Color coding: worked vs confirmed
- [ ] Filter by band/mode/date range
- [ ] Grid square overlay for VUCC
- [ ] US state map for WAS
- [ ] Azimuthal projection centered on home QTH

---

## Phase 1: ADIF & Mode Support (Current Focus)

### 1.1 Mode Registry
All 180+ modes from ADIF spec already supported via TEXT field.
- [ ] Mode validation/normalization on import
- [ ] Mode grouping (DATA, PHONE, CW, IMAGE)
- [ ] Submode handling (FT8 is MODE=FT8, SUBMODE optional)

### 1.2 ADIF Parser (Rust)
- [ ] Parse `<FIELD:length>value` format
- [ ] Handle header section before `<EOH>`
- [ ] Parse QSO records ending with `<EOR>`
- [ ] Strip comments (text after //)
- [ ] Map ADIF fields to our schema

### 1.3 ADIF Importer
- [ ] File picker dialog
- [ ] Preview import (show count, date range)
- [ ] Duplicate detection (skip/update options)
- [ ] Progress indicator for large files
- [ ] Import report (added/skipped/errors)

### 1.4 ADIF Exporter
- [ ] Export all QSOs
- [ ] Export filtered subset
- [ ] Include all standard ADIF fields
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
