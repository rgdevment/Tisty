# The Markdown editor

A document in Tisty is text of yours, kept as a plain `.md` file, that you write here and can read with anything else. Delete Tisty tomorrow and your documents still open in any text editor, on GitHub, or in another notes app.

## Writing where you read

You write on the same page you read, the one Tisty draws. There is no separate editing mode and no block to open: you click and type.

- **Click wherever you like and type.** The caret lands where you clicked, and the page is drawn again as you write, the same as it reads, prints and is shared. What you write is saved as you go.
- **The marks stay out of sight.** Bold looks bold and a heading looks like a heading. The Markdown is still underneath, and the **Markdown** view shows it as it is.
- **If you know Markdown, you can type it too.** The marks you type become what they say: `#` and a space at the start of a line make it a heading, and `[ ]` and a space, a box. On a task's page the `#` stays as text, since every heading there is a section.
- **The first line is the title.** There is no name to fill in anywhere: whatever you write at the top is what the document is called.
- **Enter starts a new paragraph, and `Shift` + `Enter` only breaks the line.** In a list, Enter starts another item of the same kind; in a quote, the quote goes on; in code, it breaks the line. Enter on an empty item leaves the list, or goes up a level when it sat inside another; on an empty quote line it leaves the quote. Enter at the start of a block opens a line above it.
- **`Tab` puts a list item under the one above, and `Shift` + `Tab` takes it back out.** In code, `Tab` indents with two spaces.
- **`Backspace` at the start of an item** takes its box away first, then its bullet or its number, and the line is left as text.
- **Tick a box with a click**, a sticky note's too. A sticky note is written inside and dragged by its paper, its tape or its pin.
- **A click on a picture or a diagram opens it large**, to zoom in, move it around and go to the next one. A picture or a file shown on the page, such as a PDF, an audio or a video, is taken away with `Delete` right before it or `Backspace` right after it, or from its right-click: **Delete picture** or **Remove from the page**. One alone on its line takes the line with it. A diagram brings its own bar when the pointer is over it: **Export** and **Open in the diagram window**; a double click also opens it in the [[diagram window|Diagrams]]. Its text is changed in the Markdown view. A widget brings **Show code**, which opens it in its [[window|Widgets]] with the code beside what it draws, and **Open apart**. A formula opens in that same window: one in a block of its own from its **Show code**, and one inside a line with a click. What you write is drawn beside it and saved in its place. **Edit**, on the properties in the panel, opens them the same way.
- **Code is written as it is.** Inside a code block there are no formats and no spell checker, and `/` is just a slash. Its lines are numbered beside the text rather than in it, so copying takes the code and nothing else. The label in its corner, like «rust» or «yaml», opens the list of languages to choose the colours it is drawn with. Typing `` ``` `` at the start of a line opens a code block that closes itself, and a language's name written on its first line, like `rust`, becomes its language with Enter.
- **A page break is neither written in nor moved.** `Backspace` at the start of the block after it, or `Delete` at the end of the block before it, takes it away, and the rest stays as it was. What you type right after it starts on a new line under it.
- **A table, a code block, a diagram, a collapsible section or a callout is not joined to what sits beside it.** `Backspace` at the start of the block after it, or `Delete` at the end of the block before it, does nothing; to take one away there is **Delete block**, on its handle. A picture or a file alone on its line is the exception: those same keys take it away.
- **`Ctrl`/`⌘` + `Z` undoes** and `Ctrl`/`⌘` + `Shift` + `Z`, or `Ctrl`/`⌘` + `Y`, redoes, only in the document in front of you. What you type in a row is undone in one go; a pause or an Enter starts another step. What you wrote in the Markdown view is undone in one go when you come back.
- **`Ctrl`/`⌘` + `C` and `Ctrl`/`⌘` + `X` copy and cut the Markdown** of the words selected, marks and all, as the right-click does; pasted into another document they come out the same.
- **Drag the words selected** to another place on the page: a line shows where they land, and letting go moves them. `Ctrl`/`⌘` + `Z` brings them back.

## The format bar

Select a few words and a small bar comes up above them:

- **Bold**, **italic**, **underline**, **strikethrough**, **code** and **link**.
- **Four pens**: yellow, green, blue and pink.

A format wraps the words selected, and when they cross several lines, each line gets its own; pressed on words that already have it, it takes it away. **Link** asks for the address in a field, and `Esc` cancels it; on a link already written, `Ctrl`/`⌘` + `K` shows its address to change it, leaving its words alone; an address like `tisty.app` gets its `https://`, and with no words selected, the address stands for its words. A click on a link puts the caret in its words, and `Ctrl`/`⌘` + click opens it. Inside code the bar does not come up.

With nothing selected, a format pressed with its shortcut or from the `/` menu starts at the next key, and what you type already comes out bold or italic. Press it again to step out of it; a space at the end of the words stays outside it too.

`Ctrl`/`⌘` + `B`, `I` and `U` put on bold, italic and underline; `Ctrl`/`⌘` + `Shift` + `X`, strikethrough; `Ctrl`/`⌘` + `Shift` + `C`, code, and `Ctrl`/`⌘` + `K`, a link. Right-click on selected words **cuts** or **copies** them as Tisty Markdown, ready to paste into another document, and offers the same formats.

## The `/` menu

Type `/` at the start of a line, on a blank line or after a space, and the menu of what fits comes up; inside a word or in code it is just a slash. `Ctrl`/`⌘` + `/` opens it anywhere, and so does the **+** that shows in the margin, level with the caret. Keep typing to narrow it down, ↑ ↓ to move, Enter to add it, Esc to close it; closed without a choice, the `/` stays written.

- Headings, lists, stages and quotes.
- The five callouts: Note, Tip, Important, Warning and Caution, and the Idea sticky note.
- Bold, italic, underline, strikethrough, code, link and the highlight pen.
- An icon, looked up by what it draws.
- A new diagram, which opens the [[diagram window|Diagrams]], a copy of an existing diagram, or a Mermaid block as text; a formula, a widget and a code block. The formula, the widget and the Mermaid block come with a sample already drawn, and what you type next goes on a line under it.
- A table, which opens on its first title.
- A **collapsible section**: a title that folds what is under it.
- A page break.
- Another document, a new document, a page that already exists or a new page, drawn as a card. Type part of its name, and Enter takes the first one that shows.

## Moving blocks

- **The handle.** With the pointer beside a block, a handle shows to its left. Drag it to take the block somewhere else; a heading takes its whole section with it. Near the top or the bottom edge, the page scrolls by itself. On a sheet too narrow to leave a margin, the handle and the **+** do not show; `Ctrl`/`⌘` + `/` and `Alt` + ↑ ↓ (`Ctrl` + `Alt` on a Mac) still work.
- **Its menu.** A click on the handle offers **Move block up**, **Move block down**, **Duplicate block** and **Delete block**. On a heading, all four act on its whole section, as dragging does: **Delete block** takes everything under it up to the next heading of its level or higher. To remove the heading alone, choose it with `Esc` and press `Delete`.
- **The keyboard.** `Alt` + ↑ ↓ (`Ctrl` + `Alt` on a Mac) moves the block the caret is in up or down, or the list item it is in.
- **Choosing whole blocks.** `Esc` chooses the block the caret is in, and `Shift` + ↑ ↓ takes in the ones above or below. Once chosen, `Delete` or `Backspace` removes them, `Alt` + ↑ ↓ (`Ctrl` + `Alt` on a Mac) moves them together and `Ctrl`/`⌘` + `C` or `X` copies or cuts them. `Esc` again, a click or any other key lets them go.

## Rich text and Markdown

A document can be seen two ways, and both write the same file:

- **Rich text** is the page as drawn, the one that is read, printed and shared. Almost everything is written there.
- **Markdown** shows the text as it is kept, with every mark in view: the `**`, the `#`, the `>`, a table's rules. It is there for changes that run across blocks, for writing a diagram as text, and for changing the letters of inline code or a tag, which on the page are one whole piece the caret does not go into.

The **⋯** on the ribbon at the top of the document switches between them, and so does `Ctrl`/`⌘` + `E`. On a task, **Show Markdown** in its column does the same, and **Show rich text** goes back. As you scroll down a document, the ribbon also keeps its title in sight.

In the Markdown view:

- **The `/` menu, the shortcuts and the tags** work as they do on the page, except that a block moves with `Alt` + ↑ ↓ on a Mac too, as a line does in a code editor. The bar also holds the kind of line, **Table** and **Diagram**, and the right-click offers **Choose block**, **Duplicate block**, moving it up or down and deleting it.
- **An empty item ends a list.** Enter on an empty bullet, number or box leaves the list with a blank line.
- **Backspace right after a line's mark** (a heading's `#`, a bullet, a quote's `>`) turns the line back into words.

## While you type

- **Tags are offered after `#`.** The tags you already use come up under the caret, so one tag is never spelled two ways. ↑ ↓ picks one and Tab or Enter writes it; Esc puts the list away, and Enter with none picked still starts a paragraph.
- **Attach a file** by dragging it onto the page, pasting it (a screenshot on the clipboard works too) or picking **Attach a file** in the `/` menu. It lands after the block you drop it on, or after the one the caret is in, on a line of its own, and it is kept inside Tisty with the document, not linked to a place on your disk that breaks when the file moves.
- **Paste with its formatting.** Text copied from a web page, an email or a word processor keeps its headings, lists, bold, links, tables and coloured words, written in Tisty Markdown: a colour becomes the nearest pen of the palette. A pasted block, like a list or a heading, starts on its own line, and inside a quote it stays in the quote. Plain text keeps its shape too: a table a terminal drew with box lines (`┌ │ ├`) becomes a table, and lines that read like code — from a code editor or a terminal — come in as a code block, unless the caret is already inside one.

| Shortcut | What it does |
| :-- | :-- |
| `Ctrl`/`⌘` + `B` | Bold |
| `Ctrl`/`⌘` + `I` | Italic |
| `Ctrl`/`⌘` + `U` | Underline |
| `Ctrl`/`⌘` + `Z` · `Ctrl`/`⌘` + `Shift` + `Z` or `Ctrl`/`⌘` + `Y` | Undo · redo |
| `Enter` · `Shift` + `Enter` | New paragraph · line break |
| `Ctrl`/`⌘` + `/` | Open the blocks menu where the caret is |
| `Alt` + ↑ ↓ (`Ctrl` + `Alt` on a Mac) | Move the block or the item the caret is in up or down |
| `Esc` · `Shift` + ↑ ↓ | Choose the whole block · take in the ones beside it |
| `Tab` · `Shift` + `Tab` | Put a list item in or take it out |
| `Ctrl`/`⌘` + `E` | Switch between Rich text and Markdown |
| `Ctrl`/`⌘` + `Shift` + `X` · `Shift` + `C` · `K` | Strikethrough · code · link |
| `Ctrl`/`⌘` + `Enter` | Page break |

The writing sits on a lit page. When the window is wide enough, a column opens beside it with what the document is, its tags, its outline and the way to its PDF.

## Tables

A table is written on the page itself:

- **Click a cell** to write in it; Enter or a click outside keeps it, Esc leaves it as it was; Enter on the last row leaves the table for a new line under it. Its title is written the same way. The cell shows its words without marks; choose words in it and a bar comes up above the cell that puts bold, italic, underline, strikethrough, code or a pen on the whole cell, or takes it off. A table made from the `/` menu opens this way at once, on its first title.
- **A row's and a column's `⋯` menu** move it, duplicate or remove a row, insert a column on either side, rename it, and give it a **type**: number, money, percent, date, duration, yes or no, choice, tags, rating, document, web address, email or phone. Money takes its currency, and a column can sum itself up at the foot: sum, average, minimum or maximum for numbers, minimum and maximum for dates, and count for any. **Edit as Markdown** opens it as text in a window of its own, with the table drawn beside it.
- **Drag a row or a column** by its handle to put it somewhere else, and **New row** under the table adds one at the end.
- **A typed cell is written as its type asks**: a calendar for a date, where you can also type «next Friday»; the column's own options for a choice, each with its colour; stars for a rating; a click for yes or no; a finder for a document.
- **Sort and filter from a column's head.** That order and those filters are on screen only and leave the file alone; **Save this order** writes the rows in that order.
- **A value not written as its type asks**, like «soon» in a date column, keeps what you wrote and shows a wavy line under it.
- **Tab and Shift + Tab walk a table's cells**, and Tab past the last one starts a new row.
- **Drag the edge of a table's column** on the page to make it wider or narrower. The width is written as the dashes of its rule, ten pixels each.
- **`Alt` + ↑ ↓ with a cell open** (`Ctrl` + `Alt` on a Mac) moves its row up or down.
- **In the Markdown view**, `Alt` + ↑ ↓ moves the row the caret is on and `Alt` + ← → its column (`Ctrl` + `Alt` on a Mac), and the bar's **Table** menu sorts it and gives a column its type.
- **Rows copied from Excel, Numbers or Google Sheets** — or a CSV — come in as a table when you paste them.

A table wider than the page scrolls sideways, and its PDF and Word copies keep every column. How a type is written in the file is in the guide's [column types](https://markdown.tisty.app/tables#column-types).

## What a document can hold

Everything a document holds, from bold and lists to callouts, pens, icons, tables, diagrams, formulas, widgets, code and pages, is plain Markdown with a few short words of Tisty's own, so the file survives without Tisty. How each one is written, with its examples drawn, is in the [Tisty Markdown guide](https://markdown.tisty.app), which also opens inside Tisty with `F1`. You never need it to write: the `/` menu and the bar write it all for you.

A web link on a line of its own is drawn as a card, made from the address itself: the site's name and the words you wrote. Nothing is fetched to draw it. Press **Bring the preview** and Tisty asks that page once for its title, its description and the picture it offers, keeps the answer on this computer so it never asks again, and reads no more of the page than it needs: a page cannot decide how much of your machine it takes.

## Markup Tisty has no words for

Some Markdown from elsewhere — HTML tags — has no word in Tisty. A document that holds it still opens, edits and saves whole: nothing is cut out. Under that block the page says which part is not in the Tisty language, with **Fix** to change it in one click. The full list is in the Tisty Markdown guide, under [what is outside the language](https://markdown.tisty.app/what-is-outside-the-language).

## When a document opens read-only

The bar at the foot of the document says why.

**It is in the archive.** Archiving a document, or the folder it sits in, closes it to writing — from the window and from an assistant alike. Bring it back and it writes again.

**You locked it.** **Lock it**, in the document's menu, closes it to writing, an assistant's too; **Unlock it**, in that menu or on the bar at the foot, opens it again.

**A page of it is archived, or the document that holds it is.** See [Pages and PDF](Pages-and-PDF).

## Offline, always

Diagrams, formulas, code colouring and widgets are all drawn by code that ships inside Tisty. Nothing is fetched, no font is loaded from anywhere, and it all works with the network unplugged.
