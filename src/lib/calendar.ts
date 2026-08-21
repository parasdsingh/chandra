/**
 * Calendar arithmetic for keyboard navigation and zone-aware dates.
 *
 * Grid layout is not here. The back end lays the 42 cells out and sends them in
 * reading order, because only it knows which civil days a lunar month contains -
 * it runs between syzygies, not between dates - and a front end that guessed the
 * neighbouring dates ended up drawing cells it had no data for.
 */

import type { DateKey } from "../ipc/types";

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
