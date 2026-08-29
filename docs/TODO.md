# Work

Two halves. **In flight** is what is being built now, broken into tasks fine
enough that none of them hides another. **Shipped** is the record of what was
built and why, kept because the reasoning is the part worth having later.

Status: `in flight` · `blocked on you` · `ready` · `queued` · `done`

---

# In flight

Three epics, each with a design document that is the specification. A task is
not done until it is built, tested, verified in the running app, and its
document says what shipped.

---

## E1. Lagna Kundali — a chart of the sky now

Specification: **`docs/design/kundali.md`**.

A gochara chakra: where the nine grahas stand right now, drawn as a chart. Not a
birth chart — there is no birth time and no birth place, and calling it one
would be a lie about what it computes.

**All three common formats are supported**, chosen in settings, defaulting to
South Indian. They share one data model: which rashi each graha occupies, plus
the rashi holding the lagna.

### E1 tasks

| # | Task | Status | Notes |
|---|---|---|---|
| 1.1 | **Sidereal ascendant in the engine** | ready | The blocker. `swe_houses_ex` with `SEFLG_SIDEREAL`, named-field request struct so the arguments cannot be transposed. §2.2 |
| 1.1a | Test: independent closed-form ascendant agrees | ready | The guard. A second derivation, not a second reading; φ is in the formula so a transposed argument cannot agree. Carries the ayanamsa identity as one assertion inside it. §2.2.1 |
| 1.2 | Obliquity, for the polar test | ready | §2.3 |
| 1.3 | `Chakra` payload — 12 rashis of occupants, lagna rashi and degree | ready | Shared by all three formats |
| 1.4 | North Indian renderer | ready | **Default.** 4 diamonds, 8 triangles, SVG polygons with a text anchor each. Cannot be drawn without a lagna |
| 1.5 | South Indian renderer | ready | 4×4, centre removed, Meena top-left, clockwise. The fallback where there is no lagna |
| 1.6 | East Indian renderer | ready | Same geometry as 1.5, different origin and direction |
| 1.6a | ~~Location quality gate~~ | superseded | Answered by **E4**: a location becomes mandatory, so there is no centroid case to gate |
| 1.7 | Chart format setting, and the schema migration | ready | §6. Schema 7 |
| 1.8 | Panel view and the header title | ready | §5.1–5.3 |
| 1.9 | Tray item, **on by default**, live lagna in the tooltip | ready | §5.4, D-028 for the tooltip. First time a new install gets two menu bar items |
| 1.10 | Spoken form of the chart | ready | §7. A list, not a grid |
| 1.11 | Degraded states: no lagna, outside the range, polar | ready | §8 |
| 1.12 | ~~Decide what the feature is called~~ | done | **`Lagna Kundali`**, glossed `Ascendant chart`. Not `Gochara` — see below |

### E1 decisions taken

| | Question | Answer |
|---|---|---|
| a | Default format | **North Indian.** All three supported, chosen in settings |
| b | Its own status item, or reached from the moon panel? | **Its own**, toggleable in settings, **on by default** |
| c | What is it called? | **`Lagna Kundali`**, glossed **`Ascendant chart`** |

**On the name.** `Gochara` was accepted and then withdrawn on evidence. In
common usage — AstroSage, Prokerala, AstroCAMP — *gochar* means transits read
**against a natal chart**, which this app does not have and will never ask for.
Using it would be the same class of error as calling the thing a birth chart.
`Kundali` alone carries the same problem: Drik Panchang's kundali pages are
*janma* kundali, birth charts.

`Lagna Kundali` is a chart cast with the ascendant as house 1 for a given
instant. Cast for now, that is exactly this object, and it implies no birth.
Drik Panchang's own English for lagna is "Hindu Ascendant / Rising Sign", which
gives the gloss.

### E1 decisions still open

None. E1 is unblocked once E4 lands.

---

## E2. Explaining the marks

Specification: **`docs/design/legend.md`**.

65 distinct marks, notations and abbreviations were inventoried from source.
**21 have no carrier a sighted reader can use** — no visible words, no tooltip,
no hint anywhere. Recommended surface: a `Marks` section in settings, with
specimens drawn by the real components so it cannot drift.

### E2 tasks

Not yet broken down — the epic is specified but not scheduled. Two defects it
found are separable and can be fixed before the epic starts:

| # | Task | Status | Notes |
|---|---|---|---|
| 2.1 | Muhurta auspiciousness is spoken but never drawn | ready | A VoiceOver user is told `avoid`/`auspicious`; a sighted reader is not. Inverts the app's own contract |
| 2.2 | `Ari` in a cell against `Mesha` in the day view, same sign | ready | Fix at the source or explain it — E2 open question |

---

## E3. Lunar and solar concepts, and where each belongs

Specification: **`docs/design/calendar-modes.md`**.

24 elements cross a calendar boundary: 11 correct and intended, 7 correct but
undocumented, 4 inconsistent, 2 wrong. One of the two wrong cases is already
fixed (below); the other is open.

Proposed principle:

> The mode decides three things and nothing else: which civil days a month
> holds, what the month is called, and what a day is called inside it.
> Everything else is a fact about a civil day or a subject, and appears in all
> three modes.

### E3 tasks

Not yet broken down. Its seven open questions need answering before the tasks
are real; two matter most — whether the solar day header carries the date or the
tithi, and whether the `Daylight` row stays.

| # | Task | Status | Notes |
|---|---|---|---|
| 3.1 | ~~A solar day is named by its date again~~ | done | Commit `40c479c`. The second of the two wrong cases |
| 3.2 | `state.rs:76` and `tray.rs:22` cite D-019 where they mean D-009 | ready | Found in passing |

---

## E4. A location becomes mandatory

No specification document yet.

Chandra currently works from first launch with no location, falling back through
D-007's chain to a timezone centroid that can be a thousand kilometres out. That
was defensible while the app showed phases and tithis. It is not defensible now:

- **The lagna needs it.** One degree of longitude is four minutes is about one
  degree of ascendant. A centroid used from the wrong end of a large country is
  around eight degrees out, so it names the wrong rashi roughly a quarter of the
  time — and the North Indian chart *is* the lagna, so a wrong lagna is a wrong
  chart, not a wrong detail.
- **It was already affecting the calendar.** Sunrise, moonrise, the tithi a day
  is named after, and every muhurta are all computed from the observer. A
  centroid a thousand kilometres east gives a sunrise about forty minutes wrong,
  which moves the tithi at sunrise near a boundary. This has been true since the
  first release and nothing said so.

### E4 tasks

| # | Task | Status | Notes |
|---|---|---|---|
| 4.1 | Decide the shape: a first-run gate, or a persistent banner until set | queued | Affects onboarding, which the app has never had |
| 4.2 | Ask for CoreLocation, and fall through to city search on refusal | queued | Both already built; what is missing is the requirement |
| 4.3 | What the app shows before a location exists | queued | Refuse to draw, or draw and mark every figure as provisional |
| 4.4 | Amend D-007 | queued | The fallback chain stops being silent |
| 4.5 | Migration for existing installs still on a centroid | queued | They have been running on a guess; say so once |

---

---

# Shipped

## 1. Correctness — the 56 audit findings are closed

`docs/AUDIT.md` carries the detail; each finding was updated in place as it was
fixed. Two independent audits ran the same instructions; they agreed on ten
findings, each caught several the other missed, and they contradicted each other
twice — both contradictions were settled by measurement and are recorded. None
ended as `wontfix`.

The method note at the top of `AUDIT.md` is the part worth keeping: two tests
passed while the thing they guarded was broken. **A fix whose test would still
pass against the old code is not finished.**

| | |
|---|---|
| Status | done |

### 1.1 The wording audit's open items — done

The second pass in `AUDIT.md` — a fact about one thing printed beside a heading
about another — closed ten immediately and left five standing. All five are now
closed.

- ~~**W-03** A Moshier month draws 42 cells with no precision note.~~ Fixed: the
  grid carries it too, and both notes were reworded. They said "Moshier
  ephemeris", which is the name of a piece of arithmetic and tells a reader
  nothing; they now name the range and the magnitude, so the note reads as a
  fact rather than a warning.
- ~~**W-04** Clock times carry no zone, and the zone need not be the
  machine's.~~ Fixed: the day view says which zone the times are in, and only
  when it is not this Mac's. For almost everyone the two are the same and a
  standing note about it would be noise on every day.
- ~~**W-06** `Ephemeris unavailable.` is shown for failures with no ephemeris in
  them, and is the fallback for any unknown code.~~ Fixed twice over. `ENGINE`
  is the back end's catch-all — a poisoned lock and a failed main-thread
  dispatch both reach it — so it now reads "This could not be computed." and
  lets the message underneath say what happened. An unrecognised code gets its
  own entry rather than borrowing `ENGINE`'s, because a code from a newer back
  end has no known cause and reusing a headline asserts one.
- ~~**W-07** An unknown elevation prints as `0 m`.~~ Fixed at the type.
  `PlaceSetting.elevation` is an `Option<f64>` and `Resolved` carries
  `elevation_known` beside the number. Rise and set are still computed at sea
  level, which is the assumption to make with no height — but the settings pane
  says "height not set" instead of reporting that assumption as a measurement.
  The city table has no elevation column, so it had been saying `0 m` for every
  city in the world. Schema 6.
- ~~**W-08** `GrahaCell.retrograde` is read at local noon while the day the cell
  opens reads its state at sunrise.~~ Fixed: the cell reads its motion *and* its
  longitude at the day's reference instant, which is what the day view uses.
  The longitude was the same bug unnoticed — the grid and the day printed
  different positions for one date.

  `graha_month`'s signature changed, as expected. The sunrises are now one
  cached primitive that both the tithi frames and the graha cells read: computed
  per subject they cost 42 sunrise searches a month that something else had
  already paid for, and the lunar budget test caught exactly that.

  The test is checked against the code it replaces, per the note above. **Guru
  turns direct on 11 March 2026 and that station falls between sunrise and noon
  at Bengaluru** — found by scanning eight years for one inside that window,
  because a month whose station falls elsewhere lets the test pass against the
  bug. Each half of the fix was reverted separately to confirm the test fails on
  each.

Five more are suspected and unverified; they are listed at the foot of
`AUDIT.md`.

| | |
|---|---|
| Status | done |

---

## 2. Contrast on the translucent panel — done

Measured on the shipped panel over ordinary content, the composited ground runs
from `#1A3033` to `#5C4E36`, and `--text-tertiary` reached 2.21:1 at the light
end — failing 4.5:1 and failing the 3:1 non-text bar as well.

Shipped as **D-023**: the scrim went to 0.55, `--text-tertiary` retired as a text
colour, and every label that used it moved to `--text-secondary`. It is kept for
the two marks with no text floor to clear — the `℞` chip's border and the switch
knob.

| | |
|---|---|
| Status | done |

---

## 3. State marks — done

Shipped as **D-024**:

- Marks do not vary between solar and lunar mode.
- No underlines. Combustion is a warm radial wash at the cell's foot, `--glare`
  `#e2603a`; colour may depict but may not encode (D-023).
- Retrograde is a dotted ring around the glyph, drawn as a bracket over the run:
  the half facing the retrograde days on the day the motion turns, a whole circle
  in between, the opposite half on the day it turns back.
- Ingress and station markers are no longer drawn at all. They are named in the
  day view's `Events` field, with their times.
- ~~Rahu and Ketu wear the retrograde mark permanently, and that is accepted.~~
  **Wrong, and never measured.** Only a *mean* node is permanently retrograde. A
  *true* node — Chandra's default — is retrograde on 74.1% of days and turns
  direct about twenty-five times a year for under four days at a time, so its
  bracket opens and closes fortnightly like any other graha's. The grid was
  right to treat them like every other subject; it was the day view that
  wrongly excluded them.
- The Moon's combustion is not suppressed in the grid, and shows in the day view
  like every other subject.

| | |
|---|---|
| Status | done |

---

## 4. Moon phase glyphs — done

The Gregorian date moved to the cell's top-right corner and the glyph took the
second line back, in both calendars and for every subject (D-024). That also
removed the two mode-dependent CSS rules — the combustion rule and the `℞`
position both had a lunar variant purely because the glyph was missing.

| | |
|---|---|
| Status | done |

---

## 5. Day view — done

Shipped as **D-025**: variant 02, one field stack for all nine subjects. A field
is a small uppercase label, the value and the hour it gives way on one line, and
a `then …` successor line. Rise and set belong to the subject, so there is no
separate sunrise row. Illuminated percentage, distance from the Sun, speed and
longitude are gone. The phase name appears in solar mode only.

### 5.1 Combustion and retrograde in the day view — done

Both are drawn, and neither costs a vertical pixel, which is what matters
given §7:

- **Combustion** is the same warm radial wash the cell carries, at the foot of
  the panel. One state, one appearance, wherever it appears. It sits behind the
  fields and darkens nothing, so every contrast figure in §2 holds unchanged.
- **Retrograde** is a dotted rail down the right gutter, turning back on itself
  at both ends. Dotted for the reason the cell's ring is: the motion is broken,
  so the line is. Suppressed for Rahu and Ketu, which are retrograde on roughly
  95% of days — a mark that is true almost always is the subject's identity, not
  its state. **Both halves of that were wrong and the exclusion is gone.** A
  mean node is retrograde on 100% of days and a true node — the default — on
  74.1%, turning direct about twenty-five times a year for under four days at a
  time. It was also a disagreement with the grid, which never excluded them.

Both are fixed to the panel rather than scrolled with the content: they are
properties of the day, not of any field in it.

Each condition is one predicate — `isCombust`, `hasRetrogradeRail` — used by
both the mark and the words, so the two cannot disagree. That is the guarantee
`the_combustion_mark_and_the_day_it_opens_agree` gives the grid, held here by
construction rather than by a test.

The design document's `--glare` `#F0C9A4` was superseded before this shipped:
the token is `#e2603a` under D-023, and the contrast figures in that document
are computed against the old value. The hairpin drawn there is simplified to a
rail; the shape it described needed a rim the panel does not have.

| | |
|---|---|
| Status | done |

---

## 6. Year navigation — done

Keeping the Moshier precision note creates an obligation: if a reading can be
degraded by navigating far enough, getting there must not be tedious.

Shipped as a jump overlay, chosen over shift-scroll because a gesture nothing
draws is not a route anyone finds. The header title is the control — it is
already the only thing on screen that names where the strip is, and giving the
picker its own button would have cost a slot the header does not have.

- `‹ VS 2083 ›` over a grid of that year's months; a month jumps the strip.
- The months come from the back end (`Almanac::month_index`), not counted in the
  front end. A Vikram Samvat year holds twelve or thirteen, and which depends on
  whether a lunation fitted inside one solar rashi. Twelve cells laid out on
  faith would put the adhika masa in the wrong place, or lose it.
- Paging uses offsets the index reports, never `± 12`, for the same reason.
- Its own IPC command: enumerating a lunar year costs up to fourteen syzygy
  searches, and every month view would have paid that for a panel rarely opened.
- Escape closes the picker before it reaches the selection underneath, and the
  arrow keys do not move a ring nobody can see.

`⇧Page Up` / `⇧Page Down` still step a year.

| | |
|---|---|
| Status | done |

---

## 7. Panchanga and jyotisha fields — done

Designed in `docs/design/panchanga.md`, then built. The day view is two panes,
split on what the fields are about rather than on how many there are:

| DAY — the civil day | POSITION — the subject |
|---|---|
| Tithi | Rashi |
| Yoga | Nakshatra |
| Karana | Nakshatra lord |
| Daylight | Motion |
| Muhurtas | Dignity |
| | Drishti |
| | Planetary war |
| | Rise and set, Combust, Events |

Day is identical on all nine panels, because a tithi does not depend on which
graha is read against it. Position is a property of the subject, and nothing in
it is the same for two subjects.

Yogas, karanas and muhurtas are toggles in **Settings › Panchanga**, all off by
default. A limb that is off is not computed rather than computed and hidden.
Dignity, drishti, planetary war and the nakshatra lord have no toggle: they cost
one positions call between them.

Notes worth keeping:

- `angles.rs` is one routine for tithi, karana and yoga. All three are divisions
  of a Sun-Moon angle that only ever increases, so there is no retrograde case
  and a boundary is crossed exactly once. `tithi.rs` was refactored onto it.
- The **Durmuhurtam table is cited, not remembered.** Two of the values I would
  have written from memory were wrong. It corroborates itself against a fact
  from a different source: Abhijit is the 8th day muhurta and is held not to
  apply on a Wednesday, and Wednesday's Durmuhurtam is the 8th day muhurta.
- **No winner is reported for a planetary war**, and Rahu and Ketu get no
  dignity. Both are disputed between authorities, and printing one reading would
  be the app asserting an interpretation.
- **Moolatrikona is not included.** Its degree ranges differ between
  authorities, and it was not among the states asked for.
- The Moon's nakshatra is the panchanga's fifth limb and is deliberately not in
  the Day pane: the payload carries the *subject's* nakshatra, so a Nakshatra
  row there would be true only when the subject happened to be the Moon.
- `DayPanchanga` stopped being optional, and `Almanac::day_detail` stopped
  taking a month system. Schema 4 migrates a settings file written before the
  toggles existed.

| | |
|---|---|
| Status | done |

---

## 8. Ingress labels — done

On the day a graha enters a rashi or a nakshatra, its cell names what it entered
instead of drawing the glyph. **Settings › Calendar › Ingress labels**, one or
the other or neither.

The label takes the glyph's place rather than joining it. A 40px cell has room
for one thing on that line, and on the one day a month a graha changes sign,
which sign it changed to is the more useful of the two: the column of glyphs
above and below still says which graha this is, and so does the header.

- **Rashi forms are western** — `Ari`, `Tau`, `Gem`. The Sanskrit names cannot
  be abbreviated to three letters and stay distinct: *Vrishabha* and
  *Vrishchika* are both `Vri`. A test asserts that collision, so if the two ever
  differ in their first three letters, transliterated forms become possible.
- **Nakshatra forms are two-part where the name is** — `P.Ash`, `U.Bha`. Six
  begin *Purva* or *Uttara* and three of each share what follows. The other
  twenty-one are the first four letters, which are distinct.
- **Not on the Moon's calendar.** It enters a nakshatra every day and a rashi
  every two and a bit, so every cell would be a label and none of them would be
  a phase.
- The label reads from the month's own event list, which was already on the
  payload for the day view. A second copy on 42 cells is a second thing to keep
  in step with the first.
- A five-character label reaches the corner the other calendar's date sits in.
  The cell takes that corner out of the space the label centres in, rather than
  letting the two overlap.

This is the one thing that puts an ingress back on the grid after D-024 took the
marker off it, and it puts it there as a word rather than as a shape.

| | |
|---|---|
| Status | done |

---

## 9. Node type — done

Chandra defaulted to the **true** node while panchangas overwhelmingly use the
**mean** one. That mismatch was also what made D-026's wrong figure plausible:
on a mean node "Rahu is always retrograde" is exactly right.

Shipped as **D-027**: a fresh install gets the mean node, existing settings files
keep whatever they name, and the picker lists the mean node first. The toggle
itself already existed, in **Settings › Astrology › Rahu and Ketu**.

| | |
|---|---|
| Status | done |

---

## 10. Not started

- GitHub remote. Deferred by D-018 until a production release; still local-only.
- CI. Described in the Makefile and the docs; no `.github/workflows` exists.
- Release DMG on tag.
