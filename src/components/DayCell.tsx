/**
 * One 40 x 40 day cell.
 *
 * Anatomy is fixed (docs/DESIGN.md 5.4): a label above, the subject's glyph
 * below. Only the label changes between the two modes:
 *
 * - Solar: the Gregorian day labels the cell.
 * - Lunar: the tithi labels the cell, because that is what the day is called.
 *
 * The other calendar's date sits in the top-right corner, so the second line
 * carries the glyph in both modes.
 *
 * State marks - combustion, retrograde, kshaya - are drawn on the cell rather
 * than on the glyph, so they survive that swap and do not vary between the two
 * calendars.
 */

import type { JSX } from "solid-js";
import { Show } from "solid-js";

import type { CellTithi, GrahaCell, GrahaInfo, MoonCell } from "../ipc/types";
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
  info: GrahaInfo | undefined;
  retro: RetroPhase | null;
  /**
   * Short form of the division entered on this day, or `null`.
   *
   * `Ari`, `P.Ash`. It replaces the glyph rather than joining it: the cell is
   * 40px and the glyph's line is the only place a five-character word fits.
   * Losing the glyph for one cell in a month costs nothing - the whole column
   * of them says which graha this is, and the header says it too.
   */
  ingress: string | null;
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
export function tithiLabel(tithi: CellTithi): {
  prefix: string;
  value: string;
} {
  if (tithi.number === 15) {
    return { prefix: "", value: tithi.paksha === "shukla" ? "P" : "A" };
  }
  // Lowercase. The prefix is a qualifier on the number, not a word of its own,
  // and a lowercase letter is both narrower and visually lighter than a capital
  // - which is the whole complaint: at full height `K14` spent 37% of its width
  // and 21% of the cell on one letter.
  //
  // A letter rather than a colour. Krishna as dark text was measured at 1.51:1
  // against the panel's darkest composited ground and 2.60:1 against its
  // lightest, where the app's floor is 4.5:1 and `--text-tertiary` was retired
  // at 2.21. The glow that would have carried it is thicker than the letterform
  // at this size, and would have made the *dark* fortnight the heavier mark.
  return {
    prefix: tithi.paksha === "shukla" ? "s" : "k",
    value: String(tithi.number),
  };
}

export function DayCell(props: Props): JSX.Element {
  const date = () => props.data.date;
  const inMonth = () => props.data.in_month;
  const tithi = () => props.data.tithi;

  const classes = () => ({
    "day-cell": true,
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
  // The day alone. `1 SEP` needed 22px and the corner has 11: beside the glyph
  // there is only the side margin. Which Gregorian month a lunar month runs
  // into is in the day view, and in the run of numbers itself.
  const gregorian = () => String(date().day);

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
          </span>
        )}
      </Show>

      {/* A tithi that began and ended inside this day is one the grid never
          names, so the numbers jump here. In the corner nothing else uses. */}
      <Show when={(tithi()?.kshaya.length ?? 0) > 0}>
        <span class="day-cell__kshaya" />
      </Show>

      {/* The other calendar's date, in the corner, where it can be read without
          taking the line that says what the day is about. */}
      <Show when={tithi()}>
        <span class="day-cell__gregorian">{gregorian()}</span>
      </Show>

      <span class="day-cell__content">
        <Show when={props.kind === "moon"} fallback={<GrahaMark {...props} />}>
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

      {/* Combustion is a span, so it is drawn as a field over the whole cell
          rather than as a mark at one point in it. Drawn for every subject in
          both calendars: a state that appears in one calendar and not the other
          is a state nobody can learn. */}
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

/**
 * The graha's symbol, on the second line in both calendars - or, on the day it
 * enters a division, the name of what it entered.
 *
 * The word takes the glyph's place rather than sitting beside it. A 40px cell
 * has room for one thing on that line, and on the one day a month the graha
 * changes sign, which sign it changed to is the more useful of the two: the
 * column of glyphs above and below still says which graha this is, and so does
 * the header.
 */
function GrahaMark(props: Props): JSX.Element {
  const ingress = () =>
    props.kind === "graha" ? (props as GrahaProps).ingress : null;

  return (
    <Show
      when={props.kind === "graha" && (props as GrahaProps).info}
      keyed={false}
    >
      {(info) => (
        <span
          class="day-cell__graha"
          classList={{ "is-outside": !props.data.in_month }}
        >
          <Show
            when={ingress()}
            fallback={
              <GrahaGlyph info={info()} size={14} colour="currentColor" />
            }
          >
            {(name) => <span class="day-cell__ingress">{name()}</span>}
          </Show>
        </span>
      )}
    </Show>
  );
}
