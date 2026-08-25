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

/** Half a lunar month: waxing or waning. */
export type Paksha = "shukla" | "krishna";

/** Which instant a day's tithi was taken at. */
export type TithiReference = "sunrise" | "local_noon";

/** A civil day's place in a tithi that spans two sunrises. */
export type Vriddhi = "first" | "second";

/** A tithi no civil day is named after, because it held no sunrise. */
export interface SkippedTithi {
  index: number;
  paksha: Paksha;
  number: number;
  /** The full form: `Shukla Shashthi`. */
  name: string;
}

/**
 * The lunar day a grid cell is named after. Absent in solar mode.
 *
 * `number` is what the cell prints, because amanta and purnimanta count from
 * opposite ends; `index` is the astronomical 1-30 and is only for equality.
 */
export interface CellTithi {
  index: number;
  number: number;
  paksha: Paksha;
  /** `Ashtami`, `Purnima`, `Amavasya`. */
  name: string;
  reference: TithiReference;
  sunrise: Moment | null;
  kshaya: SkippedTithi[];
  vriddhi: Vriddhi | null;
}

/** A tithi touching one civil day, with its true boundaries. */
export interface TithiSpan {
  index: number;
  number: number;
  paksha: Paksha;
  name: string;
  /** `null` only when the boundary did not resolve. No time is ever guessed. */
  entry: Moment | null;
  exit: Moment | null;
  /**
   * Sunrises inside the span: 0 is a kshaya, 2 a vriddhi.
   *
   * `null` where there is no count to give: a boundary that did not resolve, or
   * a latitude where the Sun rose on none of the three days. Neither is a
   * kshaya.
   */
  sunrises: number | null;
  prevailing: boolean;
  source: Source;
}

/** The panchanga limbs a lunar calendar needs. Absent in solar mode. */
export interface DayPanchanga {
  tithis: TithiSpan[];
  sunrise: Moment | null;
  reference: TithiReference;
  vara_name: string;
}

export interface MoonCell {
  /** Full civil date: a lunar month crosses Gregorian month boundaries. */
  date: DateKey;
  /** False for the grid's leading and trailing cells. */
  in_month: boolean;
  tithi: CellTithi | null;
  illumination: number;
  /**
   * Read at the day's start, where the phase name is decided; `illumination` is
   * read at local noon. The two can differ on the day of a new or full moon.
   */
  is_waxing: boolean;
  /** A principal name appears only on the day that phase actually occurs. */
  phase: PhaseKey;
  /** Within the Sun's rays; for the Moon that is the days around new moon. */
  combust: boolean;
}

export interface MoonMonth {
  /** `August 2026`, `Shravana 2083`, `Adhika Shravana 2080`. */
  label: string;
  /** The name alone, so the header can set `Adhika` apart from it. */
  name: string;
  /** Intercalary. The header reads this rather than searching the label. */
  adhika: boolean;
  /** An instant inside this month; navigation steps from it. */
  anchor_unix_ms: number;
  /** Exactly 42 cells, in reading order, laid out by the back end. */
  days: MoonCell[];
  source: Source;
}

export interface GrahaCell {
  date: DateKey;
  in_month: boolean;
  tithi: CellTithi | null;
  /** Sidereal longitude at local noon, the instant combustion is judged at. */
  longitude: number;
  retrograde: boolean;
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
  name: string;
  adhika: boolean;
  anchor_unix_ms: number;
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
  illumination: number;
  is_waxing: boolean;
  moonrise: Moment | null;
  moonset: Moment | null;
  combustion: Combustion;
  panchanga: DayPanchanga | null;
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
  panchanga: DayPanchanga | null;
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
  location: {
    mode: LocationMode;
    place: PlaceSetting | null;
    /**
     * Metres above sea level, applied on top of whichever step of the chain
     * resolved the location. `null` leaves that step's own elevation alone.
     */
    elevation: number | null;
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
  appearance: {
    /** Multiplier on every dimension of the panel, 0.8 to 1.4. */
    scale: number;
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
  /** `x y width height`, centred on the glyph's ink rather than on the grid. */
  view_box: [number, number, number, number];
}

export interface Bootstrap {
  settings: Settings;
  location: Resolved;
  /** Which subject the panel is showing; the tray sets it before opening. */
  subject: GrahaKey;
  subjects: GrahaKey[];
  /**
   * Whether the system's popover material is behind the panel. The panel paints
   * a scrim over that material, so where it is absent there is nothing to
   * darken and the panel must paint an opaque ground itself.
   */
  panel_material: boolean;
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
