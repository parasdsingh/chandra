# Outstanding work

Every open thread in one place. Ordered by what blocks what, not by size.

Status: `in flight` · `blocked on you` · `ready` · `queued` · `done`

---

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

### 1.1 The wording audit's open items — queued

The second pass in `AUDIT.md` — a fact about one thing printed beside a heading
about another — closed ten and left five standing:

- ~~**W-03** A Moshier month draws 42 cells with no precision note.~~ Fixed: the
  grid carries it too, and both notes were reworded. They said "Moshier
  ephemeris", which is the name of a piece of arithmetic and tells a reader
  nothing; they now name the range and the magnitude, so the note reads as a
  fact rather than a warning.
- **W-04** Clock times carry no zone, and the zone need not be the machine's.
- **W-06** `Ephemeris unavailable.` is shown for failures with no ephemeris in
  them, and is the fallback for any unknown code.
- **W-07** An unknown elevation prints as `0 m`. `Resolved.elevation` is an
  `f64` and cannot tell "not known" from "measured".
- **W-08** `GrahaCell.retrograde` is read at local noon while the day the cell
  opens reads its state at sunrise. A station between the two puts a retrograde
  ring on a cell whose day says `Direct`. Fixing it changes `graha_month`'s
  signature — deliberately not started rather than half-done.

Five more are suspected and unverified; they are listed at the foot of
`AUDIT.md`.

| | |
|---|---|
| Status | queued |

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
- Rahu and Ketu wear the retrograde mark permanently, and that is accepted.
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
  its state.

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

## 7. Panchanga and jyotisha fields — queued

Always on: planetary war, drishti, exaltation/debilitation/own sign, and the
nakshatra lord — "in whose nakshatra it sits". Toggleable in settings: yogas,
karanas, muhurtas.

| Tier | Work |
|---|---|
| Free | Dignity is a lookup. The nakshatra lord is already computed and already on the payload |
| Cheap | Yoga (Sun + Moon ÷ 13°20′), karana (half a tithi), drishti, planetary war (two grahas inside 1°) |
| Real | Muhurtas — Rahu Kaal, Yamaganda, Gulika, Abhijit, Brahma Muhurta, Durmuhurtam. All divide day and night into parts, so all need **sunset**, which is computed nowhere yet |

This roughly doubles what a day holds, on the surface D-025 has just made
readable. It probably has to split — a *day* pane and a *position* pane. That
decision comes before the design.

| | |
|---|---|
| Status | queued, needs the split decided first |

---

## 8. Ingress labels — queued

Agreed: three-letter rashi abbreviations on the ingress day, replacing the glyph
on that cell only. Western forms, because the Sanskrit names cannot be
abbreviated — *Vrishabha* and *Vrishchika* are identical for six letters. Since
both names are already on every rashi this is a language setting, not a data
change, and it should apply app-wide.

Nakshatras take two-part forms — `P.Ash`, `U.Bha` — because three of them begin
*Purva* and three begin *Uttara*. One or the other is marked, never both;
settings chooses.

This is the one thing that would put an ingress back on the grid after D-024
took the marker off it, and it puts it there as a word rather than as a shape.

| | |
|---|---|
| Status | queued |

---

## 9. Not started

- GitHub remote. Deferred by D-018 until a production release; still local-only.
- CI. Described in the Makefile and the docs; no `.github/workflows` exists.
- Release DMG on tag.
