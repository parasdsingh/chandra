# The chart, moving

Specification for **E6.6**. Status: **proposed.** Nothing here is built.

The Lagna Kundali is a reading of *now*, and it does not look like one. It is
redrawn every sixty seconds and every redraw is indistinguishable from the last,
so a chart left open reads as a picture of a moment rather than as an instrument.

The lagna moves about a degree every four minutes. In D1 that is one rashi every
two hours; in D9 one every thirteen minutes; in D60 one every two. There is real
motion in this object and none of it is on screen.

---

## 1. What moves, and what does not

Two things in a chart change at rates a person can see, and they are not the same
kind of change.

| | Rate | What a reader gets from it |
|---|---|---|
| **The lagna** | a degree per four minutes | which rashi is rising, and how far through it |
| **The grahas** | Chandra crosses a rashi in 2.3 days; Shani in 2.5 years | where the bodies stand |

So the lagna is the thing to animate. A graha's motion is not visible on any
timescale a panel is open for — but *where a graha stands within its sign* is
information the chart currently throws away, and it is the same mapping the
lagna's motion needs. That is E6.5, and it is here rather than in E1 because
solving it twice would be solving it twice.

**This document proposes the lagna's motion only, and defers graha placement.**
The reason is in §6.

---

## 2. What the motion is, per format

The three formats disagree about what is fixed, so they disagree about what
moves.

### North Indian — the houses are fixed, the signs move

House 1 is always the top kite. When the lagna crosses from Mesha into
Vrishabha, house 1's sign becomes Vrishabha and **every compartment's sign
advances by one**. The whole ring of twelve labels rotates one position.

That rotation is the event. The animation's job is to make it *legible before it
happens*, which is what the drift does: the sign's name crosses its compartment
as the lagna crosses the sign, so a label against the far wall is a label about
to hand over.

```
  house 1, over one rashi of lagna motion

  0%      ┌──────────┐        the lagna has just entered this rashi
          │ Mesha    │
          └──────────┘
  60%     ┌──────────┐
          │    Mesha │
          └──────────┘
  99%     ┌──────────┐        about to hand over
          │      Mesha│
          └──────────┘
  100%    ┌──────────┐        every label in the chart has moved one house
          │ Vrishabha│
          └──────────┘
```

**Direction.** Along the house order, so the drift reads as motion *around* the
chart rather than as twelve unrelated slides. Each compartment has an entry edge
and an exit edge: the edge it shares with the previous house, and the edge it
shares with the next.

### South and East Indian — the signs are fixed, the lagna moves

The compartments *are* the rashis and do not move. What changes is which cell is
house 1, marked today by a diagonal stroke.

So there is no rotation to foreshadow. The motion is the lagna mark itself
travelling across its cell, arriving at the wall it shares with the next rashi as
the handover comes.

### Both

The **caption** already prints the lagna's degree and already updates. It is the
numeric form of exactly this motion and needs no change.

---

## 3. What is computed, and how often

**Nothing is interpolated.** Every position drawn comes from a lagna the
ephemeris returned. Drawing a lagna between two computed ones would be the app
inventing a figure, which is the thing the Moshier note exists to prevent.

That is affordable because a chart is cheap. Measured with
`src-tauri/examples/bench_chakra.rs`:

| | |
|---|---|
| One chart | **47 µs** |
| At 1 Hz | 0.0047% of one core |
| At 10 Hz | 0.047% of one core |

**One hertz.** The slowest division moves a compartment's width in two hours, so
one hertz is 7,200 steps across it; the fastest, D60, is 120 steps. Neither is
choppy, and nothing is gained by asking the ephemeris more often than the eye
can resolve.

**Only while the panel is open.** The timer is created when the chart becomes the
view and destroyed when it stops being it, which is what the existing
sixty-second refresh already does. A menu bar app that computes on a timer while
nobody is looking is a menu bar app that shows up in Activity Monitor.

**A toggle in Advanced, on by default.** Motion in a panel is a preference, and
somebody who does not want it should not have to accept it to read a chart.

**`prefers-reduced-motion` overrides the toggle.** A reader who has asked their
system for stillness has asked, and the chart is fully usable static — the
caption still carries the degree.

---

## 4. What has to be true afterwards

The layout has invariants that took three rounds of measurement to establish
(D-033, and `docs/TODO.md` E1 defect ii). Motion must not cost them.

| Invariant | Why it is at risk |
|---|---|
| No label leaves its compartment | The label is now moving toward a wall on purpose |
| No label overlaps another | A drifting label passes positions a static one never occupied |
| No label sits on the compartment's caption | The caption *is* the thing moving |
| Nothing is clipped by the chart's own edge | A wall triangle's long edge is the chart's edge |

The harness checks all four geometrically over 126 labels in 16 charts. **The
same check must run over the drift**, sampled across a full rashi rather than at
one instant — an arrangement that is legal at 0% and at 100% can be illegal at
43%.

That is the acceptance test for this work, and it is the reason the harness needs
a case that steps the lagna rather than fixing it.

---

## 5. Open questions

These need answers before implementation, and none of them is settled by
research.

1. **Does the label drift, or does the whole compartment's content?** The user's
   description is of the sign's name moving. The grahas standing in that sign
   move with it in reality — the sign is what carries them. Moving the name
   alone is cheaper and safer; moving the group says something truer.

2. **What happens at the handover?** A cut, or a slide? A slide across a
   compartment wall crosses a line that means something. A cut is honest and
   abrupt. There is a third option: the label fades at the exit edge and
   reappears at the entry edge, which is neither.

3. **How far does the label actually travel?** Wall to wall looks strongest and
   collides most: a label at 0% and one at 100% are at opposite ends of the
   compartment, and the caption reservation was built for a label that does not
   move. A shorter travel — say the middle two thirds — costs some of the effect
   and most of the risk.

4. **Does South Indian animate at all?** Its lagna mark is a diagonal stroke
   across a cell. Moving it means either drawing a different stroke each frame or
   moving something else, and neither is obviously right.

5. **Does the drift show anywhere else?** The tray tooltip already rebuilds on
   hover and is current to the second. The calendar does not move and should not.

---

## 6. Why graha placement by degree is not in this pass

It is the same mapping and it belongs here eventually. It is deferred because it
does not compose with the layout as it stands.

`cluster` places bodies in rows and columns, then moves each row as a unit to
keep it inside the compartment, then shrinks the type if no arrangement fits.
Every one of those steps assumes it may put a body **anywhere** in the shape.
Placing a body at its own degree removes that freedom along one axis, and two
bodies within a degree of each other — which is a conjunction, which is exactly
what a reader is looking at the chart for — would be placed on top of one
another.

So it needs a rule of its own: degree fixes one axis, and the packing gets the
other. That is a redesign of `cluster`, not an addition to it, and it should not
ride along with the animation.

---

## 7. What this does not do

- It does not animate the grahas. They do not move on this timescale.
- It does not animate the calendar. A month is not a reading of now.
- It does not make the chart a clock. There is no second hand, no ticking, and
  nothing that draws the eye when the reader is looking at something else.
