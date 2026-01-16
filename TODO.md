# GoQSO - Development Roadmap

> ⚠️ **BACKLOG AUTHORITY**: This file is managed exclusively by the **Backlog-Architect** agent.
> No other agent may create, modify, or close items in TODO.md or CHANGELOG.md.
> See `.github/agents/Backlog-Architect.agent.md` for governance rules.

---

## Current Status (2026-01-16)

| Milestone | Status |
|-----------|--------|
| Core Architecture | ✅ Complete |
| WSJT-X Integration | ✅ Complete |
| ADIF Import/Export | ✅ Complete |
| LoTW Download Sync | ✅ Complete |
| DXCC Entity Data (340 entities) | ✅ Complete |
| DXCC Entity ID Format | ✅ Complete (strings internal, ints external) |
| DXCC Prefix Data | ✅ Complete (grid-based KG4 disambiguation) |
| v0.3.0 UI/UX | ✅ Complete |
| Tech Debt Refactoring | 🔨 4/7 Complete |
| **BUG-001: NULL DXCC** | ✅ **DONE** — Fixed + Data Repaired (42 QSOs) |
| **BUG-002: Grid Validation** | ✅ **DONE** — 3 invalid grids cleared |
| **BUG-003: KG4 Prefix** | ✅ **DONE** — Grid-based disambiguation implemented |
| **BUG-004: 9Y Prefix** | ✅ **DONE** — Was stale data, lookup was correct |
| **BUG-005: State Lookup** | ✅ **CLOSED** — By design (see architectural decision) |
| **ENH-001: Band Activity FCC** | ✅ **DONE** — FCC state now displays for US calls |
| **BUG-006: RR73 in Grid Display** | 🔴 **CRITICAL** — FT8 message showing as grid |
| **BUG-007: KC4 Antarctica** | 🟠 **HIGH** — Needs grid-based disambiguation |
| Award Progress Dashboard | ⏳ Blocked (DXCC bugs open) |
| LoTW Upload | ⏳ Blocked (DXCC bugs open) |

**Build Status:** `cargo check` 0 errors, `cargo test` 30 passed (reference), `npm run build` SUCCESS

### Log Data Quality Summary (2026-01-16)

> **🔴 2 BUGS OPEN.** Do NOT submit to LoTW.

| Issue | Count | Status |
|-------|-------|--------|
| NULL DXCC | 0 | ✅ **REPAIRED** (was 42) |
| Invalid Grid (RR73) in DB | 0 | ✅ **REPAIRED** (was 3) |
| RR73 in Band Activity display | ? | 🔴 **BUG-006** — Frontend validation missing |
| Wrong DXCC (KC4 calls) | ? | 🟠 **BUG-007** — Grid-based disambiguation needed |
| Wrong DXCC (9Y4DG) | 0 | ✅ **REPAIRED** — Fixed to Trinidad (90) |
| Wrong DXCC (KG4BHR) | 0 | ✅ **REPAIRED** — Fixed to USA (291) |
| Empty Grid | 13 | 🟢 Expected (some QSOs have no grid) |
| Missing State (US) | ~22 | 🟢 **BY DESIGN** — Populated from LoTW confirmation |

---

## 🚨 OPEN BUGS

> **🔴 2 CRITICAL BUGS OPEN** — Do NOT submit to LoTW until fixed.

---

### BUG-006: RR73 Still Appearing in Grid Column 🔴 CRITICAL

**Status:** 🟢 READY

**Symptom:** User sees "RR73" in the Grid column of Band Activity display (screenshot 2026-01-16).

**Impact:** 🔴 CRITICAL — Data integrity violation. RR73 is an FT8 message, not a Maidenhead grid. If stored, causes wrong DXCC lookup.

**Root Cause Analysis Required:**

BUG-002 repaired **historical** data in database and confirmed validation exists in `insert_qso_from_wsjtx()`. However, RR73 is appearing in **live Band Activity display**. Two possible causes:

1. **Frontend display bug:** WSJT-X decode events pass grid through without validation. Band Activity shows raw `grid` field from decode.

2. **New data leaking through:** Validation might be bypassed somewhere, storing RR73 again.

**Investigation Steps:**
1. Check if RR73 exists in database: `SELECT * FROM qsos WHERE grid = 'RR73'`
2. If NO database records: Bug is frontend-only (display not validated)
3. If YES database records: Bug is backend (validation bypassed)

**Acceptance Criteria:**
- [ ] RR73, RRR, 73, R+xx, R-xx NEVER appear in Grid column (Band Activity OR Log)
- [ ] Grid validation occurs at DISPLAY time for Band Activity (decode events)
- [ ] Grid validation occurs at STORAGE time for QSOs (existing check)
- [ ] Regression test: Mock WSJT-X decode with grid="RR73" → displays as empty/dash

**Files to Investigate:**
- `src/components/BandActivity.tsx` — Display logic for grid field
- `src-tauri/src/commands/udp.rs` — Where decode event is emitted
- `src-tauri/src/reference/mod.rs` — `is_valid_grid()` function

**Priority:** 🔴 CRITICAL — Must fix before LoTW submission

---

### BUG-007: KC4 Shows Antarctica Instead of USA 🟠 HIGH

**Status:** 🟢 READY

**Symptom:** KC4SWL in Band Activity shows "ANTARCTICA" but FCC database confirms this is a regular US ham.

**Evidence:**
- FCC ULS: `KC4SWL|NC|LAWNDALE` — Licensed in North Carolina
- Current prefix lookup: KC4 → Antarctica (013)
- User observed KC4SWL showing both "Antarctica" and "North Carolina" in same session (prefix wrong, FCC correct)

**Root Cause:** Same class of bug as KG4 (BUG-003). The KC4 prefix is currently mapped to Antarctica (013) in `prefix_rules.json`, but:
- Most KC4 calls are regular US hams (FCC database)
- Only actual Antarctic stations (KC4AAA, KC4USV) operating from Antarctica should be 013
- Antarctic stations can be identified by grid: second letter A, B, or C (≤60°S latitude)

**Solution Pattern:** Same as KG4 fix:
1. Change `prefix_rules.json`: KC4 entity_id "013" → "291" (USA default)
2. Add `is_antarctic_grid()` function: Returns true if grid second letter is A, B, or C
3. Add KC4 handling in `lookup_location()`: KC4 + Antarctic grid = Antarctica (013)
4. Regenerate `prefixes.rs` from JSON
5. Add tests

**Antarctic Grid Pattern:**
```
Maidenhead grid second letter indicates latitude band:
A = 90°S to 80°S (South Pole)
B = 80°S to 70°S (Antarctic continent)  
C = 70°S to 60°S (Antarctic coastal)
D+ = North of 60°S (NOT Antarctica)

Examples: KC29 (Antarctica), PA00 (Antarctica), EM15 (NOT Antarctica)
```

**Acceptance Criteria:**
- [ ] `lookup_location("KC4SWL", "")` returns USA (291)
- [ ] `lookup_location("KC4SWL", "EM95")` returns USA (291)
- [ ] `lookup_location("KC4AAA", "KC29")` returns Antarctica (013)
- [ ] `lookup_location("KC4USV", "LB46")` returns Antarctica (013)
- [ ] `prefix_rules.json` updated: KC4 → "291"
- [ ] Tests added: `test_kc4_grid_based_disambiguation()`
- [ ] `prefixes.rs` regenerated

**Files to Change:**
- `src-tauri/resources/prefix_rules.json` — KC4 rule: "013" → "291"
- `src-tauri/src/reference/mod.rs` — Add `is_antarctic_grid()`, KC4 handling
- `src-tauri/src/reference/prefixes.rs` — Regenerate from JSON

**Priority:** 🟠 HIGH — Affects DXCC accuracy for US hams

---

### BUG-001: DXCC Not Stored from WSJT-X UDP ✅ DONE

**Status:** ✅ DONE (2026-01-15) — Validated & Data Repaired

**Symptom:** 42 QSOs (22.6%) had NULL DXCC despite having clearly parseable callsigns.

**Root Cause:** In `insert_qso_from_wsjtx()` ([commands/udp.rs#L367](src-tauri/src/commands/udp.rs#L367)), `lookup.dxcc` (an `Option<String>` containing `"291"`) was bound directly to an INTEGER column. SQLite silently stored NULL because it couldn't coerce the string.

**Why `add_qso()` worked but `insert_qso_from_wsjtx()` didn't:**

| Function | Location | DXCC Binding | Result |
|----------|----------|--------------|--------|
| `add_qso()` | qso.rs:224 | `lookup.dxcc_as_i32()` | ✅ Correct |
| `insert_qso_from_wsjtx()` | udp.rs:367 | `lookup.dxcc` (string) | ❌ NULL |

**The Fix:**
```rust
// BEFORE (BUG):
.bind(lookup.dxcc)  // String "291" → NULL in INTEGER column

// AFTER (FIXED):
let dxcc_int = lookup.dxcc_as_i32();  // "291" → 291
.bind(dxcc_int)  // Integer binds correctly
```

**Files Changed:**
- `src-tauri/src/commands/udp.rs` — Added `dxcc_as_i32()` conversion at line 369
- `src-tauri/src/commands/qso.rs` — Added `repair_qso_data()` command for existing data
- `src-tauri/src/main.rs` — Registered `repair_qso_data` command
- `src-tauri/src/reference/mod.rs` — Added regression test `test_dxcc_as_i32_conversion`

**Validation Performed:**
- [x] Code review: Verified all 4 INSERT paths use correct integer binding
- [x] `cargo test` — 83 passed (added 1 regression test)
- [x] `cargo check` — 0 errors
- [x] Regression test covers leading-zero conversion ("001" → 1, "006" → 6)
- [x] **Data Repair Executed:** 42 QSOs repaired, 0 NULL DXCC remaining

---

### BUG-002: Grid Validation Missing at Storage Point ✅ DONE

**Status:** ✅ DONE (2026-01-15) — Historical data repaired

**Symptom:** "RR73" stored as gridsquare value for 9Y4DG and KG4BHR.

**Evidence (database query 2026-01-15, BEFORE repair):**
| Call | Grid | DXCC | Problem |
|------|------|------|---------|
| 9Y4DG | RR73 | 249 | FT8 message stored as grid → wrong DXCC |
| KG4BHR | RR73 | 105 | FT8 message stored as grid → wrong DXCC |

**Current State:** Grid validation **EXISTS** in `insert_qso_from_wsjtx()` at line 380:
```rust
let validated_grid = if is_valid_grid(&qso.grid) { Some(&qso.grid) } else { None };
```

**Root Cause:** The bad grids were **historical data** from before validation was added.

**Resolution:** `repair_qso_data` cleared 3 invalid grids (RR73 values).

**Validation:**
- [x] Grid validation exists in UDP handler
- [x] `repair_qso_data` clears invalid grids
- [x] 3 grids cleared (KG4BHR x2, 9Y4DG x1)

---

### BUG-003: KG4BHR Shows Guantanamo Instead of USA ✅ DONE

**Status:** ✅ DONE (2026-01-15) — Grid-based disambiguation implemented

**Symptom:** KG4BHR (2x3 callsign) incorrectly showed as Guantanamo Bay (105).

**Root Cause:** The `prefix_rules.json` had KG4 → 105 (Guantanamo) for ALL KG4 calls.

**FCC Evidence:** Queried 1.5M FCC records — ALL 16,499 KG4 callsigns are registered to US addresses. The "single letter = Guantanamo" rule was a myth/outdated.

**Solution Implemented:**
1. Changed `prefix_rules.json`: KG4 → 291 (USA) as default
2. Added grid-based override in `lookup_location()`:
   - KG4 + Guantanamo grid (FK19-FK39, FL19-FL30) → 105 (Guantanamo)
   - KG4 + any other grid or no grid → 291 (USA)
3. Regenerated `prefixes.rs` from JSON
4. Updated database: 2 KG4BHR records now show USA (291)

**Files Changed:**
- `src-tauri/resources/prefix_rules.json` — KG4 rule entity_id "105" → "291"
- `src-tauri/src/reference/mod.rs` — Added `GUANTANAMO_GRIDS`, `is_guantanamo_grid()`, KG4 handling in `lookup_location()`
- `src-tauri/src/reference/prefixes.rs` — Regenerated

**Tests Added:**
- `test_bug_003_kg4bhr()` — KG4BHR returns USA (291)
- `test_kg4_grid_based_disambiguation()` — Grid-based logic tests

**Validation:**
- [x] `cargo test reference::` — 30 passed
- [x] Database: KG4BHR now shows DXCC 291, UNITED STATES OF AMERICA

---

### BUG-004: 9Y4DG Shows St. Kitts Instead of Trinidad ✅ DONE

**Status:** ✅ DONE (2026-01-15) — Was stale data, lookup was correct

**Symptom:** 9Y4DG showed St. Kitts & Nevis (249) instead of Trinidad (90).

**Root Cause:** The prefix lookup `lookup_call_full("9Y4DG")` was CORRECT (returning "090"). The database had stale data from before the 9Y prefix was fixed in an earlier session.

**Verification:**
```rust
let result = lookup_call_full("9Y4DG");
assert_eq!(result.dxcc.as_deref(), Some("090")); // ✅ PASSES
```

**Resolution:** Manual database update:
```sql
UPDATE qsos SET dxcc = 90, country = 'TRINIDAD & TOBAGO' WHERE call = '9Y4DG';
```

**Validation:**
- [x] `lookup_call_full("9Y4DG")` returns "090" (Trinidad) ✅
- [x] Database: 9Y4DG now shows DXCC 90, TRINIDAD & TOBAGO ✅

---

### BUG-005: FCC State Not Populated in QSOs ✅ CLOSED (By Design)

**Status:** ✅ CLOSED (2026-01-15) — Architectural decision: NOT a bug

**Original Symptom:** US callsigns show no state in QSO Log.

---

## 📐 ARCHITECTURAL DECISION: State Field Population

**Decision Date:** 2026-01-15  
**Decision:** State field in QSO records should remain NULL until confirmed by LoTW.

### Rationale

The STATE field in ADIF represents **where the station operated from**, NOT where they are licensed:

| Scenario | FCC License | Actual Operation | Correct STATE |
|----------|-------------|------------------|---------------|
| W9ABC at home | WI | WI | WI |
| W9ABC POTA in CA | WI | CA | **CA** |
| W9ABC /MM | WI | International waters | **NULL** |

**Using FCC database for STATE would be WRONG** for portable operations.

### Data Integrity Principle

From CLAUDE.md First Principles:
> "Data integrity over convenience — Reject bad data; don't try to 'fix' it silently."

Better to have NULL (honest: we don't know) than incorrect data.

### Tiered Population Strategy

| Tier | When | State Source | Stored? |
|------|------|--------------|--------|
| **1** | At QSO time | None | NULL |
| **2** | LoTW Sync | LoTW confirmation | ✅ Authoritative |
| **3** | Manual | User entry | ✅ User override |

### For WAS Award

To claim Worked All States, you need **LoTW-confirmed** state anyway. NULL state for unconfirmed QSOs is correct behavior.

### What This Means for Code

**QSO Log (Backend):** ✅ Correct as-is
- `state: None` in `add_qso()` is CORRECT
- `state: None` in `insert_qso_from_wsjtx()` is CORRECT
- State populated when LoTW sync runs (already implemented)

**Band Activity (Frontend):** See ENH-001
- FCC lookup for DISPLAY is fine (situational awareness)
- Display state is not stored in QSO record

---

### ENH-001: Band Activity FCC State Display ✅ DONE

**Status:** ✅ DONE (2026-01-15) — Validated by user

**Root Cause:** The `wsjtx-decode` event emitted `dxcc` as string ("291") but frontend expected integer (291). The `isHomeCountry` check failed due to type mismatch.

**Fix:** Changed `lookup.dxcc` → `lookup.dxcc_as_i32()` in [udp.rs#L143](src-tauri/src/commands/udp.rs#L143).

**Validation:**
- [x] FCC lookup works in BandActivity.tsx
- [x] US callsigns show state abbreviation (SC, TX, GA, FL, NC, TN, AL verified)
- [x] Brazil (PY7ZC) correctly shows "—" for state
- [x] State is display-only — NOT written to QSO record

---

### BUG (CLOSED): DXCC Entity ID Format Inconsistency ✅ FIXED

**Status:** ✅ FIXED (2026-01-15)

**Solution:** Internal reference data uses ARRL 3-digit strings ("001", "291"). Database/ADIF uses integers. Bridge via `dxcc_as_i32()` helper.

**Verification:**
- `cargo test` — 82 passed
- `cargo clippy` — no new warnings
- All reference data consistent with ARRL format

---

### BUG (CLOSED): DXCC Prefix Data Incomplete & Incorrect ✅ FIXED

**Status:** ✅ FIXED (2026-01-15)

**Solution:** Created `prefix_rules.json` with 806 rules covering 339/340 entities (99.7%). Only Spratly Islands (247) missing — no ITU prefix allocation (contested territory).

**Verification:**
- `python scripts/validate_prefix_rules.py` — 0 errors
- `cargo test reference::` — All tests pass

---

## 📋 DXCC Reference Data Governance

### Authoritative Sources

| Data Type | Authority | URL |
|-----------|-----------|-----|
| DXCC Entity List | ARRL | https://www.arrl.org/files/file/DXCC/Current_Deleted.txt |
| Prefix Allocations | ITU | ITU Radio Regulations, Article 19 |
| Entity Changes | ARRL DXCC Desk | Announced in QST magazine |

### Update Frequency

| Change Type | Frequency | Example |
|-------------|-----------|---------|
| New DXCC entity | ~1-2 per decade | Kosovo (Z6) added 2016 |
| Entity deleted | Very rare | Soviet republics consolidated |
| Prefix reallocation | Rare | PJ split into PJ2/4/5/6/7 in 2010 |

### Target Data Flow

```
ARRL Current_Deleted.txt (authoritative)
        ↓
scripts/fetch_arrl_dxcc.py
        ↓
src-tauri/resources/dxcc_entities.json (SSOT for entities)
        +
src-tauri/resources/prefix_rules.json (manual curation for prefixes)
        ↓
scripts/generate_prefixes.py
        ↓
src-tauri/src/reference/prefixes.rs (GENERATED - DO NOT EDIT)
```

---

## 🔧 FIX PLAN: DXCC Prefix Data Rebuild

### Phase 1: Create prefix_rules.json ✅ DONE

**Goal:** Single source of truth for all prefix-to-entity mappings.

**Completed:** 2026-01-15

**Results:**
- ✅ Created `src-tauri/resources/prefix_rules.json` — 806 rules
- ✅ Coverage: 339/340 active entities (99.7%)
- ✅ Missing only Spratly Is. (247) — no ITU prefix allocation (contested)
- ✅ Entity IDs verified against `dxcc_entities.json` — 0 errors
- ✅ Includes disambiguation suffixes (HK0M, VK9X, CE0Y, VP8F, etc.)
- ✅ Includes ITU block expansions (AA-AL, KH0-KH9, DA-DR, etc.)
- ✅ Created `scripts/build_prefix_rules.py` for regeneration
- ✅ Created `scripts/validate_prefix_rules.py` for validation

**Validation:**
```
python scripts/validate_prefix_rules.py
# Coverage: 339/340 (99.7%), Errors: 0, Warnings: 1 (cosmetic)
cargo test reference:: 
# 26 passed, 0 failed
```

### Phase 2: Generate prefixes.rs from JSON ✅ DONE

**Goal:** Replace hand-coded prefixes.rs with generated code.

**Completed:** 2026-01-15

**Results:**
- ✅ Created `scripts/generate_prefixes.py`
- ✅ Generated `prefixes.rs` from JSON — 1062 lines, 806 rules
- ✅ Header includes "DO NOT EDIT", timestamp, stats, authority URL
- ✅ All 82 tests pass (11 prefix-specific)
- ✅ Entity IDs cross-validated against JSON source

**Validation:**
```
cargo test  # 82 passed, 0 failed
python scripts/validate_prefix_rules.py  # 0 errors, 339/340 coverage
```

### Phase 3: Validation Suite ⏸️ BLOCKED

**Goal:** Add programmatic tests that validate against JSON source (prevent hard-coded assertion errors).

**BLOCKED BY:** BUG: DXCC Entity ID Format Inconsistency

> ⚠️ Do not proceed until entity ID format is standardized to ARRL 3-digit strings.

**Acceptance Criteria:**
- [ ] Add test that reads `prefix_rules.json` and verifies sample lookups
- [ ] Add coverage test confirming 339/340 entities have rules
- [ ] Update `validate_prefixes.py` for JSON↔Rust round-trip check
- [ ] `cargo clippy` clean for prefixes.rs

**Status:** ⏸️ BLOCKED

### Phase 3: Validation Suite

- [ ] `validate_prefixes.py` verifies:
  - All entity_ids exist in `dxcc_entities.json`
  - All 340 active entities have at least one prefix rule
  - No duplicate prefixes at same priority
- [ ] `cargo test prefix` covers all continents
- [ ] Cross-validate 20 callsigns against WSJT-X

### Phase 4: Documentation

- [ ] CLAUDE.md updated with data governance section
- [ ] Annual update checklist documented

**Files:**
- `src-tauri/resources/prefix_rules.json` (new, authoritative)
- `scripts/generate_prefixes.py` (new)
- `scripts/validate_prefixes.py` (update)
- `src-tauri/src/reference/prefixes.rs` (regenerate)

**Effort:** XL  
**Status:** 🔴 READY — Start with Phase 1

---

### TASK: Audit & Complete Prefix Disambiguation Rules 🟡 MEDIUM

> ⚠️ **Merged into:** FIX PLAN Phase 1 above
> Disambiguation rules will be part of prefix_rules.json

---

## 🔧 TECH DEBT

### TASK: Add .gitignore entries for temp files 🟢 LOW

**Context:** Cleanup tasks deleted temp files; prevent future accumulation.

**Acceptance Criteria:**
- [ ] Add `*.log` pattern
- [ ] Add `temp_*.adi` pattern

**Effort:** XS  
**Status:** 🟢 READY

---

### TASK: Consolidate duplicate content between docs ✅ DONE

**Context:** copilot-instructions.md was slimmed down (2026-01-13). Verify no stale references remain.

**Acceptance Criteria:**
- [x] copilot-instructions.md references CLAUDE.md for details
- [x] No conflicting information between files

**Effort:** XS  
**Status:** ✅ DONE (2026-01-13)

---

## 🏗️ REFACTORING (2026-01-13 Codebase Audit)

> **Audit Summary:** Frontend structure is good. Backend `commands.rs` (3,257 lines) needs splitting.
> Reference data files lack generation scripts despite "DO NOT EDIT" headers.

### REFACTOR: Split commands.rs into modules ✅ DONE

**Context:** 3,257-line file with 34 Tauri commands violates thin-handler principle. Contains helper functions, business logic, and message handlers that should be in separate modules.

**Implemented Structure:**
```
src-tauri/src/commands/
├── mod.rs           # 30 lines - Re-exports all commands
├── adif.rs          # ADIF import/export
├── awards.rs        # Award progress commands
├── band_activity.rs # Band activity/decodes
├── diagnostics.rs   # Debug/diagnostic commands
├── fcc.rs           # FCC sync commands
├── lotw.rs          # LoTW sync commands
├── qso.rs           # QSO CRUD commands
├── settings.rs      # get_setting, set_setting
├── state.rs         # QSO state machine
├── time_utils.rs    # Time helper functions
└── udp.rs           # UDP listener commands
```

**Validation (2026-01-14):**
- [x] Created `src-tauri/src/commands/` module folder with 12 submodules ✓
- [x] Root `commands.rs` replaced by `commands/mod.rs` (30 lines) ✓
- [x] `cargo check` passes with 0 errors ✓
- [x] App functionality unchanged ✓

**Files:** `src-tauri/src/commands.rs` → 12 files  
**Effort:** L  
**Status:** ✅ DONE (2026-01-14)

---

### REFACTOR: Create generation scripts for reference data ✅ DONE

**Context:** `dxcc.rs` and `prefixes.rs` headers say "DO NOT MANUALLY EDIT" but there are no generation scripts. Manual edits have caused entity ID bugs.

**Implemented:**
- `scripts/generate_reference.py` — Generates `dxcc.rs` and `states.rs` from JSON sources
- `resources/dxcc_entities.json` — 402 DXCC entities (authoritative source)
- `resources/us_states.json` — 50 US states
- `resources/canadian_provinces.json` — 13 Canadian provinces/territories

**Validation (2026-01-14):**
- [x] Created `scripts/generate_reference.py` ✓
- [x] Created `resources/us_states.json` for US states ✓
- [x] Created `resources/canadian_provinces.json` for CA provinces ✓
- [x] Generate `dxcc.rs` from `dxcc_entities.json` ✓
- [x] Generate `states.rs` from JSON sources ✓
- [x] Generation instructions added to CLAUDE.md ✓
- [x] `cargo test reference::` — 26/26 tests pass ✓

**Note:** `prefixes.rs` remains manually curated — the JSON data lacks sufficient disambiguation detail for compound callsigns (e.g., HK0M vs HK0 for different entities). This is documented in CLAUDE.md.

**Files:** `scripts/generate_reference.py`, 3 JSON files  
**Effort:** M  
**Status:** ✅ DONE (2026-01-14)

---

### REFACTOR: Split QsoLog.tsx ✅ DONE

**Context:** 1,383-line component had multiple responsibilities: table, columns, filters, drag-drop, detail modal.

**Implemented Structure:**
```
src/components/
├── QsoLog.tsx          # 695 lines — Main table component
├── QsoLogColumns.tsx   # 113 lines — Column definitions, SortableColumnItem
├── QsoLogFilters.tsx   # 124 lines — Filter panel component
├── QsoLogHelpers.tsx   #  97 lines — Badge components, formatters
└── QsoDetailModal.tsx  # 435 lines — QSO detail view modal
src/lib/
└── constants.ts        # BAND_ORDER, MODE_OPTIONS, CONFIRM_OPTIONS, STATE_NAMES
```

**Validation (2026-01-14):**
- [x] Created `QsoLogColumns.tsx` — column definitions, SortableColumnItem ✓
- [x] Created `QsoLogFilters.tsx` — filter panel component ✓
- [x] Created `QsoLogHelpers.tsx` — badges, formatters ✓
- [x] Created `QsoDetailModal.tsx` — detail modal (~435 lines) ✓
- [x] Created `src/lib/constants.ts` with shared constants ✓
- [x] `npx tsc --noEmit` — 0 errors ✓
- [x] `npm run build` — Success ✓
- [x] No functionality changes ✓

**⚠️ Note:** QsoLog.tsx is 695 lines vs 600 target. Further extraction would require significant prop drilling. Accepted as practical outcome (50% reduction from 1,383 lines).

**Files:** `src/components/QsoLog.tsx` → 5 files  
**Effort:** M  
**Status:** ✅ DONE (2026-01-14)

---

### REFACTOR: Extract BandActivity business logic to hook 🟡 MEDIUM

**Context:** BandActivity.tsx (451 lines) has FCC lookup logic and worked-status tracking mixed with presentation.

**Acceptance Criteria:**
- [ ] Create `src/hooks/useBandActivity.ts`
- [ ] Move FCC lookup, worked tracking, decode processing to hook
- [ ] Move `stateNames` map to `src/lib/constants.ts`
- [ ] BandActivity.tsx is primarily presentation (~250 lines)

**Files:** `src/components/BandActivity.tsx`, new hook  
**Effort:** M  
**Status:** 🟢 READY

---

### REFACTOR: Consolidate type definitions 🟡 MEDIUM

**Context:** `Decode` and `DecodeEvent` interfaces defined inline in BandActivity.tsx instead of /types.

**Acceptance Criteria:**
- [ ] Create `src/types/decode.ts`
- [ ] Move Decode, DecodeEvent interfaces from BandActivity.tsx
- [ ] Export from types/index.ts if exists
- [ ] Update imports

**Files:** New `src/types/decode.ts`  
**Effort:** S  
**Status:** 🟢 READY

---

### REFACTOR: Create shared constants file ✅ DONE

**Context:** `BAND_ORDER`, `stateNames`, `MODE_OPTIONS` were duplicated or scattered across components.

**Implemented in `src/lib/constants.ts`:**
- `BAND_ORDER` — Band sorting order
- `BAND_OPTIONS` — Available bands for filtering
- `MODE_OPTIONS` — Available modes for filtering
- `CONFIRM_OPTIONS` — Confirmation status options
- `STATE_NAMES` — US state abbreviation to name mapping
- `HOME_DXCC` (291) — US entity ID constant
- `CANADA_DXCC` (1) — Canada entity ID constant

**Validation (2026-01-14):**
- [x] Created `src/lib/constants.ts` ✓
- [x] Moved BAND_ORDER from QsoLog.tsx ✓
- [x] Added STATE_NAMES (previously stateNames in BandActivity.tsx) ✓
- [x] HOME_DXCC and CANADA_DXCC defined ✓
- [x] Imports updated in QsoLog.tsx and related files ✓

**Effort:** S  
**Status:** ✅ DONE (2026-01-14)

---

### TASK: Decide on /components/ui folder 🟢 LOW

**Context:** Empty `src/components/ui/` folder suggests shadcn/ui was planned but not added.

**Acceptance Criteria:**
- [ ] Decide: use shadcn/ui or remove folder
- [ ] If using: add base components (button, input, dialog)
- [ ] If not: delete empty folder

**Effort:** S  
**Status:** 🟢 READY

---

## 📋 FEATURE BACKLOG

### Epic: Award Progress Dashboard 🟠 HIGH

**Why Critical:** Visual motivation is the killer feature that differentiates GoQSO.

**Features:**
- [ ] DXCC progress UI: X/340 worked, Y confirmed (by band/mode)
- [ ] WAS progress UI: X/50 states worked, Y confirmed
- [ ] VUCC progress: grid squares on 6m+
- [ ] Progress bars with targets (100, 200, 300 levels)
- [ ] Entity/state cards showing confirmation status

**Backend (exists):**
- [x] `get_dxcc_progress` command
- [x] `get_was_progress` command

**Effort:** L  
**Status:** 🟢 READY

---

### Epic: LoTW Upload 🟡 MEDIUM

**Why:** Complete the LoTW integration loop.

**⚠️ BLOCKED:** Must not submit test data to LoTW. Only real QSO data can be uploaded.

**Features:**
- [ ] Queue QSOs for upload (sync_queue table exists)
- [ ] TQSL CLI integration for signing
- [ ] Track upload status (pending/uploaded/failed)
- [ ] Batch upload support
- [ ] Error handling for TQSL exit codes

**Files:** `src-tauri/src/lotw/tqsl.rs` (stub exists)  
**Effort:** L  
**Status:** 🔴 BLOCKED

---

### Epic: QSO Map Visualization 🟡 MEDIUM

**Why:** Visual gratification, better than QSOmap.org.

**Technology:** MapLibre GL (WebGL, modern, beautiful)

**Features:**
- [ ] World map with QSO pins at grid centers
- [ ] Color coding: red=worked, green=confirmed
- [ ] Clustering for dense areas
- [ ] Filter by band/mode/date range
- [ ] Azimuthal equidistant projection option
- [ ] Grid square overlay for VUCC
- [ ] US state map for WAS

**Effort:** XL  
**Status:** 🟢 READY

---

### Epic: Transmission Control 🟢 LOW

**Why:** Enable "Call" button functionality in Priority Queue.

**Safety:** TX enable toggle (default OFF), confirmation dialogs, timeout auto-disable.

**Features:**
- [ ] Send Reply message to WSJT-X
- [ ] Track QSO state per callsign (Tx1-Tx6)
- [ ] Call button / double-click to call
- [ ] Mock TX system for testing (no actual transmission)

**Effort:** L  
**Status:** 🟢 READY

---

## 🔮 FUTURE PHASES (Post-MVP)

### Phase: FT8 Direct Integration

> **Vision:** Operate FT8 directly from GoQSO without WSJT-X.

See CLAUDE.md "Long-Term Vision: Standalone Radio Operation" for architecture.

- [ ] IC-7300 CI-V CAT control
- [ ] Audio I/O via `cpal` crate
- [ ] Waterfall display (WebGL FFT)
- [ ] Pure Rust FT8 codec (encode/decode)
- [ ] Period timing synchronization

---

### Phase: Additional Award Programs

- [ ] IOTA (Islands on the Air)
- [ ] SOTA (Summits on the Air)
- [ ] POTA (Parks on the Air) — API at pota.app
- [ ] CQ WAZ (Worked All Zones)
- [ ] CQ WPX (Worked Prefixes)
- [ ] USA-CA (3,077 US counties)

---

### Phase: Contest Logging

- [ ] Cabrillo export
- [ ] Dupe checking
- [ ] Rate tracking
- [ ] N+1 multiplier display

---

### Phase: Cloud & Mobile

- [ ] Cloud sync for multi-device
- [ ] iOS build (Tauri 2.x)
- [ ] Android build
- [ ] Web interface for remote viewing

---

### Phase: External Integrations

- [ ] eQSL.cc sync
- [ ] QRZ.com logbook sync
- [ ] ClubLog integration

---

## 📚 Reference

> Full technical documentation is in [CLAUDE.md](CLAUDE.md).

### Key Sections in CLAUDE.md
- Database schema and location
- LoTW API reference
- DXCC reference data philosophy
- Development commands
- Directory structure

### Archived Work
See [ARCHIVE.md](ARCHIVE.md) for completed items.

---

## Contributing

Repository: https://github.com/texasharley/goqso
