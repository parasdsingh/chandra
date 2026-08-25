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

/**
 * Local noon on a date in a given zone, as epoch milliseconds.
 *
 * The zone is not decoration. The back end resolves this instant to a civil date
 * in the observer's zone, so 12:00 UTC is already tomorrow at UTC+12 and beyond:
 * on the last day of a month the panel would open on the next one, with today
 * shown as a dimmed leading cell. In a lunar month the anchor can land on the
 * far side of a syzygy.
 *
 * Noon rather than midnight because it is the furthest an instant can be from
 * either edge of the day, so no offset change can carry it onto another date.
 */
export function noonAnchor(date: DateKey, timeZone: string): number {
  const utcNoon = Date.UTC(date.year, date.month - 1, date.day, 12);
  // Twice: the offset is first measured at a guess that may be up to fourteen
  // hours away from the answer, and a daylight saving change inside that gap
  // would make the first measurement the wrong one.
  const once = utcNoon - zoneOffsetMs(utcNoon, timeZone);
  return utcNoon - zoneOffsetMs(once, timeZone);
}

/** How far a zone runs ahead of UTC at an instant, in milliseconds. */
function zoneOffsetMs(instant: number, timeZone: string): number {
  const parts = new Intl.DateTimeFormat("en-CA", {
    timeZone,
    hourCycle: "h23",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).formatToParts(new Date(instant));

  const value = (type: string) =>
    Number(parts.find((part) => part.type === type)?.value ?? "0");

  const asIfUtc = Date.UTC(
    value("year"),
    value("month") - 1,
    value("day"),
    value("hour"),
    value("minute"),
    value("second"),
  );
  return asIfUtc - instant;
}
