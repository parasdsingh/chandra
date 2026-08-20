/**
 * The in-panel moon disc.
 *
 * Same terminator geometry as the menu bar icon (docs/DESIGN.md 5.4 and 7.1),
 * expressed as SVG here because the panel is a webview. The formula is shared by
 * definition rather than by copying: semi-minor axis a = R(1 - 2k), signed.
 */

import type { JSX } from "solid-js";
import { Show } from "solid-js";

/** Handle length for a cubic quarter-circle. */
const KAPPA = 0.5523;

const NEW_MOON_THRESHOLD = 0.02;
const FULL_MOON_THRESHOLD = 0.98;

interface Props {
  illumination: number;
  waxing: boolean;
  /** Mirrors the disc, which is what an observer south of the equator sees. */
  southern?: boolean;
  /** Diameter in px. 14 in a day cell, 16 in the header. */
  size: number;
  dim?: boolean;
}

/**
 * One cubic quarter-arc between a point on the vertical axis and one on the
 * horizontal, both measured from the centre.
 */
function quarter(from: [number, number], to: [number, number]): string {
  const c1: [number, number] = [from[0] + to[0] * KAPPA, from[1] + to[1] * KAPPA];
  const c2: [number, number] = [to[0] + from[0] * KAPPA, to[1] + from[1] * KAPPA];
  return `C ${c1[0]} ${c1[1]} ${c2[0]} ${c2[1]} ${to[0]} ${to[1]}`;
}

export function litPath(radius: number, illumination: number, lightRight: boolean): string {
  const terminator = radius * (1 - 2 * illumination);
  const side = lightRight ? 1 : -1;

  return [
    `M 0 ${-radius}`,
    quarter([0, -radius], [side * radius, 0]),
    quarter([side * radius, 0], [0, radius]),
    quarter([0, radius], [side * terminator, 0]),
    quarter([side * terminator, 0], [0, -radius]),
    "Z",
  ].join(" ");
}

export function PhaseGlyph(props: Props): JSX.Element {
  const radius = () => props.size / 2;
  const lightRight = () => props.waxing !== Boolean(props.southern);
  const isNew = () => props.illumination < NEW_MOON_THRESHOLD;
  const isFull = () => props.illumination > FULL_MOON_THRESHOLD;

  return (
    <svg
      class="phase-glyph"
      width={props.size}
      height={props.size}
      viewBox={`${-radius()} ${-radius()} ${props.size} ${props.size}`}
      aria-hidden="true"
      style={{ opacity: props.dim ? 0.45 : 1 }}
    >
      {/* The ring is what keeps a new moon visible at all. It is omitted at full,
          where it would only thicken the edge of a solid disc. */}
      <Show when={!isFull()}>
        <circle
          cx="0"
          cy="0"
          r={radius() - 0.5}
          fill="none"
          stroke={isNew() ? "var(--disc-ring-new)" : "var(--disc-ring)"}
          stroke-width="1"
        />
      </Show>

      <Show when={isFull()}>
        <circle cx="0" cy="0" r={radius()} fill="var(--disc-lit)" />
      </Show>

      <Show when={!isNew() && !isFull()}>
        <path
          d={litPath(radius(), props.illumination, lightRight())}
          fill="var(--disc-lit)"
        />
      </Show>
    </svg>
  );
}
