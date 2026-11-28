use serde_json::{Value, json};

const READ_FIELDS: [&str; 27] = [
    "id",
    "title",
    "status",
    "closed",
    "notice",
    "date",
    "deadline",
    "standing",
    "reminders",
    "tags",
    "source",
    "list",
    "priority",
    "by_agent",
    "for",
    "via",
    "said_done",
    "said_not_doing",
    "part_of",
    "parts",
    "description",
    "steps",
    "state",
    "document",
    "journal",
    "kept",
    "print",
];

const FOLDER_COLORS: [&str; 11] = [
    "red", "orange", "amber", "green", "teal", "blue", "indigo", "purple", "pink", "brown", "gray",
];

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
pub(super) fn tools_without_hints() -> Value {
    let mut all = workspace_tools();
    all.extend(task_tools());
    all.extend(doc_tools());
    all.extend(file_tools());
    Value::Array(all)
}

fn workspace_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "workspace_catch_up",
            "title": "Resume workspace state and changes",
            "description": "Call first at session start. Returns lists, folders, top tags, counts, pressing tasks, newest docs, your last open task, and changes since your last catch-up. Pass cursor `since` for read-only delta replay without advancing the cursor.",
            "inputSchema": shaped(json!({
                "properties": {
                    "since": { "type": "string", "description": "Cursor from a prior workspace_catch_up for read-only delta replay (includes your own)" }
                }
            }))
        }),
        json!({
            "name": "workspace_search",
            "title": "Search tasks, documents, and taxonomy",
            "description": "Search or list open and archived items without reading full bodies. Any task filter excludes docs. `mode` is inferred if omitted (empty `{}` returns full taxonomy).",
            "inputSchema": shaped(json!({
                "properties": {
                    "mode": { "type": "string", "enum": ["search", "source", "project", "folder", "taxonomy", "in_doc"], "description": "Query mode; inferred from params if omitted" },
                    "query": { "type": "string", "description": "Search terms (accents ignored; quote phrases); optional when filtering" },
                    "doc": { "type": "string", "description": "Document ID for mode=\"in_doc\" line search with `query`" },
                    "around": { "type": "integer", "description": "Context lines each side of an in_doc match (0-10, default 1)" },
                    "source": { "type": "string", "description": "Exact source URL/ID deduplication check (not with `query`)" },
                    "part_of": { "type": "string", "description": "Parent task ID; alone gives grouped project stage summary" },
                    "list": { "type": "string", "description": "Filter tasks by exact list name" },
                    "folder": { "type": "string", "description": "Filter/list docs by folder name, path, or ID" },
                    "tag": { "type": "string", "description": "Filter tasks by tag (# optional)" },
                    "by_agent": { "type": "boolean", "description": "Filter tasks: true = agent-created, false = person-created" },
                    "said_done": { "type": "boolean", "description": "Filter tasks pending human confirmation (true/false)" },
                    "from_source": { "type": "string", "description": "Filter tasks whose `source` starts with this prefix" },
                    "date_from": { "type": "string", "description": "Tasks with date/deadline >= YYYY-MM-DD" },
                    "date_to": { "type": "string", "description": "Tasks with date/deadline <= YYYY-MM-DD" },
                    "scope": { "type": "string", "enum": ["open", "archive", "either"], "description": "Search scope (default \"either\")" },
                    "lines": { "type": "boolean", "description": "Include matching excerpt line (default true; false returns titles only)" },
                    "limit": { "type": "integer", "description": "Max results (default 10/max 100; project 50/100; folder 50/200)" },
                    "after": { "type": "integer", "description": "Pagination offset when `total` exceeds returned count" }
                }
            }))
        }),
        json!({
            "name": "workspace_read",
            "title": "Read a task, document, card, or table",
            "description": "Read one `task` or `doc` (not both). Docs >12,000 chars (task pages >4,000) return an outline first; pass `section`, `from`/`to`, or `chars` for slices. Use `view` for cards/outlines, link graphs, highlights, tables, version history, or Markdown export.",
            "inputSchema": shaped(json!({
                "properties": {
                    "task": { "type": "string", "description": "Task ID" },
                    "doc": { "type": "string", "description": "Document ID or task-page filename (from workspace_search)" },
                    "view": { "type": "string", "enum": ["content", "card", "links", "marks", "table", "versions", "markdown"], "description": "What to return (default \"content\"); \"card\" gives outline+print+links" },
                    "fields": { "type": "array", "items": { "type": "string", "enum": READ_FIELDS }, "description": "For `task`: subset of keys to return (`id` always included)" },
                    "notes": { "type": "integer", "description": "For `task`: latest journal notes count (clipped 4000 chars); 0 returns all whole" },
                    "section": { "type": "integer", "description": "0-based heading index (`at` from card) for `content` slice or sub-outline" },
                    "from": { "type": "integer", "description": "1-based start line for doc `content`" },
                    "to": { "type": "integer", "description": "1-based end line (defaults to EOF; requires `from`)" },
                    "chars": { "type": "integer", "description": "Char budget per chunk; returns `next` cursor" },
                    "cursor": { "type": "integer", "description": "Next line cursor from prior chunked read (requires `print`)" },
                    "print": { "type": "string", "description": "Version fingerprint; returns `unchanged: true` if unmodified" },
                    "version": { "type": "string", "description": "Past version ID from `view=\"versions\"` to read in place of current body" },
                    "links": { "type": "string", "enum": ["cite", "mention", "structure", "tag", "uses", "all"], "description": "Edge filter for `view=\"links\"` (default \"all\")" },
                    "table": table_ref_field(),
                    "limit": { "type": "integer", "description": "For `view=\"links\"`: max neighbours (default 50, max 100)" }
                }
            }))
        }),
        json!({
            "name": "workspace_guide",
            "title": "Read Tisty rules, syntax, and help",
            "description": "Load Tisty reference guides progressively. Omit `topic` to fetch the index first, then request only the specific topic needed.",
            "inputSchema": shaped(json!({
                "properties": {
                    "kind": { "type": "string", "enum": ["rules", "markdown", "help"], "description": "Guide family: workflow rules, Markdown extensions, or app help" },
                    "topic": { "type": "string", "description": "Topic slug or \"all\" (omit for index). Rules: tasks|documents|tidying|ideas|all" },
                    "lang": { "type": "string", "enum": ["en", "es"], "description": "Language for `markdown` and `help` (default \"en\")" }
                },
                "required": ["kind"]
            }))
        }),
    ]
}

fn task_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "task_create",
            "title": "Create one or more tasks",
            "description": "Create actionable task(s); lasting reference belongs in `doc_create`. Every task requires single owner `for` (\"agent\" or \"person\"). Pass `source` to deduplicate; `parts` creates subtasks (1 level, <=32). Batch up to 32 via `tasks` alone. Tagged #agent.",
            "inputSchema": shaped(json!({
                "properties": {
                    "title": { "type": "string", "description": "Task title in one line (<=500 chars)" },
                    "for": { "type": "string", "enum": ["agent", "person"], "description": "Required single owner: \"agent\" (you do/verify) or \"person\" (only they can)" },
                    "tasks": { "type": "array", "items": task_item(), "description": "Batch up to 32 tasks (each shaped like a single task); omit top-level fields" },
                    "parts": { "type": "array", "items": part_item(), "description": "Up to 32 subtasks (`for` inherits parent if omitted; 1 level only)" },
                    "part_of": { "type": "string", "description": "Open parent task ID to attach this task as a subtask" },
                    "description": { "type": "string", "description": "Task details in Tisty Markdown (<=64,000 chars)" },
                    "date": { "type": "string", "description": "Planned day (YYYY-MM-DD), or start day when `deadline` is set" },
                    "deadline": { "type": "string", "description": "Hard due date (YYYY-MM-DD); only past deadlines become overdue" },
                    "remind": { "type": "array", "items": { "type": "string" }, "description": "One-off ring times (YYYY-MM-DDTHH:MM, <=32); evening before for fixed-hour events" },
                    "priority": { "type": "string", "enum": ["do", "decide", "delegate", "minor"], "description": "Eisenhower priority (only if explicitly requested)" },
                    "list": { "type": "string", "description": "Existing list name (from workspace_catch_up); omit for inbox only with reason" },
                    "tags": { "type": "array", "items": { "type": "string" }, "description": "Plain subject words (<=32; reuse existing tags)" },
                    "steps": { "type": "array", "items": { "type": "string" }, "description": "Checklist items (<=200 items, <=100 chars each)" },
                    "source": { "type": "string", "description": "Origin ID/URL for deduplication; idea name removes it from Ideas" },
                    "again": { "type": "boolean", "description": "Allow re-filing from a closed/dropped `source` only when user asks to redo" }
                }
            }))
        }),
        json!({
            "name": "task_update",
            "title": "Update open task metadata",
            "description": "Patch metadata on an open task (title, description, dates, reminders, list, or parent). `description` replaces whole (read first). `date`/`deadline` apply only to agent-owned tasks; other fields work on any open task not marked done. Null clears `date`/`deadline`/`part_of`.",
            "inputSchema": on_a_task(
                json!({
                    "title": { "type": "string", "description": "New task title (<=500 chars)" },
                    "description": { "type": "string", "description": "Full replacement description in Tisty Markdown (<=64,000 chars)" },
                    "date": { "type": ["string", "null"], "description": "Planned/start day (YYYY-MM-DD) or null to clear (agent tasks only)" },
                    "deadline": { "type": ["string", "null"], "description": "Due date (YYYY-MM-DD) or null to clear (agent tasks only)" },
                    "add_reminders": { "type": "array", "items": { "type": "string" }, "description": "Additional ring times (YYYY-MM-DDTHH:MM, <=32); additive only" },
                    "list": { "type": "string", "description": "Target existing list name" },
                    "part_of": { "type": ["string", "null"], "description": "Parent task ID (1 level) or null to detach as standalone" }
                }),
                &[],
            )
        }),
        json!({
            "name": "task_edit_steps",
            "title": "Manage task steps and state",
            "description": "Add, tick, untick, reword, or remove/hand off steps on an open task, or update its page `state` digest. On `person` tasks, `tick` records your word for user confirmation. On `remove`, `to_person: true` hands human-only steps to the companion subtask.",
            "inputSchema": on_a_task(
                json!({
                    "action": { "type": "string", "enum": ["add", "tick", "untick", "reword", "remove", "set_state"], "description": "Step/state operation" },
                    "steps": { "type": "array", "items": { "type": "string" }, "description": "Exact step texts (<=100 chars each for `add`; all-or-nothing match for others)" },
                    "step": { "type": "string", "description": "Single step exact text (required for `reword`; fallback for single-step ops)" },
                    "text": { "type": "string", "description": "Replacement step text for `reword` (<=100 chars)" },
                    "to": { "type": "string", "description": "Destination open task ID to transfer removed steps onto (`remove`)" },
                    "to_person": { "type": "boolean", "description": "Hand removed steps to task's person-owned companion subtask (`remove`)" },
                    "title": { "type": "string", "description": "Companion task title if created by `to_person`" },
                    "state": { "type": "string", "description": "Current-state summary overwritten on task page (<=3,000 chars; `tick`/`set_state`)" }
                }),
                &["action"],
            )
        }),
        json!({
            "name": "task_note",
            "title": "Add or reword a task journal note",
            "description": "Add a dated entry to an open task's journal (decisions, blockers, ruled-out paths; <=4,000 chars) or reword an existing note by ID. Long drafts/tables belong on the task page via `doc_edit`. For doc callouts use `doc_edit` with `[!NOTE]`.",
            "inputSchema": on_a_task(
                json!({
                    "action": { "type": "string", "enum": ["add", "reword"], "description": "Append new journal note or replace existing note body" },
                    "body": { "type": "string", "description": "Note text in Tisty Markdown (<=4,000 chars)" },
                    "replaces": { "type": "string", "description": "For `add`: ID of prior agent note this entry supersedes (kept marked)" },
                    "note": { "type": "string", "description": "For `reword`: existing note ID from `workspace_read`" }
                }),
                &["action", "body"],
            )
        }),
        json!({
            "name": "task_resolve",
            "title": "Propose task completion or cancellation",
            "description": "Mark an open task done or not-doing for human confirmation (never closes or deletes). `done` requires all steps ticked/vouched and all agent subtasks resolved. Include verification proof in `body`; add `branch`/`pr` when shipping code. Call once.",
            "inputSchema": on_a_task(
                json!({
                    "outcome": { "type": "string", "enum": ["done", "not_doing"], "description": "Proposed resolution: completed or moot/won't-do" },
                    "body": { "type": "string", "description": "Verification proof (commands, commits, results) or reason not doing" },
                    "branch": { "type": "string", "description": "For `done`: git branch name (<=200 chars)" },
                    "pr": { "type": "string", "description": "For `done`: pull request number or URL (<=200 chars)" }
                }),
                &["outcome", "body"],
            )
        }),
    ]
}

fn doc_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "doc_create",
            "title": "Create a new document",
            "description": "Create a new document for lasting reference (tasks use `task_create`). Search first via `workspace_search`: update existing subjects in place. First line sets title; single newlines break lines inside paragraphs. Inline `#tags` tag the doc.",
            "inputSchema": shaped(json!({
                "properties": {
                    "body": { "type": "string", "description": "Full Markdown document; first line is title, one line per paragraph (<=64,000 chars)" },
                    "folder": { "type": "string", "description": "Existing folder name, path, or ID (omit only for Ideas sheet or by request)" },
                    "page_of": { "type": "string", "description": "Parent doc ID to attach as a subpage (1 level deep; inherits folder)" },
                    "translation_of": { "type": "string", "description": "Source doc ID when creating a translated version" },
                    "lang": { "type": "string", "description": "BCP-47 tag (e.g. `es`, `en`, `pt-BR`) for `translation_of`" },
                    "anyway": { "type": "boolean", "description": "Allow creating even if an existing doc shares the subject" }
                },
                "required": ["body"]
            }))
        }),
        json!({
            "name": "doc_edit",
            "title": "Edit, append, compact, or restore a doc",
            "description": "Modify or undo changes to an existing doc or task page. Prefer `patch` (`old` or `section`/`from`+`to`+`print`) over `rewrite`. `append` adds chronologically, under `under`, or to assistant notes (task ID) / Diary. `digest_task_notes` compacts assistant notes (<=6,000 chars).",
            "inputSchema": shaped(json!({
                "properties": {
                    "doc": { "type": "string", "description": "Doc ID, task-page filename, or task ID (for `append`, `digest_task_notes`, `restore`)" },
                    "mode": { "type": "string", "enum": ["patch", "append", "rewrite", "compact", "digest_task_notes", "restore"], "description": "Edit operation" },
                    "content": { "type": "string", "description": "New text for `patch` (\"\" deletes), `append`, `rewrite`, `compact`, `digest_task_notes`" },
                    "old": { "type": "string", "description": "For `patch`: unique verbatim passage to replace (not with `section`/`from`)" },
                    "section": { "type": "integer", "description": "For `patch`: 0-based heading index from card (requires `print`)" },
                    "from": { "type": "integer", "description": "For `patch`: 1-based start line (requires `print`)" },
                    "to": { "type": "integer", "description": "For `patch`: 1-based end line (defaults to EOF; with `from` only)" },
                    "under": { "type": "string", "description": "For `append`: exact heading text to append under" },
                    "print": { "type": "string", "description": "Card/read fingerprint; required for `rewrite`, `compact`, positional `patch`" },
                    "version": { "type": "string", "description": "For `restore`: target version ID from `workspace_read(view=\"versions\")`" },
                    "even_if_more": { "type": "boolean", "description": "For `restore`: revert past newer edits after inspecting them" },
                    "echo": { "type": "boolean", "description": "Return lines around the change (`patch`/`append`)" },
                    "person_asked": { "type": "boolean", "description": "For `compact`: user explicitly asked to compact a doc they began" },
                    "keep_compaction": kept_compaction_field()
                },
                "required": ["doc", "mode"]
            }))
        }),
        json!({
            "name": "doc_edit_table",
            "title": "Edit a document table by row/column",
            "description": "Mutate one table in a doc or task page structurally (never raw Markdown). Inspect columns and types first via `workspace_read(view=\"table\")`. Rows match by 1-based index or first cell; columns by 1-based index or title.",
            "inputSchema": shaped(json!({
                "properties": {
                    "doc": { "type": "string", "description": "Document ID or task-page filename" },
                    "action": { "type": "string", "enum": ["set", "add_row", "remove_row", "sort", "set_type", "rename_column", "add_column", "remove_column"], "description": "Table write operation" },
                    "table": table_ref_field(),
                    "row": { "anyOf": [{ "type": "integer" }, { "type": "string" }], "description": "For `set`/`remove_row`: 1-based row number or first-cell text" },
                    "column": { "anyOf": [{ "type": "integer" }, { "type": "string" }], "description": "For `set`/`sort`/`set_type`/`rename_column`/`remove_column`: 1-based index or header" },
                    "value": { "type": "string", "description": "For `set`: cell value (\"\" clears)" },
                    "values": { "type": "object", "additionalProperties": { "type": ["string", "number", "boolean"] }, "description": "For `add_row`: map of column title to cell value" },
                    "title": { "type": "string", "description": "For `rename_column`/`add_column`: column title" },
                    "type": { "type": "string", "description": "For `set_type`/`add_column`: type syntax e.g. `number`, `money USD sum`, `date`, `check`" },
                    "down": { "type": "boolean", "description": "For `sort`: true sorts descending" },
                    "keep_compaction": kept_compaction_field()
                },
                "required": ["doc", "action"]
            }))
        }),
        json!({
            "name": "doc_organize",
            "title": "Organize folders, pages, and translations",
            "description": "Create/restyle folders (max 6 deep, <=64 total), move docs between folders, attach/detach/reorder subpages (1 level deep), or link/unlink document translations.",
            "inputSchema": shaped(json!({
                "properties": {
                    "action": { "type": "string", "enum": ["create_folder", "file", "add_page", "remove_page", "reorder_pages", "link_translation", "unlink_translation"], "description": "Library organization operation" },
                    "docs": { "type": "array", "items": { "type": "string" }, "minItems": 1, "description": "Target document ID(s) (1 for link/unlink_translation or positioned add_page)" },
                    "folder": { "type": "string", "description": "For `file`: destination folder (omit to unfile); for `create_folder`: parent folder" },
                    "parent": { "type": "string", "description": "Parent doc ID (`add_page`, `reorder_pages`) or primary doc (`link_translation`)" },
                    "position": { "type": "string", "enum": ["first", "last"], "description": "For `add_page`: place before/after existing page cards" },
                    "after": { "type": "string", "description": "For `add_page`: place immediately after this sibling page ID" },
                    "before": { "type": "string", "description": "For `add_page`: place immediately before this sibling page ID" },
                    "order": { "type": "array", "items": { "type": "string" }, "minItems": 2, "description": "For `reorder_pages`: >=2 page IDs of `parent` in new reading order" },
                    "lang": { "type": "string", "description": "For `link_translation`: BCP-47 tag of `docs[0]` if missing from frontmatter" },
                    "name": { "type": "string", "description": "For `create_folder`: folder name (<=40 chars)" },
                    "icon": { "type": "string", "description": "For `create_folder`: single emoji or icon name" },
                    "color": { "type": "string", "enum": FOLDER_COLORS, "description": "For `create_folder`: folder color" }
                },
                "required": ["action"]
            }))
        }),
        json!({
            "name": "doc_annotate",
            "title": "Archive, flag, or summarize documents",
            "description": "Manage non-content document status: `archive`/`unarchive` when user requests, `flag` obsolete/duplicate docs or ideas (`sheet#heading`) for user deletion (never delete directly), or `summarize` with an agent-only gist after reading.",
            "inputSchema": shaped(json!({
                "properties": {
                    "action": { "type": "string", "enum": ["archive", "unarchive", "flag", "summarize"], "description": "Status/metadata operation" },
                    "docs": { "type": "array", "items": { "type": "string" }, "minItems": 1, "description": "Document ID(s), or `sheet#heading` idea IDs for `flag`" },
                    "folder": { "type": "string", "description": "For `archive`/`unarchive`: optionally file into this folder" },
                    "reason": { "type": "string", "enum": ["duplicate_of", "moved", "superseded", "wrong", "obsolete", "scratch"], "description": "For `flag`: why it is flagged" },
                    "instead": { "type": "string", "description": "Replacement doc/task ID (required for `duplicate_of`, `moved`, `superseded`)" },
                    "body": { "type": "string", "description": "For `flag`: explanation in the person's language" },
                    "summary": { "type": "string", "description": "For `summarize`: concise doc purpose (<=2,000 chars)" },
                    "notes": { "type": "string", "description": "For `summarize`: agent navigation/staleness notes (<=4,000 chars)" }
                },
                "required": ["action", "docs"]
            }))
        }),
    ]
}

fn file_tools() -> Vec<Value> {
    vec![json!({
        "name": "file_transfer",
        "title": "Attach, import, or export local files",
        "description": "Transfer files between permitted local dirs (Downloads, Documents, Pictures, Desktop, temp) and Tisty. `attach` copies into task journal or doc end (<=650 MB); `import_doc` imports `.md` (or restores exported task page); `export_doc` copies doc + attachments to folder.",
        "inputSchema": shaped(json!({
            "properties": {
                "action": { "type": "string", "enum": ["attach", "import_doc", "export_doc"], "description": "Transfer direction/type" },
                "path": { "type": "string", "description": "Source file path (`attach`/`import_doc`) or destination directory (`export_doc`)" },
                "task": { "type": "string", "description": "For `attach`: target open task ID (mutually exclusive with `doc`)" },
                "doc": { "type": "string", "description": "Target doc ID (for `attach` or `export_doc`)" },
                "label": { "type": "string", "description": "For `attach`: display label (<=200 chars; defaults to filename)" },
                "title": { "type": "string", "description": "For `import_doc`: override title if first line is not a heading (<=500 chars)" },
                "folder": { "type": "string", "description": "For `import_doc`: destination folder name, path, or ID" },
                "page_of": { "type": "string", "description": "For `import_doc`: parent doc ID to attach as subpage" },
                "anyway": { "type": "boolean", "description": "For `import_doc`: allow import despite subject overlap" }
            },
            "required": ["action", "path"]
        }))
    })]
}

fn part_props() -> serde_json::Map<String, Value> {
    json!({
        "title": { "type": "string", "description": "Title (<=500 chars)" },
        "for": { "type": "string", "enum": ["agent", "person"], "description": "Owner" },
        "description": { "type": "string", "description": "Markdown details" },
        "date": { "type": "string", "description": "Day ISO" },
        "deadline": { "type": "string", "description": "Due ISO" },
        "remind": { "type": "array", "items": { "type": "string" }, "description": "Ring ISO times" },
        "priority": { "type": "string", "enum": ["do", "decide", "delegate", "minor"], "description": "Priority" },
        "list": { "type": "string", "description": "List name" },
        "tags": { "type": "array", "items": { "type": "string" }, "description": "Tags" },
        "steps": { "type": "array", "items": { "type": "string" }, "description": "Steps (<=100 chars)" },
        "source": { "type": "string", "description": "Origin ID/URL" }
    })
    .as_object()
    .cloned()
    .unwrap_or_default()
}

fn part_item() -> Value {
    shaped(json!({
        "properties": Value::Object(part_props()),
        "required": ["title"]
    }))
}

fn task_item() -> Value {
    let mut props = part_props();
    props.insert(
        "parts".into(),
        json!({ "type": "array", "items": part_item(), "description": "Up to 32 subtasks" }),
    );
    props.insert(
        "part_of".into(),
        json!({ "type": "string", "description": "Parent task ID" }),
    );
    props.insert(
        "again".into(),
        json!({ "type": "boolean", "description": "Redo closed source" }),
    );
    shaped(json!({
        "properties": Value::Object(props),
        "required": ["title", "for"]
    }))
}

fn table_ref_field() -> Value {
    json!({
        "anyOf": [{ "type": "integer" }, { "type": "string" }],
        "description": "1-based table index or heading above it (omit if single table)"
    })
}

fn kept_compaction_field() -> Value {
    json!({
        "type": "boolean",
        "description": "Confirm keeping prior compaction on user request"
    })
}

fn on_a_task(fields: Value, also_required: &[&str]) -> Value {
    let mut properties = serde_json::Map::new();
    properties.insert(
        "task".into(),
        json!({ "type": "string", "description": "Open task ID" }),
    );
    if let Value::Object(more) = fields {
        properties.extend(more);
    }
    let mut required = vec!["task"];
    required.extend_from_slice(also_required);
    shaped(json!({ "properties": properties, "required": required }))
}
