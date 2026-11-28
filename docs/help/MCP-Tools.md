# MCP tools reference

The tools `tisty mcp` offers an agent. The agent picks them on its own; this list is so you know what it can reach. Every tool works on this machine only. The door comes open on its own until you choose — on a new install, and on an update if you never closed it — and while you keep it closed from **Settings › Agents › Door**, everything is refused.

Each tool also tells the app the agent runs in what it does to your things: whether it only reads, whether it can write over or take away something already there, and whether doing it twice changes anything more. An app can use this to ask you before the ones that write.

See [[What an agent may do|MCP-Permissions]] for the rules behind them.

## Workspace and discovery

| Tool | What it does |
| :-- | :-- |
| `workspace_catch_up` | Where things stand at session start: your lists and how many open tasks each holds, tasks in no list, folders and most used tags; the open tasks that press, by title and list, for it to tell you and read with `workspace_read` — overdue, then due soon, for today and late, five at most, the rest counted — leaving out what an agent already said is done or not to be done, which waits for you, and what you hid; what others moved since its last `workspace_catch_up`, twenty of each kind at a time, the next bringing the rest (or pass `since` for a read-only replay); and the open task it was on, with how far it got — or why it is no longer open. It names **Ideas**, and says how many ideas wait and how many are marked. On the day your [Weekly review](Tasks#weekly-review) is due, it says how much waits at each stop and how to help you |
| `workspace_search` | Find things and list taxonomy without opening full bodies (`mode`: `search`, `source`, `project`, `folder`, `taxonomy`, `in_doc` — inferred if omitted). Searches open tasks and the archive together, ten of each at a time: titles first, then recently moved, each with the newest matching line and never a replaced note; when nothing holds every word, what holds the most, saying so. `mode: "project"` with `part_of` groups a project's open parts as blocked, under way, waiting for you and not started. `mode: "folder"` lists a folder's documents with their sizes in characters. `mode: "in_doc"` searches lines inside one document with context (`around`). `mode: "taxonomy"` (or empty `{}`) returns lists, folders and tags with counts; `list: "inbox"` brings tasks in no list. What you hid stays out. Dropped tasks matching the words come apart, at most three, as decided not to do, with only title, day and reason |
| `workspace_read` | Read one `task` or one `doc` (never both), with `view`: `content` (default), `card`, `links`, `marks`, `table`, `versions`, or `markdown`. On a task, `content` returns its state first, then its page (outlined past 4,000 characters), steps (`[~]` when an agent vouched for your step), and notes after the state (`notes: 0` for the whole log); `fields` picks a subset of keys. On a document, `content` reads a section (`section`), lines (`from`/`to`), or a chunk (`chars`), returning an outline past 12,000 characters; **Ideas** comes idea by idea. `view: "card"` gives headings, pages, backlinks, tidy signs and `guessed` alike pairs (see [[Alike|Brainsty-Network#alike]]) — or a task's state, steps and parts; `view: "links"` returns neighbours of one kind (`cite`, `mention`, `structure`, `tag`, `uses`, `all`); `view: "marks"` returns only highlighted or bold passages; `view: "table"` reads a Markdown table's rows, columns and types; `view: "markdown"` gives the document whole in standard Markdown for Confluence, Jira, Notion, Linear or GitHub (see below); `view: "versions"` lists its [earlier versions](Documents#earlier-versions) on this computer, and `version` reads one under a notice (`elsewhere` versions on another computer are neither read nor restored). Pass `print` to skip unchanged reads |
| `workspace_guide` | Read Tisty's own guides progressively (`kind`: `rules`, `markdown`, or `help`). Omit `topic` for the compact index, then ask only for the topic needed (`rules`: `tasks`, `documents`, `tidying`, `ideas`, or `all`; `markdown` and `help` take `lang`: `en` or `es`) |

All 36 earlier single-purpose tool names (such as `brain`, `catch_up`, `read`, `propose`, `write_doc`, `edit_doc`, `append_doc`, `flag_doc`, `say_done`, plus `find`, `docs`, `outline_doc`, `lists`, `tags`, `read_doc`) now map cleanly onto these 15 tools. An agent that still calls an older name is told the exact replacement tool and parameters, and nothing else happens. When every row of a list in an answer shares a field, it comes once, beside the rows. How an agent uses them in a real session is in [[How an agent works|How-an-Agent-Works]].

## Tasks

| Tool | What it does |
| :-- | :-- |
| `task_create` | File a task with a `title` and `for`, whose it is: «agent» when the agent does and verifies it, «person» when only you can. Carries steps, `parts` (subtasks one level deep), `part_of`, dates, reminders, priority, list, tags and `source` — or up to 32 tasks at once in `tasks`, each with its own fields. A task made from an idea in **Ideas** names the idea as `source`, and the idea leaves the sheet. Work you dropped before is held back unless `again` is set |
| `task_update` | Update metadata on an open task: `title`, `description` (replaces whole), `date` and `deadline` (agent tasks only; `null` clears), `add_reminders` (ring times, additive), `list`, and `part_of` (hang under another task as a part, or `null` to detach) |
| `task_edit_steps` | Manage steps and state on an open task (`action`: `add`, `tick`, `untick`, `reword`, `remove`, `set_state`). On a task of yours, `tick` records the agent's word for you to confirm. `tick` and `set_state` write the task's «State» section on its page whole each time: what is done, what is left, decisions in force and what was ruled out. On `remove`, `to` moves open steps to another task and `to_person` hands human-only steps to your one companion part (created the first time), which never holds the agent's task open |
| `task_note` | Add (`action: "add"`) to a task's journal what changes something — a decision, something ruled out, a block — up to 4,000 characters (longer goes on the task's page), with `replaces` when undoing an earlier agent note (kept marked as replaced); or rewrite (`action: "reword"`) an existing journal note by ID |
| `task_resolve` | Mark an open task finished (`outcome: "done"`, with verification proof in `body` and optional `branch` or `pr`) or not to be done (`outcome: "not_doing"`, with the reason) — yours too, always for you to confirm. Never closes or deletes |

## Documents

| Tool | What it does |
| :-- | :-- |
| `doc_create` | Create a new document (`body`, optional `folder`, `page_of` to attach as a subpage one level deep, or `translation_of` + `lang` to write another language of one of your own documents; one someone else wrote is refused) |
| `doc_edit` | Edit or undo an existing document or task page (`mode`: `patch`, `append`, `rewrite`, `compact`, `digest_task_notes`, `restore`). `patch` changes one passage (`old`, or `section`/`from`+`to` with `print`). `append` adds at the end or under a heading; on a task ID it adds under assistant notes; in **Ideas** each paragraph or list item becomes a dated sticky note (<=140 chars), and pasting an existing `> [!IDEA]` into another document removes it from Ideas; in an open **Diary** it only adds to the end of today under a minute+agent heading. `rewrite` replaces the whole body with `print`; `compact` puts a document that grew by sessions back in order (see below). `digest_task_notes` folds all assistant notes on a task page into one dated digest (<=6,000 chars) keeping any lines you wrote. `restore` undoes the last replacing edit or goes back to a `version` listed by `workspace_read(view: "versions")`, or restores digested task notes |
| `doc_edit_table` | Change one table of a document or task page by its rows and columns (`action`: `set`, `add_row`, `remove_row`, `sort`, `set_type`, `rename_column`, `add_column`, `remove_column`) without touching raw Markdown. Inspect tables first with `workspace_read(view: "table")`. A value its column cannot read is kept and named in the answer |
| `doc_organize` | Organize the library (`action`: `create_folder` up to 6 levels deep, `file` one or more documents into a folder, `add_page` / `remove_page` / `reorder_pages` for one-level subpages, and `link_translation` / `unlink_translation` for multi-language document sets) |
| `doc_annotate` | Manage non-content document status (`action`: `archive`, `unarchive`, `flag`, `summarize`). `archive`/`unarchive` puts a document or page away or brings it back only when you ask. `flag` marks one or more documents (or individual ideas in **Ideas** as `sheet#heading`, with a ◆ on each note, never the whole sheet) with the reason (`duplicate_of`, `moved`, `superseded`, `wrong`, `obsolete`, `scratch`) and what stays instead — a mark, never a deletion. `summarize` leaves an agent-only gist for the next reader (syncs and backs up, never leaves through Folio or `.tistyx`, hidden in the window) |

## Files

| Tool | What it does |
| :-- | :-- |
| `file_transfer` | Transfer local files between permitted folders and Tisty (`action`: `attach` copies a file up to 650 MB into a task or document; `import_doc` brings a local Markdown file in as a document; `export_doc` writes a document and its attachments out to a folder on this machine) |

Bringing a Markdown file in copies what its text points at, but only what sits in that file's own folder or below it, so what is taken is bounded by the file you named rather than by what its text asks for. Answers come back without the paths of your disk, and an agent names what it touched by its title and its list or folder, never by an id.

## The Diary

Your [Diary](Documents#diary) is outside every tool until you open it to agents from the Diary itself: `workspace_search`, `workspace_catch_up` and the documents offered for `@` leave it out, and a tool given one of its pages answers as for a document that does not exist. Open, the reading tools read it and `doc_edit` (`mode: "append"`) adds to the end of today; every tool that would change, move, mark or put away what you wrote there is refused, and none can start a diary. Its earlier versions are never read, not even with the Diary open.

## The Agents folder

The documents in the [[Agents|AGENTS-Folder]] folder are your agents' rules, skills and memories. Through MCP the folder is called `AGENTS`, in any language. An agent reads them like any document, with `workspace_search`, `workspace_read` and `file_transfer` (`action: "export_doc"`), but does not change them on its own:

- **`doc_edit` (`rewrite`, `patch` and `append`)** on one of them does not write it: it keeps the change apart, as a proposal, and the answer says so. You approve or discard it on its card ([[A proposal from your agent|AGENTS-Folder#a-proposal-from-your-agent]]). On a memory, not even that: it is refused.
- **`workspace_read`** of a document with a proposal brings `proposal`, with `approved: false`, so the agent knows what it reads is not what reaches your agents.
- **Every other tool that names them is refused**, touching nothing: `doc_create` inside `AGENTS`, `doc_edit` (`compact`, `restore`), `doc_edit_table`, `doc_organize` (including `create_folder` inside `AGENTS` or named `AGENTS`), `doc_annotate` (`archive`, `unarchive`, `flag`, `summarize`), and `file_transfer` (`attach`, `import_doc`). It holds the same when the document comes through `doc`, `docs`, `parent`, `page_of`, `translation_of`, `after`, `before`, `order`, `instead`, or `folder`.
- **The `agent:` line**, the one that says which file a rule goes to, is never set or changed by an agent, in any document of any folder.
- **A document of another of your computers** the agent reads as data: the `workspace_read` answer carries `of_computer`, which computer it belongs to, and `read_as`, which tells it that it is not a rule of its own for this computer, and it cannot propose on it. On this computer's documents it proposes as always. There is no new tool.

The refusal tells it what it can do instead: propose the change or tell you what it would change.

## A document's earlier versions

An agent sees and goes back to the same [earlier versions](Documents#earlier-versions) you do, through two tools it already uses, and only those this computer keeps, since versions are not synced. Emptying them has no tool: that is yours. Say you asked «put "Rina's walks" back the way it was before your change this morning»; this is what it does.

**1. It lists the versions,** with `workspace_read` and `view: "versions"`:

```json
{ "doc": "k7w2pq9d-0003", "view": "versions" }
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

**2. It reads the one it wants,** with `workspace_read` and `version`:

```json
{ "doc": "k7w2pq9d-0003", "version": "7d21e8b0c9aa" }
```

Its text comes back under a notice, so it is not taken for what the document says now: «This is a past version of "Rina's walks" …, not what it says now». It carries no `print`, so it cannot be used to write over with `doc_edit`. A long version reads in parts, like any document.

**3. It goes back to it,** with `doc_edit` (`mode: "restore"`) and `version`:

```json
{ "doc": "k7w2pq9d-0003", "mode": "restore", "version": "7d21e8b0c9aa" }
```

The document goes back to saying what that version said, over everything written since, without asking for `even_if_more`, because the version was named on purpose. What it replaces is kept, the answer tells it in `went_back_to`, and `doc_edit` (`mode: "restore"`) without `version` undoes the go-back.

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

If your agent also reaches Confluence, Jira, Notion, Linear or GitHub, it can read a Tisty document with `workspace_read` (`view: "markdown"`) and hand it over: standard Markdown suits almost all of them, and their own tools turn it into their format.

Not everything makes the journey, and Tisty says so in each answer, so the agent tells you before it publishes:

- **Attached files** stay in Tisty. `file_transfer` (`action: "export_doc"`) writes the document out with them into a folder, and from there the other tool can upload them. It never leaves them inside an agent's folder, such as `~/.claude` or the one `CLAUDE_CONFIG_DIR` names, even when it sits in Documents: only [[Agents|AGENTS-Folder]] writes there.
- **Diagrams** stay as Mermaid code: GitHub, GitLab and Notion draw them; Confluence and Jira show them as code unless a Mermaid macro is used.
- **Links to other documents and tasks** become plain words.
- **Pages** come apart: each one is read on its own.
- **Pens, underlines, icons and folds** travel as HTML tags, which not every tool draws.
- **A very long document**, past 40,000 characters in standard Markdown, is not handed over in the conversation: `file_transfer` (`action: "export_doc"`) writes it out as files instead.
- **A document put away, or marked as having had its day,** says so first, so the agent asks you before carrying it anywhere.

## Tidying what grew

An agent updates what exists instead of piling up new pages. When a document grew by sessions — a page per review, plans of finished work, copies — it can compact it: it reads every page, moves what is worth keeping to where it belongs, marks the pages that leave with their reason, and writes the document again in one go, with what holds today first, the decisions with their dates and a history of what moved where. It never states as fact what it could only read in old text; that stays dated and marked unconfirmed.

You stay in charge of a compaction:

- The window shows **Compacted by … on …** on the document, with **Undo** and **Keep**. Undo puts the document back exactly as it was; you can also ask the agent to restore it.
- Until you keep or undo it, the agent writes nothing else in that document, except one more compaction to correct its own.
- A document you began is compacted only when you ask for it in the conversation.
- Pages and copies it marked stay where they are, with the reason, until you delete them, put them away or take the mark off. Deleting or taking the mark off works one by one, or for all of a set in one click.
