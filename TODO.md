# GoQSO - Development Roadmap

## Current Status: MVP Scaffold Complete

The core architecture is in place with Tauri 2.x + React + SQLite. Commands and UI components are scaffolded.

---

## Phase 1: Core Functionality (MVP)

### ✅ Completed
- [x] Tauri project setup with React + TypeScript
- [x] SQLite schema with all tables defined
- [x] React frontend with shadcn/ui and dark mode
- [x] Tauri command signatures for all operations
- [x] Git repository initialized with proper .gitignore

### 🔄 In Progress
- [ ] **DXCC Reference Data** - Curated entity list from authoritative sources
  - Build from ARRL DXCC list (not CTY.DAT)
  - Include ITU prefix allocations
  - Version-controlled in repository

### 📋 To Do
- [ ] **Database Initialization** - Run migrations on app startup
- [ ] **UDP Listener** - Connect to WSJT-X on port 2237
- [ ] **QSO Logging** - Parse WSJT-X QSO Logged messages, insert into database
- [ ] **Basic UI Wiring** - Connect React components to Tauri commands
- [ ] **App Icon** - Design and add application icon

---

## Phase 2: LoTW Integration

### 📋 To Do
- [ ] **TQSL Detection** - Locate TQSL installation on Windows/Mac/Linux
- [ ] **ADIF Export** - Generate ADIF files for pending QSOs
- [ ] **TQSL Signing** - Invoke TQSL CLI to sign ADIF files
- [ ] **LoTW Upload** - HTTP POST signed files to lotw.arrl.org/lotw/upload
- [ ] **Confirmation Download** - Query lotwreport.adi for confirmations
- [ ] **Matching Algorithm** - Match confirmations to QSOs (CALL + BAND + MODE_GROUP + TIME ±30min)
- [ ] **Sync Queue** - Track pending uploads, handle offline mode

---

## Phase 3: Awards Tracking

### 📋 To Do
- [ ] **DXCC Progress** - Calculate worked/confirmed entities per band/mode
- [ ] **WAS Progress** - Track 50 states worked/confirmed
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
