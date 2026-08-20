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

export const moonMonth = (year: number, month: number) =>
  invoke<MoonMonth>("moon_month", { year, month });

export const grahaMonth = (graha: GrahaKey, year: number, month: number) =>
  invoke<GrahaMonth>("graha_month", { graha, year, month });

export const dayDetail = (
  graha: GrahaKey,
  year: number,
  month: number,
  day: number,
) => invoke<DayDetail>("day_detail", { graha, year, month, day });

export const snapshot = (unixMs: number) =>
  invoke<Snapshot>("snapshot", { unixMs });

export const ayanamsaDegrees = (unixMs: number) =>
  invoke<number>("ayanamsa_degrees", { unixMs });

export const updateSettings = (settings: Settings) =>
  invoke<Bootstrap>("update_settings", { settings });

export const searchCities = (query: string, limit: number) =>
  invoke<City[]>("search_cities", { query, limit });

export const requestDeviceLocation = () =>
  invoke<Resolved>("request_device_location");

export const openSettings = () => invoke<void>("open_settings");

export const closePanel = () => invoke<void>("close_panel");
