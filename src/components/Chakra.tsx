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
 * The row is taller than the 10px type, or consecutive lines touch. The column
 * is wide enough for the longest label - a two-letter name with `(r)` after it. */
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

/** How far a 10px label's glyphs reach above and below its baseline, plus the
 *  clearance it keeps from the compartment's own edge.
 *
 *  The clearance is the point: at 8 and 3 the labels were technically inside
 *  their compartments and visually sitting on the frame, which reads as a
 *  clipped label rather than a placed one. */
const LABEL_ASCENT = 12;
const LABEL_DESCENT = 6;
/** Half the width of a three-letter label at 10px, plus the same clearance. */
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
          y: (a.y + b.y) / 2 + LABEL_ASCENT / 3,
        },
        anchor: onLeft ? "start" : "end",
      };
    }
  }

  return {
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
function cluster(points: Point[], centre: Point, count: number): Point[] {
  const desired = count <= 1 ? 1 : count <= 4 ? 2 : 3;
  const here = spanAt(points, centre.y);
  const columns = Math.max(
    1,
    Math.min(
      desired,
      Math.floor((here.max - here.min - GRAHA_MARGIN) / GRAHA_COLUMN),
    ),
  );
  const rows = Math.ceil(count / columns);

  return Array.from({ length: count }, (_, index) => {
    const row = Math.floor(index / columns);
    const column = index % columns;
    const inThisRow = Math.min(columns, count - row * columns);

    const y = centre.y + (row - (rows - 1) / 2) * GRAHA_ROW;
    const x = centre.x + (column - (inThisRow - 1) / 2) * GRAHA_COLUMN;

    // Whatever the row's width, it has to sit inside the shape at its own
    // height. Half a column either side, because `x` is a text midpoint.
    const span = spanAt(points, y);
    const half = GRAHA_COLUMN / 2;
    const lowest = span.min + half + GRAHA_MARGIN / 2;
    const highest = span.max - half - GRAHA_MARGIN / 2;

    return {
      x:
        lowest <= highest
          ? Math.min(Math.max(x, lowest), highest)
          : (span.min + span.max) / 2,
      y,
    };
  });
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
    return {
      points,
      label: caption.at,
      labelAnchor: caption.anchor,
      body: middle,
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
      label: { x: left + 4, y: top + 11 },
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
  const lagnaSign = () =>
    props.data.rashis.findIndex((rashi) => rashi.house === 1);

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
              )}
            >
              {(at_, at) => {
                const graha = () => compartment.rashi.grahas[at()]!;
                return (
                  <>
                    <text
                      class="chakra__graha"
                      classList={{
                        "is-exalted": graha().dignity === "exalted",
                        "is-debilitated": graha().dignity === "debilitated",
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
        ].filter((state): state is string => state !== null);
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
  ].filter((state): state is string => state !== null);

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
