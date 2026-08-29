/** The six-row day grid, shared by both panel kinds. */

import type { JSX } from "solid-js";
import { For, Show } from "solid-js";

import { sameDate } from "../lib/calendar";
import { weekdayLabels } from "../lib/format";
import type {
  CellTithi,
  DateKey,
  GrahaCell,
  GrahaInfo,
  GrahaMonth,
  IngressMode,
  MoonCell,
  MoonMonth,
  TransitEvent,
} from "../ipc/types";
import { DayCell, tithiLabel, type RetroPhase } from "./DayCell";
import { phaseLabel } from "../lib/format";

interface Props {
  firstWeekday: number;
  /** Glyph for a graha calendar; unused for the Moon. */
  info?: GrahaInfo;
  month: MoonMonth | GrahaMonth;
  kind: "moon" | "graha";
  selected: DateKey | null;
  today: DateKey;
  southern: boolean;
  /**
   * Whether this is the month filling the window.
   *
   * Three grids are mounted at once and two of them sit off screen behind
   * `overflow: hidden`. They are hidden from assistive technology and kept out
   * of the tab order: 84 gridcells nobody can see, and a today that appears in
   * two grids at once claiming `aria-current` in both, are worse than no
   * neighbours at all.
   */
  active: boolean;
  /**
   * Which ingress the grid labels, if either.
   *
   * Never both: the label takes the glyph's place in a 40px cell and there is
   * one glyph. Ignored for the Moon, which enters a rashi every two and a bit
   * days and a nakshatra every day - a label on every cell says nothing, and it
   * would cost the phase glyph the calendar is for.
   */
  ingress: IngressMode;
  onSelect: (date: DateKey) => void;
}

export function WeekdayRow(props: { firstWeekday: number }): JSX.Element {
  return (
    <div class="weekdays" aria-hidden="true">
      <For each={weekdayLabels(props.firstWeekday)}>
        {(label) => <span>{label}</span>}
      </For>
    </div>
  );
}

/**
 * One month's 42 cells, 240px tall.
 *
 * The cells arrive already laid out: the back end decides which civil days the
 * grid holds, because only it knows where a lunar month begins and ends. All
 * that happens here is the split into rows.
 *
 * Separated from the weekday row so several months can be stacked and moved
 * together while the headings stay put.
 */
export function MonthCells(props: Props): JSX.Element {
  /**
   * The one cell that carries the tab stop.
   *
   * Three cases, not two: the selection, else today, else the first day of the
   * displayed month (DESIGN 11.3). Without the third, scrolling two months away
   * left every cell in all three grids at `tabindex="-1"` - tab went straight
   * out of the document and the calendar was unreachable by keyboard or
   * VoiceOver until an arrow key made a selection.
   *
   * Only days inside the month are eligible, so a today that appears as a
   * neighbouring month's trailing cell does not claim the stop in two grids.
   */
  const focusedDate = (): DateKey | null => {
    const inside = (props.month.days as (MoonCell | GrahaCell)[]).filter(
      (day) => day.in_month,
    );
    const candidate = props.selected ?? props.today;
    const named = inside.find((day) => sameDate(candidate, day.date));
    return named?.date ?? inside[0]?.date ?? null;
  };

  return (
    <>
      {/* Weeks are real elements with role="row".
          A grid whose gridcells are not wrapped in rows is invalid ARIA, and
          WebKit prunes the whole subtree from the accessibility tree - the
          calendar simply does not exist for VoiceOver. Verified by reading the
          live accessibility tree, which reported 12 elements and no cells before
          this wrapper was added. */}
      <div
        class="grid"
        role="grid"
        aria-rowcount={6}
        aria-colcount={7}
        aria-label={props.month.label}
        aria-hidden={props.active ? undefined : "true"}
      >
        <For each={weeks(props.month.days as (MoonCell | GrahaCell)[])}>
          {(week, row) => (
            <div class="week" role="row">
              <For each={week}>
                {(cell, column) => {
                  const cells = props.month.days as (MoonCell | GrahaCell)[];
                  const index = row() * 7 + column();
                  const label = ingressLabel(
                    props.kind === "graha" ? props.ingress : "off",
                    (props.month as GrahaMonth).events,
                    cell,
                  );
                  const common = {
                    selected: sameDate(props.selected, cell.date),
                    today: sameDate(props.today, cell.date),
                    focused: props.active && sameDate(focusedDate(), cell.date),
                    onSelect: () => props.onSelect(cell.date),
                  };

                  return (
                    <Show
                      when={props.kind === "moon"}
                      fallback={
                        <DayCell
                          {...common}
                          kind="graha"
                          data={cell as GrahaCell}
                          info={props.info}
                          retro={retroPhase(cells, index)}
                          ingress={label}
                          label={grahaCellLabel(cell as GrahaCell, label)}
                        />
                      }
                    >
                      <DayCell
                        {...common}
                        kind="moon"
                        data={cell as MoonCell}
                        southern={props.southern}
                        label={moonCellLabel(cell as MoonCell)}
                      />
                    </Show>
                  );
                }}
              </For>
            </div>
          )}
        </For>
      </div>
    </>
  );
}

/**
 * The short name of the division this cell's subject entered today, or `null`.
 *
 * Read from the month's own event list rather than from a flag on the cell: the
 * events are already on the payload for the day view, and a second copy on 42
 * cells is a second thing to keep in step with the first.
 *
 * A day can hold two ingresses of the same kind only if the graha crosses two
 * boundaries in one day, which nothing but the Moon does - and the Moon is not
 * labelled. The first is taken, which is the one the day is named for.
 */
function ingressLabel(
  mode: IngressMode,
  events: TransitEvent[] | undefined,
  cell: MoonCell | GrahaCell,
): string | null {
  if (mode === "off" || !events) return null;
  const wanted = mode === "rashi" ? "rashi_ingress" : "nakshatra_ingress";
  const found = events.find(
    (event) => event.kind === wanted && sameDate(event.date, cell.date),
  );
  return found?.target_short ?? null;
}

/**
 * Where a cell sits in a stretch of retrograde motion.
 *
 * Read from the neighbours in the grid, which arrive in date order. A run that
 * carries on past the grid's edge reads as whole there rather than as opening or
 * closing, which is true: the bracket says where the motion turns, and at the
 * edge of the six weeks on screen it does not.
 */
function retroPhase(
  cells: (MoonCell | GrahaCell)[],
  index: number,
): RetroPhase | null {
  const isRetro = (at: number) => {
    const cell = cells[at] as GrahaCell | undefined;
    return cell?.retrograde ?? false;
  };
  if (!isRetro(index)) return null;

  const before = index === 0 ? true : isRetro(index - 1);
  const after = index === cells.length - 1 ? true : isRetro(index + 1);

  if (before && after) return "within";
  if (!before) return "begins";
  return "ends";
}

/** Splits the flat 42-cell grid into six rows of seven. */
function weeks<T>(cells: T[]): T[][] {
  return Array.from({ length: 6 }, (_, row) =>
    cells.slice(row * 7, row * 7 + 7),
  );
}

/** Spoken date. A cell says the Gregorian date whichever calendar is in force. */
function spokenDate(date: DateKey): string {
  return `${date.day} ${monthName(date.month)}`;
}

/**
 * The tithi, spoken in full.
 *
 * `S8` is a printed abbreviation, not a word, so nothing that reads the cell
 * aloud is ever given it. Kshaya and vriddhi are named too: they are the reason
 * the numbers jump or repeat, and a screen reader that omitted them would leave
 * the sequence looking broken (DESIGN 11.3).
 */
function spokenTithi(tithi: CellTithi): string {
  const parts = [
    `${tithi.paksha === "shukla" ? "Shukla" : "Krishna"} ${tithi.name}`,
  ];
  // Said as the reason, not the mechanism: an instant was chosen because there
  // was no sunrise to choose, and naming only the instant leaves that unsaid.
  if (tithi.reference === "local_noon") {
    parts.push("read at noon, the Sun did not rise");
  }
  // The vriddhi is a property of the tithi, not of the day's dawns. "Second
  // sunrise" beside a date read as a claim that the day had two, which is the
  // wording the day view already dropped for the same reason.
  if (tithi.vriddhi === "first") {
    parts.push("vriddhi, the same tithi names the day after");
  }
  if (tithi.vriddhi === "second") {
    parts.push("vriddhi, the same tithi names the day before");
  }
  for (const skipped of tithi.kshaya) {
    parts.push(`${skipped.name} skipped, no sunrise`);
  }
  return parts.join(", ");
}

function moonCellLabel(cell: MoonCell): string {
  const parts = [spokenDate(cell.date)];
  if (cell.tithi) parts.push(spokenTithi(cell.tithi));
  parts.push(
    `${phaseLabel(cell.phase).toLowerCase()}, ${Math.round(cell.illumination * 100)} percent illuminated`,
  );
  if (cell.combust) parts.push("combust at noon");
  if (!cell.in_month) parts.push("outside this month");
  return parts.join(", ");
}

function grahaCellLabel(cell: GrahaCell, ingress: string | null): string {
  const parts = [spokenDate(cell.date)];
  if (cell.tithi) parts.push(spokenTithi(cell.tithi));
  // The abbreviation is what the cell prints, and `Ari` read aloud is a word
  // rather than a sign. The full name is on the event in the day view; here the
  // label says an ingress happened and which division it was into.
  if (ingress) parts.push(`enters ${ingress}`);
  if (cell.retrograde) parts.push("retrograde");
  // Both are read at the day's reference instant, so the label says so rather
  // than claiming the state held from midnight to midnight.
  if (cell.combust) parts.push("combust at noon");
  if (!cell.in_month) parts.push("outside this month");
  return parts.join(", ");
}

/**
 * What an event is called in the day view's list.
 *
 * Every entry names something that *happens*, because each one is printed
 * against the minute it happened at. `Combust` was the exception: an adjective
 * beside a time reads as "it is combust at 04:12" rather than "it becomes
 * combust at 04:12", and its own partner already said `Leaves combustion`.
 *
 * The division is named too. `Enters Magha` and `Enters Makara` were the same
 * sentence for two different kinds of thing, and the names are alike enough
 * that nothing in the words told them apart.
 */
export function describeEvent(event: TransitEvent): string {
  switch (event.kind) {
    case "rashi_ingress":
      return `Enters ${event.target ?? ""} rashi`.replace("  ", " ").trim();
    case "nakshatra_ingress":
      return `Enters ${event.target ?? ""} nakshatra`.replace("  ", " ").trim();
    case "retrograde_station":
      return "Retrograde station";
    case "direct_station":
      return "Direct station";
    case "combustion_start":
      return "Enters combustion";
    case "combustion_end":
      return "Leaves combustion";
  }
}

function monthName(month: number): string {
  return new Intl.DateTimeFormat(undefined, {
    timeZone: "UTC",
    month: "long",
  }).format(new Date(Date.UTC(2000, month - 1, 1, 12)));
}

export { tithiLabel };
