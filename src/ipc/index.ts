/**
 * Typed wrappers over the Rust commands.
 *
 * The only place `invoke` is called. Components import these functions, so a
 * command rename is a compile error rather than a runtime one.
 */

import { invoke } from "@tauri-apps/api/core";

import type {
  Bootstrap,
  Chakra,
  City,
  DayDetail,
  GrahaKey,
  GrahaMonth,
  MonthIndex,
  MoonMonth,
  Resolved,
  Settings,
  Snapshot,
} from "./types";

export const bootstrap = () => invoke<Bootstrap>("bootstrap");

/**
 * The month containing `anchorUnixMs`, stepped by `offset` months.
 *
 * `firstWeekday` is the locale's opening weekday, zero-based from Monday. The
 * back end lays the 42 cells out, because only it knows which civil days a lunar
 * month contains; the weekday is the one fact it cannot derive.
 */
export const moonMonth = (
  anchorUnixMs: number,
  offset: number,
  firstWeekday: number,
) => invoke<MoonMonth>("moon_month", { anchorUnixMs, offset, firstWeekday });

export const grahaMonth = (
  graha: GrahaKey,
  anchorUnixMs: number,
  offset: number,
  firstWeekday: number,
) =>
  invoke<GrahaMonth>("graha_month", {
    graha,
    anchorUnixMs,
    offset,
    firstWeekday,
  });

/**
 * The months of the year the cursor lands in, with the offset that reaches each.
 *
 * Fetched when the jump overlay opens rather than with the month, because
 * enumerating a lunar year costs up to fourteen syzygy searches and the overlay
 * is opened deliberately.
 */
export const monthIndex = (
  anchorUnixMs: number,
  offset: number,
  firstWeekday: number,
) => invoke<MonthIndex>("month_index", { anchorUnixMs, offset, firstWeekday });

/**
 * The Lagna Kundali at an instant.
 *
 * The instant is passed rather than read in the back end, so the front end
 * decides how often it wants a new chart and the same moment can be asked for
 * twice.
 */
export const chakra = (unix_ms: number, varga: string) =>
  invoke<Chakra>("chakra", { unixMs: unix_ms, varga });

export const dayDetail = (
  graha: GrahaKey,
  year: number,
  month: number,
  day: number,
) => invoke<DayDetail>("day_detail", { graha, year, month, day });

export const snapshot = (unixMs: number) =>
  invoke<Snapshot>("snapshot", { unixMs });

export const updateSettings = (settings: Settings) =>
  invoke<Bootstrap>("update_settings", { settings });

export const searchCities = (query: string, limit: number) =>
  invoke<City[]>("search_cities", { query, limit });

/** What to call a pair of coordinates typed by hand. A label only: the timezone
 *  is never taken from it (D-031). */
export const nearestCity = (latitude: number, longitude: number) =>
  invoke<City | null>("nearest_city", { latitude, longitude });

export const requestDeviceLocation = () =>
  invoke<Resolved>("request_device_location");

export const closePanel = () => invoke<void>("close_panel");

/** Opens one of a fixed set of pages in the default browser.
 *
 * A name, never a URL: the back end decides what each name means. A command
 * that opened whatever it was handed would turn any scripting flaw in this page
 * into "launch anything". */
export const openLink = (target: "feedback" | "support" | "source" | "licence") =>
  invoke<void>("open_link", { target });
