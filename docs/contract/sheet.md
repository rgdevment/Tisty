# The sheet

The sheet is the HTML Tisty draws from the words when it seals a version. It is
the only thing a reader shows, and it is finished: a reader never reads the
words to draw it.

It comes from whoever wrote the document, so **nobody believes it**. A reader
must clean it with [allowed.json](allowed.json) before showing it, and a server
that shows it must do the same.

The sheet carries no CSS. Each side writes its own from
[tokens.json](tokens.json).

## The components

| Component | How it is drawn |
|---|---|
| Headings, paragraphs, lists, quotes, rules | `h1` to `h6`, `p`, `ul`, `ol`, `li`, `blockquote`, `hr` |
| Emphasis and small marks | `strong`, `em`, `del`, `u`, `sub`, `sup`, `kbd`, `abbr` |
| Pen | `<mark data-pen="green">`, in yellow, green, blue or pink |
| Coloured words and icons | `<span class="ico" data-ico="dog" data-hue="orange" title="dog">` |
| Tag | `<span class="tag" data-tag="house">` |
| Callout | `<blockquote class="callout" data-callout="tip">`, or note, important, warning, caution |
| Fold | `<details>` and `<summary>` |
| Table | `<div class="table"><table>`, with `data-type` on a column and `data-total` on a sum |
| Check box | `<li class="task"><span class="check" data-done="true">` |
| Code | `<pre><code class="language-x">`, with `<span class="hljs-x">` when it arrives coloured |
| Formula | `<div class="math"><math>`, in MathML |
| Diagram | `<figure class="diagram">`, see below |
| Picture | `<img src="tisty:picture/N" alt="…" loading="lazy">` |
| Attached file that stays in Tisty | A card of `data-file` and `data-type`, not an `img` |
| Card to another document | `data-doc`, leading to an entry in `cited` |
| Footnote | `<sup class="fnref">` and `<div class="footnote" id="fn-x">` |
| Properties | `<dl class="front">` |
| Link | `<a href>`, with the schemes `allowed.json` lists |

`data-at` says which bytes of the words a block came from. A reader may use it
to link to a place, and must not need it.

A sheet in [vectors/sheets](vectors/sheets) shows every component. Those are
drawn as the translator draws today, so their pictures still carry file names
and their diagrams and code are not drawn yet. A sealed sheet has neither.

## Drawn before it travels

A reader carries no engine, so the writer draws all of it before sealing:

- **Pictures** become `tisty:picture/N`, each counted in the envelope from 0.
- **Code** arrives coloured, in `hljs-x` spans. Code that does not is shown as
  plain text, and nothing more is asked.
- **Formulas** arrive as MathML, which browsers draw on their own.
- **Diagrams** arrive as inline SVG in `<figure class="diagram">`, using only
  the SVG `allowed.json` lists. They take their colours from the tokens, as
  `var(--tisty-x)` or `currentColor`, so they follow the theme. Without an SVG,
  a diagram stays as `<pre><code class="language-mermaid">`, and a reader shows
  that as code.

## Cleaning

Cleaning works on a parsed tree, with [allowed.json](allowed.json):

- **Tags:** only those in `tags`. Anything in `never` goes with what is inside
  it, and any other tag leaves its text behind.
- **Attributes:** only those in `attributes`. No `on*`, no `aria-*`, and no
  `data-*` outside the list.
- **Styles:** a `style` stays only if each property is in `styles` with a value
  its rule allows, and otherwise it goes whole. A length is a number and a unit
  from `lengths_at_most`, not past its limit and never below zero, unless the
  rule says `negative`. There is never `url()` and never `@import`.
- **Links:** only the schemes in `urls`, or a value with no scheme. A value
  starting `//` leads to another site and goes.
- **Pictures:** `src` must match `picture_src`, so a page never asks anyone else
  for one.
- **SVG fills and strokes:** `fill` and `stroke` must match `colour_fills`.

The cleaner should also tell the reader if it took out something Tisty never
draws, because that sheet was made by hand.

A reader may clean with any engine. A new engine goes in only once it passes
[vectors/hostile.json](vectors/hostile.json).

A reader should show the sheet in something that keeps its CSS and ids from
meeting the frame's, such as a shadow root, and must keep what it decides about
the document, such as the signature's verdict, outside the sheet.

## The style rules

[tokens.json](tokens.json) says, in the W3C design tokens format, what a sheet
looks like: its letters and their scale, the colour of each role in light and in
dark, its tables, and its papers and margins.

**The sheet looks the same everywhere.** Whoever shows one must use these values
on the sheet. The frame around it is free: its bar, panels, buttons and brand.

Text must keep at least the WCAG AA contrast on the sheet, in both themes.

## The papers and the sheets

A document is laid out in one of the papers of `tokens.json`. A sheet measures
on screen what it measures on paper, at 96 pixels to the inch, with its margins
inside. `paper.offered` lists the ones a reader picks from, and a paper a
document asks for beyond them is offered too. Turned, a sheet's width and height
trade places.

With `sheets` off, the document is one long page in `sheet.width`.

The language of the sheet comes from `lang` in `config`, and a reader must set
it on what holds the sheet, because screen readers and word breaks depend on it.
The sheet itself does not declare it.
