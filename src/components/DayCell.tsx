/**
 * One 40 x 40 day cell.
 *
 * Anatomy is fixed (docs/DESIGN.md 5.4): numeral above, content below. The date
 * is the label; the phase or the event row is the content.
 */

import type { JSX } from "solid-js";
import { For, Show } from "solid-js";

import type { GridDay } from "../lib/calendar";
import type { EventKind, MoonCell, GrahaCell, TransitEvent } from "../ipc/types";
import { PhaseGlyph } from "./PhaseGlyph";

interface CommonProps {
  cell: GridDay;
  selected: boolean;
  today: boolean;
  focused: boolean;
  onSelect: () => void;
}

interface MoonProps extends CommonProps {
  kind: "moon";
  data: MoonCell | undefined;
  southern: boolean;
  label: string;
}

interface GrahaProps extends CommonProps {
  kind: "graha";
  data: GrahaCell | undefined;
  events: TransitEvent[];
  combust: boolean;
  label: string;
}

type Props = MoonProps | GrahaProps;

/** Instants get a marker; spans get a rule. Ordered so the row reads stably. */
const MARKER_ORDER: EventKind[] = [
  "rashi_ingress",
  "nakshatra_ingress",
  "retrograde_station",
  "direct_station",
];

const MAX_MARKERS = 3;

export function DayCell(props: Props): JSX.Element {
  const classes = () => ({
    "day-cell": true,
    "is-outside": !props.cell.inMonth,
    "is-selected": props.selected,
    "is-today": props.today,
  });

  const markers = () =>
    props.kind === "graha"
      ? props.events
          .filter((event) => MARKER_ORDER.includes(event.kind))
          .sort(
            (a, b) => MARKER_ORDER.indexOf(a.kind) - MARKER_ORDER.indexOf(b.kind),
          )
      : [];

  const overflow = () => markers().length > MAX_MARKERS;
  const shown = () => (overflow() ? markers().slice(0, MAX_MARKERS - 1) : markers());

  return (
    <div
      classList={classes()}
      role="gridcell"
      aria-selected={props.selected}
      aria-current={props.today ? "date" : undefined}
      aria-label={props.label}
      tabindex={props.focused ? 0 : -1}
      onClick={props.onSelect}
      data-day={props.cell.date.day}
    >
      <span class="day-cell__numeral">{props.cell.date.day}</span>

      <Show when={props.kind === "moon"}>
        <span class="day-cell__content">
          <Show when={(props as MoonProps).data}>
            {(data) => (
              <PhaseGlyph
                illumination={data().illumination}
                waxing={data().is_waxing}
                southern={(props as MoonProps).southern}
                size={14}
                dim={!props.cell.inMonth}
              />
            )}
          </Show>
        </span>
      </Show>

      <Show when={props.kind === "graha"}>
        {/* Combustion is a span, drawn as a short rule under the numeral rather
            than as another marker competing for the marker row. */}
        <Show when={(props as GrahaProps).combust}>
          <span class="day-cell__combust" />
        </Show>

        <span class="day-cell__markers">
          <For each={shown()}>
            {(event) => <span class={`marker marker--${event.kind}`} />}
          </For>
          <Show when={overflow()}>
            <span class="marker marker--more" />
          </Show>
        </span>

        {/* A retrograde period becomes one continuous line running across whole
            weeks: visible at a glance, invisible when not looked for. */}
        <Show when={(props as GrahaProps).data?.retrograde}>
          <span class="day-cell__retro" />
        </Show>
      </Show>
    </div>
  );
}
