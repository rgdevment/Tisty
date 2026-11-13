# MCP tools reference

The tools `tisty mcp` offers an assistant. The assistant picks them on its own; this list is so you know what it can reach. Every tool works on this machine only. The door comes open on its own until you choose — on a new install, and on an update if you never closed it — and while you keep it closed from **Settings › Assistants**, everything is refused.

Each tool also tells the app the assistant runs in what it does to your things: whether it only reads, whether it can write over or take away something already there, and whether doing it twice changes anything more. An app can use this to ask you before the ones that write.

See [[What an assistant may do|MCP-Permissions]] for the rules behind them.

## Looking around

| Tool | What it does |
| :-- | :-- |
| `rules` | The rules for working here, one guide at a time («tasks», «documents», «tidying», «ideas», or «all»); alone, the basics and an index. Assistants ask for the one they need first |
| `language` | The Tisty language, so what it writes is something Tisty knows |
| `help` | How Tisty itself works, from these same help pages, so it answers instead of guessing |
| `catch_up` | Where things stand, with the lists, folders and most used tags; the open tasks that press, by title and list, for it to tell you and `read` — overdue, then due soon, for today and late, five at most, the rest counted — leaving out what an assistant already said is done or not to be done, which waits for you, and what you hid; what others moved since its last `catch_up`, twenty of each kind at a time, the next bringing the rest; and the open task it was on or, if it no longer is, why: it said it is done or should not be done, or you closed, dropped or hid it. It names **Brainstorm**, and says how many ideas wait and how many are marked |
| `find` | Search tasks and documents, the archive included; documents marked as having had their day come after the live ones, saying so |
| `read` | Read a task: its page, its steps and its latest notes, with who wrote each. A page past about 4,000 characters comes as its sections, with the digest and the latest of the assistant's notes; the journal brings the latest notes up to about 3,000 characters, more when asked |

`lists` and `tags` still answer an assistant that asks for them by name, but they are no longer offered: `catch_up` already brings both.

## Tasks

| Tool | What it does |
| :-- | :-- |
| `propose` | Propose a task with a `title`, and its steps, date, list and source — or several at once in `tasks`, each carrying its own fields. A task made from an idea on **Brainstorm** names the idea as its source, and the idea leaves the sheet, as it does when you turn it into a task |
| `remind` | Set a task to ring at an hour |
| `reschedule` | Move the day of a task an agent filed |
| `describe` | Write the description of an open task, yours too, replacing the old one |
| `rename` | Rename any open task, yours too |
| `note` | Add to a task's journal: up to 4,000 characters a note; longer goes on the task's page |
| `reword_note` | Rewrite a note in a task's journal |
| `plan` | Add steps to a task |
| `reword_step` | Rewrite an open step |
| `unplan` | Take open steps off a task — or, with `to`, hand them to the task they belong on |
| `move` | Hang a task under another as a part, let a part go, or move a task to another list |
| `tick` | Tick the steps it did; on a task of yours they wait for you to confirm |
| `untick` | Take back a tick it gave by mistake |
| `say_done` | Say a task is finished, yours too — for you to confirm |
| `say_not_doing` | Say a task should not be done, yours too — for you to confirm |

## Documents

| Tool | What it does |
| :-- | :-- |
| `docs` | The documents and the folders, with what each one is about |
| `outline_doc` | What is in a document: its headings and parts, and the signs it wants tidying — documents titled alike, headings repeated across pages, links that lead nowhere |
| `read_doc` | Read a document, or one part of it — or, with `as`, the whole of it in standard Markdown, to take to Confluence, Jira, Notion, Linear or GitHub through their own tools (see below). **Brainstorm** comes in parts, idea by idea, never as a list of times |
| `sum_up` | Leave a short summary of what a document is about, for the next reader |
| `write_doc` | Write a new document, or write one again whole — or, with `compact`, put one that grew by sessions back in order (see below); with `translation_of`, write it as another language of one of your own documents (one someone else wrote is refused) |
| `append_doc` | Add to a document, at the end or under a heading; on a task's page, add under the assistant's notes, or with `replace` fold every assistant's notes into one dated digest of at most 6,000 characters that keeps any line you wrote and `restore_doc` can undo. On **Brainstorm**, each paragraph or list item becomes an idea of its own, dated and coloured, of at most 140 characters; the idea itself, its `> [!IDEA]` line and its words as the sheet has them, added to another document takes it off the sheet, as filing it from the window does, and an `[!IDEA]` note that names no idea there stays a sticky note of its own |
| `edit_doc` | Change one passage of a document, or of a task's page |
| `edit_table` | Read or change one table of a document by its rows and columns: set a cell, add, remove or sort rows, rename a column or give it a type. A value its column cannot read is kept and named in the answer |
| `restore_doc` | Put a document back the way it was, from the copy kept beside the documents; with a task, put back the notes a digest replaced |
| `folder` | Make a folder |
| `file_doc` | Put one or several documents in a folder |
| `page_doc` | Make documents into pages of another or take them out (`doc`), or put the pages of one in order (`order` with `page_of`) |
| `tongue_doc` | Join a document to the languages of another, or take it out of its own |
| `archive_doc` | Put a document or one of its pages away, or bring it back, only when you ask |
| `flag_doc` | Mark one or several documents that had their day, with the reason — a copy of another, moved, replaced, wrong, obsolete, a scratch — and what stays instead. You delete them, put them away or take the mark off: a mark, never a deletion. It marks the ideas on **Brainstorm** one by one the same way, with a ◆ on each note, and never the whole sheet: the mark takes no idea off, and Brainstorm itself is never put away, filed in a folder or joined to another document's languages by an assistant |
| `import_doc` | Bring a Markdown file on this machine in as a document |
| `export_doc` | Take a document out to a folder on this machine |

## Files

| Tool | What it does |
| :-- | :-- |
| `attach` | Keep a copy of a file with a task or inside a document |

Bringing a Markdown file in copies what its text points at, but only what sits in that file's own folder or below it, so what is taken is bounded by the file you named rather than by what its text asks for. Answers come back without the paths of your disk, and an assistant names what it touched by its title and its list or folder, never by an id.

## Taking a document to another tool

If your assistant also reaches Confluence, Jira, Notion, Linear or GitHub, it can read a Tisty document with `as: "markdown"` and hand it over: standard Markdown suits almost all of them, and their own tools turn it into their format.

Not everything makes the journey, and Tisty says so in each answer, so the assistant tells you before it publishes:

- **Attached files** stay in Tisty. `export_doc` writes the document out with them into a folder, and from there the other tool can upload them.
- **Diagrams** stay as Mermaid code: GitHub, GitLab and Notion draw them; Confluence and Jira show them as code unless a Mermaid macro is used.
- **Links to other documents and tasks** become plain words.
- **Pages** come apart: each one is read on its own.
- **A document put away, or marked as having had its day,** says so first, so the assistant asks you before carrying it anywhere.

## Tidying what grew

An assistant updates what exists instead of piling up new pages. When a document grew by sessions — a page per review, plans of finished work, copies — it can compact it: it reads every page, moves what is worth keeping to where it belongs, marks the pages that leave with their reason, and writes the document again in one go, with what holds today first, the decisions with their dates and a history of what moved where. It never states as fact what it could only read in old text; that stays dated and marked unconfirmed.

You stay in charge of a compaction:

- The window shows **Compacted by … on …** on the document, with **Undo** and **Keep**. Undo puts the document back exactly as it was; you can also ask the assistant to restore it.
- Until you keep or undo it, the assistant writes nothing else in that document, except one more compaction to correct its own.
- A document you began is compacted only when you ask for it in the conversation.
- Pages and copies it marked stay where they are, with the reason, until you delete them, put them away or take the mark off. Deleting or taking the mark off works one by one, or for all of a set in one click.
