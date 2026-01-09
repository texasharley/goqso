# GoQSO - Development Roadmap

## Current Status: Phase 1 & 2 Complete, ✅ Priority Bugs Fixed, v0.3.0 UI Improvements

Core architecture complete: Tauri 2.x + React + SQLite hybrid schema. WSJT-X UDP integration working with live FT8 decodes and auto-logging. Badge system and QSO history panel implemented.

**ADIF module complete** - Full parser, writer, and 180+ mode registry.
**LoTW client complete** - HTTP client for downloading confirmations from LoTW API.
**LoTW incremental sync FIXED** - Proper handling of `qso_qslsince` inclusive timestamp.
**Data population strategy implemented** - Tiered approach per ADIF 3.1.4 spec (see CLAUDE.md).
**✅ Reference Data Audit COMPLETE** - Regenerated from official ARRL DXCC list (January 2026 edition).
**✅ v0.3.0 UI Improvements** - SDR-style WSJT-X panel, stale data timeout, band activity fixes.

---

## ✅ v0.3.0 UI/UX Improvements (2026-01-09)

### Band Activity Table Header Fix ✅
- Fixed sticky header overlapping with table rows
- Added z-index, solid background, and shadow to header

### WSJT-X Connection Panel Redesign ✅
- New SDR-inspired UI with large frequency display
- Mode badges (FT8/FT4) with color coding
- Compact TX status indicator (pill style)
- Removed redundant "Working:" component (was showing stale data)
- Changed frequency display from green to white for readability

### Active QSO Stale Data Fix ✅
- Added 2-minute inactivity timeout to reset stale QSO data
- Clear previous QSO data when calling CQ
- Hide location info when in "calling_cq" mode
- Activity timestamp tracking on TX/RX events

### DXCC Lookup Tests Added ✅
- Added unit tests for compound callsign handling (HK0/DF3TJ → San Andrés)
- Added tests for extract_dxcc_portion() function
- Tests verify prefix rules: HK0 → entity 216 (San Andrés & Providencia)

---

## ✅ FIXED BUGS (2026-01-08)

### BUG #1: DXCC Entity ID Mismatch - "Unknown" Entities ✅ FIXED

**Symptom:** D2UY (Angola) showing as "Unknown" entity in Priority Queue.

**Root Cause:** 38 duplicate entity_ids in dxcc.rs, manually curated data out of sync with official ARRL list.

**Fix Applied:**
1. Downloaded official ARRL DXCC list: https://www.arrl.org/files/file/DXCC/Current_Deleted.txt
2. Created `src-tauri/resources/dxcc_entities.json` as single source of truth (402 entities)
3. Regenerated `src-tauri/src/reference/dxcc.rs` from JSON with NO duplicates
4. Updated struct to use zone arrays (`cq_zones: &[u8]`) for entities spanning multiple zones
5. Verified Angola = entity_id 401, CQ 36, ITU 52

**Files Changed:**
- `src-tauri/resources/dxcc_official.txt` - Raw ARRL data
- `src-tauri/resources/dxcc_entities.json` - Parsed JSON (authoritative)
- `src-tauri/src/reference/dxcc.rs` - Generated from JSON
- `src-tauri/src/reference/mod.rs` - Updated for zone arrays
- `src-tauri/src/db/init.rs` - Updated for zone arrays

---

### BUG #2: WSJT-X QSO Insert Creating Duplicates (HIGH PRIORITY)

**Symptom:** Same QSO appears twice in log with different time_on formats.

**Evidence from database:**
```
AC2SB|20260108|2026-0|...   <-- WRONG time format
AC2SB|20260108|232445|...   <-- CORRECT time format
N8FRJ|20260108|2026-0|...   <-- WRONG 
N8FRJ|20260108|233541|...   <-- CORRECT
```

**Root Cause Analysis:**

1. **WSJT-X sends TWO message types for the same QSO:**
   - `QsoLogged` (type 5): `datetime_on = "2026-01-08 23:24:45"` (full datetime)
---

### BUG #2: WSJT-X QSO Insert Creating Duplicates ✅ FIXED

**Symptom:** Same QSO appeared twice in log with different time_on formats.

**Root Cause:** `normalize_time_to_hhmmss("2026-01-08 23:24:45")` returned "2026-0" (first 6 chars) instead of "232445".

**Fix Applied:**
1. Updated `normalize_time_to_hhmmss()` to detect datetime format and extract time portion
2. Updated `insert_qso_from_wsjtx()` to extract date from datetime_on when full format is present

**Files Changed:**
- `src-tauri/src/commands.rs` - Fixed datetime parsing

---

## ✅ v0.2.3 LoTW Sync Incremental Download Fix (2026-01-08)

### ✅ CRITICAL BUG FIXED: LoTW Sync Re-downloading Same Records

**Problem:** LoTW sync kept showing the same confirmations as "new" on every sync.

**Root Causes Identified (RCA):**
1. Frontend was discarding the time portion of `last_qsl_date`, sending only "YYYY-MM-DD" instead of "YYYY-MM-DD HH:MM:SS"
2. LoTW's `qso_qslsince` parameter is **inclusive** — returns records "on or after" the specified datetime
3. Local state wasn't updated after sync, so "Sync Again" used stale date

**Solution Algorithm:**

```
┌─────────────────────────────────────────────────────────────────────┐
│                  LoTW Incremental Sync Algorithm                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  1. LOAD last sync datetime from settings (lotw_last_download)       │
│     Format: "YYYY-MM-DD HH:MM:SS" (full datetime, not just date!)    │
│                                                                      │
│  2. QUERY LoTW API with qso_qslsince = last_sync_datetime            │
│     → LoTW returns all records received ON OR AFTER that datetime    │
│                                                                      │
│  3. MATCH returned records against local QSO database                │
│     → call + date + HHMM (4 chars) + band (case-insensitive)         │
│                                                                      │
│  4. On SUCCESS, LoTW returns APP_LoTW_LASTQSL header                 │
│     → This is the datetime of the NEWEST record in the response      │
│                                                                      │
│  5. ADD 1 SECOND to APP_LoTW_LASTQSL before saving                   │
│     → Because qso_qslsince is INCLUSIVE, without this we'd           │
│       re-fetch the same record forever!                              │
│                                                                      │
│     Example:                                                         │
│       LoTW returns: APP_LoTW_LASTQSL = "2026-01-08 20:19:04"         │
│       We save:      lotw_last_download = "2026-01-08 20:19:05"       │
│                                                                      │
│  6. SAVE the adjusted datetime to settings                           │
│                                                                      │
│  7. UPDATE local React state for "Sync Again" button                 │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Files Modified:**
- `src/components/LotwSync.tsx` - Frontend datetime handling
- `src-tauri/src/commands.rs` - Added logging for debugging

**Key Code (LotwSync.tsx):**
```typescript
// After successful sync, add 1 second to prevent re-fetching same record
const dt = new Date(result.last_qsl.replace(' ', 'T'));
dt.setSeconds(dt.getSeconds() + 1);
// Format back to "YYYY-MM-DD HH:MM:SS" for LoTW API
const nextSinceDate = `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
await invoke("set_setting", { key: "lotw_last_download", value: nextSinceDate });
```

**Status:** ✅ COMPLETE - Verified working. Subsequent syncs now correctly show "0 Downloaded" when no new confirmations exist.

---

## ✅ v0.2.2 Ham Radio Reference Data Audit (2026-01-07)

### ✅ CRITICAL BUG FIXED: Montserrat (VP2M) Now Resolving Correctly

**Problem:** VP2M prefix (Montserrat) was not being resolved to DXCC entity 177.
**Root Cause:** VP2M prefix was MISSING from `prefixes.rs` - no Caribbean VP2x prefixes defined.
**Status:** FIXED - Added VP2M → entity 177 and all Caribbean prefixes.

---

### Reference Data Audit Checklist

#### ✅ Phase 1: Add Missing Caribbean/Central American Prefixes (COMPLETE)
- [x] **1.1** Add VP2E - Anguilla (entity 8)
- [x] **1.2** Add VP2M - Montserrat (entity 177) ⬅️ CRITICAL FIX
- [x] **1.3** Add VP2V - British Virgin Islands (entity 65)
- [x] **1.4** Add VP5 - Turks & Caicos (entity 84)
- [x] **1.5** Add VP9 - Bermuda (entity 51)
- [x] **1.6** Add V2 - Antigua & Barbuda (entity 12)
- [x] **1.7** Add V4 - St. Kitts & Nevis (entity 246)
- [x] **1.8** Add V3 - Belize (entity 66)
- [x] **1.9** Add HI - Dominican Republic (entity 72)
- [x] **1.10** Add CO/CM - Cuba (entity 70)
- [x] **1.11** Add TI - Costa Rica (entity 308)
- [x] **1.12** Add TG - Guatemala (entity 78)
- [x] **1.13** Add HR/HQ - Honduras (entity 80)
- [x] **1.14** Add YN/HT - Nicaragua (entity 86)
- [x] **1.15** Add YS/HU - El Salvador (entity 62)
- [x] **1.16** Add HP - Panama (entity 88)
- [x] **1.17** Add C6 - Bahamas (entity 211)
- [x] **1.18** Add 6Y - Jamaica (entity 82)
- [x] **1.19** Add 8P - Barbados (entity 62)
- [x] **1.20** Add J3 - Grenada (entity 97)
- [x] **1.21** Add J6 - St. Lucia (entity 65)
- [x] **1.22** Add J7 - Dominica (entity 95)
- [x] **1.23** Add J8 - St. Vincent (entity 98)
- [x] **1.24** Add FM - Martinique (entity 64)
- [x] **1.25** Add FG - Guadeloupe (entity 79)
- [x] **1.26** Add FS - French St. Martin (entity 213)
- [x] **1.27** Add FJ - St. Barthelemy (entity 214)
- [x] **1.28** Add PJ2/PJ4/PJ9 - Curacao/Bonaire (entities 517/52)
- [x] **1.29** Add PJ5/PJ6/PJ7 - Saba/St. Eustatius/Sint Maarten (entities 516/520)
- [x] **1.30** Add 9Y/9Z - Trinidad & Tobago (entity 249)
- [x] **1.31** Add YV - Venezuela (entity 148)
- [x] **1.32** Add HK - Colombia (entity 116)

#### ✅ Phase 2: Fix DXCC Entity ID Errors in dxcc.rs (COMPLETE)
- [x] **2.1** Audit for duplicate entity_id values (same ID used for different countries)
- [x] **2.2** Verify entity IDs against official ARRL DXCC list (https://www.arrl.org/files/file/DXCC/Current_Deleted.txt)
- [x] **2.3** Fix wrong entity IDs:
  - El Salvador: 62 → 74 (ARRL entity 074)
  - Guatemala: 78 → 76 (ARRL entity 076)  
  - St. Lucia: 65 → 97 (ARRL entity 097)
- [x] **2.4** Verified Barbados (62) and BVI (65) are CORRECT per ARRL

#### ✅ Phase 3: Add Missing South American Prefixes (COMPLETE)
- [x] **3.1** Add CE - Chile (entity 108)
- [x] **3.2** Add CE0 variants - Easter Island, Juan Fernandez, San Felix
- [x] **3.3** Add OA - Peru (entity 136)
- [x] **3.4** Add CP - Bolivia (entity 117)
- [x] **3.5** Add HC - Ecuador (entity 120)
- [x] **3.6** Add HC8 - Galapagos (entity 71)
- [x] **3.7** Add PZ - Suriname (entity 129)
- [x] **3.8** Add 8R - Guyana (entity 77)
- [x] **3.9** Add FY - French Guiana (entity 76)
- [x] **3.10** Add ZP - Paraguay (entity 132)
- [x] **3.11** Add CX - Uruguay (entity 144)
- [x] **3.12** Add VP8 variants - Falklands, S. Georgia, S. Sandwich, S. Orkney, S. Shetland

#### ✅ Phase 4: Add Missing African Prefixes (COMPLETE)
- [x] **4.1** Add 5H - Tanzania (entity 470)
- [x] **4.2** Add 5Z - Kenya (entity 430)
- [x] **4.3** Add 5X - Uganda (entity 286)
- [x] **4.4** Add 9J - Zambia (entity 482)
- [x] **4.5** Add 9O-9T - DR Congo (entity 114)
- [x] **4.6** Add 7Q - Malawi (entity 440)
- [x] **4.7** Add C9 - Mozambique (entity 181)
- [x] **4.8** Add Z2 - Zimbabwe (entity 452)
- [x] **4.9** Add A2 - Botswana (entity 402)
- [x] **4.10** Add V5 - Namibia (entity 464)
- [x] **4.11** Add 3DA - Eswatini (entity 468)
- [x] **4.12** Add 7P - Lesotho (entity 432)
- [x] **4.13** Add CN - Morocco (entity 446)
- [x] **4.14** Add 7R-7Y - Algeria (entity 400)
- [x] **4.15** Add 3V - Tunisia (entity 474)
- [x] **4.16** Add SU - Egypt (entity 478)
- [x] **4.17** Add ST - Sudan (entity 466)
- [x] **4.18** Add ET - Ethiopia (entity 403)
- [x] **4.19** Add 9G - Ghana (entity 424)
- [x] **4.20** Add 5N - Nigeria (entity 450)
- [x] **4.21+** Added 80+ additional African prefixes (see prefixes.rs for full list)

#### ✅ Phase 5: Add Missing Asian/Pacific Prefixes (COMPLETE)
- [x] **5.1** Add 9M2/9M4 - West Malaysia (entity 299)
- [x] **5.2** Add 9M6/9M8 - East Malaysia (entity 46)
- [x] **5.3** Add 9V - Singapore (entity 381)
- [x] **5.4** Add YB-YH - Indonesia (entity 327)
- [x] **5.5** Add V8 - Brunei (entity 345)
- [x] **5.6** Add XV/3W - Vietnam (entity 293)
- [x] **5.7** Add XU - Cambodia (entity 312)
- [x] **5.8** Add XW - Laos (entity 143)
- [x] **5.9** Add HS/E2 - Thailand (entity 387)
- [x] **5.10** Add S2 - Bangladesh (entity 305)
- [x] **5.11** Add 4S - Sri Lanka (entity 315)
- [x] **5.12** Add 9N - Nepal (entity 369)
- [x] **5.13** Add A5 - Bhutan (entity 306)
- [x] **5.14** Add AP - Pakistan (entity 372)
- [x] **5.15** Add A4 - Oman (entity 370)
- [x] **5.16** Add A6 - UAE (entity 391)
- [x] **5.17** Add A7 - Qatar (entity 376)
- [x] **5.18** Add A9 - Bahrain (entity 304)
- [x] **5.19** Add 9K - Kuwait (entity 348)
- [x] **5.20** Add HZ - Saudi Arabia (entity 378)
- [x] **5.21** Add 7O - Yemen (entity 492)
- [x] **5.22** Add OD - Lebanon (entity 354)
- [x] **5.23** Add 4X/4Z - Israel (entity 336)
- [x] **5.24** Add JY - Jordan (entity 342)
- [x] **5.25** Add YI - Iraq (entity 333)
- [x] **5.26** Add EP - Iran (entity 330)
- [x] **5.27+** Added 90+ additional Asian/Pacific prefixes including:
  - Central Asia (UN, UJ-UM, EY, EX, EZ)
  - Caucasus (4L, EK, 4J/4K)
  - Pacific Islands (3D2, A3, 5W, E5, E6, T8, V6, V7, etc.)
  - See prefixes.rs for complete list

#### ✅ Phase 6: Verify US States and Canadian Provinces (COMPLETE)
- [x] **6.1** Verify all 50 US state abbreviations in states.rs ✓
- [x] **6.2** Add Canadian province abbreviations for ARRL sections
  - Added CanadianProvince struct
  - Added all 13 provinces/territories: AB, BC, MB, NB, NL, NS, ON, PE, QC, SK, NT, NU, YT
  - Added get_province() and get_province_by_name() functions
  - Added CANADA_TOTAL constant
- [x] **6.3** DC (District of Columbia) already present ✓

#### ✅ Phase 7: Testing (COMPLETE)
- [x] **7.1** Add unit tests for all new prefixes - 10 prefix tests passing
- [x] **7.2** Test VP2M lookup returns entity 177 (Montserrat) ✓
- [x] **7.3** Test KL7 lookup returns entity 6 (Alaska) - *Alaska IS a separate DXCC entity per ARRL rules*
- [x] **7.4** Test KB9FIN lookup returns entity 291 (United States) ✓
- [x] **7.5** Verify no duplicate entity IDs cause data corruption - All entity IDs verified against ARRL list

---

## 📋 v0.2.1 Bug Fixes (2026-01-06) - COMPLETED

### Issue 1: Alaska Incorrectly Flagged as "NEW DXCC"

**Status:** 🔴 CRITICAL - Data integrity issue

**Problem:** Alaska (NL5Y with grid BP51) is being flagged as "NEW DXCC: Alaska" in the Priority Queue, but Alaska is NOT a separate DXCC entity - it's part of the United States (DXCC #291).

**Expected Behavior:** Alaska callsigns (KL7, NL7, WL7, AL7) should resolve to DXCC #291 (United States), same as continental US stations.

**Key Code Location:** `src-tauri/src/reference/dxcc.rs` or prefix lookup logic

**Root Cause:** The DXCC lookup is likely treating Alaska as a separate entity when it should be US.

---

### Issue 2: RX Messages Still Not Showing in Active QSO Panel

**Status:** � FIXED (2025-01-06)

**Problem:** When in a QSO, messages FROM the other station (RX) are not appearing in the Active QSO message log.

**Root Cause Discovered:** The WSJT-X Heartbeat message `id` field contains the **instance name** ("WSJT-X"), NOT the operator's callsign ("N5JKK"). The code was using `heartbeat.id` as `myCall`, so decodes directed at "N5JKK" never matched when `myCall` was "WSJT-X".

**Fix Applied:**
1. Modified Rust UDP listener (`src-tauri/src/udp/listener.rs`) to parse `de_call` from the WSJT-X Status message (was previously skipped)
2. Added `de_call` field to `UdpMessage::Status` enum and `StatusMessage` struct
3. Updated `commands.rs` to emit `de_call` in the `wsjtx-status` event
4. Updated `ActiveQso.tsx` to get the callsign from `status.de_call` instead of heartbeat ID

**Verification:** After rebuild, console should show:
- `[ActiveQso] Setting myCall from status.de_call: "N5JKK"` (the ACTUAL callsign)
- `[ActiveQso] ✓ RX message directed at us from ...` for incoming messages

---

### Issue 3: QSO Not Being Auto-Logged from WSJT-X

**Status:** � FIXED (2025-01-06)

**Problem:** When WSJT-X logs a QSO (via its own Log QSO button or auto-log), GoQSO was NOT receiving or processing the `qso-logged` event.

**Root Cause Discovered:** WSJT-X sends **LoggedADIF** (message type 12), NOT **QsoLogged** (message type 5)! The LoggedADIF message contains the full ADIF record as a string, which is a newer/different format than the structured QsoLogged message.

**Fix Applied:**
1. Added `parse_logged_adif()` function in `wsjtx.rs` to parse the LoggedADIF UDP message
2. Added `parse_adif_to_qso()` function in `listener.rs` to convert ADIF string to QsoLoggedMessage
3. Updated LoggedADIF handler in `listener.rs` to parse the ADIF and send to the existing QsoLogged pipeline
4. Added comprehensive debug logging at info level for QSO logging events

**Verification:** After rebuild, check Rust console for:
- `Received LoggedADIF message from WSJT-X (XX bytes)`  
- `Parsed ADIF QSO: call=... grid=... freq=... mode=...`
- Toast notification should appear in GoQSO
- QSO should appear in QSO log

---

### Issue 1: Alaska DXCC (NOT A BUG)

**Status:** ✅ CORRECT BEHAVIOR - VERIFIED AGAINST OFFICIAL ARRL LIST

**Explanation:** Alaska IS a separate DXCC entity (DXCC #6), distinct from continental United States (DXCC #291). This is per the official ARRL DXCC Current Entities list (September 2025 edition):

```
KL,AL,NL,WL#       Alaska                             NA    01,02 01    006
K,W,N,AA-AK#       United States of America           NA    06-08 03-05 291
```

The "NEW DXCC: Alaska" badge is **correct behavior** if the user hasn't worked Alaska before.

**Official Sources:**
- https://www.arrl.org/files/file/DXCC/Current_Deleted.txt (text version, machine-readable)
- https://www.arrl.org/files/file/DXCC/DXCC_Current.pdf (PDF version)
- https://www.arrl.org/country-lists-prefixes (main page)

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

## 🔴 Critical Gaps (Next Up)

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

## Future Phases

### Phase 3: Award Progress Tracking

#### DXCC Award
- [ ] Worked/Confirmed counts by band and mode
- [ ] DXCC Challenge (band-slot counting)
- [ ] Progress toward:
  - DXCC (100 confirmed)
  - DXCC Honor Roll (326+)
  - DXCC #1 (340 current entities)
- [ ] Entity cards showing: worked bands, confirmed bands

#### WAS (Worked All States)
- [ ] 50 states tracker
- [ ] Triple Play: Phone + CW + Digital confirmed
- [ ] State cards with confirmation status

#### VUCC (VHF/UHF Century Club)
- [ ] Grid squares on 6m, 2m, 70cm, etc.
- [ ] 100/200/300+ level tracking
- [ ] Grid map visualization

#### USA-CA (Counties)
- [ ] 3,077 US counties
- [ ] Progress tracking
- [ ] County map

---

### Phase 4: QSO Map Visualization

#### Technology Choice
Options:
- **Leaflet.js** - Open source, free, flexible
- **MapLibre GL** - Modern, WebGL, beautiful
- **D3.js** - Full control, steep learning

Recommendation: **MapLibre GL** for professional look

#### World Map
- [ ] QSO pins at grid square centers
- [ ] Color: red=worked, green=confirmed
- [ ] Clustering for dense areas
- [ ] Popup on click: call, date, band, mode
- [ ] Filter controls: band, mode, date range

#### Projections
- [ ] Azimuthal equidistant (ham favorite)
- [ ] Mercator (standard)
- [ ] Centered on user's QTH

#### Overlays
- [ ] Grid square grid (for VUCC)
- [ ] CQ zones
- [ ] ITU zones
- [ ] DXCC entity boundaries

#### US-Specific
- [ ] State boundaries for WAS
- [ ] County boundaries for USA-CA
- [ ] State fill colors by status

---

### Phase 5: Transmission Control

#### Mock Transmission System
- [ ] Test decode data for simulation
- [ ] Mock Reply message (no actual TX)
- [ ] Visual TX indicator
- [ ] QSO state machine (Tx1-Tx6)

#### Safety Features
- [ ] TX enable toggle (default OFF)
- [ ] Confirmation dialog before TX
- [ ] TX timeout auto-disable
- [ ] Clear TX active indicator

#### Reply Integration
- [ ] Send Reply message to WSJT-X
- [ ] Track QSO state per callsign
- [ ] Suggest appropriate Tx message
- [ ] Manual Tx selection

#### Call Button / Double-Click
- [ ] Wire Priority Queue "Call" button
- [ ] Double-click row to call
- [ ] Call confirmation with preview

---

### Phase 6: Polish & Release

- [ ] Error Handling - User-friendly error messages
- [ ] Settings Persistence - Save/load user preferences
- [ ] Offline Mode - Queue operations when no internet
- [ ] Performance Optimization - Efficient queries for large logs
- [ ] Installer Builds - Windows MSI, macOS DMG, Linux AppImage
- [ ] Documentation - User guide and README

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

### Cleanup Items
- [ ] Fix 80+ Rust warnings (unused code)
- [ ] Remove unused imports (Rust compiler warnings)
- [ ] Add proper error types instead of String errors
- [ ] Implement proper logging (tracing crate)
- [ ] Unit tests for FT8 parsing
- [ ] Unit tests for DXCC lookup
- [ ] Add integration tests for Tauri commands
- [ ] Error boundaries in React
- [ ] Proper error handling/display

### Performance Considerations
- [ ] Index optimization for QSO queries by date range
- [ ] Lazy loading for large QSO lists
- [ ] Background processing for LoTW sync

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
1. **Curated DXCC entities** - Version-controlled from ARRL official list
2. **ITU prefix allocations** - Official international prefix blocks
3. **LoTW as ground truth** - Confirmations include DXCC entity number
4. **Club Log API** (future) - Quality database with edge cases

### Authoritative Sources (in priority order)
1. **ARRL DXCC List** - https://www.arrl.org/files/file/DXCC/Current_Deleted.txt
2. **ARRL International Call Sign Series** - https://www.arrl.org/international-call-sign-series
3. **RSGB International Prefixes** - https://rsgb.org/main/operating/licensing-novs-visitors/international-prefixes/
4. **FCC Call Sign Systems** - https://www.fcc.gov/wireless/bureau-divisions/mobility-division/amateur-radio-service/amateur-call-sign-systems

### Reference Data Updates
- DXCC list changes ~1-2x per year with ARRL announcements
- We will version these changes in Git
- Users can update via app settings or manual download

---

## Contributing

This project is currently in early development. See GitHub issues for current work items.

Repository: https://github.com/texasharley/goqso
