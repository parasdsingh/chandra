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
| [D-010](#d-010) | v1 day detail is minimal; panchanga fields are not computed yet | accepted, amended by D-019, D-025 |
| [D-011](#d-011) | Translucent panel using the system popover material | accepted, revised |
| [D-012](#d-012) | Private repo, unsigned local build, ad-hoc codesign | accepted |
| [D-013](#d-013) | Product name is Chandra; bundle id `com.parasdsingh.chandra` | accepted |
| [D-014](#d-014) | SolidJS + Vite + TypeScript, hand-written CSS | accepted |
| [D-015](#d-015) | Time scale handling: UT vs TT | accepted |
| [D-016](#d-016) | Event times found by bracket + Brent refinement | accepted |
| [D-017](#d-017) | No background work, no notifications in v1 | accepted |
| [D-018](#d-018) | GitHub remote deferred; local VC for now | accepted |
| [D-019](#d-019) | Every state the grid draws is named in the day view; combustion judged at local noon | accepted, amended by D-024, D-025 |
| [D-020](#d-020) | One hue. Retrograde is written, not coloured; combustion never dims | accepted, amended by D-023, D-024 |
| [D-021](#d-021) | Lunar mode names days by tithi; Vikram Samvat years; grid laid out by the back end | accepted, amended by D-024, D-025 |
| [D-022](#d-022) | The menu bar carries retrograde, and nothing else | accepted |
| [D-023](#d-023) | Colour may depict, never encode; `--text-tertiary` carries no text | accepted |
| [D-024](#d-024) | One mark vocabulary for both calendars: no underlines, a combustion wash, a retrograde bracket | accepted |
| [D-025](#d-025) | The day view is one field stack for all nine subjects | accepted |

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
**Translucent panel on the system popover material, with an opaque fallback.**

Revised. The original decision was an opaque `#0A0A0B` surface with no vibrancy, on the
grounds that a sampled backdrop would drift the palette and break the contrast guarantees.
It shipped the other way: a flat fill sits oddly among the menu bar's own popovers, which are
all drawn on the same translucent, blurred backdrop.

- The window is transparent and carries `NSVisualEffectMaterial::Popover`
  (`NSVisualEffectState::Active`), corner radius matched to the panel's own so the material
  does not show square corners behind rounded content. Re-cut whenever the panel is scaled.
- The panel paints a **scrim**, not a fill: `rgba(10, 10, 11, 0.55)`. Raised from 0.28 by
  measurement — on the shipped panel over ordinary content the composited ground runs
  `#1A3033` to `#5C4E36`, and against the light end `--text-secondary` held only 3.15:1. At
  0.55 it clears 4.76:1 and the material still reads as a material.
- No alpha rescues `--text-tertiary`: 0.70 reaches 4.09:1 and would make the panel opaque.
  That is what retires it as a text colour (D-023).
- **The fallback is not cosmetic.** With no material there is nothing for the scrim to darken
  and the desktop shows through at 45%. `apply_vibrancy` reports whether it took, the answer
  travels to the front end on `Bootstrap.panel_material`, and the panel paints `--ground`
  opaque instead. `prefers-reduced-transparency: reduce` gets the same answer.
- No drop shadow: the window is built with `.shadow(false)`. The panel's edge is its 1px
  border and 12px radius.
- The panel is forced to the dark theme regardless of the system appearance. The palette is a
  single dark one, and the material's light variant would put near-white text on a near-white
  backdrop.
- Consequence: `macOSPrivateApi` is not needed. `window-vibrancy` uses public AppKit.

### D-012
**Private GitHub repo. Unsigned local build, ad-hoc codesigned.**

- Swiss Ephemeris is AGPL-3.0. AGPL obligations attach on distribution; a private repo for
  personal use is compliant. Going public later means the app is AGPL-3.0 too.
- `codesign -s -` (ad-hoc) so first launch is a right-click-Open rather than a hard Gatekeeper
  block. No Apple Developer account, no cost.
- CI: lint, format and the pure-Rust ephemeris test suite run on Linux (cheap). macOS runners
  are used only to build a DMG on a release tag, because they bill at 10x.

### D-013
**Display name is Chandra. Bundle identifier `com.parasdsingh.chandra`. Repo stays `moon-phases`.**

- Chandra is himself one of the navagrahas, so the name does not become wrong as the app grows
  past the moon calendar.
- The identifier fixes the settings path at
  `~/Library/Application Support/com.parasdsingh.chandra/`. It is now frozen; changing it later
  would strand existing settings.

### D-014
**SolidJS + Vite + TypeScript. Hand-written CSS with design tokens.**

- ~7 KB runtime, no virtual DOM. Month switching updates only the changed cells rather than
  diffing a tree, which is what keeps the popover feeling instant on open.
- Rejected: Svelte 5 (comparable, larger runtime), React (~45 KB plus reconciliation, not
  justifiable for a 320 px popover with a performance priority).
- No CSS framework. Tokens live in `src/styles/tokens.css` and are the single source of truth
  for the palette in D-011.

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

### D-018
**GitHub remote is deferred. Version control is local until asked.**

- The repo is initialised and committed locally on `main`.
- `gh repo create moon-phases --private` and the first push happen on request, not
  automatically. CI workflows are authored in M0 but only take effect once a remote exists.

### D-019
**A state the grid draws must be readable in words in the day it opens, and both must be
judged at the same instant.**

- Reported: "combust, retro, etc. statuses are not shown in day view."
- The grid marked combustion with a rule under the glyph and retrograde with a rule at the
  cell's edge. Neither appeared in the day view, so a mark had nothing to explain it.
- The day view now carries a `From Sun` row for every subject that has a combustion orb, with
  the orb named in a caption on the days it applies, and a `Motion` row reading `Direct` or
  `Retrograde`.
- This adds one field beyond the D-010 list. The argument is narrow and does not reopen
  D-010 generally: the field exists because the grid already draws the state, not because
  panchanga has more limbs.
- **Combustion is judged at local noon** — the day's midpoint — for every subject, in both the
  cell and the day detail. Divisions are still attributed at sunrise; combustion is not,
  because a mark on a cell and the reading inside it disagreeing is worse than either choice
  of instant. `chandra_almanac::events::combustion_at` is the single place the question is
  asked, and `the_combustion_mark_and_the_day_it_opens_agree` is the test that holds it.
- The Sun and the nodes have no orb. For them the block is absent, not present and
  permanently negative.
- No warning colour: combustion is an ordinary position. `--retro` stays reserved for
  retrograde motion alone.

**Amended by D-024 and D-025.** The principle stands and is what D-024 is built on; two
clauses of the implementation do not:

- There is no `From Sun` row. A `Combust` field appears only on the days the subject is
  combust, reading `inside the N° orb`; the separation in degrees is not shown at all.
- `--retro` was removed by D-020, so nothing is reserved for it. Combustion did acquire a
  colour of its own under D-023, which is not a warning colour.

### D-020
**One chromatic hue in the whole app. Retrograde is written `℞`; combustion is a rule and
never a dim.**

- Reported: "grayed out combust indicator is bad, find another way" and "do not rely on colour
  for retro, use something more apt".
- **Combustion no longer dims the glyph.** Opacity was already spoken for — it is how a cell
  says it belongs to the neighbouring month — so a dimmed glyph inside the month read as
  "not really here" rather than "inside the Sun's rays". The rule under the glyph carries it
  alone, which is what the design language asks for anyway: an instant is a mark, a span is a
  rule, and combustion is a span.
- **Retrograde is written `℞` beside the symbol**, at 9px, placed absolutely so the symbol
  itself stays on the column's centre line whether or not the day is retrograde. This is how
  every printed ephemeris writes it, so it needs no key.
- `--retro` is **removed**. It existed only to carry retrograde. A hue has to be learnt, it is
  the first thing a grayscale or colour-blind rendering loses, and it made the graha symbol
  carry two meanings at once — which graha, and what it is doing.
- The station marker and the retrograde span rule move to `--marker`. The station is still
  distinguished from an ingress by shape, which was always the primary difference.
- `--accent` remains the only hue, and still means today and nothing else.
- Amends DESIGN.md §1.4, §4.1, §6.3, §11.3.

**Amended by D-023 and D-024.** What survives is that retrograde is not carried by a hue and
that combustion never dims. The rest was replaced:

- **Combustion is not a rule.** It is a warm radial wash at the cell's foot, `--glare`
  `#e2603a` (D-023, D-024). The rule under the numeral read as underlined text.
- **The cell does not carry `℞`.** Retrograde is a dotted ring around the glyph, drawn as a
  bracket over the run (D-024). `℞` survives in the day view's `Motion` chip and, per D-022,
  in the menu bar.
- **The station marker and the span rule are gone**, so neither moves anywhere. The grid draws
  no ingress or station marks at all (D-024). `--marker` now carries the retrograde ring and
  the kshaya dot.
- **`--accent` is no longer the only hue.** It is still the only hue that *encodes* anything,
  and it still means today alone; `--glare` depicts (D-023).

### D-021
**In a lunar month the cell is named by its tithi, the year is Vikram Samvat, and the 42 cells
are laid out by the back end.**

Reported: "in lunar calendar mode, dates are still gregorian, i expect the traditional days and
dates". Designed in [docs/design/lunar-dates.md](design/lunar-dates.md), which carries the
research, the wireframes and the rejected alternatives.

- **The tithi is the label**, printed as panchangs print it: `S1`-`S14`, `P` for Purnima,
  `K1`-`K14`, `A` for Amavasya. The Gregorian day becomes the annotation on the second line,
  carrying its month only where the month changes (`1 SEP`).
- The cell string is derived from **paksha plus number within the paksha**, never from the
  astronomical 1-30 index. Amanta and purnimanta months count from opposite ends of that index,
  so deriving from it is right in one system and wrong in the other.
- **The Moon's phase glyph gives way in lunar mode**, and with it the Moon's combustion rule. A
  tithi *is* elongation divided by twelve, stated more precisely than a 14px disc can; the Moon
  is combust exactly around Amavasya, which the numeral already names. Both were duplicated ink
  (DESIGN 1.3).
- **The graha's symbol gives way too.** The same symbol on 42 cells identifies nothing the
  header does not already say, and in lunar mode the second line is spoken for. Every state it
  used to anchor - retrograde, combustion, ingress, station - is drawn on the cell and survives
  the swap.
- **Kshaya and vriddhi are drawn, never silent.** A tithi that holds no sunrise is skipped and
  the numbers jump; one that holds two is repeated across two days. A jump carries a dot in the
  cell's top left, a repeat carries a rule joining the pair along their bottom edge. An unmarked
  jump would be a bug, so the mark is what makes it legible as a calendar rather than a fault.
- **The year is Vikram Samvat**, chosen by the user over Gregorian and Shaka. The era begins at
  Chaitra, so it runs 57 ahead of the Gregorian year for most of its length and 56 ahead after
  1 January. Neither the Gregorian year alone nor the month name alone decides it: Pausha starts
  in December in some years and in January in others.
- **The grid is laid out by the back end**, which now returns exactly 42 cells with an
  `in_month` flag. The front end cannot compute which civil days a lunar month contains - it
  runs between syzygies, not between dates - and the version that guessed the neighbouring dates
  drew cells it had no data for. The locale's opening weekday travels the other way, in the
  request, because that is the one fact the back end cannot derive.
- **Solar mode is unchanged**, and pays nothing: no tithi is computed for it at all. The user
  chose to keep the Gregorian day view at the D-010 fields.
- **The day view gains a Tithi block and a Sunrise row**, and the vara name on the date line.
  Sunrise is there because it is the instant the tithi, the nakshatra and the rashi are all read
  at: without it the number in the grid cannot be checked against anything. Yoga and karana stay
  out - neither explains a number in the grid, and a karana is half a tithi and derivable from
  the row above it.
- Week start follows the system locale in both modes, chosen by the user over forcing Sunday.

**Amended by D-024 and D-025.** The naming, the derivation, the era and the back-end layout all
stand. What the cell and the day view do with them changed:

- **Nothing gives way.** The subject's glyph — the Moon's phase disc, a graha's symbol — is on
  the cell's second line in both calendars, and the other calendar's date sits in the top-right
  corner (D-024). The argument for dropping it was that the second line was spoken for; moving
  the date freed it.
- **The kshaya dot is set on the numeral**, not in the top-left corner: the corners carry the
  other calendar's date.
- **The vriddhi rule is gone.** A repeated tithi is visible in the grid as the same numeral on
  two days, and is named in the spoken label (`vriddhi, the same tithi names the day after`).
  Nothing is drawn for it.
- **There is no Sunrise row** (D-025). Sunrise is Surya's rise, on Surya's day.
- The header label names the era: `Shravana VS 2083`, not `Shravana 2083` (AUDIT W-05).

### D-022
**The menu bar carries retrograde, and nothing else.**

- Chosen by the user from: retrograde only, retrograde and combustion, or nothing.
- Drawn as `℞` in the lower right corner, with the glyph shrunk to 16.5pt to free it. The same
  notation the calendar cell uses, so one mark means one thing across both surfaces.
- A template image varies only in alpha, so colour was never available there. That is what
  settles it: a second state would have to be another shape in another corner of a 22 point
  square, and the row of tray items would stop being scannable.
- The mark is drawn rather than set in type: this crate rasterises without a font, and there is
  no text shaping and no system font to ask.

### D-023
**Colour may depict, may not encode. `--text-tertiary` carries no text.**

Amends D-020's "one hue" to a rule about what a hue is allowed to do.

- **Depicting is allowed; encoding is not.** A body inside the Sun's rays is drawn as the glare
  it is lost in, and glare is warm because glare is warm. A colour that *means* combust would
  be a code, and a code has to be learnt.
- `--glare` `#e2603a`. Orange rather than amber so it cannot be mistaken for `--accent`, which
  sits about 25° away in hue and can appear in the same 40px cell. It carries no meaning on its
  own — every combust day says so in words in the day view — so it is free to sit below the
  contrast floor a text colour must clear.
- `--accent` still means **today** and nothing else.
- **`--text-tertiary` carries no text.** Measured over the composited translucent ground
  (D-011) it reaches 2.21:1 at the light end, failing both the 4.5:1 small-text floor and the
  3:1 non-text bar. Every label that used it now uses `--text-secondary`. It is kept for the
  one thing with no text floor to clear: the border on the `℞` chip, and the switch knob.

### D-024
**One mark vocabulary, the same in both calendars. No underlines.**

Reported: the rule under a numeral read as underlined text, and marks that appeared in solar
mode and not in lunar mode could not be learnt.

- **A state mark does not vary between the solar and lunar calendars.** A state drawn in one
  and not the other is a state nobody can learn, so the marks are drawn on the cell rather than
  on the glyph and survive the swap of what labels it.
- **No underlines anywhere.** The rule under the numeral is gone and nothing replaced it in
  that position.
- **Combustion is a warm radial wash at the cell's foot**, `--glare` (D-023), behind the
  numeral and the glyph. A field rather than a mark: it costs no room in a cell that has none,
  and a run of combust days reads as one warm stretch instead of as five separate marks.
- **Retrograde is a dotted ring around the glyph, drawn as a bracket.** The half facing the
  retrograde days on the day the motion turns, a whole circle in between, the opposite half on
  the day it turns back. A run therefore reads as one shape spanning several cells. Centred on
  the glyph, not on the cell: the cell's middle falls between the numeral and the symbol.
- **Ingress and station markers are no longer drawn at all.** They are named in the day view's
  `Events` field, with their times. The marker row they occupied is gone.
- **The second line carries the subject's glyph in both calendars**, and the other calendar's
  date moved to the top-right corner — the corner the ingress marker used to hold.
- Amends D-019, D-020 and D-021. Amends DESIGN.md §5.5, §6.1–§6.4, §11.3.

### D-025
**The day view is one field stack, the same shape for all nine subjects.**

Designed as variant 02 of the mocks; the reasoning for the shape itself is not recorded beyond
that.

- **A field is a small uppercase label, the value and the hour it gives way on one line, and a
  `then …` successor line.** A day holds at most two of each span, so naming the successor
  costs one line and saves opening tomorrow.
- **Removed: illuminated percentage, distance from the Sun, speed, longitude.** They were the
  only reason the Moon's view and a graha's had different shapes.
- **Rise and set belong to the subject.** Surya's day names sunrise and sunset, Chandra's
  moonrise and moonset, and every other graha's names itself — `Shani rise`. There is no
  separate sunrise row on every subject's day.
- **Every drawn state is named in words on the same surface** — this is D-019's principle
  unchanged — and spoken labels say what was measured: `combust at noon`, not `combust`.
- The phase name appears in solar mode only; in a lunar month the tithi says the same thing
  more precisely.
- Amends D-010's field list and D-021's Sunrise row. Amends DESIGN.md §5.6, §6.4.
