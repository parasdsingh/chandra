# Chandra

A jyotisha calendar that lives in the macOS menu bar. The Moon's phase at a
glance, a panchanga for any day, transit calendars for all nine grahas, and a
Lagna Kundali with the sixteen divisional charts.

Everything is computed on the machine with the Swiss Ephemeris. There is no
account, no telemetry, and no network code in the app at all.

## What it does

- **The Moon in the menu bar.** The current phase drawn in the status item
  itself, and a month grid beneath it giving every day its phase.
- **Nine calendars.** A month of transits for any graha — rashi and nakshatra
  ingresses, stations, combustion — each switchable into the menu bar on its own.
- **A full panchanga.** Tithi, vara, nakshatra with pada, yoga, karana and the
  muhurtas, for the day you pick. Amanta, purnimanta, or a plain solar month.
- **The Lagna Kundali.** North, South or East Indian, in any of the sixteen
  divisions of the Shodasavarga. Each graha stands at its own degree along a
  route through its house, and the ring advances as the lagna crosses its sign.

## Building

Requires Rust, Node, and Xcode command line tools.

```
make dev        run against the Vite dev server
make check      lint and the full test suite
make install    build for this Mac and install to /Applications
make dmg        build a universal .dmg for distribution
```

`make install` is native to the machine it runs on and ad-hoc signs the
installed copy. `make dmg` builds for both Apple Silicon and Intel and asserts
both architectures are present before it finishes.

**The distributable is not signed with an Apple Developer ID and is not
notarised.** macOS refuses the first launch; on macOS 15 and later that has to
be cleared through System Settings › Privacy & Security. See
[I-051](docs/ISSUES.md).

A visual harness renders every view against real almanac output at
`?preview` in a dev build, and `?preview&shots` renders the release
screenshots.

## Documentation

Design and decisions live in the repository, not in chat.

- [Architecture](docs/ARCHITECTURE.md) — crates, IPC, data flow, threading
- [Decisions](docs/DECISIONS.md) — one record per decision, with what it cost
- [Research log](docs/RESEARCH.md) — verified facts, do not re-derive
- [Roadmap](docs/ROADMAP.md) and [Issues](docs/ISSUES.md)
- [Vargas](docs/design/vargas.md) — the sixteen divisions, cited to sources
- [Kundali](docs/design/kundali.md) and [Traversal](docs/design/traversal.md)

## Ephemeris and defaults

Swiss Ephemeris 2.10.03. Sidereal zodiac, Lahiri ayanamsa, **mean** node,
whole-sign houses. Ayanamsa and node type are switchable
([D-003](docs/DECISIONS.md), [D-027](docs/DECISIONS.md)).

Full precision runs 1800–2399. Outside that the ephemeris falls back to an
analytic model, and every view that shows a figure from it says so.

## Licence

[AGPL-3.0](LICENSE).

Chandra bundles the [Swiss Ephemeris](https://www.astro.com/swisseph/),
© 1997–2021 Astrodienst AG, which is dual-licensed AGPL or commercial; this
distribution takes the AGPL, which is why the whole project is under it. City
data is from [GeoNames](https://www.geonames.org/) under CC BY 4.0. Full
notices, and the conditions Astrodienst requires be preserved, are in
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) and ship inside the app.

Chandra is not affiliated with Astrodienst AG.
