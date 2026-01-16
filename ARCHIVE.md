# GoQSO - Completed Work Archive

> Items archived from TODO.md after validation by Backlog-Architect.
> See [TODO.md](TODO.md) for active work items.

---

## 2026-01-13: Tech Debt Cleanup (Health Report)

> **Validated:** 2026-01-13 by Backlog-Architect  
> **Build verification:** `cargo check` 0 warnings, `npm run build` SUCCESS

### ✅ Fix Hardcoded Paths After Workspace Move

**Context:** Project moved from `C:\dev\qso-logger\goqso` to `C:\dev\goqso`.

**Files Fixed:**
- CLAUDE.md — Updated dev server command path
- .github/copilot-instructions.md — Updated dev server command path

---

### ✅ Fix Unreachable State Patterns in states.rs

**Resolution:** Removed `grid_to_state()` and `approximate_state_from_coords()` entirely. STATE data should come from FCC database lookup or LoTW confirmation, not unreliable grid-based approximation.

**Files Changed:**
- `src-tauri/src/reference/states.rs` — Removed functions, added rationale comment
- `src-tauri/src/reference/mod.rs` — Removed wrapper, added note

---

### ✅ Delete Temporary/Debug Files from Root

**Files Deleted:**
- `tauri-debug.log`
- `temp_wsjtx_import.adi`

---

### ✅ Move Misplaced Files to Correct Locations

**Files Moved:**
- `PERFORMANCE.md` → `docs/PERFORMANCE.md`
- `app-icon.png` → `src-tauri/icons/app-icon.png`

---

### ✅ Document Undocumented Modules in CLAUDE.md

**Modules Documented:**
- `src-tauri/src/fcc/` — FCC ULS database integration
- `src-tauri/src/qso_tracker/` — QSO state machine for FT8 exchange tracking

---

### ✅ Address Unused Code Warnings

**Results:** 83 warnings → 0 warnings

**Actions:**
- Ran `cargo fix` to remove unused imports
- Added `#![allow(dead_code)]` with documentation to modules with intentional future code
- Prefixed unused variables with underscore

---

### ✅ Grid Validation (RR73 Bug)

**Problem:** Grid column sometimes showed "RR73" instead of actual Maidenhead grid.

**Fix:** `is_valid_grid()` function in `wsjtx.rs` now rejects FT8 message fragments (RR73, RRR, 73) as grid data. Comprehensive test coverage added.

---

### ✅ RST Normalization (Single Digit Bug)

**Problem:** RST column showed single digits like "9" instead of "-09" or "+09".

**Fix:** `normalize_rst()` function in `wsjtx.rs` zero-pads single digit values and ensures consistent sign prefix. Test coverage added.

---

### ✅ State Data Source

**Problem:** Grid-to-state lookup was unreliable (~90% accuracy).

**Fix:** Removed grid-to-state entirely. FCC database is PRIMARY source for US callsign → state. LoTW QSL data is SECONDARY source.

---

## 2026-01-09: v0.3.0 UI/UX Improvements

### ✅ Band Activity Table Header Fix
- Fixed sticky header overlapping with table rows
- Added z-index, solid background, and shadow to header

### ✅ WSJT-X Connection Panel Redesign
- New SDR-inspired UI with large frequency display
- Mode badges (FT8/FT4) with color coding
- Compact TX status indicator (pill style)
- Removed redundant "Working:" component (was showing stale data)

### ✅ Active QSO Stale Data Fix
- Added 2-minute inactivity timeout to reset stale QSO data
- Clear previous QSO data when calling CQ
- Activity timestamp tracking on TX/RX events

### ✅ DXCC Lookup Tests Added
- Unit tests for compound callsign handling (HK0/DF3TJ → San Andrés)
- Tests for extract_dxcc_portion() function

---

## 2026-01-08: Bug Fixes

### ✅ DXCC Entity ID Mismatch - "Unknown" Entities

**Problem:** D2UY (Angola) showing as "Unknown" entity.

**Root Cause:** 38 duplicate entity_ids in dxcc.rs, manually curated data out of sync with ARRL.

**Fix:**
1. Downloaded official ARRL DXCC list
2. Created `src-tauri/resources/dxcc_entities.json` as single source of truth (402 entities)
3. Regenerated `dxcc.rs` from JSON with NO duplicates
4. Updated struct to use zone arrays for multi-zone entities

---

### ✅ WSJT-X QSO Insert Creating Duplicates

**Problem:** Same QSO appeared twice with different time_on formats.

**Root Cause:** `normalize_time_to_hhmmss("2026-01-08 23:24:45")` returned "2026-0" instead of "232445".

**Fix:** Updated time normalization to detect datetime format and extract time portion correctly.

---

### ✅ v0.2.3 LoTW Sync Incremental Download Fix

**Problem:** LoTW sync kept showing same confirmations as "new" on every sync.

**Root Causes:**
1. Frontend discarding time portion of `last_qsl_date`
2. LoTW's `qso_qslsince` parameter is inclusive
3. Local state not updated after sync

**Fix:** Add 1 second to APP_LoTW_LASTQSL before saving to prevent re-fetching same record.

---

## 2026-01-07: v0.2.2 Reference Data Audit

### ✅ Phase 1: Caribbean/Central American Prefixes
Added 32 prefixes including VP2M (Montserrat), all Caribbean islands, Central American countries.

### ✅ Phase 2: DXCC Entity ID Corrections
- Fixed El Salvador: 62 → 74
- Fixed Guatemala: 78 → 76
- Fixed St. Lucia: 65 → 97
- Verified all against official ARRL list

### ✅ Phase 3: South American Prefixes
Added CE (Chile), OA (Peru), CP (Bolivia), HC (Ecuador), HC8 (Galapagos), PZ (Suriname), 8R (Guyana), FY (French Guiana), ZP (Paraguay), CX (Uruguay), VP8 variants.

### ✅ Phase 4: African Prefixes
Added 80+ African prefixes covering all DXCC entities on the continent.

### ✅ Phase 5: Asian/Pacific Prefixes
Added 90+ prefixes including Malaysia, Singapore, Indonesia, Vietnam, Thailand, Middle East, Central Asia, Pacific Islands.

### ✅ Phase 6: US States and Canadian Provinces
- Verified all 50 US state abbreviations
- Added CanadianProvince struct with 13 provinces/territories
- Added helper functions

### ✅ Phase 7: Testing
- 10 prefix tests passing
- VP2M → entity 177 verified
- KL7 → entity 6 (Alaska) verified
- KB9FIN → entity 291 (USA) verified

---

## 2026-01-06: v0.2.1 Bug Fixes

### ✅ Alaska DXCC Classification (NOT A BUG)
Alaska IS a separate DXCC entity (#6) per official ARRL list. "NEW DXCC: Alaska" badge is correct behavior.

### ✅ RX Messages in Active QSO Panel
**Root Cause:** Heartbeat `id` field contains instance name ("WSJT-X"), not operator callsign.
**Fix:** Parse `de_call` from Status message instead.

### ✅ Auto-Logging from WSJT-X
**Root Cause:** WSJT-X sends LoggedADIF (type 12), not QsoLogged (type 5).
**Fix:** Added `parse_logged_adif()` to handle ADIF format.

---

## 2026-01-05: Data Population Strategy

Implemented tiered data population per ADIF 3.1.4:
- **Tier 1 (QSO time):** Callsign prefix → DXCC, COUNTRY, CQZ, ITUZ, CONT
- **Tier 2 (LoTW sync):** STATE, CNTY from confirmations
- **Tier 3 (Future):** POTA_REF parsing, QRZ API

Removed grid_to_state() from decode events. STATE from LoTW is authoritative for WAS award.

---

## Core Architecture (Completed)

### Infrastructure
- [x] Tauri 2.x project with React + TypeScript
- [x] SQLite hybrid schema (indexed columns + JSON blobs)
- [x] Dark theme with Tailwind CSS

### Reference Data
- [x] DXCC entities (402) from ARRL official list
- [x] Prefix rules regenerated from JSON
- [x] US state data for WAS
- [x] Canadian province data

### WSJT-X Integration
- [x] UDP listener (port 2237)
- [x] Parse Heartbeat, Status, Decode, QsoLogged, LoggedADIF messages
- [x] Band Activity display with live FT8 decodes
- [x] Priority Queue (NEW DXCC alerts)
- [x] Auto-log QSOs with DXCC/CQZ/ITUZ enrichment
- [x] Toast notifications

### QSO Logging UX
- [x] QsoLog table with sortable columns
- [x] Badge system (new DXCC, band slot, dupe, previous QSO)
- [x] QSO detail modal
- [x] Callsign history panel
- [x] Delete QSO with confirmation
- [x] Keyboard shortcuts

### ADIF Support
- [x] Parser and writer
- [x] Mode registry (180+ modes)
- [x] Import with duplicate detection
- [x] Export

### LoTW Integration (Read Only)
- [x] HTTP client for downloading confirmations
- [x] Parse LoTW ADIF format
- [x] Match confirmations to local QSOs
- [x] Incremental sync with proper timestamp handling
