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
import { For, Show } from "solid-js";

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
    anchor === "start" ? at.x : anchor === "middle" ? at.x - width / 2 : at.x - width;

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
    // The one edge that lies on the chart's left or right wall, and its middle.
    const wall = edges(points).find(
      ([a, b]) => a.x === b.x && (a.x === INSET || a.x === WIDTH - INSET),
    );
    if (wall) {
      const [a, b] = wall;
      const onLeft = a.x === INSET;
      return {
        at: {
          x: a.x + (onLeft ? LABEL_WALL : -LABEL_WALL),
          // A third of the ascent below the wall's midpoint. `y` is a baseline
          // and the eye centres on the ink, which sits above it - so a caption
          // placed at the exact midpoint reads as sitting high.
          y: (a.y + b.y) / 2 + LABEL_ASCENT / 3,
        },
        anchor: onLeft ? "start" : "end",
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
): Point[] {
  // The band a baseline may sit in, so the whole label stays inside the
  // compartment's height. Every arrangement below is placed within it rather
  // than placed and then checked, which is what makes containment structural
  // rather than something a search can fall through.
  const ys = points.map((point) => point.y);
  const top = Math.min(...ys) + GRAHA_MARGIN / 2 + GRAHA_ABOVE;
  const bottom = Math.max(...ys) - GRAHA_MARGIN / 2 - GRAHA_BELOW;

  const desired = count <= 1 ? 1 : count <= 4 ? 2 : 3;

  // Every shape the count can be laid out in, best first. More columns is a
  // shorter, wider block and fewer is a taller, narrower one, so which of them
  // fits depends on the compartment: a corner triangle runs out of width, and a
  // tall kite runs out of height. Neither direction is always the right one to
  // try, so both are tried and the one nearest the preferred shape wins.
  const shapes = Array.from({ length: count }, (_, index) => index + 1).sort(
    (a, b) => Math.abs(a - desired) - Math.abs(b - desired) || a - b,
  );

  // Pitches, loosest first. The ordinary row spacing is tried for every shape
  // before any tighter one is, so a compartment only gets a squeezed block when
  // no roomy arrangement of any shape would fit.
  const pitches = [GRAHA_ROW, GRAHA_ROW * 0.85, GRAHA_ROW * 0.7];

  for (const pitch of pitches) {
    for (const columns of shapes) {
      const lines = Math.ceil(count / columns);
      const reach = ((lines - 1) / 2) * pitch;

      // The block's centre can only sit where the whole block stays in the
      // band. If that interval is empty the block is taller than the
      // compartment and no offset saves it.
      const lowest = top + reach;
      const highest = bottom - reach;
      if (lowest > highest) continue;

      // Three preferences, each clamped into the feasible interval rather than
      // used raw. Clamping is the fix for a whole class of near misses: the
      // South Indian eight-body block failed by an eighth of a pixel, and a
      // shift of that much fitted it.
      const preferences = [
        centre.y,
        caption.bottom + GRAHA_ABOVE + reach,
        caption.top - GRAHA_BELOW - reach,
      ];

      for (const preferred of preferences) {
        const middle = Math.min(Math.max(preferred, lowest), highest);
        const laid = rows(points, centre.x, middle, count, columns, caption, pitch);
        if (laid) return laid;
      }
    }
  }

  // Nothing fits at any shape, offset or pitch. The compartment is genuinely
  // too small for what is standing in it, so the labels are stacked down its
  // middle at whatever spacing the height allows and each one is pushed inside
  // the shape at its own row.
  //
  // Bodies may then overlap. That is the right way to lose: a graha drawn
  // outside its compartment is *wrong* - it reads as standing in a sign it is
  // not in - and overlapping text is only hard to read.
  //
  // This clamps rather than trusting a centroid. It used to centre the stack on
  // the compartment's `body`, which for a triangle is a third of the way up
  // rather than the middle of the usable band - so a four-body stack in a corner
  // triangle started fourteen units too high and its top label left the chart
  // entirely.
  // Only the heights where a label fits at all. Spreading the stack over the
  // compartment's full height puts rows where the shape is narrower than one
  // label - and in a wall triangle, whose long edge *is* the chart's edge, a
  // label centred in a five-unit slot hangs five units off the side of the
  // drawing. Rows are packed into the part of the shape that can hold them
  // instead, overlapping each other rather than leaving the chart.
  const needed = GRAHA_HALF * 2 + GRAHA_MARGIN;
  const fits = (y: number) => {
    const span = spanOver(points, y - GRAHA_ABOVE, y + GRAHA_BELOW);
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
    const span = spanOver(points, y - GRAHA_ABOVE, y + GRAHA_BELOW);

    // The caption comes out of the room here too. It was reserved in `rows` and
    // not in this path, so a squeezed stack could land on the sign's own name -
    // which it did, in a numbered corner triangle.
    const { min, max } = withoutCaption(span, y, caption);
    const lowest = min + GRAHA_HALF + GRAHA_MARGIN / 2;
    const highest = max - GRAHA_HALF - GRAHA_MARGIN / 2;

    return {
      x:
        lowest <= highest
          ? Math.min(Math.max(centre.x, lowest), highest)
          : (min + max) / 2,
      y,
    };
  });
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
): { min: number; max: number } {
  const level = y + GRAHA_BELOW > caption.top && y - GRAHA_ABOVE < caption.bottom;
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
  const needed = GRAHA_HALF * 2 + GRAHA_MARGIN;
  return kept.max - kept.min >= needed ? kept : span;
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
  pitch: number,
): Point[] | null {
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
      y - GRAHA_ABOVE < Math.min(...ys) + GRAHA_MARGIN / 2 ||
      y + GRAHA_BELOW > Math.max(...ys) - GRAHA_MARGIN / 2
    ) {
      return null;
    }

    const span = spanOver(points, y - GRAHA_ABOVE, y + GRAHA_BELOW);

    const { min, max } = withoutCaption(span, y, caption);

    const half = ((inThisRow - 1) * GRAHA_COLUMN) / 2 + GRAHA_HALF;
    const lowest = min + half + GRAHA_MARGIN / 2;
    const highest = max - half - GRAHA_MARGIN / 2;
    if (lowest > highest) return null;

    const middle = Math.min(Math.max(centreX, lowest), highest);
    placed.push({
      x: middle + (column - (inThisRow - 1) / 2) * GRAHA_COLUMN,
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

  return shapes.map((points, house) => {
    const sign = (lagnaSign + house) % 12;
    const middle = centroid(points);

    const caption = placeCaption(points, house + 1, C, middle, numbered);

    // In a wall triangle the caption hugs the side and the centroid is only a
    // little further in, so the two overlapped - `Can` ran into `Ju`. The
    // cluster steps away from the wall by the caption's own width. The other
    // eight compartments put their caption at a far vertex, which the cluster is
    // nowhere near.
    const againstWall = caption.anchor !== "middle";
    const body = againstWall
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
}): JSX.Element {
  // Falls back to Mesha rather than to -1. The payload always numbers the houses
  // from the lagna so exactly one rashi is house 1, but `findIndex` returns -1
  // when nothing matches, and -1 was fed straight into `(lagnaSign + house) % 12`
  // and then used to index the rashi array - which hands `undefined` to a
  // renderer that dereferences it. A chart drawn from the wrong sign is wrong;
  // a chart that throws takes the panel with it.
  const lagnaSign = () => {
    const found = props.data.rashis.findIndex((rashi) => rashi.house === 1);
    return found < 0 ? 0 : found;
  };

  const compartments = (): Compartment[] =>
    props.format === "north"
      ? northCompartments(props.data.rashis, lagnaSign(), props.numbered)
      : gridCompartments(props.data.rashis, props.format);

  return (
    <svg
      class="chakra"
      viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
      width={WIDTH}
      height={HEIGHT}
      role="img"
      aria-label={spoken(props.data, props.format)}
    >
      <For each={compartments()}>
        {(compartment) => (
          <>
            <polygon
              class="chakra__cell"
              classList={{ "is-lagna": compartment.rashi.house === 1 }}
              points={polygon(compartment.points)}
            />

            {/* The sign's name, in all three formats. North Indian conventionally
                writes a number here because its compartments are houses and the
                sign is what moves through them - but a number is a lookup, and
                the three letters cost the same room. The label is set in its own
                colour so it stays the compartment's caption rather than
                competing with the bodies standing in it. */}
            <text
              class="chakra__label"
              x={compartment.label.x}
              y={compartment.label.y}
              text-anchor={compartment.labelAnchor}
            >
              {props.format === "north" && props.numbered
                ? compartment.sign + 1
                : compartment.rashi.short}
              <title>
                {compartment.rashi.name} · house {compartment.rashi.house}
              </title>
            </text>

            {/* South and East mark the rising sign, because nothing else in
                those formats says which it is. North does not need to: the
                lagna is house 1 and house 1 is always the top kite. */}
            <Show
              when={props.format !== "north" && compartment.rashi.house === 1}
            >
              <line
                class="chakra__lagna-stroke"
                x1={compartment.points[0]!.x}
                y1={compartment.points[0]!.y}
                x2={compartment.points[2]!.x}
                y2={compartment.points[2]!.y}
              />
            </Show>

            <For
              each={cluster(
                compartment.points,
                compartment.body,
                compartment.rashi.grahas.length,
                captionBox(
                  compartment.label,
                  compartment.labelAnchor,
                  props.format === "north" && props.numbered,
                ),
              )}
            >
              {(at_, at) => {
                const graha = () => compartment.rashi.grahas[at()]!;
                return (
                  <>
                    <text
                      class="chakra__graha"
                      classList={{
                        "is-combust": graha().combust,
                      }}
                      x={at_.x}
                      y={at_.y}
                      text-anchor="middle"
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
                      <title>{describe(graha(), compartment.rashi.name)}</title>
                    </text>
                  </>
                );
              }}
            </For>
          </>
        )}
      </For>
    </svg>
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
        return `${graha.name}${states.length > 0 ? ` ${states.join(" ")}` : ""}`;
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
