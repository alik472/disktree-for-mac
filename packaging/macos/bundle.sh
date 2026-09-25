#!/bin/sh
# Wrap the release binary in disktree.app, ad-hoc signed.
#
#   packaging/macos/bundle.sh [OUT_DIR]     # default: target/release
#
# The icon is committed as disktree.icns; `make_icon.py` redraws it.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
out=${1:-"$root/target/release"}
version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -1)
app="$out/disktree.app"

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$root/target/release/disktree" "$app/Contents/MacOS/disktree"
cp "$here/disktree.icns" "$app/Contents/Resources/disktree.icns"
sed "s/@VERSION@/$version/" "$here/Info.plist.in" >"$app/Contents/Info.plist"
# Ad-hoc: enough for Gatekeeper to run a locally built app; a downloaded one
# still needs a right-click → Open the first time.
codesign --force --deep --sign - "$app"
echo "$app"
