# Research log

Verified facts. Do not re-derive. Each entry states what was checked, how, and the result.

---

## R-01 Ephemeris crate selection

**Question.** Which Rust binding to Swiss Ephemeris is production-viable?

**Candidates surveyed.**

| Crate | Verdict | Reason |
|---|---|---|
| `swiss-eph` 0.2.1 | **Selected** | Vendors SE C source 2.10.03, builds via `cc`, exposes full FFI + a `safe` module. Published 2026-06-21. AGPL-3.0. |
| `libswe-sys` | Rejected | Older, tied to SE 2.09, last meaningful update 2020. |
| `libswisseph-sys` | Rejected | Raw bindgen only, unmaintained. |
| `swisseph` | Rejected | Incomplete higher-level wrapper. |
| `libswe-rs` | Rejected | Partial coverage. |

**Verified by building and running** (`cargo run --release`, macOS 14.4.1, clang 15, rustc 1.89):

- `vendor/swisseph/sweph.h:65` → `#define SE_VERSION "2.10.03"` (current stable).
- Compiles clean from vendored C on macOS arm64. No system libswe needed.
- `src/stubs.c` (libc stubs) is compiled **only** for `wasm32` targets — `build.rs:79-80`. Native builds link real libc, so file I/O and `swe_set_ephe_path` work normally.
- Confirmed working: `swe_calc_ut`, `swe_julday`, `swe_set_sid_mode`, `swe_get_ayanamsa_ut`, `swe_pheno_ut`, `swe_rise_trans`, `swe_set_ephe_path`.

**Caveat found.** Constants have inconsistent Rust types — some `u32`, some `i32`
(`SE_SUN: u32` vs `SE_TRUE_NODE: i32`). Our wrapper normalises all body ids to `i32`
at one boundary so this never leaks into domain code.

---

## R-02 Ephemeris data files

- `swiss-eph-data` 0.1.1 ships `sepl_18.se1` (484 KB) + `semo_18.se1` (1305 KB) = **1.7 MB**.
- Nominal coverage 1800–2399 AD.
- Asteroid file `seas_18.se1` not shipped and not needed — navagrahas only.
- Files are bundled as Tauri resources and located via `swe_set_ephe_path` at startup.
  No runtime extraction, no `include_bytes!` bloat in the binary.

**Coverage probe result — important.** Requesting a date outside file range does **not**
error. Swiss Ephemeris silently falls back to the Moshier analytic theory and signals this
in the *returned* `iflag`, not via a negative return code:

```
1799: "ok"   2401: "ok"    <- both actually served by Moshier fallback
```

**Consequence for design.** Every call must inspect the returned flag and compare it to the
requested flag. The engine reports which theory actually served each result, and the UI
surfaces reduced precision rather than silently lying. See ADR D-006.

---

## R-03 Accuracy measured, not assumed

2026-08-20 00:00 UT, sidereal (Lahiri), SWIEPH vs Moshier:

| Body | SWIEPH (deg) | MOSEPH (deg) | delta (arcsec) |
|---|---:|---:|---:|
| Surya | 122.7865789 | 122.7865814 | 0.009 |
| Chandra | 211.5053135 | 211.5050204 | 1.055 |
| Budha | 114.8012110 | 114.8012180 | 0.025 |
| Shukra | 168.5640194 | 168.5640353 | 0.057 |
| Mangala | 71.4488271 | 71.4488298 | 0.010 |
| Guru | 106.9043814 | 106.9044068 | 0.092 |
| Shani | 350.0280559 | 350.0280523 | 0.013 |
| Rahu (true) | 305.5999286 | 305.6001232 | 0.701 |
| Ketu (true) | 125.5999286 | 125.6001232 | 0.701 |

Moon moves 0.55 arcsec/second of time, so the 1.06 arcsec Moon spread is ~2 seconds of
clock time on a nakshatra boundary. SWIEPH is the authority; Moshier is an acceptable
degraded mode outside 1800–2399.

**Independent correctness check**, same instant, Bengaluru (12.9716 N, 77.5946 E, 920 m):

| Quantity | Computed | Independent expectation | Pass |
|---|---|---|---|
| Sun tropical | 147.018 deg | 27 deg Leo for 20 Aug | yes |
| Lahiri ayanamsa | 24.2291 deg | ~24 deg 13' for 2026 | yes |
| Moon nakshatra | Vishakha pada 4 | 211.5 deg / 13.333 = 15.86 | yes |
| Moon rashi | Vrishchika | 211.5 deg / 30 = 7.05 | yes |
| Tithi | 8 (Shukla Ashtami) | elongation 88.72 deg / 12 | yes |
| Illumination | 49.02% at 91.12 deg | first quarter just past | yes |
| Moonrise / moonset | 12:36 / 00:04 (+1d) IST | Ashtami moon rises ~noon, sets ~midnight | yes |

---

## R-04 Performance baseline

Measured on this machine, release build:

- `swe_calc_ut` with SWIEPH: **7.9 microseconds/call**.
- 2976 Moon calls (31 days on a 15-minute grid): **23.5 ms**.

A brute-force grid is therefore already inside budget; adaptive bracketing plus caching
puts a month payload well under the 30 ms target. No architectural pressure here.

---

## R-05 Tauri v2 menu bar mechanics

- `TrayIcon::set_icon` accepts a runtime `Image` from raw RGBA. Tray icons are
  **live-rendered**, not a static PNG set.
- `TrayIcon::set_icon_with_as_template` exists and sets both atomically. Using the separate
  `set_icon` + `set_icon_as_template` calls causes a visible flicker, because `set_icon`
  resets the template flag. Always use the combined call.
- Multiple independent tray items: `TrayIconBuilder::with_id`, retrieved via
  `AppHandle::tray_by_id`. Each carries its own click handler.
- `tauri-plugin-positioner` with the `tray-icon` cargo feature provides `Position::TrayCenter`.
  It **requires** `tauri_plugin_positioner::on_tray_event()` to be called from the tray event
  handler; without it the window is placed at the top-left of the screen.
- macOS trap: `window.hide()` alone leaves the app in Mission Control and Cmd-Tab.
  `AppHandle::hide()` must also be called, and `AppHandle::show()` on reopen.
- Known upstream bug (tauri#8982, tauri#10912): declaring a tray icon in `tauri.conf.json`
  *and* building one in Rust produces two tray items on macOS. Build in Rust only; declare none
  in config.

---

## R-06 Location on macOS

- `tauri-plugin-geolocation` 2.3.x lists macOS as supported.
- Apple requires an `NSLocationUsageDescription` entry in `Info.plist`.
- Risk: CoreLocation reliability for an ad-hoc-signed app is unproven here and cannot be
  verified without a built bundle. Treated as an unresolved risk, not an assumption.
  Mitigated by the fallback chain in ADR D-007 — the app is fully functional if CoreLocation
  never answers.

---

## R-07 Host toolchain (verified present)

`rustc 1.89.0` · `cargo 1.89.0` · `node v22.23.1` · `npm 10.9.8` · `bun 1.3.11` ·
`clang 15.0.0` · `cmake 4.0.1` · `gh 2.69.0` · `git 2.48.1` ·
macOS 14.4.1 (23E224) · Command Line Tools at `/Library/Developer/CommandLineTools`.

Tauri v2 requires rustc >= 1.77.2 — satisfied.
