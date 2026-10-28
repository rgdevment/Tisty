# The Markdown editor

A document in Tisty is text of yours, kept as a plain `.md` file, that you write here and can read with anything else. Delete Tisty tomorrow and your documents still open in any text editor, on GitHub, or in another notes app.

![Notes that can do the maths: formulas, diagrams and code in plain Markdown](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/store/en-03-notes.png)

## Writing where you read

There is one way of writing, and it happens on the page itself.

- **Click a block and it opens right there**, with the caret where you clicked. The rest of the page stays as it reads. A click on a picture or a diagram opens it large instead, to zoom in, move it around and go to the next one; to edit it, use the block's menu, and a double click on a diagram opens it in the [[diagram window|Diagrams]].
- **The marks stay out of sight.** Bold looks bold and a heading looks like a heading; the `**` or the `#` only show where the caret touches that format, so you can see what you are changing.
- **Headings, lists, stages, quotes and callouts are always drawn**, even while you write them: a bullet is a bullet, a box is a box you can tick, a callout wears its colour and its name.
- **Esc, or a click outside, goes back to reading.** What you write is saved as you go, and the foot of the page says so.
- **The first line is the title.** There is no name to fill in anywhere: whatever you write at the top is what the document is called.
- **`Ctrl`/`⌘` + `Shift` + `M` shows every mark** in the block, for when you want to see the Markdown itself. Press it again to hide them.

A line at the foot of the page keeps the keys in sight: `/` to insert, `#` and a space for a heading, the shortcut for every mark, and Esc to read.

## The bar

While a block is open, a bar floats over it:

- **The kind of block**: Paragraph, Heading, Subheading, Small heading, Bulleted list, Numbered list, Stages or Quote. Every line you have selected changes at once. Inside a table it reads **Table** and offers a row above or below, a column on either side, the column's alignment, and removing a row or a column. The dashes under the header stay as you drew them, so the widths do too.
- **Bold**, **italic**, **underline**, **strikethrough**, **code** and **link**.
- **Four pens**: yellow, green, blue and pink.
- **Table**, which puts a table where the caret is and opens it on the page. Inside a table it steps aside.
- **Diagram**, in a document: a new diagram of each kind or an existing one, as in the `/` menu.
- **/ More**, which opens the `/` menu.

With words selected, a format wraps them; pressed on words that already have it, it takes it away. Formats never cross: one that reaches another of its kind joins it into one.

With nothing selected, a format starts right where the caret is, and what you type already looks bold, or underlined, or yellow. Press it again to step out of it, or press another pen to change the colour. Enter or an arrow ends it too.

## The `/` menu

Type `/` anywhere and the menu of what fits comes up. Keep typing to narrow it down, ↑ ↓ to move, Enter to add it, Esc to close it.

- Headings, lists, stages and quotes.
- The five callouts: Note, Tip, Important, Warning and Caution.
- Bold, italic, underline, strikethrough, code, link and the highlight pen, with their shortcuts beside them.
- An icon, looked up by what it draws.
- A new diagram, which opens the [[diagram window|Diagrams]], a copy of an existing diagram, or a Mermaid block as text; a formula, a widget and a code block.
- A table.
- A **collapsible section**: a title that folds what is under it.
- A page break.
- Another document, a new document, a page that already exists or a new page, drawn as a card.

## While you type

- **Tags are offered after `#`.** The tags you already use come up as you type, so one tag is never spelled two ways. What you typed stays first, so Enter keeps a new tag as it is.
- **An empty item ends a list.** Enter on an empty bullet, number or box leaves the list with a blank line, so the words after it never join the last item.
- **Backspace right after a line's mark** — a heading's `#`, a bullet, a quote's `>` — turns the line back into words.
- **Home** lands on the words, past the line's mark.
- **→** at the end of a new link's words steps into its address.
- **Attach a file** by dragging it onto the page, pasting it (a screenshot on the clipboard works too) or picking **Attach a file** in the `/` menu. It lands where the caret is, or after the block you drop it on, and it is kept inside Tisty with the document, not linked to a place on your disk that breaks when the file moves.
- **Paste with its formatting.** Text copied from a web page, an email or a word processor keeps its headings, lists, bold, links, tables and coloured words, written in Tisty Markdown: a colour becomes the nearest pen of the palette. Plain text keeps its shape too: a table a terminal drew with box lines (`┌ │ ├`) becomes a table, and lines that read like code — from a code editor or a terminal — come in as a code block, unless the caret is already inside one.
- **Tick a box on the page** with a click, without opening the block.
- **Tab and Shift + Tab walk a table's cells**, and Tab past the last one starts a new row.
- **Drag the edge of a table's column** on the page to make it wider or narrower. The width is written as the dashes of its rule, ten pixels each.

| Shortcut | What it does |
| :-- | :-- |
| `Ctrl`/`⌘` + `B` | Bold |
| `Ctrl`/`⌘` + `I` | Italic |
| `Ctrl`/`⌘` + `U` | Underline |
| `Ctrl`/`⌘` + `Shift` + `X` | Strikethrough |
| `Ctrl`/`⌘` + `E` | Code |
| `Ctrl`/`⌘` + `K` | Link |
| `Ctrl`/`⌘` + `Z` · `Ctrl`/`⌘` + `Shift` + `Z` | Undo · redo |
| `Ctrl`/`⌘` + `Shift` + `M` | Show or hide every mark |
| `Esc` | Back to reading |

The writing sits on a lit page. When the window is wide enough, a column opens beside it with what the document is, its tags, its outline and the way to its PDF.

## Tables

A table is written on the page itself, without opening its block:

- **Click a cell** to write in it; Enter or a click outside keeps it, Esc leaves it as it was. Its title is written the same way. The cell shows its words without marks, and a bar over the table puts bold, italic, underline, strikethrough, code or a pen on the whole cell, or takes it off. A table made from the `/` menu opens this way at once, on its first title.
- **A row's and a column's `⋯` menu** move it, duplicate or remove a row, insert a column on either side, rename it, and give it a **type**: number, money, percent, date, duration, yes or no, choice, tags, rating, document, web address, email or phone. Money takes its currency, and a column of numbers can add itself up at the foot: sum, average, minimum, maximum or count. **Edit as Markdown** opens the table as text.
- **Drag a row or a column** by its handle to put it somewhere else, and **New row** under the table adds one at the end.
- **A typed cell is written as its type asks**: a calendar for a date, where you can also type «next Friday»; the column's own options for a choice, each with its colour; stars for a rating; a click for yes or no; a finder for a document.
- **Sort and filter from a column's head.** That order and those filters are on screen only and leave the file alone; **Save this order** writes the rows in that order.
- **A value not written as its type asks**, like «soon» in a date column, keeps what you wrote and shows a wavy line under it.
- **In the text**, with the block open, `Alt` + ↑ ↓ moves the row the caret is on and `Alt` + ← → its column (`Ctrl` + `Alt` on a Mac), and the bar's **Table** menu sorts it and gives a column its type.
- **Rows copied from Excel, Numbers or Google Sheets** — or a CSV — come in as a table when you paste them.

A table wider than the page scrolls sideways, and its PDF and Word copies keep every column. How a type is written in the file is in the guide's [column types](https://markdown.tisty.app/tables).

## What a document can hold

Everything a document holds, from bold and lists to callouts, pens, icons, tables, diagrams, formulas, widgets, code and pages, is plain Markdown with a few short words of Tisty's own, so the file survives without Tisty. How each one is written, with its examples drawn, is in the [Tisty Markdown guide](https://markdown.tisty.app), which also opens inside Tisty with `F1`. You never need it to write: the `/` menu and the bar write it all for you.

## Markup Tisty has no words for

Some Markdown from elsewhere — HTML tags — has no word in Tisty. A document that holds it still opens, edits and saves whole: nothing is cut out. That part simply shows as it is written while you edit it. **Settings › Maintenance** goes through every document and lists the ones that hold some. The full list is in the Tisty Markdown guide, under [what is outside the language](https://markdown.tisty.app/what-is-outside-the-language).

## When a document opens read-only

The bar at the foot of the document says why.

**It is in the archive.** Archiving a document, or the folder it sits in, closes it to writing — from the window and from an assistant alike. Bring it back and it writes again.

**A page of it is archived, or the document that holds it is.** See [Pages and PDF](Pages-and-PDF).

## Offline, always

Diagrams, formulas, code colouring and widgets are all drawn by code that ships inside Tisty. Nothing is fetched, no font is loaded from anywhere, and it all works with the network unplugged.
