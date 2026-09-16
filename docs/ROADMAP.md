# Roadmap

Milestones are sequential. Each has an exit criterion that must be demonstrable, not asserted.

---

## M0 — Foundations *(blocked on D-013, D-014)*

- Workspace skeleton and Makefile. **CI on Ubuntu was never built** (I-003).
- `crates/ephemeris` wrapping `swiss-eph` behind the `Engine` contract.
- Golden test vectors generated from `swetest` and committed.

**Exit:** `make test` green on Linux; every value in RESEARCH.md R-03 reproduced by a test.

---

## M1 — Menu bar spike *(de-risks the largest unknown)*

- One tray item, live-rendered moon disc, template mode.
- Panel window: TrayCenter positioning, auto-hide on blur, no dock icon, no Cmd-Tab entry.
- Resolves the 22 pt vs 44 pt tray rendering risk before any glyph design work.

**Exit:** icon crisp at both scales in light and dark menu bar; no duplicate tray item;
app absent from Mission Control and Cmd-Tab.

---

## M2 — Moon calendar, v1 scope

- `crates/almanac`: zodiac, phase, roots, day, month, cache.
- `crates/geo`: location chain, tz centroids, city search.
- Month grid + day detail with the six D-010 fields.
- Month switcher, keyboard navigation, settings window.

**Exit:** cold month < 30 ms, warm < 5 ms, measured. Day detail matches a published
panchanga for a spot-check set of dates. Idle CPU 0%.

---

## M3 — Navagrahas

- Per-graha tray items with hand-drawn template glyphs.
- Graha panel: ingress, nakshatra ingress, retrograde stations, combustion markers.
- Day detail: longitude, speed, retrograde flag, rise/set.

**Exit:** a full year of events for all nine grahas verified against an independent
ephemeris; no missed or duplicated stations.

---

## M4 — Polish and ship

- Motion, focus states, empty and degraded states, accessibility pass.
- Ad-hoc signed DMG, `make install`, launch at login.
- Release workflow on tag. **Not built** — there is no CI at all; see I-003 and I-025.

**Exit:** installs and runs on a clean account; survives a location change and a DST boundary.

---

## Deferred — explicitly not in v1

Recorded so they are not silently forgotten, and not built until asked.

- ~~Tithi and vara~~ — shipped in lunar mode only *(D-021)*. Solar mode keeps the D-010 fields
- ~~Yoga and karana~~ *(D-010, argued again in D-021)* — **shipped**, behind switches in
  Settings › Panchanga, in the canonical order: after Nakshatra, before Rashi
- ~~Muhurta: Rahu Kaal, Yamaganda, Gulika, Abhijit, Brahma Muhurta, Durmuhurtam~~ — **shipped**,
  behind its own switch. `crates/almanac/src/muhurta.rs`
- Notifications for new/full moon, sankranti, retrograde stations *(D-017)*
- Aspects, drishti, conjunctions
- Windows build *(the reason for D-001, but not v1 work)*
- Signing and notarisation *(D-012)*
