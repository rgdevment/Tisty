# Icons

The mark, at the sizes used outside the build. The source of truth is
`tisty-app-icon.svg`: everything here is rendered from it, so change the SVG
and re-render rather than editing a PNG by hand.

| File | Size | Where it goes |
|---|---|---|
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
control. They stay with the rest of the icon work outside this repository.
