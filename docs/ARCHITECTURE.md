# Architecture

Status: **awaiting approval.** No implementation code is written until this is accepted.
Product name **Chandra**, bundle id `com.parasdsingh.chandra`, front end **SolidJS + Vite + TS**.
Rationale for every choice below is in [DECISIONS.md](DECISIONS.md); measured evidence is in
[RESEARCH.md](RESEARCH.md).

---

## 1. Shape of the system

```
                        macOS menu bar
      [ moon ]   [ Surya ]   [ Mangala ]        <- independent tray items
          |          |            |
          +----------+------------+
                     |  click
              +-------------------+
              |  panel window     |   one reusable shell,
              |  320 x auto       |   different content per subject
              +-------------------+
                     |  IPC (typed commands)
      ---------------------------------------------
                     |
      +--------------+---------------+
      |        src-tauri (shell)     |   tray, windows, commands, settings
      +--------------+---------------+
                     |
      +--------------+---------------+
      |     crates/almanac (domain)  |   no Tauri, no UI, no async
      +--------------+---------------+
                     |
      +--------------+---------------+
      |     crates/ephemeris (core)  |   Swiss Ephemeris FFI, isolated
      +------------------------------+
```

Rule enforced by the crate graph: **the domain does not know Tauri exists, and the UI does not
know Swiss Ephemeris exists.** `crates/almanac` and `crates/ephemeris` build and test on Linux
with no macOS dependency, which is what makes CI cheap (D-012) and the test suite fast.

---

## 2. Workspace layout

```
moon-phases/
├── Cargo.toml                     workspace root
├── Makefile                       dev / build / install / test / lint
├── docs/
│   ├── ARCHITECTURE.md            this file
│   ├── DECISIONS.md               decision log
│   ├── RESEARCH.md                verified facts
│   ├── ROADMAP.md                 milestones
│   └── ISSUES.md                  issue tracker
├── crates/
│   ├── ephemeris/                 Swiss Ephemeris boundary. The only unsafe code.
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs          Mutex singleton, ephe path, sid_mode, flag checking
│   │       ├── body.rs            Graha enum <-> SE body ids (normalises the u32/i32 mess)
│   │       ├── julian.rs          JD <-> civil, delta-T boundary
│   │       ├── position.rs        longitude, latitude, speed, provenance
│   │       ├── phenomena.rs       illumination, phase angle
│   │       ├── risetrans.rs       rise / set / transit
│   │       └── error.rs
│   ├── almanac/                   pure domain. No FFI, no unsafe.
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── zodiac.rs          Rashi, Nakshatra, Pada, boundary arithmetic
│   │       ├── phase.rs           phase naming from illumination + elongation
│   │       ├── roots.rs           bracketing + Brent (D-016)
│   │       ├── events.rs          ingress, station, syzygy detection
│   │       ├── day.rs             DayDetail assembly
│   │       ├── month.rs           MonthView assembly
│   │       └── cache.rs           keyed LRU
│   ├── geo/                       location chain, tzdb centroids, city search
│   └── glyph/                     tray icon rendering (tiny-skia -> RGBA)
├── src-tauri/                     thin shell
│   ├── src/
│   │   ├── main.rs
│   │   ├── tray.rs                tray items, live icon updates
│   │   ├── panel.rs               panel window lifecycle, positioning, auto-hide
│   │   ├── commands.rs            IPC surface (the only pub API to the front end)
│   │   ├── settings.rs            typed, versioned, migrated
│   │   └── state.rs               AppState, engine handle, cache
│   ├── resources/
│   │   ├── ephe/                  sepl_18.se1, semo_18.se1   (1.7 MB)
│   │   ├── zone1970.tab           timezone -> representative lat/lon
│   │   └── cities15000.bin        city search index
│   └── tauri.conf.json
└── src/                           front end — SolidJS + Vite + TypeScript
    ├── components/
    ├── styles/tokens.css
    └── ipc/                       generated TS types, one binding per command
```

---

## 3. Layer contracts

### 3.1 `crates/ephemeris`

The only place `unsafe` appears. Everything above it is safe Rust.

```rust
pub struct Engine { /* holds the process-wide SE lock */ }

pub struct Position {
    pub longitude: f64,      // sidereal degrees, [0, 360)
    pub latitude:  f64,
    pub speed:     f64,      // degrees/day; negative = retrograde
    pub source:    Source,   // Swieph | Moshier   <- D-006, never dropped
}

pub enum Source { Swieph, Moshier }

impl Engine {
    pub fn position(&self, jd_ut: f64, body: Graha) -> Result<Position, Error>;
    pub fn illumination(&self, jd_ut: f64) -> Result<Illumination, Error>;
    pub fn rise_set(&self, jd_ut: f64, body: Graha, obs: Observer)
        -> Result<RiseSet, Error>;
    pub fn ayanamsa(&self, jd_ut: f64) -> f64;
    pub fn reconfigure(&self, cfg: SiderealConfig) -> Result<(), Error>;
}
```

Invariants:

- Global SE state (`ephe_path`, `sid_mode`, `topo`) is set inside the lock, never outside.
- Requested flags are compared against returned flags on every call; the difference becomes
  `Source`. A silent Moshier fallback is impossible to miss (R-02).
- Body ids are normalised to `i32` at this boundary, so the crate's inconsistent constant
  types (R-01) never leak upward.
- `RiseSet` distinguishes *no event today* (circumpolar / no rise) from *error*. These are
  different states and are rendered differently.

### 3.2 `crates/almanac`

Pure functions over `Engine`. Deterministic, no I/O, no clock reads, fully testable.

```rust
pub struct DayDetail {          // v1 scope, D-010
    pub date:         Date,
    pub phase:        Phase,        // name + illuminated fraction
    pub moonrise:     Option<Instant>,
    pub moonset:      Option<Instant>,
    pub nakshatra:    Span<Nakshatra>,   // value + entry + exit
    pub rashi:        Span<Rashi>,
    pub provenance:   Source,
}

pub struct Span<T> {
    pub value: T,
    pub entry: Instant,      // may precede the day
    pub exit:  Instant,      // may follow the day
}

pub struct MonthView {
    pub month: YearMonth,
    pub days:  Vec<DayCell>,     // phase glyph data + illumination only
    pub events: Vec<Event>,      // for graha panels: ingress, station
}
```

`Span` is the reason entry/exit times work correctly at month edges: a nakshatra that began
three days before the month starts still reports its true entry instant, not a clamped one.

### 3.3 `src-tauri` IPC surface

Every command is `async`, returns `Result<T, AppError>`, and runs the engine call on
`spawn_blocking` (D-005). The front end has no other way to reach the domain.

| Command | Returns |
|---|---|
| `bootstrap()` | `Bootstrap`: settings, resolved location, subject, picker lists, glyphs |
| `moon_month(anchor_unix_ms, offset, first_weekday)` | `MoonMonth` |
| `graha_month(graha, anchor_unix_ms, offset, first_weekday)` | `GrahaMonth` |
| `day_detail(graha, year, month, day)` | `DayDetail` |
| `snapshot(unix_ms)` | `Snapshot`: what the menu bar draws |
| `update_settings(settings)` | `Bootstrap`, re-read after applying |
| `search_cities(query, limit)` | `Vec<Place>` |
| `request_device_location()` | `Resolved`, whatever the attempt left in force |
| `close_panel()` | `()` |

A month is addressed by an anchor instant and an offset, not by a year and a number: a lunar
month runs between syzygies and has neither. `first_weekday` travels in the request because the
grid is laid out here and that is the one presentation fact this layer cannot derive (D-021).

`src/ipc/types.ts` is hand-written. Generating it from the Rust types was the original plan and
is not what happened; instead `src-tauri/tests/contract.rs` serialises a real value of every
payload type, reduces it to its field names and leaf types, and compares that against a
committed fixture. A field added, removed or renamed in Rust fails that test and the diff names
exactly what to change in TypeScript.

---

## 4. Data flow, one panel open

```
tray click
   -> panel.rs: position via TrayCenter, show window, AppHandle::show()
   -> front end mounts, calls month_view(subject, y, m)
        -> cache hit?  return immediately
        -> miss: spawn_blocking
              -> lock engine
              -> per day: illumination at local noon, rise/set
              -> events: bracket scan + Brent refine (D-016)
              -> unlock, insert into cache
   -> render grid
   -> idle: prefetch month-1 and month+1 in background
day click
   -> day_detail(subject, date)   (usually already warm from the month pass)
   -> panel animates height, detail renders below the grid
blur
   -> window.hide() + AppHandle::hide()   (R-05 macOS trap)
```

---

## 5. Performance plan

Measured baseline (R-04): `swe_calc_ut` = 7.9 us; a 31-day 15-minute Moon grid = 23.5 ms.

| Concern | Approach | Target |
|---|---|---|
| Panel open, warm | serve from LRU cache | < 5 ms |
| Panel open, cold month | adaptive bracketing, not brute grid | < 30 ms |
| Month switch | prev/next prefetched while idle | perceived instant |
| Tray icon refresh | only on day rollover; the watcher looks at the clock at most ten minutes apart, because a sleeping machine does not advance a `thread::sleep` | ~0% idle CPU |
| Memory | LRU bounded to 24 month-views per subject | bounded |
| Startup | engine init is lazy; tray icon drawn from cached illumination first | < 200 ms to visible |

Cache keys name the subject, the month system, the month's first civil day and the weekday the
grid opens on - the month's own identity rather than the request that reached it. Three kinds
share the cache: a month per subject, the resolved grid, and the lunar days, the last two shared
by every subject drawn on that month.

The configuration is *not* in the key. A generation counter carries it instead: any settings
change bumps it, which hides every existing entry at once, and a value computed under one
generation is refused if it is offered after another has begun. That is what stops a month
computed under one ayanamsa being stored as if it were computed under the next.

---

## 6. Error handling

- `crates/ephemeris`: `thiserror` enum. Never panics, never `unwrap`s on FFI output.
  The SE error buffer is captured into the error value.
- `crates/almanac`: propagates. Root-finding that fails to converge returns
  `Err(NoConvergence)` — it does not return a guessed time.
- `src-tauri`: maps to `AppError` with a stable machine-readable `code` plus a display message.
- Front end: renders a specific inline state per code. There is no generic "something went
  wrong" catch-all, because every failure here has a meaningful cause the user can act on
  (no location, date out of range, degraded ephemeris).
- Degraded-but-valid states are **not** errors: `Source::Moshier`, no moonrise today,
  circumpolar. These render as annotations.

---

## 7. Settings

- Read and written by `settings.rs` itself at
  `~/Library/Application Support/<bundle-id>/settings.json`. No store plugin: settings only
  ever cross the boundary through commands, so a plugin would add a second unguarded write
  path to the same file.
- Typed in Rust, carries `schema_version`, migrated by explicit numbered steps.
  No serde defaults papering over missing fields.

```
schema_version,
location { mode: automatic|manual,
           place?: {label, zone, lat, lon, elevation},
           elevation?: metres, applied on top of whichever step resolved },
sidereal { ayanamsa, node_type },
calendar { month_system: solar|amanta|purnimanta },
tray { subjects: [Graha], colour_mode: bool }
```

`launch_at_login` and `time_format` were defined here and never wired to anything - no UI
could set either, and nothing called the autostart plugin - so both are gone rather than
left as switches with no handle.

---

## 8. UI structure

Progressive disclosure. Each surface does one thing.

| Level | Surface | Contains |
|---|---|---|
| 0 | menu bar | live moon disc; enabled graha glyphs |
| 1 | panel, month grid | date + phase glyph per cell; today ringed; month switcher |
| 2 | panel, expanded | the six v1 fields for the selected day |
| 3 | settings window | general, location, astrology, grahas, about |

- Month switcher: chevrons plus the month label; clicking the label opens a year/month picker.
- Keyboard: arrows move by day, up/down by week, PgUp/PgDn by month, `T` jumps to today,
  `Esc` closes. Focus ring is visible and follows selection.
- Graha panels reuse the identical shell; only the cell content and the detail fields differ.
- Accessibility: all colour pairs meet WCAG AA on the `#0A0A0B` ground; phase is never
  communicated by shape alone — the day detail always names it.

Design tokens (D-011): ground `#0A0A0B`, text `#EDEDEF`, muted `#8A8A90`,
hairline `rgba(255,255,255,0.08)`, accent reserved for "today" only.
Type: system UI stack, tabular numerals for all times and figures.

---

## 9. Testing

Zero-regression is a hard requirement, so the domain is tested before the UI exists.

| Layer | Method |
|---|---|
| `ephemeris` | golden vectors cross-checked against `swetest` CLI output, committed as JSON |
| `almanac` zodiac | exact boundary cases: 13.3333 deg, 30 deg, 359.9999 deg, wraparound |
| `almanac` roots | property tests — ingress instants monotone; longitude at ingress equals the boundary within 1e-6 deg; rise < transit < set |
| `almanac` events | full-year runs for every graha; assert no missed or duplicated stations |
| accuracy | assertions carry explicit documented tolerances, not eyeballed constants |
| `src-tauri` | command-level tests with a fixed clock and fixed location |
| front end | component tests; no snapshot tests of times (they would encode a timezone) |

All domain tests run on Linux in CI. macOS runners only build the DMG on a release tag.

---

## 10. Build and distribution

```
make dev        cargo tauri dev
make build      cargo tauri build --bundles app,dmg
make install    build, ad-hoc codesign, copy to /Applications, clear quarantine
make test       cargo test --workspace  +  front-end tests
make lint       cargo fmt --check, cargo clippy -D warnings, eslint, tsc --noEmit
```

- Ad-hoc signature (`codesign -s -`) so first launch is right-click-Open, not a hard block.
- Tray items are built in Rust only; `tauri.conf.json` declares none, to avoid the duplicate
  tray icon bug (R-05).
- CI on push: fmt, clippy, workspace tests — Ubuntu.
  CI on tag: macOS build, DMG uploaded to a GitHub release.

---

## 11. Known risks

| Risk | Impact | Mitigation |
|---|---|---|
| CoreLocation unreliable when ad-hoc signed (R-06) | no auto location | fallback chain D-007; app fully usable without it |
| `swiss-eph` is young (0.2.1) | upstream churn | pinned `=0.2.1`, lockfile committed, isolated behind `engine.rs` |
| Tray icon may render at 44 pt instead of 22 pt | blurry or oversized glyph | verified in milestone M1 before any glyph design work; fallback is 22x22 |
| Tauri duplicate tray icon on macOS | two icons | build tray in Rust only (R-05) |
| Panel height animation jank | feels cheap | measure before shipping; fall back to a fixed-height detail pane |

---

## 12. Open items

None blocking. D-013 and D-014 are resolved; the GitHub remote is deferred by choice (D-018).
