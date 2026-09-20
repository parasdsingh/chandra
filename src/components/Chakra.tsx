/**
 * The Lagna Kundali: where the nine grahas stand now, and what is rising.
 *
 * Three formats, one payload. They disagree about where on screen a rashi is
 * drawn and what is written in its compartment; they agree about everything that
 * is true. So the back end sends the twelve rashis with their occupants and the
 * lagna, and the whole of the difference lives here.
 *
 * All three are SVG, and the geometry is *derived* from the box rather than
 * written down. The North Indian chart is a square, its two diagonals, and the
 * diamond joining the midpoints of its sides; every vertex follows from that.
 * Twelve hand-tuned polygons would be twelve chances to be wrong with no way to
 * see that any of them was.
 */

import type { JSX } from "solid-js";
import {
  createEffect,
  createMemo,
  createUniqueId,
  For,
  Index,
  onCleanup,
  Show,
} from "solid-js";

import type {
  Chakra as ChakraData,
  ChakraGraha,
  ChakraRashi,
  ChartFormat,
} from "../ipc/types";

/** The chart's coordinate system, not its pixels.
 *
 * A rectangle, not a square. The panel is 320 wide and its region is 264 tall,
 * and every view lives inside that - so a square at the full width would need
 * the window to grow, which was tried and reverted. Drik Panchang draws its
 * North Indian chart at 3:2 for the same reason: the construction needs a
 * rectangle, not equal sides.
 *
 * 318 by 240 leaves the caption its line and the view its padding, and uses the
 * whole width rather than leaving 35px of dead margin down each side.
 *
 * The half-pixel inset is the frame. The outer polygon's points sit *on* these
 * bounds, and a 1px stroke centred on the boundary loses half its width to the
 * viewBox edge - which is why the chart read as bleeding off the window rather
 * than sitting in it. Half a pixel in, the whole stroke is inside. */
const WIDTH = 318;
const HEIGHT = 240;
const INSET = 0.5;

/** Row height and column width for the grahas clustered in one compartment.
 *
 * The type is 11px and its rendered box 12.64 tall, so a 12px row is a little
 * tighter than the boxes: adjacent rows overlap by about 0.6 of a box. Nothing
 * is visible, because a box is measured to the ascender and the ink of a
 * two-letter capitalised name reaches about 8 - and buying that 0.6 back would
 * cost a row of capacity in every compartment that is nearly full, which is
 * where it would hurt.
 *
 * The column is the *pitch* between two labels, not the room one needs. That is
 * `GRAHA_HALF`, and conflating the two is what once let a label cross a wall. */
const GRAHA_ROW = 12;
const GRAHA_COLUMN = 30;
/** Clearance kept between a cluster and the compartment's own edges. */
const GRAHA_MARGIN = 8;

interface Point {
  x: number;
  y: number;
}

/** One compartment: its shape, where its two kinds of text sit, and what it holds. */
interface Compartment {
  points: Point[];
  /** The compartment's caption - the rashi's short form, or its number. */
  label: Point;
  /** How that caption is anchored. The wall triangles hug a side, so theirs
   *  starts or ends at the wall rather than centring on a point. */
  labelAnchor: "start" | "middle" | "end";
  /** The middle of the cluster of grahas. */
  body: Point;
  rashi: ChakraRashi;
  /** Zero-based sign index, for the number the North Indian chart writes. */
  sign: number;
  /** The unit direction the sign travels as the lagna crosses it: from the
   *  previous house's middle toward the next one's. Absent where nothing
   *  travels, which is both grid formats. */
  along?: { x: number; y: number };
  /** The route the sign's contents take across this compartment, from the edge
   *  they arrive through to the edge they leave by. Absent in both grid
   *  formats, whose compartments are the signs and do not hand anything on. */
  path?: Pathway;
}

function centroid(points: Point[]): Point {
  return {
    x: points.reduce((sum, p) => sum + p.x, 0) / points.length,
    y: points.reduce((sum, p) => sum + p.y, 0) / points.length,
  };
}

function polygon(points: Point[]): string {
  return points.map((p) => `${p.x},${p.y}`).join(" ");
}

/** A point `fraction` of the way from `from` to `to`. */
function toward(from: Point, to: Point, fraction: number): Point {
  return {
    x: from.x + (to.x - from.x) * fraction,
    y: from.y + (to.y - from.y) * fraction,
  };
}

/**
 * How wide the compartment is at a given height.
 *
 * The horizontal slice of the polygon at `y`, found by intersecting its edges
 * with that line. A bounding box will not do: a corner triangle's box is as wide
 * as the whole quadrant while the shape at the height the text sits at is a
 * fraction of that, which is how `Su` came to be written outside its own
 * compartment and off the edge of the panel.
 */
function spanAt(points: Point[], y: number): { min: number; max: number } {
  let min = Infinity;
  let max = -Infinity;

  for (let i = 0; i < points.length; i++) {
    const a = points[i]!;
    const b = points[(i + 1) % points.length]!;
    // Only edges that straddle this height can cross it.
    if (a.y === b.y || y < Math.min(a.y, b.y) || y > Math.max(a.y, b.y))
      continue;
    const x = a.x + ((y - a.y) / (b.y - a.y)) * (b.x - a.x);
    min = Math.min(min, x);
    max = Math.max(max, x);
  }

  return min <= max ? { min, max } : { min: 0, max: 0 };
}

/**
 * The narrowest the shape gets over a band of heights.
 *
 * `spanAt` answers for a single scanline, and a label is not a scanline. A
 * polygon's span is piecewise linear in y, so the narrowest point over an
 * interval is at one of its ends or at a vertex between them; those are the
 * only heights worth testing, and testing them is exact rather than a sample.
 *
 * The band is clamped into the shape's own range first. A label whose box
 * reaches past the tip of a triangle would otherwise ask `spanAt` about a height
 * the shape does not occupy, and get the empty span back as though the shape
 * were zero-width there.
 */
function spanOver(
  points: Point[],
  top: number,
  bottom: number,
): { min: number; max: number } {
  const ys = points.map((point) => point.y);
  const lowest = Math.min(...ys);
  const highest = Math.max(...ys);
  const from = Math.min(Math.max(top, lowest), highest);
  const to = Math.min(Math.max(bottom, lowest), highest);

  const heights = [from, to];
  for (const y of ys) if (y > from && y < to) heights.push(y);

  let min = -Infinity;
  let max = Infinity;
  for (const y of heights) {
    const span = spanAt(points, y);
    min = Math.max(min, span.min);
    max = Math.min(max, span.max);
  }
  return { min, max };
}

/**
 * The rectangle a compartment's caption occupies.
 *
 * The anchor decides which side of `at.x` the text runs, which is the same
 * three cases `text-anchor` has.
 */
function captionBox(
  at: Point,
  anchor: "start" | "middle" | "end",
  numbered: boolean,
): Box {
  const width = numbered ? CAPTION_NUMBER_WIDTH : CAPTION_NAME_WIDTH;
  const left =
    anchor === "start"
      ? at.x
      : anchor === "middle"
        ? at.x - width / 2
        : at.x - width;

  return {
    min: left,
    max: left + width,
    top: at.y - CAPTION_ABOVE,
    bottom: at.y + CAPTION_BELOW,
  };
}

interface Box {
  min: number;
  max: number;
  top: number;
  bottom: number;
}

/** A graha label's box, measured from its baseline at the caption size.
 *
 * From the rendered text rather than derived from the font size: at 11px the box
 * runs 10.53 above the baseline and 2.11 below it. Rounded outwards, because the
 * point of them is to keep the label off the wall. */
const GRAHA_ABOVE = 11;
const GRAHA_BELOW = 3;

/** Half the widest graha label.
 *
 * `(Ke)` measures 25.0 against a plain `Mo`'s 16.4 - the retrograde brackets are
 * what makes the widest case, and they are on the name rather than beside it so
 * that this stays as small as it is. The widest is what every label is clamped
 * by: `cluster` places positions and never sees the text.
 *
 * Not `GRAHA_COLUMN / 2`, which is what it used to be. The column is the pitch
 * between two labels; this is how much room one label needs. They were the same
 * number by coincidence and neither was measured. */
const GRAHA_HALF = 13;

/** Half a graha label that is *not* retrograde.
 *
 * `Mo` measures 16.4 against `(Ke)`'s 25.0, so clamping every label by the
 * widest throws away eight units of width on eight labels in nine. The packed
 * layout can afford that - it places positions and never sees the text - and
 * the pathway cannot: a wall triangle is 79 units across at its widest and its
 * caption takes 25 of them, so the difference between 26 and 18 is the
 * difference between a route with a usable band and one without. Measured at
 * scale 1 in the harness and rounded outwards, like its neighbour. */
const GRAHA_HALF_PLAIN = 9;

/** The compartment's caption, as a box to keep grahas out of.
 *
 * Measured from the rendered labels: a three-letter name is 24 wide and a house
 * number 12.4, and both run 9.5 above their baseline and 2.1 below. Rounded
 * outwards.
 *
 * Reserving it is not optional. `cluster` used to know only the compartment's
 * outline, so a graha could be placed exactly where the caption already was -
 * `Leo` sat under `Su` in the North Indian chart, and `Cap` under `Ma` in the
 * South. Both were found by measuring label boxes against graha boxes; neither
 * was visible as anything worse than slightly heavy text. */
const CAPTION_NAME_WIDTH = 25;
const CAPTION_NUMBER_WIDTH = 13;
const CAPTION_ABOVE = 10;
const CAPTION_BELOW = 3;

/** A compartment caption's reach from its baseline, plus the clearance it keeps
 *  off the frame.
 *
 *  Larger than the measured box - `CAPTION_ABOVE`/`CAPTION_BELOW` are 10 and 3 -
 *  because these place the caption while those reserve room around it. The
 *  clearance is the point: at the measured figures the captions were technically
 *  inside their compartments and visually sitting on the frame, which reads as a
 *  clipped label rather than a placed one.
 *
 *  Two sets of numbers for one piece of text is a hazard, and they are kept
 *  apart deliberately: changing what a caption *reserves* should not silently
 *  move where it is *put*. */
/** How far outside its compartment an arriving group starts.
 *
 * Far enough to be wholly behind the wall at the moment it begins - a group that
 * fades in from just inside reads as a flicker rather than as an arrival. The
 * clip path is what makes this safe: none of it is drawn until it is inside. */
const HANDOVER = 26;

/** The gap between two positions becomes the duration of the slide between
 * them, so each slide ends as the next arrives and the motion is continuous
 * rather than a step followed by a wait. Measured rather than agreed with the
 * caller: the panel feeds this once a second and the harness ten times a
 * second, and the renderer is told neither.
 *
 * Clamped before it is used. Below the floor a slide is shorter than a frame
 * and buys nothing; above the ceiling a hiccup in the feed would leave a body
 * crawling toward a position the chart has already moved past. */
const SETTLE_LEAST = 80;
const SETTLE_MOST = 2_000;

/** The sky behind the chart.
 *
 * Fixed at module load, never regenerated. The chart is redrawn every second
 * while it animates, and a sky drawn from `Math.random` would be a different
 * sky on every one of them - the one thing a background must not be. A seeded
 * generator gives the same field on every run of the app, so the stars are a
 * property of the drawing rather than of the moment it was drawn.
 *
 * Thinned toward the middle rather than spread uniformly: labels gather near
 * the centre of the chart, and the corners of the square are where no
 * compartment reaches. It thins the odds rather than clearing the ground -
 * measured, seven of a hundred and twenty still fall inside a label's box, and
 * they have to be allowed to. The bodies move every second, so a field culled
 * against where they are now would be a different field every second, and a
 * sky that reshuffles is worse than a star behind a letter.
 */
interface Star {
  x: number;
  y: number;
  r: number;
  /** The two ends of the twinkle: resting, and at its brightest. */
  a: number;
  lit: number;
  /** Seconds for one twinkle, and how far into it this star starts, so the
   *  field does not pulse as one. */
  period: number;
  delay: number;
  /** Whether this star twinkles at all.
   *
   * Only the brightest few. A running animation per star is a compositor
   * animation per star, and a hundred and twenty of them, in a menu bar app,
   * for decoration, is not a trade worth making - the visual harness draws
   * three charts and its three hundred and sixty animations were enough to stop
   * the page answering. Seven of a hundred and twenty carry the whole effect,
   * because a sky reads as alive if anything in it moves. */
  lively: boolean;
}

/** Mulberry32. Small, fast, and - the only property that matters here -
 *  identical on every platform and every run for a given seed. */
function seeded(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** How many stars. Few and legible rather than many and faint.
 *
 * The first attempt drew a hundred and twenty, and ninety-nine of them were
 * 0.4 units across - about one and a half device pixels on a 2x screen at the
 * panel's own scale. A circle that small cannot render as a point of light: it
 * antialiases into a grey smudge, and a field of them reads as dirt on the
 * screen rather than as a sky. On the panel it is worse than in the harness,
 * because the ground there is translucent material over whatever is behind the
 * window rather than flat black, so the smudges sit on a ground that moves.
 *
 * Nothing here is now below three device pixels. */
const SKY_COUNT = 54;

const SKY: Star[] = (() => {
  const random = seeded(0x43414e44);
  const stars: Star[] = [];
  const midX = WIDTH / 2;
  const midY = HEIGHT / 2;
  let guard = 0;
  while (stars.length < SKY_COUNT && guard++ < 4000) {
    const x = INSET + random() * (WIDTH - INSET * 2);
    const y = INSET + random() * (HEIGHT - INSET * 2);
    // Kept toward the edges, hard. Labels gather across the middle of the
    // chart, and the corners of the square are where no compartment reaches -
    // so the further from centre, the likelier a star survives. Squared, so the
    // middle is nearly empty rather than merely thinner.
    const away = Math.min(1, Math.hypot((x - midX) / midX, (y - midY) / midY));
    if (random() > away * away) continue;
    // The *halo*, not the dot. The gradient's lit core is the inner fifth of
    // this, so these read as cores of roughly half a unit to one unit with a
    // glow around them - which is the shape of a point of light, and the thing
    // a flat disc of any size could not be.
    const bright = random();
    const r = bright > 0.9 ? 4.2 : bright > 0.62 ? 3.0 : 2.2;
    // Bright, because the ground beneath is now pitch. The first two attempts
    // were dim because they were drawn over the panel's translucent material,
    // where anything bright would have glared; on black a star can be a star.
    const a = 0.45 + bright * 0.5;
    stars.push({
      x,
      y,
      r,
      a,
      // Far enough to be seen. A third brighter was invisible at these sizes;
      // the dim end drops well under the resting value and the lit end goes to
      // white, so a twinkling star is doing something a still one is not.
      lit: Math.min(1, a * 1.55),
      // Varied, and short enough to be caught. Five to eleven seconds meant a
      // star changed too slowly to be seen changing, which is a still sky with
      // a cost. Two to six, and no two stars on the same clock.
      period: 2 + random() * 4,
      delay: random() * 6,
      // A third of them, not six. Six was a performance decision taken when
      // there were a hundred and twenty stars and the harness drew three charts
      // of them; at fifty-four, a third is eighteen a chart, and a sky twinkles
      // or it does not. Still not all of them: a field where everything pulses
      // reads as a fault rather than as a sky.
      lively: random() < 0.34,
    });
  }
  return stars;
})();

/** Beyond this gap between two positions, the body is moved rather than slid.
 *
 * A slide says "this is where it has got to since a moment ago". When the panel
 * has been closed, or the window occluded and its timers throttled, the last
 * drawn position is not a moment old - it is however long the panel was away -
 * and sliding from it plays the whole absence back as a swoop the instant the
 * chart appears. A chart being opened should show now, not catch up to it.
 *
 * Four seconds, against a feed that arrives every one: long enough that no
 * ordinary tick is mistaken for an absence, short enough that any real stall is.
 * Measured from the positions themselves rather than from a panel-open event,
 * because the renderer does not know what a panel is, and the same stall
 * happens to a background tab that never opened or closed anything. */
const SETTLE_STALE = 4_000;

const LABEL_ASCENT = 12;
const LABEL_DESCENT = 6;
/** Half the width of a three-letter caption, plus the same clearance. */
const LABEL_HALF = 15;

/**
 * Where a North Indian compartment's caption goes, following Drik Panchang.
 *
 * The convention is not one rule but three, and which applies depends on the
 * compartment's shape and where it sits:
 *
 * | Houses | Shape | Caption sits at |
 * |---|---|---|
 * | 1, 4, 7, 10 | diamond | the vertex farthest from the centre |
 * | 2, 6, 8, 12 | corner triangle | the vertex farthest from the centre |
 * | 3, 5, 9, 11 | wall triangle | the middle of the wall it touches |
 *
 * The last group is what a single "outermost vertex" rule gets wrong. Those four
 * triangles have their long side against the chart's left or right wall, and
 * their farthest vertex is a corner they share with a neighbour - so two
 * captions ended up in the same corner. Against the wall, vertically centred,
 * they sit in the part of the shape that is actually theirs.
 *
 * A number is placed differently again: nearest the centre, where the
 * compartment is widest and a one or two character label does not need the room
 * a name does.
 */
function placeCaption(
  points: Point[],
  house: number,
  centre: Point,
  middle: Point,
  numbered: boolean,
): { at: Point; anchor: "start" | "middle" | "end" } {
  const far = (a: Point, b: Point) =>
    Math.hypot(a.x - centre.x, a.y - centre.y) >
    Math.hypot(b.x - centre.x, b.y - centre.y)
      ? a
      : b;

  if (numbered) {
    // Nearest the centre of the chart.
    const near = points.reduce((closest, point) =>
      far(closest, point) === closest ? point : closest,
    );
    return {
      // Three tenths of the way in from the vertex. A caption placed *on* a
      // vertex is placed on the frame; this is far enough in to clear it and
      // near enough to still read as belonging to that corner. Smaller than the
      // name's 0.4 below because a one or two character number needs less room
      // to sit clear.
      at: clampInside(points, toward(near, middle, 0.3)),
      anchor: "middle",
    };
  }

  const wallHouse = house === 3 || house === 5 || house === 9 || house === 11;
  if (wallHouse) {
    // The one edge that lies on the chart's left or right wall.
    const wall = edges(points).find(
      ([a, b]) => a.x === b.x && (a.x === INSET || a.x === WIDTH - INSET),
    );
    if (wall) {
      const [a, b] = wall;
      const onLeft = a.x === INSET;
      const anchor = onLeft ? "start" : "end";
      const x = a.x + (onLeft ? LABEL_WALL : -LABEL_WALL);

      // **Where the compartment is too narrow to hold a body**, walking from
      // the chart's corner along the wall until the caption itself fits.
      //
      // It used to sit at the wall's midpoint, which is the widest part of the
      // shape and the only part of it a graha can stand in. That put the one
      // fixed obstacle in a wall triangle exactly where its degree scale wanted
      // to be, and it cost more than it looked: the band a body can stand on
      // came out at twelve to twenty-four units against a kite's hundred and
      // thirty, and two of the twelve compartments could not hold their degrees
      // at all.
      //
      // The other eight compartments have always put their caption at a far
      // vertex, which is the narrowest part of *their* shape. So this is not a
      // new rule for the wall triangles; it is the rule the rest of the chart
      // already follows, applied to the four that were the exception. The
      // convention both reference applications show - the caption against the
      // long side - is kept; only where along it changes.
      //
      // Toward the chart's corner rather than toward the middle of the side.
      // Both ends of the wall are narrow, and the corner end leaves the longer
      // run of route: a wall triangle's two gates sit either side of its apex,
      // so blocking the end nearest a gate costs less than blocking the middle.
      const corner = Math.abs(a.y - INSET) < Math.abs(b.y - INSET) ? a : b;
      const inward = corner === a ? b : a;

      const fits = (y: number) => {
        const box = captionBox({ x, y }, anchor, false);
        return (
          inside(points, box.min, box.top) &&
          inside(points, box.max, box.top) &&
          inside(points, box.min, box.bottom) &&
          inside(points, box.max, box.bottom)
        );
      };

      // A shallow scan rather than an interval: the wall is at most 120 units
      // and half a unit of resolution is finer than the caption can be placed.
      const span = inward.y - corner.y;
      const steps = 240;
      for (let i = 0; i <= steps; i++) {
        const y = corner.y + (span * i) / steps;
        // Two units further in than the first position that fits, so the
        // caption is not sitting exactly on the limit of its own room.
        if (fits(y) && fits(y + Math.sign(span) * 2)) {
          return { at: { x, y: y + Math.sign(span) * 2 }, anchor };
        }
      }

      // Nothing along the wall holds it, which no compartment in this
      // construction reaches. The midpoint is the least bad answer there is.
      return {
        at: { x, y: (a.y + b.y) / 2 + LABEL_ASCENT / 3 },
        anchor,
      };
    }
  }

  return {
    // Four tenths in from the outermost vertex, for the same reason as the
    // number's three: a three-letter name is wider and needs to start further
    // from the point before it clears both walls of the wedge.
    at: clampInside(points, toward(points.reduce(far), middle, 0.4)),
    anchor: "middle",
  };
}

/** The polygon's edges, as pairs of consecutive points. */
function edges(points: Point[]): [Point, Point][] {
  return points.map((point, i) => [point, points[(i + 1) % points.length]!]);
}

/** Clearance a wall-hugging caption keeps from the wall itself. */
const LABEL_WALL = 4;
/** How much room a wall-hugging caption takes, and so how far the grahas in that
 *  compartment step away from the wall to clear it. */
const CAPTION_WIDTH = 22;

/**
 * Pushes a caption wholly inside its compartment.
 *
 * Anchoring alone was not enough once the chart became a rectangle: the corner
 * triangles are half as tall as they are wide, so a caption placed a fraction of
 * the way in from the corner still had its ascenders above the shape's top edge
 * and was clipped by the viewBox.
 */
function clampInside(points: Point[], start: Point): Point {
  const ys = points.map((point) => point.y);
  const top = Math.min(...ys) + LABEL_ASCENT;
  const bottom = Math.max(...ys) - LABEL_DESCENT;
  const y = top <= bottom ? Math.min(Math.max(start.y, top), bottom) : start.y;

  const span = spanAt(points, y);
  const lowest = span.min + LABEL_HALF;
  const highest = span.max - LABEL_HALF;
  const x =
    lowest <= highest
      ? Math.min(Math.max(start.x, lowest), highest)
      : (span.min + span.max) / 2;

  return { x, y };
}

/**
 * Where each graha sits, as an absolute point inside its compartment.
 *
 * Not a single column. Four bodies as one tall stack spill out of a corner
 * triangle and read as a list rather than as a group - and a published chart
 * writes them as a cluster, because that is what "these are together in this
 * sign" looks like.
 *
 * So they flow into rows, and the number of columns is what the compartment can
 * actually hold at the height the row sits at rather than a constant. Rows are
 * centred on each other, short rows are centred on their own width, and each row
 * is finally pushed inside the compartment's edges - a wedge is narrow at one
 * end, and the row nearest the point is the one that would otherwise escape.
 */
function cluster(
  points: Point[],
  centre: Point,
  count: number,
  caption: Box,
): Cluster {
  // Full size first, then smaller. A crowded compartment is set in smaller type
  // rather than in overlapping type, which is what a printed chart does and what
  // the alternative forced: eight bodies in a D2 corner triangle came out at a
  // 4.1 unit row pitch against a 12.6 unit label, stacked three deep.
  //
  // D2 is why this is not optional. Every chart looks like that in the hora -
  // all nine bodies land in Karka and Simha, so one compartment routinely holds
  // eight - and a division that is illegible every day is not a degraded state,
  // it is a broken feature.
  //
  // The floor is 0.62, which is 11px down to about 7. Below that the ink is
  // thinner than the frame it sits in.
  // One size. See PATH_SIZES: two sizes in one chart read as a distinction, and
  // the only thing they distinguish is which house is busier.
  for (const scale of [1]) {
    const laid = arrange(points, centre, count, caption, scale);
    if (laid) return { at: laid, scale };
  }

  // Not even the smallest type fits. The rows are packed into the heights that
  // can hold one and allowed to overlap, because a body drawn outside its
  // compartment is *wrong* - it reads as standing in a sign it is not in - and
  // overlapping text is only hard to read.
  return { at: squeezed(points, centre, count, caption, 1), scale: 1 };
}

/** Where a compartment's bodies go, and how large they are set. */
interface Cluster {
  at: Point[];
  scale: number;
  /** The stretch of the route a label may stand on, as two fractions of its
   *  length. Only the pathway layout produces one; the development grid draws
   *  it, and nothing else reads it. */
  band?: [number, number];
  /** How the arrangement was arrived at. Written onto the group as a data
   *  attribute when the pathway is on, so the harness can say which
   *  compartments held their degrees and which had to spend something -
   *  measuring that from rendered positions alone is guesswork. */
  how?: "exact" | "spilled" | "stacked" | "packed";
}

/**
 * The best arrangement at one type size, or nothing if none fits.
 */
function arrange(
  points: Point[],
  centre: Point,
  count: number,
  caption: Box,
  scale: number,
): Point[] | null {
  const above = GRAHA_ABOVE * scale;
  const below = GRAHA_BELOW * scale;
  const half = GRAHA_HALF * scale;
  const column = GRAHA_COLUMN * scale;

  // The band a baseline may sit in, so the whole label stays inside the
  // compartment's height. Every arrangement is placed within it rather than
  // placed and then checked, which is what makes containment structural rather
  // than something a search can fall through.
  const ys = points.map((point) => point.y);
  const top = Math.min(...ys) + GRAHA_MARGIN / 2 + above;
  const bottom = Math.max(...ys) - GRAHA_MARGIN / 2 - below;

  const desired = count <= 1 ? 1 : count <= 4 ? 2 : 3;

  // Every shape the count can be laid out in, best first. More columns is a
  // shorter, wider block and fewer is a taller, narrower one, so which of them
  // fits depends on the compartment: a corner triangle runs out of width, and a
  // tall kite runs out of height. Neither direction is always the right one to
  // try, so both are tried and the one nearest the preferred shape wins.
  const shapes = Array.from({ length: count }, (_, index) => index + 1).sort(
    (a, b) => Math.abs(a - desired) - Math.abs(b - desired) || a - b,
  );

  for (const pitch of [GRAHA_ROW * scale, GRAHA_ROW * scale * 0.85]) {
    for (const columns of shapes) {
      const lines = Math.ceil(count / columns);
      const reach = ((lines - 1) / 2) * pitch;

      // The block's centre can only sit where the whole block stays in the band.
      // If that interval is empty the block is taller than the compartment and no
      // offset saves it.
      const lowest = top + reach;
      const highest = bottom - reach;
      if (lowest > highest) continue;

      // Three preferences, each clamped into the feasible interval rather than
      // used raw. Clamping is the fix for a whole class of near misses: one
      // arrangement failed by an eighth of a pixel, and a shift of that much
      // fitted it.
      const preferences = [
        centre.y,
        caption.bottom + above + reach,
        caption.top - below - reach,
      ];

      for (const preferred of preferences) {
        const middle = Math.min(Math.max(preferred, lowest), highest);
        const laid = rows(points, centre.x, middle, count, columns, caption, {
          pitch,
          above,
          below,
          half,
          column,
        });
        if (laid) return laid;
      }
    }
  }

  return null;
}

/**
 * The last resort: one column packed into whatever heights can hold a label.
 *
 * Spreading the stack over the compartment's full height puts rows where the
 * shape is narrower than one label - and in a wall triangle, whose long edge
 * *is* the chart's edge, a label centred in a five-unit slot hangs five units off
 * the side of the drawing.
 */
function squeezed(
  points: Point[],
  centre: Point,
  count: number,
  caption: Box,
  scale: number,
): Point[] {
  const above = GRAHA_ABOVE * scale;
  const below = GRAHA_BELOW * scale;
  const half = GRAHA_HALF * scale;

  const ys = points.map((point) => point.y);
  const top = Math.min(...ys) + GRAHA_MARGIN / 2 + above;
  const bottom = Math.max(...ys) - GRAHA_MARGIN / 2 - below;

  const needed = half * 2 + GRAHA_MARGIN;
  const fits = (y: number) => {
    const span = spanOver(points, y - above, y + below);
    return span.max - span.min >= needed;
  };

  const step = Math.max(0.5, (bottom - top) / 200);
  let first = top;
  let last = bottom;
  while (first < bottom && !fits(first)) first += step;
  while (last > first && !fits(last)) last -= step;

  // Nothing anywhere holds a label. Only a compartment narrower than a single
  // name reaches this, and then the whole shape's own middle is the least bad
  // answer there is.
  const usable = last > first;
  const from = usable ? first : top;
  const to = usable ? last : bottom;
  const pitch = count > 1 ? (to - from) / (count - 1) : 0;

  return Array.from({ length: count }, (_, index) => {
    const y = count > 1 ? from + index * pitch : (from + to) / 2;
    const span = spanOver(points, y - above, y + below);
    const { min, max } = withoutCaption(
      span,
      y,
      caption,
      above,
      below,
      half,
      true,
    );
    const lowest = min + half + GRAHA_MARGIN / 2;
    const highest = max - half - GRAHA_MARGIN / 2;

    return {
      x:
        lowest <= highest
          ? Math.min(Math.max(centre.x, lowest), highest)
          : (min + max) / 2,
      y,
    };
  });
}

/** A graha label's box at a placed position, for the travel test.
 *
 * `half` defaults to the widest label there can be, which is what the packed
 * layout has to assume. The pathway passes each body's own. */
function labelBox(at: Point, scale: number, half = GRAHA_HALF): Box {
  const reach = half * scale;
  return {
    min: at.x - reach,
    max: at.x + reach,
    top: at.y - GRAHA_ABOVE * scale,
    bottom: at.y + GRAHA_BELOW * scale,
  };
}

/**
 * How far a compartment's contents may slide, and which way.
 *
 * The layout is computed first and asked afterwards how much room it left. That
 * ordering is the whole design: the drift takes only slack that already exists,
 * so every invariant the static layout holds is still held at every point of the
 * motion, and nothing has to be clipped or special-cased.
 *
 * A compartment with one graha in it has a lot of room. The eight-body case in
 * D2 has almost none, and moves almost not at all - which is right. A crowded
 * house has less space to move in.
 */
function travel(
  points: Point[],
  boxes: Box[],
  caption: Box,
  along: { x: number; y: number },
): { back: number; forward: number } {
  const fits = (distance: number) =>
    boxes.every((box) => {
      const dx = along.x * distance;
      const dy = along.y * distance;
      const moved = {
        min: box.min + dx,
        max: box.max + dx,
        top: box.top + dy,
        bottom: box.bottom + dy,
      };
      // Inside the compartment, and clear of the caption.
      //
      // The caption is not in `boxes` because it does not move - but a body that
      // moves can move *onto* it, which is what happened the first time this ran:
      // `(Sa)` slid onto its own compartment's name at the end of the run. A
      // thing that stays still is still in the way.
      return (
        inside(points, moved.min, moved.top) &&
        inside(points, moved.max, moved.top) &&
        inside(points, moved.min, moved.bottom) &&
        inside(points, moved.max, moved.bottom) &&
        !touches(moved, caption)
      );
    });

  // Nothing fits even where it is. A compartment too small for what stands in it
  // reaches this, and it does not move at all rather than moving badly.
  if (!fits(0)) return { back: 0, forward: 0 };

  // Bisect rather than step: the reachable distance is bounded by the chart, and
  // ten halvings of it resolve to well under a pixel.
  const reach = (sign: number) => {
    let low = 0;
    let high = WIDTH;
    for (let i = 0; i < 10; i++) {
      const mid = (low + high) / 2;
      if (fits(sign * mid)) low = mid;
      else high = mid;
    }
    return low;
  };

  return { back: reach(-1), forward: reach(1) };
}

/** How much area two boxes share. Nothing reads it but the pathway's last
 *  resort, which has to choose between arrangements that all overlap. */
function shared(a: Box, b: Box): number {
  const across = Math.min(a.max, b.max) - Math.max(a.min, b.min);
  const down = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
  return across > 0 && down > 0 ? across * down : 0;
}

/**
 * Whether two boxes touch at all.
 *
 * There used to be a tolerant version of this - a hair of horizontal overlap
 * and up to two units of vertical - and the tolerance was argued for one case:
 * label against label, where `GRAHA_ROW` is deliberately a hair tighter than
 * the box it carries, so adjacent rows overlap by about six tenths of a unit
 * and nothing is visible, because the ink of a two-letter capital reaches
 * nowhere near the ascender.
 *
 * **The caption never earned that.** It was getting it anyway, because the same
 * function tested both, and the consequence only showed once the wall triangle's
 * caption moved into the drift's path: the slide would run until it had four
 * tenths of a unit of the caption underneath it, call that clear, and stop
 * there. A graze rather than a collision, and an invariant that had been zero.
 *
 * An obstacle is cleared or it is not.
 */
function touches(a: Box, b: Box): boolean {
  return (
    Math.min(a.max, b.max) > Math.max(a.min, b.min) &&
    Math.min(a.bottom, b.bottom) > Math.max(a.top, b.top)
  );
}

/**
 * Whether a point is inside the compartment.
 *
 * Ray casting, counting crossings to the right. The polygons here are simple and
 * closed, which is the only case this has to be right for.
 */
function inside(points: Point[], x: number, y: number): boolean {
  let within = false;
  for (let i = 0, j = points.length - 1; i < points.length; j = i++) {
    const a = points[i]!;
    const b = points[j]!;
    const straddles = a.y > y !== b.y > y;
    if (!straddles) continue;
    const crossing = a.x + ((y - a.y) / (b.y - a.y)) * (b.x - a.x);
    if (x < crossing) within = !within;
  }
  return within;
}

/* ------------------------------------------------------------------ *
 * The pathway.
 *
 * `docs/design/traversal.md` is the design; this is the whole of the
 * implementation. It is a prototype: `Chakra` draws it only when asked, and
 * the panel does not ask.
 *
 * The short form. Houses stay whole sign, so a body's compartment is fixed for
 * as long as the lagna is in its own sign. What is continuous is the ring: over
 * one crossing of the rising sign, every compartment's contents travel from the
 * edge they arrived through to the edge they will leave by. A body's own
 * progress through its part-run - its degree, in D1 - sets where it stands
 * along that travel.
 *
 * Three things here were got wrong once and are worth naming, because each was
 * a class of mistake rather than a bug:
 *
 * 1. **Lanes were perpendicular to the local route, so they rotated with it.**
 *    Two bodies on different lanes could therefore swap places on screen while
 *    their order along the route never changed. Lanes are now a family of
 *    *parallel* lines - the route translated along one fixed direction - so the
 *    order along the route is the order on screen, always.
 * 2. **The layout mode was recomputed every frame.** It is a property of what
 *    is standing in a compartment, not of an instant, and re-deciding it as the
 *    lagna moved made compartments teleport between arrangements. It is now
 *    decided once, against the whole crossing, and held.
 * 3. **The split between the degree scale and the travel was taken from an
 *    identity rather than from the layout.** Both spans are one house long in
 *    the sky, so halves looked forced; but the halving is what the crowding
 *    then could not afford. It is a designed number now, and it is stated.
 * ------------------------------------------------------------------ */

/**
 * The route through one compartment, and the direction its parallel lines are
 * offset along.
 *
 * The route is a quadratic Bézier from the middle of the edge the sign arrives
 * through to the middle of the edge it leaves by, bowed through the
 * compartment's own cluster point. The bow is not decoration: a compartment's
 * two gates always share a vertex - the outer point of a kite, the quarter
 * point of either triangle - so the straight chord between them cuts that
 * corner and runs along the part of the shape with the least room in it.
 */
interface Pathway {
  at: Point[];
  /** Cumulative arclength at each sample. `run[0]` is 0. */
  run: number[];
  /** The unit direction of the gate-to-gate chord. Every route is built so
   *  that its progress along this never reverses, which is what makes order
   *  along the route the same thing as order on screen. */
  chord: Point;
  /** Unit normal to the chord: the one direction the parallel lines are
   *  offset along. Fixed for the whole compartment, deliberately - a lane that
   *  turned with the route would carry a body sideways past its neighbours. */
  across: Point;
}

/** Samples per route. About one per unit of length on the longest route, which
 *  is finer than the band's edges can be resolved to anyway. */
const PATH_SAMPLES = 96;

/** Parallel lines either side of the route. Seventeen of them; how many are
 *  usable at a given station is a property of the shape, and the development
 *  grid draws exactly that. */
const PATH_LANES = 8;

/**
 * How much of the band the degree scale gets. The rest is the travel.
 *
 * A body's station, as a fraction of the band measured from the entry:
 *
 *     f = (1 - d) * SHARE + p * (1 - SHARE)
 *
 * so `d = 1, p = 0` sits at the entry and `d = 0, p = 1` at the exit, whatever
 * the split. Both terms are one house long in the sky, which is an argument for
 * a half each and *only* that: it makes the drawn chart a uniform half-scale of
 * the true ring, and it spends half the compartment on a motion that moves a
 * pixel every ten seconds. The degree scale is the reading.
 *
 * Measured across a whole crossing. `worst gap` is the widest degree separation
 * between two labels that overlap - the number that says whether an overlap is
 * a conjunction or a mistake. The first table is the eight sweep charts, four
 * of which are crowded on purpose:
 *
 * | Share | Worst gap | Travel, median | D60 median |
 * |---|---|---|---|
 * | 0.80 | 3.1° | 12.3 | 8.1 |
 * | 0.60 | 14.4° | 27.0 | 17.1 |
 * | **0.40** | **14.4°** | **33.8** | **28.8** |
 * | 0.25 | 14.4° | 37.1 | 42.5 |
 *
 * That table made 0.80 look forced, and it was the wrong table: the worst gap
 * in it comes from the four compartments holding seven and eight bodies, which
 * overlap at *every* share, so the figure barely moves between 0.60 and 0.25
 * and drowns out what happens to a chart anybody actually reads.
 *
 * Split out, the ordinary charts answer it:
 *
 * | Share | D1 | D9 | D30 | D60 | overlapping pairs, per frame |
 * |---|---|---|---|---|---|
 * | 0.40 | 28.7 | 30.5 | 66.6 | 28.8 | **0** in all four |
 * | 0.25 | 37.1 | 38.1 | 30.0 | 42.5 | D9 0.8, D60 0.3 at 5.1° |
 *
 * **0.40 is the knee.** Every chart that is not deliberately crowded draws with
 * no overlapping labels at all, and the travel is three and a half times what
 * 0.80 allowed: in D60 a body crosses 28.8 units in the two minutes the lagna
 * takes, which is a pixel every four seconds against a pixel every fifteen.
 * Below 0.40 overlaps start appearing in ordinary charts and buy little.
 *
 * The degree scale that remains is not the reading it was at 0.80, and that is
 * the trade taken deliberately: a tenth of a degree is not a distance a reader
 * can measure off a 318 point chart, while a body that never visibly moves is a
 * chart that does not look like a reading of now.
 */
const PATH_DEGREE_SHARE = 0.4;
const PATH_TRAVEL_SHARE = 1 - PATH_DEGREE_SHARE;

/** Progress samples a plan is tested at. The stations sweep rigidly, so
 *  containment is checked exactly - over every sample of the route the sweep
 *  covers - and only label-against-label needs sampling in time. */
const PATH_STEPS = 9;

/** How far along the degree axis a body may be pushed, in degrees, and only
 *  after every type size has been tried without pushing it at all.
 *
 * The degree is the one thing on this axis that means anything, so it is not
 * negotiable and it does not shrink: a conjunction stacks *across* the route,
 * and two bodies three degrees apart stay three degrees apart. This is the
 * stated exception, and it is in degrees rather than pixels so that the size of
 * the lie is the same figure a reader would be misled by. Three degrees is a
 * tenth of a sign. It is a constant per body for the whole crossing, so it
 * costs fidelity and never costs continuity.
 */
const PATH_SPILL_DEGREES = 3;

/** Clearance a label on a line keeps from the compartment's walls. Smaller
 *  than `GRAHA_MARGIN`, because the route already avoids the corners and the
 *  band is measured rather than assumed. */
const PATH_CLEARANCE = 2;

/**
 * How far a route may bow past its compartment's cluster point, as a multiple
 * of the distance from the straight gate-to-gate chord to that point.
 *
 * Measured rather than declared, per compartment, because the three shapes want
 * different answers and the caption moves the answer again. A candidate is
 * rejected outright unless its progress along the chord is monotonic; the rest
 * are scored by the length of the band they leave.
 */
const PATH_BOWS = [0, 0.5, 1, 1.5, 2, 2.5, 3];

/** How much band a route may give up to be flatter.
 *
 * Curvature is what makes a pair of bodies shear - their separation vector
 * follows the tangent - and it is also what makes the band exist at all: both
 * gates lie on edges meeting at one vertex, so a straight route runs through
 * the tightest part of the shape. Measured, with every route forced straight:
 * shear falls to zero and a kite's band falls from about 125 units to about 25,
 * which is the degree scale from 3.3 units per degree to 0.7. The bow is not a
 * luxury; it is the reading.
 *
 * So only the curvature that buys nothing is given up. Measured over the twelve
 * compartments, a tenth of tolerance takes the median pair's shear from 9.2
 * degrees to 5.7 - and it is not paid for out of the reading: it puts one more
 * compartment on `exact` and takes one off `stacked`, because the flatter route
 * in a wall triangle is also the one whose parallel lines survive furthest. A
 * twentieth was tried first and recovers none of it once the caption is cleared
 * strictly rather than grazed. */
const PATH_BAND_TOLERANCE = 0.9;

/** Samples used while comparing candidate routes. Half the real sampling, and
 *  not less: at a quarter of it the ranking came apart on the wall triangles,
 *  and houses 3 and 5 - the same triangle translated down the same wall - were
 *  given routes with bands of 75 and 31 units. */
const PATH_PROBE = 48;

/** The type sizes a compartment steps down through, largest first. The packed
 *  layout's own ladder (D-033): 11px, then 86, 74 and 62 per cent of it. */
/** The type sizes a compartment may use. One.
 *
 * A graha's name is two letters at 11px in a 318 point panel, and 11px is
 * already the smallest this app sets anything. Shrinking it made a label that
 * could be read into one that had to be decoded - and worse, it did so
 * *per compartment*, so a chart showed two or three sizes at once and the
 * difference read as meaning something. It meant only that one house was
 * busier than another.
 *
 * What this costs is overlap. A compartment that cannot hold its bodies at full
 * size now draws them overlapping instead of drawing them small. Measured over
 * the harness's thirty chart cases: every chart that is not deliberately
 * crowded draws with no overlapping labels at all, and the ones that do overlap
 * are the seven- and eight-body compartments - which is D2 every day, where the
 * bodies genuinely are on top of each other in the sky.
 *
 * The ladder is kept as a list of one rather than deleted, because the search
 * that reads it is the same search either way and a future decision to trade
 * legibility for density is then one constant. */
const PATH_SIZES = [1];

function midpoint(a: Point, b: Point): Point {
  return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
}

/** The middle of the edge two compartments share, or nothing if they touch at
 *  a point or not at all.
 *
 * By coordinate rather than by naming the edges: the twelve shapes are built
 * from nine shared vertices, so which edge two houses have in common is
 * already written down in the construction and does not need saying twice. */
function gate(a: Point[], b: Point[]): Point | undefined {
  const shared = a.filter((p) => b.some((q) => q.x === p.x && q.y === p.y));
  return shared.length === 2 ? midpoint(shared[0]!, shared[1]!) : undefined;
}

/** The control point of the quadratic whose midpoint is `via`. */
function control(entry: Point, via: Point, exit: Point): Point {
  return {
    x: 2 * via.x - (entry.x + exit.x) / 2,
    y: 2 * via.y - (entry.y + exit.y) / 2,
  };
}

/**
 * Whether a candidate route's progress along its own chord ever reverses.
 *
 * This is the whole guarantee against bodies swapping places. A quadratic's
 * speed along the chord is a linear blend of `(C - E) . u` and `(X - C) . u`,
 * so it keeps its sign exactly when both do. A route that fails this doubles
 * back in screen space, and two bodies at fixed stations on it exchange
 * positions as the group slides past the turn - which is what a reader sees as
 * one graha jumping across another.
 */
function forwardOnly(entry: Point, via: Point, exit: Point, chord: Point) {
  const middle = control(entry, via, exit);
  const first = (middle.x - entry.x) * chord.x + (middle.y - entry.y) * chord.y;
  const last = (exit.x - middle.x) * chord.x + (exit.y - middle.y) * chord.y;
  return first > 0 && last > 0;
}

/**
 * The best route between two gates.
 *
 * Candidates that double back are rejected before anything else is asked of
 * them; among the rest the winner is the one with the longest unbroken band at
 * full size. Length rather than fraction: the band is what the thirty degrees
 * are laid across, so a long band in a long route beats a short one that
 * happens to be all of a short route.
 *
 * Scored at full size only. Scoring it across the whole type ladder was tried
 * and is worse: the small sizes fit almost anywhere, so summing over them let a
 * route generous at 62% win over one generous at 11px, and the top kite's band
 * fell from 111 units to 49.
 */
function bestRoute(
  points: Point[],
  entry: Point,
  exit: Point,
  towards: Point,
  caption: Box,
  half: number,
): Pathway {
  const middle = midpoint(entry, exit);
  const length = Math.hypot(exit.x - entry.x, exit.y - entry.y) || 1;
  const chord = {
    x: (exit.x - entry.x) / length,
    y: (exit.y - entry.y) / length,
  };

  const viaAt = (bow: number) => ({
    x: middle.x + (towards.x - middle.x) * bow,
    y: middle.y + (towards.y - middle.y) * bow,
  });

  const candidates: { bow: number; band: number; turn: number }[] = [];
  for (const bow of PATH_BOWS) {
    const via = viaAt(bow);
    if (!forwardOnly(entry, via, exit, chord)) continue;
    const probe = route(entry, via, exit, PATH_PROBE, chord);
    const band = reach(points, probe, caption, 1, half);
    if (!band) continue;
    const total = probe.run[probe.run.length - 1]!;
    candidates.push({
      bow,
      band: (band[1] - band[0]) * total,
      turn: turning(probe),
    });
  }
  if (candidates.length === 0) {
    return route(entry, viaAt(0), exit, PATH_SAMPLES, chord);
  }

  // Longest band first, then flattest.
  //
  // Curvature is not free. Two bodies at fixed stations keep a fixed distance -
  // they are rigid in the sky - but the *direction* between them follows the
  // route's tangent, so as the group slides along a bend their separation
  // vector turns. Nothing can remove that while the route bends; what can be
  // removed is the part of it that was bought for nothing, and a route within a
  // twentieth of the best band is bought for nothing.
  //
  // `PATH_BAND_TOLERANCE` is how much band that is worth.
  const widest = Math.max(...candidates.map((c) => c.band));
  const flattest = candidates
    .filter((c) => c.band >= widest * PATH_BAND_TOLERANCE)
    .reduce((a, b) => (b.turn < a.turn ? b : a));

  return route(entry, viaAt(flattest.bow), exit, PATH_SAMPLES, chord);
}

/** How far a route's direction turns, end to end, in radians.
 *
 * The measure of how much a pair of bodies will shear as the group slides along
 * it: their separation vector follows the tangent, so the turning of the
 * tangent over the stretch they travel is the angle they will appear to rotate
 * through. */
function turning(path: Pathway): number {
  let total = 0;
  for (let i = 1; i < path.at.length; i++) {
    const a = heading(path, i - 1);
    const b = heading(path, i);
    total += Math.abs(Math.atan2(a.x * b.y - a.y * b.x, a.x * b.x + a.y * b.y));
  }
  return total;
}

function route(
  entry: Point,
  via: Point,
  exit: Point,
  samples: number,
  chord: Point,
): Pathway {
  const middle = control(entry, via, exit);

  const at: Point[] = [];
  const run: number[] = [];
  let length = 0;
  for (let i = 0; i <= samples; i++) {
    const t = i / samples;
    const u = 1 - t;
    const point = {
      x: u * u * entry.x + 2 * u * t * middle.x + t * t * exit.x,
      y: u * u * entry.y + 2 * u * t * middle.y + t * t * exit.y,
    };
    if (i > 0) {
      const last = at[i - 1]!;
      length += Math.hypot(point.x - last.x, point.y - last.y);
    }
    at.push(point);
    run.push(length);
  }

  return { at, run, chord, across: { x: -chord.y, y: chord.x } };
}

/**
 * The sample nearest a fraction of the route's *length*.
 *
 * By arclength rather than by the curve's own parameter. A quadratic moves
 * faster near its control point, so a degree scale laid out in `t` would be a
 * degree scale whose divisions are not equal - which is exactly the bunching
 * the shape of a triangle already threatens.
 *
 * The answer is fractional. Snapping to the nearest sample quantises the degree
 * to about half a degree on a kite's band, and the caption beside the chart
 * prints arcminutes.
 */
function atFraction(path: Pathway, fraction: number): number {
  const total = path.run[path.run.length - 1]!;
  const wanted = Math.min(Math.max(fraction, 0), 1) * total;
  let low = 0;
  let high = path.run.length - 1;
  while (low < high) {
    const mid = (low + high) >> 1;
    if (path.run[mid]! < wanted) low = mid + 1;
    else high = mid;
  }

  if (low === 0) return 0;
  const before = path.run[low - 1]!;
  const step = path.run[low]! - before;
  return step > 0 ? low - 1 + (wanted - before) / step : low;
}

/** The point at a fractional sample. */
function alongRoute(path: Pathway, index: number): Point {
  const last = path.at.length - 1;
  const clamped = Math.min(Math.max(index, 0), last);
  const low = Math.floor(clamped);
  const high = Math.min(low + 1, last);
  const t = clamped - low;
  const a = path.at[low]!;
  const b = path.at[high]!;
  return { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t };
}

/** The route's direction at a sample, for the handover's arrival vector. */
function heading(path: Pathway, index: number): Point {
  const a = path.at[Math.max(0, index - 1)]!;
  const b = path.at[Math.min(path.at.length - 1, index + 1)]!;
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const length = Math.hypot(dx, dy) || 1;
  return { x: dx / length, y: dy / length };
}

/**
 * How far apart two parallel lines have to be before labels on them miss.
 *
 * The boxes are axis-aligned and the family's direction is not, so the answer
 * depends on that direction: lines separated vertically need the box's height,
 * horizontally its width, and a diagonal whichever it reaches first. One number
 * for the whole compartment, because the direction is fixed.
 */
function lanePitch(across: Point, scale: number, half: number): number {
  const sideways =
    Math.abs(across.x) > 1e-6
      ? (2 * half * scale) / Math.abs(across.x)
      : Infinity;
  const down =
    Math.abs(across.y) > 1e-6
      ? ((GRAHA_ABOVE + GRAHA_BELOW) * scale) / Math.abs(across.y)
      : Infinity;
  return Math.min(sideways, down);
}

/** Where a station on one of the parallel lines is. */
function laneAt(
  path: Pathway,
  index: number,
  lane: number,
  pitch: number,
): Point {
  const at = alongRoute(path, index);
  if (lane === 0) return at;
  return {
    x: at.x + path.across.x * pitch * lane,
    y: at.y + path.across.y * pitch * lane,
  };
}

/** Whether a label's box is wholly inside the compartment and off its caption. */
function standable(points: Point[], caption: Box, box: Box): boolean {
  const c = PATH_CLEARANCE;
  return (
    inside(points, box.min - c, box.top - c) &&
    inside(points, box.max + c, box.top - c) &&
    inside(points, box.min - c, box.bottom + c) &&
    inside(points, box.max + c, box.bottom + c) &&
    !touches(box, caption)
  );
}

/**
 * The stretch of the route a label of this size can stand on, on the route
 * itself.
 *
 * The gates are on the compartment's boundary, so a label centred on one is
 * half outside it by construction: the band is always shorter than the route.
 *
 * **Measured on the base line, not across the lanes.** It was measured across
 * them first and that was wrong in a way worth recording: a station counted as
 * usable if a label could stand anywhere across from it, so a route hugging a
 * triangle's apex scored a long band that could only be reached by leaning
 * every label off it. A wall triangle's single graha then travelled five units
 * across a whole crossing instead of thirty-seven. The route is where the
 * degrees are; the lanes are for what has to stack on them.
 *
 * The band is the longest *unbroken* run, so a caption that splits the route
 * truncates the scale to the longer piece rather than holing it. Every degree
 * still has a station; the scale is compressed.
 */
function reach(
  points: Point[],
  path: Pathway,
  caption: Box,
  scale: number,
  half: number,
): [number, number] | null {
  const usable = path.at.map((at) =>
    standable(points, caption, labelBox(at, scale, half)),
  );

  let bestFrom = -1;
  let bestTo = -1;
  let from = -1;
  for (let i = 0; i <= usable.length; i++) {
    if (i < usable.length && usable[i]) {
      if (from < 0) from = i;
      continue;
    }
    if (from >= 0) {
      if (i - from > bestTo - bestFrom) {
        bestFrom = from;
        bestTo = i - 1;
      }
      from = -1;
    }
  }
  if (bestFrom < 0) return null;

  const total = path.run[path.run.length - 1]!;
  if (total <= 0) return null;
  return [path.run[bestFrom]! / total, path.run[bestTo]! / total];
}

/**
 * A compartment's arrangement, decided once and held for the whole crossing.
 *
 * **This is a plan, not a position.** Which type size, which parallel line each
 * body stands on, and how much of the degree axis was spent are properties of
 * what is standing in the compartment; the lagna's progress is not one of them.
 * Deciding them per frame is what made compartments teleport: the mode flipped
 * between `stacked` and `packed` as the group slid, and `packed` has no
 * progress term at all, so a compartment would freeze for thirty steps and then
 * jump sixty units. A plan is computed from the bodies alone and evaluated
 * against every instant of the crossing before it is accepted.
 */
interface Plan {
  scale: number;
  band: [number, number];
  pitch: number;
  /** Each body's station at `p = 0`, as a fraction of the band. Constant. */
  station: number[];
  /** Which parallel line each body stands on. Constant. */
  lane: number[];
  how: "exact" | "spilled" | "stacked" | "packed";
  /** The most any one label is covered, as a fraction of its own box, at the
   *  worst instant. Written onto the group as a data attribute when the pathway
   *  is on: it is the number the last two rungs are chosen between, and reading
   *  it back out of rendered positions is guesswork. */
  cover?: number;
  /** The packed last resort: positions that do not follow the route, plus the
   *  slide the shipped layout gives them, so that nothing is ever frozen. */
  packed?: { at: Point[]; back: number; forward: number };
}

/** Where the plan puts the bodies at one instant. */
function place(
  plan: Plan,
  path: Pathway,
  along: { x: number; y: number } | undefined,
  progress: number,
): Point[] {
  if (plan.packed) {
    const room = plan.packed;
    const at = -room.back + (room.back + room.forward) * progress;
    const dx = along ? along.x * at : 0;
    const dy = along ? along.y * at : 0;
    return room.at.map((point) => ({ x: point.x + dx, y: point.y + dy }));
  }

  const [from, to] = plan.band;
  return plan.station.map((station, index) => {
    const fraction = station + progress * PATH_TRAVEL_SHARE;
    return laneAt(
      path,
      atFraction(path, from + (to - from) * fraction),
      plan.lane[index]!,
      plan.pitch,
    );
  });
}

/**
 * The plan for one compartment.
 *
 * The compartment has two axes and they are not interchangeable. Along the
 * route is the degree, which is the reading; across it is nothing, which is
 * where a conjunction goes. So a body's station is fixed by its own progress
 * and the packing gets the parallel lines and only that.
 *
 * Four rungs, in the order things are given up. Overlapping type is given up
 * after the degree and not before, because the degree is the reading: a chart
 * that has moved a body to keep its type clean is saying something untrue
 * quietly, and overlapping type is at least visibly hard to read. The packed
 * layout makes the same call for the same reason (D-033).
 */
function planFor(
  points: Point[],
  path: Pathway,
  caption: Box,
  bodies: { progress: number; half: number }[],
  centre: Point,
  along: { x: number; y: number } | undefined,
): Plan {
  // Two widths, and they answer different questions. The **band** is the
  // degree scale's own extent - a ruler, not a slot - so it is measured with
  // the narrowest label standing here: a wide label that cannot stand at one
  // end of the ruler takes a lane where it can, and only that body is
  // constrained by its own width. The **pitch** between lanes has to clear the
  // widest, or a stack would not be evenly spaced.
  //
  // Measuring both with the widest is what put a routine D1 chart's wall
  // triangle into the packed fallback: one retrograde bracket, twenty-five
  // units wide instead of sixteen, halved the compartment's whole scale for the
  // benefit of one of the two bodies in it.
  const widest = Math.max(...bodies.map((body) => body.half), GRAHA_HALF_PLAIN);
  const narrowest = Math.min(...bodies.map((body) => body.half), GRAHA_HALF);

  // Type size outside, spill inside, and the order is the whole point. The
  // other way round exhausted every size at zero spill before trying the three
  // degrees of spill this design already declares, so a compartment needing one
  // degree of it was set at 86% instead - and a routine chart came out with two
  // type sizes in it, which reads as a mistake rather than as a measurement.
  //
  // The type size is the thing a reader compares across houses, so it is the
  // last thing to give way. The spill is already bounded and already disclosed;
  // spending it first is spending the cheaper resource.
  for (const scale of PATH_SIZES) {
    for (const [spill, strict] of [
      [0, true],
      [PATH_SPILL_DEGREES, true],
    ] as const) {
      const band = reach(points, path, caption, scale, narrowest);
      if (!band) continue;
      const laid = assign(
        points,
        path,
        caption,
        bodies,
        scale,
        widest,
        band,
        spill,
        strict,
      );
      if (laid) {
        return { ...laid, scale, band, how: spill === 0 ? "exact" : "spilled" };
      }
    }
  }

  // The third rung is not first-that-fits, because every size succeeds once
  // labels may overlap: the question is no longer whether an arrangement exists
  // but which one costs the least ink.
  let best: Plan | null = null;
  let least = Infinity;
  for (const scale of PATH_SIZES) {
    const band = reach(points, path, caption, scale, narrowest);
    if (!band) continue;
    const laid = assign(
      points,
      path,
      caption,
      bodies,
      scale,
      widest,
      band,
      PATH_SPILL_DEGREES,
      false,
    );
    if (!laid) continue;
    const plan: Plan = { ...laid, scale, band, how: "stacked" };
    const cost = worstCover(plan, path, bodies);
    plan.cover = cost;

    if (cost < least) {
      least = cost;
      best = plan;
    }
  }

  // The degrees are kept unless they have become unreadable. Holding them is
  // the point of all of this, so the packed rows take over only where a label
  // has lost more than half of itself - at which point the compartment is not
  // *showing* degrees by stacking them anyway. Five labels three units apart do
  // not say "sixteen degrees"; they say nothing, and they say it illegibly.
  // The packed rung, with the one thing the packed layout cannot know: each
  // label's own width. `travel` is the shipped slide and it clamps every label
  // by the widest there can be, because `cluster` never sees the text. Here the
  // text is in hand, and eight labels in nine are eight units narrower than
  // that - which in a wall triangle is the difference between a group that
  // slides and a group that cannot move at all.
  // The shipped packer, at whichever of its own type sizes can still move.
  //
  // Two mistakes were made here in turn and both are worth keeping. It first
  // called `squeezed` directly, which is not the shipped layout but the shipped
  // layout's *last resort*: `squeezed` spreads a single column across every
  // height that can hold a label, so it fills the compartment by construction.
  // Then it called `cluster`, which is the shipped layout - and `cluster` picks
  // the largest type that fits, which also fills the compartment. Either way a
  // full compartment has no slack and the group stood still for a whole
  // crossing.
  //
  // So the candidates are the shipped packer's arrangements at all four of its
  // sizes, plus its own last resort, and the choice between them is made here
  // rather than by taking the first that fits. Legibility first - a size that
  // covers less ink wins - and **among sizes that are equally legible, the one
  // that leaves the group room to move**. That is not a new packing rule; it is
  // the same arrangements, chosen inside a design whose subject is the
  // traversal. `cluster` and the panel are untouched.
  const options: { at: Point[]; scale: number }[] = [];
  for (const scale of PATH_SIZES) {
    const rows = arrange(points, centre, bodies.length, caption, scale);
    if (rows) options.push({ at: rows, scale });
  }
  options.push({
    at: squeezed(points, centre, bodies.length, caption, 1),
    scale: 1,
  });

  const weighed = options.map((option) => {
    const room = along
      ? travel(
          points,
          option.at.map((point, index) =>
            labelBox(point, option.scale, bodies[index]!.half),
          ),
          caption,
          along,
        )
      : { back: 0, forward: 0 };
    const plan: Plan = {
      scale: option.scale,
      band: [0, 1],
      pitch: 0,
      station: [],
      lane: [],
      how: "packed",
      packed: { at: option.at, back: room.back, forward: room.forward },
    };
    // How many labels this arrangement puts on the compartment's caption.
    // `squeezed` is allowed to do that - it is the last resort and a body
    // outside its own sign would be worse - but it is only *allowed* to, and
    // here there are other candidates to compare it against.
    const onCaption = option.at.filter((point, index) =>
      touches(labelBox(point, option.scale, bodies[index]!.half), caption),
    ).length;

    return {
      plan,
      room: room.back + room.forward,
      cover: worstCover(plan, path, bodies),
      onCaption,
    };
  });

  // Off the caption first, then least covered, then whichever of those can
  // still move. The order is the order of the invariants: a label on the
  // caption is a broken one, overlapping labels are a degraded reading, and
  // standing still is only a lost opportunity.
  const clear = Math.min(...weighed.map((w) => w.onCaption));
  const usable = weighed.filter((w) => w.onCaption === clear);
  const cleanest = Math.min(...usable.map((w) => w.cover));
  const packed = usable
    .filter((w) => w.cover <= cleanest + PATH_SAME_COVER)
    .reduce((a, b) => (b.room > a.room ? b : a)).plan;
  packed.cover = worstCover(packed, path, bodies);

  // Three lines, and the middle one is the one that was missing.
  //
  // If the degree-true arrangement is readable on its own, it wins: that is
  // what all of this is for. If it is not, the two are compared **against each
  // other** rather than the first being measured against a threshold and
  // discarded - because a compartment that cannot hold seven bodies cannot hold
  // them in rows either, and giving up the degrees buys nothing there. The
  // seven-in-a-wall-triangle case is exactly that: the stacked arrangement
  // buries a label, and so does the packed one, and only the stacked one still
  // says where anything is or moves at all.
  if (!best) return packed;
  if (least <= PATH_LEGIBLE) return best;
  return packed.cover < least ? packed : best;
}

/**
 * The most any one label is covered by another, at the worst instant of the
 * crossing, as a fraction of its own box.
 *
 * Not total overlapping area. Total area was tried as the measure that chose
 * between the degree-true arrangement and the packed one, and it is the wrong
 * question: `squeezed` spreads a single column down a tall compartment and
 * frequently overlaps *nothing*, so a comparison on total ink discarded the
 * degrees of eight bodies to save one unit of overlap. What matters is whether
 * a label can still be read, and that is a property of the worst-covered label
 * rather than of the sum.
 *
 * At the worst instant rather than at one of them: an arrangement that is clean
 * at both ends of a crossing can be a pile in the middle.
 */
function worstCover(
  plan: Plan,
  path: Pathway,
  bodies: { half: number }[],
): number {
  let worst = 0;
  for (let step = 0; step < PATH_STEPS; step++) {
    const at = place(plan, path, undefined, step / (PATH_STEPS - 1));
    const boxes = at.map((point, index) =>
      labelBox(point, plan.scale, bodies[index]!.half),
    );
    for (let i = 0; i < boxes.length; i++) {
      const own = boxes[i]!;
      const area = (own.max - own.min) * (own.bottom - own.top);
      if (area <= 0) continue;
      let covered = 0;
      for (let j = 0; j < boxes.length; j++) {
        if (i !== j) covered += shared(own, boxes[j]!);
      }
      worst = Math.max(worst, covered / area);
    }
  }
  return worst;
}

/**
 * How much of a label may be covered before its degree is not worth keeping.
 *
 * Half. A two-letter label with more than half its box under another label is
 * not read as a body standing at a degree; it is read as a smudge, and the
 * degree it was carrying is lost either way. Below that the reader still has
 * both the name and the position, which is the whole of what the pathway is
 * for.
 *
 * This is a threshold and thresholds are worth being uncomfortable about. It is
 * here because the alternative - comparing the two last resorts on overlapping
 * area - answers a different question from the one being asked, and answered it
 * wrongly in exactly the case that matters.
 */
const PATH_LEGIBLE = 0.5;

/** How close two packed arrangements have to be on covered ink before the one
 *  that can still move is preferred. Five per cent of one label's box: below
 *  that the difference is a rounding of where a row landed, not something a
 *  reader could see. */
const PATH_SAME_COVER = 0.05;

/**
 * One attempt: every body on its own station, on a parallel line that holds it
 * for the whole crossing.
 *
 * Containment is exact rather than sampled. A body's station sweeps a fixed
 * interval of the band as the lagna crosses its sign, and the sweep is rigid,
 * so the question "does this line hold this label everywhere it goes" is asked
 * of every sample of the route the sweep covers. Only label-against-label needs
 * sampling in time, because two bodies on different lines do move relative to
 * one another - the route is curved, and translating along a curve is not a
 * rigid motion of the pair.
 */
function assign(
  points: Point[],
  path: Pathway,
  caption: Box,
  bodies: { progress: number; half: number }[],
  scale: number,
  widest: number,
  band: [number, number],
  spill: number,
  strict: boolean,
): { station: number[]; lane: number[]; pitch: number } | null {
  const count = bodies.length;
  const pitch = lanePitch(path.across, scale, widest);

  // The station at p = 0. A body at the start of its part-run stands nearest
  // the exit - the lagna reaches its cusp first - and the crossing carries the
  // whole group there.
  const base = bodies.map(
    (body) =>
      (1 - Math.min(Math.max(body.progress, 0), 1)) * PATH_DEGREE_SHARE,
  );
  const order = base.map((_, index) => index).sort((a, b) => base[a]! - base[b]!);

  // Where each lane can hold a label, as a prefix count, so "is every sample
  // between here and there usable" is one subtraction.
  const runs = new Map<string, number[]>();
  const usableOn = (lane: number, half: number): number[] => {
    const key = `${lane}:${half}`;
    const found = runs.get(key);
    if (found) return found;
    const counts: number[] = [0];
    for (let i = 0; i < path.at.length; i++) {
      const ok = standable(
        points,
        caption,
        labelBox(laneAt(path, i, lane, pitch), scale, half),
      );
      counts.push(counts[i]! + (ok ? 1 : 0));
    }
    runs.set(key, counts);
    return counts;
  };

  const spillFraction = (spill / 30) * PATH_DEGREE_SHARE;
  const nudges = spill === 0 ? [0] : [0, spillFraction, -spillFraction];

  const station = new Array<number>(count);
  const lane = new Array<number>(count);
  const placed: number[] = [];

  for (const body of order) {
    let chosen: { station: number; lane: number; cost: number } | null = null;

    for (const nudge of nudges) {
      const start = Math.min(Math.max(base[body]! + nudge, 0), PATH_DEGREE_SHARE);
      const from = atFraction(path, band[0] + (band[1] - band[0]) * start);
      const to = atFraction(
        path,
        band[0] + (band[1] - band[0]) * (start + PATH_TRAVEL_SHARE),
      );
      const first = Math.floor(Math.min(from, to));
      const last = Math.ceil(Math.max(from, to));

      for (let step = 0; step <= PATH_LANES; step++) {
        for (const side of step === 0 ? [1] : [1, -1]) {
          const line = step * side;
          const counts = usableOn(line, bodies[body]!.half);
          // Every sample the sweep touches must hold the label.
          if (counts[last + 1]! - counts[first]! < last - first + 1) continue;

          const cost = clash(
            path,
            band,
            { station: start, lane: line },
            placed.map((other) => ({
              station: station[other]!,
              lane: lane[other]!,
            })),
            pitch,
            scale,
            bodies,
            body,
            placed,
          );
          if (strict && cost > 0) continue;
          if (!chosen || cost < chosen.cost) {
            chosen = { station: start, lane: line, cost };
          }
          if (cost === 0) break;
        }
        if (chosen && chosen.cost === 0) break;
      }
      if (chosen && chosen.cost === 0) break;
    }

    if (!chosen) return null;
    station[body] = chosen.station;
    lane[body] = chosen.lane;
    placed.push(body);
  }

  return { station, lane, pitch };
}

/** How much ink one candidate would put on the bodies already placed, taken
 *  over the whole crossing rather than at one instant. */
function clash(
  path: Pathway,
  band: [number, number],
  candidate: { station: number; lane: number },
  others: { station: number; lane: number }[],
  pitch: number,
  scale: number,
  bodies: { half: number }[],
  body: number,
  placed: number[],
): number {
  if (others.length === 0) return 0;
  let worst = 0;
  for (let step = 0; step < PATH_STEPS; step++) {
    const progress = step / (PATH_STEPS - 1);
    const spot = (entry: { station: number; lane: number }) => {
      const fraction = entry.station + progress * PATH_TRAVEL_SHARE;
      return laneAt(
        path,
        atFraction(path, band[0] + (band[1] - band[0]) * fraction),
        entry.lane,
        pitch,
      );
    };
    const mine = labelBox(spot(candidate), scale, bodies[body]!.half);
    let total = 0;
    for (let i = 0; i < others.length; i++) {
      total += shared(
        mine,
        labelBox(spot(others[i]!), scale, bodies[placed[i]!]!.half),
      );
    }
    worst = Math.max(worst, total);
  }
  return worst;
}

/**
 * A row's available width, with the compartment's caption taken out of it.
 *
 * Only where the row runs level with the caption. The caption sits against a
 * wall, so what is left is one interval rather than two: the side of it with
 * more space in.
 */
function withoutCaption(
  span: { min: number; max: number },
  y: number,
  caption: Box,
  above: number,
  below: number,
  half: number,
  /** Whether to hand the caption's own room back when what is left beside it
   *  could not hold a label anyway.
   *
   * True only for the last resort. `rows` can decline a row and let `arrange`
   * try another shape or another size; `squeezed` has nowhere left to go, and
   * a body sitting on the sign's name is better than a body outside its own
   * sign. Handing it back from `rows` too is how a name caption came to have
   * two grahas on it: the row was placeable *somewhere*, so nothing rejected
   * it, and the sliver test quietly withdrew the obstacle. */
  yielding: boolean,
): { min: number; max: number } {
  const level = y + below > caption.top && y - above < caption.bottom;
  if (!level) return span;

  const toTheLeft = caption.min - span.min;
  const toTheRight = span.max - caption.max;
  const kept =
    toTheRight >= toTheLeft
      ? { min: Math.max(span.min, caption.max), max: span.max }
      : { min: span.min, max: Math.min(span.max, caption.min) };

  // Unless what is left could not hold a label anyway, in which case the caption
  // is not taken out at all. Keeping the sliver would push the label off the
  // compartment entirely, and a body drawn outside its own sign is wrong in a
  // way that a body sitting on the sign's number is not.
  //
  // A house *number* is what reaches this. A name sits against a wall so one
  // side of it is always the whole compartment; a number is placed near the
  // chart's middle, where the compartment is widest, so it can leave two narrow
  // sides and no wide one.
  const needed = half * 2 + GRAHA_MARGIN;
  return !yielding || kept.max - kept.min >= needed ? kept : span;
}

/** One type size's dimensions, so an arrangement can be tried at several. */
interface Metrics {
  pitch: number;
  above: number;
  below: number;
  half: number;
  column: number;
}

/**
 * One arrangement, or nothing if it does not fit.
 *
 * A row is moved as a whole rather than each label being clamped on its own.
 * Clamping them separately was tried and is wrong twice over: it silently closes
 * the gap the pitch exists to hold - two labels pushed off opposite walls meet
 * in the middle and overlap - and it hides the real problem, which is that the
 * row is too wide for the compartment at that height. Moving the row keeps the
 * spacing and lets the row that genuinely will not fit say so.
 */
function rows(
  points: Point[],
  centreX: number,
  middleY: number,
  count: number,
  columns: number,
  caption: Box,
  size: Metrics,
): Point[] | null {
  const { pitch, above, below, half, column: columnWidth } = size;
  const lines = Math.ceil(count / columns);
  const placed: Point[] = [];

  for (let index = 0; index < count; index++) {
    const line = Math.floor(index / columns);
    const column = index % columns;
    const inThisRow = Math.min(columns, count - line * columns);

    const y = middleY + (line - (lines - 1) / 2) * pitch;

    // The narrowest the compartment gets over the label's own box, not over the
    // scanline its baseline sits on. Where a compartment has a diagonal wall the
    // top of that box is the tight point, so a baseline that measured as inside
    // still put the top corner through the wall.
    // Vertical fit, checked explicitly. `spanOver` catches it for a kite or a
    // triangle, where running past the shape means running into a converging
    // wall - but a South Indian compartment is a rectangle, whose span is the
    // same at every height, so a block hanging out of the bottom measured as
    // fitting perfectly at every row.
    const ys = points.map((point) => point.y);
    if (
      y - above < Math.min(...ys) + GRAHA_MARGIN / 2 ||
      y + below > Math.max(...ys) - GRAHA_MARGIN / 2
    ) {
      return null;
    }

    const span = spanOver(points, y - above, y + below);
    const { min, max } = withoutCaption(
      span,
      y,
      caption,
      above,
      below,
      half,
      false,
    );

    const reach = ((inThisRow - 1) * columnWidth) / 2 + half;
    const lowest = min + reach + GRAHA_MARGIN / 2;
    const highest = max - reach - GRAHA_MARGIN / 2;
    if (lowest > highest) return null;

    const middle = Math.min(Math.max(centreX, lowest), highest);
    placed.push({
      x: middle + (column - (inThisRow - 1) / 2) * columnWidth,
      y,
    });
  }

  return placed;
}

/**
 * The North Indian compartments, in house order.
 *
 * The construction is the whole specification: the outer square, both diagonals,
 * and the diamond joining the midpoints of the four sides. Each diamond edge
 * runs parallel to one diagonal and crosses the other at its midpoint, which
 * puts four vertices at the quarter points. That leaves four kites around the
 * centre and eight triangles in the corners.
 *
 * House 1 is the top kite and the count runs anticlockwise, which is the
 * published order (`docs/design/kundali.md` §3.1). Houses are fixed and the
 * signs move through them, so house 1 holds whatever the lagna is in.
 */
function northCompartments(
  rashis: ChakraRashi[],
  lagnaSign: number,
  numbered: boolean,
): Compartment[] {
  // The construction is the same whatever the proportions: the corners, the
  // midpoints of the four sides, and the quarter points where the diamond's
  // edges cross the diagonals. Width and height are simply carried separately.
  const left = INSET;
  const top = INSET;
  const right = WIDTH - INSET;
  const bottom = HEIGHT - INSET;
  const midX = (left + right) / 2;
  const midY = (top + bottom) / 2;
  const quarterX = left + (right - left) / 4;
  const threeX = left + (3 * (right - left)) / 4;
  const quarterY = top + (bottom - top) / 4;
  const threeY = top + (3 * (bottom - top)) / 4;

  const N = { x: midX, y: top };
  const E = { x: right, y: midY };
  const S = { x: midX, y: bottom };
  const W = { x: left, y: midY };
  const C = { x: midX, y: midY };

  const NE = { x: threeX, y: quarterY };
  const SE = { x: threeX, y: threeY };
  const SW = { x: quarterX, y: threeY };
  const NW = { x: quarterX, y: quarterY };

  const TL = { x: left, y: top };
  const TR = { x: right, y: top };
  const BR = { x: right, y: bottom };
  const BL = { x: left, y: bottom };

  const shapes: Point[][] = [
    [N, NE, C, NW], //  1  top kite
    [N, NW, TL], //     2  upper left
    [TL, NW, W], //     3  mid left
    [NW, C, SW, W], //  4  left kite
    [W, SW, BL], //     5  lower left
    [BL, SW, S], //     6  bottom left
    [SW, C, SE, S], //  7  bottom kite
    [S, SE, BR], //     8  bottom right
    [BR, SE, E], //     9  mid right
    [SE, C, NE, E], // 10  right kite
    [E, NE, TR], //    11  upper right
    [TR, NE, N], //    12  top right
  ];

  const built = shapes.map((points, house) => {
    const sign = (lagnaSign + house) % 12;
    const middle = centroid(points);

    const caption = placeCaption(points, house + 1, C, middle, numbered);

    // The cluster sits at the compartment's middle unless the caption is
    // standing on that row, in which case it steps aside by the caption's own
    // width. `Can` ran into `Ju` when it did not.
    //
    // Asked of the caption rather than of the compartment's shape. A wall
    // triangle's caption used to be level with its centroid by construction, so
    // "is this a wall triangle" and "is the caption in the way" were the same
    // question; now that the caption sits at the narrow end of the wall they
    // are not, and the eight compartments where the caption is nowhere near the
    // middle should not be paying for it.
    const room = captionBox(caption.at, caption.anchor, numbered);
    const level = middle.y + CAPTION_BELOW > room.top && middle.y - CAPTION_ABOVE < room.bottom;
    const body =
      level && caption.anchor !== "middle"
        ? {
            x:
              middle.x +
              (caption.anchor === "start" ? CAPTION_WIDTH : -CAPTION_WIDTH),
            y: middle.y,
          }
        : middle;

    return {
      points,
      label: caption.at,
      labelAnchor: caption.anchor,
      body,
      rashi: rashis[sign]!,
      sign,
    };
  });

  // The direction each sign travels, added once the ring exists: from the
  // previous house's middle toward the next one's. Taken from the ring rather
  // than from a compartment's own edges, so twelve slides read as one rotation
  // and no edge has to be named the entry or the exit.
  return built.map((compartment, house) => {
    // Toward the *previous* house, which is where the sign is going.
    //
    // A house is `(rashi - lagna) mod 12 + 1`, so when the lagna advances a sign
    // every rashi's house number goes *down*: Mesha sits in house 1 with Mesha
    // rising and in house 12 with Vrishabha rising. Content travels backwards
    // around the ring.
    //
    // This pointed forwards, which crept each compartment's contents away from
    // the wall they were about to cross - the exact opposite of the one thing
    // the drift exists to show.
    const previous = built[(house + 11) % 12]!;
    const next = built[(house + 1) % 12]!;
    const ahead = centroid(previous.points);
    const behind = centroid(next.points);
    const dx = ahead.x - behind.x;
    const dy = ahead.y - behind.y;
    const length = Math.hypot(dx, dy);

    // The route, from the edge shared with the *next* house to the edge shared
    // with the previous one, bowed through the compartment's own cluster point.
    // Those two edges always meet at a vertex - the outer point of a kite, the
    // quarter point of either triangle - so the chord between them runs through
    // the tightest part of the shape and the bow is what keeps the route in the
    // part of the compartment that has room in it. The cluster point is already
    // the answer to "where in this shape does text go": for a wall triangle it
    // is the centroid stepped off the wall, clear of the caption.
    const entry = gate(compartment.points, next.points);
    const exit = gate(compartment.points, previous.points);

    // The route is chosen for the widest label that will stand on it. One
    // retrograde body in a compartment widens every slot in it - the scale has
    // to be one scale, or two bodies three degrees apart would not be three
    // degrees apart - so it has to be known before the route is picked, not
    // after. A wall triangle's band halves on the strength of a single bracket.
    // The route is chosen for the *narrowest* label that will stand on it, for
    // the same reason the band is measured that way: it is the degree scale's
    // extent, and a wider body takes a lane rather than shortening the ruler
    // for everything else.
    const narrowest = compartment.rashi.grahas.every((graha) => graha.retrograde)
      ? GRAHA_HALF
      : GRAHA_HALF_PLAIN;

    // The route is the layout now, not an overlay on it, so every North Indian
    // compartment has one. Choosing it means measuring seven candidates, which
    // is what a chart costs to lay out at all.
    const path =
      entry && exit
        ? bestRoute(
            compartment.points,
            entry,
            exit,
            compartment.body,
            captionBox(compartment.label, compartment.labelAnchor, numbered),
            narrowest,
          )
        : undefined;

    return {
      ...compartment,
      along: length > 0 ? { x: dx / length, y: dy / length } : undefined,
      path,
    };
  });
}

/**
 * The South and East Indian compartments.
 *
 * The same geometry - four by four with the middle two by two removed - so one
 * function builds both. They differ in which cell holds Mesha and which way the
 * signs run, and in nothing else.
 *
 * **South Indian**: Mesha is the second cell from the left in the top row, and
 * the count is clockwise. The corners are the check: Meena upper-left, Mithuna
 * upper-right, Kanya lower-right, Dhanu lower-left.
 *
 * **East Indian**: Mesha is the top-centre cell and the count is anticlockwise
 * ([Jothishi](https://jothishi.com/different-birth-chart-formats/): "The centre
 * top square is always Aries (Mesha)… the counting is anti-clockwise"). The top
 * row has four cells and no exact centre, so Mesha takes the third - the cell
 * right of the vertical midline, which is where an anticlockwise count puts the
 * rest of the top row to its left, as the published charts show.
 */
function gridCompartments(
  rashis: ChakraRashi[],
  format: "south" | "east",
): Compartment[] {
  const cellWidth = (WIDTH - INSET * 2) / 4;
  const cellHeight = (HEIGHT - INSET * 2) / 4;

  // The twelve cells of the ring, clockwise from the top-left corner.
  const ring: [number, number][] = [
    [0, 0],
    [1, 0],
    [2, 0],
    [3, 0],
    [3, 1],
    [3, 2],
    [3, 3],
    [2, 3],
    [1, 3],
    [0, 3],
    [0, 2],
    [0, 1],
  ];

  const meshaAt = format === "south" ? 1 : 2;
  const step = format === "south" ? 1 : -1;

  return rashis.map((rashi, sign) => {
    // `+ 144` is only there to keep the modulus positive: `step` is -1 for the
    // anticlockwise format, and 144 is 12 twelves, comfortably past the most
    // negative value `step * sign` can reach. It is not a quantity of anything.
    const at = (meshaAt + step * sign + 144) % 12;
    const [column, row] = ring[at]!;
    const left = INSET + column * cellWidth;
    const top = INSET + row * cellHeight;
    return {
      points: [
        { x: left, y: top },
        { x: left + cellWidth, y: top },
        { x: left + cellWidth, y: top + cellHeight },
        { x: left, y: top + cellHeight },
      ],
      // Top left of the cell, inset by the same clearance the North Indian
      // captions keep off their frame and dropped by one ascent so the ink sits
      // under the top edge rather than on it. A grid cell has no vertex to
      // choose between, so there is no rule to follow here beyond that.
      label: { x: left + LABEL_WALL, y: top + CAPTION_ABOVE + 1 },
      labelAnchor: "start" as const,
      body: { x: left + cellWidth / 2, y: top + cellHeight / 2 + 3 },
      rashi,
      sign,
    };
  });
}

export function Chakra(props: {
  data: ChakraData;
  format: ChartFormat;
  /** Whether the North Indian compartments carry the sign's number rather than
   *  its name. Both reference applications write a number; the name is what a
   *  reader can use without a lookup, so it is the default and this is the
   *  setting. Ignored by the other two formats, whose compartments *are* the
   *  signs and are labelled by name in both references. */
  numbered: boolean;
  /** How far this chart is from changing, 0 to 1. The compartments' contents
   *  slide across the room the layout left them as it runs, so a compartment
   *  hard against its exit wall is one about to hand over.
   *
   *  Left at 0.5 — the middle, where the static layout puts things — when the
   *  chart is not animating, so the same code draws both. */
  progress: number;
  /** Whether a handover is animated. The drift is a position and reads without
   *  motion; the handover is the only event fast enough to watch, so this is
   *  what the setting mostly controls. */
  animate: boolean;
  /** Draw a field of stars behind the chart. Decoration, and the only thing in
   *  the chart that is: it carries no reading and is hidden from screen
   *  readers. */
  sky?: boolean;
  /** Draw the degree lines the compartments are laid out on, and place the
   *  bodies on them. Implies `pathway`: a scale drawn over bodies that are not
   *  standing on it would be a chart saying something untrue. Off by default;
   *  the reader turns it on in Advanced. */
  grid?: boolean;
}): JSX.Element {
  // Falls back to Mesha rather than to -1. The payload always numbers the houses
  // from the lagna so exactly one rashi is house 1, but `findIndex` returns -1
  // when nothing matches, and -1 was fed straight into `(lagnaSign + house) % 12`
  // and then used to index the rashi array - which hands `undefined` to a
  // renderer that dereferences it. A chart drawn from the wrong sign is wrong;
  // a chart that throws takes the panel with it.
  // Unique per chart, because the harness draws sixteen on one page and a clip
  // path is addressed by id.
  const id = createUniqueId();

  const lagnaSign = () => {
    const found = props.data.rashis.findIndex((rashi) => rashi.house === 1);
    return found < 0 ? 0 : found;
  };

  // A memo, and it has to be one. Two `Index`es read this - the clip paths and
  // the contents - and each wraps its `each` in a memo of its own, so a plain
  // accessor ran the whole layout twice per update and returned two unrelated
  // arrays of fresh objects. That is twelve compartments, each choosing a route
  // from seven candidates, done twice a second while the chart animates.
  //
  // The file already insists on memos inside the `Index` body for exactly this
  // reason; the accessor feeding both `Index`es was the one that was missed.
  const compartments = createMemo((): Compartment[] =>
    props.format === "north"
      ? // Routes always, for North Indian: the placement is built on them now,
        // not only the overlay. The argument is gone with the choice.
        northCompartments(props.data.rashis, lagnaSign(), props.numbered)
      : gridCompartments(props.data.rashis, props.format),
  );

  return (
    <svg
      class="chakra"
      viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
      width={WIDTH}
      height={HEIGHT}
      role="img"
      aria-label={spoken(props.data, props.format)}
    >
      {/* The sky, before anything else, so every line and every label is drawn
          over it. Its own group rather than a CSS background: the chart is an
          SVG scaled with the panel, and a background image would not scale with
          it. Hidden from the accessibility tree - it carries nothing to read. */}
      {/* Pitch, always, whether or not there are stars on it. The chart is a
          window onto the sky and a window is dark when nothing is lit; the
          black is also what every label's stroke is cut against, so it cannot
          come and go with a setting. This is the one surface in the app that
          refuses the panel's material. */}
      <rect
        class="chakra__ground"
        x={INSET}
        y={INSET}
        width={WIDTH - INSET * 2}
        height={HEIGHT - INSET * 2}
      />

      <Show when={props.sky}>
        <defs>
          {/* What makes a dot a star. A flat disc of one opacity is a speck at
              any size; a point of light has a core that is nearly white and a
              falloff around it, and the eye reads the falloff as brightness
              rather than as size. The drawn circle is the halo - the lit core
              is the inner quarter of it. */}
          <radialGradient id={`${id}-star`}>
            <stop offset="0%" stop-color="var(--disc-lit)" stop-opacity="1" />
            <stop offset="22%" stop-color="var(--disc-lit)" stop-opacity="0.85" />
            <stop offset="45%" stop-color="var(--disc-lit)" stop-opacity="0.28" />
            <stop offset="100%" stop-color="var(--disc-lit)" stop-opacity="0" />
          </radialGradient>
        </defs>

        <g class="chakra__sky" aria-hidden="true">
          <For each={SKY}>
            {(star) => (
              <circle
                classList={{ "is-twinkling": star.lively }}
                cx={star.x}
                cy={star.y}
                r={star.r}
                fill={`url(#${id}-star)`}
                style={{
                  // Both ends of the twinkle, per star, because a keyframe
                  // cannot read the value it is animating from: `opacity:
                  // inherit` in the keyframe takes the group's, which would
                  // throw away every star's own brightness and pulse the whole
                  // field between two identical values.
                  "--star-dim": String(star.a),
                  // The twinkle runs from below the resting value to above it,
                  // so the star is seen to dim as well as to flare.
                  "--star-low": String(star.a * 0.55),
                  "--star-lit": String(star.lit),
                  "animation-duration": `${star.period}s`,
                  "animation-delay": `-${star.delay}s`,
                }}
              />
            )}
          </For>
        </g>
      </Show>

      {/* One clip path per house, so a group arriving from the next compartment
          comes *through* the wall rather than over it. The wall stays a wall. */}
      <defs>
        <Index each={compartments()}>
          {(compartment, house) => (
            <clipPath id={`${id}-${house}`}>
              <polygon points={polygon(compartment().points)} />
            </clipPath>
          )}
        </Index>
      </defs>

      {/* `Index`, not `For`. A house is a position and keeps its node; what
          changes is the sign standing in it. Keyed by identity the whole ring
          would be rebuilt on every refetch - once a second while the chart
          follows the lagna - and nothing could be animated, because nothing
          would survive long enough to animate. */}
      <Index each={compartments()}>
        {(each, house) => {
          // Laid out once, then slid as one. Everything the compartment draws
          // takes the same offset, because the sign is what carries the bodies
          // standing in it - a name that moves and leaves its grahas behind is
          // drawing something that is not true.
          // Memos, not values. `Index` gives an accessor per position and runs
          // this body once, so anything read out of it here is frozen at the
          // first render and never changes again. That is not a subtle failure:
          // it is the whole chart going static while the clock runs.
          const caption = createMemo(() =>
            captionBox(
              each().label,
              each().labelAnchor,
              props.format === "north" && props.numbered,
            ),
          );
          // Two layouts, one of them a prototype. The packed one is what
          // ships: rows and columns, then a rigid slide across whatever slack
          // is left. The pathway one is a *plan* - a type size, a station and a
          // parallel line per body - and then a position read off that plan at
          // whatever instant is being drawn.
          //
          // The split is the point. `plan` deliberately does not read
          // `props.progress`, so nothing about the arrangement can be decided
          // by an instant; only `place` moves. Deciding the arrangement per
          // frame is what made compartments teleport, and it is the kind of
          // mistake that hides behind a check that samples instants one at a
          // time and finds each of them legal.
          const path = createMemo(() => each().path);
          const bodies = createMemo(() =>
            each().rashi.grahas.map((graha) => ({
              progress: graha.progress,
              // The brackets are what makes the widest label, and they are
              // known here. Eight labels in nine are eight units narrower than
              // the packed layout has to assume.
              half: graha.retrograde ? GRAHA_HALF : GRAHA_HALF_PLAIN,
            })),
          );
          // The grid implies the placement. Drawing a degree scale over bodies
          // the packed layout put wherever they fit would be the chart
          // asserting a degree it has not earned - the same fault the Moshier
          // note and the scheme on the caption's hover exist to prevent. So one
          // switch means one thing: place the grahas by degree, and show the
          // scale they stand on. `pathway` alone stays available to the harness,
          // which needs the placement without the overlay to judge it.
          // North Indian is always threaded. Placing a body at its own degree
          // asserts nothing the chart cannot support, so there was never a
          // reason to wait to be asked for it - only the *lines* were ever the
          // setting. The other two formats have no route through a compartment
          // and so have no traversal; they are packed and they are still.
          const threading = createMemo(() =>
            Boolean(props.format === "north" && path()),
          );
          const plan = createMemo(() => {
            if (!threading() || bodies().length === 0) return null;
            return planFor(
              each().points,
              path()!,
              caption(),
              bodies(),
              each().body,
              each().along,
            );
          });
          const packedLayout = createMemo(() =>
            threading()
              ? null
              : cluster(
                  each().points,
                  each().body,
                  each().rashi.grahas.length,
                  caption(),
                ),
          );
          const scale = createMemo(
            () => plan()?.scale ?? packedLayout()?.scale ?? 1,
          );
          const at = createMemo<Point[]>(() => {
            const laid = plan();
            if (laid) {
              return place(laid, path()!, each().along, props.progress);
            }
            // South and East Indian. Their compartments are cells in a grid
            // and the signs in them do not move - what moves is which cell is
            // house 1, and that is a mark, not a position. Packed, and still.
            return packedLayout()?.at ?? [];
          });

          // What the development grid draws. It exists whether or not anything
          // stands in the compartment: an empty house still has a route, and
          // the overlay is about the shape rather than about its contents.
          const scaleBar = createMemo(() => {
            const along = path();
            if (!props.grid || props.format !== "north" || !along) return null;
            const laid = plan();
            const widest = Math.max(
              ...bodies().map((body) => body.half),
              GRAHA_HALF_PLAIN,
            );
            const band =
              laid && !laid.packed
                ? laid.band
                : reach(each().points, along, caption(), 1, widest);
            if (!band) return null;
            return {
              band,
              pitch:
                laid && !laid.packed
                  ? laid.pitch
                  : lanePitch(along.across, 1, widest),
              widest,
            };
          });

          // The handover. When the sign standing in this house changes, the
          // arriving group is animated in from the wall it came through - the
          // wall toward the next house, since a rashi moves to the *previous*
          // house as the lagna advances.
          //
          // Imperative rather than a CSS class: the node is updated in place
          // rather than replaced, so there is no mount for an enter transition
          // to hang on.
          let group: SVGGElement | undefined;
          let previous: string | undefined;
          let running: Animation | undefined;
          // Left running, an animation on a detached node holds that node and
          // keeps a callback on the document timeline until it finishes. Twelve
          // of them survive leaving the chart view.
          onCleanup(() => running?.cancel());
          createEffect(() => {
            const sign = each().rashi.name;
            const changed = previous !== undefined && previous !== sign;
            previous = sign;
            // Where the arriving group comes from: the route's own direction
            // at its entry gate, which is the gate the previous compartment's
            // exit hands to. The ring's direction is the fallback for the two
            // grid formats, which have no route - and no handover slide either,
            // since their signs do not change compartment.
            const track = path();
            const along = track ? heading(track, 0) : each().along;
            if (!changed || !group || !props.animate || !along) return;

            // Cancelled first. A handover can arrive while the last one is
            // still playing - in a fast division, or in a window the system has
            // throttled - and two animations on one element compose rather than
            // replace, so they pile up and the group crawls.
            running?.cancel();
            running = group.animate(
              [
                {
                  transform: `translate(${-along.x * HANDOVER}px, ${-along.y * HANDOVER}px)`,
                  opacity: 0,
                },
                { transform: "translate(0px, 0px)", opacity: 1 },
              ],
              { duration: 260, easing: "cubic-bezier(0.2, 0, 0, 1)" },
            );
          });

          return (
            <>
              <polygon
                class="chakra__cell"
                classList={{ "is-lagna": each().rashi.house === 1 }}
                points={polygon(each().points)}
              />

              {/* The development grid. Its own clipped group, before the
                contents and outside the group the handover animates: the
                degree scale belongs to the compartment, not to the sign
                standing in it, so it must not slide in with an arriving one. */}
              <Show when={scaleBar()}>
                {(bar) => (
                  <g
                    class="chakra__grid"
                    clip-path={`url(#${id}-${house})`}
                    aria-hidden="true"
                  >
                    <DegreeGrid
                      points={each().points}
                      path={path()!}
                      caption={caption()}
                      band={bar().band}
                      pitch={bar().pitch}
                      half={bar().widest}
                      progress={props.progress}
                    />
                  </g>
                )}
              </Show>

              {/* The sign's name, in all three formats. North Indian conventionally
                writes a number here because its compartments are houses and the
                sign is what moves through them - but a number is a lookup, and
                the three letters cost the same room. The label is set in its own
                colour so it stays the compartment's caption rather than
                competing with the bodies standing in it. */}
              <g
                ref={group}
                clip-path={`url(#${id}-${house})`}
                data-layout={plan()?.how}
                data-cover={plan()?.cover?.toFixed(2)}
                data-scale={scale()}
              >
              <text
                class="chakra__label"
                classList={{ "is-on-lagna": each().rashi.house === 1 }}
                x={each().label.x}
                y={each().label.y}
                text-anchor={each().labelAnchor}
              >
                {props.format === "north" && props.numbered
                  ? each().sign + 1
                  : each().rashi.short}
                <title>
                  {each().rashi.name} · house {each().rashi.house}
                </title>
              </text>

              {/* South and East mark the rising sign, because nothing else in
                those formats says which it is. North does not need to: the
                lagna is house 1 and house 1 is always the top kite. */}
              <Show
                when={props.format !== "north" && each().rashi.house === 1}
              >
                <line
                  class="chakra__lagna-stroke"
                  x1={each().points[0]!.x}
                  y1={each().points[0]!.y}
                  x2={each().points[2]!.x}
                  y2={each().points[2]!.y}
                />
              </Show>

              <Index each={at()}>
                {(spot, index) => {
                  const graha = () => each().rashi.grahas[index]!;

                  // The slide between one computed chart and the next.
                  //
                  // The panel refetches once a second over IPC, so positions
                  // arrive a second apart and at uneven moments - the interval
                  // fires on time, the round trip does not. Drawn raw that is a
                  // twitch every second rather than a body in motion, which is
                  // the opposite of what the traversal is for.
                  //
                  // The ephemeris is still the only source of a position. What
                  // is eased is the *drawing* between two computed ones, which
                  // is what the handover has always done (`animation.md` §5) -
                  // no figure is invented, and the caption still prints only
                  // degrees the ephemeris returned.
                  //
                  // Linear, because the motion it stands for is linear. An
                  // ease-in-out would draw the lagna slowing down and speeding
                  // up once a second, which is a statement about the sky.
                  let node: SVGTextElement | undefined;
                  let before: Point | undefined;
                  let held: string | undefined;
                  let at: number | undefined;
                  let sliding: Animation | undefined;
                  onCleanup(() => sliding?.cancel());
                  createEffect(() => {
                    const now = spot();
                    const sign = each().rashi.name;
                    // Not across a handover. The sign standing here has
                    // changed, so these are different bodies at different
                    // degrees, and the compartment's own slide is already
                    // carrying them in.
                    const settled = held === sign;
                    held = sign;
                    const was = before;
                    before = now;
                    const clock = performance.now();
                    const since = at === undefined ? undefined : clock - at;
                    at = clock;
                    if (!node || !props.animate || !settled || !was) return;
                    // Nothing to catch up on. After an absence the body is put
                    // where it belongs, because the chart is a reading of now
                    // and opening it should show now rather than replay the
                    // time it was shut. A cancel and no new animation, so a
                    // slide interrupted by the panel closing does not resume
                    // from wherever it had reached.
                    if (since === undefined || since > SETTLE_STALE) {
                      sliding?.cancel();
                      sliding = undefined;
                      return;
                    }
                    const dx = was.x - now.x;
                    const dy = was.y - now.y;
                    if (dx === 0 && dy === 0) return;
                    // The slide lasts as long as the last gap between positions,
                    // so it ends as the next one arrives. Measured rather than
                    // agreed with the caller: the panel feeds this once a second
                    // and the harness ten times a second, and a duration fixed
                    // to either leaves the other permanently behind - restarted
                    // every 100ms, a 1,000ms slide covers a tenth of the way and
                    // the body never arrives anywhere.
                    sliding?.cancel();
                    sliding = node.animate(
                      [
                        { transform: `translate(${dx}px, ${dy}px)` },
                        { transform: "translate(0px, 0px)" },
                      ],
                      {
                        duration: Math.min(
                          Math.max(since, SETTLE_LEAST),
                          SETTLE_MOST,
                        ),
                        easing: "linear",
                      },
                    );
                  });

                  return (
                    <>
                      <text
                        ref={node}
                        class="chakra__graha"
                        classList={{
                          "is-combust": graha().combust,
                          // Which ground this label is cut against. Every
                          // compartment is black except the rising sign's,
                          // which carries a wash over it.
                          "is-on-lagna": each().rashi.house === 1,
                        }}
                        x={spot().x}
                        y={spot().y}
                        text-anchor="middle"
                        // Smaller type where the compartment could not hold
                        // the bodies at full size, which is what a printed
                        // chart does.
                        //
                        // An inline style, not a `font-size` attribute. A
                        // presentation attribute loses to any stylesheet rule
                        // and `.chakra__graha` sets the `font` shorthand, so
                        // the attribute was silently overridden - every
                        // measurement came back at the full size while the
                        // code believed it had shrunk.
                        style={
                          scale() === 1
                            ? undefined
                            : {
                                "font-size": `${(11 * scale()).toFixed(2)}px`,
                              }
                        }
                      >
                        {/* Brackets are the retrograde mark. `Sa(r)` put a second
                          token beside the name and made the cluster read as
                          five things rather than four; `(Sa)` marks the name
                          itself and costs no width the compartment has to find.
                          The bracket is also what a printed chart uses. */}
                        {graha().retrograde
                          ? `(${graha().short})`
                          : graha().short}
                        {/* The hover says everything the abbreviation cannot: the
                          full name, where it stands, and what it is doing
                          there. A native SVG title, so it needs no positioning
                          and cannot be clipped by the panel's own edges. */}
                        <title>
                          {describe(graha(), each().rashi.name)}
                        </title>
                      </text>
                    </>
                  );
                }}
              </Index>
              </g>
            </>
          );
        }}
      </Index>
    </svg>
  );
}

/**
 * The development grid: the family of parallel degree lines a compartment is
 * laid out on.
 *
 * Not decoration and not a debugging aid any more. It is the layout model
 * drawn: the same lines the placement uses, at the same pitch, over the same
 * band. A reader who turns it on sees why a crowded house is crowded, because
 * the number of lines that survive at a station *is* the number of bodies that
 * station can hold.
 *
 * | Mark | What it shows |
 * |---|---|
 * | Dotted lines | every parallel line, over the stretch of it that holds a label |
 * | The heavier dotted line | the base line, which the degree scale is measured on |
 * | Cross ticks | 0°, 5° … 30°, at this instant, drawn across the whole family |
 *
 * **The lines are parallel, not perpendicular offsets of a curve.** That is the
 * fix for bodies swapping places: a family of translates keeps the order along
 * the route and the order on screen the same thing. Drawing them is what makes
 * that visible rather than asserted.
 *
 * **The ticks move.** They occupy the fifth of the band nearest the entry when
 * the lagna has just entered its sign and the fifth nearest the exit when it is
 * about to leave. That is the split between the degree scale and the travel
 * made visible.
 *
 * Styled inline rather than from a stylesheet, because it is drawn from the
 * geometry and there is no state for a rule to key off.
 */
function DegreeGrid(props: {
  points: Point[];
  path: Pathway;
  caption: Box;
  band: [number, number];
  pitch: number;
  half: number;
  progress: number;
}): JSX.Element {
  const sampleAt = (fraction: number) =>
    atFraction(
      props.path,
      props.band[0] + (props.band[1] - props.band[0]) * fraction,
    );

  /** Each line, over the run of it that holds a label. A line with no such run
   *  is not drawn: the compartment does not have it. */
  const lines = () => {
    const from = Math.floor(sampleAt(0));
    const to = Math.ceil(sampleAt(1));
    const drawn: { lane: number; at: Point[] }[] = [];
    for (let step = 0; step <= PATH_LANES; step++) {
      for (const side of step === 0 ? [1] : [1, -1]) {
        const lane = step * side;
        const run: Point[] = [];
        for (let i = from; i <= to; i++) {
          const point = laneAt(props.path, i, lane, props.pitch);
          if (
            standable(
              props.points,
              props.caption,
              labelBox(point, 1, props.half),
            )
          ) {
            run.push(point);
          } else if (run.length > 1) {
            drawn.push({ lane, at: [...run] });
            run.length = 0;
          } else {
            run.length = 0;
          }
        }
        if (run.length > 1) drawn.push({ lane, at: run });
      }
    }
    return drawn;
  };

  /** A degree's tick.
   *
   * **Nothing here is solid, and nothing here spans the compartment.** Both
   * were tried. A solid orange line across a compartment stopped being a scale
   * mark and became the grid: it out-drew the dotted family it was supposed to
   * annotate, and a reader saw three bars per house rather than a set of degree
   * lines with bodies standing on them. Dashing it helped and was not enough -
   * thirty-six wall-to-wall marks still laid a rectilinear pattern over a
   * curved one.
   *
   * They are cross marks on the family now: the majors two and a half lanes
   * wide, the minors under one. The capacity that the wall-to-wall version was
   * showing is already in the drawing - it is how many dotted lines survive at
   * a station - so spanning the chord was saying it twice, in the louder of the
   * two voices.
   *
   * The hierarchy is extent, rhythm, weight and opacity, in that order.
   */
  const tick = (degree: number) => {
    const at = alongRoute(
      props.path,
      sampleAt(
        (1 - degree / 30) * PATH_DEGREE_SHARE +
          props.progress * PATH_TRAVEL_SHARE,
      ),
    );
    const across = props.path.across;
    const major = degree % 15 === 0;
    const arm = props.pitch * (major ? 1.3 : 0.45);
    const span = { back: -arm, forward: arm };
    return {
      x1: at.x + across.x * span.back,
      y1: at.y + across.y * span.back,
      x2: at.x + across.x * span.forward,
      y2: at.y + across.y * span.forward,
      major,
    };
  };

  return (
    <>
      {/* The whole route, faintly, under the family. The dotted lines stop
          where a label would not fit, which is the useful thing to see; this is
          what they stop short of.
          Dotted like everything else, on the finest rhythm on the drawing. It
          was the one solid stroke left and there is no reason for it to be: at
          a quarter opacity a fine dot still reads as one continuous line, which
          is all this has to do. */}
      <polyline
        points={polygon(props.path.at)}
        data-lane="route"
        fill="none"
        style={{
          stroke: "var(--text-tertiary)",
          "stroke-width": "0.4",
          "stroke-dasharray": "0.5 1.5",
          opacity: 0.22,
        }}
      />
      {/* `Index`. `lines()` builds fresh objects, so `For` tore down and rebuilt
          every lane polyline in all twelve compartments on every tick. */}
      <Index each={lines()}>
        {(line) => (
          <polyline
            points={polygon(line().at)}
            data-lane={line().lane}
            fill="none"
            style={{
              stroke: "var(--text-tertiary)",
              "stroke-width": line().lane === 0 ? "0.75" : "0.6",
              // The family is the mark that should read first, so it carries
              // the longest dashes and the most ink of anything drawn here.
              "stroke-dasharray": line().lane === 0 ? "4 2" : "2 2",
              // Ground, not figure. In D60 nothing on the chart visibly moves,
              // so a grid drawn at reading strength becomes the loudest thing
              // in a view whose subject is the bodies. The hierarchy among the
              // marks is unchanged - extent, then rhythm, then weight, then
              // opacity - and the whole ladder is simply set lower.
              opacity: line().lane === 0 ? 0.5 : 0.38,
            }}
          />
        )}
      </Index>
      {/* `Index`, not `For`. The array is seven identical numbers rebuilt on
          every read, so `mapArray`'s diff saw no change and never re-ran the
          child - and `tick` reads `props.progress`. The dotted lanes moved
          every second while the marks they are measured against stayed where
          they were when the chart mounted, on the one overlay whose stated job
          is to show where the degrees are *at this instant*. */}
      <Index each={[0, 5, 10, 15, 20, 25, 30]}>
        {(degree) => {
          const mark = () => tick(degree());
          return (
            <line
              x1={mark().x1}
              y1={mark().y1}
              x2={mark().x2}
              y2={mark().y2}
              style={{
                // Neutral, not --glare. The token file allows the app exactly
                // two hues and --glare is one of them: it means a body lost in
                // the Sun's rays. A grid line drawn in it borrows a meaning the
                // grid does not have, and puts a combustion-coloured mark
                // through compartments where nothing is combust.
                stroke: "var(--text-tertiary)",
                "stroke-width": mark().major ? "0.6" : "0.5",
                // A long dash for the degree marks, a fine dot for the rest.
                "stroke-dasharray": mark().major ? "2.5 2" : "0.75 2",
                opacity: mark().major ? 0.42 : 0.28,
              }}
            />
          );
        }}
      </Index>
      {/* The gates: where this compartment's route meets its neighbours'. The
          two are on the compartment's own boundary, which is why the band
          always stops short of them - a label centred on a wall is half
          through it. */}
      <For each={[props.path.at[0]!, props.path.at[props.path.at.length - 1]!]}>
        {(at) => (
          <circle
            cx={at.x}
            cy={at.y}
            r={1.1}
            style={{ fill: "var(--text-tertiary)", opacity: 0.45 }}
          />
        )}
      </For>
    </>
  );
}

/**
 * The chart, read aloud.
 *
 * A list, not a grid. A screen reader moving through twelve compartments in two
 * dimensions learns the shape and not the chart; the order from the lagna is
 * what a chart means, so that is the order it is read in.
 *
 * Never an abbreviation. `Ma` and `Me` are a pair nobody should have to
 * disambiguate by ear, which is the same reason every other spoken label in the
 * app spells its subject out (DESIGN §11.3).
 */
function spoken(data: ChakraData, format: ChartFormat): string {
  const byHouse = [...data.rashis].sort((a, b) => a.house - b.house);
  const houses = byHouse.map((rashi) => {
    // Every state the compartment draws, said in words. The bracket, the weight,
    // the underline and the wash are all silent otherwise, and a reader using
    // the spoken form would be told less than one looking at the chart.
    const bodies = rashi.grahas
      .map((graha) => {
        const states = [
          graha.retrograde ? "retrograde" : null,
          graha.combust ? "combust" : null,
          graha.dignity ? DIGNITY_WORDS[graha.dignity] : null,
          // `!= null` rather than `!== null`, which also drops `undefined`. A
          // dignity added in Rust but not in `DIGNITY_WORDS` looks up to
          // `undefined`, which passed a `!== null` test and was then asserted to
          // be a string - so the chart would have said `undefined` aloud.
        ].filter((state): state is string => state != null);
        // The degree too. It is in the hover for a sighted mouse user - and the
        // hover is an SVG `<title>` inside `role="img"`, which collapses the
        // subtree, so it reaches neither assistive technology nor the keyboard.
        // Without this line the spoken chart is the only view of it that does
        // not say where a body actually stands.
        const [at, minutes] = graha.degrees_in_rashi;
        return (
          `${graha.name} at ${at} degrees ${minutes} minutes` +
          (states.length > 0 ? ` ${states.join(" ")}` : "")
        );
      })
      .join(", ");
    return `house ${rashi.house}, ${rashi.name}${bodies ? `, ${bodies}` : ", empty"}`;
  });

  const [degrees, minutes] = data.lagna.degrees_in_rashi;
  return (
    `${FORMAT_NAMES[format]} chart. ` +
    `Lagna ${data.lagna.name}, ${degrees} degrees ${minutes} minutes. ` +
    `${houses.join(". ")}.`
  );
}

/**
 * What the hover says about one graha.
 *
 * Everything the two letters in the compartment cannot: the full name, where it
 * stands to the arcminute, and every state it is in. The abbreviation is what
 * the chart has room for; this is what it means.
 */
function describe(graha: ChakraGraha, rashi: string): string {
  const [degrees, minutes] = graha.degrees_in_rashi;
  const states = [
    graha.retrograde ? "retrograde" : null,
    graha.combust ? "combust" : null,
    graha.dignity ? DIGNITY_WORDS[graha.dignity] : null,
    // See `spoken`: `!= null` drops `undefined` too, which a dignity with no
    // entry in `DIGNITY_WORDS` would otherwise become.
  ].filter((state): state is string => state != null);

  return (
    `${graha.name} — ${rashi} ${degrees}°${String(minutes).padStart(2, "0")}′` +
    (states.length > 0 ? ` · ${states.join(", ")}` : "")
  );
}

const DIGNITY_WORDS: Record<string, string> = {
  exalted: "exalted",
  debilitated: "debilitated",
  own_sign: "own sign",
};

const FORMAT_NAMES: Record<ChartFormat, string> = {
  north: "North Indian",
  south: "South Indian",
  east: "East Indian",
};
