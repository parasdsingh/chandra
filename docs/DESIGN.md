# Design specification

Status: **awaiting approval.** Implements the UI layer of [ARCHITECTURE.md](ARCHITECTURE.md) §8
under [D-011](DECISIONS.md#d-011), [D-008](DECISIONS.md#d-008), [D-009](DECISIONS.md#d-009),
[D-010](DECISIONS.md#d-010), [D-014](DECISIONS.md#d-014).

Every value here is literal. `px` means CSS px in the panel; `pt` means macOS points in the
menu bar. Panel renders at the display scale factor; menu bar assets are authored at 2x.

---

## 1. Principles

Six. Each one decides a real conflict later in this document.

1. **One surface, one job.** The menu bar answers *what phase is it*. The grid answers *what
   month*. The detail answers *what about this day*. No surface answers two questions, and
   nothing is duplicated across two of them.
2. **Ma is a measured quantity.** Negative space is specified in px like everything else
   (§2 scale) and is never reclaimed to fit content. If content does not fit, content is cut,
   not the space.
3. **Delete before you style.** Every element must justify its own ink. There are no borders
   that could be a gap, no labels that could be a column position, no icons that could be a word.
4. **One hue, one job.** Gold means *today*. Nothing else in the app is chromatic. Every
   other distinction is made by position, weight, or form.
   *Revised.* A second hue, `--retro`, carried retrograde motion until D-020. A colour has to
   be learnt, it is the first thing a grayscale or colour-blind rendering loses, and it made
   the graha symbol carry two meanings at once. Retrograde is now written `℞` beside the
   symbol, which is how every printed ephemeris writes it and needs no key.
5. **Motion explains geometry, never decorates it.** The only things that move are things that
   changed size or position. Nothing loops, pulses, shimmers, or overshoots.
6. **Numbers never jitter.** Tabular numerals everywhere, fixed grid height regardless of
   month length, fixed panel width forever. A value changing must not move a value next to it.

---

## 2. Dimensional system

Base unit **4px**. `2px` is the only permitted sub-unit and exists solely for hairline offsets
and focus insets.

### 2.1 Spacing scale

| Token | px | Used for |
|---|---|---|
| `--s-1` | 2 | focus inset, hairline offset |
| `--s-2` | 4 | intra-component gap (numeral → glyph) |
| `--s-3` | 8 | gap between related rows |
| `--s-4` | 12 | panel top/bottom padding, block gap |
| `--s-5` | 16 | detail area padding |
| `--s-6` | 20 | panel left/right padding |
| `--s-7` | 24 | field row height |
| `--s-8` | 32 | — reserved, settings window only |
| `--s-9` | 40 | day cell, header height |

### 2.2 Panel box

| Property | Value |
|---|---|
| Width | **320px**, fixed, never animates, never responsive |
| Content width | 280px (320 − 2 × 20) |
| Min height | **332px** (collapsed, any panel) |
| Max height | **620px** (graha panel, expanded, 3 events + provenance note) |
| Corner radius | 12px |
| Border | 1px solid `rgba(255,255,255,0.12)`, inside the radius |
| Background | `#0A0A0B`, fully opaque. No vibrancy, no blur, no material (D-011) |
| Shadow | `0 12px 32px rgba(0,0,0,0.55), 0 2px 6px rgba(0,0,0,0.40)` |
| Chrome | none — no titlebar, no traffic lights, no beak/arrow |
| Position | horizontally centred on the tray item, top edge 6px below the menu bar, clamped to 12px from either screen edge |

### 2.3 Radii

| Token | px | Applies to |
|---|---|---|
| `--r-panel` | 12 | panel, settings window content |
| `--r-cell` | 8 | day cell fill, month picker cell |
| `--r-button` | 6 | header buttons |
| `--r-chip` | 4 | `℞` chip |
| `--r-focus` | 9 | focus ring on a 40px cell (`--r-cell` + 1px inset) |

### 2.4 Hairlines

- Separator inside the panel: 1px `rgba(255,255,255,0.08)`, full-bleed to the panel edge
  (x = 0 → 320), **not** inset to the content width. It divides the panel, not the content.
- Panel border: 1px `rgba(255,255,255,0.12)`.
- There is no second-level separator. Groups are separated by space (§2.1), not by rules.

---

## 3. Type

### 3.1 Stack

```css
font-family: -apple-system, "SF Pro Text", system-ui, "Helvetica Neue", sans-serif;
```

`-apple-system` resolves to SF Pro Text below 20px, which is the only size band used here.

### 3.2 The tabular rule

```css
:root { font-variant-numeric: tabular-nums; }
```

Set once on `:root`. **Never overridden anywhere.** Applies to date numerals, times,
percentages, longitudes, speeds, year values, and settings fields. Proportional figures are
forbidden in this app; a time changing from `09:11` to `19:08` must not shift a single pixel.

### 3.3 Scale

| Role | Size | Weight | Line height | Tracking | Colour | Used by |
|---|---|---|---|---|---|---|
| Headline | 17px | 600 | 22px | −0.20px | primary | phase name, graha name in detail |
| Title | 15px | 600 | 20px | −0.10px | primary | header subject + month, settings section headers |
| Numeral | 13px | 450 | 16px | 0 | primary | day-cell date numerals |
| Body | 13px | 400 | 18px | 0 | primary | field values, buttons |
| Label | 12px | 400 | 16px | 0 | secondary | field labels |
| Caption | 11px | 400 | 14px | +0.10px | tertiary | entry/exit times, annotations, event lines |
| Micro-caps | 10px | 600 | 12px | +0.60px | tertiary | weekday header row, uppercase |

Notes:

- Weight `450` for the day numeral, not 400: at 13px on a near-black ground SF renders
  optically light, and `450` matches the perceived weight of Body text on `#141417`.
- Micro-caps is the only uppercase role. Nothing else is ever uppercased.
- No italics anywhere. No underlines except the combustion marker (§6.3), which is a rule, not
  text decoration.
- Text never wraps in the panel. Long values are handled by the abbreviation rules in §5.2 and
  §6.4, never by ellipsis, except city names in settings search results.

---

## 4. Colour tokens

`src/styles/tokens.css` is the single source of truth (D-014). Contrast computed by WCAG 2.1
relative luminance; **every** text token clears 4.5:1 on **every** background it can appear on,
so no token depends on the large-text 3:1 allowance.

```css
:root {
  /* surfaces */
  --ground:            #0A0A0B;
  --surface:           #141417;
  --surface-selected:  #1C1C20;
  --surface-hover:     rgba(255,255,255,0.04);   /* composites to #141415 */

  /* text */
  --text-primary:      #EDEDEF;
  --text-secondary:    #A1A1A8;
  --text-tertiary:     #85858D;

  /* lines */
  --hairline:          rgba(255,255,255,0.08);   /* composites to #1E1E1F */
  --border:            rgba(255,255,255,0.12);   /* composites to #272728 */

  /* meaning — exactly two hues */
  --accent:            #D8B36A;   /* TODAY ONLY. Never used for anything else. */

  /* graphics */
  --marker:            #C8C8CE;   /* graha event markers */
  --disc-lit:          #D4D4D9;   /* lit fraction of the in-panel phase glyph */
  --disc-ring:         rgba(255,255,255,0.22);   /* composites to #404041 */

  /* interaction */
  --focus:             #EDEDEF;   /* 2px ring, :focus-visible only */

  /* annotation */
  --annotate:          #A1A1A8;   /* == --text-secondary, deliberately */

  /* shadow */
  --shadow-panel: 0 12px 32px rgba(0,0,0,0.55), 0 2px 6px rgba(0,0,0,0.40);
}
```

### 4.1 Contrast

| Token | Hex | vs `--ground` | vs `--surface` | vs `--surface-selected` | AA body (4.5:1) |
|---|---|---|---|---|---|
| `--text-primary` | `#EDEDEF` | **16.93** | 15.73 | 14.53 | pass |
| `--text-secondary` | `#A1A1A8` | **7.71** | 7.16 | 6.62 | pass |
| `--text-tertiary` | `#85858D` | **5.41** | 5.02 | 4.64 | pass |
| `--accent` | `#D8B36A` | **9.97** | 9.26 | 8.55 | pass |
| `--marker` | `#C8C8CE` | **11.88** | 11.04 | 10.20 | pass (non-text; 3:1 needed) |
| `--disc-lit` | `#D4D4D9` | **13.40** | 12.45 | 11.50 | pass (non-text; 3:1 needed) |
| `--focus` | `#EDEDEF` | **16.93** | 15.73 | 14.53 | pass (non-text; 3:1 needed) |
| `--annotate` | `#A1A1A8` | **7.71** | 7.16 | 6.62 | pass |
| `--hairline` | `#1E1E1F` | 1.19 | 1.10 | 1.02 | decorative — carries no meaning |
| `--border` | `#272728` | 1.33 | 1.16 | 1.07 | decorative — carries no meaning |
| `--disc-ring` | `#404041` | 2.42 | 2.24 | 2.07 | decorative — see §5.4 |

Rules that follow from the table:

- `--hairline` and `--border` are below 3:1 by design. Neither ever carries information: every
  separator has content on both sides that stands on its own, and the panel edge is established
  by the shadow, not the border. Removing them must not make the UI ambiguous.
- `--disc-ring` at 2.42:1 is the *in-panel* glyph outline only, and only for the new-moon case.
  A new-moon cell is additionally identified by the date numeral, the day detail wording, and
  the VoiceOver label, so the ring is never the sole carrier (§11.3).
- There is **no error hue.** Errors are rendered with `--text-primary` on `--surface` (§9.3).
  A red inside a 320px popover with one message adds alarm without adding findability.
- There is **no warning hue.** Degraded states are facts, not faults (§9.4).

### 4.2 Increased contrast

When `prefers-contrast: more`:

| Token | Normal | Increased | Ratio vs ground |
|---|---|---|---|
| `--text-tertiary` | `#85858D` | `#9A9AA2` | 5.41 → **7.08** |
| `--text-secondary` | `#A1A1A8` | `#B4B4BB` | 7.71 → **9.60** |
| `--hairline` | `rgba(255,255,255,0.08)` | `rgba(255,255,255,0.18)` | 1.19 → 1.64 |
| `--disc-ring` | `rgba(255,255,255,0.22)` | `rgba(255,255,255,0.44)` | 2.42 → 4.60 |
| `--focus` ring width | 2px | 3px | — |

`--accent` is unchanged; it already exceeds 7:1.

---

## 5. Panel layout — moon calendar

### 5.1 Wireframe (collapsed, 320 × 332)

```
 x:0      20                                            300     320
      ┌────────────────────────────────────────────────────────┐  y:0
      │                                                        │  12   pad-top
      │  ◐  Chandra · August 2026            [ ‹ ][ › ]  [ ⚙ ] │  40   header
      │                                                        │   4
      │   SUN   MON   TUE   WED   THU   FRI   SAT              │  20   weekday row
      │                                                        │   4
      │  ┌────┬────┬────┬────┬────┬────┬────┐                  │
      │  │ 27 │ 28 │ 29 │ 30 │ 31 │  1 │  2 │  40             │
      │  │ ◗  │ ◗  │ ●  │ ●  │ ●  │ ◖  │ ◖  │                  │
      │  ├────┼────┼────┼────┼────┼────┼────┤                  │
      │  │  3 │  4 │  5 │  6 │  7 │  8 │  9 │  40             │  240  grid
      │  ├────┼────┼────┼────┼────┼────┼────┤                  │  6 × 40
      │  │ 10 │ 11 │ 12 │ 13 │ 14 │ 15 │ 16 │  40             │  always 6 rows
      │  ├────┼────┼────┼────┼────┼────┼────┤                  │
      │  │ 17 │ 18 │ 19 │ 20 │ 21 │ 22 │ 23 │  40             │
      │  ├────┼────┼────┼────┼────┼────┼────┤                  │
      │  │ 24 │ 25 │ 26 │ 27 │ 28 │ 29 │ 30 │  40             │
      │  ├────┼────┼────┼────┼────┼────┼────┤                  │
      │  │ 31 │  1 │  2 │  3 │  4 │  5 │  6 │  40             │
      │  └────┴────┴────┴────┴────┴────┴────┘                  │
      │                                                        │  12   pad-bottom
      └────────────────────────────────────────────────────────┘  y:332
        └── 7 × 40 = 280 ──┘
```

**The grid is always 6 rows.** A 4-week February renders a 6th row of trailing days rather
than shrinking. Panel height in the collapsed state is therefore a constant 332px, and the
menu bar item never opens a differently-sized window month to month.

### 5.2 Header (40px)

```
 x:20         36                                 214  242      276    300
  ┌───────────┬─────────────────────────────────┬─────┬────────┬───────┐
  │  subject  │  "Chandra · August 2026"        │  ‹  │   ›    │   ⚙   │
  │  16 × 16  │  Title 15/600/-0.10             │24×24│ 24×24  │ 24×24 │
  └───────────┴─────────────────────────────────┴─────┴────────┴───────┘
   glyph        gap 8                            gap 4  gap 10
```

| Element | Box | Hit area | Notes |
|---|---|---|---|
| Subject glyph | 16 × 16 at x = 20, vertically centred | — | live moon disc for the moon panel; the graha template glyph for graha panels, drawn in `--text-primary` |
| Subject + month label | text button, x = 44 → 210 (166px) | 28px tall, full text width | opens the month picker (§5.7) |
| `‹` prev | 24 × 24 at x = 214 → 238, centre 226 | 28 × 28 | |
| `›` next | 24 × 24 at x = 242 → 266, centre 254 | 28 × 28 | |
| `⚙` settings | 24 × 24 at x = 276 → 300, centre 288 | 28 × 28 | opens the settings window |

Hit areas are 28 × 28 centred on each 24 × 24 visual box, so `‹` and `›` hit areas abut at
x = 240 and the `⚙` hit area overhangs the 300px content edge by 2px into the panel padding.

Button glyphs: 1.5px stroke, round cap/join, `--text-secondary`; `--text-primary` on hover;
opacity 0.6 while pressed.

**Truncation rule (deterministic, no ellipsis):** if the label string renders wider than 166px,
re-render with the month abbreviated (`Chandra · Sep 2026`). If it still exceeds 166px, drop
the subject name and render `Sep 2026` alone — the subject glyph at x = 20 still identifies the
panel. This never happens for any of the nine names in English, and is specified so that a
localisation cannot silently produce an ellipsis.

### 5.3 Weekday row (20px)

- 7 columns of 40px, label centred in each.
- Micro-caps 10/600/+0.60px, `--text-tertiary`, uppercase.
- Two-letter labels are **not** used. Three letters, locale-provided short names.
- First day of week comes from the macOS locale (`AppleFirstWeekday`), not hardcoded.

### 5.4 Day cell anatomy (40 × 40)

```
        x:0                    20                   40
    y:0  ┌───────────────────────────────────────────┐
         │                                           │   4   pad-top
    y:4  │                  ┌───┐                    │
         │                  │ 21│  numeral, centred  │  16   Numeral 13/450/16
   y:20  │                  └───┘                    │
         │                                           │   2   gap
   y:22  │                 ┌──────┐                  │
         │                 │  ◗   │  phase glyph     │  14   Ø14, centre (20,29)
   y:36  │                 └──────┘                  │
         │                                           │   4   pad-bottom
   y:40  └───────────────────────────────────────────┘
```

| Relation | Value |
|---|---|
| Numeral cap-height (SF Text 13px) | 9.3px |
| Phase glyph diameter | 14px |
| Glyph Ø : numeral cap-height | **1.51 : 1** |
| Numeral optical centre | (20, 12) |
| Glyph geometric centre | (20, 29) |
| Vertical gap, numeral baseline to glyph top | 5px |

The glyph is the larger element and sits below. The date is the label; the phase is the content.

**Phase glyph construction (in-panel, Ø14 at r = 7):**

- Ring: 1px `--disc-ring`, centred on r = 6.5 (spans 6.0 → 7.0).
- Lit region: filled `--disc-lit`.
- Terminator: half-ellipse, semi-major = 7 (vertical), semi-minor = 7 × (1 − 2k) where
  k = illuminated fraction. k < 0.5 bulges into the dark side (crescent); k > 0.5 bulges into
  the lit side (gibbous). Same maths as the tray disc (§7.1), same source value.
- k < 0.02 → ring only, at `rgba(255,255,255,0.34)`.
- k > 0.98 → solid fill, ring omitted.
- Lit side: right when waxing, left when waning. Mirrored when the resolved latitude < 0.

### 5.5 Cell states

Painted in this order; they stack.

| State | Treatment | Geometry |
|---|---|---|
| Normal | numeral `--text-primary`, glyph `--disc-lit` | — |
| Adjacent month | numeral `--text-tertiary`, glyph at 45% alpha | clickable; click navigates to that month and selects the day |
| Hover (pointer) | fill `--surface-hover` | inset 2 → 36 × 36, r 8 |
| Selected | fill `--surface-selected` | inset 2 → 36 × 36, r 8 |
| Today | 1px `--accent` ring **and** numeral in `--accent` | inset 3 → 34 × 34, r 7 |
| Today + selected | both — the ring sits 1px inside the fill edge | — |
| Focused (`:focus-visible`) | 2px `--focus` ring | inset 1 → 38 × 38, r 9 |

- Today is marked twice (ring **and** numeral colour) so it survives a colour-blind or
  grayscale rendering — the ring is a form, the colour is a reinforcement.
- Focus is drawn outside the selection fill with a 1px dark gap, so selection and focus are
  never mistaken for one another when both are present.
- Focus ring is `:focus-visible` only. Pointer clicks never draw it.

### 5.6 Expanded detail — the six v1 fields

Detail appears **below** the grid, inside the panel, above the bottom padding. Divider is
full-bleed 1px `--hairline` at x = 0 → 320.

```
      ┌────────────────────────────────────────────────────────┐
      │  ...grid...                                            │
      │                                                        │  12  pad
      ├────────────────────────────────────────────────────────┤   1  hairline, full bleed
      │                                                        │  16  pad-top
      │  THURSDAY 20 AUGUST 2026                               │  14  Micro-caps, tertiary
      │                                                        │   4
      │  Waxing Gibbous                            68.4% lit   │  24  Headline | Body, secondary
      │                                                        │  12
      │  Moonrise                                     15:42    │  24  Label | Body, tnum
      │  Moonset                                      02:11    │  24
      │                                                        │  12
      │  Nakshatra                    Purva Ashadha · Pada 2   │  24  Label | Body
      │                               04:31 → 07:12 (21 Aug)   │  18  Caption, tertiary, tnum
      │                                                        │   8
      │  Rashi                                      Dhanu      │  24  Label | Body
      │                               19 Aug 11:04 → 22:47     │  18  Caption, tertiary, tnum
      │                                                        │  16  pad-bottom
      └────────────────────────────────────────────────────────┘
       20 ├──────────── 280 ────────────┤ 20
```

Field row geometry (all rows, 24px tall, content width 280):

- Label: left-aligned at x = 20, Label 12/400, `--text-secondary`.
- Value: **right-aligned** to x = 300, Body 13/400, `--text-primary`.
- Right alignment is what makes tabular numerals pay off: `15:42` and `02:11` share a right
  edge and every digit column lines up down the block.
- Sub-caption (entry/exit): right-aligned to x = 300, Caption 11/400, `--text-tertiary`.
- Entry/exit format: `HH:MM → HH:MM`. A boundary instant outside the selected day carries a
  date prefix or suffix (`19 Aug 11:04 →` / `→ 07:12 (21 Aug)`). Never clamped to the day
  (`Span` in ARCHITECTURE §3.2 reports the true instant).
- Time format 12h/24h from settings; 24h shown throughout this document.

Field-to-source mapping (exactly six, D-010):

| # | Field | Row | Source |
|---|---|---|---|
| 1 | Phase name | headline, left | `DayDetail.phase.name` |
| 2 | Illuminated fraction | headline, right, 1 decimal + `% lit` | `DayDetail.phase.illumination` |
| 3 | Moonrise | field row | `DayDetail.moonrise` |
| 4 | Moonset | field row | `DayDetail.moonset` |
| 5 | Nakshatra + pada, entry/exit | field row + caption | `DayDetail.nakshatra: Span<Nakshatra>` |
| 6 | Rashi, entry/exit | field row + caption | `DayDetail.rashi: Span<Rashi>` |

Nothing else. No tithi, no yoga, no karana, no muhurta, no ayanamsa readout, no coordinates.

### 5.6.1 Height budget

| Block | Collapsed | Expanded |
|---|---|---|
| pad-top | 12 | 12 |
| header | 40 | 40 |
| gap | 4 | 4 |
| weekday row | 20 | 20 |
| gap | 4 | 4 |
| grid | 240 | 240 |
| pad / gap to divider | 12 | 12 |
| divider | — | 1 |
| detail | — | 238 |
| **Total** | **332** | **571** |
| + provenance note (§9.4) | — | +24 → **595** |

### 5.7 Month picker

Opened by clicking the header label. **Replaces the grid region only; panel height stays 332.**
This is the reason the picker is 4 × 3 and not a scrolling list.

```
      │  ‹            2026            ›                        │  40   year stepper
      │                                                        │  10
      │  ┌──────┬──────┬──────┬──────┐                         │
      │  │ Jan  │ Feb  │ Mar  │ Apr  │  60                     │
      │  ├──────┼──────┼──────┼──────┤                         │  180
      │  │ May  │ Jun  │ Jul  │ Aug  │  60                     │
      │  ├──────┼──────┼──────┼──────┤                         │
      │  │ Sep  │ Oct  │ Nov  │ Dec  │  60                     │
      │  └──────┴──────┴──────┴──────┘                         │
      │      └─ 4 × 70 = 280 ─┘                                │  10
```

- Month cell 70 × 60, label Body 13/400 centred.
- Displayed month: fill `--surface-selected`, inset 4 → 62 × 52, r 8.
- Current real month: label in `--accent`.
- Year stepper: `‹` / `›` 24 × 24 at x = 20 and x = 276; year Title 15/600 centred, tabular.
- The weekday row is hidden while the picker is open. The header label stays and toggles back.

---

## 6. Panel layout — graha calendar

Identical shell: same 320 width, same 40px header, same weekday row, same 6 × 40 grid, same
divider, same field-row geometry. Only cell content and detail fields differ.

### 6.1 What changes

| Element | Moon panel | Graha panel |
|---|---|---|
| Header glyph | live moon disc, Ø16 | graha template glyph, 16 × 16, `--text-primary` |
| Header label | `Chandra · August 2026` | `Mangala · August 2026` |
| Cell lower half | phase glyph Ø14 | event marker row |
| Row-level mark | none | retrograde span rule |
| Detail | six phase/panchanga fields | longitude, speed, retrograde state, rise, set |

### 6.2 Cell anatomy (40 × 40)

```
    y:0  ┌───────────────────────────────────────────┐
         │                                           │   4
    y:4  │                  ┌───┐                    │
         │                  │ 21│                    │  16   Numeral 13/450
   y:20  │                  └───┘                    │
         │       ──────────                          │   2   combustion rule (y=21.5)
   y:24  │            ●   ○   ◆                      │  10   marker row, centre y=29
   y:34  │                                           │
   y:38  │ ───────────────────────────────────────── │   1   retrograde span rule (y=37.5)
   y:40  └───────────────────────────────────────────┘
```

### 6.3 Event marking

Four event kinds, distinguished by **form first**, colour second. Markers sit on one centred
row, 5px each, 5px apart, maximum **3**.

| Event | Mark | Geometry | Colour |
|---|---|---|---|
| Sign ingress | filled disc | Ø5 | `--marker` |
| Nakshatra ingress | ring | Ø5, 1px stroke, Ø3 hole | `--marker` |
| Retrograde station | diamond | 5 × 5 square rotated 45° | `--marker` |
| Combustion (day is combust) | rule | 12 × 1px, centred, at y = 21.5 directly under the numeral | `--marker` |
| Retrograde (day is retrograde) | `℞` + span rule | `℞` at 9px beside the symbol, absolutely placed so the symbol stays on the column's centre line; span rule 40 × 1px full-bleed at y = 37.5, `--marker` at 0.5 alpha |
| More than 3 markers | `+` | 5 × 5, two 1px crossing strokes, replaces the 3rd | `--marker` |

Why this reads without clutter:

- Combustion and retrograde are **spans**, not instants, so they are drawn as rules at cell
  edges, not as markers. A retrograde period becomes one continuous 1px line running
  across whole weeks — visible at a glance, invisible when you are not looking for it.
- Instants (ingress, station) are the only things in the marker row, so most cells carry
  zero or one mark.
- The marker row occupies 10px of a 40px cell. In a typical month fewer than 8 of 42 cells
  carry a marker.
- Every marker is named in words in the day detail (§6.4). Shape and colour are never the
  sole carrier.

### 6.4 Day detail

```
      ├────────────────────────────────────────────────────────┤   1  hairline
      │                                                        │  16  pad-top
      │  THURSDAY 20 AUGUST 2026                               │  14  Micro-caps, tertiary
      │                                                        │   4
      │  Mangala                                       [ ℞ ]   │  24  Headline | chip
      │                                                        │  12
      │  Longitude                            118° 42′ 07″     │  24  Label | Body, tnum
      │  Speed                                −0.0142 °/day    │  24  Label | Body, tnum
      │                                                        │  12
      │  Rise                                        21:07     │  24
      │  Set                                         09:33     │  24
      │                                                        │  12   ┐
      │  Enters Simha                                14:02     │  18   │ events,
      │  Enters Magha                                14:02     │  18   │ 0–3 rows
      │  Retrograde station                          06:19     │  18   ┘
      │                                                        │  16  pad-bottom
      └────────────────────────────────────────────────────────┘
```

| Element | Spec |
|---|---|
| `℞` chip | 22 × 16, r 4, 1px `--text-tertiary` border, `℞` in Body 13/400 `--text-secondary`, right-aligned to x = 300. Present only when `speed < 0`. |
| Longitude | sidereal, `DDD° MM′ SS″`, zero-padded, tabular. Right-aligned to x = 300. |
| Speed | 4 decimals, explicit sign, ` °/day`. Negative is shown as `−0.0142` (U+2212 minus, not hyphen) and is reinforced by the chip. |
| Event rows | Caption 11/400 `--text-secondary` left at x = 20, time Caption `--text-tertiary` right to x = 300. |
| Event wording | `Enters <Rashi>` · `Enters <Nakshatra>` · `Retrograde station` · `Direct station` · `Combust` · `Leaves combustion` |
| Combustion (span, no instant today) | rendered as an event row with `—` in the time column instead of a time |

### 6.4.1 Height budget

| Block | Collapsed | Expanded, 0 events | Expanded, 3 events |
|---|---|---|---|
| shell (header → grid → pad) | 332 | 332 | 332 |
| divider | — | 1 | 1 |
| detail | — | 194 | 260 |
| **Total** | **332** | **527** | **593** |
| + provenance note | — | — | +24 → **617** |

Max panel height across the whole app is therefore **617px**, rounded in §2.2 to a 620px cap.

---

## 7. Menu bar glyphs

### 7.1 The moon disc — live-rendered

Drawn per day in `crates/glyph` with `tiny-skia`, applied via
`set_icon_with_as_template(true)` (D-008).

| Property | Value |
|---|---|
| Canvas | 22 × 22 pt = **44 × 44 px** at 2x |
| Disc centre | (11.0, 11.0) pt |
| Disc radius `R` | **7.2 pt** (Ø14.4pt in a 22pt slot = 65% fill) |
| Ring stroke | **1.3 pt**, centred on `R` |
| Ring alpha | **0.90** normally; **1.00** at new moon |

Revised after comparing the rendered icon against the system status icons beside it. At
R = 8.0 with a 0.55 ring the moon read both larger and dimmer than the wifi and battery
glyphs: macOS masks a template image with its own foreground colour, so a half-transparent
ring comes out grey next to fully opaque system icons. 7.2 pt at near-full opacity matches
their optical size and weight.
| Lit fill alpha | **1.00** |
| Anti-aliasing | on; no hinting; no gamma correction |
| Vertical placement | geometric centre of the 22pt slot; no optical offset (the form is radially symmetric) |

**Full-disc hairline ring: yes, always, except at full moon.** Without it a new moon renders
zero pixels and the menu bar item becomes an invisible click target. The ring is the affordance;
the fill is the data.

**Terminator geometry.** Let `k` = illuminated fraction ∈ [0, 1].

```
semi-minor axis  a = R · (1 − 2k)          (signed)
terminator x(y)  = ±a · sqrt(1 − (y/R)²)
```

- `k < 0.5` → `a > 0` → terminator bows toward the dark side → crescent.
- `k > 0.5` → `a < 0` → terminator bows toward the lit side → gibbous.
- `k = 0.5` → `a = 0` → straight terminator → exact half disc.

The lit region is one closed path: the limb semicircle on the lit side, plus the terminator
half-ellipse. Each half is 2 cubic Béziers (one per quadrant) with the handle constant
**k = 0.5523** scaled independently on each axis (`R` on the major, `|a|` on the minor).

**Orientation.**

- Lit side = **right** when waxing, **left** when waning.
- Waxing/waning determined from `(λ_moon − λ_sun) mod 360`: `[0, 180)` → waxing,
  `[180, 360)` → waning. Never from a date difference.
- If the resolved latitude < 0, mirror the whole icon about x = 11.0.

**Thresholds.**

| Condition | Render |
|---|---|
| `k < 0.02` | ring only at alpha 0.78, no fill |
| `k > 0.98` | solid disc at alpha 1.00, ring omitted |
| otherwise | lit fill at alpha 1.00 + ring at alpha 0.55 |

**Refresh.** Redrawn on local-midnight rollover only (D-017). The value used is the
illumination at local noon of the current date — the same value the grid cell uses, so the
menu bar and the grid can never disagree.

**Colour mode** (settings, off by default): lit fill `#EDEDEF`, ring `rgba(237,237,239,0.55)`,
applied with `set_icon_with_as_template(false)`. Geometry is unchanged.

### 7.2 Graha template glyphs

Authored on a **24 × 24 unit grid**. Rendered into the 22 × 22 pt canvas at uniform scale
`22 / 24 = 0.916667`, no translation.

| Property | Value |
|---|---|
| Design grid | 24 × 24 units, origin top-left, y down |
| Optical cap-height target | 18 units → **16.5 pt** rendered |
| Stroke width | **1.8 units** → 1.65 pt → 3.30 px at 2x |
| Line cap / join | round / round |
| Fill rule | nonzero (only Chandra is filled) |
| Ink alpha | 1.00; everything else 0 (template mask) |
| Commands used | `M`, `L`, `C`, `Z` only. **No `A` arcs anywhere.** |
| Arc approximation | cubic Bézier, `k = 0.5523` per quarter-circle, quarter-circle maximum per segment |

Cap-height lands at 16.5pt against the moon disc's 16.0pt diameter: a circle needs ~3% less
than a flat-terminated form to read at the same size, so the two are optically matched.

Surya's second subpath is a radius-0.9 circle. Stroked at 1.8 its inner radius collapses to
zero, producing a solid Ø3.6-unit dot. **Do not fill it separately** — one stroke pass draws
the whole glyph.

#### Path data

All nine verified: 9 present, no `A` commands, ink bounds (stroke included) within 0–24 on
both axes, and each rasterised at 44 × 44 px to confirm it is distinguishable.

**1. Surya (Sun) — stroked, width 1.8**

```
M 3.6,12 C 3.6,7.36 7.36,3.6 12,3.6 C 16.64,3.6 20.4,7.36 20.4,12 C 20.4,16.64 16.64,20.4 12,20.4 C 7.36,20.4 3.6,16.64 3.6,12 Z M 11.1,12 C 11.1,11.5 11.5,11.1 12,11.1 C 12.5,11.1 12.9,11.5 12.9,12 C 12.9,12.5 12.5,12.9 12,12.9 C 11.5,12.9 11.1,12.5 11.1,12 Z
```

**2. Chandra (Moon) — filled, no stroke**

```
M 14.74,3.43 C 11.03,2.24 6.99,3.57 4.71,6.72 C 2.43,9.87 2.43,14.13 4.71,17.28 C 6.99,20.43 11.03,21.76 14.74,20.57 C 10.26,20.23 6.8,16.49 6.8,12 C 6.8,7.51 10.26,3.77 14.74,3.43 Z
```

**3. Mangala (Mars) — stroked, width 1.8**

```
M 3.8,14.6 C 3.8,11.51 6.31,9 9.4,9 C 12.49,9 15,11.51 15,14.6 C 15,17.69 12.49,20.2 9.4,20.2 C 6.31,20.2 3.8,17.69 3.8,14.6 Z M 13.36,10.64 L 19.4,4.6 M 14,4.6 L 19.4,4.6 L 19.4,10
```

**4. Budha (Mercury) — stroked, width 1.8**

```
M 8.7,4.8 C 8.7,6.62 10.18,8.1 12,8.1 C 13.82,8.1 15.3,6.62 15.3,4.8 M 7.8,12.3 C 7.8,9.98 9.68,8.1 12,8.1 C 14.32,8.1 16.2,9.98 16.2,12.3 C 16.2,14.62 14.32,16.5 12,16.5 C 9.68,16.5 7.8,14.62 7.8,12.3 Z M 12,16.5 L 12,20.4 M 7.8,18.3 L 16.2,18.3
```

**5. Guru (Jupiter) — stroked, width 1.8**

```
M 7.2,9 C 7.2,5.4 10.5,3.9 13.2,5.1 C 16.2,6.45 16.05,10.2 14.1,12.6 C 12.15,15 9.6,17.4 8.4,19.2 L 19.2,19.2 M 13.5,12.6 L 13.5,20.4
```

**6. Shukra (Venus) — stroked, width 1.8**

```
M 7.2,9.6 C 7.2,6.95 9.35,4.8 12,4.8 C 14.65,4.8 16.8,6.95 16.8,9.6 C 16.8,12.25 14.65,14.4 12,14.4 C 9.35,14.4 7.2,12.25 7.2,9.6 Z M 12,14.4 L 12,20.4 M 8.4,17.7 L 15.6,17.7
```

**7. Shani (Saturn) — stroked, width 1.8**

```
M 5.1,7.2 L 11.1,7.2 M 8.1,3.6 L 8.1,20.4 M 8.1,14.7 C 8.7,11.4 12,10.2 14.4,11.7 C 16.8,13.2 17.1,16.8 15.6,20.4
```

**8. Rahu (ascending node) — stroked, width 1.8**

```
M 6,10.8 C 6,7.49 8.69,4.8 12,4.8 C 15.31,4.8 18,7.49 18,10.8 M 6,10.8 L 6,16.2 C 6,18.6 4.8,19.8 3.3,19.5 M 18,10.8 L 18,16.2 C 18,18.6 19.2,19.8 20.7,19.5
```

**9. Ketu (descending node) — stroked, width 1.8**

```
M 6,13.2 C 6,16.51 8.69,19.2 12,19.2 C 15.31,19.2 18,16.51 18,13.2 M 6,13.2 L 6,7.8 C 6,5.4 4.8,4.2 3.3,4.5 M 18,13.2 L 18,7.8 C 18,5.4 19.2,4.2 20.7,4.5
```

#### Verification table

| # | Graha | Mode | Ink x-min | y-min | x-max | y-max | Cap-height | `A` cmds |
|---|---|---|---|---|---|---|---|---|
| 1 | Surya | stroke 1.8 | 2.70 | 2.70 | 21.30 | 21.30 | 18.60 | 0 |
| 2 | Chandra | fill | 3.00 | 3.00 | 14.74 | 21.00 | 18.00 | 0 |
| 3 | Mangala | stroke 1.8 | 2.90 | 3.70 | 20.30 | 21.10 | 17.40 | 0 |
| 4 | Budha | stroke 1.8 | 6.90 | 3.90 | 17.10 | 21.30 | 17.40 | 0 |
| 5 | Guru | stroke 1.8 | 6.30 | 3.80 | 20.10 | 21.30 | 17.50 | 0 |
| 6 | Shukra | stroke 1.8 | 6.30 | 3.90 | 17.70 | 21.30 | 17.40 | 0 |
| 7 | Shani | stroke 1.8 | 4.20 | 2.70 | 17.42 | 21.30 | 18.60 | 0 |
| 8 | Rahu | stroke 1.8 | 2.40 | 3.90 | 21.60 | 20.44 | 16.54 | 0 |
| 9 | Ketu | stroke 1.8 | 2.40 | 3.56 | 21.60 | 20.10 | 16.54 | 0 |

Cap-height spread is 16.54 → 18.60 units. This is intentional, not drift:

- Surya and Shani sit at 18.60 because both terminate in flat strokes (a ring and a stem),
  which read short.
- Rahu and Ketu sit at 16.54 because they are the widest glyphs (19.2 units) and a wide form
  at equal height reads larger.
- Chandra sits at 18.00: a filled crescent is narrow (11.74 units) and needs full height.

Design-set constraints held constant across all nine: one stroke width (1.8), one cap and join
(round), one centre (x = 12 for the seven symmetric forms), one arc constant (0.5523), one
grid (24 × 24).

#### Distinguishability at 18px

| Pair at risk | Separator |
|---|---|
| Surya ⟷ Shukra | Surya is a closed ring with a centre dot and no descender; Shukra has a 6-unit cross below the circle |
| Shukra ⟷ Mangala | descender + horizontal crossbar vs. a 45° shaft to the upper right |
| Budha ⟷ Shukra | Budha adds the upward-opening semicircle above the circle; circle is 0.6 units smaller |
| Guru ⟷ Shani | Guru has a 10.8-unit horizontal baseline at y = 19.2; Shani has a 6-unit crossbar at y = 7.2 and a full-height left stem |
| Rahu ⟷ Ketu | 180° rotations of one another; the bowl opens down (Rahu) vs. up (Ketu) |

Rahu/Ketu are the only pair separated by orientation alone. Mitigation, per §11.3: the panel
header always shows the name in words, the menu bar item carries an accessibility label, and
the settings Grahas list pairs each glyph with its name.

### 7.3 Chandra's tray item — open item

D-009 makes the moon a permanent tray item; the brief also lists Chandra among the nine
toggleable grahas. Taken literally, enabling Chandra produces a second moon item.

**Proposed resolution, not yet approved:** the permanent moon item *is* Chandra's item. Its
row in Settings → Grahas is present, shows the Chandra glyph and name, and its toggle is on
and disabled with the caption `always shown`. The 24 × 24 Chandra crescent is used for that
row and nowhere in the menu bar — the live disc always wins there.

Flagged rather than decided, per the mid-implementation rule.

---

## 8. Motion

```css
--ease-panel:    cubic-bezier(0.32, 0.72, 0, 1);   /* decelerate, no overshoot */
--ease-standard: cubic-bezier(0.40, 0.00, 0.20, 1);
--ease-in:       cubic-bezier(0.40, 0.00, 1.00, 1);
```

### 8.1 Transition table

| Trigger | Property | Duration | Delay | Easing |
|---|---|---|---|---|
| Panel open | `opacity` 0 → 1 | 120ms | 0 | `--ease-panel` |
| Panel open | `translateY` −6px → 0 | 120ms | 0 | `--ease-panel` |
| Panel close | `opacity` 1 → 0 | 90ms | 0 | `--ease-in` |
| Day expands | window height + inner height, in lockstep | **220ms** | 0 | `--ease-panel` |
| Detail content in | `opacity` 0 → 1 | 140ms | 80ms | `--ease-standard` |
| Day collapses | window height + inner height | 180ms | 0 | `--ease-standard` |
| Detail content out | `opacity` 1 → 0 | 60ms | 0 | `--ease-standard` |
| Different day selected while expanded | detail text `opacity` 1 → 0 → 1 | 60ms out / 100ms in | 0 / 60ms | `--ease-standard` |
| Month switch | grid `opacity` 0 → 1 | 160ms | 0 | `--ease-panel` |
| Month switch | grid `translateX` ±8px → 0 | 160ms | 0 | `--ease-panel` |
| Day selection fill | `background-color` | 90ms | 0 | `--ease-standard` |
| Cell hover in / out | `background-color` | 80ms / 120ms | 0 | `--ease-standard` |
| Header button press | `opacity` 1 → 0.6 → 1 | 0ms down, 120ms up | 0 | `--ease-standard` |
| Month picker open / close | grid region `opacity` cross | 120ms | 0 | `--ease-standard` |
| Phase glyph appears (cold month) | `opacity` 0 → 1 | 120ms | staggered 0 per cell | `--ease-standard` |

Notes:

- **Panel close does not translate.** Opening explains *where the panel came from*; closing
  has nothing to explain, so it only fades. Asymmetry is deliberate.
- **Height change is 220ms**, the longest value in the app, because it is the only transition
  that moves the window frame itself. The panel window and the DOM node animate on the same
  curve and duration or the content visibly detaches from the frame.
- **Month switch translates ±8px in the direction of travel** — next month enters from the
  right, previous from the left. There is no cross-fade of two grids; the outgoing grid is
  removed at 0ms and the incoming one animates in. Two overlapping grids of numerals is noise.
- **Month picker open does not animate the panel height** (§5.7), which is exactly why the
  picker was sized to fit the grid region.

### 8.2 What must not animate

- The tray moon disc and the tray graha glyphs. They change at most once per day; a transition
  in the menu bar pulls the eye to a place the user was not looking.
- Date numerals, weekday labels, and the month label text. They may be replaced, never tweened.
- The phase glyph shape inside a cell. Shape changes only on month switch and rides the grid
  transition.
- The focus ring — appearance, position, and size are all instant. Keyboard navigation that
  lags behind the key is worse than no transition.
- Panel width. It is 320px permanently.
- The `℞` chip, event markers, retrograde span rules, combustion rules.
- Nothing loops, pulses, breathes, shimmers, or bounces. No skeleton shimmer, no spinner,
  no spring, no overshoot past a target value, no `steps()`.
- Settings window controls use unmodified AppKit-default behaviour.

### 8.3 Reduced motion

`@media (prefers-reduced-motion: reduce)`:

| Transition | Becomes |
|---|---|
| Panel open | `opacity` only, 80ms; no `translateY` |
| Panel close | `opacity` only, 60ms |
| Height change | **0ms** — resize is instant, in one frame |
| Detail content in/out | `opacity` only, 80ms, no delay |
| Month switch | `opacity` only, 80ms; no `translateX` |
| Everything else | 0ms |

---

## 9. States

### 9.1 Empty

The calendar has no empty state — there is always a month, and every date has a phase.
The only empty state in the app is the settings city search:

```
      │                                                        │  16
      │              No city matches “nowherevil”.              │  18   Body, --text-secondary
      │                                                        │  16
```

Centred, one line, sentence case, full stop. No illustration, no icon, no suggestion list.

### 9.2 Loading

| Condition | Render |
|---|---|
| Warm month (cache hit, < 5ms) | no loading state at all; the grid is present in the first frame |
| Cold month (> 120ms) | grid renders **immediately** with weekday row, all 42 date numerals, today ring and selection — everything derivable without the ephemeris. Glyph slots stay empty. |
| Glyph data arrives | each phase glyph fades in over 120ms (§8.1) |
| Day detail pending | field rows render with labels present and values blank. No placeholder characters. |

**No spinner. No progress bar. No skeleton shimmer. No dimming of the grid.** The 120ms
threshold exists so that a 30ms cold month (ARCHITECTURE §5 target) never shows a transitional
state at all.

### 9.3 Error

Errors replace **the detail area only**. The grid stays live and navigable — a failure to
compute one day must not cost the user the month.

Layout inside the detail area, on `--surface` with `--r-cell`, inset 20px left/right:

```
      │  ┌──────────────────────────────────────────────────┐  │  16 pad
      │  │  Location not set.                               │  │  18  Body 13/500, primary
      │  │  Rise and set times need a location.             │  │  16  Caption, secondary
      │  │                                                  │  │  12
      │  │  [ Set location ]                                │  │  28  button, Body 13/400
      │  └──────────────────────────────────────────────────┘  │  16 pad
```

One row per `AppError.code`. There is no generic fallback (ARCHITECTURE §6).

| Code | Headline | Cause line | Action |
|---|---|---|---|
| `DATE_OUT_OF_RANGE` | Outside 1800–2399. | Chandra has no ephemeris data for this date. | none |
| `NO_CONVERGENCE` | Boundary time unavailable. | This nakshatra entry could not be resolved. | none |
| `SETTINGS` | Settings could not be saved. | *(reason, one line, truncated at 60 chars)* | none |
| `ENGINE` | Ephemeris unavailable. | *(engine error text, one line, truncated at 60 chars)* | `Quit Chandra` |

`LOCATION_UNRESOLVED` was specified here and has been **removed as unreachable**.
§9.5 already required the D-007 chain to end in the system timezone's coordinates, which
always resolve offline, so there is no state in which the app has no location. The variant
was deleted from `AppError` rather than left in place unconstructed.

Colour: headline `--text-primary`, cause `--text-secondary`. No red, no icon, no border
emphasis. The block's `--surface` fill and its position are what mark it as different.

### 9.4 Degraded — not errors

Three of these occur in normal operation. None is a fault, so none gets a warning colour,
a warning icon, or a border. Each is stated as a fact, in words, in place.

**No moonrise on this date** (happens roughly monthly at most latitudes):

```
      │  Moonrise                                       —      │  24   em dash, --text-tertiary
      │                                    no rise on this date│  14   Caption, --text-tertiary
      │  Moonset                                      02:11    │  24
```

- Value column shows `—` (U+2014) in `--text-tertiary`, right-aligned to x = 300 like a time.
- A 14px caption row is inserted below that row only. Panel height grows by 14px.
- Same treatment, same wording pattern, for moonset.

**Circumpolar** (both rise and set absent for a structural reason):

```
      │  Moonrise                                       —      │  24
      │  Moonset                                        —      │  24
      │                                always above the horizon│  14   Caption, --text-tertiary
```

- One caption for the pair, not two.
- Wording is `always above the horizon` or `always below the horizon` — deliberately different
  from `no rise on this date`, so a circumpolar day and an ordinary skipped rise are never
  conflated.

**Moshier provenance** (dates outside 1800–2399, D-006):

```
      │  Rashi                                      Dhanu      │  24
      │                               19 Aug 11:04 → 22:47     │  18
      │                                                        │   8
      │  Moshier ephemeris — reduced precision outside 1800–2399│ 16   Caption, --annotate
      │                                                        │  16   pad-bottom
```

- Single caption line at the foot of the detail area, left-aligned at x = 20.
- Colour `--annotate` (= `--text-secondary`, 7.71:1). **No amber, no triangle, no border.**
- Adds 24px to panel height (8px gap + 16px line).
- The **grid** is never annotated for provenance. Marking 42 cells to say the same thing once
  is exactly the clutter kanso forbids.
- Rationale, per D-006: the failure mode this app must not have is presenting degraded data as
  authoritative. Stating the source once, in plain words, in the place the number is read,
  discharges that obligation. Alarming the user does not.

### 9.5 First run

- Location resolves through the D-007 chain. Step 3 (tz centroid) always succeeds offline, so
  the first panel open is never blocked and never shows `LOCATION_UNRESOLVED`.
- No onboarding, no welcome panel, no tour, no permission prompt (D-017).
- The moon item appears in the menu bar and the panel works. That is the whole first run.

---

## 10. Keyboard and focus

### 10.1 Key map

| Key | Action |
|---|---|
| `←` / `→` | move selection ∓1 / ±1 day; crosses month boundaries and switches the month |
| `↑` / `↓` | move selection ∓7 / ±7 days |
| `Page Up` / `Page Down` | ∓1 / ±1 month, selection keeps the day-of-month (clamped to month length) |
| `⇧Page Up` / `⇧Page Down` | ∓1 / ±1 year |
| `Home` / `End` | first / last day of the displayed month |
| `T` | jump to today, switching month if needed |
| `Return` or `Space` | expand the detail for the focused day; collapse if already expanded for that day |
| `Esc` | collapse the detail if expanded; otherwise close the panel |
| `Tab` / `⇧Tab` | move through the focus order (§10.3) |
| `M` | open / close the month picker |
| `⌘,` | open the settings window |
| `⌘W` | close the panel (or the settings window if it is key) |

Inside the month picker: `←` `→` `↑` `↓` move by month, `⇧←` / `⇧→` change year,
`Return` selects and closes, `Esc` closes without changing the month.

Not bound in v1: `⌘Q` behaves as macOS default, and no other key does anything. Unbound keys
are silently ignored — no beep, no shake.

### 10.2 Focus ring

| Property | Value |
|---|---|
| Colour | `--focus` `#EDEDEF` (16.93:1 on ground, 14.53:1 on selection) |
| Width | 2px (3px under `prefers-contrast: more`) |
| Inset on a 40 × 40 cell | 1px → 38 × 38, radius 9 |
| Inset on a header button | 0px around the 28 × 28 hit area, radius 6 |
| Gap to the selection fill | 1px of `--ground` — selection and focus never touch |
| Visibility | `:focus-visible` only. Pointer interaction never draws it. |
| Transition | none, ever (§8.2) |

### 10.3 Focus order

```
  1. header: subject + month label button
  2. header: ‹ previous month
  3. header: › next month
  4. header: ⚙ settings
  5. day grid                 — ONE tab stop, roving tabindex inside
  6. detail area              — only if expanded, and only if it contains a button (§9.3)
  → wraps to 1
```

- The 42 day cells are **one** tab stop. Arrow keys move inside the grid, which is what
  `role="grid"` requires and what stops `Tab` from becoming a 42-press journey.
- The roving `tabindex="0"` cell is the selected day; if no day is selected, it is today; if
  today is outside the displayed month, it is the first day of the displayed month.
- The expanded detail contains no focusable content except the error button, so it is skipped
  in the common case.
- Focus is trapped inside the panel while it is open. The panel takes key focus on open and
  the initially focused element is the grid (item 5), not the header — the user opened it to
  look at days.
- On close, focus returns to whatever owned it before the panel opened. The panel never steals
  focus back.

---

## 11. Accessibility

### 11.1 Contrast

See §4.1 for the full table. Summary of the guarantees:

- Every text token clears **4.5:1** on `--ground`, `--surface`, and `--surface-selected`.
  Nothing in the app relies on the large-text 3:1 allowance.
- Lowest text pair in the app: `--text-tertiary` on `--surface-selected` = **4.64:1**.
- Every non-text graphic that carries meaning clears **3:1**: `--accent` today ring 9.97,
  `--focus` ring 16.93, `--marker` 11.88, `--disc-lit` 13.40.
- The two tokens below 3:1 (`--hairline` 1.19, `--border` 1.33) carry no meaning and can be
  deleted without making anything ambiguous. `--disc-ring` at 2.42 is covered by §4.1.

### 11.2 Hit targets

| Target | Visual | Hit area |
|---|---|---|
| Day cell | 40 × 40 | 40 × 40 |
| Header chevrons, settings gear | 24 × 24 | 28 × 28 |
| Subject + month label | text | 28px tall × full text width |
| Month picker cell | 70 × 60 | 70 × 60 |
| Year stepper | 24 × 24 | 28 × 28 |
| Settings toggle row | full row | 320 × 28 minimum |
| Tray item | 22 × 22 pt | system-managed |

Nothing in the app has a pointer target smaller than **28 × 28**.

### 11.3 Never conveyed by colour or shape alone

| Meaning | Non-textual carrier | Textual carrier — always present |
|---|---|---|
| Moon phase | disc glyph | day detail names it: `Waxing Gibbous`; cell `aria-label` names it |
| Illumination | disc fill area | `68.4% lit` as a number |
| Today | `--accent` ring **and** `--accent` numeral | detail caption reads `TODAY · THURSDAY 20 AUGUST 2026`; `aria-current="date"` |
| Selected day | `--surface-selected` fill | `aria-selected="true"` |
| Focused day | `--focus` ring | `aria-activedescendant` |
| Retrograde | `℞` beside the symbol, `--marker` span rule, `--marker` diamond at a station | `℞` chip **and** the `Motion` row reading `Retrograde` **and** the speed value shown with an explicit `−` sign **and** the event row `Retrograde station` |
| Sign ingress | filled disc marker | detail event row `Enters Simha` with its time |
| Nakshatra ingress | ring marker | detail event row `Enters Magha` with its time |
| Combustion | 12 × 1px rule under the numeral | detail event row `Combust` |
| Rahu vs Ketu | glyph orientation | header label names the graha; tray item accessibility label names it; settings row pairs glyph with name |
| Degraded ephemeris | none | the sentence in §9.4 — this one has **no** visual carrier at all, by design |
| No rise / circumpolar | `—` in the value column | the caption sentence below the row |

Rahu/Ketu and today are the two cases where a colour or an orientation does real work. Both
carry a redundant textual carrier in the same surface, not in a tooltip.

### 11.4 VoiceOver

| Element | Role | Label |
|---|---|---|
| Panel | `dialog`, `aria-label` | `Chandra, August 2026` |
| Grid | `grid`, `aria-rowcount=6`, `aria-colcount=7` | `August 2026` |
| Week | `row` | — |
| Moon cell | `gridcell` | `20 August, waxing gibbous, 68 percent illuminated` |
| Moon cell, today | `gridcell`, `aria-current="date"` | `Today, 20 August, waxing gibbous, 68 percent illuminated` |
| Graha cell, no events | `gridcell` | `20 August` |
| Graha cell, with events | `gridcell` | `20 August, 2 events: enters Simha, retrograde station` |
| Graha cell, retrograde | `gridcell` | `20 August, retrograde, 1 event: enters Magha` |
| Detail area | `region`, `aria-live="polite"` | `Day detail` |
| Field row | `group` | `Moonrise, 15:42` |
| Field row, no event | `group` | `Moonrise, none, no rise on this date` |
| Provenance note | inside the live region | read last, after all six fields |
| Tray item | system | `Chandra, waxing gibbous` / `Mangala` |

- The detail region is `aria-live="polite"`, so changing the selected day announces the new
  detail without the user re-navigating.
- Times are announced as times, not digit strings: the DOM carries `<time datetime="…">`.
- Degrees are announced in words (`118 degrees 42 minutes 7 seconds`) via `aria-label`, while
  the visible text keeps the `°′″` symbols.

### 11.5 Other system settings

| Setting | Response |
|---|---|
| `prefers-reduced-motion: reduce` | §8.3 |
| `prefers-contrast: more` | §4.2 |
| Reduce transparency | no-op — the panel is already fully opaque (D-011) |
| Light menu bar / dark menu bar | tray glyphs are template images and invert automatically (D-008) |
| Light system appearance | **the panel stays dark.** Chandra is a single-appearance app; the palette in §4 is the only palette. |
| Increase pointer contrast | no-op |
| Larger text (Dynamic Type) | not honoured in v1 — the 320 × 332 grid has no room to reflow. Recorded as a known limitation, not a stub. |

---

## 12. Settings window

Not a popover. A standard resizable-off window, in scope only as a container for the sections
named in ARCHITECTURE §8.

| Property | Value |
|---|---|
| Size | 520 × 420, not resizable |
| Chrome | standard titlebar, close only (no minimise, no zoom) |
| Background | `--ground` |
| Tabs | `NSToolbar`-style segmented row at the top, 44px: General · Location · Astrology · Grahas · About |
| Section padding | 24px all sides |
| Row height | 28px minimum, 32px for rows with a control |
| Label column | 160px, right-aligned, Label 12/400 `--text-secondary` |
| Control column | starts at x = 200, left-aligned |
| Row gap | 8px within a group, 24px between groups |
| Group header | Title 15/600, `--text-primary`, 16px above the first row |

Grahas tab: nine rows, each `[glyph 20 × 20] [name, Body 13/400] … [toggle]`. Chandra's row
per §7.3. Toggling adds or removes a tray item immediately, with no confirmation and no restart.

---

## 13. Token file

`src/styles/tokens.css` contains exactly: §2.1 spacing, §2.3 radii, §3.3 type roles, §4 colour,
§8 easing curves, and the panel shadow. Nothing else in the front end declares a colour, a
radius, a duration, or a font size literal.

---

## 14. Open items

| # | Item | Blocks |
|---|---|---|
| 1 | Chandra's tray item vs. the nine-graha toggle list (§7.3) | Settings → Grahas, M3 |

No other open items. Everything else in this document is a decided value.
