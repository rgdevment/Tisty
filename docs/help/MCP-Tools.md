# MCP tools reference

The tools `tisty mcp` offers an agent. The agent picks them on its own; this list is so you know what it can reach. Every tool works on this machine only. The door comes open on its own until you choose — on a new install, and on an update if you never closed it — and while you keep it closed from **Settings › Agents › Door**, everything is refused.

Each tool also tells the app the agent runs in what it does to your things: whether it only reads, whether it can write over or take away something already there, and whether doing it twice changes anything more. An app can use this to ask you before the ones that write.

See [[What an agent may do|MCP-Permissions]] for the rules behind them.

## Looking around

| Tool | What it does |
| :-- | :-- |
| `rules` | The rules for working here, one guide at a time («tasks», «documents», «tidying», «ideas», or «all»); alone, the basics and an index. Agents ask for the one they need first |
| `language` | The Tisty language, so what it writes is something Tisty knows |
| `help` | How Tisty itself works, from these same help pages, so it answers instead of guessing |
| `catch_up` | Where things stand, with the lists and how many open tasks each holds, the tasks in no list, the folders and most used tags; the open tasks that press, by title and list, for it to tell you and `read` — overdue, then due soon, for today and late, five at most, the rest counted — leaving out what an agent already said is done or not to be done, which waits for you, and what you hid; what others moved since its last `catch_up`, twenty of each kind at a time, the next bringing the rest; and the open task it was on, with how far it got — not started, under way with how many steps are ticked and how many an agent said done, blocked, or with a state older than its work — or, if it no longer is, why: it said it is done or should not be done, or you closed, dropped or hid it. It names **Ideas**, and says how many ideas wait and how many are marked. On the day your [Weekly review](Tasks#weekly-review) is due, it says how much waits at each stop and how to help you |
| `brain` | Find things and see how they connect, without opening them. Tasks and documents by their words, the archive beside the rest, ten of each at a time: first what holds the words in its title, a title saying exactly those words ahead, then what moved most recently, each with the newest line that says them and never a note a later one replaced; when nothing holds every word, what holds the most, saying so. The card of one document — its headings, pages, what cites it and the signs it wants tidying — or of one task — its state, steps and parts — and its neighbours of one kind when asked. The open parts of a project, grouped as blocked, under way, waiting for you and not started. The documents of a folder, each with its size in characters, so an agent sees which is long before it opens it. With nothing asked, your lists, folders and tags with their counts; `list` «inbox» brings the tasks in no list. A list that does not exist is refused, naming yours, and one put away still answers. Filtering tasks, an answer that does not bring them all says how many are open and archived and how many press. On the card, apart and under `guessed`, what Tisty thinks is alike and nobody linked, with what they share and `method` (`words`, `typo` or `dictionary`): see [[Alike|Brainsty-Network#alike]]. What you hid or dropped stays out, even as a neighbour or an alike pair |
| `read` | Read a task: its state first, then its page, its steps and the notes written after that state, with who wrote each. A task with no state yet comes with one built from it: how many steps are done and the note that said it done. A step of yours an agent said done comes as `[~]`, «said done, waiting on the person», and is counted apart from the ticked ones until you take its word. Its page does not copy the journal, and the answer says how many notes it holds, page or not. A page past about 4,000 characters comes as its sections, with the digest and the latest of the agent's notes; the journal brings up to about 3,000 characters of notes, and the whole log when asked. Or read a document, or one part of it — or, with `as`, the whole of it in standard Markdown, to take to Confluence, Jira, Notion, Linear or GitHub through their own tools (see below); with `marks`, only what you highlighted or set in bold, under each section. **Ideas** comes in parts, idea by idea, never as a list of times. Read again with the `print` the last read gave, and if nothing changed it says only that. With `versions`, in place of the text, it lists its [earlier versions](Documents#earlier-versions), newest first: when each was written, by whom and its size; a task's id lists its page's. With `version` it reads one, under a notice that it is not what it says now. A version another of your computers replaced comes as `elsewhere`, with that computer's name, and is neither read nor gone back to |

`find`, `docs`, `outline_doc`, `lists` and `tags` moved into `brain`, and `read_doc` into `read`. An agent that still calls one of them by name is told where it went, and nothing else happens. When every row of a list in an answer shares a field, it comes once, beside the rows. How an agent uses them in a real session is in [[How an agent works|How-an-Agent-Works]].

## Tasks

| Tool | What it does |
| :-- | :-- |
| `propose` | Propose a task with a `title` and `for`, whose it is: «agent» when the agent does it, «person» when only you can. It carries its steps, date, list and source — or several at once in `tasks`, each carrying its own fields. A task made from an idea in **Ideas** names the idea as its source, and the idea leaves the sheet, as it does when you turn it into a task |
| `remind` | Set a task to ring at an hour |
| `reschedule` | Move the day of a task an agent filed |
| `describe` | Write the description of an open task, yours too, replacing the old one |
| `rename` | Rename any open task, yours too |
| `note` | Add to a task's journal what changes something — a decision, something ruled out, a block — up to 4,000 characters a note; longer goes on the task's page. A note that undoes an earlier one from an agent says so, and the old one stays in the history marked as replaced |
| `reword_note` | Rewrite a note in a task's journal |
| `plan` | Add steps to a task |
| `reword_step` | Rewrite an open step |
| `unplan` | Take open steps off a task; with `to`, hand them to the task they belong on, and with `to_person`, to your companion task, made the first time |
| `move` | Hang a task under another as a part, let a part go, or move a task to another list |
| `tick` | Tick the steps it did; on a task of yours they wait for you to confirm. In the same call it leaves the task's state, a «State» section on its page written whole each time: what is done, what is left, the decisions in force and what was ruled out |
| `untick` | Take back a tick it gave by mistake |
| `say_done` | Say a task is finished, yours too — for you to confirm, with the branch (`branch`) or pull request (`pr`) when there is one |
| `say_not_doing` | Say a task should not be done, yours too — for you to confirm |

## Documents

| Tool | What it does |
| :-- | :-- |
| `sum_up` | Leave a short summary of what a document is about, for the next reader. It replaces the one before, travels to your other computers with sync and goes into your backups; it never leaves through Folio or in a `.tistyx`, and the window does not show it |
| `write_doc` | Write a new document, or write one again whole — or, with `compact`, put one that grew by sessions back in order (see below); with `translation_of`, write it as another language of one of your own documents (one someone else wrote is refused) |
| `append_doc` | Add to a document, at the end or under a heading; on a task's page, add under the agent's notes, or with `replace` fold every agent's notes into one dated digest of at most 6,000 characters that keeps any line you wrote and `restore_doc` can undo. In **Ideas**, each paragraph or list item becomes an idea of its own, dated and coloured, of at most 140 characters; the idea itself, its `> [!IDEA]` line and its words as the sheet has them, added to another document takes it off the sheet, as filing it from the window does, and an `[!IDEA]` note that names no idea there stays a sticky note of its own. In the **Diary**, once you open it to agents, it only adds to the end of today, under a line with the minute and the agent's name |
| `edit_doc` | Change one passage of a document, or of a task's page |
| `edit_table` | Read or change one table of a document by its rows and columns: set a cell, add, remove or sort rows, rename a column or give it a type. A value its column cannot read is kept and named in the answer |
| `restore_doc` | Undo the last change that replaced a document's text, a changed passage or a whole write, from its [earlier versions](Documents#earlier-versions) on this computer; appends and your saves leave nothing to undo, and it is refused if something was written since. Undoing twice returns it. With `version` it goes back to one that `read` lists, over everything written since, and what it replaces is kept; without `version` it undoes that go-back. With a task, put back the notes a digest replaced |
| `folder` | Make a folder |
| `file_doc` | Put one or several documents in a folder |
| `page_doc` | Make documents into pages of another or take them out (`doc`), or put the pages of one in order (`order` with `page_of`) |
| `tongue_doc` | Join a document to the languages of another, or take it out of its own |
| `archive_doc` | Put a document or one of its pages away, or bring it back, only when you ask |
| `flag_doc` | Mark one or several documents that had their day, with the reason — a copy of another, moved, replaced, wrong, obsolete, a scratch — and what stays instead. You delete them, put them away or take the mark off: a mark, never a deletion. It marks the ideas in **Ideas** one by one the same way, with a ◆ on each note, and never the whole sheet: the mark takes no idea off, and Ideas itself is never put away, filed in a folder or joined to another document's languages by an agent |
| `import_doc` | Bring a Markdown file on this machine in as a document |
| `export_doc` | Take a document out to a folder on this machine |

## Files

| Tool | What it does |
| :-- | :-- |
| `attach` | Keep a copy of a file with a task or inside a document |

Bringing a Markdown file in copies what its text points at, but only what sits in that file's own folder or below it, so what is taken is bounded by the file you named rather than by what its text asks for. Answers come back without the paths of your disk, and an agent names what it touched by its title and its list or folder, never by an id.

## The Diary

Your [Diary](Documents#diary) is outside every tool until you open it to agents from the Diary itself: `brain`, `catch_up` and the documents offered for `@` leave it out, and a tool given one of its pages answers as for a document that does not exist. Open, the reading tools read it and `append_doc` adds to the end of today; every tool that would change, move, mark or put away what you wrote there is refused, and none can start a diary. Its earlier versions are never read, not even with the Diary open.

## The Agents folder

The documents in the [[Agents|AGENTS-Folder]] folder are your agents' rules, skills and memories. Through MCP the folder is called `AGENTS`, in any language. An agent reads them like any document, with `brain`, `read` and `export_doc`, but does not change them on its own:

- **`write_doc`, `edit_doc` and `append_doc`** on one of them do not write it: they keep the change apart, as a proposal, and the answer says so. You approve or discard it on its card ([[A proposal from your agent|AGENTS-Folder#a-proposal-from-your-agent]]). On a memory, not even that: it is refused.
- **`read`** of a document with a proposal brings `proposal`, with `approved: false`, so the agent knows what it reads is not what reaches your agents.
- **Every other tool that names them is refused**, touching nothing: `page_doc`, `tongue_doc`, `attach`, `flag_doc`, `archive_doc`, `file_doc`, `sum_up`, `restore_doc`, an `edit_table` that changes, and `folder` inside `AGENTS` or with that name. It holds the same when the document comes through `doc`, `page_of`, `translation_of`, `of`, `after`, `before`, `order`, `instead`, `folder` or `inside`.
- **The `agent:` line**, the one that says which file a rule goes to, is never set or changed by an agent, in any document of any folder.
- **A document of another of your computers** the agent reads as data: the `read` answer carries `of_computer`, which computer it belongs to, and `read_as`, which tells it that it is not a rule of its own for this computer, and it cannot propose on it. On this computer's documents it proposes as always. There is no new tool.

The refusal tells it what it can do instead: propose the change or tell you what it would change.

## A document's earlier versions

An agent sees and goes back to the same [earlier versions](Documents#earlier-versions) you do, through two tools it already uses, and only those this computer keeps, since versions are not synced. Emptying them has no tool: that is yours. Say you asked «put "Rina's walks" back the way it was before your change this morning»; this is what it does.

**1. It lists the versions,** with `read` and `versions`:

```json
{ "doc": "k7w2pq9d-0003", "versions": true }
```

The answer brings the newest first, how many this computer keeps and its limit in days (cut down here):

```json
{
  "title": "Rina's walks",
  "kept_here": 2,
  "kept_days": 30,
  "versions": [
    {
      "version": "a3f9c2e17b04",
      "written": "2026-11-18 10:42",
      "by": "Claude Code",
      "until": "2026-11-18 11:05",
      "bytes": 3120
    },
    {
      "version": "7d21e8b0c9aa",
      "written": "2026-11-17 19:10",
      "by": "the person",
      "until": "2026-11-18 10:42",
      "bytes": 2988
    }
  ]
}
```

`written` is when that text was written and `until`, how long the document said it. `version` is the start of its print, enough to name it. A task's id lists its page's versions. They come 40 at a time, and `next` says where to ask for older ones with `cursor`.

**2. It reads the one it wants,** with `read` and `version`:

```json
{ "doc": "k7w2pq9d-0003", "version": "7d21e8b0c9aa" }
```

Its text comes back under a notice, so it is not taken for what the document says now: «This is a past version of "Rina's walks" …, not what it says now». It carries no `print`, so it cannot be used to write over with `edit_doc` or `write_doc`. A long version reads in parts, like any document.

**3. It goes back to it,** with `restore_doc` and `version`:

```json
{ "doc": "k7w2pq9d-0003", "version": "7d21e8b0c9aa" }
```

The document goes back to saying what that version said, over everything written since, without asking for `even_if_more`, because the version was named on purpose. What it replaces is kept, the answer tells it in `went_back_to`, and `restore_doc` without `version` undoes the go-back.

**A version another computer keeps** comes in the list with `elsewhere` and that computer's name:

```json
{
  "version": "c0ffee12ab34",
  "written": "2026-11-16 08:30",
  "by": "the person",
  "until": "2026-11-16 21:15",
  "bytes": 2950,
  "elsewhere": "Mario's laptop"
}
```

Reading it or going back to it is refused and changes nothing: «that version … was written over on Mario's laptop, not on this machine». The agent tells you which computer has it, so you open it there.

## Taking a document to another tool

If your agent also reaches Confluence, Jira, Notion, Linear or GitHub, it can read a Tisty document with `as: "markdown"` and hand it over: standard Markdown suits almost all of them, and their own tools turn it into their format.

Not everything makes the journey, and Tisty says so in each answer, so the agent tells you before it publishes:

- **Attached files** stay in Tisty. `export_doc` writes the document out with them into a folder, and from there the other tool can upload them. It never leaves them inside an agent's folder, such as `~/.claude` or the one `CLAUDE_CONFIG_DIR` names, even when it sits in Documents: only [[Agents|AGENTS-Folder]] writes there.
- **Diagrams** stay as Mermaid code: GitHub, GitLab and Notion draw them; Confluence and Jira show them as code unless a Mermaid macro is used.
- **Links to other documents and tasks** become plain words.
- **Pages** come apart: each one is read on its own.
- **Pens, underlines, icons and folds** travel as HTML tags, which not every tool draws.
- **A very long document**, past 40,000 characters in standard Markdown, is not handed over in the conversation: `export_doc` writes it out as files instead.
- **A document put away, or marked as having had its day,** says so first, so the agent asks you before carrying it anywhere.

## Tidying what grew

An agent updates what exists instead of piling up new pages. When a document grew by sessions — a page per review, plans of finished work, copies — it can compact it: it reads every page, moves what is worth keeping to where it belongs, marks the pages that leave with their reason, and writes the document again in one go, with what holds today first, the decisions with their dates and a history of what moved where. It never states as fact what it could only read in old text; that stays dated and marked unconfirmed.

You stay in charge of a compaction:

- The window shows **Compacted by … on …** on the document, with **Undo** and **Keep**. Undo puts the document back exactly as it was; you can also ask the agent to restore it.
- Until you keep or undo it, the agent writes nothing else in that document, except one more compaction to correct its own.
- A document you began is compacted only when you ask for it in the conversation.
- Pages and copies it marked stay where they are, with the reason, until you delete them, put them away or take the mark off. Deleting or taking the mark off works one by one, or for all of a set in one click.
