# Third-party notices

Tisty is AGPL-3.0-only. What follows was copied into Tisty's own source and
carries its own licence, which allows that.

Tisty also bundles other people's work rather than copying it. The parts that do
the heavy lifting are [Tauri](https://tauri.app), [React](https://react.dev),
[Tailwind CSS](https://tailwindcss.com), [TipTap](https://tiptap.dev) and
[ProseMirror](https://prosemirror.net) for the editor,
[markdown-it](https://github.com/markdown-it/markdown-it) for the prose,
[Mermaid](https://mermaid.js.org) for the diagrams, [KaTeX](https://katex.org)
for the formulas, [lowlight](https://github.com/wooorm/lowlight) and
[highlight.js](https://highlightjs.org) for the code, and
[react-pdf](https://react-pdf.org) for the PDF — with
[serde](https://serde.rs), [jiff](https://github.com/BurntSushi/jiff),
[SQLite](https://sqlite.org) and [clap](https://github.com/clap-rs/clap)
underneath. Each one is permissively licensed, and `Cargo.lock` and
`app/package-lock.json` name every last transitive one with its version.

`npm run notices` writes that whole list out with each licence's own text into
[THIRD-PARTY-BUNDLED.md](THIRD-PARTY-BUNDLED.md), and the binary carries it:
Tisty shows it under About. A notice discharges nothing sitting in a repository
the person who installed Tisty never opens, so it travels with the program.

## Lucide Icons

The icon set drawn throughout the window, copied path by path into
`app/src/glyphs.ts` so that nothing is ever fetched at runtime.

- Source: <https://lucide.dev>
- Licence: ISC, and MIT for the icons Lucide inherited from Feather

```text
ISC License

Copyright (c) 2026 Lucide Icons and Contributors

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND
FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.
```

A part of the set descends from Feather and carries MIT instead.

```text
MIT License

Copyright (c) 2013-2023 Cole Bemis

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

The Rust side's dependencies are checked by `cargo deny` against the allow-list
in `deny.toml`; this file covers what npm brings into the bundle instead.
