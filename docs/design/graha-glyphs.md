# Graha glyphs: drawing the bodies instead of their symbols

Status: **design and measurement.** Nothing here is built. §2 is the recommendation; §10 is
what would have to be edited if it is accepted.

Answers: "replace the astrological symbols with glyphs that depict the actual body as seen from
space — Saturn with its rings, Jupiter with its bands, Mars with its polar cap. Rahu an asura
head, Ketu a naga head."

Works under [D-008](../DECISIONS.md#d-008) (template images, hence monochrome),
[D-011](../DECISIONS.md#d-011), [D-023](../DECISIONS.md#d-023),
[D-024](../DECISIONS.md#d-024), [D-030](../DECISIONS.md#d-030).
Expressed in the vocabulary of [DESIGN.md](../DESIGN.md) §7.1, §7.2, §11.2, §11.3.

Every number below was rendered and measured, not estimated. The measuring program, the sheets
and the raw table are listed in §12.

---

## 1. The problem, stated once

The seven bodies differ mostly by colour. D-008 makes tray glyphs macOS template images, which
carry shape in alpha alone, so colour is not available and is not being reconsidered. Everything
that separates one graha from another has to be silhouette, internal structure, or surface
marking.

Two of those three do not survive the sizes this app draws at.

| Surface | Glyph size | Design units per device pixel, 2x | …at 1x |
|---|---|---|---|
| Menu bar | 22 pt | 1.83 px/unit | 0.92 |
| Settings list | 18 px | 1.50 | 0.75 |
| Panel header | 16 px | 1.33 | 0.67 |
| Calendar cell | 14 px | 1.17 | 0.58 |

At the calendar cell on a 1x display one design unit is **0.58 px**. A 1.0-unit belt is 0.58 px
of ink. It does not disappear — it turns grey, which is worse, because a grey smear inside a
circle is indistinguishable from a slightly different grey smear inside a circle.

---

## 2. Recommendation

**Do not replace the seven planetary glyphs. Replace the two node marks — after one more pass on
Ketu, which is the one shape in this study that is not finished.**

| Scope | Verdict |
|---|---|
| The seven bodies drawn as bodies | **No.** Four of them measure as the same mark (§5.3) |
| Shani drawn as a ringed body | Works, in isolation. Held back — see below |
| Surya drawn as a rayed disc | Works, but it is a convention, not a depiction (§6.1) |
| Chandra | **Keeps the live phase disc.** It is already the only true depiction in the app (§6.2) |
| Rahu and Ketu as head marks | **Yes for Rahu. Ketu needs one more pass** (§6.8, §6.9) |

The reasoning in one line each:

- **Depiction and distinguishability pull in opposite directions here.** The thing that makes a
  planet look like a planet is a disc. Nine discs are one mark drawn nine times.
- **The features that separate real planets are interior.** Interior features are the first
  thing a rasteriser loses. Silhouette is the only channel that survives to 14 px, and only
  Saturn and the Sun have a silhouette that is not a circle.
- **The node pair is the one place the current set is measurably weak.** DESIGN.md §11.3 already
  concedes that Rahu and Ketu are separated by orientation alone. They are ☊ and ☋, the western
  node symbols, drawn as 180° rotations of each other. Iconography is a real improvement there,
  because Rahu and Ketu are not competing with six other discs — they are competing with each
  other.
- **Shani alone would put one picture in a set of eight symbols.** If the node pair goes in, a
  second vocabulary already exists and Shani may join it; on its own it is an odd mark out.

Second-best, if the pictorial direction is wanted anyway: the **mixed** set in §5.4 — rayed
Surya, phase Chandra, ringed Shani, two head marks, and the four current symbols kept for
Mangala, Budha, Guru and Shukra. It costs no measured distinguishability. It costs the rule.
That is a real cost: a set with one rule can be learnt, a set with two has to be memorised.

---

## 3. What was measured

A standalone program renders a candidate through the same `tiny-skia` calls `crates/glyph` uses,
at the sizes the app draws at, then measures the pixels. It parses SVG path data with the same
`M`/`L`/`C`/`Z` grammar `crates/glyph/src/path.rs` accepts, so a candidate that measures well is
a candidate that can be pasted into `glyphs.rs` unchanged.

Three measures, modelled on `crates/glyph/examples/icon_candidates.rs`:

| Measure | What it is | Why |
|---|---|---|
| **Coverage** | alpha-weighted ink ÷ slot area | optical weight in a row (D-008's 22 pt slot) |
| **Separation** | mean \|Δalpha\| between two glyphs' rasters, 0–1 | how different two marks actually are |
| **Interior peak** | strongest alpha strictly inside the limb (r < 6.0 units) | whether an interior feature survived at all |

Every glyph is measured at 44/36/32/28 px (2x) and again after the box filter to 1x, which is
what a non-Retina display is handed.

Separation deserves a word, because a number needs a floor to mean anything. The floor used here
is the **shipping set's own worst pair**: Budha against Shukra, 0.113 at the calendar cell on 1x.
That pair ships, and nobody has reported confusing them, so 0.113 is treated as "far enough
apart". Anything below it is worse than the worst thing that currently works.

---

## 4. Three findings that decide it

### 4.1 A depicted body is heavier than a symbol

The shipping nine cover **12.4 % to 18.3 %** of the slot. Measured, not quoted:

| Graha | Surya | Chandra | Mangala | Budha | Guru | Shukra | Shani | Rahu | Ketu |
|---|---|---|---|---|---|---|---|---|---|
| Coverage | .183 | .124 | .169 | .143 | .133 | .132 | .124 | .131 | .131 |

A depicted body is a closed circle plus something. A closed circle alone, stroked at radius 8.4
with the set's 1.8-unit stroke, is already **16.4 %**. Anything added lands outside the band. The
same circle *filled* is **38 %** — roughly two and a half times the set's average, and it
dominates a row so badly that no other reading of it is possible.

That rules out both easy answers. Solid bodies are too heavy; outlined bodies leave no budget for
the features that make them different.

**The way out is fill mode.** `Ink::Stroke` applies one global `STROKE_WIDTH` to the whole path,
so a stroked glyph cannot have a 1.8-unit limb and a 1.0-unit belt. A *filled* path can: the limb
is authored as an annulus (outer contour forward, inner contour reversed — nonzero winding makes
the hole) and each interior feature as a filled ribbon of its own width. No code changes; the
existing `Ink::Fill` and `FillRule::Winding` already do it, and SVG's default `fill-rule` is
`nonzero`, so `GrahaGlyph.tsx` renders it identically.

With a 1.8 limb and 1.0–1.15 interiors the whole depicted set lands in band:

| Graha | Surya | Chandra | Mangala | Budha | Guru | Shukra | Shani | Rahu | Ketu |
|---|---|---|---|---|---|---|---|---|---|
| Coverage | .158 | .278 | .177 | .185 | .187 | .146 | .172 | .176 | .204 |

Chandra is the live phase disc and is outside the band by construction — as it is today, and as
D-008 already accepts.

So weight is solvable. It is not what kills the idea.

### 4.2 Interior features die before the smallest size

Peak alpha strictly inside the limb, which is 1.00 if the feature rendered at full strength:

| Graha | 22 pt 2x | 18 px 2x | 16 px 2x | 14 px 2x | 14 px 1x |
|---|---|---|---|---|---|
| Mangala — solid polar cap | 1.00 | 1.00 | 1.00 | 1.00 | **0.62** |
| Budha — crater rim | 1.00 | 1.00 | 1.00 | 1.00 | **0.62** |
| Guru — two belts | 1.00 | **0.75** | 1.00 | **0.50** | **0.25** |
| Shukra — nothing, by design | 0 | 0 | 0 | 0 | 0 |

Jupiter's belts are at quarter strength in a calendar cell on a 1x display. They are not a mark
there; they are a tint. Mars' cap holds up better because a solid area survives downsampling in a
way a line does not — which is the general rule the sheets confirm: **area survives, line does
not.**

Evidence: `sheet-proposed.png`, row 5, right-hand block.

### 4.3 The four discs collapse into one mark

This is the finding that decides the question. Separation matrix for the depicted set at the
panel header size, 16 px at 2x. Lower is more confusable.

|  | Surya | Chandra | Mangala | Budha | Guru | Shukra | Shani | Rahu | Ketu |
|---|---|---|---|---|---|---|---|---|---|
| **Surya** | — | .266 | .211 | .236 | .236 | .227 | .144 | .223 | .230 |
| **Chandra** | .266 | — | .178 | .184 | .175 | .191 | .235 | .201 | .167 |
| **Mangala** | .211 | .178 | — | **.054** | **.067** | **.031** | .201 | .314 | .230 |
| **Budha** | .236 | .184 | **.054** | — | **.061** | **.038** | .232 | .271 | .236 |
| **Guru** | .236 | .175 | **.067** | **.061** | — | **.043** | .214 | .265 | .244 |
| **Shukra** | .227 | .191 | **.031** | **.038** | **.043** | — | .210 | .292 | .260 |
| **Shani** | .144 | .235 | .201 | .232 | .214 | .210 | — | .236 | .239 |
| **Rahu** | .223 | .201 | .314 | .271 | .265 | .292 | .236 | — | .153 |
| **Ketu** | .230 | .167 | .230 | .236 | .244 | .260 | .239 | .153 | — |

Mangala against Shukra is **0.031**. The floor is 0.113. The pair is three and a half times
closer than the worst pair that currently ships, and the number is the same at the menu bar's
full 22 pt as it is in a calendar cell — this is not a small-size failure that a bigger slot
fixes. It is the same shape drawn twice.

Read the block: Mangala, Budha, Guru and Shukra form a clique in which every pair is under 0.07.
The set has six marks in it, not nine.

Contrast the shipping set at the same size, where the closest pair is Budha–Shukra at 0.136 and
nothing is below it.

Evidence: `row-proposed.png` — four circles in the middle of the row, at all four sizes, in both
polarities.

---

## 5. Set-level comparisons

### 5.1 The three sets measured

| Set | What it is | Min separation, 16 px 2x | Min, 14 px 1x |
|---|---|---|---|
| `shipping` | the nine as built | 0.136 | **0.113** |
| `proposed` | all nine depicted | **0.031** | 0.032 |
| `mixed` | depiction only where it separates | 0.136 | 0.107 |

### 5.2 Shipping

Coherent. Every mark is a stroked line figure at one width, one cap, one join. Nothing is below
its own floor. Its admitted weakness is Rahu/Ketu, which are rotations of each other.

### 5.3 Proposed

Fails. Four glyphs are one glyph. Everything else about it works: the weights are in band, the
retrograde treatment survives, both polarities read, and the strongest three marks in the whole
study (Shani, Rahu, Surya) are in it.

### 5.4 Mixed

| Graha | Mark |
|---|---|
| Surya | rayed disc, new |
| Chandra | live phase disc, unchanged |
| Mangala, Budha, Guru, Shukra | current symbols, unchanged |
| Shani | ringed body, new |
| Rahu, Ketu | head marks, new |

Min separation 0.136 at 16 px, 0.107 at 14 px 1x. The 0.107 is Surya against Shani — both are now
"a round thing with pieces outside it", and they get closer than any shipping pair by a hair. Not
fatal; worth knowing.

The real cost is not measurable. `row-mixed.png` shows it: half the row is line symbols with long
descenders, half is centred pictorial marks. The set no longer states a rule.

---

## 6. Per graha

Common construction for the depicted set, so a path can be authored from this section alone:

| Property | Value |
|---|---|
| Grid | 24 × 24 units, origin top-left, y down |
| Commands | `M` `L` `C` `Z` only. No `A` — the parser rejects it by design |
| Arc approximation | cubic, `k = 0.5523` per quarter, quarter-circle maximum per segment |
| Ink | `Ink::Fill`, nonzero winding. A reversed subpath is a hole |
| **Limb** | annulus, centre radius **7.5**, width **1.8** → outer 8.4, inner 6.6 |
| Limb ink bounds | 3.60 … 20.40 on both axes; cap-height **16.8** |
| Interior feature width | **1.0** for a line, **1.15** for a dot; solid areas carry no width |
| Feature clearance | ≥ 2.0 units from the limb's inner edge |

### 6.1 Surya — a rayed disc

| Element | Geometry |
|---|---|
| Body | annulus, centre (12,12), radius **5.8**, width 1.8 |
| Rays | 8 bars at 45°, from radius **7.3** to **9.5**, width **1.5**, flat ends, first at 3 o'clock |
| Ink bounds | 2.50 … 21.50; cap 19.0 |
| Coverage | .158 |
| Drops as size falls | nothing. Eight rays hold to 14 px 1x |

**The Sun as it actually looks from space is a plain bright disc, and there are only two ways to
draw one. Both are already taken.**

| Drawn as | Measured | Verdict |
|---|---|---|
| A filled disc | .382 coverage; 0.394 from an outlined disc | Separates from the planets, but it **is the full moon** — D-008 draws `k > 0.98` as a solid disc with the ring omitted. And 38 % is two and a half times the set |
| An outlined disc | .146 coverage; **0.031** from Mangala, 0.038 from Budha | In band, and indistinguishable from the four planets |

The rayed disc is therefore not a depiction; it is the conventional sun sign, and this document
says so rather than pretending the corona justifies it.

It earns its place on separation (nearest neighbour 0.144, Shani) and on the fact that the rays
are *outside* the body, so nothing about them is an interior feature. It is the only glyph in the
set whose distinguishing marks are exempt from §4.2.

Cap-height 19.0 against the set's 16.8. Rays read light, so it does not measure as heavy — .158
against Shukra's .146 — but §7.2's documented spread of 16.54–18.60 would have to widen.

The gap between ray and limb is 0.6 units and closes under anti-aliasing at every size the app
uses; the mark is one connected shape in practice.

### 6.2 Chandra — keeps the phase disc

**Not part of this set.** Three reasons, in order of weight:

1. The phase disc is drawn from real illumination and changes daily (§7.1). It is the only mark
   in the app that is a depiction rather than a picture of one. Replacing it with a static
   cratered moon would trade live data for decoration.
2. The app would then show two different moons — a static one in the settings list and a live one
   in the menu bar — for the same graha.
3. A static cratered moon would be a disc with interior marks, which is the case §4.2 and §4.3
   dispose of. Budha, drawn exactly that way, sits 0.038 from a plain disc.

Where a static Chandra glyph is still needed (the settings row, `glyphs.rs::glyph`), keep the
filled crescent that ships.

**One collision to note.** At new moon the phase disc is a hairline ring — a plain circle. If
Shukra ever becomes a plain outlined disc, the two coincide. Another reason the depicted set is
not adopted.

### 6.3 Mangala — limb plus a solid polar cap

| Element | Geometry |
|---|---|
| Limb | standard |
| Cap | circular segment of radius 8.2 above y = **7.9**; solid, merges with the limb |
| Ink bounds | 3.60 … 20.40; cap-height 16.8 |
| Coverage | .177 |
| Drops as size falls | nothing to drop. The cap dims to 0.62 peak at 14 px 1x |

Drawn as a **solid cap**, not as a chord line, for two reasons. A line at the pole reads as a
band and puts Mars in Jupiter's family — measured at 0.067 apart, which is nothing. And a solid
area survives downsampling where a line does not (§4.2).

Rejected: north and south caps together. It reads as a two-banded planet, i.e. as Jupiter.

**This glyph is not adopted.** It sits 0.031 from Shukra. The cap is 2.6 % of the slot's ink and
it is the only thing distinguishing a quarter of the set.

### 6.4 Budha — limb plus a crater rim

| Element | Geometry |
|---|---|
| Limb | standard |
| Crater rim | annulus at (10.0, 10.1), radius **2.9**, width **1.0** |
| Mare | solid disc at (15.4, 15.2), radius **1.15** |
| Coverage | .185 |
| Drops as size falls | the mare first; below 16 px it is one dark pixel |

Three iterations, all recorded in `sheet-candidates.png`:

| Candidate | Why not |
|---|---|
| Three solid craters | Reads as a **face**. Two eyes and a mouth is a strong prior and the glyph loses to it |
| One large basin ring | Reads as an **eye**, or as a loading spinner |
| A smaller plain disc, size-coded | A glyph seen alone has no size reference. Mercury is only small next to something |
| One rim plus one mare | Shipped here. Least bad; still 0.038 from Shukra |

**Mercury and Venus cannot be told apart by depiction at any size this app uses.** Their real
difference is colour and albedo, and both are gone. This is the honest answer the brief asked
for.

### 6.5 Guru — limb plus two belts

| Element | Geometry |
|---|---|
| Limb | standard |
| Belts | bars at y = **8.6** and **15.4**, spanning the chord of radius 6.8, width **1.0** |
| Coverage | .187 |
| Drops as size falls | **the belts.** Peak 0.50 at 14 px 2x, 0.25 at 14 px 1x |

Four belts were tried and are a grey disc once halved. Two is the maximum the grid supports:
6.8 units centre-to-centre, which is 4.0 px at the panel header and 2.0 px at 1x.

The Great Red Spot was tried as an ellipse on the lower belt. It merges with the belt at every
size below the menu bar and adds nothing but weight.

Jupiter is the body the brief names first and it is the one that fails hardest, because bands are
the definition of an interior feature.

### 6.6 Shukra — the limb and nothing else

| Element | Geometry |
|---|---|
| Limb | standard, alone |
| Coverage | .146 |
| Interior peak | 0.00 at every size, by construction |

Venus is featureless in visible light. A plain disc is the truthful depiction and it is also what
every other glyph in this set degrades into. The Y-shaped ultraviolet cloud feature was tried; it
reads as a yin-yang and it is not something anyone has seen with their eyes.

The mark is unique in the set only by *absence*, which is the weakest possible claim: on a 1x
calendar cell Mangala's cap dims to 0.62 and Guru's belts to 0.25, and both approach the plain
disc from above.

### 6.7 Shani — a body inside a broken ring

The one unambiguous success in the study.

| Element | Geometry |
|---|---|
| Body | annulus, centre (12,12), radius **6.3**, width 1.8 |
| Ring | ellipse, semi-axes **9.9 × 3.0**, tilt **−20°**, ribbon width **1.3** |
| Occlusion | the ring is drawn only where it is *not* behind the body — outside radius 7.9, or on the near side |
| Ink bounds | 2.01 … 21.99 horizontally, 4.80 … 19.20 vertically |
| Cap-height | **14.4**; width **20.0** |
| Coverage | .172 |
| Drops as size falls | nothing. It reads at 14 px 1x |

Why it works when nothing else does: **the ring changes the silhouette.** It is not inside the
limb, so §4.2 does not apply to it. Nearest neighbour 0.144.

Rejected:

| Candidate | Why not |
|---|---|
| Full ellipse, no occlusion | The crossing lines mush at 1x; the body loses its edge |
| A straight bar through the disc | Reads as a **prohibition sign** |
| Larger body, radius 7.2 | Ring overruns the 24-unit grid |

The cost is the cap-height: 14.4 against the set's 16.8, because the grid is square and Saturn is
not. It is compensated by width — 20.0 units against 16.8 — so it does not measure light, but
§7.2's spread would have to widen at both ends.

### 6.8 Rahu — an asura head

The current mark is **☊**, the western ascending-node symbol: a bowl on two feet, opening
downward. Ketu is the same shape rotated 180°. DESIGN.md §11.3 lists the pair as one of only two
places in the app where a distinction is carried by orientation alone.

| Element | Geometry |
|---|---|
| Skull | closed contour, half-width 4.9, brow at y = 8.3, chin at y = 19.0 |
| Horns | two closed tapering shapes rising from the temples to points at (5.3, 5.1) and (18.7, 5.1) |
| Notch | the gap between the horns spans 4.2 units at its narrowest |
| Ink bounds | 5.28 … 18.72 horizontally, 5.12 … 19.04 vertically |
| Cap-height | 13.9; width 13.4 |
| Coverage | .176 |
| Ink | **solid**, not outlined |

Solid rather than outlined on purpose, and the rule is worth stating because it also separates the
two kinds of mark in the set: **a body is drawn as an outline because you see its limb; a node is
drawn as a solid because it is a shadow.** It also measures better: the solid holds 1.00 interior
alpha at 14 px on a 1x display, and `sheet-candidates.png` rows G1 and G2 show the outlined
version of the same silhouette thinning to grey at the same size.

Nearest neighbour 0.153 (Ketu), against the shipping pair's 0.147. The pair is further apart than
it is today and is no longer a rotation.

Variants tried, in `sheet-nodes.png`:

| Variant | Result |
|---|---|
| R1 — horned dome | Adopted. Reads at every size |
| R2 — same, with a fanged jaw | The fangs are gone below 16 px. Two extra curves for nothing |
| R3 — wider skull, horns further apart | Clearest of the three, but .205 coverage, outside the band |

**Risk, named:** R1 reads as a horned head. It also reads as a cat. There is no size at which
that ambiguity resolves, because both readings are the same silhouette.

### 6.9 Ketu — a naga head, unresolved

The current mark is **☋**, ☊ rotated. The brief asks for a serpent head.

Seven candidates were drawn and rendered. None of them reads as a serpent at 16 px.

| Variant | What it reads as at 16 px |
|---|---|
| K1 — frontal cobra hood, flared | an aeroplane |
| K2 — hood with the tips swept down | an aeroplane |
| K3 / K4 — head in profile, forked tongue | a balloon with a nick in it |
| K5 — rearing cobra, head left, body descending | a bird's head. **Least bad** |
| K6 — broad hood shield, no tail | the **account/user icon**. Unusable |
| K7 — hood with the tips turned up | a bird, and it borrows Rahu's horns |

The reason is structural and worth recording so it is not rediscovered. **A serpent's identity is
carried by small features** — the fork of the tongue, the eye, the shape of the snout, the scale
pattern of the hood. Every one of them is under 2 units, which is under 1.3 px at the panel
header. What survives is the gross outline, and the gross outline of any head is a blob.

Rahu escapes this because horns are a *large* protrusion. There is no equivalent large protrusion
that means "serpent". The hood is the only candidate, and a hood drawn large enough to read is a
pair of wings.

K5 is carried into the mixed set because it measures well — 0.153 from Rahu, .204 coverage,
asymmetric, not a rotation of anything — but it does not do what the brief asked. **This is the
open item.** Two directions not yet tried:

- A coiled serpent, one and a half turns. A spiral is a large-scale form and would survive; it is
  a body rather than a head.
- Accept the head does not read and lean on the pair instead: Rahu horned, Ketu the same skull
  without horns and with a forked tail below. The pair would then be legible as a pair even where
  neither is legible alone.

| Element | Geometry (K5, as measured) |
|---|---|
| Head | rounded wedge, snout at (3.7, 8.4), brow at (14.6, 4.5) |
| Hood | swells behind the head to (16.7, 10.7) |
| Body | descends right and down to (17.8, 20.5), 3.6 units wide at the foot |
| Tongue | two prongs at the snout, 3.2 units long, 1.9 apart at their roots |
| Ink bounds | 3.73 … 17.83 horizontally, 3.82 … 20.46 vertically |
| Coverage | .204 |

---

## 7. The four surfaces

The same path serves all four; only the box changes. `GrahaGlyph.tsx` draws the path the back end
serves, so nothing here can drift between the menu bar and the panel.

| Surface | Size | What holds | What is lost |
|---|---|---|---|
| Menu bar | 22 pt, 44 px at 2x | everything | nothing at 2x. At 1x, Guru's belts fall to 0.62 |
| Settings list | 18 px | everything at 2x | Guru's belts at 0.75; the name is beside the glyph, so it does not matter here |
| Panel header | 16 px | silhouettes | Budha's mare. The header names the graha in words |
| Calendar cell | 14 px | silhouettes only | Guru's belts (0.25 at 1x), Mangala's cap (0.62), Budha's crater (0.62) |

The calendar cell is the size that decides the design, and it is the one surface with **no
textual carrier beside the glyph** — the cell has a numeral and a symbol and nothing else. That is
why §4.3 is fatal rather than merely awkward: in a month grid the reader has to tell Mangala from
Shukra from a 14 px mark with no label anywhere near it.

The menu bar is the opposite case. Each tray item is its own glyph, seen alone, with a tooltip and
an accessibility label. A glyph that is only ambiguous *against another glyph* is safe there.

**View box.** Unchanged: 24 units square, centred on the glyph's own ink (`glyphs.rs::view_box`).
Two of the proposed marks need it more than anything shipping does — Shani's ink is 20.0 × 14.4
and Ketu's is 14.1 × 16.6, both well off the grid's centre.

---

## 8. Retrograde

`GLYPH_WITH_MARK_POINTS` shrinks the glyph to 16.5 pt and pins it top-left; `℞` goes bottom-right
at 10 pt with 0.8 pt clearance, placed by its own ink.

Rendered: `retro-proposed.png`. All nine survive. The mark stays a separate shape from every
glyph, including Shani, whose ring reaches x = 21.99 of the 24-unit grid and still clears the
mark's top bar — the same collision that Guru's baseline and Rahu's tail caused once and that
`MARK_CLEARANCE` exists to prevent.

Two consequences worth stating:

- A retrograde glyph is drawn at **0.75×**. Every threshold in §4.2 shifts down by a quarter. Guru
  retrograde in a menu bar at 1x is a disc with a hint of grey across it.
- The calendar's retrograde bracket is a dotted Ø18 ring centred on the glyph (D-024). A Ø16.8
  depicted disc at 14 px is 9.8 px across and clears it. Shani at 20.0 units wide is 11.7 px and
  still clears it, but the ring will read as loose above and below, because Shani's ink is wide
  and short and the ring is round.

---

## 9. Architecture: one path, one ink mode

Worth recording because it shaped every glyph above.

`glyphs::glyph()` returns one path and one `Ink`. `GrahaGlyph.tsx` sets either `fill` or `stroke`
for the whole path, never both. So a glyph cannot be a stroked circle with a filled cap.

This is not a limitation once you notice that **fill mode subsumes stroke mode**: an outline is
just an annulus, and authoring it as a filled shape is what gives each feature its own width
(§4.1). The depicted set is entirely fill mode and needs no change to `render.rs`, `path.rs` or
`GrahaGlyph.tsx`.

What it does need:

| Site | Change |
|---|---|
| `glyphs.rs::glyph` | ink mode per graha, not "Chandra is the only filled one" |
| `glyphs.rs` test `only_chandra_is_filled` | replaced by a test that pins each graha's mode |
| `glyphs.rs` `DOCUMENTED_BOUNDS` | new figures; `cap_heights_stay_within_the_documented_optical_spread` needs a wider range (14.4 – 19.0) |
| `DESIGN.md` §7.2 | new path data and verification table |
| `render.rs` test `every_graha_renders_visible_distinguishable_ink` | it asserts only that no two rasters are *identical*. Mangala and Shukra at 0.031 would pass it. It should assert a separation floor instead |

That last row is a finding in its own right, in the spirit of AUDIT.md's opening: the existing
test would have let this whole set through.

---

## 10. What D-008 would say afterwards

If the recommendation in §2 is accepted — and after §6.9's open item is closed, because the text
below claims a naga head that does not yet read as one — D-008 gains a fourth bullet and its
"Grahas" bullet is rewritten. Draft:

> ### D-008
> **Tray glyphs are macOS template images by default.**
>
> - Template images auto-invert with the menu bar appearance. A fixed-colour icon is invisible in
>   one of the two modes.
> - Moon: lit fraction opaque, unlit transparent, with a hairline full-disc ring so a new moon is
>   still a visible target.
> - **Grahas: the seven bodies keep their astrological symbols; Rahu and Ketu are drawn as an
>   asura head and a naga head.** Vector paths drawn in Rust, optically balanced at an 18 px
>   cap-height — not font glyphs, whose weights and baselines are inconsistent across the set.
> - **Depicting the bodies was designed, measured and rejected.** Template images carry shape in
>   alpha alone, and the planets differ mostly by colour, so a depiction has to separate them by
>   silhouette or by surface marking. Surface marking does not survive: at a 14 px calendar cell on
>   a 1x display, Jupiter's belts render at a quarter alpha. Silhouette works only where a body has
>   one — Saturn's rings, the Sun's rays. Measured on the raster, Mars, Mercury, Jupiter and Venus
>   drawn as bodies sit 0.03–0.07 apart on a 0–1 scale, against 0.113 for the closest pair that
>   ships. Four of the nine become one mark. The node marks are exempt because they depict no body
>   and compete with nothing but each other, which is the one distinction DESIGN.md §11.3 records
>   as carried by orientation alone.
> - Colour mode is available in settings, off by default, because coloured menu bar icons are
>   against platform convention.
> - Rendered at 2x into RGBA via `tiny-skia`, applied with `set_icon_with_as_template`.
>
> Supersedes nothing. Amends the Grahas clause. `docs/design/graha-glyphs.md` carries the
> measurements.

DESIGN.md §11.3's `Rahu vs Ketu` row would change from `glyph orientation` to `glyph silhouette`,
and the sentence beneath it — "Rahu/Ketu and today are the two cases where a colour or an
orientation does real work" — would drop Rahu/Ketu.

If the mixed set of §5.4 is taken instead, D-008 needs one more sentence saying that the set
deliberately holds two vocabularies and why, because otherwise the next reader will try to make it
consistent.

---

## 11. Risks, and what is not resolved

| # | Risk | Status |
|---|---|---|
| 1 | **Ketu does not read as a naga at 16 px.** Seven candidates, none succeeded | **Open.** §6.9 names two untried directions |
| 2 | Rahu reads as a horned head and also as a cat | Unresolvable by drawing. The panel names the graha; the tray has an accessibility label |
| 3 | Separation is a raster metric, not a perceptual one. It counts pixel difference, which over-weights position and under-weights structure | Sound for the negative result (0.031 is far below anything that ships) and weak for fine rankings. Not used for any |
| 4 | The 0.113 floor is one observed pair, not a study | Named as such. It is the best available anchor: it ships and works |
| 5 | Everything was judged on a light ground and a dark ground at true size, by eye, from PNGs. It was not judged in a real menu bar | The template mask makes both polarities the same alpha, so polarity is safe. Physical size on a Retina panel is half what the sheets show |
| 6 | The mixed set's Surya–Shani pair is 0.107, just under the floor | Small. Both are strong marks; the number is low because both are round with pieces outside |
| 7 | Cap-height spread would widen from 16.54–18.60 to 14.4–19.0 | Deliberate, if adopted. Shani cannot be as tall as the others; the grid is square and Saturn is not |
| 8 | No colour-mode check | Colour mode changes tint only, not geometry (§7.1). Nothing above depends on it |
| 9 | The nine were never rendered at the settings list's real row rhythm, only in a bare row | Would change nothing about §4.3, which fails at every size |

---

## 12. Evidence

Rendered with the same `tiny-skia` version and the same path grammar the crate uses. All paths
under
`/private/tmp/claude-501/-Users-parasdsingh-Repos-xplore/2ebde726-a4d0-4946-b9d1-902c1fc95922/scratchpad/`.

| File | What it shows |
|---|---|
| `glyphlab/` | the measuring program. Throwaway; not part of the repo |
| `graha-glyphs/measurements.csv` | every figure quoted above: bounds, coverage, separation, interior peaks |
| `graha-glyphs/paths.txt` | path data for every candidate, in the crate's grammar |
| `graha-glyphs/sheet-shipping.png` | the nine as built, four sizes × two pixel densities, magnified ×7 |
| `graha-glyphs/sheet-candidates.png` | the first exploration, stroked: solid and rayed suns, one and two polar caps, three crater treatments, two and four belts, three Saturns, two node treatments in both ink modes |
| `graha-glyphs/sheet-proposed.png` | the depicted nine at the same four sizes |
| `graha-glyphs/sheet-nodes.png` | three Rahu variants, seven Ketu variants |
| `graha-glyphs/sheet-mixed.png` | the §5.4 set |
| `graha-glyphs/row-*.png` | each set at **true size**, 22 pt / 18 / 16 / 14, dark ink on light and light ink on dark |
| `graha-glyphs/row-*-16px-zoom.png` | the same row at the panel header size, magnified ×6 |
| `graha-glyphs/retro-*.png` | each set at 16.5 pt with the `℞` mark, at 2x and 1x |

The judgements in §4.2, §6.4 and §6.9 come from the true-size rows and the ×7 sheets, not from
the magnified views alone. A shape that reads at 200 px and not at 22 is a failed design here,
and three of the candidates above failed exactly that way.

---

## 13. Path data

All fill mode, nonzero winding. `M` `L` `C` `Z` only; no `A`. Line breaks are whitespace
and carry no meaning.

Every string below was extracted back out of this document and run through
**`crates/glyph/src/path.rs` itself**, not through the study's own parser. All eight parse, none
contains an arc command, and the tight bounds each one reports are the figures quoted in §6 to
the hundredth of a unit. They can be pasted into `glyphs.rs` unchanged.

Chandra is absent: it keeps the live phase disc, which is generated rather than authored.

**Surya — rayed disc**

```
M 18.7,12 C 18.7,15.7 15.7,18.7 12,18.7 C 8.3,18.7 5.3,15.7 5.3,12 C 5.3,8.3 8.3,5.3 12,5.3
C 15.7,5.3 18.7,8.3 18.7,12 Z M 16.9,12 C 16.9,9.29 14.71,7.1 12,7.1
C 9.29,7.1 7.1,9.29 7.1,12 C 7.1,14.71 9.29,16.9 12,16.9 C 14.71,16.9 16.9,14.71 16.9,12 Z
M 19.3,12.75 L 21.5,12.75 L 21.5,11.25 L 19.3,11.25 Z M 16.63,17.69 L 18.19,19.25
L 19.25,18.19 L 17.69,16.63 Z M 11.25,19.3 L 11.25,21.5 L 12.75,21.5 L 12.75,19.3 Z
M 6.31,16.63 L 4.75,18.19 L 5.81,19.25 L 7.37,17.69 Z M 4.7,11.25 L 2.5,11.25 L 2.5,12.75
L 4.7,12.75 Z M 7.37,6.31 L 5.81,4.75 L 4.75,5.81 L 6.31,7.37 Z M 12.75,4.7 L 12.75,2.5
L 11.25,2.5 L 11.25,4.7 Z M 17.69,7.37 L 19.25,5.81 L 18.19,4.75 L 16.63,6.31 Z
```

**Mangala — limb + polar cap**

```
M 20.4,12 C 20.4,16.64 16.64,20.4 12,20.4 C 7.36,20.4 3.6,16.64 3.6,12
C 3.6,7.36 7.36,3.6 12,3.6 C 16.64,3.6 20.4,7.36 20.4,12 Z M 18.6,12
C 18.6,8.35 15.65,5.4 12,5.4 C 8.35,5.4 5.4,8.35 5.4,12 C 5.4,15.65 8.35,18.6 12,18.6
C 15.65,18.6 18.6,15.65 18.6,12 Z M 4.9,7.9 C 6.36,5.36 9.07,3.8 12,3.8
C 14.93,3.8 17.64,5.36 19.1,7.9 Z
```

**Budha — limb + crater rim + mare**

```
M 20.4,12 C 20.4,16.64 16.64,20.4 12,20.4 C 7.36,20.4 3.6,16.64 3.6,12
C 3.6,7.36 7.36,3.6 12,3.6 C 16.64,3.6 20.4,7.36 20.4,12 Z M 18.6,12
C 18.6,8.35 15.65,5.4 12,5.4 C 8.35,5.4 5.4,8.35 5.4,12 C 5.4,15.65 8.35,18.6 12,18.6
C 15.65,18.6 18.6,15.65 18.6,12 Z M 13.4,10.1 C 13.4,11.98 11.88,13.5 10,13.5
C 8.12,13.5 6.6,11.98 6.6,10.1 C 6.6,8.22 8.12,6.7 10,6.7 C 11.88,6.7 13.4,8.22 13.4,10.1 Z
M 12.4,10.1 C 12.4,8.77 11.33,7.7 10,7.7 C 8.67,7.7 7.6,8.77 7.6,10.1
C 7.6,11.43 8.67,12.5 10,12.5 C 11.33,12.5 12.4,11.43 12.4,10.1 Z M 16.55,15.2
C 16.55,15.84 16.04,16.35 15.4,16.35 C 14.76,16.35 14.25,15.84 14.25,15.2
C 14.25,14.56 14.76,14.05 15.4,14.05 C 16.04,14.05 16.55,14.56 16.55,15.2 Z
```

**Guru — limb + two belts**

```
M 20.4,12 C 20.4,16.64 16.64,20.4 12,20.4 C 7.36,20.4 3.6,16.64 3.6,12
C 3.6,7.36 7.36,3.6 12,3.6 C 16.64,3.6 20.4,7.36 20.4,12 Z M 18.6,12
C 18.6,8.35 15.65,5.4 12,5.4 C 8.35,5.4 5.4,8.35 5.4,12 C 5.4,15.65 8.35,18.6 12,18.6
C 15.65,18.6 18.6,15.65 18.6,12 Z M 6.11,9.1 L 17.89,9.1 L 17.89,8.1 L 6.11,8.1 Z
M 6.11,15.9 L 17.89,15.9 L 17.89,14.9 L 6.11,14.9 Z
```

**Shukra — limb alone**

```
M 20.4,12 C 20.4,16.64 16.64,20.4 12,20.4 C 7.36,20.4 3.6,16.64 3.6,12
C 3.6,7.36 7.36,3.6 12,3.6 C 16.64,3.6 20.4,7.36 20.4,12 Z M 18.6,12
C 18.6,8.35 15.65,5.4 12,5.4 C 8.35,5.4 5.4,8.35 5.4,12 C 5.4,15.65 8.35,18.6 12,18.6
C 15.65,18.6 18.6,15.65 18.6,12 Z
```

**Shani — body + broken ring**

```
M 19.2,12 C 19.2,15.98 15.98,19.2 12,19.2 C 8.02,19.2 4.8,15.98 4.8,12
C 4.8,8.02 8.02,4.8 12,4.8 C 15.98,4.8 19.2,8.02 19.2,12 Z M 17.4,12
C 17.4,9.02 14.98,6.6 12,6.6 C 9.02,6.6 6.6,9.02 6.6,12 C 6.6,14.98 9.02,17.4 12,17.4
C 14.98,17.4 17.4,14.98 17.4,12 Z M 21.91,8.39 C 22.2,9.18 21.69,10.24 20.47,11.38
L 19.24,10.95 C 20.36,10.09 20.88,9.35 20.69,8.84 Z M 19.3,12.33
C 16.86,14.1 13.22,15.68 9.74,16.47 C 6.26,17.25 3.48,17.13 2.44,16.14
C 1.4,15.15 2.26,13.44 4.7,11.67 L 5.82,12.33 C 3.62,13.7 2.76,14.91 3.57,15.49
C 4.38,16.08 6.73,15.95 9.75,15.17 C 12.76,14.38 15.97,13.05 18.18,11.67 Z M 18.92,7.04
C 20.57,7.13 21.63,7.61 21.91,8.39 L 20.69,8.84 C 20.51,8.33 19.65,8.09 18.25,8.15 Z
```

**Rahu — asura head**

```
M 12,8.32 C 14.88,8.32 16.88,10.16 16.88,12.88 C 16.88,16.16 14.72,19.04 12,19.04
C 9.28,19.04 7.12,16.16 7.12,12.88 C 7.12,10.16 9.12,8.32 12,8.32 Z M 7.68,10.88
C 6.32,8.96 5.52,7.2 5.28,5.12 C 7.2,6.08 8.8,7.52 9.92,9.28
C 9.04,9.6 8.32,10.16 7.68,10.88 Z M 16.32,10.88 C 17.68,8.96 18.48,7.2 18.72,5.12
C 16.8,6.08 15.2,7.52 14.08,9.28 C 14.96,9.6 15.68,10.16 16.32,10.88 Z
```

**Ketu — naga head (K5, provisional)**

```
M 3.73,8.43 C 5.04,6.55 7.11,5.42 9.18,5.23 C 10.68,3.73 12.94,3.35 14.63,4.48
C 16.7,5.8 17.45,8.43 16.7,10.68 C 16.14,12.38 15.01,13.32 13.69,13.5
C 15.2,15.57 16.7,17.83 17.83,20.46 L 14.26,20.46 C 13.13,18.02 11.81,15.95 10.31,14.26
C 8.8,12.56 7.3,11.62 5.8,11.44 C 4.86,11.25 4.1,10.87 3.73,10.31 L 6.92,9.74 L 3.73,8.43 Z
```
