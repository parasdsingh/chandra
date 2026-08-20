/** Calendar arithmetic for grid layout and keyboard navigation. */

import type { DateKey } from "../ipc/types";

/** The grid is always six rows, so the panel never changes height month to month. */
export const GRID_ROWS = 6;
export const GRID_COLUMNS = 7;
export const GRID_CELLS = GRID_ROWS * GRID_COLUMNS;

export interface GridDay {
  date: DateKey;
  /** False for the leading and trailing days borrowed from neighbouring months. */
  inMonth: boolean;
}

export function daysInMonth(year: number, month: number): number {
  return new Date(Date.UTC(year, month, 0)).getUTCDate();
}

export function addMonths(year: number, month: number, delta: number) {
  const zeroBased = year * 12 + (month - 1) + delta;
  return {
    year: Math.floor(zeroBased / 12),
    month: (((zeroBased % 12) + 12) % 12) + 1,
  };
}

export function sameDate(a: DateKey | null, b: DateKey | null): boolean {
  if (!a || !b) return false;
  return a.year === b.year && a.month === b.month && a.day === b.day;
}

export function addDays(date: DateKey, delta: number): DateKey {
  const value = new Date(Date.UTC(date.year, date.month - 1, date.day));
  value.setUTCDate(value.getUTCDate() + delta);
  return {
    year: value.getUTCFullYear(),
    month: value.getUTCMonth() + 1,
    day: value.getUTCDate(),
  };
}

/**
 * The 42 cells of the grid.
 *
 * `leadingBlanks` comes from the backend as a Monday-based offset; `firstDay`
 * rotates it to the locale's first weekday.
 */
export function buildGrid(
  year: number,
  month: number,
  leadingBlanks: number,
  firstDay: number,
): GridDay[] {
  const lead = (leadingBlanks - firstDay + GRID_COLUMNS) % GRID_COLUMNS;
  const first: DateKey = { year, month, day: 1 };

  return Array.from({ length: GRID_CELLS }, (_, index) => {
    const date = addDays(first, index - lead);
    return { date, inMonth: date.year === year && date.month === month };
  });
}

/** Today in a given IANA zone, which is not always today where the machine is. */
export function todayIn(timeZone: string): DateKey {
  const parts = new Intl.DateTimeFormat("en-CA", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(new Date());

  const value = (type: string) =>
    Number(parts.find((part) => part.type === type)?.value ?? "0");

  return {
    year: value("year"),
    month: value("month"),
    day: value("day"),
  };
}
