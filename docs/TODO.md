# Outstanding work

Every open thread in one place. Ordered by what blocks what, not by size.

Status: `in flight` · `blocked on you` · `ready` · `queued` · `done`

---

## 1. Correctness — in flight

**The 55 audit findings.** `docs/AUDIT.md` carries the detail; each finding is
updated in place as it is fixed. Two independent audits ran the same
instructions; they agreed on ten findings, each caught several the other missed,
and they contradicted each other twice — both contradictions were settled by
measurement and are recorded.

Worst of them, for context:
- `prevailing` decided on the scan grid rather than the refined boundary, so the
  day view can name a different tithi than the cell it opened from.
- `Engine::reconfigure` drops the lock mid-change; readers get a longitude from a
  configuration that was never selected.
- A month computed under the old ayanamsa is stored *after* the invalidation
  meant to drop it, and served as current forever.

The method note at the top of `AUDIT.md` is the part worth keeping: two tests
passed while the thing they guarded was broken. **A fix whose test would still
pass against the old code is not finished.**

| | |
|---|---|
| Status | in flight |
| Then | a second agent verifies the first's work, then I check it |

---

## 2. Contrast on the translucent panel — blocked on you

Measured on the shipped panel over ordinary content: the composited ground runs
from `#1A3033` to `#5C4E36`.

| | Worst case |
|---|---|
| `--text-primary` | 6.91:1 — fine |
| `--text-secondary` | 3.15:1 — fails |
| `--text-tertiary` | **2.21:1** — fails 4.5:1, and fails 3:1 |

No scrim alpha fixes it: 0.55 gets tertiary to 3.34:1, and 0.70 — which would
destroy the translucency — reaches only 4.09:1. The conclusion is not "darken the
panel" but that **tertiary is too light to carry text on a translucent surface**.
Every uppercase field label in the day view is tertiary today.

Proposed: scrim to 0.55, retire tertiary as a text colour, keep it for non-text
marks where the bar is 3:1. Labels and the Gregorian date line move to secondary.

| | |
|---|---|
| Status | blocked on you — it is a token change touching every surface |

---

## 3. State marks — mocked, blocked on you

Agreed already:
- Marks must not vary between solar and lunar mode.
- No underlines. The rule under a numeral read as underlined text.
- Combustion may be coloured, because **colour may depict but may not encode**.
  A flame is warm because flames are warm.
- Rahu and Ketu wear the retrograde mark permanently, and that is accepted.
- The Moon's combustion is **not** suppressed in the grid, and shows in the day
  view like every other subject.

Mocked and awaiting your word: the dotted ring for retrograde, the flame's hue
and weight, and the Gregorian date moving to the cell's corner so the glyph can
have the second line back in both modes.

| | |
|---|---|
| Status | blocked on you |

---

## 4. Moon phase glyphs — ready

Requested twice. In lunar mode the Moon's cell lost its phase disc, because the
second line was given to the Gregorian date. The date moves to the corner and the
glyph comes back, in both modes and for every subject.

This also removes two mode-dependent CSS rules — the combustion rule and the `℞`
position both had a lunar variant purely because the glyph was missing — which is
the marks rule applied to itself.

| | |
|---|---|
| Status | ready, doing now |

---

## 5. Day view — decided, queued

Settled:
- Variant **02**: label above, value and time on one line at the same size, the
  successor named underneath.
- Rise and set belong to the subject. Surya's calendar shows sunrise and sunset,
  Chandra's shows moonrise and moonset, Shani's shows Shani's. There is no
  separate sunrise row.
- Dropped: illuminated percentage, distance from the Sun, speed, longitude.
- Phase name in solar mode only.
- Title is the full tithi in lunar mode, the western date in solar.

Specified in `docs/design/day-view-states.md`: combustion as a warm field in the
two gutters the day view never uses; retrograde as a hairpin rail. Neither costs
a vertical pixel, which matters given what is queued in §7.

| | |
|---|---|
| Status | queued behind §1 — the fix agent is in `DayDetail.tsx` |

---

## 6. Year navigation — queued

Keeping the Moshier precision note creates an obligation: if a reading can be
degraded by navigating far enough, getting there must not be tedious. Today it is
one month per gesture. Needs a way to move by year.

| | |
|---|---|
| Status | queued, unspecified |

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

This roughly doubles what a day holds, on the surface currently being made
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

| | |
|---|---|
| Status | queued |

---

## 9. Not started

- GitHub remote. Deferred by D-018 until a production release; still local-only.
- CI. Described in the Makefile and the docs; no `.github/workflows` exists.
- Release DMG on tag.
