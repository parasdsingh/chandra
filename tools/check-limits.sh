#!/bin/sh
# The feedback bounds are written twice and must agree.
#
# `worker/src/feedback.ts` is the authority; `site/feedback.html` holds the
# same four numbers so a mistake is caught in the browser without a round trip.
# They are in different languages, deployed by different commands, and nothing
# in either build would notice them drifting apart - which shows up as a form
# that accepts something the endpoint then refuses, or the reverse.
set -eu

ts=worker/src/feedback.ts
html=site/feedback.html

reading() {
  # `name: 4000,` from either file, whatever surrounds it.
  sed -n "s/.*[^a-z]$2:[[:space:]]*\([0-9]\{1,\}\).*/\1/p" "$1" | head -1
}

bad=0
for name in message floor version os; do
  a=$(reading "$ts" "$name")
  b=$(reading "$html" "$name")
  if [ -z "$a" ] || [ -z "$b" ]; then
    echo "check-limits: '$name' not found in $ts ($a) or $html ($b)" >&2
    bad=1
  elif [ "$a" != "$b" ]; then
    echo "check-limits: '$name' is $a in $ts and $b in $html" >&2
    bad=1
  fi
done

exit $bad
