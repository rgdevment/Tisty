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

Each drawing fades into a surface of its own tone, so a surface takes the
opposite one: the light drawing on dark surfaces, the dark drawing on light
ones. Files made for one surface are named after it (`on-light`, `on-dark`);
where a single image has to serve both, it is the light drawing.

| File | Size | Where it goes |
| --- | --- | --- |
| `tisty-light.svg` | vector | The light drawing: dark surfaces, and wherever one image serves both |
| `tisty-dark.svg` | vector | The dark drawing: light surfaces |
| `tisty-mac.svg` | vector | The macOS 13–15 app icon: the light drawing on the tile the Dock expects |
| `Tisty.icon` | layers | The macOS 26 app icon, in light, dark and tinted; open it in Icon Composer |
| `tisty-menubar.svg` | vector | The tray and the menu bar, in one ink |
| `tisty-512.png`, `tisty-on-light-512.png` | 512×512 | Press, release pages, the MCP registry |
| `tisty-256.png`, `tisty-on-light-256.png` | 256×256 | The logo at the top of the README and the wiki |
| `tisty-128.png`, `tisty-on-light-128.png` | 128×128 | Small inline use |
| `store-logo-300.png` | 300×300 | Microsoft Store listing logo |

The application's `.ico` (16 to 256, with the 20 and 40 Windows draws at 125 %
and 250 %) and its PNGs are single images, so they wear the light drawing; the
`.icns` wears the macOS tile. On a Mac with Xcode 26 the script also compiles
`Tisty.icon` into `Assets.car`, which macOS 26 prefers to the `.icns`.

`app/src-tauri/icons/msix/` holds what the MSIX packs. The plated logos sit on
the manifest's dark `BackgroundColor`, so they wear the light drawing. The
taskbar icons come at every target size Windows asks for: `altform-unplated` is
what a dark taskbar shows, so it takes the light drawing, and
`altform-lightunplated` takes the dark one. There are no scale-qualified logos,
since `makepri` would split them into resource packs that a single MSIX never
loads.

An installed window swaps between `taskbar/on-dark-*.png` and
`taskbar/on-light-*.png` whenever the taskbar changes, even when only the
taskbar does, taking the size that matches the screen's scale (32 to 64). A
test in `app/src-tauri/src/tray.rs` keeps each one inked against its bar. A
pinned button, Start and Task Manager show the `.ico` instead, which Windows
does not let an app swap.

The tray is the menu-bar glyph: macOS tints its template by itself, and Windows
gets it in white for a dark taskbar and in near-black for a light one.
`app/src/assets/tisty-light.svg` and `tisty-dark.svg` are the copies About shows.

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
