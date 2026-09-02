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
set -eu

out="crates/geo/data"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

echo "==> downloading"
curl -sSL -o "$work/cities.zip" https://download.geonames.org/export/dump/cities15000.zip
curl -sSL -o "$work/admin1.txt" https://download.geonames.org/export/dump/admin1CodesASCII.txt
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
awk -F'\t' 'NF >= 18 { printf "%s\t%s\t%s\t%s\t%s\t%.4f\t%.4f\t%s\t%s\n", $2, $3, $9, $11, $18, $5, $6, $15, $17 }' \
  "$work/cities15000.txt" | LC_ALL=C sort > "$out/cities.tsv"

# admin1: the region code to its name. Only the two columns are kept.
echo "==> trimming regions"
awk -F'\t' 'NF >= 2 { printf "%s\t%s\n", $1, $2 }' "$work/admin1.txt" \
  | LC_ALL=C sort > "$out/admin1.tsv"

printf "==> %s cities, %s regions\n" \
  "$(wc -l < "$out/cities.tsv" | tr -d ' ')" \
  "$(wc -l < "$out/admin1.tsv" | tr -d ' ')"
