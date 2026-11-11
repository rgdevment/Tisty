use serde_json::{Value, json};

/// Every tool says its arguments the same way, and the three lines that say so were three lines
/// in each of twenty-three places.
fn shaped(what: Value) -> Value {
    let mut kept = serde_json::Map::new();
    kept.insert("type".into(), json!("object"));
    kept.insert("additionalProperties".into(), json!(false));
    if let Value::Object(more) = what {
        for (key, one) in more {
            kept.insert(key, one);
        }
    }
    Value::Object(kept)
}

/// Vetting a call needs only what each tool takes, not the hints.
#[allow(clippy::too_many_lines)]
pub(super) fn tools_without_hints() -> Value {
    json!([
        {
            "name": "propose",
            "title": "Propose a task",
            "description": "Propose a task, something to do; what is kept to read again is `write_doc`. Several from one thread go in `tasks`, each judged alone. Work of several tasks with one end goes in `parts`, not packed into steps. Tisty tags it #agent.",
            "inputSchema": shaped(json!({
                "properties": {
                    "tasks": { "type": "array", "items": { "type": "object" }, "description": "Several tasks, each shaped like a single one; left out for one" },
                    "title": { "type": "string", "description": "What has to be done, in a line" },
                    "parts": { "type": "array", "items": { "type": "object" }, "description": "Its parts, each shaped like a single task with its own steps: one level, at most 32 here, more later with `part_of`" },
                    "part_of": { "type": "string", "description": "Id of a task you may work on that this one is a part of" },
                    "description": { "type": "string", "description": "What you read, in the Tisty language" },
                    "date": { "type": "string", "description": "The day it is meant to be done, or with a deadline the day work starts (2026-08-31)" },
                    "deadline": { "type": "string", "description": "The day it truly runs out (2026-08-31); only then is it overdue" },
                    "remind": { "type": "array", "items": { "type": "string" }, "description": "When to ring, like 2026-08-30T20:00: only for a one-off at a set hour that cannot be caught up on, the evening before" },
                    "priority": { "type": "string", "enum": ["do", "decide", "delegate", "minor"], "description": "Only if someone said so" },
                    "list": { "type": "string", "description": "An existing list, by name (`lists`). Left out, it lands in the inbox, which needs a reason" },
                    "tags": { "type": "array", "items": { "type": "string" }, "description": "Its subjects, one plain word each, reusing what `tags` lists; six is already a lot" },
                    "steps": { "type": "array", "items": { "type": "string" }, "description": "A checklist, each step at most 100 characters; what needs more is a part" },
                    "source": { "type": "string", "description": "Where you read it: a message id, a thread link" },
                    "again": { "type": "boolean", "description": "Only when the person asks in this conversation to redo what was filed from this source and closed since, never for one still open; say in the description how the last one ended" }
                }
            }))
        },
        {
            "name": "remind",
            "title": "Set a task to ring",
            "description": "Make a task ring at given moments. It adds to what the task rings at and never removes a reminder (`read` shows them). Not for what repeats: a routine comes back on its own.",
            "inputSchema": shaped(json!({
                "properties": {
                    "task": { "type": "string", "description": "The task id" },
                    "at": { "type": "array", "items": { "type": "string" }, "description": "Each a day and an hour, like 2026-08-30T20:00" }
                },
                "required": ["task", "at"]
            }))
        },
        {
            "name": "describe",
            "title": "Describe a task",
            "description": "Write the description of a task you may work on (one an agent filed or the person opened to agents). It replaces the old one whole: `read` first and keep what holds; what you learnt goes in `note`.",
            "inputSchema": on_a_task(
                json!({ "body": { "type": "string", "description": "What the task is, in the Tisty language" } }),
                &["body"],
            )
        },
        {
            "name": "plan",
            "title": "Add steps to a task",
            "description": "Add steps to a task you may work on, after those already there (`read` shows them).",
            "inputSchema": on_a_task(
                json!({ "steps": { "type": "array", "items": { "type": "string" }, "description": "The steps to add, in order, each at most 100 characters" } }),
                &["steps"],
            )
        },
        {
            "name": "tick",
            "title": "Tick the steps you did",
            "description": "Tick steps you did yourself, by their text, as you finish each. All or nothing: one unknown text ticks none; a text two steps share ticks the next open one. On the person's tasks not open to you, it reports the step done for them to confirm. Refused once the task is said done: `note` instead. The whole task is `say_done`; a wrong tick, `untick`.",
            "inputSchema": on_a_task(steps_by_text("you did"), &[])
        },
        {
            "name": "untick",
            "title": "Take back a tick given by mistake",
            "description": "Take back a tick, or your word on a step, an agent gave by mistake. A tick the person gave is theirs, and naming it unticks nothing. It opens and closes nothing.",
            "inputSchema": on_a_task(steps_by_text("to untick"), &[])
        },
        {
            "name": "rename",
            "title": "Rename a task an agent filed",
            "description": "Retitle a task an agent filed, when the title no longer says what it is. The person's own tasks are refused: offer the title in a `note`. On a task you may work on, steps are `reword_step` and the page `describe`.",
            "inputSchema": on_a_task(
                json!({ "title": { "type": "string", "description": "The new title" } }),
                &["title"],
            )
        },
        {
            "name": "reword_step",
            "title": "Rewrite an open step",
            "description": "Rewrite an unticked step of a task you may work on. A ticked step is refused: `untick` it first if your tick was a mistake.",
            "inputSchema": on_a_task(
                json!({
                    "step": { "type": "string", "description": "The step, by its text as `read` shows it" },
                    "text": { "type": "string", "description": "What the step says now, at most 100 characters" }
                }),
                &["step", "text"],
            )
        },
        {
            "name": "unplan",
            "title": "Take open steps off a task",
            "description": "Take unticked steps off a task you may work on. With `to` they are handed on, after that task's own steps, instead of dropped. One ticked step among them and nothing changes.",
            "inputSchema": on_a_task(
                {
                    let mut fields = steps_by_text("to take off");
                    fields["to"] = json!({
                        "type": "string",
                        "description": "Id of another open task you may work on, not said done, to hand them to"
                    });
                    fields
                },
                &[],
            )
        },
        {
            "name": "move",
            "title": "Hang a task under another, or move it to a list",
            "description": "On a task you may work on: `part_of` hangs it under another such task, or null lets it stand alone; `list` moves a task an agent filed to a better list. One level: a task with parts is no part. A closed task takes nothing; a whole said done or not doing takes no new part.",
            "inputSchema": on_a_task(
                json!({
                    "part_of": { "type": ["string", "null"], "description": "Id of the task to hang it under, or null to let it go" },
                    "list": { "type": "string", "description": "An existing list, by name (`lists`), for a task an agent filed" }
                }),
                &[],
            )
        },
        {
            "name": "reword_note",
            "title": "Rewrite a note in a task's journal",
            "description": "Rewrite a note in the journal of a task you may work on, named by the id `read` shows beside it. The new body replaces it whole and cannot be empty.",
            "inputSchema": on_a_task(
                json!({
                    "note": { "type": "string", "description": "The note's id, from `read`" },
                    "body": { "type": "string", "description": "What the note says now, in the Tisty language" }
                }),
                &["note", "body"],
            )
        },
        {
            "name": "note",
            "title": "Add to a task's journal",
            "description": "Add to a task's journal, the person's tasks too, when something new turns up about existing work, instead of filing a duplicate. A closed task takes none: propose the work anew. A note inside a document is a callout (`> [!NOTE]`), with `edit_doc`.",
            "inputSchema": shaped(json!({
                "properties": {
                    "task": { "type": "string", "description": "The task id" },
                    "body": { "type": "string", "description": "What to record, in the Tisty language" }
                },
                "required": ["task", "body"]
            }))
        },
        {
            "name": "attach",
            "title": "Attach a file to a task or a document",
            "description": "Copy a file from this machine into Tisty: with `task` onto its journal, with `doc` at the end of that document as a picture or a card. Up to 650 MB; an HTML page is kept, never run. Attach only what you were asked to.",
            "inputSchema": shaped(json!({
                "properties": {
                    "task": { "type": "string", "description": "The task id, to keep it with a task" },
                    "doc": { "type": "string", "description": "The document's id, to add it there" },
                    "path": { "type": "string", "description": "A path to a file on this machine" },
                    "label": { "type": "string", "description": "What to call it; defaults to the file's name" }
                },
                "required": ["path"]
            }))
        },
        {
            "name": "write_doc",
            "title": "Write a document",
            "description": "Write a document, something kept and read again; work to do is `propose`. Markdown is turned into the Tisty language and `turned` says what changed. Alone it writes a new one: `find` first, as a subject with a document is updated there, and a title in use is refused. With `doc` and `print` it rewrites that one whole, page lines the body drops going back at the end; one passage is `edit_doc`. #words in its text tag it.",
            "inputSchema": shaped(json!({
                "properties": {
                    "keep_compaction": kept_compaction_field(),
                    "body": { "type": "string", "description": "The whole document; its first line is its title. One line per paragraph: a single newline is a line break, so wrapped text reads broken" },
                    "doc": { "type": "string", "description": "Id of a document to write again whole, not its title; needs `print`. Left out, a new one" },
                    "print": { "type": "string", "description": "The `print` `read_doc` gave with the text you worked from; if the document changed since, nothing is written" },
                    "folder": { "type": "string", "description": "The existing folder its subject belongs in, by name, path or id; left out needs a reason. Not with `doc`: `file_doc` moves one" },
                    "compact": { "type": "boolean", "description": "With `doc` and `print`: this write puts the document back in order, as `rules` «tidying» says. Refused on one the person began unless `person_asked`" },
                    "person_asked": { "type": "boolean", "description": "With `compact`: the person asked you in this conversation to compact a document they began" },
                    "anyway": { "type": "boolean", "description": "Write it although a document names the same subject; only when it truly is another" },
                    "page_of": { "type": "string", "description": "Id of the document this is a page of: it is named at that document's end (`page_doc` moves the line), takes its folder and holds no pages" },
                    "translation_of": { "type": "string", "description": "Id of a document this one says in another language, joined as one more of its languages; both say their language, in `lang` or a `lang:` property" },
                    "lang": { "type": "string", "description": "With `translation_of`: the body's language tag, like `es`, `en` or `pt-BR`" }
                },
                "required": ["body"]
            }))
        },
        {
            "name": "append_doc",
            "title": "Add to a document, or to a task's page",
            "description": "Add at a document's end, or under the heading in `under`, without reading it or a `print`; nothing written changes. For logs, minutes and dated notes; a changed fact is `edit_doc`. The answer says how much it `grew`. A task's id in `doc`, on a task you may work on, adds under the assistant's notes on its page, dated and signed.",
            "inputSchema": shaped(json!({
                "properties": {
                    "keep_compaction": kept_compaction_field(),
                    "doc": { "type": "string", "description": "A document's id, not its title, or a task's to add to its page" },
                    "body": { "type": "string", "description": "Markdown to add, after a blank line" },
                    "under": { "type": "string", "description": "A heading, as it reads: adds at the end of what it holds. Left out, at the end of the document" },
                    "echo": echo_field(),
                    "replace": { "type": "boolean", "description": "Only with a task's id: rewrite the assistant's notes on its page whole, as one dated digest; a box in it is no step" }
                },
                "required": ["doc", "body"]
            }))
        },
        {
            "name": "restore_doc",
            "title": "Put a document back the way it was",
            "description": "Undo the last write to a document: a changed passage or a whole write is put back; appends and the person's saves keep nothing to undo. Refused if written since, unless `even_if_more`. Undoing twice returns it. Use it when a write answers something unexpected, instead of retyping.",
            "inputSchema": shaped(json!({
                "properties": {
                    "doc": { "type": "string", "description": "A document's id, not its title, or a task's to put back the assistant's notes on its page" },
                    "even_if_more": { "type": "boolean", "description": "Go back over what was written since too; only once you read it and know, since the person may have written it" }
                },
                "required": ["doc"]
            }))
        },
        {
            "name": "edit_doc",
            "title": "Change a passage of a document",
            "description": "Replace one passage of a document, named by its text in `old` or by place with `section` or `from` and `to`, which needs no reading of the rest. What it said is kept first for `restore_doc`, or the edit is refused. The answer's `grew` should be close to what you sent.",
            "inputSchema": shaped(json!({
                "properties": {
                    "keep_compaction": kept_compaction_field(),
                    "doc": named_doc_field(),
                    "old": { "type": "string", "description": "The passage as written now, from `read_doc`, matching one place only; take in a line around it if needed. Not with `section` or lines" },
                    "new": { "type": "string", "description": "What takes its place; empty takes the passage out" },
                    "section": { "type": "integer", "description": "A heading and all under it, by its `at` in the `outline` of `outline_doc`, from 0. Needs `print`" },
                    "from": { "type": "integer", "description": "First line to replace, from 1. Needs `print`" },
                    "to": { "type": "integer", "description": "Last line; left out, to the end. Only with `from`" },
                    "print": { "type": "string", "description": "The `print` from `read_doc` or `outline_doc`, for `section` or lines only. If anyone wrote since, nothing changes: ask `outline_doc` again" },
                    "echo": echo_field()
                },
                "required": ["doc", "new"]
            }))
        },
        {
            "name": "edit_table",
            "title": "Change a table in a document",
            "description": "Read or change one table of a document by rows and columns, never its Markdown. A row is named by its first cell, a column by its title, as `show` gives them. Values are written as the column's type reads them (1234.5, 2026-10-28, true); one that does not fit is kept and named. Undone like `edit_doc`.",
            "inputSchema": shaped(json!({
                "properties": {
                    "keep_compaction": kept_compaction_field(),
                    "doc": named_doc_field(),
                    "table": { "anyOf": [{ "type": "integer" }, { "type": "string" }], "description": "Its number from 1, or the heading over it. Left out when there is one" },
                    "do": {
                        "type": "string",
                        "enum": ["show", "set", "add", "remove", "sort", "type", "rename", "add_column", "remove_column"],
                        "description": "Defaults to show: the columns with their types and the rows, changing nothing"
                    },
                    "row": { "anyOf": [{ "type": "integer" }, { "type": "string" }], "description": "For set and remove: its first cell's words, or its number from 1" },
                    "column": { "anyOf": [{ "type": "integer" }, { "type": "string" }], "description": "For set, sort, type, rename and remove_column: its title, or its number from 1" },
                    "value": { "type": "string", "description": "For set: what goes in the cell; empty clears it" },
                    "values": {
                        "type": "object",
                        "additionalProperties": { "type": ["string", "number", "boolean"] },
                        "description": "For add: each column's title with its value; columns left out stay empty"
                    },
                    "title": { "type": "string", "description": "For rename and add_column: the column's title" },
                    "type": { "type": "string", "description": "For type and add_column: as written in braces in its head, like `number`, `money USD sum`, `date`, `check` (`language` «tables»); empty removes it" },
                    "down": { "type": "boolean", "description": "For sort: largest first" }
                },
                "required": ["doc"]
            }))
        },
        {
            "name": "docs",
            "title": "The documents and the folders",
            "description": "The documents, newest first, with each one's folder, whether it is put away or locked, and what it holds (words, sections, key words): enough to pick one unopened. A task's words come with the task. Ask before writing, so nothing is kept twice.",
            "inputSchema": shaped(json!({
                "properties": {
                    "scope": { "type": "string", "enum": ["open", "archive", "either"], "description": "Defaults to either" },
                    "limit": { "type": "integer", "description": "At most 200, 50 by default" },
                    "after": { "type": "integer", "description": "Skip this many, to page on while `total` is higher" },
                    "folders": { "type": "boolean", "description": "Every folder too, with id, path, icon and size; the text already names them" },
                    "deep": { "type": "boolean", "description": "With `folder`: the folders below it too" },
                    "folder": { "type": "string", "description": "Only the documents in this folder, by name, path or id, without pages; the folders below are named under `below`" },
                    "page_of": { "type": "string", "description": "Only the pages of this document, by id, in reading order" }
                }
            }))
        },
        {
            "name": "import_doc",
            "title": "Bring a Markdown file in as a document",
            "description": "Bring one Markdown file from disk in as a document, turned into the Tisty language, saying what changed. Files it points at come in too; links to what cannot are taken out. The file on disk is untouched; a seeming live credential comes in under a warning. A task's exported page goes back to its task. One file per call.",
            "inputSchema": shaped(json!({
                "properties": {
                    "path": { "type": "string", "description": "A Markdown file under Downloads, Documents, Pictures, Desktop or the temporary folder" },
                    "title": { "type": "string", "description": "A title for a file whose first line is not one; defaults to the file name" },
                    "folder": { "type": "string", "description": "The existing folder its subject belongs in, by name, path or id. Left out needs a reason" },
                    "page_of": { "type": "string", "description": "Id of the document it becomes a page of" },
                    "anyway": { "type": "boolean", "description": "Bring it in although a document has the same subject; only when it truly is another" }
                },
                "required": ["path"]
            }))
        },
        {
            "name": "export_doc",
            "title": "Export a document to a folder",
            "description": "Write a document out as Markdown files in a folder on this machine, its pages numbered in reading order and its files beside them. A copy: nothing here changes.",
            "inputSchema": shaped(json!({
                "properties": {
                    "doc": named_doc_field(),
                    "into": { "type": "string", "description": "An existing folder under Downloads, Documents, Pictures, Desktop or the temporary folder, not used for this document before" }
                },
                "required": ["doc", "into"]
            }))
        },
        {
            "name": "archive_doc",
            "title": "Put a document away, or bring it back",
            "description": "Put a document or one page away when the person asks, or bring it back with `archived` false; nothing is deleted or changed («delete it» is `flag_doc`). A page put away stays under its document, read-only; a document's pages go and come back with it.",
            "inputSchema": shaped(json!({
                "properties": {
                    "doc": named_doc_field(),
                    "archived": { "type": "boolean", "description": "True, the default, puts it away; false brings it back" },
                    "folder": { "type": "string", "description": "A folder to file it in on the way, by name, path or id" }
                },
                "required": ["doc"]
            }))
        },
        {
            "name": "flag_doc",
            "title": "Mark a document for the person to delete",
            "description": "How «delete this document» is done: you mark it, and the person deletes it or takes the mark off. Give the `reason`, `instead` where its content lives now, and `body`. Several copies go in one call, so one click settles them. A document already marked is refused, or skipped in a set.",
            "inputSchema": shaped(json!({
                "properties": {
                    "doc": {
                        "anyOf": [
                            { "type": "string" },
                            { "type": "array", "items": { "type": "string" }, "minItems": 1 }
                        ],
                        "description": "The id of the document to mark, not its title, or a list of ids marked alike"
                    },
                    "reason": {
                        "type": "string",
                        "enum": ["duplicate_of", "moved", "superseded", "wrong", "obsolete", "scratch"],
                        "description": "A copy of another, moved into another (only once what it held worth keeping is there), superseded by another, wrong, obsolete, or a scratch nobody needs"
                    },
                    "instead": { "type": "string", "description": "Id of the document or task that holds it now; needed with duplicate_of, moved and superseded" },
                    "body": { "type": "string", "description": "In the person's language: what makes it old, and how you know" }
                },
                "required": ["doc", "body"]
            }))
        },
        {
            "name": "file_doc",
            "title": "Put documents in a folder",
            "description": "Move documents into a folder, or out of every folder by leaving `folder` out. Only the place changes; an archived document moves and stays put away.",
            "inputSchema": shaped(json!({
                "properties": {
                    "doc": many_docs_field("to file"),
                    "folder": { "type": "string", "description": "An existing folder, by name, path or id" }
                },
                "required": ["doc"]
            }))
        },
        {
            "name": "page_doc",
            "title": "Make pages, put them in order, or take one out",
            "description": "Hang a document under another as a page, or with no `page_of` take a page back out to its old folder. A page goes everywhere with its document. Hanging writes the line naming it at the document's end, or where `after`, `before` or `at` say; taking out leaves the line for you to remove. `order` reorders a document's pages. Reading order is line order, each line `![Its title](tisty:doc/its-id)`, bang included.",
            "inputSchema": shaped(json!({
                "properties": {
                    "doc": many_docs_field("to hang or take out"),
                    "page_of": { "type": "string", "description": "Id of the document it becomes a page of. Left out, it becomes a document of its own" },
                    "after": { "type": "string", "description": "Id of another page of that document, whose line this one's goes right after. Needs `page_of` and one `doc`, and that page needs a line" },
                    "before": { "type": "string", "description": "The same, right before" },
                    "order": { "type": "array", "items": { "type": "string" }, "description": "Its pages' ids, two or more, each once, in reading order: their lines trade places, nothing else moves. With `page_of` alone; each needs a line" },
                    "at": { "type": "string", "enum": ["first", "last"], "description": "Before or after every page line, for when there is no page to name" }
                }
            }))
        },
        {
            "name": "tongue_doc",
            "title": "Join or leave a document's languages",
            "description": "With `of`, join `doc` to that document's languages, moving it to its folder; without, take `doc` out of its languages. Both say their language in a `lang:` property, or `lang` for `doc`. A new translation is `write_doc` with `translation_of`.",
            "inputSchema": shaped(json!({
                "properties": {
                    "doc": { "type": "string", "description": "Id of the document to join or take out, not its title" },
                    "of": { "type": "string", "description": "Id of a document whose languages it joins. Left out, it leaves its own" },
                    "lang": { "type": "string", "description": "The language of `doc`, like `es`, `en` or `pt-BR`, when its properties lack one" }
                },
                "required": ["doc"]
            }))
        },
        {
            "name": "folder",
            "title": "Make a folder",
            "description": "Make a folder for documents (tasks go in lists), with an icon and colour if they fit. A name in use hands back that folder, found at any depth unless `inside` says where; an icon or colour sent restyles it.",
            "inputSchema": shaped(json!({
                "properties": {
                    "name": { "type": "string", "description": "Its own name, a word or two, at most 40 characters; a path only to say where to nest it, every step but the last existing" },
                    "inside": { "type": "string", "description": "An existing folder to nest it in, by name, path or id; four deep at most" },
                    "icon": { "type": "string", "description": "One emoji, or a drawn name: home, work, money, study, travel, health, food, family, code and the like" },
                    "color": { "type": "string", "description": "red, orange, amber, green, teal, blue, indigo, purple, pink, brown or gray" }
                },
                "required": ["name"]
            }))
        },
        {
            "name": "read_doc",
            "title": "Read a document, or a part of one",
            "description": "A document's text and the `print` it reads at. Past about 12,000 characters it comes back as its outline (`whole` false): then ask for a `section`, lines, or `chars` at a time. Every part brings the print, so `edit_doc` needs no whole read. `pages_loose` names pages no line places.",
            "inputSchema": shaped(json!({
                "properties": {
                    "doc": named_doc_field(),
                    "print": { "type": "string", "description": "The `print` the previous part came with; only with `cursor`" },
                    "section": { "type": "integer", "description": "A heading and all under it, by its `at` in `outline_doc`, from 0" },
                    "from": { "type": "integer", "description": "First line, from 1" },
                    "to": { "type": "integer", "description": "Last line; left out, to the end" },
                    "chars": { "type": "integer", "description": "About how many characters to bring; the answer gives `next` when there is more" },
                    "cursor": { "type": "integer", "description": "The `next` a previous answer gave, with `chars`" },
                    "as": { "type": "string", "enum": ["markdown"], "description": "The whole document in GitHub Markdown for another tool; tell the person what `left_out` lists (`rules` «documents»). Not with `section`, lines or `chars`" }
                },
                "required": ["doc"]
            }))
        },
        {
            "name": "catch_up",
            "title": "Where things stand, and what has moved",
            "description": "Where things stand: the lists, folders and tags in use, counts, the latest documents, and a `cursor`; sent back as `since`, it adds what moved since, by anyone. Ask it first and on coming back, instead of `lists`, `tags` and `docs`.",
            "inputSchema": shaped(json!({
                "properties": {
                    "since": { "type": "string", "description": "A `cursor` from a previous `catch_up`" }
                }
            }))
        },
        {
            "name": "reschedule",
            "title": "Move the day of a task an agent filed",
            "description": "Change the `date` or `deadline` of a task an agent filed, when what you learnt moves it. A day the person set is theirs. Null takes a day off; nothing else changes.",
            "inputSchema": shaped(json!({
                "properties": {
                    "task": { "type": "string", "description": "The task id" },
                    "date": { "type": ["string", "null"], "description": "The day it is to be done, or with a deadline the day work starts (2026-08-31); null takes it off" },
                    "deadline": { "type": ["string", "null"], "description": "The day it is owed by; null takes it off" }
                },
                "required": ["task"]
            }))
        },
        {
            "name": "say_done",
            "title": "Say a task is finished",
            "description": "Say a task is done when you did the work, in the turn it ends; on any task, as your word. Refused while a step is unticked or a part open and unmarked. What stopped mattering is `say_not_doing`. Nothing closes: the person decides. Say it once; news after goes in `note`.",
            "inputSchema": shaped(json!({
                "properties": {
                    "task": { "type": "string", "description": "The task id" },
                    "body": { "type": "string", "description": "What you did and what makes you sure (the command and its answer, the commit), in the Tisty language" }
                },
                "required": ["task", "body"]
            }))
        },
        {
            "name": "say_not_doing",
            "title": "Say a task should not be done",
            "description": "Say a task should not be done, on any task, as your word: it stopped mattering or became moot. Unticked steps do not stop it. Nothing drops: the person decides. Say it once; news after goes in `note`.",
            "inputSchema": shaped(json!({
                "properties": {
                    "task": { "type": "string", "description": "The task id" },
                    "body": { "type": "string", "description": "Why it should not be done, and what you found that says so, in the Tisty language" }
                },
                "required": ["task", "body"]
            }))
        },
        {
            "name": "sum_up",
            "title": "Say what a document is about",
            "description": "Leave a `summary` and `notes` on a document for the next agent, kept on this machine only and unseen by the person; a later write marks it stale. Write it after reading a long document, never instead, saying what it says.",
            "inputSchema": shaped(json!({
                "properties": {
                    "doc": named_doc_field(),
                    "summary": { "type": "string", "description": "What it says and what a reader comes to it for, in a few lines" },
                    "notes": { "type": "string", "description": "What working with it needs: which section holds what, what is stale or missing" }
                },
                "required": ["doc"]
            }))
        },
        {
            "name": "outline_doc",
            "title": "What is in a document",
            "description": "A document's headings with their `at`, lines and size, its length and `print`, and its pages with title, length and sections: a few hundred tokens. Ask it first for a long document or a change in part. Also `pointed_at` (who links here), `by` (agent or person) and the signs it wants compacting: `alike`, `repeated`, `dead`, `mislabeled`, `loose`.",
            "inputSchema": shaped(json!({
                "properties": {
                    "doc": named_doc_field()
                },
                "required": ["doc"]
            }))
        },
        {
            "name": "read",
            "title": "Read a whole task",
            "description": "Everything a task holds: description, steps, journal (note ids for `reword_note`), parts, files and its `document` page. Read it before noting or mending; `fields` narrows it. A closed task comes with `closed` and a `notice`: it is history, and nothing on it changes.",
            "inputSchema": shaped(json!({
                "properties": {
                    "task": { "type": "string", "description": "The task id" },
                    "fields": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Only these, the id always: title, status, closed, notice, date, deadline, standing, reminders, tags, source, list, priority, by_agent, via, said_done, said_not_doing, open_to_agents, part_of, parts, description, steps, journal, kept, document"
                    }
                },
                "required": ["task"]
            }))
        },
        {
            "name": "find",
            "title": "Search the list and the archive",
            "description": "Search open and archived tasks and documents: words in `query`, the task fields below to sift, `source` to see if something was filed, `doc` for lines inside one document. A task matched in its page, steps or notes brings the `line`. Open a task hit with `read`, a document hit with `read_doc`.",
            "inputSchema": shaped(json!({
                "properties": {
                    "query": { "type": "string", "description": "Words that must all turn up, in any order, accents or not (with `doc`, on one line); quote a phrase. Not needed when sifting" },
                    "source": { "type": "string", "description": "Whether this exact source was filed already; not with `query`" },
                    "doc": { "type": "string", "description": "Search inside this document: lines with every word, their numbers, neighbours cut at 200 characters, and section" },
                    "around": { "type": "integer", "description": "With `doc`: lines each side of a hit, 0 to 10, 1 by default" },
                    "tag": { "type": "string", "description": "Carrying this tag, # or not. Any sifting field leaves documents out" },
                    "list": { "type": "string", "description": "In this list, as `lists` names it" },
                    "folder": { "type": "string", "description": "Only documents in this folder or below, by name, path or id" },
                    "by_agent": { "type": "boolean", "description": "True: filed by an agent; false: by the person" },
                    "said_done": { "type": "boolean", "description": "True: an agent said done or not doing and the person has not decided; false: nobody did" },
                    "open_to_agents": { "type": "boolean", "description": "True: the person's tasks opened to agents; false: kept to themselves" },
                    "part_of": { "type": "string", "description": "Only the parts of this task, by id" },
                    "from_source": { "type": "string", "description": "Tasks whose `source` starts with this, however written: «sereno» brings all read from there" },
                    "from": { "type": "string", "description": "Date or deadline on or after this day (2026-08-31)" },
                    "to": { "type": "string", "description": "Date or deadline on or before this day" },
                    "scope": { "type": "string", "enum": ["open", "archive", "either"], "description": "Defaults to either: open and archived together" },
                    "limit": { "type": "integer", "description": "At most 100, 20 by default" },
                    "after": { "type": "integer", "description": "Skip this many tasks, to page on while `total` is higher. Documents are not paged: `docsTotal` counts them" }
                }
            }))
        },
        json!({
            "name": "lists",
            "title": "The lists that exist",
            "description": "The person's lists, to file a task in. You cannot make one; without one a task lands in the inbox.",
            "inputSchema": { "type": "object", "additionalProperties": false }
        }),
        json!({
            "name": "rules",
            "title": "The rules for working here, by topic",
            "description": "The rules for working here, one guide at a time; alone, the basics and an index. Ask «tasks» before filing or working a task, «documents» before writing one, «tidying» before compacting.",
            "inputSchema": shaped(json!({
                "properties": {
                    "topic": {
                        "type": "string",
                        "enum": ["tasks", "documents", "tidying", "all"],
                        "description": "The guide; left out, the basics and the index"
                    }
                }
            }))
        }),
        json!({
            "name": "language",
            "title": "What Tisty adds to Markdown",
            "description": "Reference for the Markdown extensions Tisty reads, to look up before writing one: pens, underline, icons, callouts, folds, tags, cards, attachments, diagrams. Plain Markdown needs none. Without `topic`, the index; then ask only the topic you will write. How the app works is `help`.",
            "inputSchema": shaped(json!({
                "properties": {
                    "topic": {
                        "type": "string",
                        "enum": super::guide::topics(),
                        "description": "One part, named as the index names it, or all"
                    },
                    "lang": { "type": "string", "enum": ["en", "es"], "description": "en, the default, or es" }
                }
            }))
        }),
        json!({
            "name": "help",
            "title": "How Tisty itself works",
            "description": "Tisty's own Help pages, for when the person asks how Tisty does something: installing, tasks, documents, sync, encryption, assistants, shortcuts, privacy. Alone, the index; `topic` gives a page.",
            "inputSchema": shaped(json!({
                "properties": {
                    "topic": {
                        "type": "string",
                        "description": "One page, named as the index names it in either language"
                    },
                    "lang": { "type": "string", "enum": ["en", "es"], "description": "en, the default, or es" }
                }
            }))
        }),
        json!({
            "name": "tags",
            "title": "The tags already in use",
            "description": "The tags in use, most used first, with counts; case and accents do not matter. Reuse one for the same subject rather than coining a second.",
            "inputSchema": { "type": "object", "additionalProperties": false }
        })
    ])
}

/// Seven tools take this field, and seven wordings of it would drift apart.
fn named_doc_field() -> Value {
    json!({ "type": "string", "description": "The document's id from `docs`, not its title" })
}

fn kept_compaction_field() -> Value {
    json!({
        "type": "boolean",
        "description": "The person told you in this conversation to keep the compaction awaiting their word"
    })
}

fn echo_field() -> Value {
    json!({ "type": "boolean", "description": "Hand back the lines around the change, instead of reading again" })
}

fn on_a_task(fields: Value, also_required: &[&str]) -> Value {
    let mut properties = serde_json::Map::new();
    properties.insert(
        "task".into(),
        json!({ "type": "string", "description": "The task id" }),
    );
    if let Value::Object(more) = fields {
        properties.extend(more);
    }
    let mut required = vec!["task"];
    required.extend_from_slice(also_required);
    shaped(json!({ "properties": properties, "required": required }))
}

fn steps_by_text(doing: &str) -> Value {
    json!({
        "steps": {
            "type": "array",
            "items": { "type": "string" },
            "description": format!("The steps {doing}, by their text as `read` shows it; case and accents do not matter")
        },
        "step": { "type": "string", "description": "One step, by its text" }
    })
}

fn many_docs_field(what: &str) -> Value {
    json!({
        "anyOf": [
            { "type": "string" },
            { "type": "array", "items": { "type": "string" }, "minItems": 1 }
        ],
        "description": format!(
            "The id of the document {what}, not its title, or a list of ids: if one cannot, none moves"
        )
    })
}
