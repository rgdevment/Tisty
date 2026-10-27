# Convert to and from Markdown

Tisty turns Word, HTML, Google Docs, rich text, plain text and Obsidian notes into Markdown, and turns Markdown back into Word, HTML, PDF and standard Markdown. There is no converter to install: every way in ends as [Tisty Markdown](https://markdown.tisty.app), and every way out starts from it. The file you bring in is never touched.

## Word to Markdown (`.docx` to `.md`)

Import the `.docx` from **Import a document** in the tree, or copy from Word and paste into a document. Headings, lists, tables, links, colours and pictures come along: a colour becomes the nearest pen of the palette, and the pictures come in as attachments. Lines in a typewriter font such as Courier or Consolas come in as code.

## HTML to Markdown

Import an `.html` or `.htm` page, or paste a selection copied from a web page or an email. Headings, lists, bold, links, tables and coloured words are kept; what HTML says that Markdown cannot becomes Markdown or plain words.

## Google Docs to Markdown

Copy from Google Docs and paste into a document. It comes in like a web page: headings, lists, links, tables and colours.

## Rich text to Markdown

Text copied with its formatting — from a web page, an email or a word processor — keeps its headings, lists, bold, links, tables and coloured words when you paste it. A colour becomes the nearest pen of the palette.

## Plain text to Markdown

Import a `.txt` file, or paste. A table a terminal drew with box lines (`┌─┬─┐`) becomes a Markdown table, and lines that read like code — from a code editor or a terminal — come in as a code block, unless the caret is already inside one.

## Obsidian to Markdown

Import the note. Its own forms become standard ones: its callouts become the nearest of the five kinds, without folding; `%%comments%%` become `<!-- -->`; `![[file]]` becomes a picture or a file link; and a note named by title — `[[Plan]]`, `[[Plan|the plan]]`, `![[Plan]]` — becomes a link or a card to the document here that bears that title. When no document bears it, or more than one does, it stays as a reference with its name, and Tisty says so; import the other note first and its links find it. Front matter, footnotes and everything else the language takes stay as written.

## What comes in with it

Pictures and files the document names beside it — in its own folder or one below — come in with it and are kept inside Tisty, and the text points at the copies. One that is not there, or sits outside that folder, is left as its plain words, and Tisty says how many could not come in. Another note it links to is not copied in as a file: it is linked if a document here already bears its title, and otherwise its name stays as a reference and Tisty reminds you to import that note on its own. A link to a web page, an email address or another app (`tel:`, `obsidian://`…) stays a link.

What has no word in Tisty Markdown is turned into one on the way in; the [guide](https://markdown.tisty.app/what-is-outside-the-language) lists it.

## Markdown to Word (`.md` to `.docx`)

**Export as Word…**, in the document's menu, makes a `.docx` with its headings, lists, tables, links, pens and pictures inside, and its code in Consolas, so it comes back in as code if you import it again. Each page and each page break starts a new sheet.

## Markdown to HTML

**Export as HTML…** makes one web page that travels alone: the document and its pages in order, with the look of the page and its pictures inside.

## Markdown to PDF

Tisty prints its own PDF, with its pages, page breaks and paper size. See [Pages and PDF](Pages-and-PDF#tistys-own-pdf).

## Markdown to standard Markdown

**Copy as Markdown** gives standard Markdown, the way GitHub and CommonMark read it, ready for any other editor. Tisty's own forms become the standard ones (`:::tip` becomes `> [!TIP]`, `:::details` becomes `<details>`, a single line break stays a break), a link to another document becomes its words, and every tag of the document — its `tags:` and its `#hashtags` — lands in the front matter. **Write it out to a folder** does the same as files, with its pages and attachments beside it.

More ways to take documents out, a `.tistyx` for another Tisty among them, are in [Documents](Documents#taking-documents-out).
