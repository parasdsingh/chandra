/** The six-row day grid, shared by both panel kinds. */

import type { JSX } from "solid-js";
import { For, Show } from "solid-js";

import type { GridDay } from "../lib/calendar";
import { sameDate } from "../lib/calendar";
import { weekdayLabels } from "../lib/format";
import type { DateKey, GrahaMonth, MoonMonth, TransitEvent } from "../ipc/types";
import { DayCell } from "./DayCell";
import { phaseLabel } from "../lib/format";

interface Props {
  grid: GridDay[];
  firstWeekday: number;
  month: MoonMonth | GrahaMonth | undefined;
  kind: "moon" | "graha";
  selected: DateKey | null;
  today: DateKey;
  southern: boolean;
  direction: number;
  onSelect: (date: DateKey) => void;
}

function eventsFor(month: GrahaMonth | undefined, date: DateKey): TransitEvent[] {
  if (!month) return [];
  return month.events.filter(
    (event) =>
      event.date.year === date.year &&
      event.date.month === date.month &&
      event.date.day === date.day,
  );
}

/**
 * Whether a day falls inside a combustion span.
 *
 * Combustion has a start and an end event, which may lie in different months, so
 * the state is carried forward from the last boundary seen rather than looked up
 * per day.
 */
function combustDays(month: GrahaMonth | undefined): Set<number> {
  const days = new Set<number>();
  if (!month) return days;

  const boundaries = month.events
    .filter(
      (event) =>
        event.kind === "combustion_start" || event.kind === "combustion_end",
    )
    .sort((a, b) => a.at.unix_ms - b.at.unix_ms);

  // A month that opens with an end event was already combust when it began.
  let combust = boundaries[0]?.kind === "combustion_end";
  let cursor = 1;

  for (const cell of month.days) {
    while (
      cursor <= boundaries.length &&
      boundaries[cursor - 1] &&
      boundaries[cursor - 1]!.date.day <= cell.day
    ) {
      combust = boundaries[cursor - 1]!.kind === "combustion_start";
      cursor += 1;
    }
    if (combust) days.add(cell.day);
  }
  return days;
}

export function MonthGrid(props: Props): JSX.Element {
  const labels = () => weekdayLabels(props.firstWeekday);
  const combust = () =>
    props.kind === "graha" ? combustDays(props.month as GrahaMonth) : new Set<number>();

  const focusedDate = () => props.selected ?? props.today;

  return (
    <div class="grid-region">
      <div class="weekdays" aria-hidden="true">
        <For each={labels()}>{(label) => <span>{label}</span>}</For>
      </div>

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
        aria-label={`${monthName(props.month?.month ?? 1)} ${props.month?.year ?? ""}`}
        style={{ "--enter-from": `${props.direction * 8}px` }}
        data-direction={props.direction}
      >
        <For each={weeks(props.grid)}>
          {(week) => (
            <div class="week" role="row">
              <For each={week}>
          {(cell) => {
            const dayData = () =>
              cell.inMonth
                ? props.month?.days.find((d) => d.day === cell.date.day)
                : undefined;

            const common = {
              cell,
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
                    data={dayData() as never}
                    events={eventsFor(props.month as GrahaMonth, cell.date)}
                    combust={cell.inMonth && combust().has(cell.date.day)}
                    label={grahaCellLabel(cell, eventsFor(props.month as GrahaMonth, cell.date), (dayData() as never as { retrograde?: boolean })?.retrograde ?? false)}
                  />
                }
              >
                <DayCell
                  {...common}
                  kind="moon"
                  data={dayData() as never}
                  southern={props.southern}
                  label={moonCellLabel(cell, dayData() as never)}
                />
              </Show>
            );
          }}
              </For>
            </div>
          )}
        </For>
      </div>
    </div>
  );
}

/** Splits the flat 42-cell grid into six rows of seven. */
function weeks(grid: GridDay[]): GridDay[][] {
  return Array.from({ length: 6 }, (_, row) => grid.slice(row * 7, row * 7 + 7));
}

/** Spoken label. Phase is never conveyed by the glyph alone (DESIGN 11.3). */
function moonCellLabel(
  cell: GridDay,
  data: { phase: string; illumination: number } | undefined,
): string {
  const date = `${cell.date.day} ${monthName(cell.date.month)}`;
  if (!data) return date;
  const phase = phaseLabel(data.phase as never).toLowerCase();
  return `${date}, ${phase}, ${Math.round(data.illumination * 100)} percent illuminated`;
}

function grahaCellLabel(
  cell: GridDay,
  events: TransitEvent[],
  retrograde: boolean,
): string {
  const parts = [`${cell.date.day} ${monthName(cell.date.month)}`];
  if (retrograde) parts.push("retrograde");
  if (events.length > 0) {
    const described = events.map(describeEvent).join(", ");
    parts.push(
      `${events.length} ${events.length === 1 ? "event" : "events"}: ${described}`,
    );
  }
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
