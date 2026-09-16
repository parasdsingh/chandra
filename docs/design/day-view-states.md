# Combustion and retrograde in the day view

Status: **built.** Both treatments this document specifies ship; `docs/TODO.md`
§5.1 records them, and the predicates are `isCombust` and `hasRetrogradeRail` in
`src/components/DayDetail.tsx`.
Answers: "combustion in day view should be done beautifully, and aesthetically. similarly retro
should also produce a different day view."
Works under [D-011](../DECISIONS.md#d-011) (revised: popover material + scrim),
[D-019](../DECISIONS.md#d-019), [D-020](../DECISIONS.md#d-020),
[D-021](../DECISIONS.md#d-021), [D-022](../DECISIONS.md#d-022).
Expressed in the vocabulary of [DESIGN.md](../DESIGN.md) §2, §4, §6.3, §6.4, §8, §11.

---

## What shipped, and what this document no longer describes

The design below is left intact. The app has moved under it in five places; where a figure here
and the code disagree, **the code wins**.

**Shipped from this document**

- The amendment at the head of it — *colour may depict, may not encode* — is now
  [D-023](../DECISIONS.md#d-023).
- The flat fallback it files as a bug in §1 and §6.1 is fixed. `apply_vibrancy` reports whether
  the material took, the answer reaches the front end on `Bootstrap.panel_material`, and the
  panel takes `.is-opaque` and paints `--ground`. `prefers-reduced-transparency: reduce` gets
  the same answer. The class is named `is-opaque`, not `is-flat`.
- Neither state is ever a sole carrier: the day view names both in words.

**Superseded**

- **`--glare` is `#e2603a`, not `#F0C9A4`** (D-023). Orange rather than the peach specified
  here, so it cannot be read as `--accent`. Every contrast figure in §4.4 and §4.5, and the
  20° hue separation in the decision summary, are computed against the old value and are stale.
- **The scrim is 0.55, not 0.28** (D-011). §2.2's composited range and every `--text-tertiary`
  figure predate it.
- **`--text-tertiary` carries no text at all** (D-023), so the caption-contrast argument in the
  decision summary — combustion *improving* tertiary from 3.16 to 3.42 — is about a colour that
  no longer sets any type.
- **The grid's marks are not the ones assumed here** (D-024). Combustion in a cell is a warm
  radial wash at the cell's foot, not a rule; retrograde in a cell is a dotted bracket ring
  around the glyph, not `℞` beside it. The `℞` this document leans on survives in the day view's
  `Motion` chip and in the menu bar (D-022).
- **§1's note is discharged.** `DESIGN.md` §2.2 has since been brought in line with the shipped
  panel; it no longer claims an opaque surface with no material.

Amendment carried through this document, from the owner: **colour may depict, may not encode.**
A warm light is a picture of a warm thing and is allowed. A grey that *means* combust is a code
and is not. `--accent` still means today and nothing else.

---

## Decision summary

- **Build Direction 2, "field and vector."** Combustion is a *field* the body is inside — a warm
  rim of light in the two 16px gutters the day view never uses. Retrograde is a *vector* — a 1px
  hairpin rail in the right gutter that runs down, folds, and comes part-way back. Field vs line
  is the difference between an environment and a direction, and it is the difference between the
  two facts.
- **Both cost zero vertical pixels.** Neither treatment touches the content column's flow. The
  day view already runs to ~420px of content in a 264px region (§2.3) and scrolls; nothing here
  makes that worse by a single px.
- **Every treatment behind text darkens, never lightens.** The warm ink lives only in the
  gutters; the content column is deepened by `rgba(0,0,0,0.10)` in the same act. Under a live
  blurred backdrop this is the only way to guarantee text contrast: darkening the ground under
  light text can only raise the ratio. Combustion therefore *improves* caption contrast
  (`--text-tertiary` 3.16 → 3.42 on the worst-case ground).
- **Marks on vibrancy are drawn as a pair, not a colour.** No single hex clears 3:1 against a
  ground that ranges #141414 → #393939. The retrograde rail is a 1px `--marker` stroke over a 3px
  `rgba(0,0,0,0.45)` keyline on the same path, which holds **7.01:1** even at its worst position
  (on the brightest part of the rim, over a white desktop). Adopt this as a general rule.
- **One hue is added, `--glare` `#F0C9A4`, and it depicts rather than encodes.** It is the colour
  of sunlight through atmosphere. Its form (a soft rim at the panel's edge) is the carrier; the
  warmth is the picture. Under `prefers-contrast: more` it drops to achromatic white and the form
  is unchanged — the hue is the first thing discarded, by design. Hue separation from `--accent`
  is 20° and the two never appear within 24px of each other (§4.2).
- **No new motion.** Nothing here moves, so §8.2 is untouched. The field appears and disappears on
  the day view's existing 140ms/60ms opacity transitions and inherits §8.3 unchanged.
- **The flat fallback is a bug today and this spec fixes it.** `apply_vibrancy` failing leaves a
  0.28 scrim over a *transparent* window — the desktop shows through at 72% and every contrast
  guarantee is void. Rust must report whether the material applied; the panel takes `.is-flat` and
  paints `--ground` opaque. Same rule for `prefers-reduced-transparency`.
- **Combust and retrograde compose** because they are different mark classes on different layers:
  a soft symmetric field, and a hard one-sided line inside it. Both keep their words — the
  `Motion` row, the `Combust — inside the N° orb` caption, the `℞` chip — so nothing here is ever
  a sole carrier (§8.1).
- **The conservative option (Direction 1) is two hairlines** lifted verbatim from the grid's own
  vocabulary. It is correct, costs nothing, and does not change the surface at all. That is what
  restraint costs here: the panel ends up annotated rather than different.
- **Direction 3 (typographic) is rejected on measurement.** A warm bloom on the graha's name is
  either visible and below 4.5:1, or compliant and invisible; there is no alpha that is both
  (§7.2).

---

## 1. Note on the source documents

`DESIGN.md` §2.2 still says the panel background is `#0A0A0B`, fully opaque, "no vibrancy, no
blur, no material." That is stale. `src-tauri/src/panel.rs::apply_material` applies
`NSVisualEffectMaterial::Popover` and `panel.css` paints `rgba(10,10,11,0.28)` over it. This
document is written against the shipped code. §2.2 needs the D-011 revision folded in; that edit
is out of scope here and is not an open question, just an omission to fix.

---

## 2. What the day view actually is

### 2.1 Geometry, from the code

| Thing | Value | Source |
|---|---|---|
| Panel | 320 × 332, fixed, never resizes | `panel.rs` `PANEL_WIDTH` / `PANEL_HEIGHT` |
| Panel padding | `12px 0` | `.panel` |
| Header | 40px | `.header` |
| Region (the stage) | **264px**, `overflow: hidden` | `.region` |
| Day view | `.detail`, `height: 100%`, `overflow-y: auto` | `.detail` |
| Day view padding | `12px 16px 16px` | `.detail` |
| Content column | x = **16 → 304**, 288 wide | derived |
| **Left gutter** | x = 0 → 16, full 264px height, **empty** | derived |
| **Right gutter** | x = 304 → 320, full 264px height, **empty** | derived |
| Bottom fade | `.region::after`, 20px, `rgba(10,10,11,0 → .55)` | `.region::after` |

x is measured from the panel's inner edge; the 1px `--border` sits outside it.

The two 16px gutters are 4,224px² of surface each, present on every view, carrying nothing. They
are the only free space in the panel. Both directions that change the surface use them.

### 2.2 The ground is not a colour

The panel paints a scrim, not a fill. Model:

```
M = 0.80 × #1E1E1E  +  0.20 × blur(desktop)      material, dark appearance
S = 0.28 × #0A0A0B  +  0.72 × M                  panel scrim over it
```

Three reference grounds follow. Every figure in this document is given against all three.

| Ref | Composite | Hex | Rel. luminance | Occurs when |
|---|---|---|---|---|
| `G_flat` | scrim only, opaque | `#0A0A0B` | 0.00306 | material unavailable, `.is-flat` (§6.1) |
| `G_dark` | material over black desktop | `#141414` | 0.00698 | dark wallpaper, dark window behind |
| `G_lite` | material over white desktop | `#393939` | 0.04089 | **worst case** — white wallpaper or a white window behind |

Existing text on those grounds:

| Token | `G_flat` | `G_dark` | `G_lite` |
|---|---|---|---|
| `--text-primary` `#EDEDEF` | 16.94 | 15.77 | **9.89** |
| `--text-secondary` `#A1A1A8` | 7.71 | 7.18 | **4.50** |
| `--text-tertiary` `#85858D` | 5.41 | 5.03 | **3.16** |
| `--marker` `#C8C8CE` | 11.88 | 11.06 | **6.94** |

Two facts fall out and drive everything below.

1. **`--text-tertiary` is already at 3.16:1 on the worst-case ground.** Every caption in the day
   view is tertiary. Nothing may lighten the ground behind them, at any alpha.
2. **No fixed colour can guarantee 3:1** against a ground that moves 13× in luminance. A mark that
   must be seen needs a luminance step in *both* directions: a light stroke with a dark keyline
   under it. This is what macOS itself does on vibrancy, and it is adopted here as a rule.

### 2.3 The vertical budget is already spent

Graha day view, solar mode, one tithi span, no events:

```
12  pad-top
14  date line
 4
24  headline (name + ℞)
12
24  Longitude
24  Motion
24  Speed
12
24  Tithi          + 18 caption
12
24  Sunrise
24  Rise
24  Set
12
24  From Sun       + 18 caption when combust
12
24  Nakshatra      + 18 caption
12
24  Rashi          + 18 caption
16  pad-bottom
───
420px of content in a 264px region → scrolls ~156px
```

Panchanga fields still to come (yoga, karana, muhurtas, dignity, aspects) add to that. **A
treatment that costs 40px is not merely disqualified; a treatment that costs 4px is.** Every
proposal below is measured at exactly 0.

---

## 3. Neither — the ordinary day

Unchanged from `DayDetail.tsx` today. No extra element is rendered: `.state-field` is absent from
the DOM, not present-and-empty.

```
 x:0     16                                                   304  320
   ┌────────────────────────────────────────────────────────────────┐ y:0
   │                                                                │  12
   │  THURSDAY 20 AUGUST 2026 · GURUVARA                            │  14
   │                                                                │   4
   │  Mangala                                                       │  24
   │                                                                │  12
   │  Longitude                                    118° 42′ 07″     │  24
   │  Motion                                              Direct    │  24
   │  Speed                                        +0.6182 °/day    │  24
   │                                                                │  12
   │  Tithi                                    Shukla Ashtami       │  24
   │                                       04:31 → 07:12 (21 Aug)   │  18
   │                                                                │  12
   │  Sunrise                                             06:04     │  24
   │  Rise                                                21:07     │  24
   │  Set                                                 09:33     │  24
   │                                                                │  12
   │  From Sun                                         88° 43′      │  24
   │                                                                │  12
   │  Nakshatra                        Purva Ashadha · Pada 2       │  24
   ├────────────────────────────────────────────────────────────────┤ y:264
    ↑                                                              ↑
    left gutter, 16px, empty            right gutter, 16px, empty
```

Both gutters are blank down the whole 264px. That blankness is the baseline the two states are
read against.

---

## 4. Direction 2 — field and vector *(recommended)*

### 4.1 The idea in one line

Combustion is something the body is **inside**, so it is drawn as a field. Retrograde is something
the body is **doing**, so it is drawn as a line with a direction. Field and line are different
mark classes; they can never be mistaken for two flavours of one wash, and they compose without
argument when both are true.

### 4.2 Combustion — the rim

The subject is lost in the Sun's glare. You cannot see the body; you see the light around it. So
the day view is drawn as if light were entering from both edges, with the middle — where the
reading is — in relative shadow.

```
 x:0     16                                                   304  320
   ┌────────────────────────────────────────────────────────────────┐ y:0
   │▓▒                                                            ▒▓│  ← mask 0 at y:0
   │▓▒░  THURSDAY 20 AUGUST 2026 · GURUVARA                     ░▒▓│  12+14
   │▓▒░                                                          ░▒▓│  ← mask full at y:24
   │█▓▒  Mangala                                                 ▒▓█│  24
   │█▓▒                                                          ▒▓█│  12
   │█▓▒  Longitude                                118° 42′ 07″   ▒▓█│  24
   │█▓▒  Motion                                          Direct  ▒▓█│  24
   │█▓▒  Speed                                    +0.6182 °/day  ▒▓█│  24
   │█▓▒                                                          ▒▓█│
   │█▓▒  From Sun                                      11° 02′   ▒▓█│  24
   │█▓▒  Combust — inside the 12° orb                            ▒▓█│  18
   │█▓▒                                                          ▒▓█│
   │▓▒░                                                          ░▒▓│  ← mask fades from y:240
   │▒░                                                            ░▒│
   └────────────────────────────────────────────────────────────────┘ y:264
    └── 16 ──┘                                          └── 16 ──┘
    warm light                                          warm light
         └──────────── content column, deepened 10% ──────────┘
```

Right gutter, 4× zoom, showing the horizontal falloff:

```
 x:  304          308          312          316          320
      │            │            │            │            │
  α:  0.00  0.05  0.11  0.16  0.21  0.26  0.32  0.37  0.42
      ░░░░  ░░░▒  ░▒▒▒  ▒▒▒▒  ▒▒▓▓  ▓▓▓▓  ▓▓██  ████  ████
      └──── linear, 16px, --glare over the composite ground ───┘
      ┆
      └ content column ends here; the deepening starts here going left
```

| Property | Value |
|---|---|
| Element | `.state-field.is-combust::before`, `position: absolute; inset: 24px 0 0 0` |
| Left rim | `linear-gradient(to right, var(--glare-edge) 0, transparent 16px)` |
| Right rim | `linear-gradient(to left, var(--glare-edge) 0, transparent 16px)` |
| `--glare-edge` | `rgba(240, 201, 164, 0.42)` |
| Vertical mask | `linear-gradient(to bottom, transparent 0, #000 24px, #000 216px, transparent 240px)` |
| Content deepening | `.state-field.is-combust::after`, `inset: 24px 16px 0 16px`, `background: var(--glare-deepen)` |
| `--glare-deepen` | `rgba(0, 0, 0, 0.10)` |
| Vertical cost | **0** |
| Motion | none |
| Scroll behaviour | fixed to the 264px stage; does not scroll with `.detail` |

Why each number:

- **16px** is the gutter, exactly. The warm ink reaches zero at x = 16 and x = 304, so it never
  crosses into the column any text occupies. This is not a margin of safety; it is the rule.
- **The mask starts at y = 24** so the rim is never beside the date line. The date line is the only
  place `--accent` appears in the day view (the word `TODAY`, x = 16 → 46, y = 12 → 26). Gold and
  glare are therefore never within 24px of one another on any day, including a day that is both
  today and combust.
- **The mask ends at y = 240** so the rim fades out before `.region::after`'s 20px bottom fade,
  and the two do not fight.
- **0.42** is the lowest peak alpha that clears 3:1 against `G_flat` (§4.4). It is a ceiling as
  much as a floor: raising it buys nothing on `G_flat` and costs quiet on `G_dark`.
- **The 0.10 deepening is not decoration.** It is what makes the treatment safe: the ground behind
  every caption goes *down* by 10%, so no reading in the day view loses contrast on any backdrop.
  It is also the correct picture — the middle of a glare is where you see least.

### 4.3 Retrograde — the rail

Backwards motion. The mark runs down the right gutter with the reading, folds, and comes part of
the way back. A hairpin: one path, one turn, unequal legs, so it reads as a return and not as a
bracket.

```
 x:   304        308  310        314  316        320
        │          │    ╷          ╷    │          │        region y
   ┌────┴──────────┴────┼──────────┼────┴──────────┴───┐
   │  THURSDAY 20 AUGUST│2026 · GUR│UVARA              │      0
   │                    ╵          ╵                   │
   │  Mangala           ┆          │           [ ℞ ]   │     24  ← rail starts
   │                    ┆          │                   │
   │  Longitude         ┆   118° 42│ 07″               │
   │  Motion            ┆     Retro│rade               │
   │  Speed             ┆   −0.0142│°/day              │
   │                    ┆          │                   │
   │  Tithi             ┆  Shukla A│shtami             │
   │                    ┆    04:31 │ 07:12             │
   │                    ┆          │                   │
   │  Sunrise           ┆      06:0│4                  │     86  ← return ends
   │  Rise              ┆      21:0│7                  │
   │  Set               ┆      09:3│3                  │
   │                    ╵          │                   │
   │  From Sun          ╷          │                   │
   │                    └──────────┘                   │    198  ← fold
   │                                                   │
   └───────────────────────────────────────────────────┘    264
                        ↑          ↑
                    309.5      314.5
```

4× zoom of the fold, with the keyline shown:

```
              309.5   314.5
                ╷       ╷
   y:190   ░░░░░█░░░░░░░█░░░░░       █ = --marker 1px, 0.55
   y:194   ░░░░░█░░░░░░░█░░░░░       ░ = rgba(0,0,0,0.45) keyline, 3px
   y:198   ░░░░░███████████░░░░
   y:202   ░░░░░░░░░░░░░░░░░░░
                └── 5px ──┘
```

| Property | Value |
|---|---|
| Element | inline `<svg class="state-field__rail" aria-hidden="true">`, 8 × 184 |
| Placement | `position: absolute; top: 24px; right: 4px` → box spans x = 308 → 316 |
| Path | `M 6.5,0 L 6.5,174 L 1.5,174 L 1.5,62` |
| Down leg | x = 314.5, region y 24 → 198 (174px) |
| Fold | y = 198, x = 309.5 → 314.5 (5px) |
| Return leg | x = 309.5, region y 198 → 86 (112px) |
| Stroke | 1px `--marker`, `opacity: 0.55`, `stroke-linejoin: round`, `stroke-linecap: butt` |
| **Keyline** | the same path, painted first, 3px `rgba(0,0,0,0.45)`, same join |
| Gutter centring | hairpin centre x = 312 = gutter centre; 5.5px clear of the content column, 5.5px clear of the panel's inner edge |
| Vertical cost | **0** |
| Motion | none |

Why each number:

- **Unequal legs (174 down, 112 back).** Equal legs read as a bracket or a staple. The return
  stopping short of where it started is what makes it a *return*.
- **The fold is at the bottom**, not the top: the eye reads the day view downward, meets the turn,
  and comes back. The turn is where the reading already is by then.
- **5px between the legs** is the smallest gap at which a 1px pair at 2× still resolves as two
  strokes rather than a thick one.
- **The keyline is not optional.** Without it the rail measures 2.08:1 in the one position that
  matters — sitting on the bright part of the rim over a white desktop (§4.4). With it, 7.01:1.
- **The rail belongs to the view, not to the content**, so it does not scroll. A mark that scrolled
  away would say the state stops halfway down the day.

### 4.4 Contrast — every figure

Rim ink, `--glare` `#F0C9A4` at peak α 0.42, against the untreated ground beside it:

| Ground | Composite at peak | Ratio | Verdict |
|---|---|---|---|
| `G_flat` `#0A0A0B` | `#6B5A4A` | **3.01** | ≥ 3:1 |
| `G_dark` `#141414` | `#706052` | **3.06** | ≥ 3:1 |
| `G_lite` `#393939` | `#867566` | **2.62** | below 3:1 — see note |

Note, and it is the same argument DESIGN §4.1 already makes for `--disc-ring` at 2.42: the rim is
never a sole carrier. On any day it is drawn, the day view also prints `From Sun` with the
separation, the caption `Combust — inside the N° orb`, and the spoken label. 2.62 occurs only on a
white desktop, and only against the pixel immediately inside the rim.

Rim peak against the deepened content column — the edge the eye actually reads:

| Ground | Rim peak L | Column L | Ratio |
|---|---|---|---|
| `G_flat` | 0.1096 | 0.00274 | 5.62 |
| `G_dark` | 0.1243 | 0.00615 | 3.05 |
| `G_lite` | 0.1886 | 0.03454 | 2.86 |

Text, with the combustion field applied. **Every value goes up:**

| Token | `G_flat` before → after | `G_dark` before → after | `G_lite` before → after |
|---|---|---|---|
| `--text-primary` | 16.94 → 17.05 | 15.77 → 16.30 | 9.89 → **10.63** |
| `--text-secondary` | 7.71 → 7.76 | 7.18 → 7.42 | 4.50 → **4.84** |
| `--text-tertiary` | 5.41 → 5.44 | 5.03 → 5.20 | 3.16 → **3.42** |

Retrograde rail, `--marker` 0.55 over its 0.45 black keyline:

| Situation | Local ground | Ratio |
|---|---|---|
| Rail alone, `G_flat` | keyline over `#0A0A0B` | 11.9 |
| Rail alone, `G_dark` | keyline over `#141414` | 11.4 |
| Rail alone, `G_lite` | keyline over `#393939` | **9.86** |
| Rail on the rim, `G_lite` (worst case in the app) | keyline over rim α 0.30 | **7.01** |
| *Same, keyline omitted* | rim α 0.30 over `#393939` | *2.08 — why the keyline exists* |

### 4.5 The hue

`--glare: #F0C9A4`. sRGB (240, 201, 164). HSL hue 29°, saturation 73%, lightness 79%.
*Superseded: the token shipped as `#e2603a` (D-023). The reasoning below is kept as filed; the
numbers in it describe the peach, not the orange.*

| Question | Answer |
|---|---|
| What does it depict? | Sunlight through atmosphere — the light a body is lost inside. Not a status. |
| Is it learnable-free? | Yes. There is no "combust = orange" convention to teach; the words carry the fact and the colour carries the picture. |
| Separation from `--accent` `#D8B36A` | Hue 29° vs 40°. More decisive: different form (16px soft gradient vs 1px ring / 13px numeral / one word), different place (gutter vs content column), different alpha regime (≤ 0.42 vs 1.00). |
| Can they collide? | No. The rim's mask starts at y = 24; `TODAY` occupies y = 12 → 26 at x = 16 → 46. Minimum separation 24px horizontally at the closest point. |
| Greyscale | At α 0.42 the chroma contribution is ~4% of the composite. In greyscale the rim is a luminance gradient and reads identically. |
| Colour vision deficiency | Deutan/protan collapse the hue toward the achromatic axis; the rim survives as luminance. Nothing is lost, because nothing was encoded. |
| First thing discarded | Under `prefers-contrast: more` the hue drops to `#FFFFFF` and the geometry is untouched (§6.4). |

### 4.6 When both are true

Both draw. They are on different layers and different mark classes.

```
 x:0     16                                                   304  320
   ┌────────────────────────────────────────────────────────────────┐
   │█▓▒  Mangala                                        [ ℞ ]  ▒│▓█│  ← rail inside the rim
   │█▓▒                                                        ▒│▓█│
   │█▓▒  Longitude                                118° 42′ 07″ ▒│▓█│
   │█▓▒  Motion                                     Retrograde ▒│▓█│
   │█▓▒  Speed                                    −0.0142 °/day▒│▓█│
   │█▓▒                                                        ▒│▓█│
   │█▓▒  From Sun                                      04° 18′ ▒│▓█│
   │█▓▒  Combust — inside the 12° orb                          ▒│▓█│
   │█▓▒                                                        ▒└─┤
   └────────────────────────────────────────────────────────────────┘
```

- The rim is soft, symmetric, and has no direction. The rail is hard, one-sided, and folds. At a
  glance the pair reads as *a line inside a light*, which is the thing being described.
- The keyline is what keeps the rail legible where the rim is brightest. 7.01:1 (§4.4).
- Words are unchanged and both are present: the `Motion` row reads `Retrograde`, the speed carries
  its `−`, the `℞` chip is in the headline, and the combustion caption is under `From Sun`.
- No layout changes. The `℞` chip stays right-aligned to x = 304; the rail is outside it.

### 4.7 The nine subjects, and both calendars

| Subject | Combustion field | Retrograde rail |
|---|---|---|
| Surya | never — no orb (D-019) | never — the Sun is never retrograde |
| Chandra | when the caption prints (§4.8) | never |
| Mangala, Budha, Guru, Shukra, Shani | when the caption prints | when `retrograde` |
| Rahu, Ketu | never — no orb (D-019) | **suppressed** — see open question 1 |

- ~~The nodes are retrograde on roughly 95% of days. A surface treatment that is true almost always
  stops being a state and becomes the subject's identity.~~ **Superseded: the nodes are not
  suppressed, and the 95% was never measured.**

  Measured over twenty years, daily: a **mean** node is retrograde on 100% of days,
and a **true** node — Chandra's default — on **74.1%**, turning direct about
twenty-five times a year for a little under four days at a time. The true node
oscillates about the mean with a fortnightly term, and that oscillation outruns
the mean retrograde rate for part of every half draconic month.

  So the state changes, and often. The suppression was also a disagreement with the grid, which
  never had one: a cell drew the bracket for Rahu and Ketu from the same flag while the day it
  opened drew nothing. Under a mean node the rail is permanent, and that is truthful rather than
  noisy — it is the model the user chose.
- **Both calendars, by construction.** `.state-field` lives on `.region`, which is identical in
  solar and lunar mode. The day view differs between the two only by the Tithi block and the vara
  name (D-021 §7), neither of which touches the gutters.

### 4.8 The single condition

**The field is drawn if and only if the day view prints the corresponding words.**

| Treatment | Condition |
|---|---|
| Combustion rim | `combustion.orb !== null && combustion.combust` — exactly the condition on `CombustionBlock`'s caption |
| Retrograde rail | `detail.retrograde && graha ∉ {rahu, ketu}` — the condition on the `℞` chip, minus the nodes |
| Either | suppressed entirely when the day view is showing an `ErrorBlock` (§9.3) |

One condition, one place, testable. The mark and the words cannot disagree, which is the same
guarantee `the_combustion_mark_and_the_day_it_opens_agree` gives the grid (D-019).

### 4.9 Implementation

New tokens, `src/styles/tokens.css`:

```css
:root {
  /* Depiction, not encoding: sunlight through atmosphere. The rim's form is
     the carrier; the warmth is the picture. Never used outside the two 16px
     gutters, and never within 24px of --accent. */
  --glare-edge:   rgba(240, 201, 164, 0.42);
  --glare-deepen: rgba(0, 0, 0, 0.10);

  /* A mark on vibrancy is a pair: a light stroke over a dark keyline. Neither
     half clears 3:1 alone across the ground's range. */
  --rail:     rgba(200, 200, 206, 0.55);
  --rail-key: rgba(0, 0, 0, 0.45);
}

@media (prefers-contrast: more) {
  :root {
    --glare-edge:   rgba(255, 255, 255, 0.58);
    --glare-deepen: rgba(0, 0, 0, 0.16);
    --rail:         rgba(200, 200, 206, 0.85);
    --rail-key:     rgba(0, 0, 0, 0.65);
  }
}
```

`src/styles/panel.css`:

```css
/* One layer, behind the day view's text, pinned to the 264px stage. It never
   scrolls with the content and never costs a row: the whole treatment lives in
   the two 16px gutters the day view has never used. */
.state-field {
  position: absolute;
  inset: 0;
  z-index: 0;
  pointer-events: none;
}

.detail { position: relative; z-index: 1; }

/* The scroll fade belongs on top of the field, not under it. */
.region::after { z-index: 2; }

/* Combustion: light entering at the two edges, and the column the reading sits
   in going darker by the same act. The warm ink reaches zero at the content
   edge, so no text ever stands on a lightened ground. Masked to start below the
   date line, which is the one place --accent appears in this view. */
.state-field.is-combust::before {
  content: "";
  position: absolute;
  inset: 24px 0 0 0;
  background:
    linear-gradient(to right, var(--glare-edge) 0, transparent 16px),
    linear-gradient(to left,  var(--glare-edge) 0, transparent 16px);
  -webkit-mask-image: linear-gradient(to bottom, transparent 0, #000 24px, #000 216px, transparent 240px);
          mask-image: linear-gradient(to bottom, transparent 0, #000 24px, #000 216px, transparent 240px);
}

.state-field.is-combust::after {
  content: "";
  position: absolute;
  inset: 24px 16px 0 16px;
  background: var(--glare-deepen);
}

/* Retrograde: down, fold, part of the way back. Unequal legs, so it reads as a
   return rather than a bracket. */
.state-field__rail {
  position: absolute;
  top: 24px;
  right: 4px;
}
```

`src/components/DayDetail.tsx` — one element, no restructuring:

```tsx
<Show when={combustField() || retroField()}>
  <div class="state-field" classList={{ "is-combust": combustField() }} aria-hidden="true">
    <Show when={retroField()}>
      <svg class="state-field__rail" width="8" height="184" viewBox="0 0 8 184">
        <path d="M 6.5,0 L 6.5,174 L 1.5,174 L 1.5,62"
              fill="none" stroke="var(--rail-key)" stroke-width="3" stroke-linejoin="round" />
        <path d="M 6.5,0 L 6.5,174 L 1.5,174 L 1.5,62"
              fill="none" stroke="var(--rail)" stroke-width="1" stroke-linejoin="round" />
      </svg>
    </Show>
  </div>
</Show>
```

The element is a sibling of `.detail` inside `.region`, not a child of `.detail`. It carries
`aria-hidden` because every fact it draws is already spoken by the rows beside it (§8.1).

---

## 5. Direction 1 — rules *(the conservative option)*

No hue, no surface change, no new token, no new element in the region. The day view borrows the
grid's own two rules, which the user has already learnt from six weeks of cells.

DESIGN §6.3 gives the grid two marks:

| Grid mark | Meaning | Form |
|---|---|---|
| Rule under the numeral, inset | combustion | inset, short, attached to the thing |
| Rule at the cell's bottom edge, full-bleed | retrograde | full-bleed, long, at the boundary |

Direction 1 transplants both, at day-view scale.

```
 x:0     16                                                   304  320
   ┌────────────────────────────────────────────────────────────────┐
   │  THURSDAY 20 AUGUST 2026 · GURUVARA                            │  14
   │                                                                │   4
   │  Mangala                                          [ ℞ ]        │  24
   │  ───────                                                       │   ← combustion, inset,
   ├────────────────────────────────────────────────────────────────┤   ← retrograde, full-bleed
   │                                                                │  12
   │  Longitude                                    118° 42′ 07″     │  24
   │  Motion                                         Retrograde     │  24
```

| Mark | Geometry | Ink | Cost |
|---|---|---|---|
| Combustion | `.detail__phase::after`, `left: 0; right: 0; bottom: -1px; height: 1px` — the width of the name, 62px for `Mangala` | `--marker` at 0.55 | 0 (uses the headline row's 5px of slack under a 17/22 line in a 24px row) |
| Retrograde | `.detail__headline::after`, `left: -16px; right: -16px; bottom: -6px; height: 1px` — reaches x = 0 → 320 through the day view's own 16px padding | `--marker` at 0.50 | 0 (uses the 12px gap between the headline row and the first block) |

Contrast, `--marker` at 0.55:

| Ground | Ratio |
|---|---|
| `G_flat` | 4.16 |
| `G_dark` | 4.13 |
| `G_lite` | 3.25 |

All three clear 3:1 without a keyline, because a 1px rule at 0.55 is dense enough on its own —
which is exactly why the field in Direction 2 needs one and this does not.

**What restraint costs.** Two hairlines. The surface does not change; the panel does not *look*
different; it looks annotated. Held next to a non-combust day the difference is a 62 × 1px line
you have to be looking for. This is a correct answer to "the state must be legible" and not an
answer to "the day view should look different." It is here so the owner can price the gap.

**What it is good at, and Direction 2 should keep:** one vocabulary across two surfaces. Inset
rule = attached to a thing = combustion. Full-bleed rule = at the boundary = retrograde. If
Direction 2 is built, Direction 1 remains the correct treatment for any future third state that
does not deserve the whole surface.

---

## 6. Degradation

### 6.1 No material

`apply_vibrancy` fails, or the platform is not macOS, or the OS declines the effect.

**This is currently broken and the fix belongs in this change.** `panel.rs` logs the failure and
carries on; the window stays `.transparent(true)` and the panel paints `rgba(10,10,11,0.28)`. The
result is not "less at home next to the system's popovers" as the comment claims — it is the
desktop showing through at 72%, with every contrast figure in DESIGN §4.1 void.

| Step | Change |
|---|---|
| `panel.rs` | `apply_material` returns `bool`; `create` stores it in `AppState` |
| IPC | the front end reads it once at startup, alongside the other panel facts |
| `panel.css` | `.panel.is-flat { background: var(--ground); }` — opaque `#0A0A0B` |
| Effect on this spec | the ground collapses to `G_flat` exactly; every figure becomes the `G_flat` column, which is the most generous one |

Nothing else changes. The rim, the deepening and the rail are all specified relative to whatever
the ground is, so they are correct on an opaque panel with no edit.

### 6.2 Reduced transparency

`prefers-reduced-transparency: reduce`. macOS also makes `NSVisualEffectView` draw opaque on its
own, but the resulting tint is not a value this document can name, so the panel takes the
deterministic route:

```css
@media (prefers-reduced-transparency: reduce) {
  .panel { background: var(--ground); }
}
```

Ground becomes `G_flat`. Rim 3.01:1, rail 11.9:1, all text at its DESIGN §4.1 figures. No
geometry changes.

### 6.3 Reduced motion

Nothing in Direction 2 moves. The field appears and disappears with `.detail`'s existing
transitions:

| Event | Behaviour | Under `prefers-reduced-motion` |
|---|---|---|
| Day view opens | inherits `detail-in`, 140ms opacity, 55% delay | 80ms, no delay (§8.3, already in `tokens.css`) |
| Different day selected, state changes | inherits the 60ms out / 100ms in text cross-fade (§8.1) | 0ms out / 80ms in |
| Anything else | — | — |

No new entry in the §8.1 table and no new entry in §8.2's forbidden list. Nothing loops, pulses,
shimmers or overshoots, because nothing moves at all.

### 6.4 Increased contrast

`prefers-contrast: more`, per the token block in §4.9:

| Property | Normal | Increased | Ratio change |
|---|---|---|---|
| Rim peak | `rgba(240,201,164,0.42)` | `rgba(255,255,255,0.58)` | `G_flat` 3.01 → **6.87**; `G_lite` 2.62 → **4.31** |
| Rim hue | 29° warm | **achromatic** | the depiction is the first thing dropped; the form is untouched |
| Content deepening | `rgba(0,0,0,0.10)` | `rgba(0,0,0,0.16)` | tertiary on `G_lite` 3.16 → **3.60** |
| Rail stroke | `--marker` 0.55 | `--marker` 0.85 | on the rim over `G_lite` 7.01 → **8.9** |
| Rail keyline | `rgba(0,0,0,0.45)` | `rgba(0,0,0,0.65)` | — |
| Geometry | — | unchanged | — |

The hue going first is the point of the amendment. If colour only ever depicted, removing it can
only cost a picture, never a fact.

### 6.5 Grayscale, colour vision deficiency, and the tray

| Condition | Effect |
|---|---|
| Grayscale display | rim becomes a luminance gradient, identical in structure; rail is already achromatic |
| Deutan / protan / tritan | same — the hue contributes ~4% of the composite at α 0.42 and carries nothing |
| Screenshot at 1× | 1px rail with a 3px keyline survives; the rim is a gradient and does not alias |
| Screenshot at 2× | no hairline is thinner than 1 CSS px, so nothing disappears |
| Menu bar | unaffected. D-022 stands: the tray carries `℞` and nothing else, and combustion never reaches it |

### 6.6 Never a sole carrier

Extends DESIGN §11.3:

| Meaning | Non-textual carrier | Textual carrier, always present |
|---|---|---|
| Combustion, day view | warm rim in both gutters, content column deepened | `From Sun` row with the separation, **and** the caption `Combust — inside the N° orb`, **and** the spoken row label |
| Retrograde, day view | hairpin rail in the right gutter | `℞` chip in the headline, **and** the `Motion` row reading `Retrograde`, **and** the `−` on the speed, **and** the `Retrograde station` event row on a station day |

`.state-field` is `aria-hidden`. It adds nothing to the spoken day view because everything it
draws is already in the rows beside it, and a live region that announces a decoration twice is
worse than one that stays quiet.

---

## 7. Rejected alternatives

### 7.1 Rejected on contrast

| # | Alternative | Fails |
|---|---|---|
| R-1 | Tint the whole panel warm when combust | Lightens the ground under every caption. `--text-tertiary` is already 3.16:1 on `G_lite`; a 10% warm tint takes it to 2.69. Below 3:1 with no mitigation available. |
| R-2 | Warm bloom behind the graha's name (`text-shadow`) | At α 0.35 the headline falls to 4.09:1 on `G_lite` — below AA. At α 0.20 it clears 6.04:1 and is not visible at 6px spread. There is no alpha that is both compliant and legible. See §7.2. |
| R-3 | `backdrop-filter: blur()` over the day view for combustion | A second blur over the material's blur, running on every frame of a 264px scroller. Text over an unpredictable double-blurred ground has no computable contrast figure at all. |
| R-4 | Any state colour behind text | Same as R-1 generalised. On vibrancy the only safe operation behind text is darkening. |

### 7.2 Rejected on legibility

| # | Alternative | Fails |
|---|---|---|
| R-5 | Set the graha's name hollow for combustion (`-webkit-text-stroke: 0.6px`, transparent fill) | The most beautiful idea in the document: the limb survives, the disc does not, which is exactly what combustion is. But a 0.6px stroke at 17/600 has roughly a fifth of the ink of the filled form and no WCAG model covers it. The headline of the view is the wrong place to spend legibility to convey a state that is already in words two rows down. |
| R-6 | Mirror the headline row for retrograde (name right, `℞` left) | The most *legible* retrograde idea, and still rejected: it breaks the panel's left rag, it reads as a layout bug before it reads as a state, and it makes horizontal position carry a second meaning — which DESIGN §1.3 spends carefully on one. |
| R-7 | Draw the retrograde loop (the S-path a retrograde planet traces) as a background figure | Illegible at 16px of gutter. At panel width it stops being a diagram and becomes wallpaper. |
| R-8 | Diagonal hatch or texture behind the day view | DESIGN §1.3: decoration that could have been a rule. Also moirés against the 2× grid. |

### 7.3 Rejected on the design system

| # | Alternative | Fails |
|---|---|---|
| R-9 | Dim the day view when combust | D-020 explicitly. Opacity is spoken for — it is how a cell says it belongs to the neighbouring month — and a dimmed panel reads as disabled. |
| R-10 | `--accent` gold for the glare | D-020. Gold means today and nothing else. |
| R-11 | A coloured chip or badge beside the name | D-020 as amended: the colour would be the carrier, which is encoding. Also competes with the `℞` chip for the one slot at x = 304. |
| R-12 | Change the panel's border or radius for the state | DESIGN §4.1 guarantees `--border` (1.33:1) carries no meaning and can be deleted without ambiguity. Making it carry a state voids that guarantee. |
| R-13 | Reverse `.region::after`'s scroll fade for retrograde | The fade tells the user the content continues below. Inverting it to mean "backwards" makes the panel lie about its own scroll position. |
| R-14 | Mirror the ledger for retrograde — labels right, values left | DESIGN §1.6 (numbers never jitter): every value in the block would shift as the selection moved between a direct and a retrograde day. Also §5.6 — right alignment is the whole reason tabular numerals pay off. |

### 7.4 Rejected on cost

| # | Alternative | Fails |
|---|---|---|
| R-15 | A `Combust` / `Retrograde` row in the field block | 24px each, and both facts are already printed — the caption under `From Sun`, and the `Motion` row. §2.3: the view is 156px oversubscribed before the panchanga fields land. |
| R-16 | A banner or strip above the field blocks | 24–40px. Same. |
| R-17 | Animate the glare — a slow drift, a breathing alpha | DESIGN §8.2, verbatim: nothing loops, pulses, breathes, shimmers or bounces. |
| R-18 | Narrow the content column by 4px on each side when combust, and fill the freed space with the rim | Genuinely tempting — the squeeze *is* the depiction, and §8 permits motion that explains a geometric change. Rejected because the content measure drops 288 → 280 and the longest value strings (`does not rise or set today`, `does not rise; read at noon`) already sit close to the label. Text never wraps in this panel (§3.3), so a collision has no graceful outcome. |

---

## 8. Verification

| # | Check | Method |
|---|---|---|
| V-1 | The field is drawn on exactly the days the caption prints | Unit test over a synthetic month per subject, asserting `is-combust` ⟺ `combustion.combust && orb !== null`. The grid already has `the_combustion_mark_and_the_day_it_opens_agree`; this is its day-view twin. |
| V-2 | Zero vertical cost | Measure `.detail`'s `scrollHeight` for the same day with and without the field. Must be equal. |
| V-3 | No warm ink in the content column | Sample the rendered panel at x = 16 → 304 on a combust day; every pixel's chroma must equal the non-combust render's. |
| V-4 | Contrast holds on the worst-case ground | Render the panel over a pure-white desktop, screenshot, and compute the tables in §4.4 from the actual pixels. This is also what settles open question 3. |
| V-5 | Gold and glare never meet | Assert the rim's mask origin (y = 24) is below the date line's box (y = 12 → 26) for every locale's weekday string. |
| V-6 | The flat fallback is opaque | Force `apply_material` to fail; assert the computed background of `.panel` is `rgb(10, 10, 11)` with alpha 1. |
| V-7 | Composition | Render Shani on a day that is both combust and retrograde; assert the rail clears 3:1 against its keyline at both leg positions. |

---

## 9. Open questions

1. ~~**Rahu and Ketu.**~~ **Closed: not suppressed.** The figure below was never measured and is
   wrong — see §4.7. The original text follows.

   The true nodes are retrograde on roughly 95% of days, so a rail on their
   panels would be the normal state rather than a state. This spec suppresses the rail for both
   and keeps the `℞` chip and the `Motion` row. The alternative is to draw it anyway for
   consistency across the nine, and accept that two panels look permanently marked. Suppression is
   assumed; consistency is defensible.

2. **The Moon in lunar mode.** D-021 removed the Moon's combustion rule from the *grid* in lunar
   mode as duplicated ink — the Moon is combust around Amavasya, which the numeral already names.
   The day view still prints the `Combust` caption in both modes, so §4.8's single condition draws
   the rim there too. Either the day view follows the grid (suppress, and break the
   one-condition rule), or the grid and the day view legitimately differ here because the tithi
   numeral is in the grid and not in the day view. Drawing it is assumed.

3. **The bright end of the ground model.** `G_lite` `#393939` is derived from modelling the
   dark-appearance `.popover` material as `0.80 × #1E1E1E + 0.20 × desktop`, not from a
   measurement. Every worst-case figure in §2.2 and §4.4 moves with it. One pixel probe of the
   shipped panel over a white desktop (V-4) confirms it or shifts every number by a known amount,
   and should be taken before the rim's peak alpha of 0.42 is frozen.
