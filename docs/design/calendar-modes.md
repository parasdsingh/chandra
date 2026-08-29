# Calendar modes: what belongs in which

Status: **audit and proposal.** Nothing here is built. §5 is a list of edits for approval; §7 is
a list of questions that need an answer before those edits are correct.

Answers: "the mix of lunar cal concepts present in solar, and vice versa is a detail to be fully
understood and approved".

Every claim below is cited to a file and a line. Where the code and a document disagree, the code
is what is reported and the document is listed as drift in §3.5.

---

## 1. The three modes, and the minimum a mode must decide

`MonthSystem` has three values (`crates/almanac/src/lunar.rs:19-27`):

| Mode | Month runs | Named from |
|---|---|---|
| `solar` | Gregorian month | the Gregorian month name (`almanac.rs:730-751`) |
| `amanta` | new moon to new moon | the rashi the Sun occupies at the opening new moon (`lunar.rs:73-86, 322-323`) |
| `purnimanta` | full moon to full moon | the rashi the Sun occupies at the new moon *inside* it (`lunar.rs:314-320`) |

The two lunar modes differ in exactly two places in the whole codebase:

- `MonthSystem::boundary_elongation` — 0° or 180° (`lunar.rs:52-59`).
- `naming_jd` — a purnimanta month takes its name from the new moon it contains, not from the
  full moon it opens on (`lunar.rs:314-320`).

Everywhere else they are collapsed by `is_lunar()` (`lunar.rs:61-63`). **No surface in the app
tells amanta from purnimanta.** Both print `Shravana VS 2083` in the header for date ranges that
differ by about a fortnight.

### 1.1 The seven places the running app branches on the mode

Excluding the `key()` and `label()` string tables, this is the complete list:

| # | Site | What it decides |
|---|---|---|
| 1 | `lunar.rs:52-59` | amanta vs purnimanta boundary |
| 2 | `lunar.rs:317-320` | purnimanta naming instant |
| 3 | `almanac.rs:203-223` | solar grid extent and header label |
| 4 | `almanac.rs:276-278` | solar grid carries **no** tithi frames |
| 5 | `almanac.rs:344-369` | solar year index is January–December |
| 6 | `src-tauri/src/tray.rs:213` | moon tooltip says phase or tithi |
| 7 | `src/components/Panel.tsx:674` | the `lunar` prop into the day view |

Site 7 has two consumers: `DayDetail.tsx:175` (the civil date on the date line) and
`DayDetail.tsx:421` (the Phase field). Nothing else in the front end reads the mode.

### 1.2 The minimum a mode must decide

From sites 1–5, and nothing more is forced by the data model:

1. **Which civil days a month holds.** `Resolved.grid`, 42 cells with an `in_month` flag
   (`almanac.rs:58-63`, `month.rs:37-41`).
2. **What the month is called**, including the era and the `Adhika` qualifier
   (`MonthLabel`, `month.rs:117-127`).
3. **What a day is called inside that month** — the Gregorian ordinal, or the tithi
   (`DayCell.tsx:124-136`).

Everything else in the app is a fact about a civil day or about a subject. That claim is what §4
tests.

---

## 2. The inventory

Rows are grouped by surface. `=` means identical to the column to its left.

### 2.1 The month grid

| Element | Solar | Amanta | Purnimanta | Where |
|---|---|---|---|---|
| Cell numeral | Gregorian day-of-month | `S1`–`S14`, `P`, `K1`–`K14`, `A` | = | `DayCell.tsx:124-136`, `tithiLabel` `DayCell.tsx:76-87` |
| Paksha prefix | absent | `S` / `K`, none on `P` and `A` | = | `DayCell.tsx:130-132` |
| Corner date | **absent** | Gregorian day, no month | = | `DayCell.tsx:146-148` (gated on `tithi()`) |
| Kshaya dot | absent | drawn when `kshaya.length > 0` | = | `DayCell.tsx:140-142` |
| Vriddhi mark | absent | **nothing drawn**, spoken only | = | `MonthGrid.tsx:245-251`, D-024 |
| Moon phase glyph | drawn | drawn | = | `DayCell.tsx:150-159` |
| Graha glyph | drawn | drawn | = | `DayCell.tsx:241-266` |
| Ingress label (replaces glyph) | drawn | drawn | = | `DayCell.tsx:255-262`, `MonthGrid.tsx:118-122, 175-186` |
| Retrograde ring | drawn | drawn | = | `DayCell.tsx:164-166`, `MonthGrid.tsx:196-212` |
| Combustion wash | drawn | drawn | = | `DayCell.tsx:172-174` |
| Weekday row | locale short names | = | = | `MonthGrid.tsx:53-61`, `format.ts:127-137` |
| Grid extent | 42 cells, month + neighbours | 42 cells, syzygy to syzygy | = | `almanac.rs:210-222` / `255-260` |
| `MoonCell.tithi` / `GrahaCell.tithi` | `None` | `Some` | = | `almanac.rs:276-278`, `month.rs:53-55, 88-90` |

### 2.2 Spoken labels on a cell

| Element | Solar | Amanta | Purnimanta | Where |
|---|---|---|---|---|
| Leading token | Gregorian `20 August` | Gregorian `20 August` | = | `MonthGrid.tsx:221-224` |
| Tithi, spoken in full | absent | `Shukla Ashtami` | = | `MonthGrid.tsx:234-256, 260, 271` |
| Kshaya / vriddhi, spoken | absent | named | = | `MonthGrid.tsx:243-254` |
| `read at noon, the Sun did not rise` | absent | said | = | `MonthGrid.tsx:240-242` |
| Phase and illumination (Moon) | said | said | = | `MonthGrid.tsx:261-263` |
| `combust at noon` | said | said | = | `MonthGrid.tsx:264, 279` |
| `enters Ari` (graha) | said | said | = | `MonthGrid.tsx:275` |

### 2.3 The header

| Element | Solar | Amanta | Purnimanta | Where |
|---|---|---|---|---|
| Calendar title | `Chandra · August 2026` | `Chandra · Shravana VS 2083` | = | `Panel.tsx:568`, `almanac.rs:217-221 / 246-254` |
| Era | Gregorian year | `VS` + Vikram Samvat | = | `almanac.rs:218 / 251`, `lunar.rs:140-147` |
| `Adhika` qualifier | never (`adhika: false`) | set apart in `--text-secondary` | = | `almanac.rs:220 / 253`, `Header.tsx:187-211` |
| Kshaya masa (`Name–Name`) | never | shown | = | `lunar.rs:124-130` |
| Day-view title | **the tithi** | the tithi | = | `Panel.tsx:559-567` |
| Fallback to `20 August 2026` | **unreachable** | unreachable | = | `Header.tsx:44-47` |
| Subject glyph | drawn | drawn | = | `Header.tsx:103-121` |
| Year in the jump overlay | `2026` | `VS 2083` | = | `almanac.rs:363-368 / 437-442` |
| Months in the jump overlay | always 12 | 12 or 13 | = | `almanac.rs:349-361 / 392-430` |

### 2.4 The day view — the date line and the Day pane

| Element | Solar | Amanta | Purnimanta | Where |
|---|---|---|---|---|
| `TODAY` | shown | shown | = | `DayDetail.tsx:167-170` |
| Weekday | shown | shown | = | `DayDetail.tsx:171` |
| Civil date | **absent** | shown | = | `DayDetail.tsx:175-177` (gated on `lunar`) |
| Vara name | shown | shown | = | `DayDetail.tsx:178` |
| Tithi field | **shown** | shown | = | `DayDetail.tsx:313` |
| `· kshaya` note | **shown** | shown | = | `DayDetail.tsx:641-655` |
| Yoga field | **shown** when on | shown when on | = | `DayDetail.tsx:315-317` |
| Karana field | **shown** when on | shown when on | = | `DayDetail.tsx:319-324` |
| Daylight (sunrise, sunset) | **shown** | shown | = | `DayDetail.tsx:329-346` |
| Muhurtas | **shown** when on | shown when on | = | `DayDetail.tsx:351-375` |
| `DayPanchanga` on the payload | **present** | present | = | `day.rs:44-60, 185-233` |

### 2.5 The day view — the Position pane

| Element | Solar | Amanta | Purnimanta | Where |
|---|---|---|---|---|
| Phase | shown | **absent** | = | `DayDetail.tsx:421-427` (gated on `!lunar`) |
| Rashi | shown | shown | = | `DayDetail.tsx:429-432` |
| Nakshatra + pada | shown | shown | = | `DayDetail.tsx:434-437` |
| Nakshatra lord | shown | shown | = | `DayDetail.tsx:443-447` |
| Motion | shown | shown | = | `DayDetail.tsx:452-469` |
| Dignity | shown | shown | = | `DayDetail.tsx:471-479` |
| Drishti | shown | shown | = | `DayDetail.tsx:483-511` |
| Planetary war | shown | shown | = | `DayDetail.tsx:516-527` |
| Rise and set | shown | shown | = | `DayDetail.tsx:536-547` |
| Combust | shown | shown | = | `DayDetail.tsx:553-559` |
| Events | shown | shown | = | `DayDetail.tsx:561-577` |
| Glare wash, retrograde rail | drawn | drawn | = | `DayDetail.tsx:128-133` |

### 2.6 The menu bar

| Element | Solar | Amanta | Purnimanta | Where |
|---|---|---|---|---|
| Moon disc | drawn from illumination now | = | = | D-028 |
| Moon tooltip | `Chandra - Waxing Gibbous` | `Chandra - Shukla Ashtami` | = | `tray.rs:212-218` |
| Graha tooltip | rashi, and `retrograde` | = | = | `tray.rs:221-231` |
| `Snapshot.tithi` computed | **always** | always | = | `almanac.rs:645-647` |

---

## 3. Classification

**Twenty-four elements cross a calendar boundary.** Counted below.

### 3.1 Correct and intended — 11

A fact about the civil day or about the subject, belonging to neither calendar. Each is already
argued on the record.

| # | Element | Why it is right | Record |
|---|---|---|---|
| 1 | Vara name in solar (`DayDetail.tsx:178`) | The seven-day cycle is not a lunar invention and is what the muhurta table is keyed on (`day.rs:202, 220`) | D-021 |
| 2 | Sunrise as the reference instant, both modes (`day.rs:158-164`) | A property of the day and the place | `day.rs:147-157` |
| 3 | Rashi, nakshatra, pada, nakshatra lord (`DayDetail.tsx:429-447`) | Sidereal positions of the subject; no calendar owns them | D-025 |
| 4 | Dignity, drishti, planetary war (`DayDetail.tsx:471-527`) | Same | `panchanga.md:103-118` |
| 5 | Motion, rise and set, combustion (`DayDetail.tsx:452, 536, 553`) | Same | D-019, D-025 |
| 6 | Combustion wash and retrograde ring on every cell (`DayCell.tsx:164, 172`) | "A state drawn in one and not the other is a state nobody can learn" | **D-024** |
| 7 | Subject glyph on the cell in both modes (`DayCell.tsx:150-159, 241-266`) | The corner freed the second line | **D-024** |
| 8 | Gregorian date in the corner of a lunar cell (`DayCell.tsx:146-148`) | A lunar day still has to be findable | **D-024** |
| 9 | Civil date on the lunar day view's date line (`DayDetail.tsx:175-177`) | Same | `DayDetail.tsx:172-174` |
| 10 | Weekday row, locale first-day, both modes (`MonthGrid.tsx:53-61`) | Chosen by the user over forcing Sunday | **D-021** |
| 11 | Ingress labels in all three modes (`MonthGrid.tsx:175-186`) | Gated on the subject, never on the mode | **TODO §8** |

### 3.2 Correct but undocumented — 7

Right under any reading of §4's principle, but no decision records them and two documents still
assert the opposite.

| # | Element | Where | The gap |
|---|---|---|---|
| 12 | Tithi field in the solar day view | `DayDetail.tsx:313` | `lunar-dates.md:825-828` files this as an **open question**. It was built and never answered. |
| 13 | Yoga in solar | `DayDetail.tsx:315` | D-021 says yoga "stays out"; it is now in, in both modes |
| 14 | Karana in solar | `DayDetail.tsx:319` | Same |
| 15 | Muhurtas in solar | `DayDetail.tsx:351` | No record. `panchanga.md:78-84` argues it in passing |
| 16 | `DayPanchanga` non-optional | `day.rs:44, 185-233` | Argued in `panchanga.md:73-101`, never lifted into DECISIONS |
| 17 | Spoken cell date is Gregorian in lunar mode | `MonthGrid.tsx:221-224` | Stated only in a code comment |
| 18 | `Snapshot.tithi` computed in solar mode | `almanac.rs:645-647` | Free — 12° of an elongation already in hand — but nothing says so outside the comment |

### 3.3 Inconsistent — 4

Present in one mode and absent in the other with no principled reason.

**19. The grid has no tithi in solar; the day does.**
`almanac.rs:276-278` returns `None` in solar mode, so no cell carries a tithi. `day.rs:185`
computes one unconditionally. The two were decided by the same argument (D-010) five months
apart, and only one of them was revisited. The cost asymmetry is real — 42 sunrises against 1 —
but it has never been written as the reason for the split, and `docs/DECISIONS.md:330` still
claims solar mode "pays nothing: no tithi is computed for it at all", which the day view has
made false.

**20. The Phase field is suppressed in lunar mode.**
`DayDetail.tsx:421` gates Phase on `!props.lunar`. D-025's reason (`DECISIONS.md:424-425`) is
that the tithi says it more precisely and is already the header's title. That reason no longer
separates the modes: the header now carries the tithi in **both** (§3.4, case 22), and the Day
pane carries a Tithi field in both. The tithi is therefore printed twice in lunar mode and the
phase is dropped there for a duplication that exists identically in solar.

**21. The cell's corner is empty in solar.**
`DayCell.tsx:146-148` gates the corner on `tithi()`. In lunar the corner carries the Gregorian
day; in solar it carries nothing, because the frames that would supply a tithi are not computed
(case 19). This is defensible — see §5.5 — but nothing states it, and read from the outside the
corner looks like a lunar-only affordance rather than a consequence of a cost decision.

**22. `· kshaya` in the solar day view.**
`DayDetail.tsx:649` appends the note in all three modes. The comment above it
(`DayDetail.tsx:643-648`) justifies it as "explains a number missing from the grid" — a grid that
in solar mode never printed a tithi number in the first place. The note itself is a true fact
about the tithi; the stated reason for printing it is false in one of the three modes.

### 3.4 Wrong — 2

Asserts something untrue in that mode.

**23. In solar mode the day view's header shows the tithi.**
`Panel.tsx:553-569`. The `view() === "day"` branch reads `detail()?.panchanga` and returns
`Shukla Ashtami`. It contains no mode test. The guard `if (!panchanga) return ""`
(`Panel.tsx:561`) **was** the solar branch: when this was written at `f949f4e`, `panchanga` was an
`Option` and was `None` in solar. Commit `0b628aa` made it non-optional and did not touch this
function. The comment three lines above it — "In solar mode the western date is the name and the
header keeps it" (`Panel.tsx:557-558`) — now describes behaviour the code no longer has.

**24. In solar mode the day view prints no civil date anywhere.**
A consequence of 23. `Header.tsx:44-47` falls back to `formatDateHeading(props.selected)` only
when `props.title` is empty, and it never is, so that fallback is dead code.
`DayDetail.tsx:175-177` prints the civil date only when `props.lunar`, which is false. A solar
user opening 20 August 2026 sees `Krishna Saptami` in the header and `THURSDAY · Guruvara` under
it. The date is gone from the surface, and the mode whose whole premise is that the Gregorian
date names the day is the one that stopped saying it.

Both are regressions from `0b628aa`, not design choices. Neither is recorded anywhere.

### 3.5 Documentation drift — 7 statements now false

Not behaviour, but each would mislead the next change.

| Where | What it says | Truth |
|---|---|---|
| `almanac.rs:583-593` | Two stacked doc comments. The first: "The month system is a parameter because it decides whether the day has a panchanga at all" | It is not a parameter. The second comment, directly below, says so |
| `day.rs:32-37` | `DayPanchanga` is "Present only in a lunar month" | Present in both (`day.rs:89`) |
| `day.rs:62-68` | `MoonDay`: "Tithi, yoga, karana and muhurta are deliberately absent... see D-010" | The struct carries all four at `day.rs:90` |
| `DECISIONS.md:330` (D-021) | "Solar mode is unchanged, and pays nothing: no tithi is computed for it at all" | True of the grid, false of the day view |
| `DECISIONS.md:334-336` (D-021) | "Yoga and karana stay out" | Both are in, in both modes |
| `DECISIONS.md:351` (D-021 amendment) | "There is no Sunrise row (D-025). Sunrise is Surya's rise" | The Day pane has a `Daylight` row carrying sunrise and sunset, on all nine panels, in all three modes (`DayDetail.tsx:329-346`) |
| `lunar-dates.md:556-562` | "Solar mode... unchanged, pixel for pixel"; "The day view keeps exactly the six D-010 fields. No tithi row, no sunrise row, no vara name" | All three are present in solar mode |
| `DayCell.tsx:4-11` | "Only the label changes between the two modes" | There are three modes, and the corner and the kshaya dot change too |

---

## 4. The principle

Three candidates were put. Each is tested against the twenty-four cases above, not against
intuition.

### 4.1 Candidate C — "each mode is self-contained; nothing crosses"

**Rejected on three counted cases.**

It forbids case 8 (the Gregorian corner), case 9 (the civil date on the lunar date line) and case
17 (the Gregorian spoken date). All three were argued for deliberately: D-024 moved the corner
there, and `DayDetail.tsx:172-174` states the reason — "A lunar day still has to be findable in
the world the user lives in."

It also forbids case 1, the vara in solar mode, on the grounds that the Gregorian calendar has
weekdays. The vara and the weekday are the same seven-day cycle, so this is a rule against
printing a fact in Sanskrit.

Worst, it has nothing to say about cases 3, 4 and 5 — rashi, nakshatra, dignity, drishti, war,
rise, set, combustion. Those belong to neither calendar, so a rule about crossing between the two
does not reach them, and they are the majority of the day view.

### 4.2 Candidate B — "a lunar concept appears in solar mode only where the solar calendar has no equivalent"

**Rejected: "equivalent" carries the whole rule and cannot be settled.**

- Muhurtas (case 15): no Gregorian equivalent, so keep. Clear.
- Tithi in the solar day view (case 12): the Gregorian equivalent is the moon phase, which the
  Position pane prints (`DayDetail.tsx:421`). B therefore **deletes** the Tithi field in solar.
- Vara (case 1): the equivalent is the weekday, printed one span earlier
  (`DayDetail.tsx:171, 178`). B deletes the vara in solar — and with it the label on the thing
  the Durmuhurtam table is indexed by.
- Yoga (case 13): is the equivalent nothing, or is it the phase? Both readings are arguable.

Every hard case turns on a judgement B does not supply, and on the two cases where it does give
a clear answer, it deletes a field the user asked for.

### 4.3 Candidate A — "the calendar decides only how months are bounded and named; everything else is a fact about a day"

**Survives, with one clause added.** As stated it cannot explain the cell numeral: the Gregorian
ordinal and the tithi are both names for a day, and a rule about months does not reach them. The
clause makes that explicit rather than leaving it to be inferred:

> **The calendar mode decides three things and nothing else: which civil days a month holds, what
> that month is called, and what a day is called inside it. Every other field is a fact about a
> civil day or about a subject, and appears identically in all three modes.**

Tested against all twenty-four:

| Case | A's verdict | Current behaviour |
|---|---|---|
| 1–5, 10, 11, 13–16, 18 | facts → all modes | agrees |
| 6, 7 | facts → all modes | agrees (this **is** D-024, restated) |
| 8, 9, 17 | see §4.4 | agrees |
| 12 | tithi is a fact about the day; it is only the *name* in lunar | agrees |
| 19 | the grid numeral is a name → lunar only | agrees for the numeral; the corner needs §4.4 |
| 20 | phase is a fact → all three modes | **disagrees** |
| 21 | see §4.4 | agrees, once stated |
| 22 | kshaya is a fact about the tithi → all modes; the *dot* explains a numbering and is a name → lunar only | agrees; the comment does not |
| 23, 24 | the day-view header names the day → civil date in solar, tithi in lunar | **disagrees** |

A also settles what §1 could not: **the `Adhika` qualifier, the era, the kshaya masa and the
13-month year are all naming**, so their absence in solar is correct rather than an omission.
And it settles the ingress label (case 11) without a special case: it is a fact about the
subject, so it is gated on the subject and not on the mode, which is what the code does.

### 4.4 The clause A needs for the corner

Cases 8, 9, 17 and 21 all concern the same object: **the civil date shown in a mode that does not
use it as the day's name.** A does not govern it, because it is neither a name nor a fact the
mode owns. State it as its own rule rather than leaving it to look like an inconsistency:

> Where a mode names days by something other than the civil date, the civil date is carried
> alongside — in the cell's corner, on the day view's date line, and first in every spoken label.
> Where a mode names days *by* the civil date, there is nothing for that slot to hold.

That makes case 21 principled instead of accidental: the solar corner is empty because the
civil date is already the numeral, not because the tithi was too expensive to compute. The cost
argument then becomes a second, independent reason not to compute frames in solar — which is
where it belongs, since a cost is not a design.

### 4.5 The one exception A does not cover

The menu-bar tooltip (`tray.rs:212-218`) switches on the mode and prints a fact — the phase, or
the tithi. Under A, both are facts and neither is a name, so the tooltip should not branch at all.
D-028 argues it on **relevance** rather than on truth: the percentage lit is a solar-calendar
question the icon already answers, and in a lunar month the useful fact is the tithi.

That argument holds, and it is about a 22-point icon rather than about the calendar. It is
recorded here as a **named exception** to A, not as a case A gets wrong. §7 asks whether it should
stay one.

---

## 5. What the principle changes

Five edits. Each is stated as the whole change, not as a step.

### 5.1 The day-view header names the day in the mode in force

`Panel.tsx:553-569` must branch on `month_system`. In solar the title is
`formatDateHeading(selected)`; in either lunar mode it is the prevailing tithi. This restores
what `Panel.tsx:557-558` already claims and what `Header.tsx:41-47` was written for.

The empty-string protocol goes with it. `Header.tsx:44-47` currently infers "the header is not
carrying a title" from `!props.title` and reaches for the date itself. Once `Panel.tsx` decides,
that inference is a second place the same decision is made, and the two can disagree — which is
how this broke. `Header` should take the finished title.

Closes cases 23 and 24. Makes `Header.tsx:44-47` reachable or deletes it.

### 5.2 The Phase field appears in all three modes

Remove the `!props.lunar` gate at `DayDetail.tsx:421`. The phase is a fact about the day; it is
what the cell's glyph draws in every mode (`DayCell.tsx:150-159`) and what the tray icon draws in
every mode. Suppressing it in lunar mode is the only place in the app where a state is drawn in
one calendar and named in words in only one of them — the thing D-024 exists to forbid.

The duplication D-025 was avoiding survives the change, and is not made worse by it: after 5.1
the lunar header says `Shukla Ashtami` and the Day pane says `Shukla Ashtami` on the Tithi row
regardless of what Position does.

Closes case 20. Amends `DECISIONS.md:424-425`.

Rejected alternative: delete the Tithi row from the Day pane in lunar mode, so the header is the
only place the tithi appears. That would make the Day pane differ between modes, which is exactly
what `panchanga.md:50-56` built the pane split to avoid — "Every one of these nine panels shows
the identical Day pane."

### 5.3 The `lunar` prop shrinks to one job

After 5.1 and 5.2, `props.lunar` has exactly one consumer: the civil date at
`DayDetail.tsx:175-177`, which is §4.4's rule. `PositionPane`'s `lunar` prop
(`DayDetail.tsx:202, 390`) is then unused and goes. The doc comment at `DayDetail.tsx:69-75`
currently lists two jobs and must say one.

### 5.4 The kshaya note is justified by what it is

`DayDetail.tsx:643-648` explains the note as a caption for a missing grid number. Under §4.3 it
is a fact about the tithi — it held no sunrise, so no day is named after it — and that is true in
all three modes. The note stays; the comment is rewritten to the fact. The **dot** on the cell
(`DayCell.tsx:140-142`) stays lunar-only, because it annotates a numbering that only lunar mode
prints.

Closes case 22. No behaviour change; this is a claim being corrected.

### 5.5 The solar grid keeps no tithi, and a decision says why

Two independent reasons, and both should be written down because either alone is weak:

1. §4.4: the solar cell's corner has nothing to hold, because the civil date is already the
   numeral.
2. Cost: 42 sunrise searches a month for a slot with nothing in it. `almanac.rs:266-268` states
   this; `almanac.rs:298-326` shows the sunrises are shared with the graha cells, so the true
   marginal cost is the tithi search, not the sunrise.

Closes cases 19 and 21. Corrects `DECISIONS.md:330`, which must now read that the solar **grid**
pays nothing, and that the solar **day** pays for one.

### 5.6 Documentation

The seven statements in §3.5. A new decision record — D-029 — carrying §4.3, §4.4 and §4.5, and
amending D-010, D-021 and D-025. Without it, cases 12–16 stay undocumented: there is currently
**no decision record for the largest behavioural change ever made to solar mode.**

---

## 6. Where the recent panchanga work went too far, and not far enough

The work is `4ff5aa6` → `7bd623d`, designed in `docs/design/panchanga.md` and recorded in
`docs/TODO.md:212-262`. It is the immediate cause of most of §3.

### 6.1 The reasoning that holds

`panchanga.md:73-84` is right, and `day.rs:176-184` restates it correctly:

> That reasoning was about the **grid**, where 42 cells each cost a sunrise. It does not carry to
> a **day**, which costs one.

D-010's argument was a cost argument wearing a design argument's clothes. Separating the two is
the correct move and it is what makes §4.3 possible. The pane split
(`panchanga.md:50-56`) is the same principle applied one level down — "Day is a property of the
civil day; Position is a property of the subject" — and is the strongest sentence in the project's
design documents.

### 6.2 Too far — the Daylight row reverses D-025 without saying so

`DayDetail.tsx:329-346` puts sunrise and sunset on every day, on all nine panels, in all three
modes. D-025 removed exactly this: "There is no separate sunrise row on every subject's day"
(`DECISIONS.md:419-421`), and D-021's amendment records the removal
(`DECISIONS.md:351`). `panchanga.md:35` lists `Daylight` in the DAY column with no note that it
is a reversal, and `panchanga.md:8` — which lists three deliberate departures from the design —
does not list it.

The reversal may be right: the muhurtas are defined as fractions of the interval between them
(`panchanga.md:144-148`), so a muhurta list with no daylight row cannot be checked. But that
argument is not in any document, and D-025 stands unamended.

### 6.3 Too far — the payload change was made without its consumers

`panchanga.md:73-101` argues `DayPanchanga`'s `Option` away on its own terms and never asks what
the `Option` was being read as. Two places read it as the mode:

- `Panel.tsx:561` — `if (!panchanga) return ""`, the solar branch of the day-view title.
- `DayDetail.tsx:421` — the Phase gate, whose stated reason depends on that title.

Removing the `Option` deleted a mode test that was written as a null test. That is cases 23, 24
and 20 — the only three findings in this audit that are wrong rather than merely unrecorded, and
all three come from one commit. The lesson is narrow and worth keeping: **an `Option` that is
`None` in exactly one mode is a mode flag, and removing it removes a branch.**

### 6.4 Not far enough — the principle was found and not generalised

`panchanga.md:50-59` states the Day/Position split as a fact about the data:

> That makes the pane a fact about the data rather than a bin, so a field added later has one
> obvious home.

That is §4.3 in miniature. It was applied to two panes and never to three modes, so the grid was
never brought into it, and the mode question was left to be settled field by field — which is how
§3.3's four inconsistencies got in.

### 6.5 Not far enough — an open question was built rather than answered

`lunar-dates.md:825-828` files this exact question, under "Three, and only three. Each is a
genuine fork where a reasonable designer needs the user's preference rather than an argument":

> **Tithi in solar mode.** ... Add it to solar mode's day view, or keep solar strictly to the six
> D-010 fields?

It was one of "three, and only three" genuine forks flagged as needing the user's preference. The
panchanga work built the first branch. `panchanga.md`, `TODO.md:212-262` and `DECISIONS.md` all
fail to record that the question was answered, so `lunar-dates.md:825` still reads as open.

---

## 7. Open questions

1. **Does the solar day view's header carry the civil date or the tithi?** §5.1 assumes the
   date, on the grounds that it is what solar mode names the day by and what the code claims to
   do. Accepting what shipped instead is defensible — it would make the header say the same kind
   of thing in all three modes — but then §4.4's rule inverts and the solar day view needs the
   civil date somewhere else.

2. **Should the Phase field appear in lunar mode?** §5.2 says yes. The counter-argument is
   D-025's, that the tithi says it more precisely. Both cannot be right, because the tithi is
   already in the Day pane in both modes.

3. **Should the `Daylight` row exist at all?** It reverses D-025 (§6.2). Either D-025 is amended
   to say why, or the row goes and the muhurtas are read against Surya's rise on Surya's panel.

4. **Should the solar grid pay for tithi frames?** §5.5 says no, on two reasons. The cost is
   about 42 tithi searches a month against a warm-cache budget of 5ms
   (`crates/almanac/tests/calendar.rs:575-582`). If the answer is yes, the solar cell's corner
   gains a tithi and §4.4's rule becomes symmetric.

5. **Is the menu-bar tooltip a permanent exception?** §4.5. D-028's relevance argument is sound
   but is the only place in the app where a fact is shown or hidden by the mode.

6. **Should amanta and purnimanta ever differ on screen?** Today nothing tells them apart (§1):
   the header prints `Shravana VS 2083` in both, for month ranges about a fortnight apart, and a
   user who changes the setting sees the grid move with nothing saying why. Options: name the
   reckoning in the header, name it in the jump overlay's year row, or leave it to the settings
   pane.

7. **Should `DayOptions` be gated on the mode?** Today the three toggles
   (`SettingsView.tsx:565-592`) are mode-independent, so a solar user can switch on muhurtas. Under
   §4.3 that is right — they are facts about a day. It is recorded here only because it is the one
   remaining place the modes *could* diverge and deliberately do not.
