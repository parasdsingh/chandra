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
| I-027 | bug | White ring around the panel | M2 | done |
| I-028 | bug | Week started on Tuesday in every locale | M2 | done |
| I-029 | bug | Calendar invisible to VoiceOver: gridcells had no rows | M2 | done |
| I-030 | feat | Day detail inside the calendar region, not a taller panel | M2 | open |
| I-031 | feat | Settings inside the panel, reorganised for the surface | M2 | open |
| I-032 | feat | Lunar month flow alongside solar, with correct names | M2 | open |
| I-033 | feat | Month change by scroll and drag, not arrow buttons | M2 | open |
| I-034 | bug | Moonset absence reshapes the detail with a dash and a new line | M2 | open |
| I-035 | bug | Selected day is hard to see; today needs a second attribute | M2 | open |
| I-036 | bug | Tray glyph weight does not match system menu bar icons | M2 | open |
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

### I-027 — White ring around the panel — done
A blanket `:focus-visible { outline: 2px solid var(--focus) }` in `base.css`
applied to the panel container, which `Panel` focuses on mount so the keyboard
works immediately. `--focus` is `#EDEDEF`, so the ring read as a bright white
border. Fixed by declaring focus rings per control instead of globally. The
matching blanket `:focus { outline: none }` was removed at the same time: it
would have stripped the native focus rings from the settings window's AppKit
controls.

First diagnosed only after Screen Recording permission was granted and the real
panel could be screenshotted. Two earlier guesses - an opaque webview background
and a white CSS canvas - were wrong and were reverted.

### I-028 — Week started on Tuesday — done
`localeFirstWeekday` mapped `Intl` week info with `firstDay % 7`. `getWeekInfo`
reports 1 = Monday through 7 = Sunday, and the grid is Monday-based and
zero-indexed, so Monday must map to 0: a subtraction, not a modulo. The modulo
sent Monday to 1, starting the week on a Tuesday, which no locale uses.

### I-029 — Calendar invisible to VoiceOver — done
Day cells carried `role="gridcell"` with no `role="row"` parent. That is invalid
ARIA and WebKit prunes the whole subtree, so the calendar did not exist for
assistive technology. Found by reading the live accessibility tree: 12 elements
before the fix, 193 after, with every day cell exposing its spoken label.

### I-030 to I-036 — requested changes
Raised after seeing the running app. I-032 is the substantial one: a lunar month
is not a relabelled Gregorian month, it runs new moon to new moon (amanta) or
full moon to full moon (purnimanta), is named from the rashi the Sun occupies at
that syzygy, and needs adhika masa detection for a lunar month containing no
sankranti.
