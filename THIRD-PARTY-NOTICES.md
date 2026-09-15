# Third-party notices

Chandra is distributed under the GNU Affero General Public License v3.0; see
[LICENSE](LICENSE). The components below are redistributed with it and carry
their own notices, reproduced here as their licences require.

---

## Swiss Ephemeris

Planetary and lunar positions, rise and set times, the ascendant and the
ayanamsa are all computed by the Swiss Ephemeris, via the `swiss-eph` crate,
which vendors the Astrodienst C sources. The two ephemeris data files shipped
inside the app (`sepl_18.se1`, `semo_18.se1`) are Astrodienst's.

**Copyright © 1997–2021 Astrodienst AG, Switzerland. All rights reserved.**

Authors of the Swiss Ephemeris: Dieter Koch and Alois Treindl.

Swiss Ephemeris is dual-licensed: under the GNU Affero General Public License,
or under a Swiss Ephemeris Professional License purchased from Astrodienst.
**Chandra is distributed under the AGPL**, and is therefore itself AGPL-3.0,
which is why this repository is under that licence in whole.

The licence conditions distributed with the Swiss Ephemeris sources, reproduced
as the licence requires:

> Swiss Ephemeris is distributed with NO WARRANTY OF ANY KIND. No author or
> distributor accepts any responsibility for the consequences of using it, or
> for whether it serves any particular purpose or works at all, unless he or she
> says so in writing.
>
> The License grants you the right to use, copy, modify and redistribute Swiss
> Ephemeris, but only under certain conditions described in the License. Among
> other things, the License requires that the copyright notices and this notice
> be preserved on all copies.
>
> The authors of Swiss Ephemeris have no control or influence over any of the
> derived works, i.e. over software or services created by other programmers
> which use Swiss Ephemeris functions.
>
> The names of the authors or of the copyright holder (Astrodienst) must not be
> used for promoting any software, product or service which uses or contains the
> Swiss Ephemeris. This copyright notice is the ONLY place where the names of
> the authors can legally appear, except in cases where they have given special
> permission in writing.
>
> The trademarks 'Swiss Ephemeris' and 'Swiss Ephemeris inside' may be used for
> promoting such software, products or services.

<https://www.astro.com/swisseph/>

Chandra is not affiliated with, endorsed by, or supported by Astrodienst AG.

**Note for anyone writing about Chandra:** the clause above is why neither
Astrodienst nor the authors are named anywhere promotional — not on the landing
page's feature copy, not in the App Store style description, not in a release
note. The product name *Swiss Ephemeris* is a trademark the licence explicitly
permits for that purpose, and it is the only part that may be used so. The
copyright notice belongs here and in the app's About pane.

---

## GeoNames

The bundled city and region tables (`crates/geo/data/cities.tsv`,
`admin1.tsv`) are derived from the GeoNames geographical database.

<https://www.geonames.org/>

Licensed under the Creative Commons Attribution 4.0 International licence:
<https://creativecommons.org/licenses/by/4.0/>

Derived by `tools/geonames.sh`, which names every column it keeps. Nothing is
added; columns are dropped and coordinates are rounded to four decimal places.

---

## IANA time zone database

`zone.tab` and `iso3166.tab` are from the IANA time zone database and are in
the public domain.

<https://www.iana.org/time-zones>
