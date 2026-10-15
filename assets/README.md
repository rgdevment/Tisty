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

The source of truth is `icons/tisty-app-icon.svg`: everything in `icons/` is
rendered from it, so change the SVG and re-render rather than editing a PNG by
hand.

| File | Size | Where it goes |
| --- | --- | --- |
| `tisty-app-icon.svg` | vector | The master. Everything else comes from it |
| `store-logo-300.png` | 300×300 | Microsoft Store listing logo |
| `tisty-512.png` | 512×512 | Press, release pages, anything that scales |
| `tisty-256.png` | 256×256 | The logo at the top of the README |
| `tisty-128.png` | 128×128 | Small inline use |

The icons the application itself ships with — the `.ico`, the `.icns`, and the
`Square*Logo` tiles Windows needs — are not copied here. They live in
`app/src-tauri/icons/`, which is what the build reads, and one set is easier to
keep honest than two.

The tray icons are a different drawing, not a smaller version of this one: the
mark is the only one with a tile, because it lands on surfaces we do not
control. What the build reads of them lives in `app/src-tauri/icons/tray/`; the
drawing they come from stays with the rest of the icon work outside this
repository.

## Mascot

The beaver is called Tisto. `mascot/tisty-head.svg` is the colour drawing;
`app/src/assets/tisty.svg` is the copy the window shows in About and in the news
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
