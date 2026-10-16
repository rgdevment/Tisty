# The Tisty language

A Tisty document is a plain `.md` file. What it holds is Markdown, the same Markdown GitHub and Obsidian read, plus a handful of short words of Tisty's own for what Markdown has no way to say: a pen colour, an underline, an icon, a section that folds. This page lists every one of them: what it is for, how it is written in the file, how it reads, and the small rules that decide when it counts.

Nothing on this page needs learning by heart. The `/` menu and the bar in the editor write all of it for you; this is what you find if you open the file somewhere else, or want to write it by hand.

## At a glance

| What | Written | GitHub | Obsidian |
| :-- | :-- | :-: | :-: |
| [Headings](#headings) | `# Title` … `###### Title` | yes | yes |
| [Bold, italic, struck, code](#bold-italic-struck-through-and-code) | `**b**` `*i*` `~~s~~` `` `c` `` | yes | yes |
| [Underline](#underline) | `++words++` | no | no |
| [Pens](#pens) | `==words==` `==words=={green}` | no | yellow only |
| [Links](#links) | `[words](https://…)` `[words][one]` `https://…` | yes | yes |
| [Line breaks](#paragraphs-and-line-breaks) | one line per line | joins them | yes |
| [Lists and stages](#lists) | `- item` `1. item` `- [ ] stage` | yes | yes |
| [Quotes and callouts](#quotes-and-callouts) | `> words` `> [!NOTE]` | yes | yes |
| [Collapsible sections](#collapsible-sections) | `:::details Title` … `:::` | no | no |
| [Tags](#tags) | `#contract` | as words | yes |
| [Icons](#icons) | `::dog orange::` | no | no |
| [Emoji](#emoji) | `🎉` or `:tada:` | yes | `🎉` only |
| [Other documents and pages](#other-documents-and-pages) | `![Title](tisty:doc/…)` | no | no |
| [Attachments](#attachments) | `![Label](file-1a2b3c4d.pdf)` | no | no |
| [References](#references) | `[[Name]]` `[[#2]]` | no | yes |
| [Footnotes](#footnotes) | `[^1]` … `[^1]: note` | yes | yes |
| [Code](#code-blocks) | ` ```lang title="name" ` | yes | yes |
| [Diagrams](#diagrams-formulas-and-widgets) | ` ```mermaid ` | yes | yes |
| [Formulas](#diagrams-formulas-and-widgets) | ` ```math ` `$x$` `$$x$$` | yes | `$` only |
| [Widgets](#diagrams-formulas-and-widgets) | ` ```widget ` | as code | as code |
| [Tables](#tables) | `\| a \| b \|` | yes | yes |
| [Page breaks](#page-breaks) | `---` | as a rule | as a rule |
| [Comments and entities](#comments-and-entities) | `<!-- note -->` `&copy;` | yes | yes |

Where the table says *no*, the other reader shows the words exactly as they are written, so nothing is lost: `++words++` stays readable, and `::dog::` is still a dog.

## The title

The first line with words in it is the document's title. There is no name to fill in anywhere. A callout's marker and a code fence are stepped over to find it.

```markdown
# Rina's vaccines
```

## Headings

Six levels, from `#` to `######`, always followed by a space. The `/` menu offers the first three.

```markdown
## Vaccines

### Rabies
```

- `# Hola` is a heading; `#Hola`, with no space, is a [tag](#tags).
- To start a line with a `#` that is neither, write `\#`.

## Paragraphs and line breaks

A blank line starts a new paragraph. A single line break stays a line break: what you write on the next line is drawn on the next line, in the same paragraph.

```markdown
A line ends where you end it.
This line is the same paragraph, one line down.

This one is a new paragraph.
```

GitHub joins those two lines into one when it shows a file; Tisty and Obsidian keep them apart. A backslash at the end of a line breaks it everywhere.

To write a line that would otherwise start something — a heading, a list, a quote — put a backslash before it: `\#`, `\-`, `1\.`, `\>`.

## Bold, italic, struck through and code

```markdown
**Bold**, *italic*, ***both at once***, ~~struck through~~ and `loose code`.
```

- A mark touching a space on its inner side is not a mark: `** bold **` stays as written, and so does `5 * 3`.
- A word joined by underscores, like `snake_case` or `__dunder__`, is not italic or bold.
- Inside backticks nothing else counts: a tag, a pen or an emoji between them is just text.

## Underline

Markdown has no underline, so Tisty has two plus signs.

```markdown
++underlined++
```

- Exactly two `+` on each side. The words inside cannot start or end with a space.
- The walls cannot touch a letter, a digit or another `+` on their outer side, so `C++`, `i++` and `a ++ b ++ c` are never underlines.
- An underline cannot cross the edge of a bold or a link: both walls sit inside the same one.
- Other readers show `++underlined++` as it is. When a document leaves Tisty as plain Markdown — **Copy as Markdown**, or written out to a folder — the walls are dropped and the words stay.

## Pens

A highlight, in a colour. Yellow is the plain one; any other colour is named in braces right after the closing wall.

```markdown
==yellow==, ==green=={green}, ==blue=={blue} and ==pink=={pink}.
```

The bar offers four pens — **yellow**, **green**, **blue** and **pink** — and the file takes eight more colours: `red`, `orange`, `amber`, `teal`, `indigo`, `purple`, `brown` and `gray`.

- Exactly two `=` on each side: `===` and longer are something else. The words inside cannot start or end with a space.
- The colour goes straight after the closing `==`, with no space: `==words=={green}`. `=={yellow}` is the same as no colour.
- A colour the palette does not have, like `{chartreuse}`, leaves a yellow pen with `{chartreuse}` written after it, in sight.
- A pen cannot cross the edge of a bold or a link.
- Obsidian draws `==words==` in yellow and shows `{green}` as words. When the document leaves Tisty as plain Markdown, pens are written as `<mark>` so other readers can still paint them.

## Links

```markdown
A [link to a website](https://github.com/rgdevment/Tisty), an address
between angle brackets <https://github.com/rgdevment/Tisty>, and a mail
address the same way: <rina@example.com>.
```

- A web address written on its own, `https://github.com/rgdevment/Tisty`, is a link too. A full stop, a comma or a closing bracket right after it is not part of it.
- A link can also be written by reference, with its address said once on a line of its own, anywhere in the document:

  ```markdown
  The [walk map][park] and the [vet][vet].

  [park]: https://example.com/park
  [vet]: https://example.com/vet "Open on Saturdays"
  ```

- A link with no address yet, `[words]()`, reads as its words.
- Only web addresses, `mailto:` and Tisty's own `tisty:` are followed. Any other scheme reads as plain words.
- An address with spaces goes between angle brackets: `[clip](<C:/Users/me/a clip.mkv>)`.
- A link opens your browser only when you click it.

## Lists

```markdown
- Lead
- Poo bags
  - The green ones
- Water

1. Put her lead on
2. Walk to the park

3. A numbered list keeps the number it starts on
```

- Bullets are `-`, `*` or `+`; numbers are `1.` or `1)`. Indent an item to nest it.
- An item can open on words or on another block — a quote, a heading, a picture, a fence — the way Markdown has it.

### Stages

A list of boxes you tick. They are not Tisty tasks: they never close, have no date and never show up in *Today*.

```markdown
- [x] Bring Rina's record book
- [ ] Ask about the tick tablet
```

`[ ]` is open, `[x]` (or `[X]`) is ticked. A click on the box ticks it, on the page and while you write.

## Quotes and callouts

A quote is a line starting with `>`.

```markdown
> What is not written down did not happen.
```

A callout is a quote whose first line names its kind in brackets. Five kinds, each with its own colour:

```markdown
> [!NOTE]
> For what is worth knowing, with no urgency at all.

> [!TIP]
> For the shortcut you found the hard way.

> [!IMPORTANT]
> For what cannot be passed over.

> [!WARNING]
> For what can go wrong if you do not look.

> [!CAUTION]
> For what has no way back.
```

- The kind is read in any case: `[!note]` is a note.
- GitHub wants the marker alone on its line; Tisty also takes words after it, `> [!TIP] Take the spare roll.`, and those words become the callout's first line.
- A kind Tisty does not know, like `[!FOO]`, leaves an ordinary quote.
- Nearly anything fits inside a callout — lists, code, rules, a section — as long as every line carries its `>`.
- The same five kinds can also be written the way other tools write them, as a block between `:::tip` and a lone `:::`; Tisty draws it as the same callout.

## Collapsible sections

A title you click to open or fold what is under it. Pick **Collapsible section** in the `/` menu, or write it:

```markdown
:::details Rina's full record

Born in 2023. Six-in-one and rabies every March.

- Worming every three months

:::
```

- The opening line, `:::details` and its title, stands as a paragraph of its own: a blank line before and after it. So does the closing `:::`.
- Without its closing `:::` it is not a section: both lines stay as written.
- A section starts folded. With no title after `:::details`, it shows an empty one.
- Sections nest. Any `:::word` line opens a container that the next lone `:::` closes, so a section inside another, or a `:::tip` block written by another tool, never steals its closing fence. `details` folds and the five callout kinds draw a callout; any other `:::word` block shows its lines as written.
- They work inside a callout too.
- GitHub and Obsidian show both fence lines as text and everything between them unfolded. Tisty also draws a `<details>` block written in HTML, the way GitHub writes them, but that is HTML, which is [outside the language](#what-is-outside-the-language).

## Tags

A word after `#` files the document under that tag. It lives in the sentence, so it travels with the file.

```markdown
Booster due in March #rina #vet
```

- `#Hola` is a tag; `# Hola`, with a space, is a heading.
- Accents and case come off: `#Camión` and `#camion` are one tag, `camion`. An underscore becomes a dash: `#mi_tag` is `mi-tag`.
- Two characters at least, one of them a letter: `#1234` is a ticket number, not a tag, and neither is `#a`. `#2026x` is a tag.
- A `#` stuck to what comes before it is not a tag: `C#`, `bg-#fff`, `page.html#top` and the anchor in a link's address are all left alone.
- Nothing inside code is a tag, and neither is the `#2` in a [reference](#references) like `[[#2]]`.
- A document keeps its first 64 tags.
- While you write, typing `#` offers the tags you already use, so one tag is never spelled two ways.

## Icons

An icon marks a line without spending a heading on it. Pick **An icon** in the `/` menu and search it by what it draws — «dog», «motorcycle», «fist» — in your language. It is written between double colons, with an optional colour after a space:

```markdown
::dog orange:: Rina's vaccines
::star:: Done
```

- The name is one from Tisty's set, in English: lowercase, with dashes (`dog`, `bone`, `heart`, `rocket-thing`). The picker knows them all.
- The colours are `red`, `orange`, `amber`, `green`, `teal`, `blue`, `indigo`, `purple`, `pink`, `brown` and `gray`.
- An icon cannot touch a letter or a digit: `a::star::b` stays as written.
- A name or a colour Tisty does not have stays as written, `::nothing::` included.
- Other readers show `::dog orange::` as words. When the document leaves Tisty as plain Markdown, an icon is written as a small piece of HTML whose words are its name between colons, `:dog:`.

## Emoji

An emoji is best written as itself: 🎉. A shortcode GitHub knows, such as `:tada:`, `:+1:` or `:dog:`, is drawn as its emoji too, and stays written as the shortcode.

- A shortcode cannot touch a letter, a digit or another colon: `10:30:00` and `a:tada:b` stay as written.
- One Tisty does not know stays as words.

## Other documents and pages

Another document is named by its id after `tisty:doc/`. The `/` menu writes these for you (**A document**, **A new document**, **A page that already exists**, **A new page**).

```markdown
![Rina's vet card](tisty:doc/01JC4M7Q3B8YH2A9K6TN5VXZRD)

The [walks of October](tisty:doc/01JC4M9F2W7DQ1E3S8RB6PKMHT) went better.
```

- With `!` in front and alone in its paragraph, it is a **card**: the document's current title, its first words, and — for a page — its place in the book. The words in brackets are only what other readers show.
- The same thing inside a sentence is a word you can click.
- For a page, only the card gives it its place in the book; a link in a sentence points at it without placing it. See Pages and PDF.
- A `[`, `]` or `\` in a title is written with a backslash before it.

## Attachments

A file you drag in is kept inside Tisty and named by a stamp of its bytes. It is written like a picture, with the label you see:

```markdown
![Rina on her chair](rina-on-her-chair-3f9a1c2be8d47a60.jpg)

![Rina's vet card](vet-card-9d02e4b7c1f3a85e.pdf)
```

- A picture is drawn; any other file is a chip with its label that opens it.
- The stamp is 8 to 16 lowercase hexadecimal characters after the last dash, and the whole name is lowercase.
- Documents written by an older Tisty that say `attachments/ab/name` still read; when the document is rewritten, only the name is kept.
- When a document is written out to a folder, its attachments go beside it and the links point at them.

## References

Two square brackets name something without linking to it.

```markdown
See [[Rina's record]] and step [[#2]].
```

- `[[name]]` is drawn as a chip with the name.
- In a task's description or journal, `[[#2]]` shows the words of that task's second step.
- An empty name, or one with a `[` or a line break in it, is not a reference.
- Obsidian reads `[[name]]` as a link to a note of that name.

## Footnotes

A note at the foot of the text is called by a short label in square brackets with a caret, and said once on a line of its own:

```markdown
The vet confirmed it[^vet].

[^vet]: Visit of 3 March, with the blood test.
```

- The call is drawn as a small raised label that takes you to the note; the note sits where it is written, with its label beside it.
- A label is any short word or number: `[^1]`, `[^vet]`.
- GitHub and Obsidian read them the same way.

## Code blocks

A fence of three or more backticks (or tildes), the language after it, and optionally a name:

````markdown
```bash title="backup.sh"
rsync -a ~/Tisty/ /Volumes/Backup/Tisty/
```
````

- The name goes in `title="…"`; a `"` inside it is written `\"`. Other readers ignore it.
- Whatever else other tools write after the language, like `showLineNumbers`, is kept as written; Tisty reads the language and the name.
- To show a fence inside a block, use a longer fence around it.
- A block indented four spaces is code too.
- Nothing in code is a tag, a pen, an icon or an emoji. See Code blocks.

## Diagrams, formulas and widgets

Three languages are drawn instead of shown as code.

````markdown
```mermaid
graph TD
  R[Rina] --> V[Vaccines]
```

```math
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
```

```widget title="Rina this month"
<div class="stat"><small>Walks</small><b>42</b></div>
```
````

- `mermaid` (or `mmd`) draws a diagram. One too large to lay out — past 32 KiB, or a thousand boxes and arrows — or one that does not parse, is shown as code.
- `math` sets a formula. Past 8 KiB it is shown as code; a command it does not know is skipped.
- `widget` draws a small, sealed-off page.

A formula inside a sentence goes between single dollars: `$E=mc^2$`.

- A digit right after the closing dollar makes it a price, not a formula: `$5 and $10` stays as written.
- `\$` is a dollar sign.
- A formula between double dollars, `$$…$$`, stands on a line of its own, like a `math` block:

  ```markdown
  $$
  \int_0^1 x\,dx = \tfrac{1}{2}
  $$
  ```

GitHub draws all three fences except `widget`, `$…$` and `$$…$$`. Obsidian draws `mermaid`, `$…$` and `$$…$$`, and shows a `math` block as code.

## Tables

```markdown
| Vaccine        | Given | Due again |
| :------------- | :---: | --------: |
| Six-in-one     | march | 12 months |
| Rabies         | march | 12 months |
```

- Colons in the rule under the header lean a column: `:---` left, `:---:` centre, `---:` right.
- A column's width rides in that rule: four dashes or more set it, ten pixels a dash; three leave it to the page. Other readers ignore how many dashes there are. See Tables.
- A `|` inside a cell is written `\|`.
- A cell holds a line of words and marks, not blocks.

## Page breaks

A rule on its own line splits the sheet: on screen it looks like a break, and in print or PDF what follows starts on the next sheet.

```markdown
What follows starts on the next sheet.

---

This is already on the next one.
```

Leave a blank line above it: right under a line of words, `---` turns that line into a heading.

## Comments and entities

A note to yourself that the page does not show goes between `<!--` and `-->`, on a line or across several. An entity names a character that is awkward to type: `&copy;` is ©, `&amp;` is &, `&nbsp;` is a space that does not break.

```markdown
Booster due in March. <!-- ask about the tick tablet -->

&copy; 2026 Rina's vet
```

- Both are kept in the file exactly as written; the page shows what they mean.
- Inside code they are just text.

## What is outside the language

A few things other Markdown writes have no word in Tisty. A document that holds them still opens, edits and saves whole, nothing is cut out; but that part is drawn as best the page can, and it shows as it is written while you edit it:

- front matter at the top of the file (`---` … `---`),
- HTML tags of any kind, in blocks or in a line.

**Settings › Maintenance** goes through every document and lists the ones that hold something of this.

### Words from an older Tisty

Earlier versions wrote pens as `<mark data-pen="green">`, icons as `<span data-ico="dog">` and underlines as `<u>`. Those are turned into the words on this page when the documents come in, every other byte left as it was.
