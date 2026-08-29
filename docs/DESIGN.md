# Design specification

Status: **built.** Describes the shipped UI; where this document and the code disagree, the code
is right and this document is the defect. Implements the UI layer of
[ARCHITECTURE.md](ARCHITECTURE.md) §8 under [D-008](DECISIONS.md#d-008),
[D-009](DECISIONS.md#d-009), [D-010](DECISIONS.md#d-010), [D-011](DECISIONS.md#d-011),
[D-014](DECISIONS.md#d-014), and amended by [D-019](DECISIONS.md#d-019) through
[D-025](DECISIONS.md#d-025).

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
   *Revised twice.* A second hue, `--retro`, carried retrograde motion until D-020: a colour
   has to be learnt, it is the first thing a grayscale or colour-blind rendering loses, and it
   made the graha symbol carry two meanings at once. D-023 then narrowed the principle to what
   a hue is allowed to *do*. **Colour may depict, never encode.** `--glare` is a picture of the
   Sun's glare, not a code for combustion; `--accent` still means today and nothing else. In
   the grid, retrograde is a dotted bracket around the glyph (§6.3); `℞` is how the day view
   and the menu bar write it, which is how every printed ephemeris writes it and needs no key.
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
| `--s-1` | 2 | gap between a cell's numeral and its glyph line; hairline offsets |
| `--s-2` | 4 | cell top padding, weekday row → grid, `℞` chip padding |
| `--s-3` | 8 | header gap, value → time gap in a field, settings top padding |
| `--s-4` | 12 | panel top/bottom padding, gap between fields |
| `--s-5` | 16 | panel left/right padding — header, day view, settings — and the day view's foot |
| `--s-9` | 40 | day cell, header height |

There is no 20, 24 or 32 step. `--s-6`, `--s-7` and `--s-8` were specified here, described
nothing that was built, and are not in the token file.

### 2.2 Panel box

| Property | Value |
|---|---|
| Width | **320px** composed, fixed, never animates, never responsive |
| Content width | **288px** (320 − 2 × 16) |
| Height | **332px**, constant — 12 pad + 40 header + 4 + 264 region + 12 pad |
| Corner radius | 12px, and the material behind it is cut to the same radius |
| Border | 1px solid `rgba(255,255,255,0.12)`, inside the radius |
| Background | a **scrim**, `rgba(10,10,11,0.55)`, over `NSVisualEffectMaterial::Popover` on a transparent window (D-011) |
| Background, no material | opaque `--ground`, when `apply_vibrancy` fails or the user asks for reduced transparency |
| Shadow | **none.** The window is built with `.shadow(false)` |
| Chrome | none — no titlebar, no traffic lights, no beak/arrow |
| Position | horizontally centred on the tray item, top edge 6px below the menu bar, clamped to 12px from either screen edge |
| Scale | `appearance.scale`, 0.8–1.4 (§2.5) |

**The height is constant, and that is the whole point.** There is no collapsed and expanded
state: the calendar, the day view and settings all swap inside one 264px region, so the window
never resizes and the material behind it never has to be resized either. Nothing here animates
a frame.

### 2.3 Radii

| Token | px | Applies to |
|---|---|---|
| `--r-panel` | 12 | panel, and the corner the popover material is cut to |
| `--r-cell` | 8 | day cell fill, settings row fill |
| `--r-button` | 6 | header buttons |
| `--r-chip` | 4 | `℞` chip |
| `--r-focus` | 9 | focus ring on a 40px cell (`--r-cell` + 1px inset) |

### 2.4 Hairlines

- **There are no separators.** The full-bleed 1px divider specified here, and the `--hairline`
  token it used, do not exist: nothing in the panel is divided by a rule. It was there to part
  the grid from a detail pane below it, and there is no pane below the grid — views swap
  (§2.2).
- Panel border: 1px `rgba(255,255,255,0.12)`, the only line the panel draws.
- Groups are separated by space (§2.1), not by rules. That was always the rule; it now has no
  exception.

### 2.5 Scale

One multiplier, `appearance.scale`, clamped to **0.8 – 1.4**. Settings offers four steps, which
reach both ends of that clamp: 0.8 Compact · 1.0 Default · 1.2 Large · 1.4 Larger. Steps rather
than a slider, because the panel is a fixed composition and only a few sizes of it look
composed; the names say what the user gets rather than what the number is.

- The panel is composed at 320 × 332 and drawn at `scale`. Every dimension in this document is
  the composed one; the whole surface is transformed as a unit, so a larger panel is the same
  design bigger rather than a different layout.
- The back end resizes the window from the same number and re-cuts the material's corner to
  `12 × scale`. The two share one value or the drawn panel and its frame stop agreeing — which
  shows as a clipped corner at one end and a band of desktop at the other.
- Scaling the type alone was rejected: it grows the words inside a window that stays put, and
  takes the room out of the calendar.
- Below 0.8 the 10px labels stop being legible; above 1.4 the panel stops behaving like a menu
  bar popover.

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
| Headline | 17px | 600 | 22px | −0.20px | primary | the resolved place in settings → location |
| Title | 15px | 600 | 20px | −0.10px | primary | header subject + month, settings section title |
| Value | 15px | 400 | 20px | 0 | primary / secondary | a field's value, and the hour it gives way |
| Numeral | 13px | 450 | 16px | 0 | primary | day-cell date numerals |
| Body | 13px | 400 | 18px | 0 | primary | `then …` successor lines, settings rows, `℞` chip |
| Label | 12px | 400 | 16px | 0 | secondary | settings control labels |
| Caption | 11px | 400 | 14px | +0.10px | secondary | provenance note, settings hints and values |
| Micro-caps | 10px | 600 | 12px | +0.60px | secondary | weekday row, field keys, the day view's date line — uppercase |

Notes:

- **Value is larger than Body, deliberately.** A value and the hour it ends are what the day is
  opened to read; at 13px they were the same size as everything around them. The value and the
  time are set at the same size for the same reason — the hour is half of what the field says.
- Weight `450` for the day numeral, not 400: at 13px SF renders optically light, and `450`
  matches the perceived weight of Body text.
- Micro-caps is the only uppercase role. Nothing else is ever uppercased.
- **Caption and Micro-caps are secondary, not tertiary.** `--text-tertiary` carries no text
  anywhere in the app (§4, D-023).
- No italics anywhere. **No underlines anywhere** — the combustion rule that was the one
  exception is gone (§6.3, D-024).
- Text never wraps in the panel. Long values are handled by the abbreviation rules in §5.2,
  never by ellipsis, except city names in settings search results.

---

## 4. Colour tokens

`src/styles/tokens.css` is the single source of truth (D-014). **Every text token clears 4.5:1
against the worst case of the composited translucent ground** (§4.1) — which is what the ground
now is, and is not the same test as the opaque `#0A0A0B` this section used to measure against.

```css
:root {
  /* surfaces */
  --ground:            #0a0a0b;   /* the opaque fallback, and the scrim's own colour */
  --surface:           #141417;
  --surface-selected:  #2b2b32;   /* raised from #1C1C20: at 1.1:1 the selection was invisible */
  --surface-hover:     rgba(255, 255, 255, 0.04);

  /* text */
  --text-primary:      #ededef;
  --text-secondary:    #a1a1a8;
  --text-tertiary:     #85858d;   /* marks only, never text (D-023) */

  /* lines */
  --border:            rgba(255, 255, 255, 0.12);

  /* meaning — exactly two hues */
  --accent:            #d8b36a;   /* TODAY ONLY. Never used for anything else. */
  --glare:             #e2603a;   /* combustion, drawn as the glare it is (D-023) */

  /* graphics */
  --marker:            #c8c8ce;   /* retrograde ring, kshaya dot */
  --disc-lit:          #d4d4d9;   /* lit fraction of the in-panel phase glyph, graha symbols */
  --disc-ring:         rgba(255, 255, 255, 0.22);
  --disc-ring-new:     rgba(255, 255, 255, 0.34);

  /* interaction */
  --focus:             #ededef;   /* :focus-visible only */
  --focus-width:       2px;
}
```

The panel's scrim, `rgba(10, 10, 11, 0.55)`, is declared in `panel.css` rather than as a token:
it is one surface's background, not a value anything else may reach for.

**Two tokens this section used to declare do not exist.** `--hairline` went with the separator
(§2.4) and `--shadow-panel` with the drop shadow (§2.2). A third, `--annotate`, is still
*referenced* by `panel.css` for the provenance line but is not declared anywhere; that line
therefore inherits its colour. Recorded as a defect, not a design.

### 4.1 Contrast

**The ground is not a colour.** The panel is a scrim over live blurred content (§2.2), so what
text sits on depends on what is behind the window. Measured on the shipped panel over ordinary
content at the original 0.28 scrim, the composited ground ran `#1A3033` to `#5C4E36`. Every
figure below is the **worst case** — the light end of that range.

| Token | Hex | Worst-case composited ground | Verdict |
|---|---|---|---|
| `--text-primary` | `#ededef` | 6.91 measured at the 0.28 scrim; 0.55 is darker still, so higher | pass |
| `--text-secondary` | `#a1a1a8` | **4.76** at the 0.55 scrim (3.15 at 0.28) | pass |
| `--text-tertiary` | `#85858d` | **3.34** at 0.55, 2.21 at 0.28, 4.09 at 0.70 | **fails** — carries no text (D-023) |
| `--accent` | `#d8b36a` | lighter than `--text-secondary`, so ≥ 4.76 | pass (non-text; 3:1 needed) |
| `--marker` | `#c8c8ce` | lighter than `--text-secondary`, so ≥ 4.76 | pass (non-text; 3:1 needed) |
| `--disc-lit` | `#d4d4d9` | lighter than `--text-secondary`, so ≥ 4.76 | pass (non-text; 3:1 needed) |
| `--focus` | `#ededef` | = `--text-primary` | pass (non-text; 3:1 needed) |
| `--glare` | `#e2603a` | between tertiary and secondary; not separately measured | exempt by construction — see below |
| `--border` | `rgba(255,255,255,0.12)` | far below 3:1 | decorative — carries no meaning |
| `--disc-ring` | `rgba(255,255,255,0.22)` | far below 3:1 | decorative — see §5.4 |

Only the three text tokens were measured directly; the four graphic tokens are all lighter than
`--text-secondary`, so each clears whatever it clears.

Rules that follow from the table:

- **No scrim alpha rescues `--text-tertiary`.** 0.55 reaches 3.34, under both the 4.5:1 text
  floor and the 3:1 non-text bar at its worst; 0.70 reaches 4.09 and would make the panel
  opaque. The conclusion was not "darken the panel" but that the colour is too light to carry
  text on a translucent surface. It is kept for two marks with no text floor to clear: the
  `℞` chip's border and the settings switch knob.
- **`--glare` is exempt on purpose** (D-023). It depicts rather than encodes, it is never a
  sole carrier — every combust day says so in words in the day view — and it is drawn as a soft
  wash whose peak is only partly opaque, so a contrast figure for the hex would not describe
  what is on screen anyway.
- `--border` is below 3:1 by design and never carries information. The panel's edge is the
  border and the radius; there is no shadow behind it (§2.2), so it must be visible enough to
  find and is not required to be readable.
- `--disc-ring` is the *in-panel* glyph outline only, and only for the new-moon case. A
  new-moon cell is additionally identified by the date numeral, the day view wording, and the
  VoiceOver label, so the ring is never the sole carrier (§11.3).
- There is **no error hue.** Errors are rendered with `--text-primary` on `--surface` (§9.3).
  A red inside a 320px popover with one message adds alarm without adding findability.
- There is **no warning hue.** Degraded states are facts, not faults (§9.4).

### 4.2 Increased contrast

When `prefers-contrast: more`:

| Token | Normal | Increased |
|---|---|---|
| `--text-secondary` | `#a1a1a8` | `#b4b4bb` |
| `--text-tertiary` | `#85858d` | `#9a9aa2` |
| `--disc-ring` | `rgba(255,255,255,0.22)` | `rgba(255,255,255,0.44)` |
| `--focus-width` | 2px | 3px |

`--accent` and `--glare` are unchanged. `--text-tertiary` is lifted even though it sets no
type, because the marks it does draw are held to the same preference. No ratios are given: the
ground is composited and varies (§4.1), so a single figure would be a fiction.

---

## 5. Panel layout — moon calendar

### 5.1 Wireframe (320 × 332)

```
 x:0    16                                              304    320
      ┌────────────────────────────────────────────────────────┐  y:0
      │                                                        │  12   pad-top
      │  ◐  Chandra · August 2026                        [ ⚙ ] │  40   header
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
than shrinking. Panel height is therefore a constant 332px, and the menu bar item never opens a
differently-sized window month to month.

The 280px grid is centred in the 320px panel, so its own edges fall at x = 20 and x = 300 —
4px outside the 288px content column the header and the day view are padded to.

### 5.2 Header (40px)

```
 x:16      40    48                                    272   280   304
  ┌──────────┬───┬────────────────────────────────────────┬───┬───────┐
  │ subject  │ 8 │  "Chandra · August 2026"               │ 8 │   ⚙   │
  │  24 × 16 │   │  Title 15/600/-0.10                    │   │ 24×24 │
  └──────────┴───┴────────────────────────────────────────┴───┴───────┘
```

Three slots, `24px 1fr 24px` with an 8px gap, inside the panel's 16px side padding.

| Element | Box | Hit area | Notes |
|---|---|---|---|
| Subject glyph | 24 × 16 at x = 16, vertically centred | — | live moon disc for the moon panel; the graha template glyph for graha panels. Replaced by a `‹` back button outside the calendar |
| Subject + month label | text, not a button | — | states the month; sets an `Adhika` qualifier apart from the name it qualifies. In the day view it names the day (§5.6.2); in settings, the section name |
| `⚙` settings | 24 × 24 at x = 280 → 304 | 24 × 24 | opens settings inside the panel |

**There are no month chevrons.** They went with the picker (§5.7).

Button glyphs: 1.5px stroke, round cap/join, `--text-secondary`; `--text-primary` on hover;
opacity 0.6 while pressed.

**Truncation rule (deterministic, no ellipsis):** a ladder, tried in order until the label fits
its slot — `Chandra · August 2026`, then `Chandra · August`, then `August 2026`, then `August`.
The year is dropped first and the subject name last: the name is what tells a Mangala calendar
from a Chandra one, and losing it leaves two panels that read identically. The glyph at x = 16
still identifies the panel either way.

Measured from real layout — `scrollWidth` against `clientWidth` — not from a canvas. Assigning
a computed `font` shorthand to a canvas context silently fails for `-apple-system`, so the
canvas reported every candidate as fitting and the label clipped mid-character.

### 5.3 Weekday row (20px)

- 7 columns of 40px, label centred in each.
- Micro-caps 10/600/+0.60px, `--text-secondary`, uppercase.
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

**In a lunar month the two lines do not move.** The tithi takes the numeral's position, because
that is what the day is called, and the Gregorian date goes to the top-right corner — the
glyph keeps the second line in both calendars (D-021, D-024).

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
| Normal | numeral `--text-primary`; the Moon's disc and a graha's symbol in `--disc-lit` | — |
| Adjacent month | numeral `--text-secondary`; glyph, corner date, retrograde ring and combustion wash all at 45% alpha | clickable; click navigates to that month and selects the day |
| Hover (pointer) | fill `--surface-hover` | inset 2 → 36 × 36, r 8 |
| Selected | fill `--surface-selected` | inset 2 → 36 × 36, r 8 |
| Today | 1px `--accent` ring, numeral in `--accent`, **and** the phase glyph's own outline tinted `--accent` | inset 3 → 34 × 34, r 7 |
| Today + selected | both — the ring sits 1px inside the fill edge | — |
| **Combust** | radial `--glare` wash rising from the cell's foot, peak 50% alpha, out to 72% of the cell | full-bleed behind the content, r 8 |
| **Retrograde** | dotted 1px `--marker` ring, Ø18, dash `1.6 2.2`, round caps | centred on the *glyph* at (20, 29), not on the cell |
| **Kshaya** (lunar) | Ø3 `--marker` dot | on the numeral's top right, like a footnote mark |
| Focused (`:focus-visible`) | 2px `--focus` ring on the 36 × 36 fill layer, spreading out to the cell's edge | r 9 |

- Today is marked three times — ring, numeral colour, and the disc's own outline — so it
  survives a colour-blind or grayscale rendering. The third mark is the glyph's outline rather
  than an extra element: a dot under the numeral collided with the glyph, and a 40px cell has
  no room for a fourth thing. It is a bonus where there is an outline to tint: a graha has
  none, and the Moon loses it at full, where the outline is not drawn.
- **Combustion is a field, not a mark.** It lies behind everything, costs no room in a cell
  that has none, and a run of combust days reads as one warm stretch instead of five separate
  marks. It never dims the glyph: opacity is already spoken for by the adjacent-month state, so
  a dimmed glyph inside the month read as "not really here" rather than "inside the Sun's rays"
  (D-020, D-024).
- **The retrograde ring is a bracket over a run**: the half facing the retrograde days on the
  day the motion turns, a whole circle in between, the opposite half on the day it turns back.
  A run therefore reads as one shape spanning several cells. Centred on the glyph because the
  cell's middle falls between the numeral and the symbol, and a ring there cut through both.
  A run that carries on past the grid's edge is drawn whole there, which is true — the bracket
  says where the motion turns, and at the edge of six weeks on screen it does not.
- Both marks are drawn for every subject in both calendars. A state that appears in one
  calendar and not the other is a state nobody can learn (D-024).
- Focus is drawn on the fill layer rather than as an outline on the cell, so it sits outside
  the selection fill and the two are never confused when both are present.
- Focus ring is `:focus-visible` only. Pointer clicks never draw it.

### 5.6 The day view — one field stack

The day view **replaces** the calendar inside the 264px region. It does not appear below the
grid, there is no divider, and the panel does not grow (§2.2, §2.4).

One shape for all nine subjects and both calendars (D-025). What differs between subjects is
which fields exist, never how they are drawn.

```
      ┌────────────────────────────────────────────────────────┐
      │  ...header: Shukla Ashtami...                          │  40
      │                                                        │  12  pad-top
      │  TODAY · THURSDAY · 20 Aug · Guruvara                  │  14  Micro-caps, secondary
      │                                                        │  12
      │  TITHI                                                 │  12  Micro-caps, secondary
      │  Shukla Ashtami                                14:02   │  20  Value | Value, secondary
      │  then Shukla Navami                                    │  18  Body, secondary
      │                                                        │  12
      │  NAKSHATRA                                             │  12
      │  Purva Ashadha · Pada 2                  07:12, 21 Aug │  20
      │  then Uttara Ashadha                                   │  18
      │                                                        │  12
      │  MOONRISE                                              │  12
      │  15:42                                    sets 02:11   │  20
      │                                                        │  16  pad-bottom
      └────────────────────────────────────────────────────────┘
       16 ├──────────── 288 ────────────┤ 16
```

**A field** is three lines, and the third is often absent:

| Line | Role | Colour |
|---|---|---|
| key | Micro-caps 10/600/+0.60, uppercase | `--text-secondary` |
| value + time | Value 15/400 left, Value 15/400 right, baseline-aligned, 8px minimum gap | `--text-primary` / `--text-secondary` |
| `then …` | Body 13/400 | `--text-secondary` |

- **Nothing is aligned to a column but the right-hand time**, so the stack reads as a list of
  statements rather than as a table. The old right-aligned two-column field row is gone.
- **The value and the time are the same size deliberately.** The hour is half of what the field
  says; at caption size it was the smallest thing on the surface.
- **The `then …` line names the successor.** A day holds at most two of each span, so it costs
  one line and answers the question the boundary time raises, instead of leaving the reader to
  open tomorrow. The successor is the next span *in time*, not merely the next one that is not
  prevailing — which on any day whose reference instant falls in the second span names the one
  already gone.
- 12px between fields (`--s-4`). The stack scrolls inside the region if it is taller than 264px;
  the scrollbar is hidden and a 20px fade at the region's foot says it continues.

Fields, in order, and which subjects have them:

| Field | Present when | Value | Time | Then |
|---|---|---|---|---|
| Phase | the Moon, solar mode only | `Waxing Gibbous` | — | — |
| Tithi | lunar mode, any subject | `Shukla Ashtami`, with ` · kshaya` where the tithi held no sunrise | when it gives way | the next tithi |
| Nakshatra | always | `Purva Ashadha · Pada 2` | when it gives way | the next nakshatra |
| Rashi | always | `Dhanu` | when it gives way | the next rashi |
| Motion | a graha | `Direct` / `Retrograde`, with the `℞` chip when retrograde | — | — |
| Rise | every subject but the nodes | the rise time, or `does not rise` | `sets HH:MM`, or `does not set` | — |
| Combust | the subject is combust that day | `inside the N° orb` | — | — |
| Events | the selected day has any | one line per event | its time | — |

**The boundary time is one number, chosen by scale.** A boundary today is a clock time,
`14:02`; the day either side keeps its time and gains a date, `07:12, 21 Aug`, because a tithi
ending at two in the morning is still a time to a reader; further off it is a date alone,
`9 Oct`. The word "until" is not printed — the spoken label says it (§11.4). A boundary that
did not resolve reads `time unavailable`; no time is ever guessed.

- **Rise and set belong to the subject, and the field is named for it**: `SUNRISE` on Surya's
  day, `MOONRISE` on Chandra's, and the graha's own name on a graha's — `SHANI RISE`. There is
  no separate sunrise row on every subject's day; sunrise *is* Surya's rise, and a bare `RISE`
  made the reader look back at the header to find out whose.
- **The field is absent for Rahu and Ketu.** They are points on the ecliptic and do cross the
  horizon; the app declines to model it, and `does not rise` would report that decision as an
  observation — the same sentence a circumpolar Moon gets for a different reason.
- **The phase does not appear in a lunar month.** The tithi says the same thing more precisely.
- **`Events` is not headed `Today`.** These are the selected day's, and the selected day is
  usually not today.

**Gone from this view**, and not replaced: illuminated percentage, distance from the Sun,
speed, longitude (D-025). They were the only reason the Moon's view and a graha's had different
shapes, and none of them was what anyone opened the day to read. The separation in degrees is
gone with them — the orb is the fact; the reading behind it is not.

### 5.6.1 Height

There is no height budget. The panel is 332px in every view (§2.2); a day view taller than the
264px region scrolls inside it.

### 5.6.2 What names the day

The header and the date line divide the day's name between them, and the division follows the
calendar in force:

| Mode | Header | Date line under it |
|---|---|---|
| Solar | `20 August 2026` | `THURSDAY` |
| Lunar | `Shukla Ashtami` — the prevailing tithi | `THURSDAY · 20 Aug · Guruvara` |

A lunar day is called by its tithi, so that is the title there; the civil date it also has drops
to the line below, beside the weekday and the vara, because a lunar day still has to be findable
in the world the user lives in. In solar mode the western date **is** the name, and the header
keeps it.

`TODAY · ` prefixes the date line when the selected day is today, in `--accent`.

### 5.7 Changing month

There is no month picker and no header chevrons. The grid scrolls: three months are stacked and
moved together by a wheel or a drag, and the strip settles onto whichever fills the window when
the gesture ends. `Page Up` / `Page Down` step a month, with `⇧` a year.

The picker specified here was a 4 × 3 grid of month names inside the region, with a year
stepper above it. It was designed for a calendar addressed by a year and a number, and a lunar
month has neither: it runs between syzygies, so there is nothing to lay out in a grid of twelve
and no year that contains a fixed set of them. Continuous scrolling is one gesture that works
in both systems, and it removed the header's chevrons and its label button with it — which is
why §10.3's focus order below is four items shorter than it was.

The header label is not a button. It states the month and, where the month is intercalary, sets
the `Adhika` qualifier apart from the name it qualifies.

---

## 6. Panel layout — graha calendar

Identical shell: same 320 width, same 40px header, same weekday row, same 6 × 40 grid, same
region, same field stack. Only the cell's glyph and which fields exist differ.

### 6.1 What changes

| Element | Moon panel | Graha panel |
|---|---|---|
| Header glyph | live moon disc, Ø16 | graha template glyph, 24 × 16 |
| Header label | `Chandra · August 2026` | `Mangala · August 2026` |
| Cell second line | phase glyph Ø14 | the graha's symbol, 14px |
| Cell marks | combustion wash | combustion wash, retrograde ring |
| Day view | Phase (solar only), Tithi, Nakshatra, Rashi, Moonrise, Combust | Tithi, Nakshatra, Rashi, Motion, Rise, Combust, Events |

The glyph says **which** graha and nothing else. Every state it can be in is carried beside it
(§5.5), which is what lets both calendars draw the same marks in the same places.

### 6.2 Cell anatomy (40 × 40)

```
    y:0  ┌───────────────────────────────────────────┐
         │ ░░░░░░░░░░░░░░░░ combustion ░░░░░░░░░░░░░ │       --glare wash,
    y:4  │                  ┌───┐             ┌────┐ │       behind everything
         │                  │ 21│             │ 1 SEP│ 16   Numeral | corner date
   y:20  │                  └───┘             └────┘ │
         │                                           │   2   gap
   y:22  │                ⁘ ┌──────┐ ⁘               │  14   symbol, centre (20,29)
         │                ⁘ │  ♂   │ ⁘               │       retrograde ring Ø18,
   y:36  │                ⁘ └──────┘ ⁘               │       dotted, on the glyph
         │                                           │   4
   y:40  └───────────────────────────────────────────┘
```

Nothing sits at the cell's edges. The rules that used to run along the top and bottom edges —
combustion under the numeral, retrograde across the foot — are gone (D-024).

### 6.3 What the cell marks, and what it does not

**Two states, both spans, both drawn as fields or brackets rather than as points:**

| State | Mark | Geometry | Colour |
|---|---|---|---|
| Combust | radial wash from the cell's foot | `130% 62% at 50% 112%`, 50% → 20% → 0% alpha, r 8 | `--glare` |
| Retrograde | dotted ring, drawn as a bracket over the run | Ø18, 1px, dash `1.6 2.2`, round caps, centred on the glyph | `--marker` |

**Instants are not marked at all.** Sign ingress, nakshatra ingress and the two stations were
drawn as a row of 5px discs, rings and diamonds under the numeral; that row is gone, and with
it the `+` overflow mark. They are named in words, with their times, in the day view's `Events`
field (D-024).

Why this reads without clutter:

- **A span is not a point.** Combustion and retrograde last for days, so they are drawn as a
  field and as a bracket, both of which span cells. A run of combust days reads as one warm
  stretch; a retrograde run reads as one shape opening, holding and closing.
- **No underlines.** The 12 × 1px rule under the numeral read as underlined text, which is what
  removed it, and nothing was put back in that position.
- **Colour depicts here, it does not encode** (D-023). The wash is warm because glare is warm;
  it is not a code for "combust".
- Every mark is named in words in the day view (§5.6) and in the cell's spoken label (§11.4).
  Shape and colour are never the sole carrier.

### 6.4 Day view

Same stack as §5.6, with the fields listed in §6.1. Nothing about a graha's day is laid out
differently from the Moon's — that was the point of D-025.

| Element | Spec |
|---|---|
| `℞` chip | min-width 22 × 16, r 4, 1px `--text-tertiary` border, `℞` in Body 13/400 `--text-secondary`, immediately after the word `Retrograde` |
| Motion value | `Direct` or `Retrograde`, in words. A minus sign on a speed is not a state anyone should have to infer — and the speed is gone |
| Combust value | `inside the N° orb`. The separation in degrees is not shown |
| Event rows | one line per event: `describeEvent` on the left, its time on the right |
| Event wording | `Enters <Rashi>` · `Enters <Nakshatra>` · `Retrograde station` · `Direct station` · `Combust` · `Leaves combustion` |

**Longitude and speed are not in this view** (D-025). Neither is the `℞` chip's old right-hand
alignment: it sits with the word it qualifies.

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

**Refresh.** Redrawn on the local hour (D-028), and on every settings change, panel open and
location answer. The value used is the illumination **now**.

The disc and the grid cell therefore answer different questions and can differ within a day:
the cell is the illumination at that day's local noon, the disc is the illumination at this
moment. That is deliberate. A menu bar says what is happening; a calendar says what a day is.
Bound to noon the disc was right twice a day at best, and redrawn only at midnight it was up to
twenty-four hours stale — the Moon's lit fraction moves by as much as thirteen points across a
day, so by evening it was visibly wrong.

**Tooltip.** Rebuilt when the pointer arrives, from that instant (`TrayIconEvent::Enter`), so it
is current to the second rather than to the hour.

It does not carry the percentage lit. That is the answer to "how much of the disc is showing",
which is a solar-calendar question and is already drawn — the icon beside the pointer *is* that
number. What it carries instead follows the calendar in force:

| Calendar | Tooltip |
|---|---|
| Solar | `Chandra - Waning Gibbous` |
| Lunar | `Chandra - Krishna Dvitiya` |

The tithi is the one in force at that instant, not the one today is named after. The panel names a
day from its sunrise, which is the tradition; the menu bar answers "now", which is what it is for
and what a transit is read against.

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

### 7.3 Chandra's tray item

D-009 makes the moon a permanent tray item; the brief also lists Chandra among the nine
toggleable grahas. Taken literally, enabling Chandra produces a second moon item.

**Resolved, and built as proposed:** the permanent moon item *is* Chandra's item. Its row in
Settings → Menu bar is present, shows the Chandra glyph and name, and its toggle is on and
disabled; the section's footnote reads `The moon is always shown.` rather than captioning the
row itself. The Chandra crescent is used for that row and nowhere in the menu bar — the live
disc always wins there.

---

## 8. Motion

```css
--ease-panel:    cubic-bezier(0.32, 0.72, 0, 1);   /* decelerate, no overshoot */
--ease-standard: cubic-bezier(0.40, 0.00, 0.20, 1);
```

`--ease-in` was specified here for a close transition that does not exist: the panel is hidden
by the window server, not faded out by CSS. It is not in the token file.

### 8.1 Transition table

| Trigger | Property | Duration | Delay | Easing |
|---|---|---|---|---|
| Panel open | `opacity` 0 → 1 and `translateY` −6px → 0 | 120ms | 0 | `--ease-panel` |
| Panel close | none | — | — | — |
| Day view in | `opacity` 0 → 1 | 140ms | 77ms | `--ease-standard` |
| Phase glyph in | `opacity` 0 → 1 | 120ms | 0 | `--ease-standard` |
| Day selection fill | `background-color` | 90ms | 0 | `--ease-standard` |
| Cell hover in / out | `background-color` | 80ms / 120ms | 0 | `--ease-standard` |
| Header button hover | `color` | 120ms | 0 | `--ease-standard` |
| Header button press | `opacity` → 0.6 | instant down, 120ms up | 0 | `--ease-standard` |
| Month strip settle | `translateY` to the nearest month | 140–300ms by distance | 0 | cubic ease-out, driven per frame |

Notes:

- **Nothing animates the window frame.** The height-change rows this table used to carry —
  220ms out, 180ms back — described a panel that grew to show a day. The panel is one height
  (§2.2) and the day view swaps into the region, so there is no frame to animate and no
  lockstep to keep.
- **Panel close does not animate at all.** Opening explains *where the panel came from*;
  closing has nothing to explain, and the window is simply hidden.
- **The month switch is not a transition either.** There is no cross-fade and no `translateX`:
  the strip moves under the gesture, and the settle is driven frame by frame, because a CSS
  transition ends by firing an event that never arrives when the target equals the current
  value — which left the strip wedged after a plain click. A frame loop finishes because it
  counts frames.
- The delay on the day view is `--d-detail-in × 0.55`, not a separate token.

### 8.2 What must not animate

- The tray moon disc and the tray graha glyphs. They change at most once per day; a transition
  in the menu bar pulls the eye to a place the user was not looking.
- Date numerals, weekday labels, and the month label text. They may be replaced, never tweened.
- The phase glyph shape inside a cell. Shape changes only on month switch and rides the grid
  transition.
- The focus ring — appearance, position, and size are all instant. Keyboard navigation that
  lags behind the key is worse than no transition.
- Panel width **and height**. Both are fixed (§2.2); the panel never resizes, so there is
  nothing there to animate or to suppress.
- The `℞` chip, the combustion wash, the retrograde ring, the kshaya dot.
- Nothing loops, pulses, breathes, shimmers, or bounces. No skeleton shimmer, no spinner,
  no spring, no overshoot past a target value, no `steps()`.
- The native controls inside the settings sections use unmodified AppKit-default behaviour.

### 8.3 Reduced motion

`@media (prefers-reduced-motion: reduce)`:

| Transition | Becomes |
|---|---|
| Panel open | 80ms |
| Day view in | 80ms |
| Phase glyph in | 0ms |
| Selection fill, cell hover | 0ms |
| Month strip settle | 80ms flat, regardless of distance |
| Everything else | 0ms |

The durations are redefined on the tokens themselves, so every rule that reads one follows
without a second code path.

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
| Cold month (> 120ms) | the weekday row is present; the grid appears whole when it arrives |
| Day view pending | nothing renders until the day arrives. No labels, no placeholder characters, no blank rows |

**No spinner. No progress bar. No skeleton shimmer. No dimming of the grid.** The 120ms
threshold exists so that a 30ms cold month (ARCHITECTURE §5 target) never shows a transitional
state at all.

The partial skeleton this table used to specify — numerals and today's ring drawn immediately,
glyphs fading in behind them — is not implemented and cannot be. The front end no longer knows
which civil days a month holds: a lunar month runs between syzygies, so the back end lays the
42 cells out and sends them in reading order (D-021). There are no numerals to draw before the
month arrives. What replaced it is the months already in hand: the panel is not reloaded
between opens and the neighbours either side are already fetched, so a cold month is rare and a
warm one has no transitional state at all.

### 9.3 Error

**Every view renders the failure it can cause**, in place:

| View | Where the error goes |
|---|---|
| Day view | replaces the fields |
| Calendar | replaces the grid. The month in the window is the one that failed, so there is nothing left for the grid to stay live for |
| Settings | above the controls, not instead of them — the controls are what the user needs in order to try something else |

The earlier rule that errors replace the detail area only, and that the grid always stays
navigable, described a signal that reached the day view alone: a month that failed to load
showed nothing at all, and a settings save that failed fired while the user was by definition
in settings, which made `SETTINGS` unreachable text.

Layout, on `--surface` with `--r-cell`:

```
      │  ┌──────────────────────────────────────────────────┐  │  16 pad
      │  │  Outside 1800–2399.                              │  │  18  Body 13/500, primary
      │  │  Chandra has no ephemeris data for this date.    │  │  16  Caption, secondary
      │  └──────────────────────────────────────────────────┘  │  16 pad
```

One row per `AppError.code`. There is no generic fallback (ARCHITECTURE §6) and **no action
button**: none of the four failures has a button that would help.

| Code | Headline | Cause line |
|---|---|---|
| `INVALID_DATE` | Not a date. | That day does not exist in the calendar. |
| `NO_CONVERGENCE` | Boundary time unavailable. | This entry could not be resolved. |
| `SETTINGS` | Settings could not be saved. | *(reason, one line, truncated at 60 chars)* |
| `ENGINE` | Ephemeris unavailable. | *(engine error text, one line, truncated at 60 chars)* |

`LOCATION_UNRESOLVED` was specified here and has been **removed as unreachable**.
§9.5 already required the D-007 chain to end in the system timezone's coordinates, which
always resolve offline, so there is no state in which the app has no location. The variant
was deleted from `AppError` rather than left in place unconstructed.

Colour: headline `--text-primary`, cause `--text-secondary`. No red, no icon, no border
emphasis. The block's `--surface` fill and its position are what mark it as different.

### 9.4 Degraded — not errors

These occur in normal operation. None is a fault, so none gets a warning colour, a warning
icon, or a border. Each is stated as a fact, in words, in place.

**No rise, or no set** (happens roughly monthly at most latitudes):

```
      │  MOONRISE                                              │  12
      │  does not rise                        sets 02:11       │  20   Value | Value
```

- The **value says it in words**, in the slot the time would occupy. There is no em dash and
  no caption row: an em dash is a symbol to decode, and a caption row grew the panel.
- Same wording pattern for the set: `does not set`.
- The nodes get no field at all rather than `does not rise`. They are points on the ecliptic
  and do cross the horizon; the app declines to model it, and reporting that decision as an
  observation would put a circumpolar Moon's sentence on a completely different fact.

**A boundary that did not resolve:**

```
      │  TITHI                                                 │  12
      │  Shukla Ashtami                    time unavailable    │  20
```

- The value is still correct; only the instant is missing. No time is ever guessed.

**A tithi no day is named after** (kshaya):

```
      │  TITHI                                                 │  12
      │  Shukla Shashthi · kshaya                     14:02    │  20
```

- Said as a property of the tithi, not of the day: `two sunrises` beside a heading naming one
  civil date read as a claim that the day had two dawns.

**Moshier provenance** (dates outside 1800–2399, D-006):

```
      │  RASHI                                                 │  12
      │  Dhanu                                        22:47    │  20
      │                                                        │   8
      │  Moshier ephemeris — reduced precision outside 1800–2399│ 14   Caption
      │                                                        │  16   pad-bottom
```

- Single caption line at the foot of the day view, left-aligned with the fields.
- **No amber, no triangle, no border.** The colour is `--annotate`, which is not declared
  anywhere (§4) — the line therefore inherits `--text-primary`. That is a defect, not the
  design; the design is `--text-secondary`.
- The **grid** is never annotated for provenance. Marking 42 cells to say the same thing once
  is exactly the clutter kanso forbids. That leaves a Moshier *month* silent, which is open as
  W-03 in `docs/AUDIT.md`.
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
| `Page Up` / `Page Down` | ∓1 / ±1 month. **The selection does not follow** — the month moves under it |
| `⇧Page Up` / `⇧Page Down` | ∓12 / ±12 months |
| `Home` / `End` | first / last day of the displayed month |
| `T` | jump to today, switching month if needed |
| `Return` or `Space` | open the day view for the selected day, or for today if nothing is selected |
| `Esc` | close the day or settings view; otherwise clear the selection; otherwise close the panel |
| `Tab` / `⇧Tab` | move through the focus order (§10.3) |
| `⌘,` | open settings |
| `⌘W` | close the panel |

Arrow keys, `Home`, `End`, `T`, `Return` and `Space` are live in the calendar only. `Esc`,
`⌘,` and `⌘W` are live in every view.

There is no month picker and no `M`. Months change by scrolling or dragging the grid, which is
the only gesture that works the same in a lunar month, where stepping goes from one syzygy to
the next rather than through a numbered sequence (§5.7).

Not bound in v1: `⌘Q` behaves as macOS default, and no other key does anything. Unbound keys
are silently ignored — no beep, no shake.

### 10.2 Focus ring

| Property | Value |
|---|---|
| Colour | `--focus` `#ededef`, the same value as `--text-primary` (§4.1) |
| Width | `--focus-width`, 2px (3px under `prefers-contrast: more`) |
| On a 40 × 40 cell | a 2px spread on the 36 × 36 fill layer, reaching the cell's edge, radius `--r-focus` 9 |
| On a header button | a 2px outline on the 24 × 24 box, radius `--r-button` 6 |
| Visibility | `:focus-visible` only. Pointer interaction never draws it. |
| Transition | none, ever (§8.2) |

Focus rings are declared per control, never globally. A blanket `:focus-visible` rule put a ring
around the panel container itself, which takes key focus on mount — that was the bright white
border around the whole panel. A blanket `:focus { outline: none }` is equally wrong: it strips
the native rings from the AppKit controls in the settings sections, which §8.2 keeps at their
platform default.

### 10.3 Focus order

```
  1. header: ⚙ settings, or ‹ back outside the calendar
  2. day grid                 — ONE tab stop, roving tabindex inside
  3. settings controls        — only in the settings view
  → wraps to 1
```

Three items, not seven: the month label button and the two month chevrons went with the picker
(§5.7).

- The 42 day cells are **one** tab stop. Arrow keys move inside the grid, which is what
  `role="grid"` requires and what stops `Tab` from becoming a 42-press journey.
- The roving `tabindex="0"` cell is the selected day; if no day is selected, it is today; if
  today is outside the displayed month, it is the first day of the displayed month. Only days
  inside the displayed month are eligible, and only the month filling the window carries the
  stop: three grids are mounted at once, and the two off screen are hidden from assistive
  technology entirely.
- The day view contains nothing focusable at all — no buttons, no links, no controls — so it is
  skipped entirely. Settings is where the panel's other tab stops live.
- Focus is trapped inside the panel while it is open. The panel itself takes key focus on open
  — it is a focus host, not a control, and must never draw a ring for it — and the keyboard map
  above works immediately, without a `Tab` first.
- On close, focus returns to whatever owned it before the panel opened. The panel never steals
  focus back.

---

## 11. Accessibility

### 11.1 Contrast

See §4.1 for the table and how the ground was measured. Summary of the guarantees:

- Every colour that **sets text** clears **4.5:1** against the worst case of the composited
  translucent ground. Nothing in the app relies on the large-text 3:1 allowance.
- Lowest text pair in the app: `--text-secondary` at **4.76:1**. It is the floor because
  `--text-tertiary` no longer sets any type — it failed the test and was retired rather than
  the test being relaxed (D-023).
- Every non-text graphic that carries meaning is lighter than `--text-secondary` and so clears
  what it clears: the `--accent` today ring, the `--focus` ring, `--marker`, `--disc-lit`.
- `--border` and `--disc-ring` sit below 3:1, carry no meaning, and could be deleted without
  making anything ambiguous. `--glare` is exempt by construction (§4.1).

### 11.2 Hit targets

| Target | Visual | Hit area |
|---|---|---|
| Day cell | 40 × 40 | 40 × 40 |
| Settings gear, back chevron | 24 × 24 | 24 × 24 |
| Settings row | full row | 288 × 36 |
| Tray item | 22 × 22 pt | system-managed |

The header buttons are 24 × 24, not the 28 × 28 this table used to promise: the hit area is the
visual box, with no overhang into the panel's padding.

### 11.3 Never conveyed by colour or shape alone

| Meaning | Non-textual carrier | Textual carrier — always present |
|---|---|---|
| Moon phase | disc glyph | the `Phase` field names it, `Waxing Gibbous`; the cell's `aria-label` names it |
| Illumination | disc fill area | the cell's `aria-label`: `68 percent illuminated`. It is no longer a number on screen (§5.6) |
| Today | `--accent` ring, `--accent` numeral **and** the disc's tinted outline | the day view's date line reads `TODAY · …`; `aria-current="date"` |
| Selected day | `--surface-selected` fill | `aria-selected="true"` |
| Focused day | `--focus` ring | roving `tabindex` |
| Tithi | the numeral itself, `S8` | the `Tithi` field names it in full, `Shukla Ashtami`; the spoken label never gets the abbreviation |
| Retrograde | dotted `--marker` ring around the glyph | the `Motion` field reads `Retrograde` with the `℞` chip; the cell's spoken label says `retrograde`; an `Events` line names the station |
| Combustion | `--glare` wash at the cell's foot | the `Combust` field reads `inside the N° orb`; the spoken label says `combust at noon`, naming the instant it was measured at |
| Sign / nakshatra ingress | **none** — no mark is drawn (§6.3) | the `Events` field: `Enters Simha`, with its time |
| Kshaya | Ø3 `--marker` dot on the numeral | the `Tithi` value carries ` · kshaya`; the spoken label says `<name> skipped, no sunrise` |
| Vriddhi | **none** — no mark is drawn | the spoken label: `vriddhi, the same tithi names the day after` |
| Rahu vs Ketu | glyph orientation | header label names the graha; tray accessibility label names it; the settings row pairs glyph with name |
| Degraded ephemeris | none | the sentence in §9.4 — this one has **no** visual carrier at all, by design |
| No rise / does not set | none | the value itself reads `does not rise` / `does not set` |

Rahu/Ketu and today are the two cases where a colour or an orientation does real work. Both
carry a redundant textual carrier in the same surface, not in a tooltip.

Two rows above have no visual carrier at all — an ingress and a vriddhi. Both were drawn once
and both marks were removed (D-024); the words are what remains, and they are why removing the
marks was allowed.

### 11.4 VoiceOver

| Element | Role | Label |
|---|---|---|
| Panel | `dialog`, `aria-label` | `Chandra` |
| Grid | `grid`, `aria-rowcount=6`, `aria-colcount=7` | the month label, `Shravana VS 2083` |
| Week | `row` | — |
| Moon cell | `gridcell` | `20 August, waxing gibbous, 68 percent illuminated` |
| Moon cell, lunar mode | `gridcell` | `20 August, Shukla Ashtami, waxing gibbous, 68 percent illuminated` |
| Moon cell, today | `gridcell`, `aria-current="date"` | as above, unchanged |
| Moon cell, combust | `gridcell` | `…, combust at noon` |
| Graha cell | `gridcell` | `20 August` |
| Graha cell, retrograde and combust | `gridcell` | `20 August, retrograde, combust at noon` |
| Cell outside the month | `gridcell` | `…, outside this month` |
| Day view | `region`, `aria-live="polite"` | `Day detail` |
| Field | `group` | `Moonrise, 15:42, sets 02:11` |
| Span field | `group` | `Tithi, Shukla Ashtami, until 14:02` |
| Off-screen months | `aria-hidden="true"` | — |
| Tray item | system | `Chandra, waxing gibbous` / `Mangala` |

- **The spoken label says what was measured.** `combust at noon`, not `combust`: the state is
  read at one instant, and naming only the state claims it held all day.
- **The spoken label never gets an abbreviation.** `S8` is a printed notation, not a word, so
  the cell says `Shukla Ashtami`. Kshaya and vriddhi are named too — they are the reason the
  numbers jump or repeat, and omitting them leaves the sequence looking broken.
- Today carries `aria-current="date"` and no spoken prefix. A `Today,` in front of the label
  would say the same thing twice to a screen reader that already announces the attribute, and
  say it in a word that no other cell's label uses.
- The day view is `aria-live="polite"`, so changing the selected day announces the new fields
  without the user re-navigating.
- The word `until` is spoken but not printed: a span field's visible time column is the number
  alone (§5.6), and the label supplies the preposition.
- Three grids are mounted at once; the two off screen are hidden from assistive technology
  entirely, so `today` never claims `aria-current` in two grids at the same time.
- Weeks are real elements with `role="row"`. A grid whose gridcells are not wrapped in rows is
  invalid ARIA, and WebKit prunes the whole subtree — the calendar simply does not exist for
  VoiceOver. Verified against the live accessibility tree, which reported 12 elements and no
  cells before the wrapper was added.
- **Times are plain text, not `<time datetime="…">`.** The element this section specified was
  never built.

### 11.5 Other system settings

| Setting | Response |
|---|---|
| `prefers-reduced-motion: reduce` | §8.3 |
| `prefers-contrast: more` | §4.2 |
| Reduce transparency | the panel drops the material and paints `--ground` opaque — the same path taken when `apply_vibrancy` fails (§2.2, D-011) |
| Light menu bar / dark menu bar | tray glyphs are template images and invert automatically (D-008) |
| Light system appearance | **the panel stays dark.** Chandra is a single-appearance app; the palette in §4 is the only palette. |
| Increase pointer contrast | no-op |
| Larger text (Dynamic Type) | not honoured. The 320 × 332 grid has no room to reflow. Settings → Size (§2.5) scales the whole panel instead, which is a different thing and does not discharge this. Recorded as a known limitation, not a stub. |

---

## 12. Settings, inside the panel

**There is no settings window.** The 520 × 420 window with a segmented toolbar specified here
was never built. A 320px panel with a 264px region will not hold five tabbed panes side by
side, and a second surface is a second thing to learn.

Settings are a **drill-down inside the panel**, in the same region the calendar and the day
view use. The panel does not resize (§2.2).

| Property | Value |
|---|---|
| Container | the 264px region, scrolling, scrollbar hidden |
| Padding | 8px top, 16px sides, 16px foot |
| Header | the panel's own — the section name as the title, `‹` back in the glyph slot |
| Back | `‹` returns a section to the root list; from the root list it returns to the calendar |
| Row height | 36px for a navigation row |
| Radius | `--r-cell` on a row fill |

### 12.1 Sections

Seven, including the root list:

| Section | Root row shows | Contains |
|---|---|---|
| root | — | the six rows below, each with its current value |
| calendar | `Solar` / `Amanta` / `Purnimanta` | month system, and what it applies to |
| location | the resolved place | current place with coordinates and provenance, city search, device location |
| astrology | the ayanamsa's first word | ayanamsa, Rahu and Ketu node type |
| menubar | `Moon only` / `Moon + N` | the nine graha toggles, and colour mode |
| size | `Compact` / `Default` / `Large` / `Larger` | the four scale steps (§2.5) |
| about | — | Swiss Ephemeris version and credits |

- Each root row states **where it leads and what it is set to now**, so the list answers most
  questions without being entered.
- **Every change applies immediately.** Nothing here needs confirming: each setting is
  reversible and its effect is visible in the calendar behind it. There is no Save, no Cancel,
  no restart.
- Chandra's row in `menubar` is present, on, and disabled: the permanent moon item *is*
  Chandra's item (§7.3, D-009).
- The switch is drawn — a track and a knob — rather than a native checkbox, so it matches the
  panel. The controls that *are* native keep their platform appearance and their own focus
  rings (§8.2, §10.2).

---

## 13. Token file

`src/styles/tokens.css` contains exactly: §2.1 spacing, §2.3 radii, §3.3 type roles with their
tracking, §4 colour, `--focus-width`, §8 easing curves, and the transition durations — plus the
`prefers-contrast` and `prefers-reduced-motion` overrides of those same tokens. Nothing else in
the front end declares a colour, a radius, a duration or a font size literal.

It does **not** contain a shadow (§2.2) or a hairline (§2.4); neither exists. The panel's scrim
is declared in `panel.css`, because it is one surface's background rather than a value anything
else may reach for.

## 14. Open items

| # | Item | Blocks |
|---|---|---|
| 1 | `--annotate` is referenced by `panel.css` and declared nowhere (§4) | the provenance line's colour |

The one item this table used to carry — Chandra's tray item against the nine-graha toggle list
— is resolved and built (§7.3).

Two treatments are designed but not built, and are tracked in `docs/TODO.md` rather than here:
combustion and retrograde in the day view (`docs/design/day-view-states.md`), and a pointer
route for year navigation.

Everything else in this document is a decided value.
