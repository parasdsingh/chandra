# Issue tracker

Status: `open` | `in-progress` | `blocked` | `done`.
Type: `feat` | `bug` | `risk` | `chore` | `question`.

New issues append to the table and get a detail section only when they need one.

| ID | Type | Title | Milestone | Status |
|---|---|---|---|---|
| I-001 | question | Product name and bundle identifier | M0 | done |
| I-002 | question | Front-end framework selection | M0 | done |
| I-003 | chore | Workspace skeleton, Makefile, Ubuntu CI | M0 | open |
| I-004 | feat | `Engine` wrapper over `swiss-eph` with provenance | M0 | open |
| I-005 | chore | Generate and commit `swetest` golden vectors | M0 | open |
| I-006 | risk | Tray icon scale: 22 pt vs 44 pt RGBA | M1 | open |
| I-007 | feat | Tray item + live moon disc rendering | M1 | open |
| I-008 | feat | Panel window: position, auto-hide, no dock/Cmd-Tab | M1 | open |
| I-009 | risk | Duplicate tray icon on macOS | M1 | open |
| I-010 | feat | Zodiac: rashi, nakshatra, pada, boundary arithmetic | M2 | open |
| I-011 | feat | Root finder: bracket + Brent | M2 | open |
| I-012 | feat | `DayDetail` assembly, v1 fields only | M2 | open |
| I-013 | feat | `MonthView` assembly + LRU cache + prefetch | M2 | open |
| I-014 | feat | Location chain incl. tz centroid fallback | M2 | open |
| I-015 | risk | CoreLocation reliability when ad-hoc signed | M2 | open |
| I-016 | feat | City search over bundled GeoNames index | M2 | open |
| I-017 | feat | Month grid UI + switcher + keyboard nav | M2 | open |
| I-018 | feat | Day detail panel + height animation | M2 | open |
| I-019 | feat | Settings window and versioned store | M2 | open |
| I-020 | feat | Graha tray glyphs, hand-drawn vector paths | M3 | open |
| I-021 | feat | Graha events: ingress, station, combustion | M3 | open |
| I-022 | feat | Graha panel content | M3 | open |
| I-023 | chore | Accessibility and contrast audit | M4 | open |
| I-024 | chore | `make install`, ad-hoc signing, autostart | M4 | open |
| I-025 | chore | Release workflow, DMG on tag | M4 | open |
| I-026 | chore | Create private GitHub remote and push | M4 | blocked |

---

### I-001 — Product name and bundle identifier — done
Resolved: display name **Chandra**, identifier `com.parasdsingh.chandra`, repo stays
`moon-phases`. Identifier is frozen. See [D-013](DECISIONS.md#d-013).

### I-002 — Front-end framework selection — done
Resolved: SolidJS + Vite + TypeScript, hand-written CSS with tokens.
See [D-014](DECISIONS.md#d-014).

### I-006 — Tray icon scale
Tauri `Image` takes raw RGBA with pixel dimensions. Unverified whether macOS treats a 44x44
buffer as 44 pt (oversized) or as 22 pt @2x (correct). Must be settled in M1, before glyph
design, because it determines the drawing grid every glyph is built on.
Fallback: render 22x22 and accept softness on Retina.

### I-009 — Duplicate tray icon
Upstream tauri#8982 and tauri#10912: declaring a tray in `tauri.conf.json` *and* building one
in Rust yields two items on macOS, one of them inert. Mitigation is to build in Rust only.
Needs an explicit check once multiple graha items exist, since the bug reports predate
multi-tray usage.

### I-015 — CoreLocation when ad-hoc signed
`tauri-plugin-geolocation` lists macOS support and Apple requires
`NSLocationUsageDescription` in `Info.plist`. Whether CoreLocation answers for an
ad-hoc-signed, non-notarised bundle is untested and cannot be tested without a real build.
Not a blocker: [D-007](DECISIONS.md#d-007) makes the app fully correct without it.
Outcome to record here once M2 builds a real bundle.

### I-026 — GitHub remote
Deferred by choice ([D-018](DECISIONS.md#d-018)). CI workflows are written in M0 but stay
inert until a remote exists. Unblock with `gh repo create moon-phases --private`.
