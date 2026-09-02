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

export interface YogaSpan {
  /** 1 to 27. */
  index: number;
  name: string;
  entry: Moment | null;
  exit: Moment | null;
  prevailing: boolean;
  source: Source;
}

export interface KaranaSpan {
  /** 1 to 60, counted from the start of the lunar month. */
  index: number;
  name: string;
  /** One of the four that occur once a month, rather than one of the seven
   * that repeat. Carried so the front end need not know the four by heart. */
  fixed: boolean;
  entry: Moment | null;
  exit: Moment | null;
  prevailing: boolean;
  source: Source;
}

/** Which half of the day-and-night a window divides. */
export type MuhurtaHalf = "day" | "night";

export interface Muhurta {
  name: string;
  half: MuhurtaHalf;
  start: Moment;
  end: Moment;
  /** True for every window except Abhijit and Brahma Muhurta. */
  inauspicious: boolean;
}

/**
 * The limbs of a civil day. Present in both calendars.
 *
 * `yogas`, `karanas` and `muhurtas` are empty when the corresponding setting is
 * off - which is indistinguishable from "none today", and deliberately so: the
 * row is absent either way. `muhurtas` is also empty where the Sun does not rise
 * and set, because a window defined as a fraction of daylight has no meaning on
 * a day with none.
 */
export interface DayPanchanga {
  tithis: TithiSpan[];
  yogas: YogaSpan[];
  karanas: KaranaSpan[];
  muhurtas: Muhurta[];
  sunrise: Moment | null;
  sunset: Moment | null;
  reference: TithiReference;
  vara_name: string;
}

/** A graha's relationship to the rashi it occupies. */
export type Dignity = "exalted" | "debilitated" | "own_sign";

/**
 * Two grahas within a degree of each other.
 *
 * No winner is reported: which graha wins a graha yuddha is decided differently
 * by different authorities, so naming one would be asserting an interpretation.
 */
export interface War {
  with: GrahaKey;
  /** Angular separation in degrees, always positive. */
  separation: number;
}

/** How a graha stands among the nine, at the day's reference instant. */
export interface Standing {
  /** `null` for Rahu and Ketu, which have no agreed dignity table, and for a
   * graha standing in a rashi it has no relationship to. */
  dignity: Dignity | null;
  /** The lord of the nakshatra it occupies - whose ground it stands on. */
  nakshatra_lord: GrahaKey;
  /** Grahas this one casts a full drishti on. */
  aspects: GrahaKey[];
  /** Grahas casting a full drishti on this one. */
  aspected_by: GrahaKey[];
  /** `null` unless this graha is one of the five that can be at war, and is. */
  war: War | null;
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
  /** The same thing in three to five characters, for a 40px calendar cell:
   * `Ari`, `P.Ash`. `null` wherever `target` is. */
  target_short: string | null;
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
  panchanga: DayPanchanga;
  standing: Standing;
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
  panchanga: DayPanchanga;
  standing: Standing;
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
  /**
   * The tithi in force at this instant — `Shukla Ashtami`.
   *
   * At this instant, not at today's sunrise. The panel names a day after the
   * tithi its sunrise fell in; this is what the Moon is doing now, which is the
   * question the menu bar answers.
   */
  tithi: string;
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
  /** Metres above sea level, where the step that resolved this place knew.
   * `null` is not zero: it means nobody supplied a height, and rise and set are
   * then computed at sea level. A place picked from search now carries one,
   * which it could not before E5. */
  elevation: number | null;
}

/**
 * The Lagna Kundali at an instant.
 *
 * One shape for all three chart formats. They differ in where on screen a rashi
 * is drawn and what is written in the compartment; they do not differ in what is
 * true.
 */
export type VargaKey =
  | "d1"
  | "d2"
  | "d3"
  | "d4"
  | "d7"
  | "d9"
  | "d10"
  | "d12"
  | "d16"
  | "d20"
  | "d24"
  | "d27"
  | "d30"
  | "d40"
  | "d45"
  | "d60";

export interface Chakra {
  unix_ms: number;
  varga: VargaKey;
  /** The scheme the division was computed by. Always `Parashari`, and carried
   *  anyway: a D9 computed one way looks exactly like a D9 computed another, so
   *  a chart that cannot say what produced it invites the reader to assume it
   *  matches whatever they last saw elsewhere (D-032). */
  scheme: string;
  /** Where the chart was cast for. Not decoration: the lagna moves a degree
   *  every four minutes, so a chart computed for the wrong city looks exactly
   *  like one computed for the right one. */
  place: string;
  lagna: Lagna;
  /** Twelve, always, in zodiacal order from Mesha. An empty rashi is present
   *  and empty: the chart draws twelve compartments whatever stands in them. */
  rashis: ChakraRashi[];
  source: Source;
}

export interface Lagna {
  rashi: string;
  name: string;
  longitude: number;
  /** Degrees, arcminutes, arcseconds into the rashi. The part that goes stale
   *  fastest — the ascendant moves about a degree every four minutes. */
  degrees_in_rashi: [number, number, number];
}

export interface ChakraRashi {
  rashi: string;
  name: string;
  /** Three letters, for a compartment with no room for `Vrishchika`. */
  short: string;
  /** Counted inclusively from the lagna's rashi, 1 to 12, whole sign. */
  house: number;
  grahas: ChakraGraha[];
}

export interface ChakraGraha {
  graha: GrahaKey;
  /** Two Latin letters — `Su`, `Mo`, `Ma`, `Me`, `Ju`, `Ve`, `Sa`, `Ra`, `Ke`.
   *  Sanskrit does not abbreviate to two: Shukra and Shani are both `Sh`. */
  short: string;
  /** The full name, for the hover. An abbreviation is what a compartment has
   *  room for; it is not what anyone should have to decode. */
  name: string;
  longitude: number;
  degrees_in_rashi: [number, number, number];
  /** Written as brackets around the name: `(Sa)`. */
  retrograde: boolean;
  /** Inside the Sun's rays. Drawn as the warm wash the calendar cell uses. */
  combust: boolean;
  /** `null` where the graha has no relationship to the sign it stands in, and
   *  always for the nodes, which have no agreed dignity table (D-026). */
  dignity: Dignity | null;
}

/** The three chart formats in common use. */
export type ChartFormat = "north" | "south" | "east";

/** Which division a cell names when the subject enters one. */
export type IngressMode = "off" | "rashi" | "nakshatra";

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
    /** Which ingress the grid labels, if either. Never both: the label takes
     * the glyph's place in a 40px cell and there is one glyph. */
    ingress: IngressMode;
  };
  /**
   * The optional limbs of the day view. Only the three that cost something have
   * a switch; dignity, drishti, planetary war and the nakshatra lord are always
   * on because they cost one positions call between them.
   */
  panchanga: {
    yogas: boolean;
    karanas: boolean;
    muhurtas: boolean;
  };
  /** The Lagna Kundali: which format it draws. It has no menu bar switch - it
   *  is the one item that is always there (D-030). */
  chart: {
    format: ChartFormat;
    /** Divisions with their own menu bar item. Several at once: a practitioner
     *  reads D1 and D9 together, and switching between them to compare is not
     *  reading them together. `d1` is always in here — it is the permanent chart
     *  item (D-030). */
    vargas: VargaKey[];
    /** Whether a North Indian compartment carries the sign's number rather than
     *  its name. Both references write a number; a number is a lookup, so the
     *  name is the default here and this is the choice. */
    numbered: boolean;
  };
  tray: {
    /** Calendars with their own menu bar item. Chandra is one of these since
     *  D-030; it is simply the one on by default. */
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
  /** Metres above sea level, and what the observer is built with. Zero where
   * nothing knew - which is the right value to compute with, and the wrong one
   * to print. See `elevation_known`. */
  elevation: number;
  /** Whether `elevation` is a height something actually supplied. `false` means
   * nobody knew and zero is standing in. */
  elevation_known: boolean;
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
  /** Which subject the panel is showing: a graha's key, or `chart` for the
   *  Lagna Kundali, which has its own status item and is not a graha. */
  subject: GrahaKey | "chart";
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
  /** The divisional charts on offer. Served rather than written out here so
   *  nothing can drift from the Rust `Varga`'s own answers. */
  vargas: VargaInfo[];
}

export interface VargaInfo {
  key: VargaKey;
  /** `D9 · Navamsa`, for a list. */
  label: string;
  /** `Navamsa`, for a title that has room for little else. */
  name: string;
  /** The number in the name, which is what the menu bar icon draws. */
  division: number;
}

export interface City {
  zone: string;
  city: string;
  /** The state or province, where the source names one. 1309 of the 34,129
   *  city names are not unique, so this is what tells two Springfields apart.
   *  `null` for an entry that came from the timezone table, which has no
   *  regions in it. */
  region: string | null;
  country: string;
  country_code: string;
  latitude: number;
  longitude: number;
  /** Metres, from a digital elevation model. `null` where the source has none.
   *  Before E5 there was no elevation column at all, which is why every city in
   *  the world reported 0 m (W-07). */
  elevation: number | null;
}

/** Every failure carries a stable code; there is no generic fallback. */
export interface AppError {
  code: "INVALID_DATE" | "NO_CONVERGENCE" | "ENGINE" | "SETTINGS";
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

/**
 * The months of one year, and the cursor offset that reaches each.
 *
 * A year is twelve months in solar mode and twelve or thirteen in lunar mode,
 * so the list is what the back end found rather than a shape the front end
 * assumes. `previous_year` and `next_year` are offsets that land in the
 * neighbouring years, for the same reason: paging must not have to know how
 * many months it is stepping over.
 */
export interface MonthIndex {
  /** `VS 2083`, or `2026`. */
  year: string;
  months: IndexedMonth[];
  previous_year: number;
  next_year: number;
}

export interface IndexedMonth {
  /** The month's own name. The year is above the grid, not in every cell. */
  name: string;
  offset: number;
  adhika: boolean;
}
