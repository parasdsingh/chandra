/**
 * The IPC contract.
 *
 * These mirror the Rust types the commands return. Drift between the two is
 * caught by `src-tauri/tests/contract.rs`, which serialises a value of every
 * payload type and compares the resulting key set against a committed fixture:
 * a change on the Rust side fails that test and the diff names exactly what to
 * change here.
 */

export type GrahaKey =
  | "surya"
  | "chandra"
  | "mangala"
  | "budha"
  | "guru"
  | "shukra"
  | "shani"
  | "rahu"
  | "ketu";

export type PhaseKey =
  | "new_moon"
  | "waxing_crescent"
  | "first_quarter"
  | "waxing_gibbous"
  | "full_moon"
  | "waning_gibbous"
  | "last_quarter"
  | "waning_crescent";

/** Which ephemeris actually produced a value (docs/DECISIONS.md D-006). */
export type Source = "swieph" | "moshier";

export type EventKind =
  | "rashi_ingress"
  | "nakshatra_ingress"
  | "retrograde_station"
  | "direct_station"
  | "combustion_start"
  | "combustion_end";

export interface DateKey {
  year: number;
  month: number;
  day: number;
}

/**
 * An instant. `unix_ms` is formatted by the front end against the view's zone,
 * so presentation stays in the presentation layer. `day_offset` is how many
 * civil days away from the day being displayed it falls, which is what lets a
 * moonset after midnight read as belonging to this row.
 */
export interface Moment {
  unix_ms: number;
  day_offset: number;
}

/** Gregorian, new moon to new moon, or full moon to full moon. */
export type MonthSystem = "solar" | "amanta" | "purnimanta";

export interface MoonCell {
  /** Full civil date: a lunar month crosses Gregorian month boundaries. */
  date: DateKey;
  illumination: number;
  is_waxing: boolean;
  phase: PhaseKey;
  principal: boolean;
  /** Within the Sun's rays; for the Moon that is the days around new moon. */
  combust: boolean;
}

export interface MoonMonth {
  /** `August 2026`, `Shravana 2026`, `Adhika Shravana 2026`. */
  label: string;
  system: MonthSystem;
  /** An instant inside this month; navigation steps from it. */
  anchor_unix_ms: number;
  time_zone: string;
  days: MoonCell[];
  source: Source;
}

export interface GrahaCell {
  date: DateKey;
  longitude: number;
  rashi: string;
  nakshatra: string;
  retrograde: boolean;
  speed: number;
  combust: boolean;
}

export interface TransitEvent {
  kind: EventKind;
  graha: GrahaKey;
  date: DateKey;
  at: Moment;
  target: string | null;
  source: Source;
}

export interface GrahaMonth {
  graha: GrahaKey;
  label: string;
  system: MonthSystem;
  anchor_unix_ms: number;
  time_zone: string;
  days: GrahaCell[];
  events: TransitEvent[];
  source: Source;
}

export type CalendarMonth = MoonMonth | GrahaMonth;

export interface NakshatraSpan {
  nakshatra: string;
  name: string;
  lord: GrahaKey;
  pada: number;
  entry: Moment;
  exit: Moment;
  prevailing: boolean;
}

export interface RashiSpan {
  rashi: string;
  name: string;
  western: string;
  entry: Moment;
  exit: Moment;
  prevailing: boolean;
}

/**
 * Where a subject stands relative to the Sun, judged at local noon.
 *
 * `orb` is absent for the Sun and the nodes, which have no combustion at all;
 * `combust` is then always false and the day view says nothing about it.
 */
export interface Combustion {
  /** Angular distance from the Sun, 0 to 180 degrees. */
  separation: number;
  orb: number | null;
  combust: boolean;
}

export interface MoonDay {
  kind: "moon";
  date: DateKey;
  phase: PhaseKey;
  principal_at: Moment | null;
  illumination: number;
  is_waxing: boolean;
  moonrise: Moment | null;
  moonset: Moment | null;
  combustion: Combustion;
  nakshatras: NakshatraSpan[];
  rashis: RashiSpan[];
  source: Source;
}

export interface GrahaDay {
  kind: "graha";
  date: DateKey;
  graha: GrahaKey;
  longitude: number;
  /** Degrees, arcminutes, arcseconds within the containing rashi. */
  degrees_in_rashi: [number, number, number];
  speed: number;
  retrograde: boolean;
  rise: Moment | null;
  set: Moment | null;
  combustion: Combustion;
  nakshatras: NakshatraSpan[];
  rashis: RashiSpan[];
  source: Source;
}

export type DayDetail = MoonDay | GrahaDay;

export interface SnapshotGraha {
  graha: GrahaKey;
  longitude: number;
  rashi: string;
  nakshatra: string;
  retrograde: boolean;
  source: Source;
}

export interface Snapshot {
  unix_ms: number;
  illumination: number;
  is_waxing: boolean;
  phase: PhaseKey;
  grahas: SnapshotGraha[];
  source: Source;
}

export type TimeFormat = "system" | "hour12" | "hour24";
export type LocationMode = "automatic" | "manual";
export type Provenance = "manual" | "core_location" | "time_zone";

export interface PlaceSetting {
  label: string;
  zone: string;
  latitude: number;
  longitude: number;
  elevation: number;
}

export interface Settings {
  schema_version: number;
  launch_at_login: boolean;
  time_format: TimeFormat;
  location: {
    mode: LocationMode;
    place: PlaceSetting | null;
  };
  sidereal: {
    ayanamsa: string;
    node_type: string;
  };
  calendar: {
    month_system: MonthSystem;
  };
  tray: {
    subjects: GrahaKey[];
    colour_mode: boolean;
  };
}

export interface Resolved {
  label: string;
  zone: string;
  latitude: number;
  longitude: number;
  elevation: number;
  provenance: Provenance;
}

export interface Choice {
  key: string;
  label: string;
}

export interface GrahaInfo {
  key: GrahaKey;
  name: string;
  english: string;
  /** SVG path data on a 24 x 24 grid, served by the backend from chandra-glyph. */
  path: string;
  filled: boolean;
  stroke_width: number;
}

export interface Bootstrap {
  settings: Settings;
  location: Resolved;
  /** Which subject the panel is showing; the tray sets it before opening. */
  subject: GrahaKey;
  subjects: GrahaKey[];
  library_version: string;
  ayanamsas: Choice[];
  node_types: Choice[];
  month_systems: Choice[];
  grahas: GrahaInfo[];
}

export interface City {
  zone: string;
  city: string;
  country: string;
  country_code: string;
  latitude: number;
  longitude: number;
}

/** Every failure carries a stable code; there is no generic fallback. */
export interface AppError {
  code:
    | "DATE_OUT_OF_RANGE"
    | "NO_CONVERGENCE"
    | "ENGINE"
    | "SETTINGS";
  message: string;
}

export function isAppError(value: unknown): value is AppError {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    typeof (value as AppError).code === "string"
  );
}
