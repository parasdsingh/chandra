# Lunar dates in the calendar grid

Status: **proposed, then built.** Shipped as [D-021](../DECISIONS.md#d-021) and closed as
[I-045](../ISSUES.md); parts of it were later replaced. See the note below before reading any
pixel value here as current.
Resolves the second half of [I-032](../ISSUES.md) — lunar months already resolve and label
correctly, but every cell still shows a Gregorian numeral.
Extends [D-010](../DECISIONS.md#d-010) by the smallest amount that makes the claim on the
header true. Expressed in the vocabulary of [DESIGN.md](../DESIGN.md).

---

## What shipped, and what was replaced

The design below is left intact as filed. Where it and the code disagree, **the code wins**.

**Shipped, and still true**

- The tithi is the cell's primary numeral in lunar mode, in panchang notation — `S1`–`S14`,
  `P`, `K1`–`K14`, `A` — derived from paksha plus number within the paksha, never from the
  astronomical 1–30 index.
- The 7-column vara grid, unchanged, with locale-provided weekday labels.
- The Gregorian day carries its 3-letter month only on a Gregorian 1st: `1 SEP`.
- The back end lays out all 42 cells with an `in_month` flag; the opening weekday travels the
  other way, in the request.
- Vikram Samvat years, and the tithi span behaviour in the day view (§7.4).
- Solar mode pays nothing: no tithi is computed for it.

**Superseded**

- **Nothing gives way** ([D-024](../DECISIONS.md#d-024)). §2.3 and the §4.2 cell anatomy give
  the second line to the Gregorian date and drop the phase glyph and the graha symbol. Both are
  back, on the second line, in *both* calendars; the Gregorian date moved to the cell's
  top-right corner, where it is set in `--text-secondary` rather than Micro-caps under the
  numeral.
- **The kshaya dot is on the numeral**, not in the cell's top-left corner — that corner is the
  Gregorian date's now. **The vriddhi rule is not drawn at all**: the §5 rule joining two cells
  along their bottom edge went with the rest of the underline vocabulary (D-024). A repeated
  tithi is visible as the same numeral on two cells and named in the spoken label.
- **The `S`/`K` prefix is `--text-secondary`, not `--text-tertiary`**, as is the Gregorian date
  ([D-023](../DECISIONS.md#d-023)). Tertiary carries no text anywhere in the app, so §11 and
  §12.3's figures for it are moot.
- **There is no Sunrise row** ([D-025](../DECISIONS.md#d-025)). The day view is one field stack
  for all nine subjects, and rise belongs to the subject: sunrise is Surya's rise, on Surya's
  day. The Tithi block of §7 is one field in that stack.
- **The header label names the era**: `Shravana VS 2083`, not the `Shravana 2026` of §1
  (AUDIT W-05). `Adhika` is set apart in `--text-secondary`, as §6.2 specifies.

---

## Decision summary

- **Tithi at sunrise is the cell's primary numeral in lunar mode**, in the notation printed
  panchangs already use: `S1`–`S14`, `P` (Purnima), `K1`–`K14`, `A` (Amavasya). Numeral role,
  `--text-primary`; the `S`/`K` prefix in `--text-tertiary`. No new colour token.
- **The phase glyph gives way**, and with it the Moon's combustion rule. Both are duplicated ink
  in lunar mode: tithi *is* elongation ÷ 12, stated more precisely than a Ø14 disc can show, and
  the Moon's combustion is Amavasya, which the numeral already names (DESIGN §1.3).
- **The Gregorian day moves into the freed band**, Micro-caps 10/600 `--text-tertiary`, centred
  under the tithi. It carries the 3-letter month only on a Gregorian 1st: `1 SEP`.
- **The 7-column vara grid survives**, unchanged. The two authentic alternatives were weighed and
  both fail this panel: the transposed 7-row wall-calendar grid needs 280px in a 264px region, and
  a paksha-blocked layout would have to lie about kshaya and vriddhi. Weekday labels stay
  locale-provided — a romanised Sanskrit abbreviation has no source to copy.
- **Kshaya and vriddhi are drawn, never silent.** A day that swallows a skipped tithi carries a
  Ø4 `--marker` corner dot; the two days of a repeated tithi are joined by a 1px `--marker` rule
  at the cell's bottom edge, reading as one span across two cells. Both are named in words in the
  day view and in the spoken label.
- **The day view gains exactly two things**: a Tithi span block (first field block) and a Sunrise
  row. Yoga and karana stay out — neither explains a numeral in the grid.
- **Solar mode is unchanged**, pixel for pixel. Every rule below is gated on
  `settings.calendar.month_system != "solar"`.
- **Degraded is stated, never invented.** No sunrise → tithi taken at local noon, said once in the
  day view. Unresolved boundary → `time unavailable` in the caption; the numeral itself never
  needs root-finding and so never goes missing.

---

## 1. Scope

| Surface | Solar mode | Lunar mode (`amanta` \| `purnimanta`) |
|---|---|---|
| Header label | `Chandra · August 2026` | `Chandra · Shravana 2026` — wording unchanged; `Adhika` is now set in `--text-secondary` (§6.2) |
| Vara row | locale short names | locale short names (unchanged) |
| Cell top | Gregorian day | **tithi + paksha** |
| Cell bottom | phase glyph Ø14 | **Gregorian day, Micro-caps** |
| Cell marks | combustion rule | **kshaya dot, vriddhi rule** |
| Day view | 6 D-010 fields | **+ Tithi block, + Sunrise row, + vara name** |
| Panel geometry | 320 × (40 header + 264 region) | identical, no exception |

---

## 2. Why the cell reads this way

### 2.1 The primary numeral

- The user's complaint is that the grid reads as Gregorian. The numeral is what makes it read
  that way, so the numeral is what changes.
- Tithi at sunrise is the correct primary: a Hindu civil day is *named* after the tithi in force
  at its sunrise. Every printed and digital panchang consulted (§14) applies the same rule.
- Numbering is **1–15 within the paksha**, not 1–30. Sewell & Dikshit, *The Indian Calendar*
  (1896) §29, states it plainly: "There are 30 tithis in a lunar month, 15 to each fortnight…
  these are used for the fifteen tithis of each fortnight."
- The **fifteenth of each paksha is a name, not a number**. The notation adopted here is
  `S1`…`S14`, then `P` for Purnima; `K1`…`K14`, then `A` for Amavasya — the legend mypanchang's
  calendars carry (§14). Where this disagrees with older print practice is recorded in §2.5.
- Consequence, and a good one: the two days people actually search a calendar for are the two
  that stand out in the grid. Position confirms them — in amanta the last in-month cell is always
  `A`, in purnimanta always `P`.
- The cell string is derived from **paksha + number**, never from a continuous index: `Shukla 15`
  → `P`, `Krishna 15` → `A`, otherwise `S`/`K` + the number. This is the only derivation that is
  correct in both systems, because the two count from opposite ends (§2.5).

| Tithi | Cell | Day view |
|---|---|---|
| 1st of Shukla | `S1` | `Shukla Pratipada` |
| 8th of Shukla | `S8` | `Shukla Ashtami` |
| 11th of Krishna | `K11` | `Krishna Ekadashi` |
| 15th of Shukla | `P` | `Shukla Purnima` |
| 15th of Krishna | `A` | `Krishna Amavasya` |

Names, in order, as they appear in the day view: Pratipada, Dwitiya, Tritiya, Chaturthi,
Panchami, Shashthi, Saptami, Ashtami, Navami, Dashami, Ekadashi, Dwadashi, Trayodashi,
Chaturdashi, Purnima / Amavasya. Transliterated without diacritics, matching `zodiac.rs`.

### 2.2 Paksha without a new colour

Three carriers were considered.

| Carrier | Verdict |
|---|---|
| Lit side of the phase glyph (right = Shukla, left = Krishna) | Astronomically exact — Shukla *is* the waxing half — but a glyph alone, which DESIGN §11.3 forbids as a state carrier |
| A colour split (gold Shukla / grey Krishna) | Rejected. `--accent` is TODAY ONLY (DESIGN §1.4). A new hue would need a fourth meaning in a two-hue app |
| Latin prefix `S` / `K` | **Selected.** It is text, so DESIGN §11.3 is satisfied outright; it is the notation mypanchang and most English-language panchangs already print; it costs 8px of a 40px cell |

- The prefix is set at the numeral role, `--text-tertiary`, with **no gap**: `S8`, not `S 8`.
  A 4px gap would read as two separate values.
- On the today cell the whole string takes `--accent`. Splitting the colour of one word reads as
  a rendering fault.

### 2.3 What gives way

The moon cell has five slots between DESIGN §5.4 and §6.3. Lunar mode reassigns four of them.

| Slot | Solar moon cell | Lunar moon cell | Reason |
|---|---|---|---|
| Numeral (top) | Gregorian day | tithi + paksha | §2.1 |
| Content (bottom, Ø14) | phase glyph | **removed** | Duplicated ink: tithi = ⌊elongation / 12⌋ + 1, verified at [RESEARCH.md R-03](../RESEARCH.md) — elongation 88.72° → tithi 8, Shukla Ashtami, 20 Aug 2026. The numeral states the phase to 12° where the disc states it to perhaps 60°. DESIGN §1.3 |
| Combustion rule (12 × 1px under numeral) | Moon within 12° of the Sun | **removed** | For the Moon that condition is exactly K14–S1. The cell already says `A`. DESIGN §1.1 — nothing is duplicated across two carriers |
| Corner mark (Ø4, top-right) | unused in the moon panel | **kshaya** | §5 |
| Bottom-edge span rule (40 × 1px) | unused in the moon panel | **vriddhi** | §5 |

The freed 14px content band takes the Gregorian day. Total ink in the cell is unchanged;
one datum was swapped for another and two duplicates were deleted.

### 2.4 Where the Gregorian date lives

- It must stay in the grid, not only in the day view: a user checks "which cell is the 20th"
  while scanning, and one round trip per lookup is not a calendar.
- Bare day-of-month is unambiguous inside a lunar month: a 29- or 30-day span never repeats a
  day number.
- The Gregorian month changes once inside every lunar month. That cell — and only that cell —
  renders `1 SEP` instead of `1`. Anywhere else the month abbreviation would be 42 repetitions
  of one fact (DESIGN §9.4's own rule about grid-level annotation).
- The 3-letter form is the locale's `Intl` short month, uppercased, matching the vara row's
  micro-caps role. `1 SEP` measures 29.6px inside a 40px cell.

### 2.5 Where the sources disagree

Four disagreements are real. None is a defect to design around; all are recorded so nobody
"fixes" them later.

| Disagreement | Resolution |
|---|---|
| **Amanta vs purnimanta.** Amanta runs new moon → new moon and orders the pakshas Shukla then Krishna; purnimanta runs full moon → full moon and orders them Krishna then Shukla. Sewell & Dikshit §51: "the bright fortnights have the same name by both schemes while the dark fortnights differ by a month, and thus the purnimanta scheme is always a fortnight in advance of the amanta scheme." Verified side by side for February 2026 — Kalnirnay (amanta) prints `माघ अमावास्या` on 17 Feb where Thakur Prasad (purnimanta) prints `फाल्गुन कृष्ण अमावस्या`, while both agree on the Shukla days | Already a user setting. `lunar.rs::build` names a purnimanta month from the new moon *inside* it, which is exactly the rule that produces the one-fortnight lead. Correct as implemented; nothing here changes it |
| **Which states use which.** Britannica lists Goa and Telangana under amanta; Wikipedia excludes Assam, West Bengal, Odisha, Tamil Nadu and Kerala from the amanta list because they keep their own solar calendars. Both are defensible — those states date civil life by solar months but reckon tithi and festivals amanta | Does not reach the UI. The app never infers the system from a location; the user picks it in Settings. No state list is shipped |
| **Tithi ordering within a month.** Wikipedia (*Tithi*): "In *amānta* lunar calendars, *tithi*s are counted beginning at *śukla pratipada*, while in the *pūrṇimānta* lunar calendars, *tithi*s are counted from *kr̥ṣṇa pratipada*" | A continuous 1–30 index therefore means different things in the two systems. The cell string is derived from **paksha + number**, which is invariant (§2.1). `tithi_index` on the wire stays astronomical — always counted from Shukla Pratipada — and `MoonMonth.first_paksha` carries the ordering (§10) |
| **Amavasya written as `30`.** Sewell & Dikshit §29: "The numeral 30 is generally applied to the amavasya (new moon day) in panchangs, **even in Northern India** where according to the purnimanta system… the amavasya [is] really the 15th tithi." Kalnirnay still does this, running `Krishna 14 → Krishna 30 → Shukla 1`. mypanchang's legend instead substitutes the letter `A` | `A` is used. `30` inside a grid whose every other numeral is 1–14 would read as an error, and the mixed-radix jump 14 → 30 is precisely the "silent renumbering" this design exists to prevent. `A` and `30` denote the same tithi; the day view spells it `Krishna Amavasya`, which both conventions agree on |

---

## 3. Grid structure

**Decision: keep the 7-column, 6-row vara grid, unchanged from solar mode.**

Three real layouts were weighed. Two of them are what actual panchangs use, so the argument is
not "nobody does this" — it is that neither survives a 320 × 264 region with a fixed 40 × 40 cell.

### 3.1 The transposed grid — 7 weekday rows × 5 week columns

This is the **Indian wall-calendar norm**, not an outlier: Kalnirnay (Marathi, amanta) and Thakur
Prasad Panchang (Hindi, purnimanta, published since 1930) both use it, with a weekday rail down
the left edge (§14). It is genuinely the most authentic option.

| Against | Weight |
|---|---|
| **It does not fit.** 7 rows × 40px = 280px against a 264px region (DESIGN §5.1). Shrinking the row to 36px would fit at 252px, but the cell is 40 × 40 with `--r-cell` 8 and a 4px internal grid throughout DESIGN §5.4 — the whole cell anatomy is built on it | decisive |
| **It wastes the width it does not use.** 5 columns × 40px = 200px inside a 320px panel, leaving 120px of dead rail | decisive |
| **It would fork the two modes.** Solar mode must stay pixel-identical (§8). Two different grid geometries in one panel is two calendars, not one app with a setting | strong |

### 3.2 The paksha-blocked layout — Shukla 1–15, then Krishna 1–15

Also real, and the research corrects an earlier assumption here: it is the **almanac-book** form,
not the wall-calendar form. Sewell & Dikshit reproduce an 1894 Poona panchang in which "the month
is divided into its two fortnights", one horizontal row per tithi, with the weekday as a column
*inside* the row. The Government of India's *Rashtriya Panchang* is the same day-per-block genre.

Structurally seductive for this panel — 5 columns × 3 rows per paksha lands in exactly 6 rows of
40px = 240px, the region's whole budget — and rejected anyway.

| Against | Weight |
|---|---|
| **A paksha block is not always 15 cells.** A kshaya tithi makes a paksha 14 civil days; a vriddhi tithi makes it 16. A fixed 15-cell block would have to renumber, pad, or hide — every one of which is the silent renumbering this design exists to prevent | decisive |
| **Vara is a limb.** Tithi, vara, nakshatra, yoga, karana. The week is not a Gregorian import into panchanga; Ravivara…Shanivara map 1:1 onto Sunday…Saturday. Deleting the week to display a limb costs a limb | decisive |
| **The genre does not transfer.** The almanac form is one row per day with several fields in it, read sequentially. It is 30 rows tall. A 264px region shows six. What survives the compression is the grid, not the block | strong |

### 3.3 The 7-column grid — selected

- It is what every *digital* panchang uses (Drik Panchang, mypanchang, the ISKCON calendars) and
  what the transposed wall calendars carry too, merely on the other axis. The weekday structure is
  common to all three layouts; only its orientation differs.
- It is the shipped geometry. Nothing in DESIGN §5.1, §5.4 or §5.5 has to move.

Consequences, all already satisfied by the shipped grid:

- 6 rows × 40px = 240px. Unchanged. Panel never resizes.
- A 29- or 30-day lunar month plus lead and trail always fits 42 cells.
- Lead and trail cells are the adjacent **lunar** months' days, dimmed per DESIGN §5.5 — not
  Gregorian filler. They therefore need tithi data too (§10, `grid`).
- Month navigation stays the scroll/drag strip; there is no numbered sequence to page through.

---

## 4. Lunar-mode wireframes

### 4.1 The panel — Shravana 2026, amanta, 13 Aug – 11 Sep

New moon 12 Aug 2026; Purnima 27 Aug; Amavasya 11 Sep. Tithi values consistent with
[RESEARCH.md R-03](../RESEARCH.md) (20 Aug 2026 = Shukla Ashtami).

```
 x:0    20                                             300  320
      ┌──────────────────────────────────────────────────┐  y:0
      │                                                  │  12   pad-top
      │  ◐  Chandra · Shravana 2026                 [⚙]  │  40   header
      │                                                  │   4
      │  SUN   MON   TUE   WED   THU   FRI   SAT         │  20   vara row
      │                                                  │   4
      │ ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┐      │
      │ │ K12 │ K13 │ K14 │  A  │ S1  │ S2  │ S3  │  40  │   ← Ashadha trail, dim
      │ │  9  │ 10  │ 11  │ 12  │ 13  │ 14  │ 15  │      │
      │ ├─────┼─────┼─────┼─────┼─────┼─────┼─────┤      │
      │ │ S4  │ S5  │ S6  │ S7  │ S8  │ S9  │ S10 │  40  │
      │ │ 16  │ 17  │ 18  │ 19  │ 20  │ 21  │ 22  │      │  240
      │ ├─────┼─────┼─────┼─────┼─────┼─────┼─────┤      │  grid
      │ │ S11 │ S12 │ S13 │ S14 │  P  │ K1  │ K2  │  40  │  6 rows,
      │ │ 23  │ 24  │ 25  │ 26  │ 27  │ 28  │ 29  │      │  always
      │ ├─────┼─────┼─────┼─────┼─────┼─────┼─────┤      │
      │ │ K3  │ K4  │ K5  │ K6  │ K7  │ K8  │ K9  │  40  │
      │ │ 30  │ 31  │1 SEP│  2  │  3  │  4  │  5  │      │   ← month change
      │ ├─────┼─────┼─────┼─────┼─────┼─────┼─────┤      │
      │ │ K10 │ K11 │ K12 │ K13 │ K14 │  A  │ S1  │  40  │
      │ │  6  │  7  │  8  │  9  │ 10  │ 11  │ 12  │      │   ← Bhadrapada, dim
      │ ├─────┼─────┼─────┼─────┼─────┼─────┼─────┤      │
      │ │ S2  │ S3  │ S4  │ S5  │ S6  │ S7  │ S8  │  40  │   ← all dim
      │ │ 13  │ 14  │ 15  │ 16  │ 17  │ 18  │ 19  │      │
      │ └─────┴─────┴─────┴─────┴─────┴─────┴─────┘      │
      │                                                  │  12   pad-bottom
      └──────────────────────────────────────────────────┘
        └────────── 7 × 40 = 280 ──────────┘
```

Reading it:

- The Shukla half runs 13–27 Aug, the Krishna half 28 Aug – 11 Sep. In **purnimanta** the same
  grid opens on `K1` and closes on `P`; nothing else about the layout differs.
- The Gregorian range is legible without a header subtitle: the second line of the first and last
  in-month cells is `13` and `11`, with `1 SEP` marking the crossing.
- `P` on 27 Aug and `A` on 11 Sep are the syzygies, found without counting.

### 4.2 Cell anatomy (40 × 40)

Every value is a multiple of 4 — one step cleaner than the solar cell, which needs a 2px gap to
fit a Ø14 disc.

```
        x:0                    20                   40
    y:0  ┌───────────────────────────────────────────┐
         │                                           │   4   pad-top   --s-2
    y:4  │                 ┌──────┐                  │
         │                 │  S8  │  tithi + paksha  │  16   Numeral 13/450/16
   y:20  │                 └──────┘                  │
         │                                           │   4   gap       --s-2
   y:24  │                  ┌────┐                   │
         │                  │ 20 │  Gregorian day    │  12   Micro-caps 10/600/12
   y:36  │                  └────┘                   │
         │                                           │   4   pad-bottom --s-2
   y:40  └───────────────────────────────────────────┘
```

| Relation | Value |
|---|---|
| Tithi numeral cap-height (SF Text 13px) | 9.3px |
| Gregorian cap-height (SF Text 10px) | 7.1px |
| Cap-height ratio, primary : secondary | **1.31 : 1** |
| Widest tithi string `K14` | 22.3px (tabular); narrowest `P` is 7.9px |
| Widest Gregorian string `1 SEP` | 29.6px (+0.6px tracking per glyph) |
| Both strings optically centred on | x = 20 |

The numeral is the label and the Gregorian date is the annotation — the inverse of the solar
cell, where the numeral labels and the glyph is the content. In lunar mode there is no content
below the label because the label *is* the content.

### 4.3 Cell states, 4× zoom

Each box is 40 design-px wide. Marks are drawn at their real offsets.

**Normal** — Thursday 20 August 2026, Shukla Ashtami.

```
  ┌────────────────────────────────────────┐  y 0
  │                                        │
  │                 S8                     │  y 4–20   S: --text-tertiary
  │                                        │           8: --text-primary
  │                 20                     │  y 24–36  --text-tertiary
  │                                        │
  └────────────────────────────────────────┘  y 40
```

**Today** — ring inset 3 (34 × 34, r 7), whole string in `--accent`.

```
  ┌────────────────────────────────────────┐
  │ ╭────────────────────────────────────╮ │  inset 3px, 1px --accent, r 7
  │ │               S8                   │ │  S8 both glyphs --accent
  │ │                                    │ │
  │ │               20                   │ │  --text-tertiary (accent does
  │ ╰────────────────────────────────────╯ │   not spread to the annotation)
  └────────────────────────────────────────┘
```

**Selected** — fill inset 2 (36 × 36, r 8) `--surface-selected`, numeral weight 450 → 600.

```
  ┌────────────────────────────────────────┐
  │╭──────────────────────────────────────╮│  inset 2px, fill --surface-selected
  ││                                      ││
  ││               S8                     ││  weight 600
  ││                                      ││
  ││               20                     ││  weight 600, still tertiary
  │╰──────────────────────────────────────╯│
  └────────────────────────────────────────┘
```

**Kshaya-adjacent** — illustrative. Shukla Shashthi begins and ends between two sunrises, so the
grid runs … `S5`, `S7` … and the `S5` cell — the one that contains the skipped tithi — is marked.

```
  ┌────────────────────────────────────────┐┌───────────────
  │                                   ●    ││              ← Ø4 --marker,
  │                 S5                     ││   S7            top 5, right 6
  │                                        ││                 (the DESIGN §6.3
  │                 17                     ││   18            marker slot)
  │                                        ││
  └────────────────────────────────────────┘└───────────────
        marked: contains Shukla Shashthi        numeral jumps 5 → 7
```

**Adhika (vriddhi) repeated** — illustrative. Krishna Tritiya spans two sunrises. Both cells read
`K3`, joined by one rule.

```
  ┌───────────────────┐┌───────────────────┐
  │                   ││                   │
  │        K3         ││        K3         │
  │                   ││                   │
  │        30         ││        31         │
  │                   ││                   │
  │▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂││▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂│  1px --marker at y 39.5,
  └───────────────────┘└───────────────────┘  full-bleed 40px, both cells
       first sunrise        second sunrise    → one continuous span
```

---

## 5. Kshaya and vriddhi

Both are real, both are common enough to meet within a year, and a design that renumbers around
them silently is wrong. The rule set is derived from DESIGN §6.3, not invented:
**an instant gets a mark; a span gets a rule.**

| Case | Definition | Grid | Rule kind |
|---|---|---|---|
| **Kshaya tithi** | A tithi that contains no sunrise: it begins after one sunrise and ends before the next, so no civil day is named after it and the grid numerals jump | Ø4 filled `--marker` dot, top-right (top 5, right 6), on the day that contains it — i.e. the day *before* the jump | mark — the whole tithi is compressed inside one day, so it behaves as an instant at day scale |
| **Vriddhi (adhika) tithi** | A tithi that contains two sunrises. Two consecutive civil days carry the same numeral | 1px `--marker` rule, full-bleed 40 × 1 at y = 39.5, on **both** days | rule — it is a span, and drawn this way the pair reads as one tithi bracketed across two cells |

Distinguishing them from a bug:

- **The mark is the difference.** An unmarked numeral jump would be a bug; a marked one is a
  kshaya. A repeated numeral with no rule would be a bug; with the rule it is a vriddhi.
- **The day view names both, in words**, in the tithi block (§7.3). Nothing is carried by shape
  alone (DESIGN §11.3).
- **The spoken label names both** (§12.2).
- The rule is `--marker`, never `--retro`. The bottom-edge slot is the retrograde span rule in the
  *graha* panel; the moon panel never draws retrograde, so within this surface the slot is
  unambiguous (DESIGN §1.1 — one surface, one job).

Frequency, so the marks are known to be rare rather than assumed to be:

- A tithi is 12° of elongation, and the Moon's motion is uneven, so a tithi runs anywhere from
  about **19 to 26 hours** (§14). Against a 24-hour civil day, both outcomes are ordinary.
- One synodic month is 29.53 days and carries 30 tithis, so roughly one civil day per month is
  either a kshaya swallow or a vriddhi repeat.
- The two never occur on the same cell: a day whose sunrise tithi has two sunrises cannot also
  contain a tithi with none.

---

## 6. Vara row and header label

### 6.1 Vara row

**Decision: keep the locale's own short weekday names, in both modes. Keep the locale's first
day of week.**

| Option | Verdict |
|---|---|
| Locale short names — `MON TUE WED …` | **Selected** |
| Sanskrit transliteration — `RAV SOM MAN BUD GUR SHU SHA` | Rejected |
| Both | Impossible — one 20px row of 40px columns |

Arguments:

- The Hindu week and the civil week are the **same week**. Ravivara *is* Sunday. A transliteration
  adds a decoding step and no information — the opposite of intuition without instruction.
- `Somavara` at Micro-caps 10/600 measures 56px against a 40px column. Three-letter
  abbreviations collide: Shukravara and Shanivara both give `SHA`/`SHU`, which is exactly the kind
  of ambiguity DESIGN §7.2 spends a page avoiding for Rahu and Ketu.
- **A Latin vara abbreviation has no source to copy.** Print panchangs abbreviate in Devanagari —
  Kalnirnay uses single aksharas `र सो मं बु गु शु श`, Drik and Thakur Prasad use
  `रवि सोम मंगल बुध गुरु शुक्र शनि`. Every *English*-side abbreviation in every calendar
  examined is `Sun Mon Tue…`. A romanised `Ra So Ma Bu Gu Shu Sha` would be invented, which is
  the one thing this design is not allowed to do (§14).
- The vara **name** is given in full in the day view (§7.2), where there is room for it. That is
  where the limb is named, once, in words.
- Forcing a Sunday start in lunar mode was considered and rejected as a silent override of a
  system setting the user already chose; it is raised as an open question instead (§13).

Unchanged from today: Micro-caps 10/600/+0.60px, `--text-tertiary`, uppercase, three letters,
`AppleFirstWeekday` via `Intl.Locale.getWeekInfo` (DESIGN §5.3).

---

### 6.2 Header label

Unchanged in form from what ships today; specified here so the rules are written down.

| Case | Label | Notes |
|---|---|---|
| Ordinary lunar month | `Chandra · Shravana 2026` | Title 15/600, `--text-primary` |
| Intercalary month | `Chandra · Adhika Shravana 2023` | `Adhika` is the prefix Drik Panchang prints (§14). `Purushottama Masa` and `Mala Masa` are the same month under devotional names and are not used here — the calendar states the calendrical fact, not the observance |
| Kshaya masa | `Chandra · Pausha–Magha 2124` | En dash, both names. Requires `sankranti_count` (§10.1); not detected today |
| Solar month | `Chandra · August 2026` | Unchanged |

- **No paksha in the header.** A lunar month contains both pakshas; naming one would be wrong for
  half the grid. Paksha lives in the cell, where it is true.
- **`Adhika` is set in `--text-secondary`** while the month name stays `--text-primary`. It is a
  qualifier on the name, not part of it, and the distinction is what stops `Adhika Shravana` from
  reading as a thirteenth month name.
- **Truncation is already handled.** `Header.tsx` walks a ladder — full string, then without the
  year, then without the subject name. `Chandra · Adhika Shravana 2023` measures ≈ 225px against a
  216px slot and drops one rung to `Chandra · Adhika Shravana`. No ellipsis, per DESIGN §5.2.
- **The year stays Gregorian** — see open question 1 (§13).

---

## 7. Day view

Region is the same 264px and scrolls. The block order below is the reading order.

### 7.1 Wireframe — 20 August 2026, Shukla Ashtami

```
      ┌──────────────────────────────────────────────────┐
      │  ‹   20 August 2026                         [⚙]  │  40   header
      ├──────────────────────────────────────────────────┤
      │                                                  │  12   pad-top
      │  THURSDAY · GURUVARA                             │  14   Micro-caps, tertiary
      │                                                  │   4
      │  Waxing Gibbous                       68.4% lit  │  24   Headline | Body sec.
      │                                                  │  12
      │  Tithi                        Shukla Ashtami     │  24   Label | Body
      │                        19 Aug 21:14 → 18:02      │  18   Caption, tertiary
      │                                                  │  12
      │  Sunrise                                 06:11   │  24
      │  Moonrise                                15:42   │  24
      │  Moonset                                 02:11   │  24
      │                                                  │  12
      │  Nakshatra              Purva Ashadha · Pada 2   │  24
      │                        04:31 → 07:12 (21 Aug)    │  18
      │                                                  │  12
      │  Rashi                                  Dhanu    │  24
      │                        19 Aug 11:04 → 22:47      │  18
      │                                                  │  16   pad-bottom
      └──────────────────────────────────────────────────┘
       20 ├──────────────── 280 ────────────────┤ 20
```

Total 314px inside a 264px region: it scrolls, and the existing `.region::after` fade already
says so. No new mechanism.

### 7.2 What was added, and why each

| Added | Argument |
|---|---|
| **Tithi** span block, first block | The grid now states a number. DESIGN §11.3 requires the number to be named in words somewhere on the same surface. It is also the day's identity in this mode, so it leads |
| **Sunrise** row | The tithi, nakshatra and rashi in this view are all attributed to the day's reference instant, which is sunrise (`day.rs::reference_instant`). Without it the primary numeral is unauditable — the user cannot check *why* the cell says `S8`. This is the same obligation D-006 discharges for provenance |
| **Vara name** appended to the date line | Completes the second limb at zero cost: no new row, one word, in the row that already carries the weekday. Reads `THURSDAY · GURUVARA`; with `TODAY · THURSDAY · GURUVARA` when applicable, 27 characters at Micro-caps ≈ 178px inside 280px. Spelling: `Guruvara`, matching Drik Panchang's `Guruwara` in substance. Wikipedia's *Indian national calendar* gives `Brihaspativara` as primary with *Guruvara* as the alternate, and Britannica uses the `-vasara` suffix throughout; `-vara` is chosen because it is shorter and is what the digital panchangs print |

### 7.3 What was refused

| Refused | Argument |
|---|---|
| **Yoga** | Not in the grid, explains no numeral, adds a row and a computation. D-010 excluded it; nothing in this change gives a reason to include it. Noted: the canonical panchang field order *is* Tithi → Nakshatra → Yoga → Karana (Rashtriya Panchang, §14), so if yoga and karana are ever added they slot in after Nakshatra and before Rashi, not at the end |
| **Karana** | A karana is half a tithi. Its value and its boundaries are fully derivable from the tithi row already present — duplicated ink (DESIGN §1.3) |
| **Sunset** | Nothing in the view is referenced to sunset. It is added only by reflex, because panchangs print rise/set as a pair |
| **Muhurta (Rahu Kaal etc.)** | Deferred in [ROADMAP.md](../ROADMAP.md) and untouched by this change |
| **Tithi in solar mode** | §8. Raised as an open question instead |

### 7.4 Tithi block behaviour

Reuses `SpanBlock` exactly as the nakshatra and rashi blocks do — label on the first row only,
continuation rows dimmed to `--text-secondary`, caption right-aligned.

| Situation | Rows |
|---|---|
| One tithi at sunrise, one after | `Shukla Ashtami` / `19 Aug 21:14 → 18:02` then `Shukla Navami` / `18:02 → 15:20 (21 Aug)`, second row dimmed |
| **Kshaya present** | Extra dimmed row: `Shukla Shashthi` / `09:41 → 06:02 (19 Aug) · kshaya, no sunrise` |
| **Vriddhi, first day** | `Krishna Tritiya` / `29 Aug 23:06 → 07:02 (31 Aug) · first of two sunrises` |
| **Vriddhi, second day** | `Krishna Tritiya` / `29 Aug 23:06 → 07:02 · second sunrise` |
| Purnima / Amavasya | `Shukla Purnima` / `Krishna Amavasya` — the paksha word is kept, because the cell shows only `P` / `A` and the day view is where the paksha is spelled out |
| Boundary unresolved | caption reads `21:14 → time unavailable` |

Suffixes (`· kshaya, no sunrise`, `· second sunrise`) are Caption 11/400 `--text-tertiary`, in the
existing caption row. No new row, no new colour, no icon.

**Entry → exit, not end-time-only.** Printed panchangs give the *end* time alone — the
Rashtriya Panchang prints `Tithi: (Phalguna Krishna) Dvadasi h. 7-56`, one instant. This app shows
both boundaries because `Span` already carries the true entry and exit (ARCHITECTURE §3.2) and the
nakshatra and rashi rows directly below already read that way. One convention inside one panel
beats matching print on one row and contradicting it on the next. The end time is still the
right-hand value, where the eye lands.

---

## 8. Solar mode

Unchanged, pixel for pixel.

- Every rule in this document is gated on `month_system != "solar"`.
- The cell keeps its Gregorian numeral, its Ø14 phase glyph, and its combustion rule.
- The day view keeps exactly the six D-010 fields. No tithi row, no sunrise row, no vara name.
- The only shared change is the `grid` field in §10, which supplies 42 cells instead of 29–31.
  In solar mode this makes adjacent-month cells carry phase data, which is what
  DESIGN §5.5 already specifies ("glyph at 45% alpha") and the shipped grid does not do. That is
  a conformance fix to the published spec, not a new behaviour, and it is the one line of this
  design that touches solar mode. **Flagged for approval before implementation.**

---

## 9. Degraded and edge cases

None of these is an error. DESIGN §9.4: degraded states are facts, not faults — no warning hue,
no icon, no border.

| Case | Cell | Day view | Spoken |
|---|---|---|---|
| **Tithi number unavailable** | Cannot happen in isolation. The number is `⌊elongation(sunrise) / 12⌋ + 1` — one ephemeris call, no root-finding. If that call fails the whole month fails and the existing `ENGINE` / `DATE_OUT_OF_RANGE` path in DESIGN §9.3 applies | — | — |
| **Tithi boundary time unresolved** (`NoConvergence`) | Nothing. The numeral is unaffected | Caption reads `21:14 → time unavailable`. The resolved side still prints. No guessed time is ever shown (ARCHITECTURE §6) | `… ends, time unavailable` |
| **No sunrise** — polar day or night | Numeral drawn normally. **No cell mark**: polar night runs for weeks, and marking 42 cells to say one thing is the clutter DESIGN §9.4 forbids for provenance | One `--annotate` line at the foot, in the provenance slot: `No sunrise at this latitude — tithi taken at local noon.` | Each cell appends `, tithi at local noon` |
| **Sun never sets** | Same as above; the reference instant is local noon either way (`day.rs::reference_instant`, already implemented) | Same line | Same suffix |
| **Adhika masa** | Nothing in the cell — it is a property of the month | Header reads `Adhika Shravana 2023` (already implemented) | Grid label `Adhika Shravana 2023` |
| **Kshaya masa** — a lunar month containing two sankrantis, so a month name is skipped entirely | Nothing in the cell | Header renders the two names joined by an en dash: `Pausha–Magha 2124`. Requires `sankranti_count` (§10.1); **not detected by the current back end** | Grid label carries the compound name |
| **Moshier provenance** | Unchanged | Unchanged note, read last | Unchanged |
| **DST-shortened day** | Unchanged — `CivilDay::length()` already handles it, and sunrise is resolved through tzdb | — | — |

Explicitly *not* a degraded case: a lunar month of 29 days rather than 30. The grid is always
6 rows and the panel never resizes (DESIGN §5.1).

---

## 10. Back-end contract

Every field below is new. Types are Rust; the TypeScript mirror in `src/ipc/types.ts` follows
`serde` naming, and `src-tauri/tests/contract.rs` catches drift.

### 10.1 `MoonMonth`

| Field | Type | Meaning |
|---|---|---|
| `grid` | `Vec<MoonCell>` — exactly 42 | The 42 cells the grid draws, in reading order from the first cell of the first week. Replaces the front end's `buildGrid` guesswork: the front end cannot compute which civil days belong to a lunar month, and today it borrows adjacent dates that carry no data |
| `month_name` | `String` | `Shravana` — the name without the `Adhika` prefix, so the header can style the prefix separately |
| `adhika` | `bool` | Intercalary month. Already computed in `lunar.rs::build`, not yet carried on the payload |
| `sankranti_count` | `u8` | Sankrantis inside the month. `0` = adhika, `1` = ordinary, `2` = kshaya masa. Supplying the count rather than two booleans makes the three states mutually exclusive by construction |
| `kshaya_masa_name` | `Option<&'static str>` | The second month name when `sankranti_count == 2`; `None` otherwise. Named for the month, not the tithi — `MoonCell::kshaya` is a different thing entirely |
| `first_paksha` | `Paksha` | Which paksha the month opens with: `Shukla` for amanta, `Krishna` for purnimanta. Derivable from `system`, supplied so the front end holds no calendar rule, and so that the one place the two systems genuinely diverge (§2.5) is a value rather than an `if` |

### 10.2 `MoonCell`

| Field | Type | Meaning |
|---|---|---|
| `in_month` | `bool` | False for the lead and trail cells of `grid`. Drives the `is-outside` dim |
| `tithi` | `u8`, 1..=15 | Tithi number **within its paksha** at the day's reference instant |
| `paksha` | `Paksha` — `shukla` \| `krishna` | Half of the lunar month the tithi belongs to |
| `tithi_index` | `u8`, 1..=30 | **Astronomical** index, always counted from Shukla Pratipada: `⌊elongation / 12⌋ + 1`. `15` = Purnima, `30` = Amavasya, in both systems. It is *not* the tithi's position within the displayed month — amanta and purnimanta count from opposite ends (§2.5), and that ordering is carried by `MoonMonth.first_paksha`. Supplied for sorting and equality, never for display |
| `tithi_name` | `String` | `Ashtami`, `Purnima`, `Amavasya`. Transliteration matches `zodiac.rs` house style — no diacritics |
| `reference` | `Reference` — `sunrise` \| `local_noon` | Which instant the tithi was taken at. `local_noon` only where the Sun does not rise |
| `sunrise` | `Option<Moment>` | The reference sunrise. `None` at polar latitudes |
| `kshaya` | `Vec<SkippedTithi>` | Tithis that begin **and** end inside this civil day, i.e. contain no sunrise. Empty on almost every day. A `Vec` rather than an `Option` because the definition permits more than one, even though one is the only case that occurs. `SkippedTithi { index: u8, name: String }`, where `name` is the full form the spoken label needs — `Shukla Shashthi` |
| `vriddhi` | `Option<Vriddhi>` — `first` \| `second` | This day is the first or the second sunrise of a repeated tithi |

`illumination`, `is_waxing`, `phase`, `principal` and `combust` stay on `MoonCell` unchanged —
solar mode still draws the glyph from them, and the day view still uses them in both modes.

### 10.3 `MoonDay`

| Field | Type | Meaning |
|---|---|---|
| `tithis` | `Vec<TithiSpan>` | Every tithi touching the civil day, in order of occurrence |
| `sunrise` | `Option<Moment>` | Local sunrise. `None` at polar latitudes |
| `reference` | `Reference` | As `MoonCell::reference` |
| `vara` | `u8`, 0..=6 | `0` = Ravivara. Independent of the locale's first day of week |
| `vara_name` | `&'static str` | `Guruvara`. Sanskrit vara name, transliterated without diacritics |

### 10.4 `TithiSpan`

| Field | Type | Meaning |
|---|---|---|
| `index` | `u8`, 1..=30 | Astronomical index, always from Shukla Pratipada, in both systems. As `MoonCell::tithi_index` |
| `number` | `u8`, 1..=15 | Number within the paksha |
| `paksha` | `Paksha` | |
| `name` | `String` | `Ashtami` / `Purnima` / `Amavasya` |
| `entry` | `Option<Moment>` | Instant elongation crossed `12 × (index − 1)` degrees. Never clamped to the day. `None` when Brent did not converge |
| `exit` | `Option<Moment>` | Instant elongation crosses `12 × index` degrees. Same rules |
| `sunrises` | `u8`, 0..=2 | Sunrises contained in the span. The primitive both other states derive from: `0` = kshaya, `2` = vriddhi. Supplying it rather than two booleans makes a contradictory payload unrepresentable |
| `prevailing` | `bool` | In force at the day's reference instant. Exactly one span per day has this true |
| `source` | `Source` | D-006. Never dropped |

### 10.5 Computation notes for the implementer

- Tithi index at an instant: `⌊((λ_moon − λ_sun) mod 360) / 12⌋ + 1`. Verified against
  [RESEARCH.md R-03](../RESEARCH.md): 88.72° / 12 = 7.39 → index 8 → Shukla Ashtami.
- `paksha = if index <= 15 { Shukla } else { Krishna }`; `number = if index <= 15 { index } else { index - 15 }`.
- Cell string, derived in the front end from **`paksha` + `tithi`**, never from `tithi_index`:
  `(Shukla, 15) → "P"`, `(Krishna, 15) → "A"`, otherwise `"S"|"K"` + `tithi`. Deriving it from the
  continuous index is correct in amanta and wrong in purnimanta, where the month counts from
  Krishna Pratipada (§2.5).
- Boundaries are **not** reachable through `spans.rs::divisions_in_day`. That routine samples one
  graha's longitude; a tithi boundary is a crossing of the *difference* of two. It needs a sibling
  that brackets `roots::signed_delta(elongation(t), 12k)` and refines with `roots::refine`
  (D-016), with the Moon's 1h scan step.
- `sunrises` is counted by testing the span `[entry, exit)` against the sunrises of the civil days
  it touches — not by comparing tithi indices between consecutive days, which cannot tell a kshaya
  from a missing computation.
- The 42-cell `grid` costs 12 more day evaluations than the current 29–31. At the measured
  7.9 µs per `swe_calc_ut` call (RESEARCH R-04) this is immaterial against the 30ms cold-month
  target.

---

## 11. Typography and spacing

Every value below already exists in `src/styles/tokens.css`. **No new token is declared** — no colour, no radius, no duration, no font size (DESIGN §13).

| Element | Type role | Token | Colour | Notes |
|---|---|---|---|---|
| Tithi numeral | Numeral | `--type-numeral` (450 13/16) | `--text-primary` | tabular, as everywhere |
| Tithi numeral, today | Numeral | `--type-numeral` | `--accent` | prefix included |
| Tithi numeral, selected | Numeral, weight 600 | `--type-numeral` | `--text-primary` | matches existing `.is-selected` rule |
| Tithi numeral, outside month | Numeral | `--type-numeral` | `--text-tertiary` | |
| Paksha prefix `S` / `K` | Numeral | `--type-numeral` | `--text-tertiary` | no gap; inherits `--accent` on today |
| `P` / `A` (Purnima, Amavasya) | Numeral | `--type-numeral` | `--text-primary` | a name, not a prefix, so it takes the primary colour whole |
| Gregorian day | Micro-caps | `--type-micro` (600 10/12) + `--track-micro` (0.6px) | `--text-tertiary` | uppercase only when it carries `SEP` |
| Gregorian day, outside month | Micro-caps | as above | `--text-tertiary` at 0.45 alpha | matches the 45% dim of the solar adjacent glyph |
| Vara row | Micro-caps | unchanged | `--text-tertiary` | unchanged |
| Day view tithi label | Label | `--type-label` | `--text-secondary` | as nakshatra |
| Day view tithi value | Body | `--type-body` | `--text-primary` | `--text-secondary` for continuation rows |
| Day view tithi caption | Caption | `--type-caption` + `--track-caption` | `--text-tertiary` | includes the kshaya / vriddhi suffix |
| Day view vara name | Micro-caps | unchanged date-line role | `--text-tertiary` | appended after `·` |

Cell box, lunar mode:

| Band | y | Height | Token |
|---|---|---|---|
| pad-top | 0 → 4 | 4 | `--s-2` |
| tithi numeral | 4 → 20 | 16 | line-height of `--type-numeral` |
| gap | 20 → 24 | 4 | `--s-2` |
| Gregorian day | 24 → 36 | 12 | line-height of `--type-micro` |
| pad-bottom | 36 → 40 | 4 | `--s-2` |

Marks:

| Mark | Geometry | Colour |
|---|---|---|
| Kshaya dot | Ø4, top 5, right 6 | `--marker` |
| Vriddhi rule | 40 × 1px, bottom 0, full-bleed | `--marker` |
| Today ring | inset 3 → 34 × 34, r 7, 1px | `--accent` |
| Selection fill | inset 2 → 36 × 36, r 8 | `--surface-selected` |
| Focus ring | inset 1 → 38 × 38, r `--r-focus` | `--focus`, `--focus-width` |

No new colour token is proposed. `--marker` is already at 11.88:1 on `--ground` and 11.04:1 on
`--surface` (DESIGN §4.1), well clear of the 3:1 required of a non-text graphic.

Motion: **nothing new animates.** The tithi numeral and the Gregorian day are replaced, never
tweened (DESIGN §8.2). The kshaya dot and the vriddhi rule join the list of things that never
transition, alongside the event markers and the retrograde rules.

---

## 12. Accessibility

Everything here is keyboard-reachable already: the grid is one tab stop with a roving tabindex,
and no new interactive element is introduced. The day view gains rows, not controls.

### 12.1 Never carried by a mark alone

Extends the DESIGN §11.3 table.

| Meaning | Non-textual carrier | Textual carrier, always present |
|---|---|---|
| Tithi | none — it is text | the numeral itself; named in full in the day view and the spoken label |
| Paksha | none — `S`/`K` is text | named in full (`Shukla` / `Krishna`) in the day view and the spoken label |
| Kshaya | Ø4 `--marker` dot | day view row `… · kshaya, no sunrise`; spoken label names the skipped tithi |
| Vriddhi | 40 × 1px `--marker` rule | day view caption `· first of two sunrises` / `· second sunrise`; spoken label |
| Tithi taken at local noon | **none, by design** | the `--annotate` sentence in the day view, and a suffix on every cell's spoken label |

### 12.2 Spoken label formats

Literal strings. `{}` are placeholders; `[]` are conditional segments appended in the order shown.

**Grid cell, lunar mode** — `role="gridcell"`:

```
[Today, ]{Paksha} {TithiName}, {D} {Month}[, {Phase}, {Pct} percent illuminated][, {KshayaName} is kshaya][, first of two sunrises|, second sunrise of this tithi][, tithi at local noon]
```

Worked examples:

```
Shukla Ashtami, 20 August, waxing gibbous, 68 percent illuminated
Today, Krishna Chaturdashi, 10 September, waning crescent, 4 percent illuminated
Shukla Panchami, 17 August, waxing crescent, 22 percent illuminated, Shukla Shashthi is kshaya
Krishna Tritiya, 31 August, waning gibbous, 74 percent illuminated, second sunrise of this tithi
Shukla Purnima, 27 August, full moon, 100 percent illuminated
Krishna Amavasya, 11 September, new moon, 0 percent illuminated
```

- The phase segment is kept even though the glyph is gone: it is the only spoken carrier of
  illumination, which the tithi does not give.
- Adjacent-month cells are announced identically; `aria-disabled` is not set, because they are
  navigable (DESIGN §5.5).

**Grid** — `role="grid"`:

```
aria-label="{MonthLabel}"        e.g. "Shravana 2026", "Adhika Shravana 2023", "Pausha–Magha 2124"
```

**Day view tithi row** — `role="group"`:

```
Tithi, {Paksha} {TithiName}, {Entry} to {Exit}
Tithi, {Paksha} {TithiName}, {Entry} to {Exit}, kshaya, contains no sunrise
Tithi, {Paksha} {TithiName}, {Entry} to {Exit}, second sunrise of this tithi
Tithi, {Paksha} {TithiName}, {Entry} to, time unavailable
```

**Day view sunrise row** — `role="group"`:

```
Sunrise, {Time}
Sunrise, none, the Sun does not rise on this date
```

**Day view date line:**

```
Thursday, Guruvara
Today, Thursday, Guruvara
```

**Provenance line at polar latitudes** — read last, inside the existing `aria-live="polite"`
region, after every field:

```
No sunrise at this latitude. Tithi taken at local noon.
```

### 12.3 Contrast

No new pair is introduced. The two that are new in *combination*:

| Pair | Ratio | Requirement |
|---|---|---|
| `--text-tertiary` `#85858D` prefix on `--surface-selected` | 4.64 | 4.5 text — pass (already the lowest text pair in the app, DESIGN §11.1) |
| `--marker` `#C8C8CE` on `--surface-selected` | 10.20 | 3.0 non-text — pass |

Under `prefers-contrast: more`, `--text-tertiary` lifts to `#9A9AA2` (7.08 on ground) exactly as
DESIGN §4.2 already specifies. The paksha prefix and the Gregorian day both benefit; no
lunar-specific override is needed.

---

## 13. Open questions

Three, and only three. Each is a genuine fork where a reasonable designer needs the user's
preference rather than an argument.

1. **Era year in the header.** Today the lunar header reads `Shravana 2026` — a Sanskrit month
   name against a Gregorian year. A printed panchang would carry Vikram Samvat (`Shravana 2083`)
   or Shaka Samvat (`Shravana 1948`). Gregorian keeps the anchor to the world the user lives in;
   an era year is what makes the header read as a real panchang. Which — and if an era, which era?
   The two disagree by 135 years and by region.

2. **Tithi in solar mode.** Solar mode is the default and is untouched by this design. A user who
   keeps the Gregorian grid may still want the tithi named in the day view — one extra block, no
   grid change, no D-010 breach beyond the one already taken here. Add it to solar mode's day
   view, or keep solar strictly to the six D-010 fields?

3. **Week start in lunar mode.** The grid follows the macOS locale, so an `en-GB` machine starts
   the week on Monday even in lunar mode. The Government of India's own convention is Sunday-first
   — "Sunday as the first and Saturday as the last day of the week" (india.gov.in, §14) — which is
   the *opposite* of ISO 8601, the rule most European locales report. Force a Sunday start when
   `month_system != "solar"`, or keep the system setting authoritative everywhere?

---

## 14. Sources

Panchang references consulted before designing. Each line states what it supports.

| Source | Supports |
|---|---|
| [mypanchang — Fiji 2013 Hindu Calendar (PDF)](https://www.mypanchang.com/2013pdfs/2013Nadi.pdf) | The `S`/`K` prefix notation and the `P` / `A` substitution for the fifteenth tithi of each paksha. `S` = Shukla, `K` = Krishna, `P` = Pournami, `A` = Amavasya; `S1`–`S14` and `K1`–`K14` carry numbers, the fifteenth carries a name |
| [Drik Panchang — Purushottam Maas / Adhik Maas](https://www.drikpanchang.com/calendars/hindu/months/purushottam/hindu-calendar-adhik-maas.html) | The intercalary month is an extra lunar month inserted roughly every three years to keep the lunar and solar years in step |
| [Drik Panchang — Adhika Jyeshtha Purnima 2026](https://www.drikpanchang.com/purnima/adhika/adhika-purnima-data-time.html?date=31%2F05%2F2026) | Drik Panchang's own label form is `Adhika <Month>`, which is what `lunar.rs::display_name` already produces |
| [Drik Panchang — Purnimanta and Amanta schools](https://www.drikpanchang.com/faq/faq-ans8.html) | The two lunar-month systems and the regional split |
| [Drik Panchang — 2026 Hindu Festivals](https://www.drikpanchang.com/calendars/hindu/hinducalendar.html) | Entry form `Pausha Purnima, January 3, 2026, Saturday, Pausha, Shukla Purnima` — the paksha word is retained alongside the tithi name. Also shows the Vikrama Samvata year (`2082–2083`) beside the Gregorian one, which is the basis of open question 1 |
| [Drik Panchang — 2026 Vaishnava (ISKCON) calendar](https://www.drikpanchang.com/iskcon/iskcon-month-calendar.html) | ISKCON calendars carry tithi with an end time, plus paksha, nakshatra, yoga, karana and vara, and present purnimanta masas inside Gregorian month views |
| [ISVARA — Purushottama Adhika Masa](https://www.isvara.org/archive/purushottama-adhika-masa/) | `Purushottama Adhika Masa`, `Adhika <Month>`, and `Mala Masa` are the same month under different names — the reason this design uses only the calendrical `Adhika` |
| [Wikipedia — Adhika-masa](https://en.wikipedia.org/wiki/Adhika-masa) | The no-sankranti rule for detecting an intercalary month, and the existence of kshaya masa (two sankrantis in one lunar month) |
| [GeoTimeDate — More about Paksha, Tithi and Karana](https://geotimedate.org/articles/panchang/more-about-paksha-tithi-and-karana) | Tithi length varies between roughly 19 and 26 hours, which is what makes kshaya and vriddhi possible; the udaya (sunrise) tithi rule assigns each civil day one unambiguous tithi |
| [Astrobix — What is Tithi Vridhi and Tithi Kshaya?](https://astrobix.com/hindupath/408-what-is-tithi-vridhi-and-tithi-kshaya.html) | Definitions used verbatim in §5: vriddhi = a tithi spanning two sunrises, repeated; kshaya = a tithi beginning and ending between two sunrises, skipped |
| [Purnimanta vs Amanta — month name shift](https://monthnameshindi.com/purnimanta-vs-amanta-indian-calendar-difference/) | The purnimanta month name runs one paksha ahead: amanta Phalguna Amavasya is purnimanta Chaitra Amavasya. Amanta orders Shukla then Krishna; purnimanta orders Krishna then Shukla |
| [ZODIAQ — North vs South Indian panchang](https://www.myzodiaq.in/en/online-library/panchang/regional-panchang/north-vs-south-indian-panchang-core-differences-and-understanding) | The state-by-state split: purnimanta in Bihar, Chhattisgarh, Haryana, HP, J&K, Jharkhand, MP, Odisha, Punjab, Rajasthan, Uttarakhand, UP; amanta in Andhra, Assam, Gujarat, Goa, Karnataka, Kerala, Maharashtra, Tamil Nadu, Telangana, Tripura, West Bengal |
| [Sewell & Dikshit, *The Indian Calendar* (1896)](https://archive.org/details/indiancalendarwi00sewerich) | The oldest authoritative statement of the numbering rule. §29: "There are 30 tithis in a lunar month, 15 to each fortnight… these are used for the fifteen tithis of each fortnight," and "The numeral 30 is generally applied to the amavasya… even in Northern India." §51: "the bright fortnights have the same name by both schemes while the dark fortnights differ by a month… the purnimanta scheme is always a fortnight in advance." Also reproduces an 1894 Poona panchang laid out by paksha — the almanac-book form discussed in §3.2 |
| [Wikipedia — Tithi](https://en.wikipedia.org/wiki/Tithi) | Ordering differs by scheme: amanta counts tithis from Shukla Pratipada, purnimanta from Krishna Pratipada. This is why the cell string is derived from paksha + number and not from a continuous index (§2.1, §10.5) |
| [Kalnirnay](https://www.kalnirnay.com/) | India's best-selling almanac, Marathi, amanta. **Transposed grid** — 7 weekday rows × 5 week columns, weekday rail in single Devanagari aksharas `र सो मं बु गु शु श`. Runs `Krishna 14 → Krishna 30 → Shukla 1` |
| [Thakur Prasad Panchang 2026](https://thakurprasad.com/february/) | Hindi, purnimanta, published since 1930. The same transposed 7 × 5 grid with `रवि/SUN … शनि/SAT` badges — two independent best-selling calendars, opposite month systems, same layout. Supplies the February 2026 side-by-side that demonstrates the one-fortnight offset (§2.5) |
| [Rashtriya Panchang (Government of India)](https://mausam.imd.gov.in/imd_latest/contents/rashtriy_panchang.php) · [archive](https://archive.org/details/rAShTriya-panchAnga-archive) | Day-per-block almanac, no grid. Confirms the canonical field order and the end-time-only convention: `Tithi: (Phalguna Krishna) Dvadasi h. 7-56 · Nakshatra: Dhanistha h. 19-40 · Yoga: Siddha h. 12-22 · Karana: Taitila h. 7-56 then Gara h. 21-01` |
| [Britannica — Hindu calendar](https://www.britannica.com/topic/Hindu-calendar) | Amanta/purnimanta state list including Goa and Telangana; uses the `-vasara` vara suffix (`Ravivasara`…), "the planet followed by the word for 'day of the week': *vasara* or *vara*" |
| [Wikipedia — Hindu calendar](https://en.wikipedia.org/wiki/Hindu_calendar) | The competing state list: excludes Assam, West Bengal, Odisha, Tamil Nadu and Kerala from amanta because they keep their own solar calendars. The disagreement is recorded in §2.5 |
| [india.gov.in — National Calendar](https://www.india.gov.in/calendar) via [Wikipedia — Indian national calendar](https://en.wikipedia.org/wiki/Indian_national_calendar) | "Sunday as the first and Saturday as the last day of the week" — the Government of India convention, opposite to ISO 8601. Basis of open question 3. Also gives `Brihaspativara` as the primary Thursday vara name with `Guruvara` as the alternate |
| [ISKCON Bangalore — Vaishnava Calendar 2026–27](https://www.iskconbangalore.org/vaishnava-calendar/) | A published Vaishnava calendar in the same 7-column weekday form |
| [RESEARCH.md R-03](../RESEARCH.md) — this repo | The verified worked example the whole numeral rests on: 20 Aug 2026, Bengaluru, elongation 88.72° → tithi 8 → Shukla Ashtami, cross-checked against an independent expectation |

Two things the research changed, recorded rather than quietly absorbed:

- **The transposed grid is the print norm, not an outlier.** An earlier draft of §3 asserted that
  every panchang is a 7-column weekday grid. That is true of the digital ones and false of the two
  best-selling printed ones, which put weekdays on rows. §3.1 now rejects the transposed layout on
  geometry — 7 × 40px = 280px against a 264px region — and not on a claim about convention.
- **A paksha-blocked layout does exist**, in the almanac-book genre rather than the wall-calendar
  genre. §3.2 rejects it on the kshaya/vriddhi argument, which is unaffected, and no longer on the
  false claim that nobody uses it.

Everything above was checked against a published source. Where a layout could only be confirmed
from a description or a legend rather than from the printed grid — the scanned PDFs and image-only
calendars — the claim is limited to what the text actually supports.

