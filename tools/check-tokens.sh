#!/bin/sh
# Every custom property a stylesheet reads must be declared somewhere.
#
# A `var(--x)` with no declaration does not fail a build, does not warn, and is
# invisible in the page: the property simply inherits, and the rule quietly
# paints the wrong colour. That is how `--annotate` survived being deleted while
# a rule still read it - the removal was checked with a grep that failed on its
# own shell quoting and reported nothing, which is exactly the failure this
# guards against.
set -eu

styles="src/styles src/dev"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

grep -rhoE 'var\(--[a-z0-9-]+' $styles | sed 's/var(//' | sort -u > "$work/used"

{
  # Declared as a token, or beside the rule that reads it.
  grep -rhoE '^[[:space:]]*--[a-z0-9-]+:' $styles | tr -d ' \t:'
  # Set from script. Matched as a bare string literal, because the call is
  # usually wrapped across lines and a pattern anchored to `setProperty(`
  # would miss it.
  grep -rhoE '"--[a-z0-9-]+"' src --include='*.ts' --include='*.tsx' | tr -d '"'
} | sort -u > "$work/declared"

missing=$(comm -23 "$work/used" "$work/declared")

if [ -n "$missing" ]; then
  echo "custom properties read but never declared:" >&2
  printf '  %s\n' $missing >&2
  exit 1
fi
