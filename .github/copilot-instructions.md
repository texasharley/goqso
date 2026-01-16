```instructions
# GitHub Copilot Instructions for GoQSO

> **📖 Full documentation**: See [CLAUDE.md](../CLAUDE.md) for complete architecture, schema, and reference data details.

## Quick Reference

| Command | Purpose |
|---------|---------|
| `npm run tauri dev` | Start dev server |
| `npm run tauri build` | Production build |
| `cargo test` | Rust unit tests |
| `npm test` | Frontend tests |

## Terminal Usage

**Always start dev servers in external windows** to avoid interrupting the server:

```powershell
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd c:\dev\goqso; npm run tauri dev"
```

## Key Rules

1. **Database**: SQLite via `sqlx` (NOT tauri-plugin-sql)
2. **Reference Data**: Use `src-tauri/resources/dxcc_entities.json` as single source of truth — never CTY.DAT
3. **Code Style**: Explicit types (no `any`), functional React components
4. **Backlog**: Only Backlog-Architect modifies TODO.md and CHANGELOG.md

## Database Access

```powershell
sqlite3 "$env:APPDATA\com.goqso.app\goqso.db" "SELECT COUNT(*) FROM qsos"
```

```
