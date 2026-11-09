# Documents

Documents live beside the tasks, for reference material that has no date and never gets ticked. They are plain Markdown files, and search reads them too. A task can point at a document; a document never creates tasks.

For writing in one, see [[The Markdown editor|Markdown-Editor]]; for everything it can hold — callouts, tables, diagrams, math, widgets, code — see the [Tisty Markdown guide](https://markdown.tisty.app).

![Documents beside the tasks, with formulas, diagrams, code and small widgets drawn inside Tisty](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/store/en-03-notes.png)

## A task or a document?

Tisty keeps two kinds of things, and telling them apart is most of using it well.

- **A task is something to do.** It ends, often soon: you complete it and it goes to the archive. But it does not go away. What you wrote on its page, its steps and its notes stay, and months later a search finds it and tells you how you solved it. A task is knowledge with a date.
- **A document is something to keep.** A manual, a recipe, the notes on a client or a machine, a year of minutes: it is read again and again, corrected and added to, and it grows for years. It rarely ends.

That is why a task has a page of its own: a task is also a document, one with an end. Take a **spike** — finding out how something can be done before doing it:

1. It starts as a task, «Find out how to sync with OneDrive». On its page you write what you read, what you tried and what you ruled out, section by section, and you go back and correct it as you learn. You attach what a single description could never hold: the recording of the meeting, a video of the fault, the code analysis, the PDF from the vendor.
2. When the way is clear, the work is split into **parts**: «Sign in», «Upload large files», «Resume a cut upload». Each one is a task with its own steps and notes.
3. The parts are completed one by one, and then the spike itself. It goes to the archive.
4. Nothing is lost. A year later, «how did we solve the OneDrive uploads?» finds it: its page, its parts and every step, as they were left.

```mermaid
flowchart TD
  A[Something comes up] --> B{Will you do it,<br/>or come back to read it?}
  B -->|Do it| T[A task]
  B -->|Read it again| D[A document]
  T --> P[Its page: notes, steps,<br/>files, recordings]
  P --> S{Too big for one go?}
  S -->|Yes| K[Split it into parts]
  K --> C
  S -->|No| C[Complete it]
  C --> R[The archive:<br/>search finds how you solved it]
  P -.->|You learned something<br/>you will need again| D
  D --> G[Read, correct and add to it<br/>for years]
```

Something you will do is a task. Something you will come back to is a document — a recipe is not a task with steps, and a to-do is not a document. When doing a task teaches you something you will need again beyond it — the fix for a fault that returns, how a supplier wants an order — put it in a document as well.

How a task's page, steps and parts work is in [[Tasks]]; what a closed one keeps, in [[The archive|Archive]].

## Making one

Go to Documents and use the **+** at the top: **New document**. The same button inside a folder makes it there. **New diagram** makes a document holding a diagram and opens the [[diagram window|Diagrams]]. **New folder** groups them, and a folder can hold folders of its own.

The first line of the document is its title — there is no name to fill in.

`Ctrl`/`⌘` + `X` and `V` move a document between folders.

## Folders

- **Every folder starts folded**, so a store with hundreds of documents is a handful of rows until you go looking.
- **Archive a whole folder** with the right button when work is over: it drops to the **Archived** shelf at the foot of the tree with its icon, subfolders and pages exactly as you left them. There it reads and exports, but nothing writes into it and nothing new goes in.
- **Bring it back** and it returns the same — including documents you had archived by hand in there, which stay archived.

## Tags

A document takes tags the way a task does: write `#contract` in the middle of a sentence and the document is filed under it. The tag lives in the sentence, not a hidden header, so taking the file to another editor takes the tag with it. A note brought from elsewhere with `tags:` in its [properties](https://markdown.tisty.app/properties) is filed under those too.

- The **Tags** screen shows the tasks under a tag and the documents beneath them. Pressing a tag inside a document opens exactly that.
- How a tag is written, which letters it takes and what does not count as one, is in the [Markdown guide](https://markdown.tisty.app/tags).

## Attachments

Drag a file into a document or onto a task to attach it — the photo from the vet, the insurance PDF, the recording of a meeting. A task takes files up to 650 MB, the same as a document. It is kept as itself, inside Tisty, and travels with the document when you export it.

An attachment is drawn as what it is: a picture as itself, a sound or a video with its player, and any other file, an `.html` page among them, as a card with its kind and size.

A sound or a video already on this computer plays from here, without touching your cloud. One that is still only in your cloud shows **Bring it from your cloud** with its size, and comes down only when you press it; once here, it stays. If it cannot come — no connection, or it never arrived — the player says why and offers **Try again**. A picture that cannot come says so the same way.

The **⋯** on a card saves a copy, turns the card into a plain link, or takes it off the page. On the card of a page, it can also make that page a document of its own.

## Pages

A long document in parts — a year of minutes, a book by chapters — is one document holding pages. See [Pages and PDF](Pages-and-PDF).

## Several languages

A document can be told in several languages: each language is a document of its own, and Tisty shows them as one, in the language of the app, with a switch in the title panel, or above the sheet when the panel is shut, that keeps your place.

- **Translate into…**, in the document's menu, writes a copy of its words and pages in the language you pick, ready to translate section by section. If the document does not say its language yet, it asks.
- **Join with its other language…** puts two documents you already have together, in either order: it asks each one's language by its name, and **Take out of its languages** parts them again.
- The languages of a document stand in one folder, and move, rest and go together. Searching finds them once, open in the language the words turned up in.
- Taking a document out writes `translationKey:` in its properties, and bringing it back in joins it to the others again.

## Importing

**Import a document** in the tree takes a `.md`, `.markdown`, `.txt`, `.html`, `.htm` or Word `.docx` file and brings it in, one at a time. A web page or a Word document comes in as Tisty Markdown, with its headings, lists, links, tables and colours (a colour becomes the nearest pen of the palette), and a Word document's pictures come in with it as attachments. A `.tistyx` from another Tisty unpacks whole.

Word, HTML, Google Docs, rich text, plain text and Obsidian notes all come in as Markdown, with their pictures and files: [Convert to and from Markdown](Convert-to-and-from-Markdown) tells each way in.

Tasks have to be typed: there is no importer for another program's task list.

## Opening a Markdown file from outside

Tisty does not need to own a file to show it. Pick Tisty under **Open with** for a `.md` or `.markdown` file — it never takes over as the default program — or follow a `tisty://open?path=…` link, and the file opens **read only** in a light viewer of its own, with the system's title bar, without coming into your store. It reads up to 1 MB, about a million characters, and says so when a file is heavier.

- **The column beside the page** holds **Edit**, **See Markdown** (`Ctrl`/`⌘` + `Shift` + `M`; **See the document** returns from it), the file's headings and its details: its folder, size, date and properties. In a narrow window it folds away, and its button in the bar (`Ctrl`/`⌘` + `\`) opens it.
- **The bar** has **Find in the text** (`Ctrl`/`⌘` + `F`), the theme and **More**, which also holds larger and smaller text, **Copy Markdown**, **Print** and **Show in folder**. The theme and your place in the file are remembered.
- **It reads the file as it was written for other programs.** A centred header stays centred, a badge sits in its line, lines an editor cut at a column join up, and `#ff0000` or `[[x]]` stay as words. Diagrams and formulas are drawn, and so is a [widget](Widgets), since nothing in it runs.
- **Pictures** beside the file show when they sit in its folder or below it. Pictures from the internet wait: a line above the page says how many, and **Show them** fetches them. It asks again each time you open the file ([what that sends](Privacy#the-network-requests-tisty-makes)).
- **Links**: a link to another `.md` beside it opens in the same window, with **Back** (`Alt` + ←). A web link opens your browser.
- **Edit** makes a copy in your store, written in Tisty Markdown, and opens it; the file stays as it is and the viewer closes. A file over 500 KB can be read but not edited, and **Edit** says so. If a file beside it could not come along, the viewer stays open and says how many.
- **On a Tisty not set up yet**, the file reads at once and **Edit** makes no copy: it brings up the welcome, and the viewer says to finish it first. When the welcome ends, the copy is made and opened.

A `tisty://open` link reaches only a readable text file on this computer.

### A document in a window of its own

**Open in a window of its own**, under **Window** in a document's panel, shows it read only beside your work.

- **Contents and page** (`Ctrl`/`⌘` + `\`) opens a column with its headings, what the document is, and the width of the page: how wide it reads, and the paper of its PDF. **Sheets**, first in that row, unlocks the paper and shows it on sheets, **Portrait** or **Landscape**, and its PDF comes out that way; the file does not change.
- **Reading** and **Markdown** switch between the drawn page and its text.
- **More** has **Find in the text**, larger and smaller text, **Copy as Markdown** (standard Markdown), **Preview** of the PDF it would make — **← Back to the document** returns from it — **Export to PDF…**, **Export as HTML…**, **Export as Word…** and **Duplicate**, which makes a copy.
- **Edit in Tisty** takes you back to the document. Its PDF there carries who wrote it and when unless you untick **Signed** in the preview.

## Taking documents out

A document leaves whole — every conversion is in [Convert to and from Markdown](Convert-to-and-from-Markdown):

- **Copy as Markdown** — standard Markdown, ready for any other editor.
- **Write it out to a folder**, as standard Markdown files with its pages and attachments beside it; a link to one of its pages points at that page's file.
- **Markdown to HTML** — **Export as HTML…**, in the document's menu, makes one web page that travels alone.
- **Markdown to Word** — **Export as Word…** makes a `.docx` with its headings, lists, tables and pictures.
- **Export for Tisty** into a `.tistyx` file, which also carries what Markdown cannot say: folders with their order, icon and colour, which document each page hangs from, what is archived, and the alias it was signed with. It includes a `README.txt`, so whoever receives it can read the documents without Tisty.
- **Markdown to PDF** — see [Pages and PDF](Pages-and-PDF#tistys-own-pdf).

Exporting all of them asks who they are for: **open**, **to hand to somebody**, or **locked with six digits for another machine of your own** — which is what makes them arrive there as yours rather than as a stranger's. Either way, not one line of the history travels inside.

## Sharing

The panel beside a document has a **Shared** row with **Share…**. Sharing goes through Folio, a service apart from Tisty. Pressing **Share…** reads a small signed file to learn whether Folio has opened. While it has not, it says so, with a link to [tisty.cl](https://tisty.cl), and nothing of your document leaves this computer; the ways out above still hand a document to someone. If that file cannot be reached, it says so and offers to try again. Once Folio is open, you make your identity first if this computer has none yet — the key that signs what you share, backed up with your 24-word phrase — then accept its terms the first time, choose what the other person can do and how long it lasts, and get a link. If Folio changes its terms, Share and Sync ask you to accept the new ones as soon as they can be checked; until then you keep sharing under the ones you accepted.

Once a document is shared, its row has **Sync**, and nothing goes up until you press it. It sends only what changed, and if nothing did it only brings in a newer version someone else sent. Folio takes a new version of each document after a short wait, and a few new documents a day, each document it cites included: while a wait runs, the button says how long is left and whether changes are waiting, and the dialog says before you share when a document and what it cites would not fit today.

Its row says who created it, who edited it last and everyone who edited it, each under the name they chose and signed by their own key; a ✓ marks the maker of Tisty. Whoever edits through the edit link chooses between their alias and their name. To take that power from everyone who has the link, press **Change the edit link**: theirs stops sending versions, they keep their copies, and you give the new link to whoever should keep editing. What reached you through a read link stays a copy of your own: Sync brings newer versions into it, and only an edit link lets you send yours. Stop sharing a document others edited and share it again later, and the dialog reminds you who they were first.

## Your alias

Write an alias in Settings — optional, and yours to choose — and every document is signed with it, while whatever arrives from somebody else keeps theirs. It travels with your sync and inside any parcel, so pick a name you are content for others to read. Your real name is never asked for.
