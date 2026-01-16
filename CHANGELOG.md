# Changelog

All notable changes to GoQSO will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.4.0] - 2026-01-16

### Added

- **Grid-based DXCC disambiguation** for KG4 prefix (Guantanamo vs USA)
  - KG4 + Guantanamo grid (FK/FL area) → Guantanamo Bay (105)
  - KG4 + any other grid or no grid → USA (291)
- **`prefix_rules.json`** — Authoritative prefix-to-DXCC mapping (806 rules, 339 entities)
- **`is_valid_grid()` validation** — Rejects FT8 messages stored as grids
- **`repair_qso_data` command** — Repairs historical data integrity issues
- **FCC state display in Band Activity** — Real-time state lookup for US callsigns
- **`dxcc_as_i32()` helper** — Bridges ARRL string format to database integers
- **Grid-based location lookup** — `lookup_location(call, grid)` uses grid as primary source
- **Reference data generation scripts** — `generate_reference.py` for DXCC/states/prefixes
- **QsoDetailModal component** — Detailed QSO view with all fields
- **QsoLog column management** — Draggable, resizable columns with persistence
- **Agent communication files** — COMMUNICATION.md for multi-agent workflow

### Changed

- **DXCC entity ID format** — Internal uses ARRL 3-digit strings, external uses integers
- **Modular commands structure** — Split monolithic `commands.rs` into `commands/` directory
- **Reference data architecture** — Generated from JSON source files, not hand-coded
- **State field population strategy** — NULL until LoTW confirmation (architectural decision)

### Fixed

- 🐛 **BUG-001: NULL DXCC from WSJT-X** — String-to-integer binding issue in `insert_qso_from_wsjtx()`
- 🐛 **BUG-002: RR73 stored as grid** — Historical data repaired, validation now in place
- 🐛 **BUG-003: KG4 always Guantanamo** — Now uses grid-based disambiguation
- 🐛 **BUG-004: 9Y4DG wrong entity** — Stale data repaired to Trinidad (90)
- 🐛 **ENH-001: FCC state not displaying** — Fixed DXCC type mismatch in event emission

### Security

- No security changes in this release

---

## [0.3.0] - 2026-01-14

### Added

- Initial WSJT-X UDP integration
- ADIF import/export functionality
- LoTW download sync
- DXCC entity reference data (340 entities)
- Awards tracking (DXCC, WAS, VUCC)
- FCC ULS database integration
- Basic QSO logging from WSJT-X

---

## [0.2.0] - 2026-01-10

### Added

- Tauri 2.x application scaffold
- React + TypeScript + Tailwind frontend
- SQLite database with sqlx
- Basic UI components

---

## [0.1.0] - 2026-01-05

### Added

- Initial project setup
- Core architecture design
