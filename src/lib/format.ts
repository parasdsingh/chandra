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
export function formatDayMonth(moment: Moment, context: FormatContext): string {
  return new Intl.DateTimeFormat(undefined, {
    timeZone: context.timeZone,
    day: "numeric",
    month: "short",
  }).format(new Date(moment.unix_ms));
}

/**
 * A boundary instant, qualified with its date only when it falls on another day.
 *
 * Spans carry true instants and are never clamped to the day, so an unqualified
 * time would silently read as belonging to the day on screen.
 */
export function formatBoundary(moment: Moment, context: FormatContext): string {
  const time = formatTime(moment, context);
  if (moment.day_offset === 0) return time;
  return `${time} (${formatDayMonth(moment, context)})`;
}

export function formatSpan(
  entry: Moment,
  exit: Moment,
  context: FormatContext,
): string {
  const from =
    entry.day_offset === 0
      ? formatTime(entry, context)
      : `${formatDayMonth(entry, context)} ${formatTime(entry, context)}`;
  return `${from} → ${formatBoundary(exit, context)}`;
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

/**
 * `THURSDAY 20 AUGUST 2026`, the full date on one line.
 *
 * Takes no zone: the value is already a calendar date, resolved in the
 * observer's zone by the backend. Formatting it at UTC noon keeps it from
 * shifting back across the date line during presentation.
 */
export function formatFullDate(date: DateKey): string {
  const value = new Date(Date.UTC(date.year, date.month - 1, date.day, 12));
  return new Intl.DateTimeFormat(undefined, {
    timeZone: "UTC",
    weekday: "long",
    day: "numeric",
    month: "long",
    year: "numeric",
  })
    .format(value)
    .toUpperCase()
    .replace(/,/g, "");
}

export function formatMonthYear(year: number, month: number): string {
  const value = new Date(Date.UTC(year, month - 1, 1, 12));
  return new Intl.DateTimeFormat(undefined, {
    timeZone: "UTC",
    month: "long",
    year: "numeric",
  }).format(value);
}

export function formatMonthShort(month: number): string {
  const value = new Date(Date.UTC(2000, month - 1, 1, 12));
  return new Intl.DateTimeFormat(undefined, {
    timeZone: "UTC",
    month: "short",
  }).format(value);
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

/** `68.4% lit`. One decimal: two is false precision, none loses the day-to-day change. */
export function formatIllumination(fraction: number): string {
  return `${(fraction * 100).toFixed(1)}% lit`;
}

/**
 * `18° 42′ 07″`, zero-padded so the columns align.
 *
 * Two places for the degrees, not three: this is a position *within* a rashi and
 * so is 0 to 29, and padding to three printed a leading zero on every longitude
 * in the app.
 */
export function formatDegrees(parts: [number, number, number]): string {
  const [degrees, minutes, seconds] = parts;
  return `${pad(degrees)}° ${pad(minutes)}′ ${pad(Math.floor(seconds))}″`;
}

function pad(value: number): string {
  return value.toString().padStart(2, "0");
}

/**
 * An angle split into whole degrees and arcminutes, with the carry applied.
 *
 * Rounding 59.7 arcminutes up has to carry into the degree rather than print 60,
 * and that arithmetic was written out twice - once for the printed form and once
 * for the spoken one, which is two places for one rule to be wrong in.
 */
function degreesAndMinutes(degrees: number): [number, number] {
  const whole = Math.floor(degrees);
  const minutes = Math.round((degrees - whole) * 60);
  return minutes === 60 ? [whole + 1, 0] : [whole, minutes];
}

/**
 * A separation from the Sun: `7° 12′`.
 *
 * Arcseconds are dropped. It is compared against an orb quoted in whole degrees,
 * so a third place would be precision the reading has no use for.
 */
export function formatSeparation(degrees: number): string {
  const [whole, minutes] = degreesAndMinutes(degrees);
  return `${whole}° ${pad(minutes)}′`;
}

/** Spoken form for a separation, for assistive technology. */
export function spokenSeparation(degrees: number): string {
  const [whole, minutes] = degreesAndMinutes(degrees);
  return minutes === 0
    ? `${whole} degrees`
    : `${whole} degrees ${minutes} minutes`;
}

/** Spoken form for assistive technology, where `°′″` are not read usefully. */
export function spokenDegrees(parts: [number, number, number]): string {
  const [degrees, minutes, seconds] = parts;
  return `${degrees} degrees ${minutes} minutes ${Math.floor(seconds)} seconds`;
}

/**
 * `−0.0142 °/day`, with a real minus sign.
 *
 * U+2212 rather than a hyphen: at 13px a hyphen is easy to miss, and the sign is
 * the difference between direct and retrograde motion.
 */
export function formatSpeed(speed: number): string {
  const sign = speed < 0 ? "−" : "+";
  return `${sign}${Math.abs(speed).toFixed(4)} °/day`;
}

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
