# GoQSO - Development Roadmap

## Current Status: Band Activity & Decode Display Working

Core architecture complete: Tauri 2.x + React + SQLite. WSJT-X connection working with live FT8 decode display.

---

## ✅ Completed

- [x] Tauri project setup with React + TypeScript
- [x] SQLite schema with all tables defined
- [x] React frontend with Tailwind CSS dark theme
- [x] DXCC reference data (340 entities from ARRL list)
- [x] Prefix rules for callsign → DXCC lookup
- [x] US state data for WAS tracking
- [x] Grid square → US state mapping
- [x] WSJT-X UDP listener on port 2237
- [x] Parse Heartbeat, Status, Decode, QsoLogged messages
- [x] Band Activity display with live FT8 decodes
- [x] Priority Queue showing NEW COUNTRY / NEW STATE stations
- [x] Database initialization with proper migration handling

---

## Phase 1: QSO Logging (Current Focus)

### 1.1 Log View Component
- [ ] Create `LogView.tsx` component for the Log tab
- [ ] Minimal table: Date, Time, Call, Band, Mode, RST, Entity, Status
- [ ] Sortable columns (click header)
- [ ] Search/filter box
- [ ] Pagination or virtual scroll

### 1.2 QSO Detail Modal
- [ ] Full QSO editor with all ADIF fields
- [ ] Sections: Basic, Station, QSL, Notes
- [ ] Edit existing QSOs
- [ ] Delete with confirmation

### 1.3 Auto-Logging from WSJT-X
- [ ] Handle QsoLogged UDP messages properly
- [ ] Auto-populate DXCC/state/grid
- [ ] Toast notification on new QSO

### 1.4 Manual QSO Entry
- [ ] "Add QSO" button with quick entry form
- [ ] Callsign lookup as you type

### 1.5 Import/Export
- [ ] Import ADIF file
- [ ] Export to ADIF file

---

## Phase 2: Transmission Control (Mock First)

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
