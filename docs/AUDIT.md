# Audit, August 2026

Two independent audits were run over the same tree with byte-identical
instructions, so that either could catch what the other missed. They agreed on
ten findings, each found several the other did not, and they contradicted each
other twice. Both contradictions were settled by hand and are recorded below.

Source column: **A**, **B**, or **A+B** where both found it independently.
Status is one of `open`, `fixed`, `wontfix` (with a reason), `verified`.

---

## Method note worth keeping

Two tests passed while the thing they guard was broken. Both are the same
mistake — a test that reads as a guarantee and is not one.

- `the_cell_and_the_day_it_opens_name_the_same_tithi` walks a single month
  (August 2026, Asia/Kolkata) which happens to contain no failing day. Widened
  to a year it fails on 18 days in 1063.
- The retrograde-mark test measures ink inside a corner box and total coverage.
  Neither can detect two shapes touching, which is exactly the defect.

**When fixing anything below, widen the test that should have caught it.** A fix
whose test would still pass on the old code is not finished.

---

## Correctness — high

### F-01 `prevailing` is decided on the scan grid, not the refined boundary — A+B
`crates/almanac/src/tithi.rs:317`, `crates/almanac/src/spans.rs:142`

Both compute `prevailing` by comparing `reference_jd` against `samples[..].jd`,
the scan grid, rather than against the `entry`/`exit` the same loop has already
refined with Brent to 1e-6 d. The scan step is 1 h for the Moon and
`graha.scan_step_days()` for the rest — 6 h for Sun/Budha/Shukra/Mangala, **24 h
for Guru/Shani/Rahu/Ketu**. Any boundary crossed inside that window before
sunrise is attributed to the wrong side.

Measured: 18 of 1063 days over 36 amanta months name a different tithi in the
grid cell than in the day view; 12/730 nakshatra, 5/730 rashi for the Moon over
two years. Guru, 17 Apr 2024, Bengaluru: `GrahaDay.longitude` says Krittika
while the span beside it says Bharani.

Also corrupts pada: `day.rs:266` gives the real pada only to the span marked
prevailing, so a misattributed span carries the *other* nakshatra's pada.

Fix: `entry <= reference < exit`, falling back to the sample comparison only when
a boundary did not resolve. Keep the "exactly one prevailing" backstop.
Widen `the_cell_and_the_day_it_opens_name_the_same_tithi` to a year, and add the
graha equivalent.

**fixed** — `prevailing` is now `entry <= reference < exit` in both routines,
falling back to the sample comparison only for a tithi boundary that did not
resolve. `the_cell_and_the_day_it_opens_name_the_same_tithi` walks thirteen
months instead of one, and `a_graha_day_and_its_prevailing_span_name_the_same_division`
holds a year of Guru, Shani, Rahu and Ketu days to their own longitude and pada.

### F-02 `Engine::reconfigure` drops the lock mid-change — B
`crates/ephemeris/src/engine.rs:176-186`

`reconfigure` sets `inner.config`, drops the guard, then `apply_config` re-locks
and calls `swe_set_sid_mode`. In the gap `inner.config` holds the new node type
while Swiss Ephemeris' global still holds the old ayanamsa. A concurrent
`position()` returns a longitude from a mix. Measured while flipping
`(Lahiri,True)` ↔ `(Suryasiddhanta,Mean)` with four readers: 2.98 M of 6.6 M
results were in states that were never selected. Worst-case error is the full
ayanamsa difference, over a degree.

Reachable: every command runs on `spawn_blocking`, so `update_settings` genuinely
overlaps `moon_month` / `snapshot` / `day_detail`.

Contradicts D-005 directly, which claims the config change and the invalidation
share one critical section.

Fix: one critical section. `apply_config` must not re-acquire.

**fixed** — `reconfigure` holds one guard across both writes; `apply_config` is
gone and `set_sid_mode` is a free function the caller must already be holding the
lock for. `a_configuration_change_is_never_half_applied` reads Rahu under four
flipping threads and refuses any pairing that was never selected.

### F-03 A result computed before an invalidation is stored after it — A+B
`crates/almanac/src/almanac.rs` — `set_sidereal`, `moon_month`, `graha_month`, `store`

`set_sidereal` reconfigures then bumps the generation. A reader mid-computation
then calls `store()`, which stamps its value with the *current* generation — so a
month computed under ayanamsa A is inserted as if computed under B, and never
expires. Measured: after 40 concurrent flips, 42/42 cells of a cached Shani month
were stale, off by 1.45° — enough to cross a rashi boundary.

`CacheKey` carries neither ayanamsa, node type nor location; correctness rests
entirely on the generation counter, and nothing serialises a reader against a
reconfigure.

Fix: capture the generation before computing and refuse the store if it moved, or
put the configuration in the key. Also `moon_month` reads settings three separate
times; take one snapshot.

**fixed** — `Lru::insert` now takes the generation the value was computed under
and discards it if the cache has moved on; `moon_month` and `graha_month` capture
that generation and one settings snapshot before any work starts.
`a_month_computed_under_one_ayanamsa_is_never_served_under_another` changes the
ayanamsa under a running reader and then checks every cell of eight cached months
against a cold recomputation.

### F-04 A failed settings save leaves the engine and the file diverged — B
`src-tauri/src/state.rs:81-103`

`apply` mutates the engine (`set_sidereal`, `set_location`) *before*
`next.save()`, the first thing that can fail. On failure it returns `Err`, the UI
reports the change was refused, and every later computation uses the new
configuration while the file and the Astrology pane report the old one.
`*self.settings.write() = next` is never reached.

`apply` also holds no lock spanning itself, so `update_settings` and
`accept_device_location` can interleave and one settings document can overwrite
the other.

Fix: validate and persist first, mutate after; or roll back on failure. Serialise
`apply`.

**fixed** — `apply` validates the zone, saves, then mutates, and is serialised
by a mutex so the settings pane and a CoreLocation answer cannot interleave.
`a_refused_save_leaves_the_engine_on_the_settings_that_are_on_disk` obstructs the
configuration directory and asserts the engine kept the ayanamsa that is on disk.

### F-05 `noonAnchor` builds UTC noon, not local noon — A+B
`src/lib/calendar.ts:48`, consumed at `src/components/Panel.tsx:65,273,300,386`

The doc comment says "local noon on a date"; the body is
`Date.UTC(y, m-1, d, 12)`, which takes no zone. The back end resolves that
instant to a civil date **in the observer's zone**. At UTC+12 and beyond, 12:00 Z
is already the next day locally, so on the last day of a month the panel opens on
the wrong month with today shown only as a dimmed leading cell. Affects
Auckland, Fiji, Kamchatka, Samoa, Tonga, Kiribati, Chatham. In lunar mode the
anchor can land on the wrong side of a syzygy.

Fix: build noon in the observer's zone. `todayIn` is already zone-aware; this
throws the zone away.

**fixed** — `noonAnchor` takes the observer's zone and resolves noon in it,
measuring the offset twice so a daylight saving change inside the gap cannot
throw it. No test: the front end has no test harness at all, and adding one is a
dependency decision for the owner rather than a fix. Verified by hand against
eleven zones from Kiritimati to Honolulu, where the previous form landed on the
following civil day for all six at UTC+12 and beyond.

### F-06 `rise_set` clamps to a fixed 1.0 day — A
`crates/ephemeris/src/engine.rs:277`

`within_day` filters on `jd_ut_start + 1.0`. `CivilDay` exists precisely because
a day is not always 1.0 long, and `sunrise_of` / `sunrise_anchor` both filter on
`day.end_jd`. On a 25-hour fall-back day a real moonrise inside the day is
discarded and appears nowhere; on a 23-hour spring-forward day one event is
reported on both days.

Reproduces: America/New_York, 1 Nov 2026 and 8 Mar 2026.

Fix: pass the day's true end, or filter at the caller against `day.end_jd`.
`moonrise_and_moonset_stay_inside_the_day_they_are_reported_for` runs only in
Asia/Kolkata, which has no DST — add a DST zone.

**fixed** — `rise_set` takes the window's end rather than assuming 1.0, so a
`CivilDay` passes its own length. `rise_and_set_are_searched_over_the_window_the_caller_asked_for`
proves a wider window never loses an event and that both the twenty-fifth and the
twenty-fourth hour matter over two months; `a_daylight_saving_day_reports_every_moonrise_exactly_once`
walks March and November 2026 in America/New_York and holds consecutive moonrises
to one interval apart.

---

## Correctness — medium

### F-07 Two routes to "which month is this day in" disagree — B
`crates/almanac/src/lunar.rs` — `month_containing` vs `build`

`month_containing` picks the month by the raw syzygy instant; `build` defines the
month's civil extent by the first-sunrise-after-syzygy rule. They differ by up to
a day. Over 2026 at Bengaluru: amanta 6/365 and purnimanta 7/365 days open a grid
in which today is a dimmed out-of-month cell. On **31 May 2026 amanta** today is
not among the 42 cells at all — no today ring anywhere, and `jumpToToday`
re-anchors to the same month.

Fix: select the month by the same rule that defines its extent.

**fixed** — `month_containing` now checks the civil date against the month it
built and steps one month where the syzygy rule and the sunrise rule disagree.
`every_day_opens_the_lunar_month_that_draws_it` walks every day of 2026 in both
systems and requires today to be among the 42 cells and inside the month.

### F-08 `Home` / `End` select days outside the month — B
`src/components/Panel.tsx:365`

`selectEdge` indexes `days.at(0)` / `days.at(-1)` on the 42-cell grid. Viewing
August 2026, `Home` selects 27 July and `End` selects 6 September, then `Enter`
opens a day detail while the header still reads August. DESIGN specifies "first /
last day of the displayed month".

**fixed** — `selectEdge` takes `"first"`/`"last"` and filters to `in_month`,
so Home and End select the first and last day of the displayed month as DESIGN
specifies. No front-end test harness exists to hold it.

### F-09 A lead or trail day reports no transit events — B
`src/components/Panel.tsx:239`, `crates/almanac/src/month.rs:360`

Events are deliberately built from in-month days only. The front end filters that
list by the selected date without noticing the cell is out of month, so clicking
the trailing `3` in August shows no ingress while September shows it. Same date,
two answers.

**fixed** — `selectedEvents` finds the month that actually owns the selected
date among the three in hand and reads that month's events, so the same date
reports the same ingress whichever month's grid it is clicked in.

### F-10 The `℞` mark fuses with the Guru and Rahu glyphs — B
`crates/glyph/src/render.rs:129-190`

The mark's inked box lands at x 13.81–19.40 pt, y 12.56–20.85 pt. Rahu's right
tail and Guru's baseline bar both intersect its top bar. Connected-component
analysis: every other graha yields glyph plus a separate mark; Guru and Rahu
yield one blob. Verified by rendering — Guru's baseline and the mark's top bar
form one continuous rule.

Severity is raised by the nodes being retrograde most of the time, so the broken
icon is the one drawn most often.

Fix: place the mark clear of the glyph's ink, or shrink the glyph further, or
inset the mark. Replace the corner-ink test with a connectivity test.

**fixed** — the mark is placed by its inked box rather than its nominal one, so
it sits in the corner it was meant to and clears every glyph by at least 0.77pt.
`the_retrograde_mark_is_a_separate_shape_beside_the_glyph` replaces the
corner-ink measurement with connected-component analysis at zero threshold: the
marked icon must hold exactly one region more than the plain one. Against the old
placement it fails on the first graha it reaches.

### F-11 The panel is placed against the wrong display — B
`src-tauri/src/panel.rs:157-183`

`place` uses `window.current_monitor()` — the monitor the panel is on, not the
one the clicked tray item is on. The tray rect is physical, produced with the
status item's own backing scale, and is divided by the *panel* monitor's scale,
then clamped into that monitor's bounds. On a Retina laptop plus a 1x external
display, clicking the tray item on the external display opens the panel on the
built-in one and it never self-corrects.

`current_monitor()` returning `None` makes `place` return early while `toggle`
shows the window anyway, so a positioning failure is indistinguishable from
success. `set_position` is `let _`-discarded.

Fix: `monitor_from_point` seeded from the tray rect; do not show on a placement
failure.

**fixed** — `place` finds the monitor from the tray rectangle with
`monitor_from_point`, uses that monitor's scale for every conversion, and returns
a `Result`; `toggle` places before it shows and shows nothing on failure. Not
testable here: the answer comes from the window server.

### F-12 The grid loses its tab stop, or grows two — B
`src/components/MonthGrid.tsx:63,88`, `src/components/DayCell.tsx:131`

`focusedDate() = selected ?? today`, and only the matching cell is `tabindex=0`.
`open()` sets `selected = null`, so after scrolling two months no cell in any of
the three rendered grids matches today: all 126 are `-1`, tab leaves the
document, and the grid is unreachable by keyboard or VoiceOver until an arrow
key sets a selection. DESIGN specifies the missing third case — "if today is
outside the displayed month, it is the first day of the displayed month".

Converse: when today falls in the first or last week it also appears in the
adjacent month's simultaneously-mounted grid, so two elements carry
`tabindex=0` and `aria-current="date"`.

**fixed** — `focusedDate` has the missing third case, restricted to days inside
the month, and `MonthCells` takes an `active` flag so only the month filling the
window carries the tab stop. The two off-screen grids are `aria-hidden`, which
also closes F-48.

### F-13 `rise_set` and `ayanamsa` carry no provenance — B
`crates/ephemeris/src/engine.rs:99-111`, `:294-306`

ARCHITECTURE says flags are compared on every call. Two of four Swiss Ephemeris
entry points do not. At 1700-06-15 `illumination` reports Moshier correctly while
`ayanamsa` and `rise_set` carry no marker. Month scrolling is continuous, so a
user can navigate outside 1800–2399 and get a moonrise with no precision note —
the exact failure D-006 exists to prevent.

`swe_rise_trans` returns a status code rather than a flag word, so `RiseSet` as
typed cannot carry one; `ayanamsa` throws its flag word away.

**fixed** — `RiseSet` carries a `source`, read from a position call for the same
body at the same instant, and `ayanamsa` returns a `Reading` that carries one
too. `bundled_data_serves_the_navigable_range_at_full_precision` now asks all
four entry points at each of the five sample years instead of only `position`.

### F-14 The scroller's gesture state is not reset on open — A (B: speculative)
`src/components/CalendarScroller.tsx:109`, `src/components/Panel.tsx:381`

Introduced by the change that stopped reloading the page. `open()` resets nine
Panel signals but cannot reach the scroller's `shift`, `visible`, `odometer`,
`samples`, pending `wheelTimer` or pending rAF handle, and the scroller only
unmounts when the view leaves `calendar`. Closing mid-gesture leaves the strip
translated; a resumed settle then fires `onCommit(±1)` *after* `open()` set the
offset to 0.

B could not establish WebKit's rAF behaviour for an occluded window and rated it
speculative. The absence of any reset path is confirmed either way.

Fix: give the scroller an imperative reset, or remount it on open.

**fixed** — the scroller is keyed on an open counter, so every open builds a
fresh one and its offset, wheel timer and pending frame go with the old one.
Remounting cannot miss a field the way an imperative reset can.

### F-15 Re-anchoring at six months blanks the strip — A+B
`src/components/Panel.tsx:194`

The effect sets `anchor` and zeroes `offset`; `monthKey` includes both, so all
three keys become strings never inserted, `monthAt(-1|0|1)` all return undefined,
nothing renders and the title empties until three IPC round trips land. The
comment claims "the cache still hits and nothing on screen changes" — the Rust
cache hits, the front-end Map does not. This is I-042 §2 recurring.

Fix: rekey the Map entries under the new anchor when re-anchoring, or key on the
identity of the month returned rather than on the request parameters.

**fixed** — a month's address is split into what it is an answer to and where
it sits, and re-anchoring re-addresses the entries it already holds instead of
orphaning them. Only the current context and the anchor being left are moved, so
a month remembered under an older configuration is never given a current address.

### F-16 Errors are captured into a signal almost nothing renders — A+B
`src/components/Panel.tsx:165,307,457`

`error()` is passed only into `<DayDetail>`, rendered only when
`view() === "day"`. A month-load failure in calendar view shows nothing at all. A
settings-save failure fires while the user is by definition in the settings view,
so `ERROR_TEXT.SETTINGS` is unreachable. Opening any day clears the signal before
it could have been read.

**fixed** — the calendar and the settings views render the error too. A month
that fails to load replaces the strip, which has nothing left to show; a settings
failure appears above the list rather than instead of it, because the controls
are what the user needs to try something else. A month or a settings save that
succeeds clears the signal, so `ERROR_TEXT.SETTINGS` is now reachable and now
goes away.

### F-17 `principal_at` is a linear interpolation — A+B
`crates/almanac/src/day.rs:189`

`principal_phase_in` returns an elongation fraction across a whole civil day and
`moon_day` converts it to an instant by linear interpolation. Against
`lunar::next_syzygy` (bracket + Brent) the error reaches **3.63 minutes** — the
same instant computed twice by two routes that disagree at the app's own display
resolution. D-016 says the bracket+Brent machinery applies to syzygies.

`MoonDay.principal_at` is also never rendered.

Fix: use the refined syzygy, or delete the field. Do not keep both routes.

**fixed** — the field is deleted, and with it the interpolation. `phase` already
names a principal phase only on the day it occurs, so nothing was lost;
`principal_phase_in` now returns the name alone, which removes the second route
to a syzygy instant rather than leaving two that disagree by 3.6 minutes.

### F-18 Elevation is discarded in automatic mode — A+B
`src/components/SettingsView.tsx:257-275`, `src-tauri/src/location.rs:56-58`

The elevation field writes a `place` while preserving `mode: automatic`;
`location.rs` uses `place` only in manual mode. The value appears to stick
because Solid only writes `input.value` when the tracked value changes, and every
rise/set computation keeps using the chain-resolved elevation. A also notes the
converse: the written place makes `resolve_offline` report
`Provenance::CoreLocation`, so a timezone-derived location is relabelled "from
this Mac", and the button that would clear it is hidden in automatic mode.

**fixed, and the headline was wrong.** `resolve_offline` consults `place` in both
modes - the manual branch is checked first, then an unconditional one - so the
typed elevation did reach every rise and set, and `a_cached_automatic_place_is_reused`
already asserted as much. The converse A noted is the real defect and it is the
one fixed: elevation is now `location.elevation`, applied on top of whichever
step of the chain answered, so setting it no longer fabricates a place that then
reports itself as having come from the device. That is a schema change, so
`SCHEMA_VERSION` is 2 and `migrate` has its first real step;
`a_version_one_document_migrates_without_changing_what_it_meant` holds a version
1 file to the observer it used to resolve to. `Use my timezone` is offered
whenever a stored place is standing in for the chain, not only in manual mode.

### F-19 `today` and `anchor` do not follow a location change — A+B
`src/components/Panel.tsx:61,65`

Both are computed from the zone at construction and refreshed only in `open()`.
Changing location across the date line updates every computed value but leaves
the today ring on the old zone's date until the panel is closed and reopened.

**fixed** — `today` and the anchor follow the observer's zone through an
effect deferred past construction, so a location change across the date line
moves the today ring without waiting for the panel to be closed and reopened.

### F-20 An ayanamsa change does not refresh the tray tooltips — B
`src-tauri/src/state.rs:101`

`icons_changed: tray_changed || location_changed` omits sidereal, so
`tray::refresh_icons` is skipped. Tooltips read "{name} — {rashi}", and rashi is
exactly what the ayanamsa moves. Stale for up to 24 hours.

**fixed** — `icons_changed` now includes a sidereal change, computed once beside
the other two in the reordered `apply`.

### F-21 "Use this Mac" is a silent no-op while a manual location is set — B
`src-tauri/src/state.rs:117`, `src/components/SettingsView.tsx:281`

The refusal is correct per D-007, but `request_device_location` returns
`Ok(location())` — the unchanged manual location — and the button is rendered
unconditionally. The user grants permission, CoreLocation answers, the answer is
dropped, and nothing is said.

**fixed** — the button is not offered while a manual location is in force,
which is the state in which it can do nothing (D-007). `Use my timezone` is now
offered whenever a stored place is standing in for the chain, so a cached device
fix can be cleared as well.

### F-22 A warm lunar month costs 100–300× a warm solar one — B
`crates/almanac/src/almanac.rs:252,281`

`resolve(cursor)` runs before the cache lookup, because the key is derived from
the resolved first day. So a warm lunar month still pays the syzygy walk plus 42
`CivilDay` constructions. Measured in release: solar warm 13–22 µs; amanta warm
2.16 ms at offset 0, 5.38 ms at offset 5 — over the documented 5 ms budget, per
subject, per open.

`a_lunar_month_stays_inside_the_budget_and_is_computed_once` never measures a
warm lunar month, unlike its solar counterpart.

**fixed** — the resolution is cached under the cursor, which is the step being
avoided: the old key was derived from the resolution's own answer, so a warm
lunar month walked its syzygies and built 42 civil days before the cache was
consulted. `a_lunar_month_stays_inside_the_budget_and_is_computed_once` now
measures a warm month at offset 0 and at offset 5; the second took 5.74 ms before
the fix, over the documented budget.

### F-23 `count_sunrises` conflates "zero" with "unknown" — A
`crates/almanac/src/tithi.rs:360`, `src/components/DayDetail.tsx:352`

It returns 0 when a boundary did not resolve — its own doc says the day view
should print the boundary as unavailable rather than infer a state — and the day
view infers exactly that, captioning it "kshaya, no sunrise". Separately, at a
latitude with no sunrise on any of three days the slice is empty, so every tithi
that day is captioned a kshaya while the cell, which falls back to local noon, is
right.

### F-56 The panel has no fallback when the popover material fails to apply — coordinator
`src-tauri/src/panel.rs:73-89` (`apply_material`), `src/styles/panel.css` (`.panel` background)

`apply_material` logs and continues when `apply_vibrancy` returns an error. Its
comment claims the panel "falls back to its own scrim, which is legible". That
fallback does not exist: the window is built `transparent(true)` and `.panel`
paints only `rgba(10, 10, 11, 0.28)`, so with no material behind it the desktop
shows through at 72% and every text-contrast guarantee in the app is void.

Measured over ordinary colourful content, the composited ground ranges from
`#1A3033` to `#5C4E36`. At the bright end `--text-primary` still holds 6.91:1 but
`--text-tertiary` falls to **2.21:1**, under both the 4.5:1 small-text threshold
and 3:1.

Fix: `apply_material` reports whether it applied, the panel records it, the front
end is told, and `.panel` paints an opaque `--ground` where the material is
absent. Same treatment under `prefers-reduced-transparency: reduce`.

Out of scope, deliberately: `--text-tertiary` at 2.21:1 over an arbitrary
backdrop is a token-level decision for the owner, and no achievable scrim alpha
fixes it. Every colour token is left exactly as it is.

**fixed** — `apply_material` returns a bool, `panel::PanelMaterial` records it,
`bootstrap` carries `panel_material`, and `.panel.is-opaque` plus a
`prefers-reduced-transparency` block paint `--ground`. Non-macOS builds report
`false`, which is the truthful answer there. No test: this is a window-server
outcome no headless suite can produce, and the false branch is what the preview
harness has always rendered under.

**fixed** — `TithiSpan.sunrises` is `Option<u8>`: `None` for an unresolved
boundary and for a latitude with no sunrise on any of the three days, `Some(0)`
only for a real kshaya. The day view captions only the count it is given.
`a_polar_night_reports_no_sunrise_count_rather_than_none_at_all` holds both
halves - Longyearbyen in January reports no count, and a year at Bengaluru
reports a count on every span and a genuine zero among them.

---

## Gaps

### F-24 `launch_at_login` is a dead setting — A+B
Defined, persisted, in the IPC contract, plugin registered, never called. No UI.
`capabilities/default.json` grants only `core:default`, so a front-end call would
be denied anyway. ARCHITECTURE §10 claims the feature exists.
Either wire it end to end or remove it and correct the doc.

**removed**, with the doc corrected. Nothing ever called the plugin and no UI
could set the flag, so wiring it would have been adding a feature under cover of
an audit. The field, the `tauri-plugin-autostart` dependency and its registration
are gone; ARCHITECTURE §7 and §10 say so.

### F-25 `time_format` has no UI — A+B
Read at `Panel.tsx:464`, never settable. Every time is formatted at the locale
default forever. Wire it or remove it.

**removed.** The same judgement as F-24: it was read but never settable, so
every time was already formatted at the locale default. Adding a three-way
picker to the Calendar pane would be new design, not a fix. `TimeFormat`,
`hourCycle` and `FormatContext.timeFormat` go with it; `formatTime` says in one
line that the locale decides.

### F-26 City search has no debounce, no ordering guard, no catch — B
`src/components/SettingsView.tsx:180`
One `invoke` per keystroke; a rejection becomes an unhandled promise rejection
and the pane then reports "No city matches …", a data statement for a failure.

**fixed** — one search per 180 ms pause, a per-effect guard so only the newest
answer is taken, and a `catch` that says "City search is unavailable." rather
than reporting a failure to ask as a fact about the data.

### F-27 `IN_FLIGHT` retains the CLLocationManager after the timeout — B
`src-tauri/src/location.rs:237`

**fixed** — `location::release` clears the slot, called on the main thread as
soon as the caller stops waiting, so an unanswered authorisation prompt no longer
holds a `CLLocationManager` for the life of the process.

### F-28 `update_settings` does filesystem IO on the async runtime — B
`src-tauri/src/commands.rs:181`. The module doc claims every command moves its
work to a blocking thread; `state.apply` takes the engine mutex and does
`fs::write` + `fs::rename` inline.

**fixed** — `state.apply` runs through `blocking`, like every other command.

### F-29 Shift+PageUp / PageDown silently step one month — B
`src/components/Panel.tsx:331` filters meta/ctrl/alt but not shift. DESIGN
specifies ±1 year.

**fixed** — `event.shiftKey` selects ±12 months, which is the year DESIGN 10.1
specifies.

---

## Quality, dead code, doc drift

Each is small. None is optional — leaving a comment that lies is how the next
defect gets written.

- **F-30** Dead code: `month::days_between`, `time::first_weekday_offset`,
  `commands::ayanamsa_degrees` + its IPC wrapper, `Observer::is_polar`,
  `Almanac::sankrantis_between` (tests only).
  **fixed** — all removed except `sankrantis_between`, which is kept and marked
  `#[doc(hidden)]`: it is what lets the intercalary test state the rule in its
  own terms instead of re-running the comparison the detection uses, which would
  assert only that the code agrees with itself. Same treatment `Settings::sample`
  already had.
- **F-31** `MoonDay.source` chains `Source::Swieph` per nakshatra, a constant, so
  the aggregation is a no-op that reads as if it accounts for span provenance.
  `spans.rs:150` computes a real per-span source that `NakshatraSpan` /
  `RashiSpan` then drop. `Source::weakest` over an empty iterator returns
  `Swieph` — full precision claimed from no evidence.
- **F-32** `MoonCell.principal` is documented as "the grid marks these"; nothing
  draws it. Draw it or drop the field.
  **dropped.** `phase` already carries it exactly: `intermediate_phase` never
  returns a principal name, so a principal name appears only on the day that
  phase occurs. Drawing it was not available - the cell is being redesigned in
  parallel - and a redundant boolean with a comment that lies is the worse of
  the two things to leave.
- **F-33** Two routes to combustion in one layer: `moon_month` calls
  `combustion_at`, `graha_month` calls `combustion_from`. They agree today.
  **fixed** — `combustion_from` takes the whole `Position` rather than a
  longitude and a separate retrograde flag, so a caller cannot pair a longitude
  from one instant with a direction of travel from another; `combustion_at` is
  that with the fetch in front of it, and `graha_month` now takes both bodies in
  one call at the instant D-019 names.
- **F-34** `GrahaCell.rashi` / `.nakshatra` are read at local noon while the day
  view reads them at sunrise, which D-019 specifies. Never consumed by the front
  end, and computed for all 42 cells regardless.
  **fixed** — both removed. Nothing drew them and they could name a different
  nakshatra from the day they opened.
- **F-35** Payload fields never read: `time_zone`, `era_year`, `adhika`,
  `kshaya_masa_name`, `system`. `Header.tsx:130` re-parses "Adhika " back out of
  `label` instead of using the field that carries it.
  **fixed** — `adhika` is now read: the header takes it and splits the qualifier
  only for a month that has one, rather than for any month whose name contains
  the word. `time_zone`, `era_year`, `kshaya_masa_name` and `system` are removed;
  the label already carries the era year and the kshaya pair, and the front end
  formats every timestamp against the observer's zone from the bootstrap.
- **F-36** `formatDegrees` pads degrees to three characters for a value that is
  0–29, so every longitude prints a leading zero. The docstring's example cannot
  occur.
  **fixed** — padded to two, with an example that can occur.
- **F-37** `formatSeparation` / `spokenSeparation` duplicate the same
  degrees-and-carry arithmetic.
  **fixed** — one `degreesAndMinutes` applies the carry; the two formatters
  print it. The spoken form also stops saying "0 minutes".
- **F-38** `Panel.step()` and the scroller's `onCommit` handler are the same
  one-line mutation written twice.
  **fixed** — `onCommit={step}`.
- **F-39** `if (!key) return undefined` in the detail resource is unreachable.
  **fixed** — removed. A resource with a falsy source never calls its fetcher.
- **F-40** Three near-copies of "sunrise, or noon": `lunar.rs:227` takes it
  unfiltered while `month.rs:203` and `day.rs:120` both filter to the day.
  **fixed** — `day::sunrise_of` and `day::reference_instant` are the only copies;
  `month.rs` and `lunar.rs` call them, so `first_civil_day` no longer admits the
  next day's sunrise on a day the Sun does not rise.
- **F-41** A `CivilDay` for year 1 CE is constructed purely to reach `date_of`,
  in two places; `lunar.rs:307` builds the same `CivilDay` twice in one
  expression.
  **fixed** — `time::date_at(jd, zone)` is the question asked directly, and
  `CivilDay::date_of` defers to it. The doubled construction is one binding.
- **F-42** `is_waxing` is read at day start, `illumination` at local noon. They
  disagree on principal new/full days. A comment is probably the right fix.
  **fixed by comment** — day start is right for `is_waxing`, because the phase
  name is decided there and the two have to agree or the glyph would point one
  way while the words said the other. Stated on both `MoonDay` and `MoonCell`,
  with why the disagreement is invisible.
- **F-43** `DayDetail.tsx:309` says tithis are "prevailing one first". They are
  not sorted.
  **fixed by comment** — time order is what the boundaries in the captions are
  for, so the comment is corrected rather than the order. The prevailing one is
  marked by weight, not position.
- **F-44** `glyphs.rs` says the retrograde stroke is "heavier than a glyph's
  because it is drawn at roughly a third of the size". It is drawn at 61% and
  renders lighter. `render.rs`'s inset comment describes a box that is not the
  inked box.
  **fixed** — the stroke comment now gives the real ratio and the real rendered
  weights, 1.21pt against the glyph's 1.24pt; the inset comment is gone with the
  constant it described, replaced by the ink-based placement of F-10.
- **F-45** `se_body_id` is computed before the `Rahu | Ketu` early return.
  **fixed** — the node return moved above it, and above the lock it no longer
  needs to take.
- **F-46** `missing_data_directory_is_an_error_not_a_silent_downgrade` calls
  `engine()` first, so it returns `AlreadyConstructed` and never reaches
  `construct`. It duplicates `a_second_engine_is_refused`, and the `is_dir()`
  check it was meant to cover has no coverage — an existing but empty directory
  passes and downgrades every result to Moshier.
  **fixed, both halves.** `construct` asks the data files for a date they must be
  able to serve and refuses the directory if the flags come back Moshier, so an
  empty directory is `EphemerisDataUnusable` rather than a silent downgrade. The
  guards moved to `tests/construction.rs`, its own process, where they can reach
  `construct` at all; the empty-directory assertion fails against the old check.
- **F-47** `<For each={months()}>` allocates three fresh objects per read and
  keys by reference, so every commit disposes and rebuilds 126 cells mid-gesture.
  `<Index>` is the right primitive. Same pattern in `DayDetail`.
  **fixed** — `Index` in the scroller and in `DayDetail`'s span and event lists.
- **F-48** Three grids sit in the accessibility tree: 18 rows, 126 gridcells, 84
  of them off-screen behind `overflow: hidden` with no `aria-hidden`.
  **fixed** — the two off-screen grids are `aria-hidden` and their cells are out
  of the tab order, by the same `active` flag F-12 needed.
- **F-49** `role="radio"` on a boolean toggle, and no `role="radiogroup"` around
  the `Choice` groups.
  **fixed** — a `ChoiceGroup` carries `role="radiogroup"` around each set, and
  the colour-mode row is a `Toggle` with `role="switch"`. Identical markup and
  classes, so nothing moves on screen.
- **F-50** `an_unknown_zone_falls_back_to_greenwich` builds a `Resolved` locally
  and asserts nothing about `from_time_zone`.
  **fixed** — `for_zone` is the part that does not read the machine's clock, so
  the test asserts both branches of it.
- **F-51** Doc drift, all of it: ARCHITECTURE §3.3's IPC table matches none of
  the ten registered commands; the ts-rs claim is contradicted by
  `contract.rs`; §5's cache key names fields that do not exist; §7 lists
  `location.manual` and `appearance.theme` which do not exist and omits
  `calendar.month_system`; §10's autostart claim is unimplemented. ISSUES I-038
  still records `?subject=` navigation as the resolution, which the no-reload
  change replaced. DESIGN §10.1 / §10.3 document a month picker and header
  chevrons that were removed; §9.2's cold-month skeleton is not implemented and
  cannot be since layout moved to the back end; the `Today,` spoken prefix and
  the Home/End and focus rules do not match the code.
- **F-52** `shift_gregorian` overflows on a huge offset — debug panic, release
  wrap to a nonsense year. Not reachable through the UI.
  **fixed** — counted in `i64` and the year clamped, so an impossible request
  reaches `DateKey::new` and is refused as an invalid date.
- **F-53** `geo::distance_km` can produce NaN for antipodal points and
  `nearest_place` then silently skips the entry; degenerate inputs return
  confident wrong answers rather than `None`.
- **F-54** `watch_for_midnight` uses `thread::sleep` across system sleep, so a
  Mac asleep through midnight may keep yesterday's disc. D-017 rules out polling;
  a wake notification is the fix.
- **F-55** *(speculative)* `animate(0)` can leave the strip wedged one month off
  after a single wheel delta beyond ~720 px, which WebKit's coalescing makes
  plausible.
