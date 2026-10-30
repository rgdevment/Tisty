# Assets

What is drawn for Tisty and used outside the build: the mark, the mascot and
the Microsoft Store images. What the application itself ships with lives next to
the code that reads it, in `app/src-tauri/icons/` and `app/src/assets/`.

| Folder | What it holds | Where it goes |
| --- | --- | --- |
| `icons/` | The mark, from its SVG master | README, Store listing, press |
| `mascot/` | Tisto, Tisty's beaver, in colour and in one ink | Sources of the drawings the app and the Store use |
| `store/` | The Microsoft Store screenshots, one set per language | The Store listing, the READMEs and the wiki |

## Icons

The SVGs are the source of truth, and `scripts/icons.sh` renders every PNG,
`.ico` and `.icns` from them — the ones here and the ones the build reads in
`app/src-tauri/icons/`. Change an SVG and run it again rather than editing a PNG
by hand.

The light drawing is the mark everywhere: it reads on light and dark surfaces
alike, while the dark one fades into dark surfaces and weighs on light ones. Its
border is a shade darker than delivered, so it still holds on white at small
sizes. `originals/` keeps the four SVGs exactly as the designer delivered them.

| File | Size | Where it goes |
| --- | --- | --- |
| `tisty-light.svg` | vector | The mark, everywhere it is shown |
| `tisty-mac.svg` | vector | The macOS 13–15 app icon: the mark on the tile the Dock expects |
| `Tisty.icon` | layers | The macOS 26 app icon, in light, dark and tinted; open it in Icon Composer |
| `tisty-menubar.svg` | vector | The tray and the menu bar, in one ink |
| `originals/` | vector | The light, dark, macOS and menu-bar SVGs as delivered |
| `tisty-512.png` | 512×512 | Press, release pages, the MCP registry |
| `tisty-256.png` | 256×256 | The logo at the top of the README and the wiki |
| `tisty-128.png` | 128×128 | Small inline use |
| `store-logo-300.png` | 300×300 | Microsoft Store listing logo |

The application's `.ico` (16 to 256, with the 20 and 40 Windows draws at 125 %
and 250 %), its PNGs and the MSIX logos wear the light drawing; the `.icns`
wears the macOS tile. On a Mac with Xcode 26 the script also compiles
`Tisty.icon` into `Assets.car`, which macOS 26 prefers to the `.icns`.

`app/src-tauri/icons/msix/` holds what the MSIX packs, at every target size
Windows asks for. There are no scale-qualified logos, since `makepri` would
split them into resource packs that a single MSIX never loads.

The tray and the menu bar take the one-ink glyph instead. macOS tints its
template like every other menu-bar icon; Windows gets it in white for a dark
taskbar and in near-black for a light one, following the taskbar even when only
it changes. `app/src/assets/tisty-light.svg` is the copy About shows.

## Mascot

The beaver is called Tisto. `mascot/tisty-head.svg` is the colour drawing;
`app/src/assets/tisty.svg` is the copy the window shows in the news
after an update.
`mascot/tisty-head-mono.svg` is the same head in one ink, for places that print
a single colour. `mascot/tisty-head-left.svg` and `mascot/tisty-head-mono-left.svg`
are the same two turned to look left; the window draws that one at the end of
the tasks bar, so Tisto looks at what is being written.

## Store images

`store/` holds one screenshot per feature and language, named
`<language>-<order>-<subject>.png`: `en-` in English and `es-` in Spanish, with
the same order in both. The READMEs show them from this folder, and the wiki
links to them on `main`, so renaming one means updating the wiki in the same
change.
