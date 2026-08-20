# Decision log

One row per decision. Status: `accepted` | `open` | `superseded`.
Decisions marked **open** block implementation of the areas they touch.

| ID | Decision | Status |
|---|---|---|
| [D-001](#d-001) | Tauri v2 (Rust + web UI), not native Swift | accepted |
| [D-002](#d-002) | Swiss Ephemeris via `swiss-eph` crate, SWIEPH data files bundled | accepted |
| [D-003](#d-003) | Sidereal, Lahiri ayanamsa, True Rahu/Ketu as defaults | accepted |
| [D-004](#d-004) | Geocentric for panchanga, topocentric for rise/set | accepted |
| [D-005](#d-005) | Engine is a `Mutex`-guarded singleton on a blocking pool | accepted |
| [D-006](#d-006) | Ephemeris provenance is reported per result, never silent | accepted |
| [D-007](#d-007) | Location resolves through an ordered fallback chain | accepted |
| [D-008](#d-008) | Tray icons rendered as macOS template images by default | accepted |
| [D-009](#d-009) | Moon permanent in menu bar; each graha its own toggleable item | accepted |
| [D-010](#d-010) | v1 day detail is minimal; panchanga fields are not computed yet | accepted |
| [D-011](#d-011) | Opaque dark surface, no vibrancy | accepted |
| [D-012](#d-012) | Private repo, unsigned local build, ad-hoc codesign | accepted |
| [D-013](#d-013) | Product name | **open** |
| [D-014](#d-014) | Front-end framework | **open** |
| [D-015](#d-015) | Time scale handling: UT vs TT | accepted |
| [D-016](#d-016) | Event times found by bracket + Brent refinement | accepted |
| [D-017](#d-017) | No background work, no notifications in v1 | accepted |

---

### D-001
**Tauri v2 (Rust + web UI), not native Swift.**

- Driver: a Windows build is wanted later.
- Swift the *language* runs on Windows, but SwiftUI and AppKit do not. A Swift menu bar app
  would require a full UI rewrite to ship on Windows. Tauri keeps one UI codebase.
- Cost accepted: ~12 MB bundle, ~120 MB RSS, versus ~5 MB / ~30 MB for native.
- Correction to the original comparison: Tauri tray icons are **not** a static PNG set.
  `TrayIcon::set_icon` takes runtime RGBA, so the moon disc is drawn per day. See R-05.

### D-002
**Swiss Ephemeris via `swiss-eph` 0.2.1, with `sepl_18.se1` + `semo_18.se1` bundled.**

- Sub-arcsecond accuracy was a stated requirement. Verified in R-03.
- Data files add 1.7 MB and cover 1800–2399.
- The crate is young (0.2.1, 15% documented). Mitigated by D-005: all of it sits behind our
  own `engine` module, so replacing it touches one file.
- Version is pinned exactly (`=0.2.1`) and `Cargo.lock` is committed.

### D-003
**Sidereal zodiac. Lahiri (Chitrapaksha) ayanamsa. True Rahu/Ketu.**

- Lahiri is the Indian government standard and the most widely used.
- True node shows real retrograde wobble; mean node moves uniformly at -3'11"/day.
- Both are user-switchable in settings. Every ayanamsa Swiss Ephemeris supports is offered.
- Ketu is computed as Rahu + 180 deg, not requested separately.

### D-004
**Geocentric positions for panchanga; topocentric only for rise/set.**

- Traditional panchanga is computed geocentric. Lunar parallax is up to ~1 deg, which would
  visibly shift nakshatra boundary times if topocentric positions were used.
- `swe_rise_trans` handles observer parallax internally; that is correct and expected there.
- Rise/set uses Swiss Ephemeris default flags = **upper limb with refraction**, the almanac
  convention. Not disc-centre.

### D-005
**The Swiss Ephemeris handle is a process-wide singleton behind a `Mutex`, invoked from a
blocking pool.**

- The SE C library is not thread-safe: `swe_set_ephe_path`, `swe_set_sid_mode` and
  `swe_set_topo` mutate global state.
- A single `Mutex<Engine>` serialises all access. Calls run under
  `tauri::async_runtime::spawn_blocking` so the UI thread never blocks.
- Rejected alternative: a dedicated actor thread with a channel. Equivalent correctness,
  more moving parts, and concurrency here is trivially low (one popover at a time).
- Config changes (ayanamsa, node type, location) take the lock, reset global state, and
  invalidate the cache in the same critical section.

### D-006
**Every computed value carries the ephemeris that actually produced it.**

- Swiss Ephemeris silently degrades to Moshier outside the data file range and reports this
  only in the returned flag word, not as an error (R-02).
- The engine compares requested flags to returned flags and tags each result
  `Swieph` or `Moshier`.
- The UI shows a precision note when a displayed value is not Swieph-backed.
- This exists because silently presenting degraded data as authoritative is the specific
  failure mode this app must not have.

### D-007
**Location resolves through an ordered chain; the first hit wins and nothing blocks the UI.**

1. Manual override, if the user has set one. Authoritative, never overridden.
2. CoreLocation via `tauri-plugin-geolocation`, resolved asynchronously and cached to disk.
3. IANA timezone → representative coordinates from the bundled `zone1970.tab`
   (~450 zones, each with lat/lon). Offline and deterministic.

- Manual selection searches a bundled GeoNames `cities15000` dataset (~25k cities, CC-BY).
- Rationale: CoreLocation for an ad-hoc-signed app is an unverified risk (R-06). The app must
  be fully correct without it. Step 3 guarantees a sane default with zero permissions and
  zero network.
- Timezone always comes from the resolved location, never from the system clock's zone.

### D-008
**Tray glyphs are macOS template images by default.**

- Template images auto-invert with the menu bar appearance. A fixed-colour icon is invisible
  in one of the two modes.
- Moon: lit fraction opaque, unlit transparent, with a hairline full-disc ring so a new moon
  is still a visible target.
- Grahas: vector paths drawn in Rust, optically balanced at an 18 px cap-height — not font
  glyphs, whose weights and baselines are inconsistent across the set.
- Colour mode is available in settings, off by default, because coloured menu bar icons are
  against platform convention.
- Rendered at 2x into RGBA via `tiny-skia`, applied with `set_icon_with_as_template`.

### D-009
**Moon is a permanent tray item. Each of the nine grahas is independently toggleable and gets
its own tray item and its own panel.**

- Default state: moon only. The menu bar stays clean until the user opts in.
- Each item is an independent click target with an identical panel shell and different content.

### D-010
**v1 day detail shows only: phase name, illumination, moonrise, moonset, nakshatra + pada with
entry/exit, rashi with entry/exit.**

- Explicit instruction: minimal first, polish the UX, add detail later.
- Tithi, yoga, karana and muhurta are **not computed in v1**. They are not stubbed, not
  dead-coded, and not hidden behind a flag. Adding them later is additive work in `panchanga`.
- The domain model is shaped so those additions do not require restructuring.

### D-011
**Opaque near-black surface. No macOS vibrancy.**

- Vibrancy samples the desktop wallpaper, which would drift the palette away from the chosen
  `#0A0A0B` ground and break contrast guarantees.
- Panel is opaque with a 1 px hairline border, 12 px radius, and a soft drop shadow.
- Consequence: `macOSPrivateApi` is not needed.

### D-012
**Private GitHub repo. Unsigned local build, ad-hoc codesigned.**

- Swiss Ephemeris is AGPL-3.0. AGPL obligations attach on distribution; a private repo for
  personal use is compliant. Going public later means the app is AGPL-3.0 too.
- `codesign -s -` (ad-hoc) so first launch is a right-click-Open rather than a hard Gatekeeper
  block. No Apple Developer account, no cost.
- CI: lint, format and the pure-Rust ephemeris test suite run on Linux (cheap). macOS runners
  are used only to build a DMG on a release tag, because they bill at 10x.

### D-013 — OPEN
**Product name.**

- Repo directory is `moon-phases`, but the app covers all nine grahas.
- Proposal: **Chandra** as the display name, repo stays `moon-phases`.
- Needs a decision before the bundle identifier is fixed; changing it later moves the
  settings file path.

### D-014 — OPEN
**Front-end framework.**

- Proposal: **SolidJS + Vite + TypeScript**. ~7 KB runtime, no virtual DOM, fine-grained
  reactivity. Best fit for a small popover that must feel instant.
- Alternatives: Svelte 5 (comparable, larger runtime), React (heaviest, unjustified here).
- Styling is hand-written CSS with design tokens. No CSS framework.

### D-015
**All internal time is Julian Day in UT. Civil time is derived only at the display boundary.**

- `swe_calc_ut` and `swe_rise_trans` take UT. Delta-T is applied by Swiss Ephemeris internally.
- Civil conversion uses `jiff` with the location's IANA timezone, so DST transitions and
  historical offset changes are handled by tzdb rather than by arithmetic.
- No `chrono` + manual offset arithmetic anywhere.

### D-016
**Event times are found by adaptive bracketing then Brent refinement.**

- Applies uniformly to sign ingress, nakshatra ingress, retrograde stations and syzygies.
- Scan step per body class: Moon 1 h; Sun/Budha/Shukra/Mangala 6 h; Guru/Shani/nodes 24 h.
- Refine the bracketed root with Brent's method to 1e-6 day (~0.09 s). No derivatives needed,
  guaranteed convergence on a sign change.
- Longitudes are unwrapped before root-finding so the 360→0 discontinuity is not mistaken for
  a crossing.
- Retrograde stations bracket a sign change in `speed`, using the same machinery.

### D-017
**No background timers, no notifications, no permission prompts in v1.**

- Compute happens only when a panel opens or the displayed day rolls over.
- Idle CPU target: 0%.
