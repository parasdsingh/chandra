/**
 * Presentation formatting.
 *
 * Every timestamp arrives as epoch milliseconds plus the IANA zone of the view.
 * Formatting happens here with `Intl`, against that zone rather than the
 * machine's, so a user reading a calendar for another city sees that city's
 * clock.
 */

import type { DateKey, Moment, PhaseKey } from "../ipc/types";

export interface FormatContext {
  timeZone: string;
}

/**
 * Whether the times on screen are in a zone other than this machine's.
 *
 * Every time in the app is printed in the observer's zone, which is the right
 * answer - a sunrise is a fact about a place. It is also silently wrong-looking
 * when the two differ: `05:14` for a location eight hours away is not what the
 * Mac's own clock will read, and nothing on screen said so.
 *
 * Asked rather than always printed, because for almost every user the two are
 * the same zone and a standing note about it would be noise on every day.
 */
export function zoneDiffersFromMachine(context: FormatContext): boolean {
  try {
    return (
      new Intl.DateTimeFormat().resolvedOptions().timeZone !== context.timeZone
    );
  } catch {
    // A machine whose own zone cannot be resolved is not evidence that the two
    // differ, and claiming they do would put a note on every day for nothing.
    return false;
  }
}

/**
 * A clock time, in the zone of the view rather than of the machine.
 *
 * The 12 or 24 hour choice is the locale's. A `time_format` setting existed in
 * the schema and was read here, but nothing could ever set it, so every time was
 * formatted at the locale default anyway; it is gone rather than left as a
 * switch with no handle.
 */
export function formatTime(moment: Moment, context: FormatContext): string {
  return new Intl.DateTimeFormat(undefined, {
    timeZone: context.timeZone,
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(moment.unix_ms));
}

/** `19 Aug`, used to qualify a boundary that falls outside the selected day. */
function formatDayMonth(moment: Moment, context: FormatContext): string {
  return new Intl.DateTimeFormat(undefined, {
    timeZone: context.timeZone,
    day: "numeric",
    month: "short",
  }).format(new Date(moment.unix_ms));
}

/**
 * When a span gives way, as one value.
 *
 * One number, chosen by scale: a boundary today is a clock time, and a boundary
 * further off is a date. "Until 9 October" is the fact; the 8:51 PM that goes
 * with it is noise at that distance. The day either side keeps its time, because
 * a tithi ending at two in the morning is still a time to a reader.
 */
export function formatUntil(exit: Moment, context: FormatContext): string {
  const time = formatTime(exit, context);
  if (exit.day_offset === 0) return time;
  if (Math.abs(exit.day_offset) === 1) {
    return `${time}, ${formatDayMonth(exit, context)}`;
  }
  return formatDayMonth(exit, context);
}

/** `20 Aug`, for a line that already carries the weekday. */
export function formatShortDate(date: DateKey): string {
  const value = new Date(Date.UTC(date.year, date.month - 1, date.day, 12));
  return new Intl.DateTimeFormat(undefined, {
    timeZone: "UTC",
    day: "numeric",
    month: "short",
  }).format(value);
}

/** `20 August 2026`, for the day view's header. */
export function formatDateHeading(date: DateKey): string {
  const value = new Date(Date.UTC(date.year, date.month - 1, date.day, 12));
  return new Intl.DateTimeFormat(undefined, {
    timeZone: "UTC",
    day: "numeric",
    month: "long",
    year: "numeric",
  }).format(value);
}

/** `THURSDAY`, the weekday alone. The header already carries the date. */
export function formatWeekday(date: DateKey): string {
  const value = new Date(Date.UTC(date.year, date.month - 1, date.day, 12));
  return new Intl.DateTimeFormat(undefined, {
    timeZone: "UTC",
    weekday: "long",
  })
    .format(value)
    .toUpperCase();
}

const PHASE_LABELS: Record<PhaseKey, string> = {
  new_moon: "New Moon",
  waxing_crescent: "Waxing Crescent",
  first_quarter: "First Quarter",
  waxing_gibbous: "Waxing Gibbous",
  full_moon: "Full Moon",
  waning_gibbous: "Waning Gibbous",
  last_quarter: "Last Quarter",
  waning_crescent: "Waning Crescent",
};

export const phaseLabel = (phase: PhaseKey): string => PHASE_LABELS[phase];

/** Weekday initials for the grid header, from the locale, starting on `firstDay`. */
export function weekdayLabels(firstDay: number): string[] {
  const formatter = new Intl.DateTimeFormat(undefined, {
    timeZone: "UTC",
    weekday: "short",
  });
  // 2024-01-01 was a Monday, so index 0 of this sequence is Monday.
  return Array.from({ length: 7 }, (_, index) => {
    const day = new Date(Date.UTC(2024, 0, 1 + ((index + firstDay) % 7), 12));
    return formatter.format(day).toUpperCase();
  });
}

/**
 * First day of the week for the current locale.
 *
 * `Intl.Locale.prototype.getWeekInfo` is available in WKWebView on macOS 14 and
 * later; older systems fall back to Monday, which is the ISO default and matches
 * the design's own wireframe.
 */
export function localeFirstWeekday(): number {
  type WeekInfo = { firstDay: number };
  const locale = new Intl.Locale(
    new Intl.DateTimeFormat().resolvedOptions().locale,
  ) as Intl.Locale & {
    getWeekInfo?: () => WeekInfo;
    weekInfo?: WeekInfo;
  };

  const info = locale.getWeekInfo?.() ?? locale.weekInfo;
  // getWeekInfo reports 1 = Monday through 7 = Sunday. The grid is Monday-based
  // and zero-indexed, so Monday must map to 0 and Sunday to 6 - a subtraction,
  // not a modulo. `firstDay % 7` sends Monday to 1, which starts the week on a
  // Tuesday: a day no locale actually uses.
  return info ? info.firstDay - 1 : 0;
}
