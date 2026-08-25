/**
 * A graha's symbol, drawn from the path data the backend serves.
 *
 * The same nine paths the menu bar renders, so panel and menu bar cannot show
 * different symbols for the same graha.
 */

import type { JSX } from "solid-js";

import type { GrahaInfo } from "../ipc/types";

interface Props {
  info: GrahaInfo;
  size: number;
  colour?: string;
}

export function GrahaGlyph(props: Props): JSX.Element {
  const colour = () => props.colour ?? "var(--text-primary)";

  return (
    <svg
      class="graha-glyph"
      width={props.size}
      height={props.size}
      viewBox={props.info.view_box.join(" ")}
      aria-hidden="true"
    >
      <path
        d={props.info.path}
        fill={props.info.filled ? colour() : "none"}
        stroke={props.info.filled ? "none" : colour()}
        stroke-width={props.info.filled ? 0 : props.info.stroke_width}
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  );
}
