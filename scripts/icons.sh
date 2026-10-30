#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

PATH="$PWD/app/node_modules/.bin:$PATH"
src=assets/icons
icons=app/src-tauri/icons
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

render() {
  tauri icon "$1" -o "$2" ${3:+-p "$3"} >"$work/render.log" 2>&1 || {
    cat "$work/render.log" >&2
    return 1
  }
}

# Every size a surface asks for: the bundle, the .ico, the MSIX logos and target sizes, the READMEs.
sizes=16,20,24,30,32,36,40,44,48,50,60,64,71,72,80,96,128,150,256,300,512
render "$src/tisty-light.svg" "$work/light" "$sizes"
render "$src/tisty-dark.svg" "$work/dark" "$sizes"
render "$src/tisty-mac.svg" "$work/mac"

# The bundle's images are single, so they wear the light drawing; macOS takes the tile its Dock expects.
for size in 32 64 128; do
  cp "$work/light/${size}x${size}.png" "$icons/"
done
cp "$work/light/256x256.png" "$icons/128x128@2x.png"
cp "$work/light/512x512.png" "$icons/icon.png"
cp "$work/mac/icon.icns" "$icons/"

# tauri writes the .icns chunks in hash order, so they are sorted to keep an unchanged icon unchanged.
node - "$icons/icon.icns" <<'EOF_ICNS'
const { readFileSync, writeFileSync } = require("node:fs");
const [file] = process.argv.slice(2);
const icns = readFileSync(file);
const chunks = [];
for (let at = 8; at < icns.length; at += icns.readUInt32BE(at + 4)) {
  chunks.push(icns.subarray(at, at + icns.readUInt32BE(at + 4)));
}
chunks.sort((a, b) => a.subarray(0, 4).toString().localeCompare(b.subarray(0, 4).toString()));
writeFileSync(file, Buffer.concat([icns.subarray(0, 8), ...chunks]));
EOF_ICNS

# tauri's .ico stops short of 20 and 40, the sizes Windows draws at 125 % and 250 %.
node - "$icons/icon.ico" "$work"/light/{16x16,20x20,24x24,32x32,40x40,48x48,64x64,256x256}.png <<'EOF_ICO'
const { readFileSync, writeFileSync } = require("node:fs");
const [out, ...files] = process.argv.slice(2);
const images = files.map((file) => readFileSync(file));
const head = Buffer.alloc(6 + 16 * images.length);
head.writeUInt16LE(1, 2);
head.writeUInt16LE(images.length, 4);
let offset = head.length;
images.forEach((png, at) => {
  const side = png.readUInt32BE(16);
  const entry = 6 + 16 * at;
  head.writeUInt8(side % 256, entry);
  head.writeUInt8(side % 256, entry + 1);
  head.writeUInt16LE(1, entry + 4);
  head.writeUInt16LE(32, entry + 6);
  head.writeUInt32LE(png.length, entry + 8);
  head.writeUInt32LE(offset, entry + 12);
  offset += png.length;
});
writeFileSync(out, Buffer.concat([head, ...images]));
EOF_ICO

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

# Each drawing fades into its own tone, so surfaces take the opposite: unplated is a dark taskbar.
msix="$icons/msix"
rm -rf "$msix"
mkdir -p "$msix"
# No scale-qualified logos: makepri would split them into resource packs that a single MSIX never loads.
for logo in Square44x44Logo:44 Square71x71Logo:71 Square150x150Logo:150 StoreLogo:50; do
  cp "$work/light/${logo#*:}x${logo#*:}.png" "$msix/${logo%:*}.png"
done
for size in 16 20 24 30 32 36 40 48 60 64 72 80 96 256; do
  cp "$work/light/${size}x${size}.png" "$msix/Square44x44Logo.targetsize-$size.png"
  cp "$work/light/${size}x${size}.png" "$msix/Square44x44Logo.targetsize-${size}_altform-unplated.png"
  cp "$work/dark/${size}x${size}.png" "$msix/Square44x44Logo.targetsize-${size}_altform-lightunplated.png"
done

# An installed (non-MSIX) window swaps its icon as the taskbar changes, at its size from 100 % to 200 %.
rm -rf "$icons/taskbar"
mkdir -p "$icons/taskbar"
for size in 32 40 48 64; do
  cp "$work/light/${size}x${size}.png" "$icons/taskbar/on-dark-$size.png"
  cp "$work/dark/${size}x${size}.png" "$icons/taskbar/on-light-$size.png"
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
  cp "$work/light/${size}x${size}.png" "$src/tisty-$size.png"
  cp "$work/dark/${size}x${size}.png" "$src/tisty-on-light-$size.png"
done
cp "$work/light/300x300.png" "$src/store-logo-300.png"
cp "$work/light/128x128.png" app/public/icon.png
cp "$src/tisty-light.svg" "$src/tisty-dark.svg" app/src/assets/
