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

/**
 * Where a day sits in a stretch of retrograde motion.
 *
 * The ring is a bracket, not a badge: it opens on the day the motion turns,
 * closes on the day it turns back, and is whole in between. A run therefore
 * reads as one shape spanning several cells, which is what the span rule at the
 * cell's edge used to say and says better, because it is attached to the graha
 * it is about.
 */
export type RetroPhase = "begins" | "within" | "ends";

interface GrahaProps extends CommonProps {
  kind: "graha";
  data: GrahaCell;
  events: TransitEvent[];
  info: GrahaInfo | undefined;
  retro: RetroPhase | null;
}

type Props = MoonProps | GrahaProps;

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

  const classes = () => ({
    "day-cell": true,
    "is-lunar": tithi() !== null,
    "is-outside": !inMonth(),
    "is-selected": props.selected,
    "is-today": props.today,
  });

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

      {/* Drawn around the graha rather than beside it, so the bracket encloses
          the thing it describes. Dotted because the motion is. */}
      <Show when={props.kind === "graha" && (props as GrahaProps).retro}>
        {(phase) => <RetroRing phase={phase()} />}
      </Show>

      {/* Combustion is a span. A rule says "all day", where a marker would read
          as an instant. Drawn for every subject in both calendars: a state that
          appears in one calendar and not the other is a state nobody can learn. */}
      <Show when={props.data.combust}>
        <span class="day-cell__combust" />
      </Show>


    </div>
  );
}

/**
 * The bracket a retrograde stretch is drawn with.
 *
 * A day the motion turns on shows the half facing the days that are retrograde,
 * so a run opens with a right half, closes with a left one, and is a whole
 * circle in between. Centred on the glyph rather than on the cell: the cell's
 * middle sits between the numeral and the symbol, and a ring there cut through
 * both.
 */
function RetroRing(props: { phase: RetroPhase }): JSX.Element {
  const RADIUS = 9;
  const path = () =>
    props.phase === "begins"
      ? `M 0 ${-RADIUS} A ${RADIUS} ${RADIUS} 0 0 1 0 ${RADIUS}`
      : `M 0 ${RADIUS} A ${RADIUS} ${RADIUS} 0 0 1 0 ${-RADIUS}`;

  return (
    <svg
      class="day-cell__retro-ring"
      viewBox="-10 -10 20 20"
      width="20"
      height="20"
      aria-hidden="true"
    >
      <Show
        when={props.phase === "within"}
        fallback={
          <path
            d={path()}
            fill="none"
            stroke="var(--marker)"
            stroke-width="1"
            stroke-dasharray="1.6 2.2"
            stroke-linecap="round"
          />
        }
      >
        <circle
          cx="0"
          cy="0"
          r={RADIUS}
          fill="none"
          stroke="var(--marker)"
          stroke-width="1"
          stroke-dasharray="1.6 2.2"
          stroke-linecap="round"
        />
      </Show>
    </svg>
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
