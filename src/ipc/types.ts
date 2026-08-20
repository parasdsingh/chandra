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

export interface MoonCell {
  day: number;
  illumination: number;
  is_waxing: boolean;
  phase: PhaseKey;
  principal: boolean;
}

export interface MoonMonth {
  year: number;
  month: number;
  time_zone: string;
  leading_blanks: number;
  days: MoonCell[];
  source: Source;
}

export interface GrahaCell {
  day: number;
  longitude: number;
  rashi: string;
  nakshatra: string;
  retrograde: boolean;
  speed: number;
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
  year: number;
  month: number;
  time_zone: string;
  leading_blanks: number;
  days: GrahaCell[];
  events: TransitEvent[];
  source: Source;
}

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

export interface MoonDay {
  kind: "moon";
  date: DateKey;
  phase: PhaseKey;
  principal_at: Moment | null;
  illumination: number;
  is_waxing: boolean;
  moonrise: Moment | null;
  moonset: Moment | null;
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
  subjects: GrahaKey[];
  library_version: string;
  ayanamsas: Choice[];
  node_types: Choice[];
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
