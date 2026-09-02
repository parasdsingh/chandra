#!/bin/sh
# Rebuilds the bundled city list from GeoNames.
#
#     sh tools/geonames.sh
#
# The result is committed, so this runs only when the list is to be refreshed.
# It is here so the data in the repository is reproducible rather than a blob
# somebody once downloaded: every column kept is named below, and anyone can
# re-run it and diff.
#
# Source: https://download.geonames.org/export/dump/
# Licence: CC BY 4.0. The attribution this obliges is in the app's About pane
# and in data/NOTICE.
# `pipefail` matters here specifically: every write below is `awk ... | sort >
# file`, and `sort` exits 0 on empty input. Without it, awk dying leaves the
# committed 2.3 MB table truncated to nothing and the script reports success.
set -eu
if (set -o pipefail) 2>/dev/null; then set -o pipefail; fi

out="crates/geo/data"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

echo "==> downloading"
# `--fail` so an HTTP error page is an error rather than a file. Without it the
# body flows into awk and overwrites the tables with nothing.
curl -sSL --fail -o "$work/cities.zip" https://download.geonames.org/export/dump/cities15000.zip
curl -sSL --fail -o "$work/admin1.txt" https://download.geonames.org/export/dump/admin1CodesASCII.txt
unzip -o -q "$work/cities.zip" -d "$work"

# cities15000: every place with more than 15000 inhabitants. The columns kept,
# by their position in the GeoNames export:
#
#   2  name         as written locally, in its own script. What is displayed
#   3  asciiname    the same name with the diacritics stripped. What is
#                   searched, alongside the local form: a reader who types
#                   "Cesky Tesin" or "Zurich" is looking for Cesky Tesin and
#                   Zurich, and matching only the local spelling finds neither
#   9  country code ISO 3166-1 alpha-2, joined to iso3166.tab for the name
#   11 admin1 code  the region, joined to admin1.tsv below. 1309 city names in
#                   this file are not unique, so a region is what tells two
#                   Springfields apart
#   18 timezone     IANA. Taken from here rather than from the coordinates: a
#                   zone boundary is political and cannot be derived from a
#                   point
#   5,6 latitude, longitude, to four decimal places - about 11 metres, which is
#                   two orders finer than the arcminute grid this replaces and
#                   far finer than the lagna can tell apart
#   15 population   not displayed. It ranks the results, so a search for "york"
#                   offers New York before York, Nebraska
#   17 dem          elevation in metres from a digital elevation model. The old
#                   list had no elevation at all, which is why every city
#                   reported 0 m (W-07)
echo "==> trimming cities"
# `LC_ALL=C` on awk as well as on sort. `printf "%.4f"` follows the locale's
# decimal separator, so under a comma-decimal locale this would write `36,9101`
# and the parser would drop every row.
#
# Written to a temporary file and moved into place, so a failure leaves the
# committed table as it was rather than half of one.
LC_ALL=C awk -F'\t' 'NF >= 18 { printf "%s\t%s\t%s\t%s\t%s\t%.4f\t%.4f\t%s\t%s\n", $2, $3, $9, $11, $18, $5, $6, $15, $17 }' \
  "$work/cities15000.txt" | LC_ALL=C sort > "$work/cities.tsv"

# admin1: the region code to its name. Only the two columns are kept.
echo "==> trimming regions"
LC_ALL=C awk -F'\t' 'NF >= 2 { printf "%s\t%s\n", $1, $2 }' "$work/admin1.txt" \
  | LC_ALL=C sort > "$work/admin1.tsv"

# Both parsed before either is committed.
test -s "$work/cities.tsv" || { echo "cities.tsv came out empty" >&2; exit 1; }
test -s "$work/admin1.tsv" || { echo "admin1.tsv came out empty" >&2; exit 1; }
mv "$work/cities.tsv" "$out/cities.tsv"
mv "$work/admin1.tsv" "$out/admin1.tsv"

# GeoNames publishes a new dump daily, so these counts move. Several comments
# and one decision record quote them; when this number changes, they are what to
# check.
printf "==> %s cities, %s regions\n" \
  "$(wc -l < "$out/cities.tsv" | tr -d ' ')" \
  "$(wc -l < "$out/admin1.tsv" | tr -d ' ')"
