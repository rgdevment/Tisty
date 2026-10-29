# Pages, page breaks and PDF

## A document with pages

A year of minutes, a book in chapters, a dog's life told year by year. That does not fit on one sheet, and it does not fit in a folder of loose papers with nothing tying them together either.

A **page** is a document like any other — the same `.md` file on disk, the same way out — with one difference: it says which document it belongs to. There is one level and no more: a page holds no pages.

### Adding a page

Type `/` where that chapter should start and pick **A new page**, or **A page that already exists** if it is already written. It is named right there, drawn as a card with its title, its first words and its place in the book.

In the tree, a document dropped on another becomes a page of it.

> [!IMPORTANT]
> The order pages are named in is the order they sit in: in the tree, in the export and in print. To move a chapter, use the arrows on its row in the index at the end of the document, or `Alt` and an arrow key on the row itself — or cut and paste its block.

### The index

At the end of the document, on its own sheet, comes the index: the pages the text names, numbered, and after a rule the ones it does not, one click from a place in the text. Inside a page you see which document it belongs to and where it sits, arrows to its sisters, and the step to the next one at the foot.

A link to a page in the middle of a paragraph points at it without giving it a place — the way a footnote points at a chapter without being one. A page only written about that way waits under the rule until you put its card in.

### Putting a page away

A page is archived on its own with the right button, and it does not leave its document: it stays where it lives, dimmed, with the archive icon, readable but closed to writing. Archive the whole document and every page goes with it; bring it back and each one returns as it was — the one you had set apart stays apart.

## Page break

Each document picks its paper — **A4**, **Letter** or **Folio** (8.5 × 13 in) — under **Paper** in its panel; a document that already asked for **Tabloid** keeps it. Turn on **Sheets** there to read it on those sheets, **Portrait** or **Landscape**: a sheet that fills up turns on its own, on screen as on paper. Sheets are off until you turn them on, and what you pick travels with the file in its [properties](https://markdown.tisty.app/properties). In a read-only window, **Sheets** changes only how you see it and its PDF: the file stays as it was.

A rule on its own line (`---`) always splits the sheet, sheets or not, and `Ctrl`/`⌘` + `Enter` writes one while you write. With sheets on, it is marked **Page break**. How one is written, and what comes in as one from LaTeX, Word or a web page, is in the [Markdown guide](https://markdown.tisty.app/page-breaks).

## Tisty's own PDF

Tisty makes its own PDF, and shows it to you before you export it:

- **A4**, **Letter** or **Folio** paper, upright or across.
- Its own margins.
- Its pictures carried inside; any other attachment is named where it sits.
- Optionally, **who wrote it and when** — a PDF leaves Tisty for good, so what goes in it is your choice.
- The page exactly as it looks on screen — code colours, callouts, diagrams, formulas and icons — since the PDF is the page itself, laid on paper.

Print, or save as PDF, from the document's panel.
