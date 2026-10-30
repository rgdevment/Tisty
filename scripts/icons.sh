#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

PATH="$PWD/app/node_modules/.bin:$PATH"
src=assets/icons
icons=app/src-tauri/icons
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

render() {
  tauri icon "$1" -o "$2" ${3:+-p "$3"} >/dev/null 2>&1
}

render "$src/tisty-light.svg" "$work/light"
render "$src/tisty-mac.svg" "$work/mac"
render "$src/tisty-light.svg" "$work/sizes" 16,20,24,30,32,36,40,48,64,96,128,256,300,512

# The light drawing serves every surface, dark ones too: the dark drawing fades into them.
# Only macOS takes the tile, since its Dock expects one.
for file in "$work"/light/*.png "$work"/light/icon.ico; do
  cp "$file" "$icons/"
done
cp "$work/mac/icon.icns" "$icons/"

# macOS 26 draws only an Icon Composer icon without a grey frame, and only actool on a Mac compiles one.
if xcrun --find actool >/dev/null 2>&1; then
  xcrun actool "$PWD/$src/Tisty.icon" --compile "$work" --app-icon Tisty \
    --platform macosx --target-device mac --minimum-deployment-target 13.3 \
    --include-all-app-icons --enable-on-demand-resources NO --development-region en \
    --output-partial-info-plist "$work/icon.plist" \
    --output-format human-readable-text --errors --warnings </dev/null
  cp "$work/Assets.car" "$icons/"
elif [ "$(uname)" = Darwin ]; then
  echo "actool comes with Xcode, not the Command Line Tools: Assets.car was left as it was" >&2
fi

# MSIX: without a plate, the taskbar shows the mark as it is drawn.
for size in 16 20 24 30 32 36 40 48 64 96 256; do
  cp "$work/sizes/${size}x${size}.png" "$icons/Square44x44Logo.targetsize-${size}_altform-unplated.png"
done

# The tray is the one-ink glyph: macOS tints its template itself, Windows needs one per taskbar.
render "$src/tisty-menubar.svg" "$work/menubar" 18,36
cp "$work/menubar/18x18.png" "$icons/tray/tray-macos/menubarTemplate.png"
cp "$work/menubar/36x36.png" "$icons/tray/tray-macos/menubarTemplate@2x.png"
sed 's/fill="#ffffff"/fill="#1c1c1c"/' "$src/tisty-menubar.svg" >"$work/menubar-ink.svg"
render "$src/tisty-menubar.svg" "$work/on-dark" 16,20,24,32,40,48
render "$work/menubar-ink.svg" "$work/on-light" 16,20,24,32,40,48
for size in 16 20 24 32 40 48; do
  cp "$work/on-dark/${size}x${size}.png" "$icons/tray/tray-windows/on-dark-$size.png"
  cp "$work/on-light/${size}x${size}.png" "$icons/tray/tray-windows/on-light-$size.png"
done

for size in 128 256 512; do
  cp "$work/sizes/${size}x${size}.png" "$src/tisty-$size.png"
done
cp "$work/sizes/300x300.png" "$src/store-logo-300.png"
cp "$work/sizes/128x128.png" app/public/icon.png
cp "$src/tisty-light.svg" app/src/assets/
