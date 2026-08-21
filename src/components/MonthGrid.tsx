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
  MoonCell,
  MoonMonth,
  TransitEvent,
} from "../ipc/types";
import { DayCell, tithiLabel } from "./DayCell";
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
  onSelect: (date: DateKey) => void;
}

function eventsFor(month: GrahaMonth, date: DateKey): TransitEvent[] {
  return month.events.filter(
    (event) =>
      event.date.year === date.year &&
      event.date.month === date.month &&
      event.date.day === date.day,
  );
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
  const focusedDate = () => props.selected ?? props.today;

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
      >
        <For each={weeks(props.month.days as (MoonCell | GrahaCell)[])}>
          {(week) => (
            <div class="week" role="row">
              <For each={week}>
                {(cell) => {
                  const common = {
                    selected: sameDate(props.selected, cell.date),
                    today: sameDate(props.today, cell.date),
                    focused: sameDate(focusedDate(), cell.date),
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
                          events={eventsFor(props.month as GrahaMonth, cell.date)}
                          info={props.info}
                          label={grahaCellLabel(
                            cell as GrahaCell,
                            eventsFor(props.month as GrahaMonth, cell.date),
                          )}
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

/** Splits the flat 42-cell grid into six rows of seven. */
function weeks<T>(cells: T[]): T[][] {
  return Array.from({ length: 6 }, (_, row) => cells.slice(row * 7, row * 7 + 7));
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
  if (tithi.reference === "local_noon") parts.push("tithi at local noon");
  if (tithi.vriddhi === "first") parts.push("first of two sunrises");
  if (tithi.vriddhi === "second") parts.push("second sunrise");
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
  if (cell.combust) parts.push("combust");
  if (!cell.in_month) parts.push("outside this month");
  return parts.join(", ");
}

function grahaCellLabel(cell: GrahaCell, events: TransitEvent[]): string {
  const parts = [spokenDate(cell.date)];
  if (cell.tithi) parts.push(spokenTithi(cell.tithi));
  if (cell.retrograde) parts.push("retrograde");
  if (cell.combust) parts.push("combust");
  if (events.length > 0) {
    parts.push(
      `${events.length} ${events.length === 1 ? "event" : "events"}: ${events
        .map(describeEvent)
        .join(", ")}`,
    );
  }
  if (!cell.in_month) parts.push("outside this month");
  return parts.join(", ");
}

export function describeEvent(event: TransitEvent): string {
  switch (event.kind) {
    case "rashi_ingress":
    case "nakshatra_ingress":
      return `Enters ${event.target ?? ""}`.trim();
    case "retrograde_station":
      return "Retrograde station";
    case "direct_station":
      return "Direct station";
    case "combustion_start":
      return "Combust";
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
