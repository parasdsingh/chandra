/** Calendar arithmetic for grid layout and keyboard navigation. */

import type { DateKey } from "../ipc/types";

/** The grid is always six rows, so the panel never changes height. */
export const GRID_ROWS = 6;
export const GRID_COLUMNS = 7;
export const GRID_CELLS = GRID_ROWS * GRID_COLUMNS;

export interface GridDay {
  date: DateKey;
  /** False for days borrowed from the neighbouring months to fill the grid. */
  inMonth: boolean;
}

export function daysInMonth(year: number, month: number): number {
  return new Date(Date.UTC(year, month, 0)).getUTCDate();
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

/** Days between two dates, positive when `to` is later. */
export function daysBetween(from: DateKey, to: DateKey): number {
  const a = Date.UTC(from.year, from.month - 1, from.day);
  const b = Date.UTC(to.year, to.month - 1, to.day);
  return Math.round((b - a) / 86_400_000);
}

/** Weekday as an offset from Monday, 0 to 6. */
export function weekdayMondayZero(date: DateKey): number {
  const day = new Date(Date.UTC(date.year, date.month - 1, date.day)).getUTCDay();
  // getUTCDay is Sunday-zero.
  return (day + 6) % 7;
}

const key = (date: DateKey) => `${date.year}-${date.month}-${date.day}`;

/**
 * The 42 cells of the grid, built from the month's own days.
 *
 * Driven by the returned day list rather than by a year and month number,
 * because a lunar month begins and ends mid-Gregorian-month and has no
 * month number of its own.
 */
export function buildGrid(days: DateKey[], firstWeekday: number): GridDay[] {
  if (days.length === 0) return [];

  const first = days[0]!;
  const member = new Set(days.map(key));
  const lead = (weekdayMondayZero(first) - firstWeekday + GRID_COLUMNS) % GRID_COLUMNS;

  return Array.from({ length: GRID_CELLS }, (_, index) => {
    const date = addDays(first, index - lead);
    return { date, inMonth: member.has(key(date)) };
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

/** Local noon on a date, as epoch milliseconds, for use as a month anchor. */
export function noonAnchor(date: DateKey): number {
  return Date.UTC(date.year, date.month - 1, date.day, 12);
}
