# GoQSO Logging UX Design

## Design Philosophy: "Zero-Friction, Maximum Insight"

DXKeeper is **powerful but overwhelming**. GoQSO will be **powerful AND delightful**.

### Core Principles

1. **Auto-Enrichment** - Never make the user enter what we can derive
2. **Progressive Disclosure** - Essential info at a glance, details on demand  
3. **Real-Time Feedback** - Instant dupe checking, award progress, confirmation status
4. **Keyboard-First** - Power users never touch the mouse
5. **Memory** - Remember previous QSOs with each station

---

## Feature Comparison: GoQSO vs DXKeeper

| Feature | DXKeeper | GoQSO |
|---------|----------|-------|
| Auto DXCC lookup | ✅ | ✅ Auto on log |
| Previous QSO history | Manual lookup | ✅ **Inline indicator** |
| Award status on log | No | ✅ **New DXCC/Band badge** |
| LoTW status | Status column | ✅ **Color-coded badge** |
| Edit QSO | Separate form | ✅ **Inline + Modal** |
| Bulk operations | Limited | ✅ **Multi-select + batch edit** |
| Search | Basic | ✅ **Fuzzy + field-specific** |
| UI | Win32 dated | ✅ **Modern dark theme** |
| Speed | Slow on large logs | ✅ **SQLite + virtual scroll** |

---

## QSO Log Table Design

### Visible Columns (Default)

| Column | Width | Content |
|--------|-------|---------|
| **Badges** | 48px | Award/dupe/confirmation icons |
| **Date/Time** | 140px | 2025-01-03 14:32 |
| **Call** | 100px | W1ABC |
| **Band** | 60px | 20m |
| **Mode** | 60px | FT8 |
| **RST S/R** | 80px | -12/-08 |
| **Country** | 120px | United States |
| **Grid** | 70px | FN31 |
| **Name** | 100px | John |
| **LoTW** | 50px | ✓/○/✗ badge |

### Badge System (First Column)

```
🆕 = New DXCC entity (never worked before)
🎯 = New band-slot (worked on other bands, not this one)  
🔄 = Duplicate (same call/band/mode within 24h)
⭐ = Previous QSO exists with this call
```

### Confirmation Status Badges

```
LoTW:  ✓ green = confirmed | ○ yellow = uploaded | ✗ gray = not sent
eQSL:  Same pattern
QRZ:   Same pattern
Paper: 📬 = sent | 📫 = received
```

---

## Interaction Patterns

### 1. Single Click Row
- Highlights row
- Shows **Quick Preview Panel** on right (if enabled)

### 2. Double-Click Row  
- Opens **Full Edit Modal**
- All fields editable
- Shows Previous QSO History with this station

### 3. Right-Click Row
- Context menu: Edit, Delete, Lookup on QRZ, Send QSL, Mark as...

### 4. Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `/` | Focus search |
| `n` | New QSO |
| `e` | Edit selected |
| `Delete` | Delete selected (with confirm) |
| `↑/↓` | Navigate rows |
| `Enter` | Open detail modal |
| `Escape` | Close modal / clear selection |
| `Ctrl+F` | Advanced search |
| `Ctrl+E` | Export selected |

---

## QSO Detail/Edit Modal

### Layout: Two-Column with Tabs

```
┌─────────────────────────────────────────────────────────────┐
│  QSO with W1ABC                                    [X Close]│
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────┐  ┌─────────────────────────────────┤
│  │ ESSENTIAL           │  │ PREVIOUS QSOs WITH W1ABC       ││
│  │                     │  │                                 ││
│  │ Date: [2025-01-03]  │  │ 2024-06-15 40m FT8 -12/-10     ││
│  │ Time: [14:32:00]    │  │ 2023-11-22 20m FT8 -08/-06     ││
│  │ Band: [20m     ▼]   │  │ 2023-03-10 15m CW  599/599     ││
│  │ Mode: [FT8     ▼]   │  │                                 ││
│  │ Freq: [14.074]      │  │ Total: 4 QSOs on 3 bands       ││
│  │ RST Sent: [-12]     │  │                                 ││
│  │ RST Rcvd: [-08]     │  │ ─────────────────────────────  ││
│  │                     │  │ AWARD STATUS                   ││
│  │ LOCATION            │  │                                 ││
│  │ Country: USA        │  │ 🆕 First QSO on 20m with USA   ││
│  │ State: [MA     ▼]   │  │ DXCC: 287/340 (Mixed)          ││
│  │ Grid: [FN31]        │  │ WAS: 48/50                     ││
│  │ CQ Zone: 5          │  │                                 ││
│  │ ITU Zone: 8         │  │ ─────────────────────────────  ││
│  │                     │  │ CONFIRMATION STATUS            ││
│  │ STATION INFO        │  │                                 ││
│  │ Name: [John]        │  │ LoTW:  ○ Uploaded 2025-01-03   ││
│  │ QTH: [Boston]       │  │ eQSL:  ✗ Not sent              ││
│  │                     │  │ QRZ:   ✗ Not sent              ││
│  └─────────────────────┘  └─────────────────────────────────┤
│                                                             │
│  [🗑 Delete]                      [Cancel]  [💾 Save Changes]│
└─────────────────────────────────────────────────────────────┘
```

### Tabs for Additional Fields

- **Essential** (default) - Core fields shown above
- **Extended** - Propagation, SOTA/POTA/IOTA, Contest info
- **My Station** - My call, grid, rig, antenna, power
- **Notes** - Free-form comments, user data

---

## Add QSO Modal (Manual Entry)

For voice/CW QSOs not from WSJT-X:

### Smart Features

1. **Callsign lookup on blur** - Auto-fill country, name, grid from:
   - Our previous QSOs with this call
   - QRZ.com API (if configured)
   - HamQTH API (free fallback)

2. **Frequency → Band auto-detection** - Type 14.250, band auto-selects 20m

3. **UTC clock** - Shows current UTC, one-click to use "now"

4. **Duplicate warning** - "You worked W1ABC on 20m FT8 today at 10:32"

---

## Search & Filter System

### Quick Search Bar
- Searches: call, country, grid, name, notes
- Fuzzy matching: "W1A" finds "W1ABC", "W1AW", etc.

### Advanced Search (Ctrl+F)
```
┌──────────────────────────────────────────────┐
│ ADVANCED SEARCH                              │
│                                              │
│ Callsign: [       ] contains/exact/prefix    │
│ Country:  [United States    ▼]               │
│ Band:     [☐160 ☐80 ☑40 ☑20 ☐15 ☐10]        │
│ Mode:     [☑FT8 ☑FT4 ☐CW ☐SSB ☐RTTY]        │
│ Date:     [2024-01-01] to [2025-01-03]       │
│ LoTW:     [○All ○Confirmed ○Pending ○None]   │
│                                              │
│           [Clear]  [Search]                  │
└──────────────────────────────────────────────┘
```

### Smart Filters (Quick Access)
- "Unconfirmed on LoTW"
- "New DXCC this year"
- "Need to QSL"
- "Today's QSOs"

---

## Real-Time Features

### 1. Live Dupe Check
When WSJT-X sends a decode, we check our log and show:
- 🔄 if dupe (same band/mode/call within 24h)  
- ⭐ if worked before on different band/mode
- 🆕 if new call entirely

### 2. Toast Notifications
On QSO logged:
```
┌────────────────────────────────────────┐
│ ✓ QSO Logged: W1ABC on 20m FT8        │
│   🆕 New DXCC: United States!          │
│   DXCC Progress: 288/340               │
└────────────────────────────────────────┘
```

### 3. Status Bar
Bottom of log shows:
```
Total: 1,234 QSOs | Showing: 156 | Selected: 3 | DXCC: 287 | WAS: 48 | Grids: 412
```

---

## Implementation Priority

### Phase 1: Core Excellence (Now)
- [ ] Badge system for award status
- [ ] Previous QSO indicator  
- [ ] Enhanced edit modal with history panel
- [ ] Keyboard shortcuts
- [ ] Column customization

### Phase 2: Smart Entry
- [ ] Manual QSO entry form
- [ ] Callsign lookup integration
- [ ] Live dupe checking
- [ ] Duplicate warning

### Phase 3: Advanced
- [ ] Bulk operations
- [ ] Advanced search modal
- [ ] Right-click context menu
- [ ] Export with filters

---

## Why This Beats DXKeeper

1. **Instant Context** - See previous QSOs and award status without leaving the log
2. **Visual Clarity** - Color-coded badges > text columns
3. **Speed** - Keyboard-first, no hunting through menus
4. **Modern UX** - Feels like 2025, not 1995
5. **Auto-Enrichment** - Less typing, more operating
6. **Real-Time Feedback** - Know immediately if it's a new one
