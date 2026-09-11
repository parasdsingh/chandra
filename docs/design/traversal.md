# The traversal

Degree-based placement and a pathway around the ring, for the North Indian
Lagna Kundali. Specification for **E6.5**.

Status: the **degree grid ships** as a switch in Advanced, off by default
(schema 13). The **pathway placement is a prototype**, behind `pathway` on
`Chakra`, drawn only in the visual harness at `?preview`, and does not ship
until it has been looked at.

Read `docs/design/animation.md` first. This replaces its §7, which deferred
degree placement on the grounds that it did not compose with `cluster`. It does
not compose with `cluster`; `cluster` is what changes.

---

## 1. Five mistakes worth naming

Each build of this passed every check it was given and was wrong in ways the
checks could not see. Each was a class of error rather than a bug, so each is
written down here before the design that replaces it.

### 1.1 A constant taken from an identity instead of from the layout

A body's true position on the ring spans **two** house lengths while whole-sign
houses pin it to one compartment (§2). I resolved that by drawing it at half
scale, and justified the half by an identity: in the sky the degree span and one
crossing of the rising sign are both exactly one house long, so a half each
makes the chart a uniform half-scale of the true ring.

The identity is true and the constant was still wrong. **Elegance is not a
derivation.** The two halves are not interchangeable: the degree scale is the
reading, and the crossing is a motion of about a pixel every ten seconds. Half
the compartment was being spent on the second to make the arithmetic pretty,
and the first is the resource that crowding actually consumes — so the halving
was the direct cause of bodies being drawn on top of each other while up to
twenty degrees apart.

It is now a measured split (§4), and the measurement says so plainly.

### 1.2 A layout decision made a function of an instant

`threaded()` returned a mode — which type size, which arrangement, whether the
degrees had been given up — and it was called from a memo that depended on the
lagna's progress. So the mode was re-decided on every frame.

**A layout mode is a property of a compartment's contents, not of an instant.**
Re-deciding it per frame meant a compartment could be laid out one way at 41%
and another way at 42%, and the two arrangements have no relation to each other:
the group teleported. Worse, the fallback came from `squeezed`, which has no
progress term at all, so a compartment that flipped into it froze — one body
stood still for 51 of 100 steps and then jumped 58 units.

The check in place at the time sampled instants and tested four invariants at
each. Every instant was legal. **A property that holds at every sample and fails
between samples is invisible to a per-sample check**, and continuity, order and
monotonicity are all of that kind. §8 measures them.

The fix is structural, not a patch: `planFor()` decides the arrangement from the
bodies alone and is verified against the whole crossing before it is accepted;
`place()` reads a position off that plan. `planFor` cannot see the progress. It
is not a discipline, it is an argument it does not take.

### 1.3 An axis that rotated

Bodies that shared a degree were stacked perpendicular to the route **at the
point they stood on**. The route is a curve, so that perpendicular turns along
it — and two bodies on different offsets could therefore exchange positions on
screen while their order along the route never changed at all. Over one crossing
Guru and Budha, 9.2° apart, ended with their separation vector rotated a half
turn: same distance, opposite order.

**An offset direction that varies along a curve is not a packing axis; it is a
second motion.** The lanes are now a family of *parallel* lines — the route
translated along one fixed direction, perpendicular to the gate-to-gate chord —
and candidate routes that double back along that chord are rejected outright. A
parallel offset cannot change the chord projection, and the chord projection is
monotone in arclength by construction, so **order along the route is order on
screen**, and it is a property of the construction rather than of a test.

---

### 1.4 An obstacle placed in the only part of its own room that was usable

Eight of the twelve compartments put their caption at a far vertex. The four
wall triangles put theirs at the **middle of the wall** — which is the widest
part of that shape and the only part of it a body can stand in. So the one fixed
obstacle in the compartment sat exactly where its degree scale wanted to be.

That was not read as a defect for a long time because it was read as a
convention. It is a convention about *which side* the caption sits on, and it
had been extended, silently, into a claim about where along that side. The
principle the other eight compartments actually follow is: **a caption goes
where a graha cannot.** The wall triangles were the exception, and they were the
exception by accident.

Moving it to the narrow end of the same wall roughly doubles what the
compartment can carry:

| | Before | After |
|---|---|---|
| Band, the four wall triangles | 12, 19, 24, 25 | 25, 38, 40, 45 |
| Parallel lines available | 1–2 | 2–3 |
| Compartments that abandon their degrees | 2 | 1 |
| Slowest body, units per crossing | 2.5 | 5.8 |

It also flushed out two other things, and they are §1.5 and this one.
`withoutCaption` hands the caption's own room back when what is left beside it
could not hold a label anyway — right for the last resort, where a body outside
its sign would be worse, and wrong for the row packer, which can simply decline
the row and try another size. With the caption at the narrow end that fallback
started firing for *names*, and two grahas were drawn on top of one. The row
packer no longer yields; only `squeezed` does. **That fixed a fault the shipped
chart already had**: its overlapping label pairs fall from 61 to 51.

### 1.5 A tolerance applied to something that never earned it

`overlaps` was deliberately forgiving — a hair of horizontal overlap, up to two
units of vertical — and the forgiveness was argued for exactly one case. Label
against label: `GRAHA_ROW` is a hair tighter than the box it carries, so
adjacent rows overlap by about six tenths of a unit and nothing shows, because
the ink of a two-letter capital reaches nowhere near the ascender.

**The caption was getting that tolerance too, because the same function tested
both.** Nobody argued for it; it came along for the ride. It had no visible
consequence until §1.4 moved the wall triangle's caption into the drift's path,
at which point the slide would run until it had four tenths of a unit of the
caption underneath it, call that clear, and stop there. Three grazes in the
shipped chart, four in the pathway. A graze rather than a collision, and an
invariant that had been zero.

An obstacle is cleared or it is not. `touches` is the strict test and it is what
the caption gets everywhere: the drift, the pathway's placement, and the packed
rung's choice between candidates. `overlaps` had no callers left afterwards and
is gone; every label-against-label test in the pathway was already strict.

**Measure with the tolerance and you cannot see this.** The check that reported
zero was the same check that let the layout do it.

## 2. The tension, and which way out

The app uses whole-sign houses. House 1 *is* the rashi the lagna occupies
(`kundali.md` §3.4, and `standing.rs` counts drishti the same way). So a graha's
degree within its sign does not change on any timescale a panel is open for, and
a position that is a pure function of degree never moves.

| | What it is | Verdict |
|---|---|---|
| **Slide the ring** | Keep whole-sign houses. The lagna's progress through its own sign moves every compartment's contents from the edge they arrived through to the edge they will leave by | **Taken** |
| **Move the cusps** | Equal houses from the lagna's degree, so every graha's house position genuinely changes minute by minute | **Refused** |

Moving the cusps is refused on the record, not on taste: `kundali.md` §3.4
commits to whole sign and §10's *Not built* table names "a house system other
than whole sign" explicitly. It changes what the chart means and needs its own
decision record.

A body's true ring position, in house lengths forward from the ascendant, is

```
    R = (k - 1) + d - p
```

with `k` its whole-sign house, `d` its progress through its part-run and `p` the
lagna's. Both run 0 to 1, so `R` covers two house lengths inside one
compartment. The station drawn is

```
    f = (1 - d) * SHARE + p * (1 - SHARE)          measured from the entry
```

which puts `d = 1, p = 0` at the entry and `d = 0, p = 1` at the exit for any
split. `SHARE` is §4.

### The handover cannot be continuous

For a seamless handover a body at `f` in house *k* as `p → 1` must be the same
point as its position in house *k−1* as `p → 0`. The bands meet at the shared
wall, so the condition is `f(d, 1) = f(d, 0) + 1` for every `d`, which forces the
crossing to cover the whole band **and** the degree term to be zero.

> Under whole-sign houses, a degree scale along the path and a jump-free
> handover are mutually exclusive.

The degree scale is what was asked for, so it wins. The handover keeps its
clipped 260 ms slide, and the jump is bounded by the degree spread.

### The back-end change

`ChakraGraha` gained `progress: f64`, from `part_progress(varga, longitude)` —
the call `Lagna::progress` already uses, read on the body. `degrees_in_rashi`
cannot serve: it is deliberately the D1 degree in every chart, so in D9 it names
a position in a sign the chart is not drawing.

---

## 3. The model: a family of parallel degree lines

The single route is gone. A compartment is laid out on **a family of parallel
lines**, and this is both the layout and what the grid draws.

| | |
|---|---|
| **Along a line** | the degree. Thirty degrees of the sign, uniform in arclength |
| **Which line** | the packing. Nothing astronomical, which is why a conjunction goes there |

- The **base line** is a quadratic Bézier from the middle of the edge the sign
  arrives through to the middle of the edge it leaves by, bowed through the
  compartment's cluster point. Both gates share a vertex — a kite's outer point,
  either triangle's quarter point — so the straight chord between them runs
  along the tightest part of the shape.
- Every other line is that curve **translated** along one fixed direction: the
  normal to the gate-to-gate chord. Not the local normal (§1.3).
- The **band** is the stretch of the base line on which a label stands. It is
  the ruler the thirty degrees are laid across.
- A line's **usable run** is where a label stands on *it*. Lines are shorter
  than the band near the walls, and some do not exist at all. That is the
  compartment's capacity, and with the grid on it is visible.

**Two labels sharing a degree occupy the same station on adjacent lines.** That
is what a conjunction looks like, and it is the answer to bodies being drawn on
top of one another: the packing axis is a real axis with real capacity, rather
than a licence to shove things along the degree axis.

### Capacity, per shape

At full size, from the rendered chart. *Lines* is how many the family has
anywhere; *full lines* is how many span at least half the band.

| Shape | Houses | Band | Lines | Full lines |
|---|---|---|---|---|
| Kite | 1, 4, 7, 10 | 111–134 | 6 | 3–4 |
| Corner triangle | 2, 6, 8, 12 | 51–59 | 2–3 | 2 |
| Wall triangle | 3, 5, 9, 11 | 25–45 | 2–3 | 1–2 |

One degree is therefore **3.0–3.6 units in a kite**, 1.4–1.6 in a corner
triangle and 0.7–1.2 in a wall triangle.

**Can eight bodies in a kite hold their degrees?** Yes. February 1962 — eight in
the bottom kite, five of them inside two and a half degrees — lays out at 62% of
11px with every degree held to within the stated three-degree spill, and does
not reach either of the two fallbacks. Before the parallel family it went to the
packed rows and lost its degrees entirely.

### The ruler is measured with the narrowest label

The band sets the scale; the lane pitch sets the stack spacing. They were both
measured with the widest label in the compartment, and that was wrong: `(Ke)` is
25 units against a plain `Mo`'s 16.4, so **one retrograde bracket halved a
compartment's entire degree scale** for the benefit of one body in it. A routine
D1 chart's wall triangle fell into the packed fallback on the strength of it.

The band is now measured with the *narrowest* label present — it is a ruler, not
a slot — and each body's containment is checked with its own width, so a wide
label takes a line where it fits instead of shortening the scale for everything
else. The pitch still clears the widest, or a stack would not be evenly spaced.

---

## 4. The split between the degree scale and the travel

`PATH_DEGREE_SHARE`, derived by measurement rather than from the identity that
produced the original half. Re-measured after §1.4, because the wall triangles'
bands roughly doubled and that changes what the split can afford. `Worst gap` is
the widest degree separation between two labels that overlap — the number that
says whether an overlap is a conjunction or a mistake.

| Share | exact / spilled / stacked / packed | Worst gap | Travel, median | Travel, slowest |
|---|---|---|---|---|
| 0.50 | 22 / 1 / 1 / 6 | 22.8° | 8.7 | **0.0** |
| 0.65 | 24 / 1 / 4 / 1 | 6.8° | 22.2 | 11.5 |
| 0.75 | 25 / 0 / 4 / 1 | 6.8° | 15.6 | 7.5 |
| **0.80** | **25 / 1 / 3 / 1** | **3.1°** | **12.3** | **5.8** |
| 0.90 | 24 / 2 / 3 / 1 | 3.1° | 5.6 | 1.2 |

Three things the table settles.

**The original halving was the defect.** At 0.5 the worst overlapping pair is
nearly a whole sign apart, and one compartment does not move at all — a short
degree scale drives compartments into the packed fallback, and that fallback
moves only across whatever slack the rows leave.

**0.80 is a knee, not a preference.** It is the point at which every pair of
labels that overlap is inside the three degrees the spill is allowed to move a
body — the point at which every overlap on the chart is a conjunction. A
twentieth less doubles the worst overlap to nearly seven degrees, which is two
bodies a fifth of a sign apart drawn on top of each other.

**What it costs is the motion, and the cost is real.** 0.65 buys almost twice
the travel. §9.2 is what that is worth.

## 5. Crowding: what survives of `cluster`

| | Fate |
|---|---|
| The type-size ladder — 100%, 86%, 74%, 62% of 11px | **Kept**, unchanged (D-033) |
| `squeezed`, plus the `travel`/`drift` slide the shipped chart gives it | **Kept**, as the last rung — and it now slides, so nothing is ever frozen |
| `arrange`, `rows`, `withoutCaption` | **Gone.** They assume a body may go anywhere |
| `drift` as the pathway's motion | **Gone.** The crossing is already part of every station |

Four rungs, decided once per compartment and held for the crossing:

| Rung | What it allows | Compartments |
|---|---|---|
| 1. **exact** | every type size, degrees held exactly, nothing overlapping | 26 |
| 2. **spilled** | the same, a body may move up to **3°** along the degree axis | 1 |
| 3. **stacked** | the same, labels may overlap; the size that covers least wins | 2 |
| 4. **packed** | the shipped rows, degrees abandoned, still sliding | 1 |

Overlapping type is given up *after* the degree, because a chart that has moved
a body to keep its type clean is saying something untrue quietly, and
overlapping type is at least visibly hard to read.

**The rung-3-against-rung-4 rule**, which took three goes.

- *Total overlapping area* answers the wrong question. `squeezed` runs a single
  column down a tall compartment and frequently overlaps nothing at all, so the
  comparison threw away the degrees of eight bodies to save one unit of ink.
- *A threshold on the worst-covered label* is the right question and was asked
  of the wrong thing: it measured the degree-true arrangement against a constant
  and never against the alternative. A compartment that cannot hold seven bodies
  cannot hold them in rows either, and giving up the degrees buys nothing there.
- **Both.** If the degree-true arrangement is readable on its own — no label
  more than half covered — it wins. If it is not, the two are compared against
  each other on the same measure and the more readable one wins.

**Rung 4 is the shipped packer, and twice it was not.** It called `squeezed`
directly, which is the shipped layout's *last resort* rather than the shipped
layout; then it called `cluster`, which is the shipped layout but takes the
largest type that fits. Either way the arrangement fills the compartment and
`travel` finds no slack, and a compartment stood still for a whole crossing. The
candidates are now the shipped packer's arrangements at all four of its sizes
plus its own last resort, ordered by: off the caption first, then least covered,
then whichever of those can still move. Those are the invariants in their own
order — a label on the caption is broken, overlapping labels are degraded, and
standing still is only an opportunity lost.

---

## 6. The grid, which now ships

Advanced › **Degree grid**, off by default. Schema **13**, with an explicit
migration from 12 and a test that a 12 document without the field still loads
and arrives with the grid off.

Off by default, and it is the one switch here that does not ship on. `animate`
ships on because a reader who has never seen the chart move cannot ask for it;
the grid draws the chart's own scaffolding, and scaffolding that appeared
unasked over an existing reader's chart is a change to what their chart looks
like rather than an addition to what it can do.

| Mark | Dash | What it shows |
|---|---|---|
| Faint solid curve | — | the whole route, gate to gate |
| Base line | `4 2` | the line the degree scale is measured on |
| Other parallel lines | `2 2` | the rest of the family, over the run of each that holds a label |
| Degree marks, 0/15/30 | `2.5 2` | where those three degrees fall, at this instant |
| Minor marks, 5/10/20/25 | `0.75 2` | the same, quieter |
| Dots at the ends | — | the gates, where this compartment's route meets its neighbours' |

**Nothing is solid and nothing spans the compartment.** Both were tried and both
were wrong the same way. A solid orange line drawn wall to wall stopped being a
scale mark and became the grid: it out-drew the dotted family it was supposed to
annotate, and a reader saw three bars per house rather than degree lines with
bodies standing on them. Dashing it helped and was not enough — thirty-six
wall-to-wall marks still laid a rectilinear pattern over a curved one. They are
cross marks on the family now, the majors two and a half lanes wide.

The hierarchy is **extent, then rhythm, then weight, then opacity**, and the
family carries the longest dashes because it is the thing to read first.

The lines stopping short is the point: **the number of lines that survive at a
station is the number of bodies that station can hold**, so a reader can see why
a crowded house is crowded rather than being told. That is also why the degree
marks no longer reach the walls — the capacity was already in the drawing, and
spanning the chord said it twice in the louder of the two voices.

North Indian only. The other two formats' compartments are the signs and hand
nothing on, so there is no ring and no route.

---

## 7. Gate continuity

Content travels from the edge shared with the next house to the edge shared with
the previous one. Gates are found by intersecting two compartments' vertex
lists — the twelve shapes are built from nine shared points, so which edge two
houses have in common is already in the construction.

**Verified:** across all twelve joins and all three shapes, house *k*'s exit
point and house *k−1*'s entry point are the same point to the last bit. Worst
gap over the ring: **0.000000 units**.

---

## 8. Verification

The four static invariants are necessary and not sufficient — §1.2 is what they
missed, and §1.5 is what a tolerant check missed after that. Everything below is
measured **strictly**: any shared area at all counts, in every test, with no
tolerance anywhere. 101 steps of a whole crossing across the seven hardest
charts: D1, eight in a kite (February 1962), seven in a corner triangle, seven
in a wall triangle, D2 hora, D9 navamsa, D30 trimsamsa. Thirty occupied
compartments, 63 bodies, 106 pairs.

Layout rungs: **exact 26, spilled 1, stacked 2, packed 1.**

### Containment, over 6,363 rendered labels

| | |
|---|---|
| Label leaves its compartment | **0** |
| Label clipped by the chart's edge | **0** |
| Label touching a caption | **0** |

Cross-wall collisions are structurally impossible, not merely absent: every
compartment is convex and every label sits two units inside its own polygon, so
two labels in adjacent compartments cannot meet.

### Motion

| | First build | Now |
|---|---|---|
| Largest single-step displacement | 63.5 units | **0.29** |
| Median single-step displacement | — | 0.13 |
| Bodies backtracking along the route | — | **0 of 63** |
| Pairs exchanging order | 2 known | **0 of 106** |
| Bodies frozen (under 1 unit per crossing) | 2, one for 51 of 100 steps | **0 of 63** |
| Travel per crossing, median | — | 12.3 units |
| Travel per crossing, slowest | 0.0 | 5.5 units |

### Shear

A pair of bodies is rigid in the sky; on a curved route their separation vector
follows the tangent, so it turns as the group slides. Measured as the total
swing of that vector over one crossing.

| | Median | p95 | Max |
|---|---|---|---|
| Separation vector swing | **5.7°** | 56.0° | 66.3° |

**How much is free and how much is forced.** Preferring the flattest route among
those within a tenth of the best band takes the median from 9.2° to 5.7°, and it
is not paid for out of the reading — it puts one more compartment on `exact` and
takes one off `stacked`. The rest is forced, and forcing every route straight is
the measurement that shows it: shear falls to **zero**, and a kite's band falls
from about 125 units to about 25 — the degree scale from 3.3 units per degree to
0.7. A compartment's two gates lie on edges meeting at one vertex, so a route
that does not bend runs through the tightest part of the shape. **The bend is
the reading; the shear is its price.**

### Overlaps — both figures, labelled

An overlap between two bodies a degree apart is a conjunction. An overlap
between two bodies twelve degrees apart is a failure. The distinction lives
entirely in the one compartment that falls back to the packed rows, so quoting
one number without the other would hide exactly the case that fails.

| | Pairs | Worst separation of an overlapping pair |
|---|---|---|
| **Excluding the packed fallback** | 296 | **3.05°** |
| The packed fallback alone — `seven in a wall triangle` | 404 | **12.97°** |
| **Everything** | 700 | **12.97°** |

Excluding the fallback, every overlapping pair on the chart is inside the three
degrees the spill is allowed to move a body — every overlap is a conjunction.
Including it, the worst is 12.97°, and all 404 of those pairs are in the single
compartment §9.1 is about. The first build's figure, for comparison, was
**20.65°** and was not confined to any one compartment.

### Degree fidelity

Each label projected back onto its compartment's base line, the station
recovered, and the drawn gap compared with the true one. D1 charts only: in a
division the title carries the D1 degree while the station carries the part-run.
Reported by occupancy, not pooled.

| Compartment | Rung | Pairs | Median | p95 | Max |
|---|---|---|---|---|---|
| 2–3 bodies | exact | 22 | **0.00°** | 0.00° | 0.00° |
| 7–8 bodies | spilled | 297 | 2.39° | 3.54° | **3.55°** |
| 7–8 bodies | stacked | 220 | 2.39° | 3.35° | **3.35°** |
| 7–8 bodies | packed | 220 | 4.89° | 13.25° | 17.25° |

### The shipped chart, which none of this may make worse

The pathway is off in the panel, but §1.4 and §1.5 changed `placeCaption`,
`withoutCaption` and the drift's caption test, which the panel does use.
Measured by stashing the working tree, reloading, measuring, restoring, and
measuring again — the same method, the same box, the same box model.

| 23 cases, 207 labels | Baseline | Now |
|---|---|---|
| Overlapping label pairs | 61 | **51** |
| Labels outside their compartment | 3 | **0–1** |
| Labels touching a caption | 0 | **0** |

The shipped chart is better on two invariants and unchanged on the third. The
remaining `outside` is the handover case caught mid-animation: the arriving
group is deliberately animated in from behind the wall and clipped by the
compartment's own outline (`animation.md` §5), so it is a label that is not
drawn rather than a label in the wrong place. It is 0 when the animation is not
in flight, and it was 3 at baseline for the same reason.

The 23 cases also render **0 grid groups and 0 route lines** with the setting
off, so the panel builds no routes at all.

### Cost

A chart lays out in about 16 ms and then costs **4 ms per progress step**,
because the plan is memoised against the bodies and only `place` runs per frame.
Before the plan/place split every step re-decided every compartment and cost
111 ms.

## 9. Not resolved

1. **One compartment still abandons its degrees**, and it is the one the second
   figure in §8 is about: 404 overlapping pairs, worst separation 12.97°. Seven
   bodies in a wall triangle, five of them inside three degrees. The shape carries two or three
   parallel lines and the conjunction needs five, so the degree-true arrangement
   buries a label completely and the shipped rows are the more readable answer.
   Everything payable has been paid: the caption has moved to the narrow end of
   the wall (§1.4), the ruler is measured with the narrowest label, and the
   route is chosen for the longest band. What is left is the shape.

   What would fix it is not in this document: a wall triangle is 79 units across
   at its widest and its caption takes 25 of them, so the remaining moves are to
   drop the caption from those four compartments, or to set them in smaller type
   than the rest of the chart always. Both change what every reader sees for the
   sake of a conjunction most charts do not have.

2. **No body moves fast enough to see moving.** The floor is arithmetic:
   travel per crossing is the band times one minus the share, so the slowest
   compartment on the chart moves 5.8 units in the two hours D1 takes to cross a
   sign. That is a fiftieth of a unit a minute.

   Raising it is possible and costs the reading. §4 measures it: a share of 0.65
   nearly doubles the travel and doubles the worst overlapping pair's separation
   from three degrees to seven. Under whole-sign houses the two come out of the
   same band and there is no third place to take it from.

   So this is a limit of the design rather than a number waiting to be improved.
   **The traversal is a position, not an animation** — the same thing
   `animation.md` §1 says of the drift it replaces, and the reason the chart is
   still legible with the motion switched off. What the traversal buys is that
   the position now *means* something: it is the body's degree, and it is where
   the ring will carry it. The one motion in the chart fast enough to watch is
   still the handover.

3. **Shear of up to 66°, in the two most deeply bowed kites.** §8 shows the free
   part has been given back and the forced part is what buys the degree scale.
   It could only be removed by straightening the routes, which costs four fifths
   of the band.

4. **The handover's jump is bounded but not looked at.** §2 shows it is at most
   the degree spread and cannot be removed. Whether it reads as a slide or a
   flicker at 260 ms is a thing to watch.

5. **Three judgements, not measurements.** The 3° spill, the half-covered
   legibility threshold, and the tenth of band a route may trade for flatness.
   Each is stated in the units a reader would be misled in.

6. **Nothing here touches South or East Indian**, and the pathway placement does
   not ship. The grid does.
