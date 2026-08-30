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
  ChakraRashi,
  ChartFormat,
} from "../ipc/types";

/** The drawing box. Square, because all three formats are.
 *
 * 208 and not larger. The region is 264px tall and the caption above the chart
 * takes 14 with its gap, the padding 24 - so anything over 214 makes the chart
 * scroll, and a chart you cannot see whole is not a chart. At 208 the grid
 * formats get 52px cells. */
const SIZE = 208;

interface Point {
  x: number;
  y: number;
}

/** One compartment: its shape, where its two kinds of text sit, and what it holds. */
interface Compartment {
  points: Point[];
  /** The compartment's own name - the rashi's short form, or its number. */
  label: Point;
  /** Where the grahas stack. */
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

/** Move `point` directly away from `from` by `distance`. */
function nudge(point: Point, from: Point, distance: number): Point {
  const dx = point.x - from.x;
  const dy = point.y - from.y;
  const length = Math.hypot(dx, dy) || 1;
  return {
    x: point.x + (dx / length) * distance,
    y: point.y + (dy / length) * distance,
  };
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
): Compartment[] {
  const s = SIZE;
  const half = s / 2;
  const quarter = s / 4;
  const three = (3 * s) / 4;

  const N = { x: half, y: 0 };
  const E = { x: s, y: half };
  const S = { x: half, y: s };
  const W = { x: 0, y: half };
  const C = { x: half, y: half };

  // Where each diamond edge crosses a diagonal.
  const NE = { x: three, y: quarter };
  const SE = { x: three, y: three };
  const SW = { x: quarter, y: three };
  const NW = { x: quarter, y: quarter };

  const TL = { x: 0, y: 0 };
  const TR = { x: s, y: 0 };
  const BR = { x: s, y: s };
  const BL = { x: 0, y: s };

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
    return {
      points,
      // The sign number sits toward the compartment's outer edge and the grahas
      // toward the middle, so a crowded compartment does not run the two
      // together. "Outer" differs per compartment, so it is taken as the
      // direction away from the centre of the chart.
      label: nudge(middle, C, 11),
      body: nudge(middle, C, -3),
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
  const cell = SIZE / 4;

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
    const left = column * cell;
    const top = row * cell;
    return {
      points: [
        { x: left, y: top },
        { x: left + cell, y: top },
        { x: left + cell, y: top + cell },
        { x: left, y: top + cell },
      ],
      label: { x: left + 4, y: top + 11 },
      body: { x: left + cell / 2, y: top + cell / 2 + 3 },
      rashi,
      sign,
    };
  });
}

export function Chakra(props: {
  data: ChakraData;
  format: ChartFormat;
}): JSX.Element {
  const lagnaSign = () =>
    props.data.rashis.findIndex((rashi) => rashi.house === 1);

  const compartments = (): Compartment[] =>
    props.format === "north"
      ? northCompartments(props.data.rashis, lagnaSign())
      : gridCompartments(props.data.rashis, props.format);

  return (
    <svg
      class="chakra"
      viewBox={`0 0 ${SIZE} ${SIZE}`}
      width={SIZE}
      height={SIZE}
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

            {/* North Indian writes the sign's number, because its compartments
                are houses and the sign is what moves through them. South and
                East write the sign's name, because their compartments are the
                signs and it is the houses that move. */}
            <text
              class="chakra__label"
              x={compartment.label.x}
              y={compartment.label.y}
              text-anchor={props.format === "north" ? "middle" : "start"}
            >
              {props.format === "north"
                ? compartment.sign + 1
                : compartment.rashi.short}
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

            <For each={compartment.rashi.grahas}>
              {(graha, at) => (
                <text
                  class="chakra__graha"
                  x={compartment.body.x}
                  y={
                    compartment.body.y +
                    (at() - (compartment.rashi.grahas.length - 1) / 2) * 11
                  }
                  text-anchor="middle"
                >
                  {graha.short}
                  {/* The same mark the grid and the menu bar use, so retrograde
                      looks like one thing wherever it appears (D-022). */}
                  {graha.retrograde ? "℞" : ""}
                </text>
              )}
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
    const bodies = rashi.grahas
      .map(
        (graha) =>
          `${GRAHA_NAMES[graha.graha]}${graha.retrograde ? " retrograde" : ""}`,
      )
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

const FORMAT_NAMES: Record<ChartFormat, string> = {
  north: "North Indian",
  south: "South Indian",
  east: "East Indian",
};

const GRAHA_NAMES: Record<string, string> = {
  surya: "Surya",
  chandra: "Chandra",
  mangala: "Mangala",
  budha: "Budha",
  guru: "Guru",
  shukra: "Shukra",
  shani: "Shani",
  rahu: "Rahu",
  ketu: "Ketu",
};
