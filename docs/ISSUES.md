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
| I-024 | chore | `make install`, ad-hoc signing | M4 | open |
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
| I-036 | bug | Tray glyph weight does not match system menu bar icons | M2 | done |
| I-037 | bug | Enabling a graha crashed the app | M2 | done |
| I-038 | bug | A graha's tray item opened the Moon's calendar | M2 | done |
| I-039 | feat | Panel uses the system popover material | M2 | done |
| I-040 | feat | Continuous month scrolling, settling on release | M2 | done |
| I-041 | bug | Clicking below the panel did not close it | M2 | done |
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

### I-037 — Enabling a graha crashed the app — done
`update_settings` called `tray::rebuild` directly. Commands run on the async
runtime, and a status item is an AppKit object: creating, removing or redrawing
one off the main thread takes the process down. All three call sites - the
settings write, the midnight redraw and the location update - now dispatch
through `run_on_main_thread`.

### I-038 — A graha's tray item opened the Moon's calendar — done
The panel window is created once and reused for every tray item, so it has to be
told which subject it was opened for. Three ways were tried and all failed:

1. A Tauri event. One-shot; never observed arriving.
2. A direct call evaluated in the page. Demonstrably arrives - the same eval can
   write to the DOM - but a Solid signal set from that context never reached the
   render.
3. Re-reading the subject from `bootstrap` on window focus. The backend returned
   the right subject every time, but the webview raises no focus or
   visibilitychange event when the window is shown, so nothing triggered it.

A fourth attempt exposed a separate bug worth recording on its own: a non-keyed
`<Show>` memoises its condition with `equals: (a, b) => !a === !b`, so the
accessor it hands a callback child only re-emits when truthiness changes.
Replacing one bootstrap object with another never notified, and the child kept
the value from mount.

First resolved by carrying the subject in the page's own URL: the backend
navigated the panel to `?subject=<key>` as it opened, and the page read its own
URL on load, so nothing had to propagate. Measured at roughly 450 ms from click
to a fully drawn panel.

**That is no longer the resolution.** Reloading the document on every open threw
away every month already in hand: the window was empty for 100 to 200 ms and the
calendar landed at 265 to 414 ms. The page is now never reloaded, and the subject
is pushed twice - a `chandra://open` event as the window is shown, and a second
announcement when it takes focus, which the platform raises after the show has
completed. Both call the same idempotent handler with the same value, so arriving
twice is arriving once, and neither can be the one that is missed. What the
reload used to give implicitly - today, no selection, the calendar view, a fresh
scroller - is now done on purpose in that handler.

### I-041 — Clicking below the panel did not close it — done
The window was 320 x 620 while the panel drew only 332px of it, so clicks in the
transparent remainder landed on the window and did nothing. Now that every view
swaps inside a fixed-height region, the panel's height is constant and the window
is exactly its size. That also made the popover material possible: the material
fills the whole window, so an oversized window would have shown a large
translucent rectangle below the panel.

### I-042 — Month scrolling stalled and lost its place — done
Reported as: scroll down a month, scroll back up, and the strip shows the previous
month but hangs before it aligns and the title catches up. Four separate defects,
each found by measuring rather than by reading:

1. **The strip could wedge permanently.** Settling animated a CSS transition and
   committed on `transitionend`. That event never arrives when the target equals
   the current value, which is exactly what a click on a day produced: settle to
   an offset already held, no transition, no event, and the `settling` guard left
   raised, refusing every gesture afterwards. `transitionend` also bubbles, so any
   day cell's own transition could commit the wrong month. Now a
   `requestAnimationFrame` loop, which finishes because it counts frames.

2. **Every month blanked at the moment of the commit.** The grid read straight
   from three Solid resources, and a resource drops to `undefined` while it
   refetches. Committing changed all three keys at once, so the strip emptied and
   the title cleared until three IPC round trips returned - even though every
   month involved was already in hand. `Panel` now holds months in a `Map` keyed
   by identity and renders from that; the resources only fetch.

3. **An interrupted settle threw its month away.** The commit was owed until the
   animation ended, so a gesture arriving mid-settle cancelled it. Four quick
   flicks moved three months. Movement is now banked as it happens: a whole
   month of travel promotes the neighbour and reduces the offset by that month's
   height in one batch, which is a no-op on screen because the neighbour is
   already drawn there. Nothing is ever owed.

4. **The flick detector could not fire.** Two causes, both only visible with an
   on-screen trace of every event. The sample window kept a minimum of one
   sample, and WebKit coalesces wheel events under load - a whole gesture can
   arrive as one delta more than a window apart from the last - so the window
   collapsed to a single sample and reported no motion. And release speed was
   tested against a wall clock, which is a race with the re-render a month change
   triggers: the idle timer fired 190ms after the last event rather than 110, past
   the allowance. The window now keeps two samples whatever their age, and a
   wheel is given no staleness test at all, because the idle timer that ends the
   gesture is the stop.

Verified in the installed app with synthesised trackpad events: one gesture each
way returns to the starting month with the title correct and the grid aligned;
five separated gestures step exactly five months; the same five back returns to
the start. Travel maps to distance, so a run of quick partial gestures covers the
ground it was given rather than one month per gesture.

### I-043 — Combustion and retrograde were invisible in the day view — done
The grid marked both; the day that mark opens said nothing about either. See
D-019. Combustion was also judged at local noon in the cell and at sunrise in the
day, which could have made the two disagree outright; both now ask
`combustion_at` at local noon, and a test walks a whole month asserting they
match.

### I-044 — The month system read as a per-graha setting — done
Not a data defect: the system is held once in settings, the cursor is built in one
place from it, and every subject resolves the same days and the same label. Now
proved by `every_subject_shares_one_month_system`, which compares each graha's
month against the Moon's across all three systems. The report came from the
settings section being reached from whichever subject's panel was open, with
nothing on screen saying the choice was calendar-wide. The section now says so.

### I-045 — Lunar mode still showed Gregorian dates — done
The month label and the day range were lunar; every cell still printed the
Gregorian day. Designed first in `docs/design/lunar-dates.md`, then implemented
as D-021. The cell now prints the tithi in panchang notation with the Gregorian
day in the top-right corner, a skipped tithi is marked rather than smoothed over,
and the year is Vikram Samvat.

Two things had to move to the back end to make it correct:

- **The grid.** The front end laid out 42 cells by guessing dates either side of
  the month's first day, which works for a Gregorian month and cannot work for a
  lunar one. It now receives 42 cells with an `in_month` flag, so the leading and
  trailing cells carry real data instead of being drawn blank.
- **The tithi.** Computed once per month and shared by every subject, because a
  day is named the same whichever graha is plotted on it. Measured at 11.7ms for
  a cold lunar month against a 30ms budget; the second subject over the same
  month costs 7.7ms because the tithis are already resolved.

Verified in the installed app against Shravana 2083 (13 Aug - 11 Sep 2026): the
month opens on `S1` and closes on `A`, `P` falls on 28 August, `K13` is skipped
between 10 and 11 August with the dot on the earlier cell, and `S12` is repeated
across 24 and 25 August.

The verification originally also named a rule joining the two days of a vriddhi.
That mark was removed with the rest of the underline vocabulary (D-024). A
repeated tithi is now visible only as the same numeral on two cells, and in the
spoken label: `vriddhi, the same tithi names the day after`. The kshaya dot moved
from the cell's top-left corner onto the numeral, because the corner now carries
the Gregorian date.

### I-046 — The menu bar showed no state at all — done
`℞` in the lower right of a retrograde graha's icon, with the glyph shrunk to
free the corner. Verified against Shani, retrograde on 21 August 2026: the icon
reads `♄℞` in the menu bar at real size. Retrograde only, by the user's choice -
a template image varies in alpha alone, so a second state would have to be
another shape in another corner of a 22 point square.
