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

dmg=$(ls target/universal-apple-darwin/release/bundle/dmg/*.dmg 2>/dev/null | head -1)
[ -n "$dmg" ] || { echo "no .dmg built; run make dmg first" >&2; exit 1; }

# Both architectures, asserted again here rather than trusted from the build
# that produced it - this is the last point before it reaches a stranger.
archs=$(lipo -archs target/universal-apple-darwin/release/bundle/macos/Chandra.app/Contents/MacOS/chandra)
case " $archs " in
  *" x86_64 "*) ;; *) echo "missing x86_64: $archs" >&2; exit 1 ;;
esac
case " $archs " in
  *" arm64 "*) ;; *) echo "missing arm64: $archs" >&2; exit 1 ;;
esac

size=$(( ($(wc -c < "$dmg") + 524288) / 1048576 ))

echo "==> uploading $dmg"
npx wrangler r2 object put "$bucket/$key" --file="$dmg" --remote

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
npx wrangler pages deploy site --project-name=chandra --branch=main --commit-dirty=true

echo
echo "Released $version. Commit site/index.html to keep the repo in step."
