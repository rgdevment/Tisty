# Brainsty: everything joined

This sheet is a page of the guide. It tells you about Brainsty, Tisty's brain: how to join a document to another one and to its tasks while you write, and how to see everything you keep as one network. Almost all of it can be tried right here, on this guide.

What you keep was already joined: a document cites another, a task was born in some minutes, a book has pages. Brainsty makes those ties visible, lets you walk them, and serves them to your agent too.

## Linking a document to another

Type `[[` where you want the link and go on with a few words of the title. Tisty offers up to eight documents that match, ignoring accents and capitals, those that start that way first. Pick one with `Enter` and a link with its title stays, pointing at the document and not at its name: if you rename it, the link still gets there.

![The menu two brackets open in Rina's note: the documents that match what was typed, each with its folder](linking.png)

**Try it:** at the end of this paragraph type `[[rina` and pick **Rina**, this guide's sample page. You get a link like this one: [Rina](tisty:page).

To land on a section, follow the title with `#`: `[[what is#tasks` offers the sections of this guide's cover that hold that word, and the link opens the guide right at **Tasks**. If that section stops existing, the link opens the document at the top.

Rest the pointer on a link to another document and its preview appears: its first lines, or the section it points at. To open it while you write, `Ctrl` + click (`⌘` + click on a Mac); a plain click only puts the caret there.

## Who links to it

The panel beside a document has a **Linked from** row: every document and every task that cites or mentions it, with the section where it does. A click takes you there.

**Try it:** open Rina's page with the link above and, in the panel beside it, open **Linked from**: this sheet is there, and the guide's cover. Tisty builds that list in the background, so a link you just wrote shows up a few seconds later.

## Citing a task

What you know about a subject lives in its document, and the work you did on it, in its tasks. Cite the task in the document and you find it where you will look for it. In the `/` menu, **A task** searches the open and the archived ones, and leaves its card alone in its paragraph.

![The card of «Renew the car insurance» in the «Car» document, with its steps unfolded, and the one of «Change the tyres», closed](card.png)

The card reads the task every time it is drawn: its list, its dates, how many steps are done and, if it closed, the day. A click opens it; its circle marks it done; its count unfolds the steps to tick them right there. A closed task is read, but no longer changes from the document.

**Try it:** on an empty line of this sheet type `/`, pick **A task** and write a word of one of your tasks. If you have none yet, the quick capture is one `Ctrl` + `Shift` + `Space` away.

## Creating a task without leaving the document

In some minutes or a plan a «we need to…» comes up. In the `/` menu, **A new task** opens the quick capture's field right there: write the task the way you would say it and `Enter` creates it with its date, its list, its tags and its priority, and leaves its card in that place.

**Try it:** on an empty line, below here, type `/task`, `Enter`, and then `Walk Rina and Roger on Saturday #rina`. Open the task with a click on its card: beside its page, **Born in** names this sheet, and a click brings you back. `Esc` in the field lets it go without creating anything.

On a task's page, **A new task** creates a part of that task, in its same list.

## The network

**Brainsty**, in the sidebar under the task views, opens everything you keep as one network. It starts on what you were looking at and, when you close it with `Esc` or **Back**, the window is just as it was.

![The network zoomed out: each folder and each list in its colour, documents as squares, tasks as circles and tags as purple dots](network.png)

| In the network | It is |
| --- | --- |
| Square | a document or a page, in its folder's colour |
| Circle | a task, in its list's colour: bright while under way, faint once done |
| Purple dot | a tag |
| Grey dot | a picture or a file |
| Solid line | a citation, a card |
| Dashed line | a mention, a link in a sentence |
| Thin line | a page and its book, a part and its task, the document a task was born in |

Lists and folders are not dots: they are the colour and the zone where their things gather.

The network changes with the distance. From afar, with the wheel backwards, its dots make a brain seen from the side, with no titles. At middle distance the coloured zones show. Up close, every title, each kind of line, the tags and each picture's thumbnail.

![«Rina» picked up close: her neighbours with titles, the rest dimmed and, beside it, the document's preview](network-near.png)

- **A click** on a dot lights it up with its neighbours and opens its preview; **Open**, or a double click, opens it.
- **Search the network**, at the top, or `/`: type part of a title and `Enter` centres the network on it.
- **Filters**, the funnel beside the search box: which kinds to show, the state of tasks, a tag and **Touched**, to see only what changed in the last days.
- **With the keyboard**: the arrows walk from a dot to its neighbour, `Enter` opens and `Esc` lets go, first the search, then the dot and last the network.

**Try it:** open **Brainsty** now. The network starts on this sheet, with its neighbours around it: the guide's cover, which it is a page of, and what names it or what it names. Search `one of` and you reach **One of each**.

## What your agent reads

Your agent reads the same network, but as an index. With a single tool, `brain`, it finds tasks and documents by their words, the Archive beside the rest; it brings a document's card, with its sections, who cites it and its decisions in force first, without reading it whole; and it sees a project grouped as blocked, under way, waiting for you and not started. When it names a task or a document by its exact title between quotes, Tisty links it when saving, as when you do. [Your agent's page](tisty:agent) tells it step by step.

## Good to know

- **Everything stays on this computer.** The network is worked out here and rebuilds itself: it is not synced and does not leave with what you share or export. Each computer draws it again from your documents and tasks.
- **What you hide does not show** in the network, nor does it reach your agent.
- **Outside Tisty**, a link to another document and a task's card stay only as their words: in standard Markdown, in Word, in the PDF and in Folio.
- **Up to 12,000 dots.** Past that, the network keeps what you touched last, its neighbours, your documents and the tasks under way, and tells you how much was left out.

Every detail, with an example from start to end, is in the help: [linking documents](https://www.tisty.app/docs/linking-documents/), [tasks in documents](https://www.tisty.app/docs/tasks-in-documents/) and [the Brainsty network](https://www.tisty.app/docs/brainsty-network/).
