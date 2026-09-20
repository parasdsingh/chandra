#!/bin/sh
# Publish a release: upload the disk image, then point the page at it.
#
# One command, because the two halves must not drift. A page offering a
# download that is not there is worse than a page offering none, and a release
# nobody is told about is not a release - so uploading without updating the page
# and updating the page without uploading are both mistakes this makes
# impossible to make separately.
set -eu

version=${VERSION:?VERSION=x.y.z is required}
bucket=chandra-downloads
key="chandra/Chandra-${version}-universal.dmg"
endpoint="https://chandra-downloads.parasdeep29.workers.dev/download"

# Exactly one, and it must be the one just built.
#
# `ls ... | head -1` takes the alphabetically first, which is not the newest:
# a .dmg left behind by an earlier build would be picked and then uploaded
# under *this* version's name - a 0.1.0 binary published as 0.2.0, with nothing
# anywhere to say so. The bundle directory is expected to hold one file; if it
# holds more, that is a stale artefact and the fix is to say so rather than to
# guess which was meant.
bundle=target/universal-apple-darwin/release/bundle/dmg
count=$(ls "$bundle"/*.dmg 2>/dev/null | wc -l | tr -d ' ')
if [ "$count" = "0" ]; then
  echo "no .dmg built; run make dmg first" >&2
  exit 1
fi
if [ "$count" != "1" ]; then
  echo "$count disk images in $bundle - remove the stale ones:" >&2
  ls -l "$bundle"/*.dmg >&2
  exit 1
fi
dmg=$(ls "$bundle"/*.dmg)

# The version asked for is the version that was built.
#
# `VERSION` is an environment variable and `tauri.conf.json` is the source of
# truth, and nothing compared them. `VERSION=0.2.0 make release` on a tree still
# at 0.1.0 published a correctly-named Chandra-0.2.0-universal.dmg whose About
# pane read 0.1.0 and whose feedback link tagged every report `?v=0.1.0`. The
# name on the bucket would have been the only place the number 0.2.0 existed.
built=$(sed -n 's/^[[:space:]]*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
  src-tauri/tauri.conf.json | head -1)
if [ -z "$built" ]; then
  echo "cannot read the version from src-tauri/tauri.conf.json" >&2
  exit 1
fi
if [ "$built" != "$version" ]; then
  echo "VERSION is $version but the tree is at $built." >&2
  echo "Set the version in src-tauri/tauri.conf.json, rebuild, then release." >&2
  exit 1
fi

# And the app inside the disk image agrees with both. The check above reads the
# file the build was configured from; this reads what the build produced, which
# is what a stranger installs.
app=target/universal-apple-darwin/release/bundle/macos/Chandra.app
if [ ! -f "$app/Contents/Info.plist" ]; then
  echo "no built app at $app; run make dmg first" >&2
  exit 1
fi
bundled=$(/usr/libexec/PlistBuddy -c "Print :CFBundleShortVersionString" \
  "$app/Contents/Info.plist")
if [ "$bundled" != "$version" ]; then
  echo "the built app reports $bundled, not $version - rebuild with 'make dmg'" >&2
  exit 1
fi

# Both architectures, asserted again here rather than trusted from the build
# that produced it - this is the last point before it reaches a stranger.
#
# Through the `archs` target rather than re-implemented. This script had its own
# copy of the same two `case` statements against the same binary, so the check
# and the check existed twice and could have disagreed about what "universal"
# means.
make --no-print-directory archs BIN="$app/Contents/MacOS/chandra"

size=$(( ($(wc -c < "$dmg") + 524288) / 1048576 ))

echo "==> uploading $dmg"
# `--cwd worker`, so wrangler's state directory lands in `worker/.wrangler`,
# which `.gitignore` names. Run from the repository root it created an
# untracked `.wrangler/` beside the source tree on every release.
npx wrangler --cwd worker r2 object put "$bucket/$key" --file="../$dmg" --remote

echo "==> pointing the page at it"
python3 - "$version" "$size" "$endpoint" <<'PY'
import re, sys
version, size, endpoint = sys.argv[1], sys.argv[2], sys.argv[3]
path = "site/index.html"
page = open(path).read()
for key, value in (("downloadUrl", endpoint), ("version", version), ("sizeMb", size)):
    pattern = re.compile(r'(  ' + key + r': ")[^"]*(",)')
    page, count = pattern.subn(r"\g<1>" + value + r"\g<2>", page)
    assert count == 1, f"{key} appears {count} times in {path}"
open(path, "w").write(page)
print(f"   downloadUrl, version {version}, {size} MB")
PY

echo "==> deploying the page"
npx wrangler --cwd worker pages deploy ../site --project-name=chandra --branch=main --commit-dirty=true

echo
echo "Released $version. Commit site/index.html to keep the repo in step."
