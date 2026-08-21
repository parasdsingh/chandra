/**
 * One 40 x 40 day cell.
 *
 * Anatomy is fixed (docs/DESIGN.md 5.4): numeral above, content below. The date
 * is the label; the phase or the event row is the content.
 */

import type { JSX } from "solid-js";
import { Show } from "solid-js";

import type { GridDay } from "../lib/calendar";
import type {
  EventKind,
  GrahaCell,
  GrahaInfo,
  MoonCell,
  TransitEvent,
} from "../ipc/types";
import { GrahaGlyph } from "./GrahaGlyph";
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
  info: GrahaInfo | undefined;
  label: string;
}

type Props = MoonProps | GrahaProps;

/** Events drawn as a mark on the cell. Spans are drawn as rules instead. */
const MARKED_EVENTS: EventKind[] = [
  "rashi_ingress",
  "nakshatra_ingress",
  "retrograde_station",
  "direct_station",
];

export function DayCell(props: Props): JSX.Element {
  const combustToday = () =>
    props.kind === "moon"
      ? ((props as MoonProps).data?.combust ?? false)
      : ((props as GrahaProps).data?.combust ?? false);

  const classes = () => ({
    "day-cell": true,
    "is-outside": !props.cell.inMonth,
    "is-selected": props.selected,
    "is-today": props.today,
  });

  const marked = () =>
    props.kind === "graha"
      ? props.events.filter((event) => MARKED_EVENTS.includes(event.kind))
      : [];

  /** A station outranks an ingress: it is the rarer and larger event. */
  const isStation = () =>
    marked().some(
      (event) =>
        event.kind === "retrograde_station" || event.kind === "direct_station",
    );

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

      {/* Both kinds put their subject's glyph in the same place, so the two
          calendars read the same way: the numeral labels the day, the glyph is
          what the day is about. */}
      <span class="day-cell__content">
        <Show when={props.kind === "moon"}>
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
        </Show>

        <Show when={props.kind === "graha"}>
          <Show when={(props as GrahaProps).info}>
            {(info) => (
              <span
                class="day-cell__graha"
                classList={{ "is-outside": !props.cell.inMonth }}
              >
                <GrahaGlyph info={info()} size={14} colour="currentColor" />
              </span>
            )}
          </Show>
        </Show>
      </span>

      {/* Retrograde is written the way every ephemeris writes it, beside the
          symbol it applies to. A colour cannot do this job: it has to be learnt,
          it is the first thing a grayscale or colour-blind rendering loses, and
          it left the glyph itself carrying two meanings at once. */}
      <Show when={props.kind === "graha" && (props as GrahaProps).data?.retrograde}>
        <span class="day-cell__retro-mark" aria-hidden="true">
          ℞
        </span>
      </Show>

      {/* Combustion is a span. A rule under the numeral says "all day", where a
          marker would read as an instant. */}
      <Show when={combustToday()}>
        <span class="day-cell__combust" />
      </Show>

      {/* A retrograde period becomes one continuous line running across whole
          weeks: visible at a glance, invisible when not looked for. */}
      <Show when={props.kind === "graha" && (props as GrahaProps).data?.retrograde}>
        <span class="day-cell__retro" />
      </Show>

      {/* An event is an instant, so it gets a single mark in the corner rather
          than a row competing with the glyph for the cell's 40 pixels. */}
      <Show when={marked().length > 0}>
        <span
          class="day-cell__event"
          classList={{ "is-station": isStation() }}
        />
      </Show>
    </div>
  );
}
