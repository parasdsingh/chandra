# The legend — explaining every mark the app draws

Status: **proposed.** Design only. Nothing here is built.

Answers: "a way to explain all graphical elements to the user. many of the aspects like dotted
circles for retrograde, orange tint for combust, even the S2, K12, etc. terminology should be
explained to the user assuming they know basic lunar calendar concepts."

Works under [D-011](../DECISIONS.md#d-011), [D-017](../DECISIONS.md#d-017),
[D-019](../DECISIONS.md#d-019), [D-020](../DECISIONS.md#d-020),
[D-021](../DECISIONS.md#d-021), [D-023](../DECISIONS.md#d-023),
[D-024](../DECISIONS.md#d-024), [D-025](../DECISIONS.md#d-025),
[D-026](../DECISIONS.md#d-026).
Expressed in the vocabulary of [DESIGN.md](../DESIGN.md) §5.4, §5.5, §6.3, §9.5, §11.3, §12.

**The reader is assumed to know lunar calendar basics.** They know what a tithi, a paksha, a
nakshatra, a rashi and a graha are. They do not know what *this app* does with them. Nothing
below teaches the calendar; everything below decodes a mark.

---

## 1. Inventory

Every element on screen that carries meaning, read from the code rather than from the design
docs. **65 items.** The last column is what a *user* can read today, not what a developer can
find in `docs/`.

Legend for that column:

- **words** — the same fact is stated in visible text somewhere in the panel.
- **spoken** — carried by an `aria-label` only. Invisible to a sighted reader.
- **docs** — documented in `DESIGN.md` or `DECISIONS.md`. Not shipped to the user.
- **nothing** — no carrier of any kind on any surface.

### 1.1 The day cell — `DayCell.tsx`, `panel.css` §day-cell

| # | Element | Where | What it means | Explained? |
|---|---|---|---|---|
| 1 | Day numeral | cell, line 1, solar mode | the Gregorian day | conventional |
| 2 | `S1`–`S14`, `K1`–`K14` | cell, line 1, lunar mode | the tithi the day is named for; `S` shukla, `K` krishna | spoken (`Shukla Ashtami`) |
| 3 | `P`, `A` | cell, line 1, lunar mode | Purnima, Amavasya. No number, no prefix | spoken |
| 4 | Paksha letter set in `--text-secondary` | the `S`/`K` prefix | it is a prefix, not a word; the number reads first | docs |
| 5 | Kshaya dot, Ø3 `--marker`, top-left | cell corner | a tithi began and ended between two sunrises; the numbers jump here | words (`· kshaya` in the day view) + spoken |
| 6 | Corner number, bottom-right, 9px | cell, lunar mode only | the Gregorian day. Never carries its month | nothing |
| 7 | Phase disc, Ø14 | cell, line 2, moon calendar | the Moon's lit fraction | words (`Phase`, solar) + spoken |
| 8 | New moon — hairline ring, dark fill | disc at k < 0.02 | a new moon, not an empty cell | spoken |
| 9 | Full moon — solid disc, ring omitted | disc at k > 0.98 | a full moon | spoken |
| 10 | Lit side, and its mirroring | disc | right while waxing; flipped when the resolved latitude is south | spoken (phase name only, not the side) |
| 11 | Graha symbol, 14px | cell, line 2, graha calendar | which graha. Nothing else | header names it |
| 12 | `Ari`, `P.Ash` in the symbol's place | cell, line 2, on an ingress day | the division entered that day. **Western abbreviations for Sanskrit rashis** | spoken (`enters Ari`), words in `Events` (`Enters Mesha rashi`) |
| 13 | Dotted half-ring, right half | cell, on the glyph | retrograde motion **begins** this day | spoken (`retrograde`) |
| 14 | Dotted whole ring | cell, on the glyph | retrograde, and it was yesterday and is tomorrow | spoken |
| 15 | Dotted half-ring, left half | cell, on the glyph | retrograde motion **ends** this day | spoken |
| 16 | Warm radial wash from the cell's foot, `--glare` | behind the cell's content | the graha is inside the Sun's rays | words (`Combust`) + spoken (`combust at noon`) |
| 17 | 1px `--accent` ring, inset 3 | cell | today | words (`TODAY`) + `aria-current` |
| 18 | Numeral in `--accent` | cell | today, said a second time | as above |
| 19 | Phase disc outline tinted `--accent` | moon cell only, and not at full | today, said a third time | as above |
| 20 | `--surface-selected` fill, numeral to 600 | cell | the day whose detail is open | `aria-selected` |
| 21 | `--surface-hover` fill | cell | pointer is over it | conventional |
| 22 | 2px `--focus` ring | cell, `:focus-visible` | keyboard focus | roving `tabindex` |
| 23 | Numeral to `--text-secondary`; glyph, corner date, retro ring and wash to 45% | cell | belongs to the month either side. Still clickable | spoken (`outside this month`) |

### 1.2 The header — `Header.tsx`

| # | Element | Where | What it means | Explained? |
|---|---|---|---|---|
| 24 | Subject glyph, 24 × 16 | header left | live moon disc on Chandra's panel; the graha's symbol elsewhere | label beside it |
| 25 | `Chandra · August 2026` | header centre | subject, then the month in view | self-evident |
| 26 | `Adhika` in `--text-secondary` before the name | header, lunar mode | intercalary month. A qualifier on the name, not a name | nothing |
| 27 | `VS` in `Shravana VS 2083` | header, lunar mode | Vikram Samvat | nothing |
| 28 | The month label is a button | header centre | tapping it opens the month and year picker | `aria-expanded`, hover fill |
| 29 | Truncation ladder | header centre | the year is dropped first, the subject's name last. No ellipsis | docs |
| 30 | `⚙` gear, `‹` chevron | header right / left | settings; back | `aria-label` |

### 1.3 The month and year picker — `MonthJump.tsx`

| # | Element | Where | What it means | Explained? |
|---|---|---|---|---|
| 31 | `Adhika` set apart again | a month tile | intercalary, as #26 | nothing |
| 32 | `.jump__month.is-current` | a month tile | the month the strip is showing | conventional |

### 1.4 The day view — `DayDetail.tsx`, `panel.css` §detail

| # | Element | Where | What it means | Explained? |
|---|---|---|---|---|
| 33 | `Day` / `Position` tabs | above the field stack | `Day` is the civil day, identical on all nine panels; `Position` is this subject's | `role="tab"` only |
| 34 | `TODAY` in `--accent` | date line | today | self-evident |
| 35 | Vara name on the date line | `… · Guruvara` | the day's own name | conventional |
| 36 | Field shape: micro-caps key, value left, number right | every field | one statement per field | — |
| 37 | The right-hand number, unlabelled | every span field | **when the value on the left gives way.** `14:02`, `14:02, 21 Aug`, or `21 Aug` alone at more than a day's distance | spoken only (`until`) |
| 38 | `then …` line | under a span value | what follows it in time | words |
| 39 | ` · kshaya` on a tithi | `Tithi` value | no day is named after this tithi | words, but the link to the grid's dot is not made |
| 40 | ` · fixed` on a karana | `Karana` value | one of the four karanas that occur once a lunar month | nothing |
| 41 | ` · Pada N` on a nakshatra | `Nakshatra` value | the quarter of the nakshatra | conventional |
| 42 | Warm glare at the panel's foot | fixed behind the fields | combust, drawn as the cell draws it | words (`Combust`) |
| 43 | Dotted rail down the right gutter, turning back on itself | fixed behind the fields | retrograde | words (`Motion`) |
| 44 | `℞` chip, bordered, after `Retrograde` | `Motion` value | the printed-ephemeris mark for retrograde | the word beside it |
| 45 | `inside the 12° orb` | `Combust` value | the orb is per graha and narrows while retrograde: Chandra 12, Guru 11, Budha 14/12, Shukra 10/8, Shani 15, Mangala 17 | the number is shown; the table is not |
| 46 | `Daylight` field | Day pane | the day's own sunrise and sunset — the instants the divisions are read at | nothing says why it matters |
| 47 | `Moonrise` / `Sunrise` / `Shani rise`; `does not rise`, `does not set` | Position pane | the subject's own rise and set. **Absent for Rahu and Ketu** | the row's own label; the absence is unexplained |
| 48 | Muhurta rows | Day pane | six windows. Rahu Kaal, Yamaganda, Gulika, Durmuhurtam are to avoid; Abhijit and Brahma Muhurta are sought | **spoken only** (`avoid` / `auspicious`) — no visual carrier at all |
| 49 | `Nakshatra lord` | Position pane | the graha owning the nakshatra the subject stands in | words |
| 50 | `Dignity` — `Exalted` / `Debilitated` / `Own sign` | Position pane | the row is absent where the graha is in none of the three | the absence is unexplained |
| 51 | `Drishti` — `aspects …` / `aspected by …` | Position pane | the same relation from both ends | words |
| 52 | `Planetary war`, with `1.23°` in the **time** column | Position pane | two grahas within a degree. No winner is named | the degree sign; the column is otherwise times |
| 53 | `Enters Simha rashi`, `Retrograde station`, `Direct station`, `Enters combustion`, `Leaves combustion` | `Events` | what happened, and at what minute. **Ingresses and stations are drawn nowhere in the grid** | words; the "drawn nowhere" is not stated |
| 54 | `Outside 1800–2399. Times here are approximate, by about a second.` | foot of the day view, and on the grid | the analytic fallback produced these numbers | words |
| 55 | `Times are Asia/Kolkata, not this Mac's clock.` | foot of the day view | printed only when the two zones differ | words |
| 56 | `Phase` field | Position pane, solar mode only | in a lunar month the tithi says it more precisely, so it is not repeated | the absence is unexplained |
| 57 | Error block — headline plus cause | in place of the fields | a computation failed | words |

### 1.5 The menu bar — `crates/glyph`, `src-tauri/src/tray.rs`

| # | Element | Where | What it means | Explained? |
|---|---|---|---|---|
| 58 | Moon disc, template image | tray | the Moon **now**, redrawn on the hour | tooltip |
| 59 | Graha template glyph, 18pt cap height | tray | which graha | tooltip |
| 60 | `℞` in the lower right, glyph shrunk to 16.5pt to free it | tray | retrograde. **The only state the menu bar carries** | nothing |
| 61 | Tooltip on hover | tray | Moon: phase name, or the tithi in a lunar month. Graha: its position | itself |
| 62 | Coloured icons | tray, off by default | template icons follow the menu bar appearance | settings hint |

### 1.6 Settings — `SettingsView.tsx`

| # | Element | Where | What it means | Explained? |
|---|---|---|---|---|
| 63 | Drawn switch; Chandra's row on and disabled | Menu bar section | the permanent moon item **is** Chandra's item | words (`The moon is always shown.`) |
| 64 | `Rashi — Ari, Tau, Gem` / `Nakshatra — Ashw, P.Ash` | Calendar section | a sample of the abbreviations the cell will print | partially — three of thirty-nine |
| 65 | Root row values (`Amanta`, `Moon + 2`, `Default`) | Settings root | what each section is set to now | self-evident |

---

## 2. What a reader who knows the calendar still cannot decode

Of the 65, **21 have nothing a sighted user can read** — no visible words, no tooltip, no
settings hint. They are the working set.

| # | Element | Why the reader is stuck |
|---|---|---|
| 4 | The dimmed paksha letter | reads as two labels, `S` and `8`, rather than one |
| 6 | The corner day number | a bare `1` beside `S12`. Not obviously a Gregorian date, and never carries a month |
| 10 | The mirrored disc | a southern-hemisphere user sees the opposite of every printed panchang and is told nothing |
| 12 | `Ari` for Mesha | **the cell prints a western abbreviation and the day view prints the Sanskrit name.** `Ari` and `Mesha` are the same sign and nothing on screen says so |
| 13–15 | The three retrograde ring phases | a half-ring is not a shape anyone has seen before. Which half means *begins* is unguessable |
| 16 | The combustion wash | by design it encodes nothing (D-023). That is exactly why it needs a sentence somewhere |
| 19 | Today's tinted disc outline | invisible as a mark; reads as a rendering artefact |
| 26 | `Adhika` set apart | a reader who knows adhika masa still does not know why the word is a different colour |
| 27 | `VS` | recoverable from the number, but only by arithmetic |
| 29 | The truncation ladder | the subject's name disappears from the header and the panel looks like a different panel |
| 33 | The `Day` / `Position` split | "where is the nakshatra?" has a rule behind it — Position carries the *subject's* nakshatra, which is the Moon's only on Chandra's panel — and the tabs do not say it |
| 37 | The unlabelled right-hand time | reads as "at 14:02", not "until 14:02". VoiceOver is told `until`; the screen is not |
| 40 | `· fixed` | `fixed` is this app's word. The four fixed karanas are a known idea under other names |
| 45 | The orb | `inside the 12° orb` states a number without saying it is per graha, or that it narrows while retrograde |
| 47 | No rise row for Rahu and Ketu | an absence reads as a bug. The app declines to model it; it is not that they never rise |
| 48 | Muhurta auspiciousness | **the worst case in the app.** VoiceOver says `avoid`; the screen says nothing. `muhurta.rs` states the intent outright: "a reader should not have to know which by name" |
| 50 | A missing `Dignity` row | absence means neutral, and absence is not a statement |
| 52 | Degrees in the time column | every other number in that column is a clock time |
| 53 | Ingresses and stations drawn nowhere | a reader who noticed the old marker row, or who expects one, hunts for a mark that does not exist |
| 56 | No `Phase` row in a lunar month | as #50 |
| 60 | The tray `℞` | the notation is standard; that it is the *only* state the menu bar carries is not |

Three of these are absences, not marks — #47, #50, #56 — and DESIGN §11.3 already records two more
(sign ingress, vriddhi) that were drawn once and are now drawn never. **A legend that lists only
what is drawn leaves those five unanswered.** The legend must be able to say "nothing is drawn for
this, and here is where the words are." That is a design requirement, not a footnote.

Items decodable without help, and deliberately left out of the legend's working set: `℞` itself
(D-020 argues it needs no key, and printed ephemerides agree), the today ring and numeral, the
selection fill, the hover fill, the focus ring, the outside-month dimming, the vara name, `Pada`,
`Dignity`'s three words, `Drishti`. They still appear in the legend — a reader scanning for one
mark should not have to decide in advance whether it qualifies — but they are one short line each.

---

## 3. The surface

### 3.1 The constraints that decide it

- The panel is **320 × 332** at scale 1, with a **264px region** the three views share. There is
  no second window, and DESIGN §12 rejects one outright: "a second surface is a second thing to
  learn."
- **DESIGN §9.5 forbids onboarding**: "No onboarding, no welcome panel, no tour, no permission
  prompt." First run is the menu bar item and a working panel.
- **DESIGN §11.3 forbids tooltips as the carrier**: the redundant carrier for today and for
  Rahu/Ketu is required to be "in the same surface, not in a tooltip."
- **DESIGN §10.3**: the day view "contains nothing focusable at all — no buttons, no links, no
  controls." Adding an info button to it is a spec change, not an addition.
- The panel scales 0.8–1.4 as one transform, so nothing may be sized in a way that only works at
  one step.

### 3.2 Options

| Option | Pixels | Code | Can go stale? | Reaches someone not looking? |
|---|---|---|---|---|
| **A. A `Marks` section in settings** | one 36px root row; the section reuses the 264px region | one `SettingsSection` variant, one component, one CSS block | yes — mitigated in §5 | only via the settings root list |
| **B. First-run card or tour** | a full-region overlay | a persisted "seen" flag → settings schema bump; dismissal logic; a second layout | yes | yes, once — at the moment the user has least context |
| **C. Long-press / right-click a cell** | a popover the panel has nowhere to put | context-menu plumbing, a popover layer, a keyboard equivalent, per-mark hit-testing inside a 40px cell | no — it would read live state | no |
| **D. A static page rendered from the tokens** | none in the panel | a build step, a second output, a way to open a browser | no, if generated | no |
| **E. `title` tooltips on cells and fields** | none | one attribute per element | yes | no |

### 3.3 What each option is rejected for

**B — first run.** Rejected on D-017 and DESIGN §9.5, which are explicit. It also fails on its
own merits: a tour explains marks before the user has seen one, and the state it must remember is
a new settings field with a migration behind it. A tour that has to be dismissed is a cost paid by
every user on every install to serve the one reading in ten who wants a key.

**C — contextual.** Rejected on discoverability and on hit-testing. A right-click has no visible
affordance on macOS and no keyboard equivalent, so it reaches nobody who is not already looking —
the same failure as A with far more machinery. Worse, the marks it would explain overlap in one
40px cell: the combustion wash is full-bleed, the retrograde ring sits on the glyph, and the today
ring is inset 3. A single hit point cannot say which of the three the user meant, so the popover
would list all of them — which is a legend, rendered in the worst place for one.

**D — a static page.** Rejected as the *surface*. A page the panel cannot open is not part of the
app, and opening a browser from a menu bar popover is a second surface by any reading of §12. It
is, however, the right *fallback* for a reader who wants to read the whole thing at once, and §7
keeps it as a non-goal rather than a rejection.

**E — tooltips.** Rejected three times over. DESIGN §11.3 names tooltips as insufficient. They
have no keyboard path, so the one population most likely to need a key — VoiceOver users — is the
one that cannot reach them, and that population is already served better by the spoken labels. And
a `title` on a cell would sit beside an `aria-label` saying something else about the same element,
which is two carriers that can drift from each other in one tag.

### 3.4 Recommended: A, a `Marks` section in settings

**Why.**

- It costs one row in a list the user already reads. Settings is not an advanced surface in this
  app — location, month system and the menu bar toggles are all day-one settings — so the root
  list is a place nearly every user visits, and a `Marks` row sits in it permanently rather than
  once.
- It is the only option that can render **the real mark at real size** next to its sentence. That
  is what makes it correct-by-construction (§5) and what makes the three retrograde ring phases
  explicable at all: they are a shape, and a shape needs to be shown, not described.
- It adds no view, no overlay, no window, no timer and no persisted state. `SettingsSection` gains
  one variant; the header, the back chevron and the scroll behaviour are already built.
- It can say "nothing is drawn for this" — the five absences from §2 — which no contextual or
  hover surface can, because there is nothing there to hover.

**What it does not do.** It does not reach a user who never opens settings. That is a real cost
and there is no way to pay it that survives §9.5. The honest framing: the app has decided that a
tour is worse than an unreached reader, and this proposal does not reopen that.

**The one addition worth arguing for** is a pointer, not a second surface: the settings root row
already states what each section is set to. `Marks` has no setting, so its value slot is free.
Putting the count there — `65 marks` — is noise. Leaving it empty follows `About`, which already
does. Recommend empty.

---

## 4. Draft copy

The actual words. Each entry is a **name** (Body 13/400 `--text-primary`) and **one or two lines**
(Caption 11/400 `--text-secondary`), beside a specimen drawn at the size the app draws it.

Order follows where a reader meets the mark: the grid, then a day, then the menu bar.

### 4.1 In the calendar

**The tithi label**
> `S` is shukla paksha, `K` is krishna. `S8` is the eighth tithi of the waxing half.
> `P` is Purnima and `A` is Amavasya — those two take no number.
> The letter is set dimmer than the number so the number reads first. It is one label, not two.

**The number in the corner**
> The Gregorian day. It appears only in a lunar month, where the tithi has taken the main line.
> It never carries its month; the day view has the full date.

**The dot beside the number**
> A tithi began and ended between two sunrises, so no day is named after it.
> The numbers jump at this cell. That is the calendar, not a fault.
> The day view spells it out: the tithi's row reads `· kshaya`.

**The moon disc**
> How much of the Moon is lit, and which side.
> Lit on the right while waxing. Mirrored when your location is south of the equator.
> A hairline ring with a dark centre is a new moon. A solid disc is a full moon.

**The graha symbol**
> Which graha this calendar is about, and nothing else.
> Every state it can be in is drawn around it rather than on it, so the symbol never changes.

**A word in the symbol's place**
> The division the graha entered that day. `Ari` is Mesha, `P.Ash` is Purva Ashadha.
> The rashis are abbreviated to their western names: three Sanskrit names cannot be told apart in
> three letters. The day view names the sign in full.
> Turn these labels off in Settings › Calendar.

**The dotted ring**
> The graha is retrograde.
> The ring opens on the day the motion turns, is whole while it lasts, and closes on the day it
> turns back — so one stretch reads as one shape across several cells.
> A stretch that runs past the edge of the six weeks on screen is drawn whole there.

**The warm glow**
> The graha is inside the Sun's rays — combust.
> It is a picture of the glare, not a colour code. The day view says it in words.
> Judged at noon, so a mark on a cell and the day behind it always agree.

**Gold**
> Gold means today, and nothing else in this app is gold.
> Today is marked three times: the ring, the numeral, and — on the Moon's calendar — the disc's
> own outline.

**A filled cell** — The day you have open.

**Faded days** — They belong to the month either side. Click one to go there.

**Adhika**
> An intercalary month. It qualifies the name rather than being one, so it is set apart from it.
> `Adhika Shravana` is a second Shravana, not a thirteenth month with its own name.

**VS**
> Vikram Samvat, the era the lunar year is counted in.
> It runs 57 ahead of the Gregorian year for most of its length, and 56 after 1 January.

**The month name is a button** — Tap it to jump to another month or year.

### 4.2 In a day

**Day and Position**
> `Day` is the civil day: tithi, yoga, karana, daylight, muhurtas. It is the same on all nine
> calendars.
> `Position` is where **this** subject stands. Its nakshatra is the subject's — on Mangala's
> calendar it is Mangala's, not the Moon's.

**The time on the right**
> When the value on the left gives way.
> A clock time is today or the day either side. A bare date is further off than that, where the
> minute is not the fact you want.

**then …** — What follows it, so you need not open tomorrow.

**· kshaya** — No day is named after this tithi. This is the jump you see in the grid.

**· fixed**
> One of the four karanas that come round once in a lunar month, always in the same place.
> The other seven repeat eight times each.

**· Pada** — Which quarter of the nakshatra the subject is in.

**The glare at the foot** — The same glare the cell carries. The `Combust` field says it in words.

**The dotted rail down the margin** — Retrograde. The `Motion` field says it in words.

**℞** — How every printed ephemeris writes retrograde. It also appears in the menu bar.

**inside the N° orb**
> How close to the Sun this graha has to be to count as combust. The orb is per graha, and
> narrows while the graha is retrograde.
> Chandra 12° · Guru 11° · Budha 14°, 12° retrograde · Shukra 10°, 8° retrograde · Shani 15° ·
> Mangala 17°.
> Surya and the nodes have no orb, so they never carry the mark.

**Daylight** — The day's own sunrise and sunset. The tithi, nakshatra and rashi are all read at
sunrise.

**Muhurtas**
> Windows inside the day.
> Avoid: Rahu Kaal, Yamaganda, Gulika, Durmuhurtam.
> Sought: Abhijit, Brahma Muhurta.
> Abhijit does not occur on a Wednesday — its slot is already Wednesday's Durmuhurtam.

**Rise and set** — Named for the subject: `Sunrise` on Surya's calendar, `Moonrise` on Chandra's,
`Shani rise` on Shani's. Rahu and Ketu have no row: they are points on the ecliptic, and the app
does not model them crossing the horizon.

**Nakshatra lord** — The graha that owns the nakshatra the subject is standing in.

**Dignity** — `Exalted`, `Debilitated` or `Own sign`. There is no row where the graha is in none of
the three.

**Drishti** — Which grahas this one aspects, and which aspect it. The same relation from both ends.

**Planetary war** — Two grahas within a degree of each other. The number on the right is their
separation, not a time. No winner is named: the authorities decide it differently.

**Events** — What happened that day, with the minute. Ingresses and the two stations are **not**
drawn in the grid at all — this is where they are.

**Outside 1800–2399** — Positions here come from an approximation rather than the ephemeris files.
Times are out by about a second.

**A note about the zone** — Every time in the app is the observer's, because a sunrise is a fact
about a place. The note appears only when that is not this Mac's zone.

### 4.3 In the menu bar

**The moon** — The Moon as it is now, redrawn every hour.

**℞ under a graha** — Retrograde, and the only state the menu bar carries. Combustion, ingresses
and stations are in the panel. A menu bar icon is 22 points, and a second mark in a second corner
would stop the row being scannable.

**Point at an item** — The Moon says its phase, or the tithi in a lunar month. A graha says where
it is. The tooltip is current to the second; the icon to the hour.

### 4.4 What is drawn nowhere

A short closing group. It exists so a reader stops hunting.

> **Not drawn, by design.** A sign or nakshatra ingress, a retrograde or direct station, a
> repeated tithi (vriddhi), and the ephemeris a value came from. All four are named in words in
> the day view instead. They were marks once; a row of five-pixel discs under a numeral was more
> to decode than to read.

---

## 5. How it stays true

This is the part with a failure already on record. `--annotate` was deleted while `panel.css`
still read it, and the grep that checked survived its own shell quoting and reported nothing.
`AUDIT.md`'s method note is the rule: **a fix whose test would still pass against the old code is
not finished.**

Two facts constrain what is possible here:

- **The front end has no test harness at all.** `AUDIT.md` says so; `package.json` confirms it —
  `tsc --noEmit` is the whole of it. A legend guard that needs Vitest is a proposal to adopt a
  test framework, and that is a separate decision.
- **`tools/check-tokens.sh` is the precedent.** A POSIX shell script, wired into `make lint` and
  therefore into CI, guarding exactly this class of silent drift. The legend should use the same
  mechanism rather than invent one.

Three layers, strongest first.

### 5.1 The specimen is the component, not a picture of it

Every mark in the legend is drawn by the code that draws it in the app.

| Specimen | Drawn by |
|---|---|
| tithi label, paksha prefix, kshaya dot, corner date, today ring, selection fill, combustion wash, outside-month dimming | a real `DayCell` in the state being described |
| the three retrograde ring phases | `RetroRing`, exported from `DayCell.tsx` |
| the moon disc at new, crescent, full | `PhaseGlyph` |
| a graha symbol | `GrahaGlyph`, from the same `GrahaInfo` the panel is served |
| the `℞` chip | the `.chip` class |
| the glare and the rail | `.detail__glare`, `.detail__rail` |

**A legend that redraws its own approximation of a mark is the drift bug, shipped deliberately.**
There must be exactly one drawing of each mark in the codebase and the legend must be looking at
it. `RetroRing` is currently private; exporting it is the only source change layer 1 needs.

Consequences to specify, not to discover later:

- Legend specimens are synthetic days. They must carry `aria-hidden="true"`, must sit outside any
  `role="grid"`, and must not be clickable. The row's accessible name is its heading and sentence,
  which are real text.
- `DayCell` takes an `onSelect` today. The legend passes a no-op and the cell must not be given a
  `tabindex` — it is not in a grid, so there is no roving index to join.
- The specimens are laid out in a 40px column so a real `DayCell` sits in one without scaling. The
  panel's own transform handles 0.8–1.4; nothing in the legend re-scales.

### 5.2 A build check that the two lists agree — `tools/check-legend.sh`

Layer 1 keeps the *drawing* honest. It cannot notice a mark that is added and never documented,
which is the failure that actually happens.

The legend module declares each entry with a stable id and the **class or component that draws
it**:

```
{ id: "retro-ring",    draws: "RetroRing" }
{ id: "combust-wash",  draws: ".day-cell__combust" }
{ id: "kshaya-dot",    draws: ".day-cell__kshaya" }
```

The script runs both directions:

1. **Documented but not drawn.** For each `draws`, grep the source. A class that no stylesheet
   declares, or a component no module exports, fails: *"the legend documents `combust-wash` and
   nothing draws it."*
2. **Drawn but not documented.** Enumerate every `.day-cell__*`, `.detail__*` and `.chip` selector
   declared in `panel.css`. Any not named by an entry fails: *"nothing in the legend explains
   `.day-cell__vriddhi`."*

Direction 2 needs an exemption list, because some classes are layout and carry no meaning —
`.day-cell__content`, `.day-cell__numeral`, `.detail__scroll`, `.detail__panes`. **The list is
declared in the script with a one-line reason each**, so adding to it is a decision someone makes
in a diff rather than a silence. That is the whole design: the exemption list is the place the
argument happens.

Wire it into `make lint` beside `check-tokens.sh`.

**Definition of done for the guard itself,** per the method note — it must fail against the old
code:

- Delete `.day-cell__kshaya` from `panel.css`: the script must fail on direction 1.
- Add a `.day-cell__vriddhi` rule: the script must fail on direction 2.
- Rename `RetroRing`: the script must fail on direction 1.

A guard that passes all three of those is finished. One that does not is the `--annotate` grep
again.

### 5.3 One prose source, quoted rather than copied

DESIGN §11.3's table stays the specification — it is about *carriers*, and it will keep the two
rows that have no visual carrier at all. The legend is the product.

They are kept from diverging by adding one line under §11.3 pointing at the legend module as the
user-facing wording, and by direction 2 above: a mark added to the app fails the build until it
has a legend entry, and writing that entry is when §11.3 gets updated too.

**Rejected: generating §11.3 from the legend, or the legend from §11.3.** They are different
documents with different columns — §11.3 pairs a visual carrier with a textual one for an audit;
the legend pairs a specimen with an explanation for a reader. Generating either from the other
would force one of them into the wrong shape, and a generator between a markdown table and a TSX
module is more machinery than the thing it guards.

---

## 6. Accessibility

The legend must not touch the spoken contract. It is a reference page, not an annotation layer.

- **No spoken label changes anywhere.** Every mark in §1 already has its carrier per DESIGN §11.3,
  and the legend adds no eleventh hour exception. It reads the app; it does not modify it.
- **Specimens are `aria-hidden`.** A screen reader hears the heading and the sentence — real text
  it can navigate — and never hears a synthetic day announced as a date.
- **No new tab stops.** The legend contains no controls. Focus order (DESIGN §10.3) is unchanged:
  header, then the settings controls, and the legend section has none. The back chevron and `Esc`
  work as in every other section.
- **The legend is genuinely useful to a VoiceOver user**, and not redundant with the spoken labels.
  The spoken label of a cell says `retrograde`. It does not say that the ring is a bracket over a
  run, that the orb is per graha, or that Rahu and Ketu have no rise row on purpose. The legend
  carries the *rules*; the labels carry the *values*.
- **The wording is the same for both.** Where the legend explains a mark, the sentence it uses
  should not contradict what the spoken label says about it. Two cases need care:
  - Combustion. The spoken label is `combust at noon`. §4.1 says **noon** and not `midday`, so
    the two match word for word; a reader who hears one and reads the other must not have to
    decide whether they are the same instant.
  - `until` — spoken, never printed. The legend is the only place a sighted reader is told the
    right-hand column means `until`. That is the whole reason item #37 is in the working set.
- **Contrast.** Every text style in the legend is Body or Caption at `--text-primary` /
  `--text-secondary`, which clear 4.5:1 (DESIGN §11.1). Nothing in it uses `--text-tertiary`
  (D-023). `--glare` appears only as a specimen, which is a picture of a mark and is exempt on the
  same grounds the mark itself is.
- **`prefers-contrast: more`** raises `--disc-ring` and `--text-secondary` as it does everywhere,
  because the specimens are the real components and inherit the same tokens.

### 6.1 One gap this proposal exposes but does not close

Item #48. Muhurta auspiciousness is spoken (`avoid` / `auspicious`) and drawn nowhere. That
inverts the app's own contract: DESIGN §11.3's rule is that a **visual** carrier must have a
textual one, and here the textual carrier exists with no visual peer, so a sighted reader is told
less than a VoiceOver user.

A legend can list which four to avoid, and §4.2 does. That is a workaround, not a fix — a reader
would have to memorise the list rather than read the day. The fix is a mark on the muhurta row,
and that is a design change to the day view, outside this document. Raised as an open question.

---

## 7. Where it sits

### 7.1 In settings

`SettingsSection` gains `"marks"`. `SECTION_TITLES.marks = "Marks"`.

Root list order — the two rows above it are the ones that decide which marks the user will see at
all: the month system decides the tithi label and the corner date, and the panchanga limbs decide
which day-view fields exist.

```
  Calendar        Amanta
  Panchanga       All
  Marks                          ← new
  Location        Bengaluru
  Astrology       Lahiri
  Menu bar        Moon + 2
  Size            Default
  About
```

- **Value slot: empty.** `About` already ships an empty value, so the precedent exists. A count
  (`65 marks`) is a number nobody wants; a word (`Reference`) restates the title.
- **Cost, stated precisely.** The root list is already 252px of rows plus 8px padding plus a ~26px
  footnote in a 264px region — it scrolls by about 22px today. An eighth row makes that 58px.
  Both scroll; the change is that `About` moves further below the fold. Acceptable, and the
  alternative — a `Marks` row promoted above `Calendar` — puts a reference above the settings
  people actually came to change.

### 7.2 Inside the section

One scrolling list in the 264px region, four group headings, 36 rows — the entries in §4.

```
  ┌──────────────────────────────────────┐
  │  ‹    Marks                      ⚙   │   the panel's own header
  ├──────────────────────────────────────┤
  │  IN THE CALENDAR                     │   micro-caps, --text-secondary
  │  ┌────┐                              │
  │  │ S8 │  The tithi label             │   40px specimen · Body 13
  │  │ ◗  │  S is shukla paksha, K is …  │   Caption 11, --text-secondary
  │  └────┘                              │
  │  ┌────┐                              │
  │  │ 12 │  The dotted ring             │
  │  │ ⁘♂⁘│  The graha is retrograde …   │
  │  └────┘                              │
  │  …                                   │
  │  IN A DAY                            │
  │  …                                   │
  │  IN THE MENU BAR                     │
  │  …                                   │
  │  DRAWN NOWHERE                       │
  │  …                                   │
  └──────────────────────────────────────┘
```

- Row: a 40px specimen column, 8px gap, a 240px text column. Roughly 56px per row.
- Total ≈ 2000px, about eight screens of scroll. That is long for a settings pane and normal for
  a reference. `.settings` already scrolls with the scrollbar hidden.
- **Rejected: a second drill level** (`Marks › In the calendar`). It would put the three groups
  behind three more taps and force `back()` in `Panel.tsx` to learn a parent for one section —
  changing the navigation model of the whole settings tree to shorten one list. A reference is
  scrolled, not navigated.
- Rows that explain an absence (§4.4) carry no specimen; the text column runs the full width. That
  is the visual difference between "this is what the mark looks like" and "there is no mark."

### 7.3 New files

| File | What |
|---|---|
| `src/components/LegendView.tsx` | the entries, the copy, the specimens |
| `src/styles/settings.css` | one block: `.legend__group`, `.legend__row`, `.legend__specimen`, `.legend__name`, `.legend__note` |
| `tools/check-legend.sh` | §5.2, wired into `make lint` |

Changed: `SettingsView.tsx` (one variant, one title, one root row), `DayCell.tsx` (export
`RetroRing`), `DESIGN.md` §11.3 and §12.1 (one line each), `docs/DECISIONS.md` (a new decision
recording the surface choice and the §9.5 tension).

Not changed: panel size, region height, focus order, any spoken label, any existing mark.

---

## 8. Open questions

1. **Does a `Marks` section in settings satisfy the request, given that it does not reach a user
   who never opens settings?** DESIGN §9.5 forbids a first-run tour, so the alternative is
   reopening that decision. Is the unreached reader acceptable, or is §9.5 up for revision?

2. **Item #48 — should the day view mark which muhurtas are to be avoided?** Today it is spoken
   and not drawn, which is the inverse of the app's own accessibility contract, and `muhurta.rs`
   says a reader should not have to know which by name. If yes, that is a separate design and this
   document's §4.2 becomes a stopgap. If no, the legend's list is the answer permanently.

3. **Item #12 — the cell prints `Ari` where the day view prints `Mesha`.** The legend can say they
   are the same sign. Should it instead be fixed at the source — a three-letter Sanskrit
   abbreviation scheme, accepting that some pairs collide — or is the western short form a
   deliberate choice worth keeping and explaining?

4. **How much reasoning belongs in the copy?** §4 states what a mark means and, in three places,
   why it is drawn that way — "it is a picture of the glare, not a colour code." That reasoning is
   what stops a reader looking for a code that is not there. It is also longer. Cut to meaning
   alone, or keep the why?

5. **Should the legend name the four fixed karanas, the six muhurtas, the six orbs?** §4.2 lists
   all three. Each is a table inside a caption line, and a reference that lists everything becomes
   a document rather than a key. Keep, or point at the day view and let the values speak?

6. **`tools/check-legend.sh` direction 2 needs an exemption list for layout-only classes.** Is a
   commented list in the script the right home for that argument, or should the exemption live in
   `panel.css` as a marker comment beside each class, so the two are never read apart?

7. **Does this warrant a decision entry?** The surface choice contends with DESIGN §9.5 and with
   §12's "a second surface is a second thing to learn". Both are resolved here in prose. If the
   recommendation is accepted, it should probably be D-029 rather than a paragraph in a design
   document.
