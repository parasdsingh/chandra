/**
 * Typed wrappers over the Rust commands.
 *
 * The only place `invoke` is called. Components import these functions, so a
 * command rename is a compile error rather than a runtime one.
 */

import { invoke } from "@tauri-apps/api/core";

import type {
  Bootstrap,
  City,
  DayDetail,
  GrahaKey,
  GrahaMonth,
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

export const requestDeviceLocation = () =>
  invoke<Resolved>("request_device_location");

export const closePanel = () => invoke<void>("close_panel");
