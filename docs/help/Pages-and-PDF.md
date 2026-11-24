# Pages, page breaks and PDF

## A document with pages

A year of minutes, a book in chapters, a dog's life told year by year. That does not fit on one sheet, and it does not fit in a folder of loose papers with nothing tying them together either.

A **page** is a document like any other — the same `.md` file on disk, the same way out — with one difference: it says which document it belongs to. There is one level and no more: a page holds no pages.

### Adding a page

Type `/` where that chapter should start and pick **A new page**, or **A page that already exists** if it is already written. It is named right there, drawn as a card with its title, its first words and its place in the book.

In the tree, a document dropped on another becomes a page of it.

> [!IMPORTANT]
> The order pages are named in is the order they sit in: in the tree, in the export and in print. To move a chapter, open **Pages**, under the sheet, and use the arrows on its row, or `Alt` and an arrow key on the row itself — or cut and paste its block.

### The index

Under the document's sheet there is a **Pages** row: a click opens it and shows the index — the pages the text names, numbered, and after a rule the ones it does not, each with **Put it in the text** to give it its place. Inside a page you see which document it belongs to and where it sits, arrows to its sisters, and the step to the next one at the foot.

A link to a page in the middle of a paragraph points at it without giving it a place — the way a footnote points at a chapter without being one. A page only written about that way waits under the rule until you put its card in.

### Putting a page away

A page is archived on its own with the right button, and it does not leave its document: it stays where it lives, dimmed, with the archive icon, readable but closed to writing. Archive the whole document and every page goes with it; bring it back and each one returns as it was — the one you had set apart stays apart.

## Page break

Turn on **Sheets** in a document's panel to choose its paper — **A4**, **Letter** or **Folio** (8.5 × 13 in) — and read it on those sheets, upright or, with **Landscape** on, across: a sheet that fills up turns on its own, on screen as on paper. A document that already asked for **Tabloid** keeps it. Sheets are off until you turn them on; the paper stays chosen when you turn them off, its PDF comes out on it, and what you pick travels with the file in its [properties](https://markdown.tisty.app/properties). In a read-only window, **Sheets** changes only how you see it and its PDF: the file stays as it was.

A rule on its own line (`---`) always splits the sheet, sheets or not, and `Ctrl`/`⌘` + `Enter` writes one while you write. With sheets on, it is marked **Page break**. How one is written, and what comes in as one from LaTeX, Word or a web page, is in the [Markdown guide](https://markdown.tisty.app/page-breaks).

## Document designs

A recipe you cook from and the minutes of a meeting do not have to look alike. The row of colored lines in the document's panel, under the paper, changes how that document looks, without touching a word of it. Each line is a design: rest the pointer on one, or reach it with `Tab`, to read its name, and the thickest is the one it wears.

| Design | How it looks |
| --- | --- |
| **Default** | The one every document is born with |
| **Forest** | Greens, with a line under the title |
| **Coral** | Warm, with a bar under the title |
| **Fjord** | Blues and grays |
| **Violet** | Violets, with a bar under the title |
| **Your own** | Background, headings, text and font, picked by you |

Each has its light colors and its dark ones, and follows the window. It works with sheets on or off, and changes the letters, the title, the headings and the head of every table. Diagrams keep their own colors.

For example, open **Rina's vaccines**, press the line of **Forest** and turn the window dark: the paper goes dark green and the headings light green. Press **Default**, the first one, and it is as it was.

- **Your own** is made in the panel: press its line, the last one, and pick a background, the color of the headings, the color of the text and one of nine fonts. For example, **Sand**, **Wine**, **Coffee** and **Lora** for a recipe book. Each color brings its light one and its dark one, any of them read well together, and sizes do not change. The fonts come with Tisty, so nothing is fetched.
- **From the text:** a link to `tisty:design/coral`, or to any of the others, puts that design on the document when it is pressed. The starting guide has them to try; `tisty:design/tisty` goes back.
- **Notes, code, diagrams and widgets** keep their own colors on every design.
- **On paper and in the PDF** it wears its light colors over white paper, even from a dark window.
- **In the file** it is one line of its [properties](https://markdown.tisty.app/properties), `design: bosque` or `design: own/arena/vino/cafe/lora`, and the first line of the panel, **Default**, leaves none. A name Tisty does not know stays written and the document looks as **Default**. If a document of yours already had a property of your own with one of those four names, like `design: coral`, it now takes effect: press the first line of the panel, or name the property something else. A property of your own with any other value stays as it is until you pick another design in that document.
- **With pages**, a document wears one design, and its pages wear it too, on screen, in print and shared. You pick it in the document; a page has nothing to pick.
- **In several languages**, the design is the same in all of them: picking one, or joining a language, puts it on the rest. A property of your own called `design` stays as you wrote it.
- **In a window of its own**, the one that only reads, the sheet wears the design too; there is nothing to pick there.

## Tisty's own PDF

Tisty makes its own PDF, and shows it to you before you export it:

- **A4**, **Letter** or **Folio** paper, upright or across.
- Its own margins.
- Its pictures carried inside; any other attachment is named where it sits.
- Optionally, **who wrote it and when** — a PDF leaves Tisty for good, so what goes in it is your choice.
- The page with everything it draws on screen — code colours, callouts, diagrams, formulas and icons — in the type it has with **Sheets** on.

From **Export**, in the document's panel: **Preview the PDF** shows it and **Export to PDF…** saves it.
