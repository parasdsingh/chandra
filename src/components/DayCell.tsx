/**
 * One 40 x 40 day cell.
 *
 * Anatomy is fixed (docs/DESIGN.md 5.4): a label above, content below. Which is
 * which depends on the calendar in force, and that is the whole of the
 * difference between the two modes:
 *
 * - Solar: the Gregorian day labels the cell, the phase or graha glyph is the
 *   content.
 * - Lunar: the tithi labels the cell, because that is what the day is called,
 *   and the Gregorian day becomes the annotation underneath.
 *
 * State marks - combustion, retrograde, ingress, station, kshaya, vriddhi - are
 * drawn on the cell rather than on the glyph, so they survive that swap.
 */

import type { JSX } from "solid-js";
import { Show } from "solid-js";

import type {
  CellTithi,
  EventKind,
  GrahaCell,
  GrahaInfo,
  MoonCell,
  TransitEvent,
} from "../ipc/types";
import { GrahaGlyph } from "./GrahaGlyph";
import { PhaseGlyph } from "./PhaseGlyph";

interface CommonProps {
  selected: boolean;
  today: boolean;
  focused: boolean;
  label: string;
  onSelect: () => void;
}

interface MoonProps extends CommonProps {
  kind: "moon";
  data: MoonCell;
  southern: boolean;
}

interface GrahaProps extends CommonProps {
  kind: "graha";
  data: GrahaCell;
  events: TransitEvent[];
  info: GrahaInfo | undefined;
}

type Props = MoonProps | GrahaProps;

/** Events drawn as a mark on the cell. Spans are drawn as rules instead. */
const MARKED_EVENTS: EventKind[] = [
  "rashi_ingress",
  "nakshatra_ingress",
  "retrograde_station",
  "direct_station",
];

/**
 * What a tithi is called in a cell.
 *
 * `S1`-`S14`, `P` for Purnima, `K1`-`K14`, `A` for Amavasya - the notation
 * printed panchangs use. Derived from the paksha and the number within it, never
 * from the astronomical 1-30 index: amanta and purnimanta months count from
 * opposite ends of that index, so it is right in one and wrong in the other.
 */
export function tithiLabel(tithi: CellTithi): { prefix: string; value: string } {
  if (tithi.number === 15) {
    return { prefix: "", value: tithi.paksha === "shukla" ? "P" : "A" };
  }
  return {
    prefix: tithi.paksha === "shukla" ? "S" : "K",
    value: String(tithi.number),
  };
}

/** Three-letter month, uppercased, for the cell that opens a Gregorian month. */
function monthAbbreviation(month: number): string {
  return new Intl.DateTimeFormat(undefined, { timeZone: "UTC", month: "short" })
    .format(new Date(Date.UTC(2000, month - 1, 1, 12)))
    .toUpperCase();
}

export function DayCell(props: Props): JSX.Element {
  const date = () => props.data.date;
  const inMonth = () => props.data.in_month;
  const tithi = () => props.data.tithi;
  const retrograde = () =>
    props.kind === "graha" && (props as GrahaProps).data.retrograde;

  const classes = () => ({
    "day-cell": true,
    "is-lunar": tithi() !== null,
    "is-outside": !inMonth(),
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

  /**
   * The Gregorian day, which becomes the annotation in lunar mode.
   *
   * It carries the month only where the month changes. Printing it on every
   * cell would be the same three letters 30 times; printing it nowhere would
   * leave a lunar month that runs 13 Aug to 11 Sep with no visible seam.
   */
  const gregorian = () =>
    date().day === 1 ? `1 ${monthAbbreviation(date().month)}` : String(date().day);

  return (
    <div
      classList={classes()}
      role="gridcell"
      aria-selected={props.selected}
      aria-current={props.today ? "date" : undefined}
      aria-label={props.label}
      tabindex={props.focused ? 0 : -1}
      onClick={props.onSelect}
      data-day={date().day}
    >
      <Show
        when={tithi()}
        fallback={<span class="day-cell__numeral">{date().day}</span>}
      >
        {(lunar) => (
          <span class="day-cell__numeral">
            <Show when={tithiLabel(lunar()).prefix}>
              {(prefix) => <span class="day-cell__paksha">{prefix()}</span>}
            </Show>
            {tithiLabel(lunar()).value}
            {/* A tithi that began and ended inside this day is one the grid
                never names, so the numbers jump here. Marked like a footnote,
                on the numeral that jumps, because the corners are spoken for -
                the date on one side, an ingress on the other. */}
            <Show when={lunar().kshaya.length > 0}>
              <span class="day-cell__kshaya" />
            </Show>
          </span>
        )}
      </Show>

      {/* The other calendar's date. In the corner, where it can be read without
          taking the line that says what the day is about. */}
      <Show when={tithi()}>
        <span class="day-cell__gregorian">{gregorian()}</span>
      </Show>

      <span class="day-cell__content">
        <Show
          when={props.kind === "moon"}
          fallback={<GrahaMark {...props} />}
        >
          <PhaseGlyph
            illumination={(props as MoonProps).data.illumination}
            waxing={(props as MoonProps).data.is_waxing}
            southern={(props as MoonProps).southern}
            size={14}
            dim={!inMonth()}
          />
        </Show>
      </span>

      {/* Beside the glyph, in both calendars, placed out of the flow so the
          glyph stays on the column's centre line whether or not the day is
          retrograde. A colour cannot do this job: it has to be learnt, and it is
          the first thing a grayscale or colour-blind rendering loses. */}
      <Show when={retrograde()}>
        <span class="day-cell__retro-mark" aria-hidden="true">
          ℞
        </span>
      </Show>

      {/* Combustion is a span. A rule says "all day", where a marker would read
          as an instant. Drawn for every subject in both calendars: a state that
          appears in one calendar and not the other is a state nobody can learn. */}
      <Show when={props.data.combust}>
        <span class="day-cell__combust" />
      </Show>

      {/* A tithi holding two sunrises names two days. The rule runs full width
          on both of them, so the pair reads as one span bracketed across two
          cells rather than as a repeated number. */}
      <Show when={tithi()?.vriddhi}>
        <span class="day-cell__vriddhi" />
      </Show>

      {/* An event is an instant, so it gets a single mark in the corner rather
          than a row competing with the content for the cell's 40 pixels. */}
      <Show when={marked().length > 0}>
        <span class="day-cell__event" classList={{ "is-station": isStation() }} />
      </Show>
    </div>
  );
}

/** The graha's symbol, drawn only in solar mode where the second line is free. */
function GrahaMark(props: Props): JSX.Element {
  return (
    <Show when={props.kind === "graha" && (props as GrahaProps).info}>
      {(info) => (
        <span
          class="day-cell__graha"
          classList={{ "is-outside": !props.data.in_month }}
        >
          <GrahaGlyph info={info()} size={14} colour="currentColor" />
        </span>
      )}
    </Show>
  );
}
