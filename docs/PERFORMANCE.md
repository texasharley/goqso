# GoQSO Performance Architecture

> **Goal**: The most responsive ham radio logging app ever built.
> **Target**: App loads in < 500ms, all operations feel instant.

## Current Performance Profile (Dev Mode)

| Phase | Current | Target | Notes |
|-------|---------|--------|-------|
| Window visible | ~4s | <100ms | Tauri dev compilation |
| React mount | ~2s | <200ms | Vite HMR overhead |
| DB ready | ~1s | <100ms | Already async, good |
| First paint | ~24s total | <500ms | Unacceptable in dev |

## Root Causes Identified

### 1. Dev Mode Overhead (MAJOR - 80% of delay)
- Tauri recompiles Rust on every change (~10-15s)
- Vite dev server has HMR overhead
- Debug builds are 28MB vs ~5MB release

**Solution**: Production builds are fast. Dev mode slowness is acceptable for now.

### 2. Blocking Operations in Frontend
- `SyncStatus` polls `get_sync_status` immediately on mount
- `Dashboard` polls DB status every 500ms before ready
- Multiple components independently check DB readiness

**Solution**: Single source of truth for DB state in App.tsx, pass down via context.

### 3. No Query Caching
- Every tab switch re-fetches all data
- Award calculations recalculated on every render

**Solution**: Implement query cache with staleness tracking.

### 4. Unoptimized Database Queries
- `get_sync_status` runs 3 separate queries
- No prepared statements
- Award progress queries can be expensive

**Solution**: Combined queries, SQLite query caching.

## Performance Rules (MANDATORY)

### Rule 1: No Blocking Startup Operations
```rust
// ❌ BAD - Blocks app startup
fn main() {
    let db = init_db().await; // Blocks!
}

// ✅ GOOD - Async with loading state
fn main() {
    tauri::async_runtime::spawn(async { init_db().await });
}
```

### Rule 2: Single DB Ready Source
```tsx
// ❌ BAD - Multiple components poll independently
function Dashboard() { useEffect(() => checkDbReady(), []); }
function SyncStatus() { useEffect(() => checkDbReady(), []); }

// ✅ GOOD - App.tsx manages state, passes via context
function App() {
  const [dbReady, setDbReady] = useState(false);
  return <DbContext.Provider value={dbReady}>...</DbContext.Provider>;
}
```

### Rule 3: Lazy Load Heavy Components
```tsx
// ❌ BAD - Import everything upfront
import { AwardsMatrix } from "./AwardsMatrix";
import { QsoLog } from "./QsoLog";

// ✅ GOOD - Lazy load tabs not visible
const AwardsMatrix = lazy(() => import("./AwardsMatrix"));
const QsoLog = lazy(() => import("./QsoLog"));
```

### Rule 4: Debounce Expensive Operations
```tsx
// ❌ BAD - Fetch on every keystroke
onChange={(e) => searchQsos(e.target.value)}

// ✅ GOOD - Debounce search
onChange={(e) => debouncedSearch(e.target.value)}
```

### Rule 5: Virtual Scroll for Large Lists
```tsx
// ❌ BAD - Render all 10,000 QSOs
{qsos.map(q => <QsoRow key={q.id} />)}

// ✅ GOOD - Virtual scroll
<VirtualList items={qsos} renderItem={(q) => <QsoRow />} />
```

### Rule 6: Combine Backend Queries
```rust
// ❌ BAD - 3 round trips
let count = query("SELECT COUNT...").await;
let last = query("SELECT value...").await;
let has_creds = query("SELECT COUNT...").await;

// ✅ GOOD - Single query
let result = query("SELECT 
    (SELECT COUNT(*) FROM qsos WHERE...) as pending,
    (SELECT value FROM settings WHERE key='lotw_last_download') as last_dl,
    EXISTS(SELECT 1 FROM settings WHERE key='lotw_username') as has_creds
").await;
```

### Rule 7: Index All Filtered Columns
```sql
-- Every WHERE clause column needs an index
CREATE INDEX idx_qsos_call ON qsos(call);
CREATE INDEX idx_qsos_date ON qsos(qso_date DESC);
CREATE INDEX idx_qsos_dupe ON qsos(call, band, mode, qso_date);
```

### Rule 8: Measure Before Optimizing
```rust
// Add timing to critical paths
let start = std::time::Instant::now();
// ... operation
log::info!("Operation took {:?}", start.elapsed());
```

## Implementation Priority

### Phase 1: Quick Wins (This Session)
- [ ] Combine `get_sync_status` into single query
- [ ] Remove polling from SyncStatus (only fetch once on mount)
- [ ] Add React.lazy() for tab components

### Phase 2: Architecture (Next Session)
- [ ] Create DbContext for centralized state
- [ ] Implement simple query cache in Zustand
- [ ] Add performance timing logs to backend

### Phase 3: Scale Prep (When > 1000 QSOs)
- [ ] Virtual scroll for QSO log
- [ ] Paginated awards matrix
- [ ] Background award recalculation

## Benchmarking Commands

```powershell
# Time a production build
Measure-Command { npm run tauri build }

# Check release binary size
Get-ChildItem "src-tauri\target\release\goqso.exe" | Select Length

# Profile database queries
& "$env:TEMP\sqlite\sqlite3.exe" "$env:APPDATA\com.goqso.app\goqso.db" ".timer on" "SELECT * FROM qsos LIMIT 100;"
```

## Performance Testing Checklist

Before each release:
- [ ] Cold start time < 2s (production build)
- [ ] Tab switch < 100ms
- [ ] Search response < 50ms
- [ ] Import 1000 QSOs < 5s
- [ ] Awards matrix render < 200ms

## Production vs Dev Mode

| Metric | Dev Mode | Production |
|--------|----------|------------|
| Binary size | 28 MB | ~5 MB |
| Startup | 15-25s | 1-2s |
| Hot reload | Yes | No |
| Source maps | Yes | No |

**Always test performance on production builds!**
