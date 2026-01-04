# GitHub Copilot Instructions for GoQSO

## Development Environment

### Terminal Usage
- **Always start dev servers in external windows** - Use `Start-Process` or `cmd /c start` to launch dev servers in separate windows, not in the VS Code integrated terminal. This prevents interrupting the server when running other commands.

Example:
```powershell
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd c:\dev\qso-logger\goqso; npm run tauri dev"
```

### Project Structure
- **Tauri 2.x** backend in `src-tauri/`
- **React + TypeScript** frontend in `src/`
- **SQLite** database via tauri-plugin-sql

### Reference Data Philosophy
- Do NOT use CTY.DAT from country-files.com
- Use curated DXCC entity data in `src-tauri/src/reference/`
- Source reference data from ARRL official lists and ITU allocations
- LoTW confirmations are the ground truth for DXCC credit

### Code Style
- Rust: Follow standard Rust conventions
- TypeScript: Use functional components with hooks
- Prefer explicit types over `any`

### Testing
- Run `cargo test` for Rust unit tests
- Run `npm test` for frontend tests

### Building
- Dev: `npm run tauri dev`
- Build: `npm run tauri build`
