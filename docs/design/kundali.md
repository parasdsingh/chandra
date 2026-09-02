# Kundali — a transit chart for now

Requested: "a visual kundali for the current date and time, accessed as a menu bar icon,
enabled in settings."

Status: **proposed.** Nothing here is built. Five decisions in §9 need the user before any of
it is.

The request leaves the hard parts open — which chart style, whether it draws houses, where it
lives, and what "for the current date and time" means when a kundali normally needs a birth.
This document settles those before code exists, per the project's rule that architecture
precedes implementation.

---

## 1. What is being drawn, precisely, and what it is called

### 1.1 It is not a birth chart

A janma kundali is cast for a birth instant and a birth place. Chandra has neither, and
acquiring them means a form, a stored document and a second surface — none of which a menu bar
app has.

What can be cast from what the app already knows is a **gochara chakra**: the positions of the
nine grahas at the current instant, at the resolved location.

- Gochara means the current movement of the grahas through the zodiac, recalculated
  continuously ([AstroPatri](https://astropatri.com/blog/what-is-gochar-or-transit-chart-and-how-to-read-it),
  [Wikipedia](https://en.wikipedia.org/wiki/Hindu_astrology) — "the grahas have continued to
  move around the zodiac, interacting with the natal chart grahas").
- Classically, gochara is read **against** something: from the natal lagna, or from the natal
  Moon (Chandra lagna). Both registers need birth data.

**So the honest statement of what this feature is:** a rashi chart of the sky now. It is one
half of a gochara reading — the transiting half — drawn on its own. It is complete and correct
as a statement of where the grahas stand; it makes no claim about any person.

That has to be said on the surface, not only here. §5.3 puts it in the header: the view is
titled `Gochara`, not `Kundali`.

### 1.2 The lagna

The ascendant is the point of the ecliptic rising on the eastern horizon. It depends on:

- the instant, to the second — it moves about 1° per 4 minutes of clock time,
- the observer's longitude — it enters the sidereal time directly,
- the observer's latitude — it enters the horizon geometry.

A transit lagna is a real quantity, not a substitute for a natal one. It turns over into the
next rashi roughly every two hours. It is what makes the chart a chart rather than a table:
without it there is no first house and nothing to count from.

### 1.3 What the chart shows

| Shown | Not shown | Why not |
|---|---|---|
| The twelve rashis, in fixed cells | Bhava numbers | No room, and the lagna mark is the anchor (§3.4) |
| Each of the nine grahas, in its rashi | Degrees within the rashi | No room, and they change while being read |
| Retrograde, as `℞` | Combustion | §10 |
| The lagna, as a diagonal in its cell | Any varga (navamsa, etc.) | §10 |
| The instant and the place | Any interpretation | §10 |

**Rashi names, not numbers.** The app has never printed a rashi as a number. Its three-letter
short forms already exist (`zodiac.rs::short` — `Ari`, `Tau`, …). Numbering signs 1–12 is the
North Indian convention (§3.1) and comes with that chart style, not with this one.

**Nine grahas, all of them.** Not the tray's enabled subset. The tray subset is a choice about
menu bar clutter; a chart missing a graha is a wrong chart.

---

## 2. What the engine can already answer, and what is missing

### 2.1 Already there

| Need | Where | Note |
|---|---|---|
| Nine sidereal longitudes at an instant | `Engine::positions(jd, &Graha::ALL)` | one lock, one batch |
| Rashi from a longitude | `Rashi::from_longitude` | exact by division |
| Retrograde | `Position::is_retrograde` | `speed < 0.0` |
| Provenance | `Position::source`, `Source::weakest` | D-006 |
| Configured ayanamsa and node type | `Engine` global `sid_mode`, `SiderealConfig` | D-003, D-027 |
| Whole-sign house counting | `standing.rs::houses_between` | already used for drishti |
| The resolved observer | `Almanac::location() -> Location` | D-007 chain |
| Nine glyphs, as path data to the front end | `crates/glyph`, `commands.rs::GrahaInfo` | DESIGN §7.2 |

`standing.rs` is worth naming twice: it already counts houses inclusively from an occupied
rashi, and its comment already records **why whole-sign** ("a graha aspects a house, and
whatever stands in it"). The chart needs the identical counting rule, so it inherits an
argument the project has already made and tested.

### 2.2 Missing: the ascendant

> **Verified first-hand, 30 August 2026.** Every claim in this section was
> checked against the installed crate and its vendored C before the epic was
> scheduled:
>
> - `swiss-eph-0.2.1/src/safe.rs:1057` — `houses()` calls plain `swe_houses`.
>   No `iflag`, so no `SEFLG_SIDEREAL`, so a tropical ascendant.
> - `swiss-eph-0.2.1/src/lib.rs:586-587` — `swe_houses_ex` and `swe_houses_ex2`
>   are declared `pub` externs. Reachable without vendoring or forking.
> - `vendor/swisseph/swehouse.c:229` and `:269` — `swe_houses_ex2` does branch
>   on `SEFLG_SIDEREAL`. The support is there; only the safe wrapper omits it.
> - `crates/ephemeris/src/engine.rs:437, 496` — we already call raw externs this
>   way for `swe_calc_ut` and `swe_rise_trans`, under the engine lock with a
>   SAFETY comment. Calling `swe_houses_ex` is the same pattern, not a new one.
> - `crates/ephemeris/src/observer.rs:31` — `as_se_geopos()` returns
>   `[longitude, latitude, elevation]`. `swe_houses_ex` takes **latitude first**.
>   The transposition trap below is real.


**Nothing in the workspace calls any house or ascendant function.** `grep` for
`swe_houses`/`ascendant`/`lagna` over `crates/` and `src-tauri/` returns only `standing.rs`'s
whole-sign *counting* helpers. This is the one real gap.

What it takes:

- **The FFI is reachable.** `swiss-eph` 0.2.1 declares `swe_houses`, `swe_houses_ex`,
  `swe_houses_ex2`, `swe_houses_armc`, `swe_houses_armc_ex2`, `swe_house_pos` and
  `swe_house_name` as `pub` externs in `src/lib.rs`. No upstream change is needed.

- **The crate's safe wrapper is unusable here.** `swiss_eph::houses(jd_ut, lat, lon, system)`
  calls plain `swe_houses`. Verified in the vendored `swehouse.c`: `swe_houses` computes ARMC
  and hands straight to `swe_houses_armc_ex2` with no sidereal branch, whereas
  `swe_houses_ex2` carries `if (iflag & SEFLG_SIDEREAL) { … sidereal_houses_trad(…) }`. The
  safe wrapper therefore returns a **tropical** ascendant. Using it would put the lagna about
  24° out — nearly a whole rashi — and it would look plausible.

- **So the engine calls the raw FFI itself**, under the existing lock, exactly as
  `Engine::illumination` already calls `se::swe_pheno_ut`. That keeps all `unsafe` inside
  `crates/ephemeris`, which is the crate's stated invariant.

```rust
/// Sidereal ascendant at an instant, for an observer.
///
/// Whole-sign houses ('W'): it is the classical bhava scheme, it is what
/// `standing::houses_between` already counts, and it is the only system in
/// `swehouse.c` with no polar failure branch (§8.3).
pub fn ascendant(&self, jd_ut: f64, observer: Observer) -> Result<f64>;
```

- **`SEFLG_SIDEREAL` is already defined** in `engine.rs` (`64 * 1024`) and is the flag to pass.
  `swe_houses_ex` then honours whatever `swe_set_sid_mode` last set, which is the user's
  ayanamsa. Nothing extra to wire.

- **Argument order is a trap.** `swe_houses_ex(tjd_ut, iflag, geolat, geolon, hsys, …)` takes
  **latitude first**. `Observer::as_se_geopos` returns `[longitude, latitude, elevation]`,
  because that is what `swe_rise_trans` wants — and its doc comment exists precisely to stop
  the swap being reintroduced. That helper **must not** be reused here. A silently transposed
  Bengaluru (12.97 N, 77.59 E) still returns a plausible lagna, so this fails quietly. It needs
  a golden-vector test against `swetest -house`.

- **Provenance does not apply.** `swe_houses_ex` returns `OK`/`ERR`, not a flag word, so
  `Source::from_returned_flags` has nothing to read. It also has nothing to degrade: the
  ascendant is built from sidereal time, obliquity, nutation and the ayanamsa, none of which
  come from `sepl_18.se1`. The chart's `Source` is therefore the weakest of the nine graha
  positions, and the ascendant contributes nothing to it. That is a statement about the
  mechanism, not a shortcut — it should be a comment on the method.

### 2.2.1 How the lagna is made correct by construction, not by care

An earlier draft called the argument order "a trap" and left it there. That is
not good enough: a transposed call returns a real ascendant for a real place, so
nothing at runtime looks wrong. Correctness here cannot rest on remembering the
order.

**The arguments are made untypeable.** The FFI call takes a named-field struct,
not two bare `f64`s in an order a reader has to recall:

```rust
struct HouseRequest { jd_ut: f64, latitude: f64, longitude: f64, system: HouseSystem }
```

One call site, destructured at the boundary. `as_se_geopos()` — which is
`[longitude, latitude, elevation]`, the opposite order — cannot be passed by
mistake, because the types do not line up.

**One test, not four.** Three candidate checks were considered and two dropped.
The reasoning matters more than the conclusion:

| Check | Catches a transposed latitude? | Kept |
|---|---|---|
| Sidereal minus tropical equals the ayanamsa | **No** | as one line inside the test below |
| An independent closed-form ascendant | **Yes** | **yes — this is the test** |
| Published lagna values | Yes | no |

**Why the ayanamsa identity is not enough on its own.** It is blind to the exact
bug it was proposed to guard. The ayanamsa does not depend on location at all,
so a transposed call transposes *both* the sidereal and the tropical reading
identically and their difference is still exactly the ayanamsa. The check passes
cleanly on the broken code. It is worth one assertion — it does catch the flag
being dropped, and `swehouse.c:230` silently substituting Fagan-Bradley if
`swed.ayana_is_set` were false — but it cannot be the guard.

**The independent implementation is the test.** The ascendant has a closed form:

> Asc = atan2( cos(RAMC), −( sin(RAMC)·cos(ε) + tan(φ)·sin(ε) ) )

with RAMC the right ascension of the midheaven, ε the obliquity and φ the
geographic latitude. Implemented in Rust from the engine's own sidereal time and
obliquity, then compared against `swe_houses_ex`.

It subsumes the ayanamsa check rather than sitting beside it: the closed form
produces a *tropical* ascendant, so the comparison is "closed form minus the
engine's ayanamsa equals `swe_houses_ex` with `SEFLG_SIDEREAL`". A dropped flag
shows up as a 24° disagreement in the same assertion.

And **φ appears in the formula**, so a swapped argument cannot agree. That is
the whole reason to prefer it: it is not a second reading of the same number, it
is a second derivation from the inputs.

**Published values are dropped.** They were argued for as the only check that
could catch the others being consistently wrong together. That residual is not
real: for the closed form and `swe_houses_ex` to agree while both are wrong, the
engine's own sidereal time or obliquity would have to be wrong — and those come
from Swiss Ephemeris, which is the reference the published values are themselves
computed from. A table of hand-copied instants would add maintenance and check
nothing the closed form does not.

### 2.2.2 The guard, concretely

**What it computes.** The same number, from the inputs rather than from the
library.

| Input | Where from |
|---|---|
| RAMC — right ascension of the midheaven | `swe_sidtime(jd_ut)` gives Greenwich apparent sidereal time in hours. LST = GAST + longitude ÷ 15. RAMC = LST × 15 |
| ε — true obliquity | `swe_calc_ut(jd, SE_ECL_NUT, …)`, first element |
| φ — geographic latitude | the observer |

> Asc = atan2( cos RAMC, −( sin RAMC · cos ε + tan φ · sin ε ) )

normalised to `[0, 360)`, with the quadrant resolved against the MC — the
ascendant lies in the semicircle east of it. That quadrant fix is the most
likely place to get this wrong, and getting it wrong shows up as a clean 180°
disagreement, which is about as loud as a test failure gets.

**What it asserts.**

```
closed_form_tropical(jd, lat, lon) − ayanamsa(jd)  ==  engine.ascendant(jd, observer)
```

**Coverage.** Latitudes 0, ±23.4, ±45, ±60 — every band except polar, which
§2.3 handles separately. Instants every two hours across a day so all twelve
rashis take a turn rising, and a handful of dates across a year so the obliquity
and the equation of time both move. A test that only ever sees one rising sign
proves one twelfth of the thing.

**The wrinkle, and why the tolerance is a finding rather than a knob.**

`Engine::ayanamsa` is deliberately *not* `swe_get_ayanamsa_ex_ut`. It is the
difference between the tropical and sidereal longitude of the Sun, because that
is the quantity actually applied to every position the app displays
(`engine.rs:365-384`). The two resolve the equinox differently and can differ by
the nutation in longitude, up to about 17 arcseconds.

`swe_houses_ex` with `SEFLG_SIDEREAL` does its own sidereal transformation
(`swehouse.c:269`). If it uses the library's ayanamsa rather than ours, the
assertion above will not close to arcsecond precision — it will close to about
17 arcseconds.

That is worth knowing rather than absorbing. **If the residual is ~17″, it means
the lagna sits on a very slightly different sidereal frame from every rashi the
app already prints**, and two figures on one screen would disagree about where a
sign boundary is — within 17″ of it, or about one crossing in six thousand. The
tolerance is therefore set at arcsecond precision first, and if it fails at ~17″
the fix is to put the lagna on the app's own frame, not to widen the tolerance
until it passes.

**One discipline note.** If the two disagree, the question is which is wrong.
The closed form is implemented from a cited source and then left alone; tuning
it until it agrees with `swe_houses_ex` would turn the guard into an echo of the
thing it guards, and this project has already recorded what that failure looks
like — `docs/AUDIT.md`: *a fix whose test would still pass against the old code
is not finished.*

**The test's location is chosen so a swap is loud.** Latitude and longitude far
apart, in a different hemisphere band: Bengaluru at 12.97N 77.59E transposes to
77.59N 12.97E, inside the Arctic circle, where the polar branch of §2.3 fires as
well. The assertion fails on the value; the polar path failing too is a second
signal, not the first.

### 2.3 Missing: the obliquity, for the polar test

§8.3 needs to know whether the observer is inside the polar circle, which is
`|latitude| ≥ 90° − ε`. Rather than writing 66.5 into the source:

- `swe_calc_ut(jd, SE_ECL_NUT, 0, xx, err)` with `SE_ECL_NUT = -1` returns
  `xx[0]` = true obliquity in degrees, `xx[1]` = mean obliquity, `xx[2]` = nutation in
  longitude, `xx[3]` = nutation in obliquity. Verified in the vendored `sweph.c` (the
  assignment block at the `ipl == SE_ECL_NUT` branch).
- The same file records that this call "is not dependent on ephemeris", so it too never
  degrades.

One extra call, and the polar circle stops being a constant somebody has to keep current.

### 2.4 Cost

R-04 measured `swe_calc_ut` at 7.9 µs. The whole chart is one `positions` call for nine bodies
(~71 µs), one `swe_houses_ex`, one `SE_ECL_NUT`. Under 200 µs.

**Not cached.** Every key would be a distinct instant, so the `Lru` would fill with entries
that can never be hit. It is cheaper than the cache lookup.

---

## 3. Chart style: the options, the recommendation, the reasoning

### 3.1 The three formats

| | North Indian | South Indian | East Indian |
|---|---|---|---|
| Shape | square, both diagonals plus the midpoint square — 4 diamonds and 8 triangles | 4×4 grid, centre 2×2 removed | 4×4 grid, centre 2×2 removed |
| Fixed | **houses** | **signs** | **signs** |
| Moves | signs | houses | houses |
| Origin | house 1 is the top-centre diamond | Meena is the top-left cell | Mesha is the top-centre cell |
| Direction | anticlockwise | clockwise | anticlockwise |
| Sign written as | a number, 1–12 | the name or abbreviation | the name or abbreviation |
| Lagna shown by | being house 1 | a diagonal stroke through its cell | a mark in its cell |
| Also called | diamond, lozenge | rasi chakra | Surya chakra, Bengali, Odia |
| Used in | the Hindi belt | the four southern states | West Bengal, Odisha, Assam |

North Indian compartment order, counted anticlockwise from the top-centre diamond:

| House | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Position | top ◆ | upper-left △ | mid-left △ | left ◆ | lower-left △ | bottom-left △ | bottom ◆ | bottom-right △ | mid-right △ | right ◆ | upper-right △ | top-right △ |

South Indian fixed sign layout:

| | col 1 | col 2 | col 3 | col 4 |
|---|---|---|---|---|
| **row 1** | Meena | Mesha | Vrishabha | Mithuna |
| **row 2** | Kumbha | — | — | Karka |
| **row 3** | Makara | — | — | Simha |
| **row 4** | Dhanu | Vrishchika | Tula | Kanya |

The four corners are the check: Meena upper-left, Mithuna upper-right, Kanya lower-right,
Dhanu lower-left — stated in exactly those terms by
[Dirah](https://www.dirah.nl/square.htm) ("The upper left hand square is always Pisces, the
upper right hand square is always Gemini. The lower right hand square is always Virgo and the
lower left hand square is Sagittarius"), which also gives the clockwise reading and the
diagonal-stroke lagna mark.

**On which is more common: no source found that quantifies it.** Every source consulted states
it regionally, not by share. The honest claim is the regional one in the table, and this
document does not make a stronger one. If a share figure is ever wanted for a decision, it
needs looking up — the project has been burned once by a percentage written from memory
(D-026).

Sources for §3.1:
[Dirah](https://www.dirah.nl/square.htm) ·
[Edith Hathaway, North Indian](https://edithhathaway.com/how-to-read-a-north-indian-chart/) ·
[AstrologyNow](https://www.innerknowing.yoga/northindianchart) ·
[KundliGPT, North Indian layout](https://kundligpt.com/blog/how-to-read-north-indian-birth-chart/) ·
[KundliGPT, three styles](https://kundligpt.com/blog/north-south-east-indian-chart-styles-compared/) ·
[VedicPlanet](https://www.vedicplanet.com/jyotish/learn-jyotish/chart-formats-in-jyotish-astrology/)

### 3.2 Decision: all three formats, chosen in settings

**The user has decided: all common formats are supported, and the default is North Indian.**
§3.2 and §3.3 below were written before that decision and argued for South Indian alone; they
are kept because the reasoning is still what the degraded cases look like — it is now an
argument about fallback rather than about the default.

What the decision changes, and what it does not:

- **The data model does not change at all.** Every format draws the same two facts: which rashi
  each graha occupies, and which rashi holds the lagna. A chart is `[Rashi; 12]` of occupants
  plus a lagna. The formats differ only in where on screen a rashi is drawn and what is written
  in the cell.
- **South Indian and East Indian share their geometry** — a 4×4 grid with the centre 2×2
  removed. They differ in origin cell and direction only, which is one 12-entry table each.
- **North Indian is different geometry**, and needs its own compartment table: 4 diamonds and 8
  triangles, with a text anchor per compartment. §3.3's objection that "text does not lay out in
  a triangle" stands as a rendering fact, and the answer is an SVG polygon plus an explicit
  anchor point per compartment — twelve coordinates derived from the geometry, in the manner of
  `crates/glyph/src/glyphs.rs`, not twelve hand-tuned rectangles.
- **North Indian cannot be drawn at all without a lagna.** In South and East Indian the lagna is
  one mark that can be absent; in North Indian it *is* the frame. So the lagna moves from
  "desirable" to "required", and §2.2 becomes a blocker for the feature rather than for a
  detail of it.

**The default is North Indian.** It is the format most likely to be recognised, and the user
has chosen it. The consequence is stated plainly: North Indian has no degraded form. Where
there is no lagna there is no house 1 and no chart, so §8.2's "no location" case cannot fall
back to drawing the grahas without a frame — it must either say why there is no chart, or offer
the South Indian view, which can be drawn without one.

### 3.2.1 Why South Indian was argued for, and what that argument is now good for

Four reasons, in order of weight. They no longer choose the default; they describe what is lost
when the lagna is unavailable, and they are the reason the fallback is worth building.

**1. It degrades to something, and the North Indian chart degrades to nothing.**
Chandra's location comes from a fallback chain (D-007) whose last step is a timezone centroid
that can be hundreds of kilometres out. In a South Indian chart the lagna is one diagonal
stroke in one cell — remove it and the chart is still complete and still correct, because every
graha is still in its rashi. In a North Indian chart the lagna **is** the frame: with no lagna
there is no house 1, and there is no chart to draw. For a menu bar app that must work offline
with a guessed location, that settles it.

**2. Uniform cells.** The North Indian chart's smallest compartment — a corner triangle — is
roughly a third the area of a central diamond, so type must be sized for the triangle and the
diamond wastes the difference. A grid of identical rectangles has one cell size, decided once,
and one worst case to prove.

**3. Text does not lay out in a triangle.** WebKit does not implement CSS `shape-inside`, so
running text inside the eight triangles means a hand-tuned text box per compartment — twelve
magic rectangles, which is exactly the sort of table this project refuses to write.

**4. Chandra is already a sign-first app.** Its grid labels ingresses by rashi, its day view has
a `Rashi` field, its tray tooltip reads `Mangala - Vrishchika`. A chart whose cells are fixed
rashis speaks the vocabulary the app already speaks. A chart whose cells are fixed houses
introduces bhavas, which no surface of the app has ever shown.

### 3.3 What is being rejected

- **North Indian.** It is the format a user from the Hindi belt is most likely to recognise,
  and losing that is a real cost, not a free win. It loses on §3.2's first reason.
- **East Indian.** Same geometry as South Indian, different origin and direction. Smaller user
  base, and a second layout is a second set of tests for one chart. Not worth it unless asked.
- ~~**Offering all three behind a setting.** Three layouts, three sets of geometry constants,
  three spoken-label orders, for a feature that has not shipped once.~~ **Overruled by the user:
  all common formats are supported.** The cost is smaller than this paragraph assumed — the
  three share one data model, and two of the three share one geometry. What it does buy is a
  hard dependency on the lagna, which South Indian alone could have shipped without.

### 3.4 Houses: counted, not printed

- The bhava scheme is **whole sign** — the rashi containing the lagna is the first house, and
  the eleven that follow it are houses 2–12. This is the scheme `standing.rs` already uses for
  drishti, and using a second scheme in the same app would make the chart and the day view
  disagree about what "the seventh" means.
- Whole sign also has no polar failure mode (§8.3), unlike Placidus and Koch.
- **House numbers are not drawn.** There is no room, and the diagonal already anchors the
  count. They are spoken (§7.2), where there is room.
- No bhava chalit chart, no cusp-based houses, no other house system. Offering one would be
  asking the user a question the app cannot help them answer.

### 3.5 Body abbreviations

Two letters, Latin: **Su Mo Ma Me Ju Ve Sa Ra Ke**. This is the published chart convention
([KundliGPT](https://kundligpt.com/blog/how-to-read-north-indian-birth-chart/)).

Sanskrit two-letter forms are not available: *Shukra* and *Shani* are both `Sh`, and *Surya*,
*Shukra* and *Shani* all start `S`. Three-letter Sanskrit (`Sur Cha Man Bud Gur Shu Sha Rah
Ket`) is 50% wider in a cell that has no width to give.

**The codebase has already made this exact trade.** `zodiac.rs::short()` returns Western
abbreviations (`Ari`, `Tau`) with the comment: "the Sanskrit names cannot be abbreviated to
three letters and stay distinct: `Vrishabha` and `Vrishchika` are identical for six." The rule
is: abbreviate in the Western form, name in the Sanskrit form. The grahas follow it.

`Ma`/`Me` is the one pair at risk. Mitigation is §11.3's standard one: the spoken label never
carries an abbreviation, and says `Mangala` and `Budha` in full.

---

## 4. Layout at 320 × 332

### 4.1 Budget

Panel geometry is fixed (DESIGN §2.2): 12 pad + 40 header + 4 + **264 region** + 12 pad.
The chart is a view inside that 264px region. The panel does not resize.

| | |
|---|---|
| Content column | 288px, x = 16 → 304 |
| Region | 264px tall |
| Chart | **288 × 240**, region-local y = 4 → 244 |
| Cell | **72 × 60** |
| Centre void | **144 × 120**, columns 2–3 × rows 2–3 |
| Foot | 20px, region-local y = 244 → 264 |

The 20px foot is where `.region::after`'s existing fade gradient sits. Leaving the chart clear
of it means nothing has to be changed about the fade, and nothing that must be read is under
it.

**The cells are not square (72 × 60).** Printed charts usually are, but nothing in the format
requires it — the format is the fixed sign positions and the clockwise order. A square chart
would be limited to 240 × 240, which costs 12px of width per cell, and width is the dimension
the cell is short of.

### 4.2 The chart, at proportion

`4px = 1 character`, `12px = 1 line`.

```
 x:16          88          160         232         304
    ┌───────────┬───────────┬───────────┬───────────┐   y:4
    │ PIS       │ ARI       │ TAU       │ GEM       │
    │           │           │           │           │
    │ Sa℞       │           │ Ra        │ Ju        │   60
    │           │           │           │           │
    ├───────────┼───────────┴───────────┼───────────┤   y:64
    │ AQU       │                       │ CAN       │
    │           │        20:14          │           │
    │           │                       │ Mo        │   60
    │           │    Vrishchika lagna   │           │
    ├───────────┤    Bengaluru          ├───────────┤   y:124
    │ CAP       │                       │ LEO       │
    │           │                       │           │
    │           │                       │ Su Me Ve  │   60
    │           │                       │           │
    ├───────────┼───────────┬───────────┼───────────┤   y:184
    │ SAG       │ SCO     ╲ │ LIB       │ VIR       │
    │           │       ╲   │           │           │
    │           │ Ke  ╲     │           │ Ma        │   60
    │           │   ╲       │           │           │
    └───────────┴───────────┴───────────┴───────────┘   y:244
     └── 72 ────┘
```

Header above it, per DESIGN §5.2's three slots:

```
 x:16      40    48                                    272   280   304
  ┌──────────┬───┬────────────────────────────────────────┬───┬───────┐
  │    ‹     │ 8 │  "Gochara"                             │ 8 │   ⚙   │
  └──────────┴───┴────────────────────────────────────────┴───┴───────┘
```

The `‹` is automatic: `Header.tsx` already renders the back chevron whenever the view is not
the calendar.

### 4.3 Cell anatomy (72 × 60)

```
    x:0                                            72
 y:0 ┌──────────────────────────────────────────────┐
     │                                              │   4  pad
 y:4 │  ┌──────┐                                    │
     │  │ SCO  │  rashi, micro 10/600/+0.6          │  12  --text-secondary
y:16 │  └──────┘                                    │
     │                                              │   2
y:18 │  ┌──────────┬──────────┬──────────┐          │
     │  │  Su      │  Me      │  Ve      │          │  12  micro, --text-primary
y:30 │  ├──────────┼──────────┼──────────┤          │
     │  │  Ma      │          │          │          │  12
y:42 │  ├──────────┼──────────┼──────────┤          │
     │  │          │          │          │          │  12
y:54 │  └──────────┴──────────┴──────────┘          │
     │                                              │   6  pad
y:60 └──────────────────────────────────────────────┘
      └─ 4 ─┘└── 21.3 ──┘        └── 64 usable ──┘
```

| | |
|---|---|
| Inset | 4px each side → 64 × 52 usable |
| Rashi label | `--type-micro`, uppercased, `--text-secondary`, top-left |
| Body slots | 3 columns × 3 rows = **9** |
| Slot | 21.3 × 12 |
| Body | `--type-micro`, `--text-primary` |
| Retrograde | `℞` after the abbreviation, superscript |

**Nine slots is the proof the layout cannot overflow.** The true maximum is eight — Rahu and
Ketu are 180° apart by construction and can never share a rashi — and eight requires seven
bodies plus a node inside one 30° arc, which does not occur in a human lifetime. Sizing for
nine means the worst case is bounded by geometry rather than by hoping.

Five bodies in one rashi is ordinary: Budha stays within 28° of Surya and Shukra within 47°, so
Surya + Budha + Shukra together is common, and the Moon joins them for about two and a half
days a month.

`Ra℞` at micro with `+0.6` tracking is about 17px against a 21.3px slot. **The exact superscript
size is to be measured against real layout**, not computed — DESIGN §5.2 records that measuring
type from a canvas silently fails for `-apple-system`, and the truncation ladder there is
already measured from `scrollWidth` for that reason. The same method applies.

### 4.4 The centre void

144 × 120, 8px inset → 128 × 104. Printed South Indian charts put the identifying block here;
so does this one.

| Line | Type | Content |
|---|---|---|
| 1 | `--type-title` | `20:14` — the instant the chart was computed |
| 2 | `--type-caption`, `--text-secondary` | `Vrishchika lagna` |
| 3 | `--type-caption`, `--text-secondary` | the resolved place's label |
| 4 | `--type-caption`, `--text-secondary` | §8's degraded sentence, when one applies |

20 + 14 + 14 = 48px used of 104. The remaining 56px is four caption lines at 128px wide, which
is what the Moshier sentence needs.

**Line 3 is not decoration.** The lagna is the first value in this app whose correctness depends
materially on which step of the D-007 chain resolved the location: it moves 1° per 4 minutes of
clock time, so a longitude that is 300 km out moves it by about 3°, and a timezone centroid can
easily be that far from the user. Naming the place is what lets the reader judge the number.

### 4.5 Colour

| Element | Token |
|---|---|
| Cell borders | `--border` |
| Rashi labels, centre block lines 2–4 | `--text-secondary` |
| Body abbreviations, `℞`, centre block line 1 | `--text-primary` |
| Lagna diagonal | `--marker` |

That is the whole palette. Two notes:

- **`--accent` appears nowhere.** It means today and nothing else (D-023). A chart of *now* is
  entirely today, so it has nothing to distinguish; using gold for the lagna would be a hue
  encoding a meaning, which is what D-023 forbids.
- **`--glare` appears nowhere.** §10.

`--marker` is `#c8c8ce`, lighter than `--text-secondary`, so it clears what `--text-secondary`
clears and satisfies §11.1's non-text bar.

---

## 5. Where it lives

### 5.1 A panel view, not a window

ARCHITECTURE §8 states it plainly: "The panel is the only surface the app has, and it never
resizes." DESIGN §12 refused a settings window for the same reason. A chart window would be a
fourth surface with its own material, its own scale handling and its own auto-hide.

So: a fourth member of `Panel.tsx`'s `type View = "calendar" | "day" | "settings"` →
`| "kundali"`, rendered in the same 264px `.region`.

### 5.2 The panel's subject has to widen

`panel::toggle(app, subject: Graha, rect)` assumes every tray item is a graha. The kundali item
is not one.

- Widen to `enum TraySubject { Graha(Graha), Kundali }`, threaded through `current_subject`,
  `set_current_subject`, `announce_subject` and `panel::toggle`.
- The `chandra://open` payload is already a bare string (`subject.key()`). Adding `"kundali"`
  widens that string's domain by one value, and it cannot collide: the nine graha keys are
  `surya` … `ketu`.
- `Panel.tsx::open()` maps `"kundali"` to `setView("kundali")` and leaves `subject` where it
  was, so closing the chart with `‹` returns to whichever graha calendar was last open.

### 5.3 The header title

`Gochara`, not `Kundali`. The word states that this is the transiting sky and not a natal
chart (§1.1), which is the one thing about this feature a user could misread.

The truncation ladder (DESIGN §5.2) has nothing to shorten — one word always fits — so this
view has no ladder.

### 5.4 The menu bar item

A tenth status item, off by default.

- **Built last.** `tray::build` creates the moon first so it lands rightmost, then the grahas in
  reverse. Creating the kundali item last puts it at the **left-hand end**, outside the graha
  run — which keeps the graha run contiguous and therefore learnable, which is why `build`
  reverses in the first place.
- **Icon: two concentric squares**, the South Indian chart's silhouette. Outer 18 units, inner
  9 units, on the 24-unit design grid, stroked at 1.8 units — the same grid, stroke, cap and
  join every glyph in DESIGN §7.2 holds constant. A template image, per D-008.
- **No internal cell divisions.** A 4×4 frame at 22pt gives 4.5-unit cells; a 1.8-unit stroke
  is 40% of that, and the result is a moiré rather than a chart. Reducing the stroke for one
  glyph would break the nine-glyph design set.
- **The icon is static.** It carries no live state. This follows D-022's reasoning rather than
  contradicting it: a template image varies only in alpha, the frame has no cell to put a lagna
  diagonal in, and a diagonal across a 4.5-unit corner reads as a solid blob. What the menu bar
  can carry, it carries; the rest goes in the tooltip.
- **Distinguishability.** None of the nine graha glyphs contains a square — they are circles,
  crescents, arrows and hooks — so the pair-at-risk table in DESIGN §7.2 gains no row. This
  still needs the 44 × 44 rasterisation check that section records for the other nine, before
  the path data is fixed.
- **Tooltip:** `Gochara - Vrishchika lagna`, rebuilt on `TrayIconEvent::Enter` (D-028). That
  makes it current to the second, which is what a two-hourly value needs.

### 5.5 Refresh cadence

| Surface | When | Why |
|---|---|---|
| Tray icon | never redrawn | it carries no state; the hourly tick (D-028) has nothing to do for it |
| Tray tooltip | on `TrayIconEvent::Enter` | D-028's second half: the lagna turns over every ~2 h, and the pointer arriving is a second's notice |
| The chart | once, when the panel opens | — |

**The chart is not refreshed while open.** The lagna moves 1° per 4 minutes; the panel
auto-hides on blur, so it is open for seconds. Nothing visible changes unless the lagna crosses
a rashi boundary, which happens on about one panel-open in a hundred. A timer would be
background work (D-017) for a change that is almost always invisible, and the instant printed in
the centre block is what makes the drawn chart checkable against the clock either way.

---

## 6. Settings and the schema migration

### 6.1 The setting

One boolean, in the block that already owns the menu bar:

```rust
pub struct TraySetting {
    pub subjects: Vec<Graha>,
    pub colour_mode: bool,
    /// Whether the gochara chart has its own menu bar item.
    pub kundali: bool,
}
```

- Default `false`. D-009's principle — "the menu bar stays clean until the user opts in."
- It is a tray setting rather than a section of its own because the item **is** the only way to
  reach the chart. Turning the item off removes the feature; there is no second entry point to
  leave orphaned.

### 6.2 The migration

`SCHEMA_VERSION` 6 → 7.

```rust
// 6 -> 7. The menu bar gained a gochara chart item. A file written before it
// means the state it was in, which is off: there was no item to enable.
if version == 6 {
    value
        .get_mut("tray")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| AppError::Settings("settings schema 6 has no tray block".into()))?
        .insert("kundali".into(), serde_json::Value::Bool(false));
    value["schema_version"] = serde_json::Value::from(7u32);
    version = 7;
}
```

Written the same way as the 3 → 4 and 4 → 5 steps: the key is **inserted**, not left to a serde
default, so a migrated document is complete on disk and the next migration has a file to be
written against. That is the reasoning the 3 → 4 comment already records.

Tests to add, matching the existing pattern:

- `a_version_six_document_migrates_without_changing_what_it_meant` — asserts `kundali == false`
  and that the rest of the document survives.
- The existing `a_version_one_document_migrates_without_changing_what_it_meant` runs the whole
  ladder and needs the same assertion appended.

### 6.3 The settings UI

Settings → Menu bar, below the nine graha toggles and above `Coloured icons`.

- A `Toggle` row: label `Kundali`, `settings__hint` reading `A transit chart for now`.
- The root row's value already counts items — `Moon only` / `Moon + N`. The kundali counts as
  one of the N, so no wording changes. The row still answers what it is for: how many things
  are in the menu bar.

### 6.4 Ayanamsa and node type

Both apply, and neither is a choice.

- The ascendant is computed with `SEFLG_SIDEREAL` under the engine's current `sid_mode`, which
  is set from `sidereal.ayanamsa`.
- The nine positions come from `Engine::positions`, which already honours `node_type` (D-027:
  the mean node by default).
- A chart drawn under a different ayanamsa from the calendar behind it would disagree with
  itself, which is the failure the generation counter in `almanac.rs` exists to prevent.

Changing either invalidates nothing, because the chart is not cached (§2.4). The next open
recomputes.

---

## 7. Accessibility — the spoken form of the chart

### 7.1 Structure: a list, not a grid

A 4×4 `role="grid"` would announce row-major — Meena, Mesha, Vrishabha, Mithuna, then Kumbha,
Karka — which is not the chart's clockwise order and is not any order a reader wants.

So: `role="list"` with twelve `listitem`s, in **rashi order** (Mesha → Meena). That is the
order the data already has, the order every other surface in the app uses, and an ordered
reading of the zodiac rather than a shape.

CSS Grid `grid-area` places each item in its fixed cell, decoupling DOM order from visual
order. That is normally a focus-order anti-pattern; it does not apply here, because **no cell is
focusable** — the chart has no selection, no hover state and no click-through (§10).

### 7.2 Labels

| Element | Role | Label |
|---|---|---|
| The view | `region` | `Gochara chart` |
| A cell | `listitem` | `Vrishchika, first house, lagna, Ketu` |
| A cell with a retrograde body | `listitem` | `Meena, sixth house, Shani retrograde` |
| An empty cell | `listitem` | `Kumbha, ninth house, empty` |
| The centre block | `group` | `Computed 20:14 at Bengaluru` |

Following DESIGN §11.4's established rules:

- **No abbreviation is ever spoken.** `Shani`, not `Sa`. `Vrishchika`, not `Sco`. The
  abbreviation is a printed notation, not a word — the same argument that makes the tithi cell
  say `Shukla Ashtami` rather than `S8`.
- **An empty cell says `empty`.** A listitem with no content is skipped, and the listener loses
  the count — which is how you tell the ninth house from the tenth by ear.
- **The house ordinal precedes the bodies**, because it is a property of the cell and the bodies
  are its contents.
- **`lagna` and `first house` are said together** in the one cell where both are true. They are
  the same fact, but `lagna` is what the diagonal draws and §11.3 requires the drawn thing to
  have the word.

### 7.3 Additions to DESIGN §11.3

| Meaning | Non-textual carrier | Textual carrier — always present |
|---|---|---|
| Lagna | diagonal stroke in the rising sign's cell | the cell's label says `lagna`; the centre block reads `Vrishchika lagna` |
| A graha's rashi | which cell its abbreviation is in | the cell's label names the sign and the graha in full |
| Retrograde | `℞` after the abbreviation | the cell's label says `retrograde` |
| Which sign a cell is | the cell's fixed position | the printed three-letter label, and the spoken full name |

The last row is why the cells are labelled at all. A printed South Indian chart often omits the
sign names, because a reader who works with them has learnt the fixed layout. A first-time
reader has not, and position alone is a shape carrying a meaning — which §11.3 does not allow.

---

## 8. Degraded states

None of these is an error. §9.4's rule holds: each is stated as a fact, in words, in place, with
no warning colour, icon or border.

### 8.1 Outside the ephemeris range

- Reachable only by a system clock set before 1800 or after 2399. Unlikely; not impossible; and
  D-006 forbids presenting degraded data as authoritative regardless of how it was reached.
- Line 4 of the centre block: `Moshier ephemeris — reduced precision outside 1800–2399`, the
  same sentence the day view already prints.
- The ascendant itself does not degrade (§2.2), so the sentence is about the nine positions.
  It is true of the chart because `Source::weakest` makes a composite no stronger than its worst
  input.

### 8.2 No location

**Cannot happen.** D-007 step 3 — the bundled `zone.tab` centroid — is offline,
deterministic and always resolves, which is why DESIGN §9.5 can say the first panel open never
shows `LOCATION_UNRESOLVED`.

What can happen is a location that is *resolved but coarse*, and for the lagna that matters more
than for anything else the app draws (§4.4). The response is not a warning; it is line 3 of the
centre block naming the place, so the reader can see what the number was computed for.

### 8.3 Polar latitudes

Inside the polar circle the ecliptic can lie wholly above or below the horizon, and the
ascendant is not continuous.

What Swiss Ephemeris does, verified in the vendored `swehouse.c`:

- **Koch and Placidus fail.** `if (fabs(fi) >= 90 - ekl) { retc = ERR; strcpy(serr, "within
  polar circle, switched to Porphyry"); goto porphyry; }`
- **Whole sign ('W') does not fail.** Its branch is
  `acmc = swe_difdeg2n(ac, mc); if (acmc < 0) { ac = degnorm(ac + 180); } cusp[1] = ac -
  fmod(ac, 30);` — it applies a 180° correction and returns `OK`. That is one more reason
  whole sign is the right system here (§3.4), beyond its being the classical one.

Consequence: at high latitude the lagna can jump by six rashis between two nearby instants.
That is a property of the sky, not a fault, and it is silent — the correction happens inside the
C and the returned `ascmc` gives no sign that it fired.

- **Detect it from the latitude**, which is the only thing the condition depends on:
  `|latitude| ≥ 90° − ε`, with ε from `SE_ECL_NUT` (§2.3) rather than written into the source.
- **Say it in place.** Line 4 of the centre block: `inside the polar circle — the lagna is not
  continuous here`. This shares line 4 with §8.1's sentence, and the two cannot both apply
  in practice; if they ever do, the polar sentence wins, because it is about the number on
  screen and the other is about its precision.
- **The chart is still drawn.** Every graha is still in its rashi and every one of those
  positions is correct. Only the lagna is unstable, and only the diagonal depends on it.

### 8.4 An engine failure

`AppError` with a stable code, rendered as the specific inline state ARCHITECTURE §6 requires.
No new error path — this view uses the one every other view uses.

---

## 9. Open questions

1. **South Indian or North Indian?** §3.2 recommends South Indian on the grounds that it
   survives a coarse location and the North Indian chart does not. If you would rather have the
   diamond, that is the chart to build — not both.
2. **Does the chart get its own status item?** The request says yes. It costs a permanent slot
   in a menu bar that may already be full, and it is the only entry point to the feature. Is a
   tenth item what you want, or should the chart be reachable from the moon panel instead?
3. **Two-letter Latin abbreviations (`Su Mo Ma Me Ju Ve Sa Ra Ke`) or three-letter Sanskrit
   (`Sur Cha Man Bud Gur Shu Sha Rah Ket`)?** §3.5 recommends Latin, following the rule
   `zodiac.rs` already applies to rashi names. Sanskrit costs 50% more width in the cell that
   has the least of it.
4. **Should the chart print degrees within the sign?** There is no room in a 72 × 60 cell, and
   a degree in a chart of *now* changes while it is being read. Recommendation: no.
5. **Is `Gochara` the right title, or would you rather it said `Kundali`?** §5.3 argues for
   `Gochara` because it is the word that says this is not a natal chart. `Kundali` is what you
   asked for and is the more familiar word.

---

## 10. What is deliberately not being built, and why

| Not built | Why |
|---|---|
| A birth chart, or stored charts | Needs birth data, a form and a document store. Chandra has one window and no forms. |
| Navamsa or any other varga | A varga is a second chart, and the region holds one. |
| Any interpretation — yogas, dashas, strength | `standing.rs` already records the app's position: it reports what a graha is doing, not what it means. |
| A house system other than whole sign | Whole sign is what `standing.rs` counts for drishti. A second scheme would make the chart and the day view disagree about "the seventh". |
| Printed house numbers | No room, and the diagonal anchors the count. Spoken instead (§7.2). |
| A combustion wash in the chart | A cell holds up to nine bodies, so a cell-level wash cannot say which one is combust — and the Sun's own abbreviation is in that same cell, so it would read as marking the Sun. At micro size a wash behind two characters is a smudge. Combustion is named in words in the day view, which is where it stays. |
| `--accent` anywhere in the chart | It means today, and the whole chart is today (§4.5). |
| Per-cell interaction — hover, selection, click-through to a day | The chart is of an instant; the day view is of a day. There is nothing for a cell to open. |
| A refresh timer while the panel is open | D-017. §5.5. |
| A live lagna mark on the tray icon | The icon has no cell to put it in (§5.4). The tooltip carries it, current to the second. |
| A second window | ARCHITECTURE §8. |
| A new colour token | The four in §4.5 are enough. |

---

## 11. Order of work

1. `Engine::ascendant`, with a golden vector cross-checked against `swetest -house` — including
   a southern-hemisphere and an eastern/western-longitude case, because the argument order is
   the trap (§2.2).
2. The obliquity read, and the polar-circle predicate.
3. `Almanac::gochara(unix_ms) -> Gochara`, reusing `SnapshotGraha` for the nine bodies.
4. The `gochara` command, its TS binding, and the contract fixture.
5. `TraySubject`, and `panel::toggle` widened.
6. The kundali tray item and its glyph path, with the 44 × 44 rasterisation check.
7. The `"kundali"` view: the twelve-cell layout, the centre block, the lagna diagonal.
8. The setting, the 6 → 7 migration, and both migration tests.
9. The spoken labels, verified against the live accessibility tree — DESIGN §11.4 records that
   WebKit pruned the whole calendar subtree once, and that it was only found by looking.

---

## 12. A defect found while writing this

`src-tauri/src/state.rs:76` and `src-tauri/src/tray.rs:22` both cite **D-019** for "the
permanent moon item is Chandra's item, so listing it here would put two moons in the menu bar".

That is **D-009**. D-019 is "every state the grid draws is named in the day view". Two
comments, one wrong reference each. Both are fixed.

The sentence they cited has since gone too. **D-030** moved the permanent slot from the moon to
the chart, so there is no permanent moon item and every calendar is toggleable; the doc comment
that carried the wrong citation was rewritten rather than corrected.
