#!/bin/sh
# Numbers and names that must agree across the Rust/TypeScript boundary.
#
# None of these can drift silently in one language and fail in the other: a
# listener registered for an event nobody emits is not an error anywhere, and a
# window sized 320x332 around a panel drawn at some other size just shows a
# clipped corner at one end and a band of desktop at the other. The comments on
# both sides already say "the two share one number"; this is what makes that
# true.
set -eu

bad=0
fail() { echo "check-wire: $*" >&2; bad=1; }

# --- Panel geometry --------------------------------------------------------
# `panel.rs` sizes the window; `panel.css` sizes the panel drawn inside it.
rust_w=$(sed -n 's/^const PANEL_WIDTH: f64 = \([0-9]*\)\.0;/\1/p' src-tauri/src/panel.rs)
rust_h=$(sed -n 's/^const PANEL_HEIGHT: f64 = \([0-9]*\)\.0;/\1/p' src-tauri/src/panel.rs)
css_w=$(awk '/^\.panel-frame \{/,/^\}/' src/styles/panel.css | sed -n 's/.*width: \([0-9]*\)px;.*/\1/p')
css_h=$(awk '/^\.panel-frame \{/,/^\}/' src/styles/panel.css | sed -n 's/.*height: \([0-9]*\)px;.*/\1/p')
[ -n "$rust_w" ] && [ -n "$css_w" ] || fail "could not read the panel width from both sides"
[ "$rust_w" = "$css_w" ] || fail "panel width is $rust_w in panel.rs and $css_w in panel.css"
[ "$rust_h" = "$css_h" ] || fail "panel height is $rust_h in panel.rs and $css_h in panel.css"

# --- Scroller ---------------------------------------------------------------
# The strip translates by MONTH_HEIGHT; the viewport and each month are that
# tall in CSS. A mismatch shows as a month settling short of its own edge.
ts_month=$(sed -n 's/^const MONTH_HEIGHT = \([0-9]*\);/\1/p' src/components/CalendarScroller.tsx)
for block in '^\.scroller \{' '^\.scroller__month \{'; do
  css_month=$(awk "/$block/,/^\}/" src/styles/panel.css | sed -n 's/.*height: \([0-9]*\)px;.*/\1/p')
  [ "$ts_month" = "$css_month" ] ||
    fail "MONTH_HEIGHT is $ts_month and ${block} declares ${css_month:-nothing}"
done

# --- Events -----------------------------------------------------------------
# Every `chandra://` name Rust declares must be listened for, and every name the
# front end listens for must be declared.
emitted=$(grep -ho 'chandra://[a-z]*' src-tauri/src/*.rs | sort -u)
heard=$(grep -rho 'chandra://[a-z]*' src --include='*.ts' --include='*.tsx' | sort -u)
for name in $emitted; do
  echo "$heard" | grep -qx "$name" || fail "$name is emitted by Rust and nothing listens for it"
done
for name in $heard; do
  echo "$emitted" | grep -qx "$name" || fail "$name is listened for and nothing emits it"
done

exit $bad
