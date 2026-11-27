# Assets

What is drawn for Tisty and used outside the build: the mark, the mascot and
the screens that present it. What the application itself ships with lives next
to the code that reads it, in `app/src-tauri/icons/` and `app/src/assets/`.

| Folder | What it holds | Where it goes |
| --- | --- | --- |
| `icons/` | The mark, from its SVG master | README, Store listing, press |
| `mascot/` | Tisto, Tisty's beaver, in colour and in one ink | Sources of the drawings the app and the Store use |
| `screens/` | The screens that present Tisty, one set per language | The Store listing, the READMEs, the help and tisty.app |
| `help/` | Real captures of the app, taken in a clean machine, `es-` and `en-` | The pages of the help that show a step the guide does not |

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
| `tisty-mac.svg` | vector | The macOS 13–15 app icon: the mark on a dark warm tile |
| `Tisty.icon` | layers | The macOS 26 app icon, the same dark tile in every appearance; open it in Icon Composer |
| `tisty-menubar.svg` | vector | The tray and the menu bar, in one ink |
| `originals/` | vector | The light, dark, macOS and menu-bar SVGs as delivered |
| `tisty-512.png` | 512×512 | Press, release pages, the MCP registry |
| `tisty-256.png` | 256×256 | The logo at the top of the README and the wiki |
| `tisty-128.png` | 128×128 | Small inline use |
| `store-logo-300.png` | 300×300 | Microsoft Store listing logo |

The application's `.ico` (16 to 256, with the 20 and 40 Windows draws at 125 %
and 250 %), its PNGs and the MSIX logos wear the light drawing; the `.icns`
wears the macOS tile, dark in light and dark mode alike, since the Dock gives
no surface behind an icon and a white tile lost the white sheets on it. On a
Mac with Xcode 26 the script also compiles `Tisty.icon` into `Assets.car`,
which macOS 26 prefers to the `.icns`.

`app/src-tauri/icons/msix/` holds what the MSIX packs, at every target size
Windows asks for, `lightunplated` included: without it a light taskbar plates
the logo on the manifest's `BackgroundColor`, which is transparent so no other
surface plates it either. There are no scale-qualified logos, since `makepri`
would split them into resource packs that a single MSIX never loads.

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

## Screens

`screens/` holds the screens that present Tisty, one per subject and language,
named `<language>-<order>-<subject>`: `en-` in English and `es-` in Spanish,
with the same order in both, `.png` in `store/` and `web/` and `.webp` in
`readme/`. They present what Tisty is, not what a version
brings. The order is the one the Store shows, and the READMEs and the help
follow it for the first ones: memory, diagrams, brainstorm, priorities, agent,
documents, day, Folio, data, close. `store/` also keeps the three Store
promotional images: `store-box-1x1.png`, `store-hero-16x9.png` and
`store-poster-9x16.png`.

| Folder | Size | Where it goes |
| --- | --- | --- |
| `store/` | 1920×1080 | The Microsoft Store listing, and the picture tisty.app shows and shares |
| `web/` | 1280×720, lighter | The four screens the help has no capture of yet: agent, Folio, data and close |
| `readme/` | 1336 wide, WebP | The 1.45 captures still in use: the day and the archive that `docs/posts.json` posts |
| `app/` | 1558×818, lossless WebP with rounded, transparent corners; Brainsty alive 960 wide, animated | Real captures of the app since 1.49, for the READMEs and the help |
| `help/` | PNG | Pictures only one help page shows, `help/agents/` for the Agents folder's and `help/settings/` for Settings' |

`app/` is named `<language>-<subject>-<theme>.webp`: `en-` or `es-`, a subject
such as `archive` or `revision-semanal`, and `light` or `dark`. The READMEs show
each pair in a `<picture>`, so a reader sees the theme GitHub is in; the help
shows the light one. Two subjects come only in the dark theme: the tasks of today
(`tasks-today`, `tareas-hoy`) and Brainsty, animated: `es-brainsty-3d-dark`,
the 3D network in Spanish flying in to a task, which both READMEs show, and
`en-brainsty-living-dark`, the whole window in English, which the help shows.

The help links to `web/` and `app/` on `main`, so renaming a screen means
updating the help in the same change, and tisty.app reads its picture from
`store/` through `scripts/apps.mjs` in its own repository. The READMEs show the
app itself, not the Store's presentation.
