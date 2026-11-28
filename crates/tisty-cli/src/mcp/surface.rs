use serde_json::{Map, Value, json};
use tisty_core::Paths;

use super::asked::{Takes, body, day, in_order, moments, strings, text};
use super::attaching::attach;
use super::brain;
use super::catalogue;
use super::chores::{
    already_said_done, describe, note, plan, propose, reaching, remind, reschedule, untick,
};
use super::gists::sum_up;
use super::guide::language;
use super::helping::help;
use super::jsonrpc::told;
use super::looking::{catch_up, read};
use super::mending::{rename, reword_note, reword_step, unplan};
use super::moving::{hung_under, list_named, move_task};
use super::papers::editing::edit_doc;
use super::papers::filing::{archive_doc, check_archive, file_doc, folder};
use super::papers::flagging::flag_doc;
use super::papers::importing::import_doc;
use super::papers::paging::page_doc;
use super::papers::reading::export_doc;
use super::papers::tabling::{edit_table, shown as show_table};
use super::papers::tongues::tongue_doc;
use super::papers::writing::{append_doc, restore_doc, write_doc};
use super::saying::{say_done, say_not_doing};
use super::spoken::spoken;
use super::task_state;
use super::{Refused, opened};

pub(super) fn dispatch(
    paths: &Paths,
    name: &str,
    args: Value,
    taken: Option<&Takes>,
) -> Result<Value, Refused> {
    match name {
        "workspace_catch_up" => catch_up(paths, &args),
        "workspace_search" => workspace_search(paths, &args),
        "workspace_read" => workspace_read(paths, &args),
        "workspace_guide" => workspace_guide(&args),
        "task_create" => spoken(paths, name, &args, "description", |args| {
            propose(paths, taken, args)
        }),
        "task_update" => task_update(paths, &args),
        "task_edit_steps" => task_edit_steps(paths, &args),
        "task_note" => task_note(paths, &args),
        "task_resolve" => task_resolve(paths, &args),
        "doc_create" => spoken(paths, name, &args, "body", |args| write_doc(paths, args)),
        "doc_edit" => doc_edit(paths, &args),
        "doc_edit_table" => doc_edit_table(paths, &args),
        "doc_organize" => doc_organize(paths, &args),
        "doc_annotate" => doc_annotate(paths, &args),
        "file_transfer" => file_transfer(paths, &args),
        _ => Err(Refused::Protocol(-32602, format!("unknown tool: {name}"))),
    }
}

fn require_enum<'a>(args: &'a Value, key: &str, valid: &[&str]) -> Result<&'a str, Refused> {
    match args.get(key).and_then(Value::as_str).map(str::trim) {
        Some(said) if valid.contains(&said) => Ok(said),
        Some(other) => Err(Refused::Tool(format!(
            "`{key}` must be one of {}, not {other:?}.",
            valid.join(", ")
        ))),
        None => Err(Refused::Tool(format!(
            "`{key}` is required: one of {}.",
            valid.join(", ")
        ))),
    }
}

fn map_without(args: &Value, skip: &[&str]) -> Map<String, Value> {
    args.as_object()
        .map(|all| {
            all.iter()
                .filter(|(k, _)| !skip.contains(&k.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn workspace_search(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let mode = match text(args, "mode") {
        Some(said) => Some(match said.as_str() {
            "search" | "source" | "project" | "folder" | "taxonomy" | "in_doc" => said,
            other => {
                return Err(Refused::Tool(format!(
                    "`mode` must be search, source, project, folder, taxonomy, or in_doc — not {other:?}."
                )));
            }
        }),
        None => None,
    };
    let mut mapped = map_without(args, &["mode", "date_from", "date_to"]);
    if let Some(from) = args.get("date_from") {
        mapped.insert("from".into(), from.clone());
    }
    if let Some(to) = args.get("date_to") {
        mapped.insert("to".into(), to.clone());
    }
    let mapped = Value::Object(mapped);
    let doc = text(&mapped, "doc");
    let query = text(&mapped, "query");
    let source = text(&mapped, "source");

    match mode.as_deref() {
        Some("taxonomy") => brain::zones::all(paths),
        Some("folder") => {
            if let Some(stray) = [
                "query",
                "doc",
                "source",
                "part_of",
                "list",
                "tag",
                "by_agent",
                "said_done",
                "from_source",
                "date_from",
                "date_to",
            ]
            .into_iter()
            .find(|k| args.get(*k).is_some())
            {
                return Err(Refused::Tool(format!(
                    "`mode: \"folder\"` lists documents in a folder and does not take `{stray}`; \
                     use `mode: \"search\"` to search or filter."
                )));
            }
            brain::zones::documents(paths, &mapped)
        }
        Some("project") => {
            if text(&mapped, "part_of").is_none() {
                return Err(Refused::Tool(
                    "`mode: \"project\"` requires `part_of`, the parent task ID.".into(),
                ));
            }
            brain::project::view(paths, &mapped)
        }
        Some("source") => {
            if source.is_none() {
                return Err(Refused::Tool(
                    "`mode: \"source\"` requires `source`.".into(),
                ));
            }
            brain::search::found(paths, &mapped)
        }
        Some("in_doc") => {
            if doc.is_none() || query.is_none() {
                return Err(Refused::Tool(
                    "`mode: \"in_doc\"` requires both `doc` and `query`.".into(),
                ));
            }
            brain::search::found(paths, &mapped)
        }
        Some("search") => {
            if doc.is_some() || source.is_some() {
                return Err(Refused::Tool(
                    "Use `mode: \"in_doc\"` with `doc` or `mode: \"source\"` with `source`.".into(),
                ));
            }
            brain::search::found(paths, &mapped)
        }
        None => {
            if doc.is_some() && query.is_none() {
                return Err(Refused::Tool(
                    "To inspect a document card or outline, call `workspace_read` with `view: \"card\"`; `workspace_search` with `doc` requires `query`.".into(),
                ));
            }
            brain::brain(paths, &mapped)
        }
        Some(_) => unreachable!(),
    }
}

fn workspace_read(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let view = match text(args, "view") {
        Some(said) => match said.as_str() {
            "content" | "card" | "links" | "marks" | "table" | "versions" | "markdown" => said,
            other => {
                return Err(Refused::Tool(format!(
                    "`view` must be content, card, links, marks, table, versions, or markdown — not {other:?}."
                )));
            }
        },
        None => "content".to_string(),
    };
    let task = text(args, "task");
    let doc = text(args, "doc");
    if task.is_some() && doc.is_some() {
        return Err(Refused::Tool(
            "`workspace_read` reads one `task` or one `doc`, not both.".into(),
        ));
    }
    if task.is_none() && doc.is_none() {
        return Err(Refused::Tool(
            "`workspace_read` requires a `task` ID or a `doc` ID; `workspace_search` finds either."
                .into(),
        ));
    }
    let clean = Value::Object(map_without(args, &["view"]));
    match view.as_str() {
        "card" => {
            if args.get("links").is_some() {
                return Err(Refused::Tool(
                    "`links` goes with `view: \"links\"`, not `view: \"card\"`.".into(),
                ));
            }
            if args.get("section").is_some() && doc.is_none() {
                return Err(Refused::Tool(
                    "`section` is asked of one thing: send a `doc` with it.".into(),
                ));
            }
            match (doc, task) {
                (Some(which), None) => brain::cards::of_doc(paths, &which, &clean),
                (None, Some(which)) => brain::cards::of_task(paths, &which, &clean),
                _ => unreachable!(),
            }
        }
        "links" => {
            let mut mapped = map_without(args, &["view"]);
            mapped.entry("links").or_insert_with(|| json!("all"));
            let mapped = Value::Object(mapped);
            match (doc, task) {
                (Some(which), None) => brain::cards::of_doc(paths, &which, &mapped),
                (None, Some(which)) => brain::cards::of_task(paths, &which, &mapped),
                _ => unreachable!(),
            }
        }
        "table" => {
            let Some(which) = doc else {
                return Err(Refused::Tool(
                    "`view: \"table\"` requires `doc` (document ID or task-page filename).".into(),
                ));
            };
            show_table(paths, &clean, &which)
        }
        "marks" => {
            if doc.is_none() {
                return Err(Refused::Tool(
                    "`view: \"marks\"` requires `doc` (document ID or task-page filename).".into(),
                ));
            }
            let mut mapped = map_without(args, &["view"]);
            mapped.insert("marks".into(), json!(true));
            read(paths, &Value::Object(mapped))
        }
        "versions" => {
            let mut mapped = map_without(args, &["view"]);
            if let Some(id) = task {
                mapped.remove("task");
                mapped.insert("doc".into(), json!(id));
            }
            mapped.insert("versions".into(), json!(true));
            read(paths, &Value::Object(mapped))
        }
        "markdown" => {
            if doc.is_none() {
                return Err(Refused::Tool(
                    "`view: \"markdown\"` requires `doc` (document ID or task-page filename)."
                        .into(),
                ));
            }
            let mut mapped = map_without(args, &["view"]);
            mapped.insert("as".into(), json!("markdown"));
            read(paths, &Value::Object(mapped))
        }
        "content" => {
            let mut mapped = map_without(args, &["view"]);
            if let Some(id) = task
                && text(args, "version").is_some()
            {
                mapped.remove("task");
                mapped.insert("doc".into(), json!(id));
            }
            read(paths, &Value::Object(mapped))
        }
        _ => unreachable!(),
    }
}

fn workspace_guide(args: &Value) -> Result<Value, Refused> {
    let kind = require_enum(args, "kind", &["rules", "markdown", "help"])?;
    let clean = Value::Object(map_without(args, &["kind"]));
    match kind {
        "rules" => Ok(catalogue::rules(&clean)),
        "markdown" => Ok(language(&clean)),
        "help" => Ok(help(&clean)),
        _ => unreachable!(),
    }
}

fn task_update(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(task_id) = text(args, "task") else {
        return Err(Refused::Tool("`task_update` requires `task`.".into()));
    };
    let has_title = args.get("title").is_some();
    let has_desc = args.get("description").is_some();
    let has_dates = args.get("date").is_some() || args.get("deadline").is_some();
    let has_remind = args.get("add_reminders").is_some();
    let has_move = args.get("list").is_some() || args.get("part_of").is_some();
    if !(has_title || has_desc || has_dates || has_remind || has_move) {
        return Err(Refused::Tool(
            "`task_update` needs at least one field to change: `title`, `description`, `date`, \
             `deadline`, `add_reminders`, `list`, or `part_of`."
                .into(),
        ));
    }
    precheck_task_update(paths, args, &task_id)?;
    let mut answers = Vec::new();
    if has_title {
        let sub = json!({ "task": task_id, "title": args["title"] });
        answers.push(rename(paths, &sub)?);
    }
    if has_desc {
        let sub = json!({ "task": task_id, "body": args["description"] });
        answers.push(spoken(paths, "task_update", &sub, "body", |a| {
            describe(paths, a)
        })?);
    }
    if has_dates {
        let mut sub = Map::new();
        sub.insert("task".into(), json!(task_id));
        if let Some(d) = args.get("date") {
            sub.insert("date".into(), d.clone());
        }
        if let Some(d) = args.get("deadline") {
            sub.insert("deadline".into(), d.clone());
        }
        answers.push(reschedule(paths, &Value::Object(sub))?);
    }
    if has_remind {
        let sub = json!({ "task": task_id, "at": args["add_reminders"] });
        answers.push(remind(paths, &sub)?);
    }
    if has_move {
        let mut sub = Map::new();
        sub.insert("task".into(), json!(task_id));
        if let Some(l) = args.get("list") {
            sub.insert("list".into(), l.clone());
        }
        if let Some(p) = args.get("part_of") {
            sub.insert("part_of".into(), p.clone());
        }
        answers.push(move_task(paths, &Value::Object(sub))?);
    }
    Ok(merge_answers(answers))
}

fn precheck_task_update(paths: &Paths, args: &Value, task_id: &str) -> Result<(), Refused> {
    let has_title = args.get("title").is_some();
    let has_desc = args.get("description").is_some();
    let has_dates = args.get("date").is_some() || args.get("deadline").is_some();
    if has_title && text(args, "title").is_none() {
        return Err(Refused::Tool("renaming needs the new `title`.".into()));
    }
    if has_desc && text(args, "description").is_none() {
        return Err(Refused::Tool(
            "describing needs `description`, in the Tisty language.".into(),
        ));
    }
    let (on, owed) = match has_dates {
        true => (day(args, "date")?, day(args, "deadline")?),
        false => (None, None),
    };
    if args.get("add_reminders").is_some() && moments(args, "add_reminders")?.is_empty() {
        return Err(Refused::Tool(
            "`add_reminders` needs at least one timestamp like 2026-08-31T09:00.".into(),
        ));
    }
    let (state, _) = opened(paths)?;
    let (_, task) = reaching(&state, task_id)?;
    if (has_title || has_desc || has_dates)
        && let Some(why) = already_said_done(task, "updating it")
    {
        return Err(why);
    }
    if has_dates {
        if !state.agents_own(task) {
            return Err(Refused::Tool(format!(
                "{:?} is the person's task, so its dates stay theirs; nothing changed. Leave \
                 what you learnt with `task_note`.",
                task.title
            )));
        }
        let clears = |key: &str| args.get(key).is_some_and(Value::is_null);
        if on.is_none() && owed.is_none() && !clears("date") && !clears("deadline") {
            return Err(Refused::Tool(
                "moving a day needs a `date` or a `deadline`. Send null to take one off.".into(),
            ));
        }
        let eff_on = match (&on, clears("date")) {
            (Some(v), _) => Some(v.clone()),
            (None, true) => None,
            (None, false) => task.date.clone(),
        };
        let eff_owed = match (&owed, clears("deadline")) {
            (Some(v), _) => Some(v.clone()),
            (None, true) => None,
            (None, false) => task.deadline.clone(),
        };
        in_order(eff_on.as_ref(), eff_owed.as_ref())?;
    }
    if args.get("list").is_some() {
        let Some(name) = text(args, "list") else {
            return Err(Refused::Tool(
                "`list` must be a non-empty list name.".into(),
            ));
        };
        list_named(&state, &name)?;
    }
    if let Some(p) = args.get("part_of")
        && !p.is_null()
    {
        let Ok(up) = text(args, "part_of")
            .unwrap_or_default()
            .parse::<tisty_core::model::TaskId>()
        else {
            return Err(Refused::Tool(
                "`part_of` must be a parent task ID or null to detach.".into(),
            ));
        };
        hung_under(&state, task, up)?;
    }
    Ok(())
}

fn merge_answers(mut answers: Vec<Value>) -> Value {
    if answers.len() == 1 {
        return answers.remove(0);
    }
    let mut texts = Vec::new();
    let mut merged = Map::new();
    for one in answers {
        if let Some(text) = one["content"][0]["text"].as_str() {
            texts.push(text.to_string());
        }
        if let Some(fields) = one["structuredContent"].as_object() {
            for (k, v) in fields {
                if k != "said" {
                    merged.insert(k.clone(), v.clone());
                }
            }
        }
    }
    told(texts.join(" "), Value::Object(merged))
}

fn task_edit_steps(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let action = require_enum(
        args,
        "action",
        &["add", "tick", "untick", "reword", "remove", "set_state"],
    )?;
    let mut clean = map_without(args, &["action"]);
    match action {
        "add" => {
            if clean.get("steps").is_none()
                && let Some(one) = text(args, "step")
            {
                clean.remove("step");
                clean.insert("steps".into(), json!([one]));
            }
            plan(paths, &Value::Object(clean))
        }
        "tick" => {
            let mapped = Value::Object(clean);
            spoken(paths, "task_edit_steps", &mapped, "state", |a| {
                task_state::tick(paths, a)
            })
        }
        "set_state" => {
            if text(args, "step").is_some() || !strings(args, "steps")?.is_empty() {
                return Err(Refused::Tool(
                    "`action: \"set_state\"` updates `state` only; use `action: \"tick\"` to tick steps too.".into(),
                ));
            }
            if text(args, "state").is_none() {
                return Err(Refused::Tool(
                    "`action: \"set_state\"` requires `state`.".into(),
                ));
            }
            let mapped = Value::Object(clean);
            spoken(paths, "task_edit_steps", &mapped, "state", |a| {
                task_state::tick(paths, a)
            })
        }
        "untick" => untick(paths, &Value::Object(clean)),
        "reword" => reword_step(paths, &Value::Object(clean)),
        "remove" => unplan(paths, &Value::Object(clean)),
        _ => unreachable!(),
    }
}

fn task_note(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let action = require_enum(args, "action", &["add", "reword"])?;
    let clean = Value::Object(map_without(args, &["action"]));
    match action {
        "add" => spoken(paths, "task_note", &clean, "body", |a| note(paths, a)),
        "reword" => spoken(paths, "task_note", &clean, "body", |a| {
            reword_note(paths, a)
        }),
        _ => unreachable!(),
    }
}

fn task_resolve(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let outcome = require_enum(args, "outcome", &["done", "not_doing"])?;
    let clean = Value::Object(map_without(args, &["outcome"]));
    match outcome {
        "done" => spoken(paths, "task_resolve", &clean, "body", |a| {
            say_done(paths, a)
        }),
        "not_doing" => {
            if text(args, "branch").is_some() || text(args, "pr").is_some() {
                return Err(Refused::Tool(
                    "`branch` and `pr` apply only to `outcome: \"done\"`.".into(),
                ));
            }
            spoken(paths, "task_resolve", &clean, "body", |a| {
                say_not_doing(paths, a)
            })
        }
        _ => unreachable!(),
    }
}

fn doc_edit(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let mode = require_enum(
        args,
        "mode",
        &[
            "patch",
            "append",
            "rewrite",
            "compact",
            "digest_task_notes",
            "restore",
        ],
    )?;
    let mut mapped = map_without(args, &["mode", "content"]);
    match mode {
        "patch" => {
            let Some(content) = args.get("content") else {
                return Err(Refused::Tool(
                    "`doc_edit` (`mode: \"patch\"`) requires `content` (empty string deletes)."
                        .into(),
                ));
            };
            mapped.insert("new".into(), content.clone());
            edit_doc(paths, &Value::Object(mapped))
        }
        "append" => {
            if body(args, "content").is_none() {
                return Err(Refused::Tool(
                    "`doc_edit` (`mode: \"append\"`) requires non-empty `content`.".into(),
                ));
            }
            mapped.insert("body".into(), args["content"].clone());
            let mapped = Value::Object(mapped);
            spoken(paths, "doc_edit", &mapped, "body", |a| append_doc(paths, a))
        }
        "rewrite" => {
            if body(args, "content").is_none() {
                return Err(Refused::Tool(
                    "`doc_edit` (`mode: \"rewrite\"`) requires `content`.".into(),
                ));
            }
            mapped.insert("body".into(), args["content"].clone());
            let mapped = Value::Object(mapped);
            spoken(paths, "doc_edit", &mapped, "body", |a| write_doc(paths, a))
        }
        "compact" => {
            if body(args, "content").is_none() {
                return Err(Refused::Tool(
                    "`doc_edit` (`mode: \"compact\"`) requires `content`.".into(),
                ));
            }
            mapped.insert("body".into(), args["content"].clone());
            mapped.insert("compact".into(), json!(true));
            let mapped = Value::Object(mapped);
            spoken(paths, "doc_edit", &mapped, "body", |a| write_doc(paths, a))
        }
        "digest_task_notes" => {
            if body(args, "content").is_none() {
                return Err(Refused::Tool(
                    "`doc_edit` (`mode: \"digest_task_notes\"`) requires `content`.".into(),
                ));
            }
            mapped.insert("body".into(), args["content"].clone());
            mapped.insert("replace".into(), json!(true));
            let mapped = Value::Object(mapped);
            spoken(paths, "doc_edit", &mapped, "body", |a| append_doc(paths, a))
        }
        "restore" => restore_doc(paths, &Value::Object(mapped)),
        _ => unreachable!(),
    }
}

fn doc_edit_table(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let action = require_enum(
        args,
        "action",
        &[
            "set",
            "add_row",
            "remove_row",
            "sort",
            "set_type",
            "rename_column",
            "add_column",
            "remove_column",
        ],
    )?;
    let internal = match action {
        "add_row" => "add",
        "remove_row" => "remove",
        "set_type" => "type",
        "rename_column" => "rename",
        other => other,
    };
    let mut mapped = map_without(args, &["action"]);
    mapped.insert("do".into(), json!(internal));
    edit_table(paths, &Value::Object(mapped))
}

fn doc_organize(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let action = require_enum(
        args,
        "action",
        &[
            "create_folder",
            "file",
            "add_page",
            "remove_page",
            "reorder_pages",
            "link_translation",
            "unlink_translation",
        ],
    )?;
    match action {
        "create_folder" => {
            let mut mapped = map_without(args, &["action", "folder"]);
            if let Some(inside) = args.get("folder") {
                mapped.insert("inside".into(), inside.clone());
            }
            folder(paths, &Value::Object(mapped))
        }
        "file" => {
            let docs = strings(args, "docs")?;
            if docs.is_empty() {
                return Err(Refused::Tool(
                    "`action: \"file\"` requires `docs` with at least one document ID.".into(),
                ));
            }
            let mut mapped = map_without(args, &["action", "docs"]);
            mapped.insert("doc".into(), doc_arg(&docs));
            file_doc(paths, &Value::Object(mapped))
        }
        "add_page" => {
            let docs = strings(args, "docs")?;
            if docs.is_empty() || text(args, "parent").is_none() {
                return Err(Refused::Tool(
                    "`action: \"add_page\"` requires `docs` and `parent`.".into(),
                ));
            }
            let mut mapped = map_without(args, &["action", "docs", "parent", "position"]);
            mapped.insert("doc".into(), doc_arg(&docs));
            mapped.insert("page_of".into(), args["parent"].clone());
            if let Some(pos) = args.get("position") {
                mapped.insert("at".into(), pos.clone());
            }
            page_doc(paths, &Value::Object(mapped))
        }
        "remove_page" => {
            let docs = strings(args, "docs")?;
            if docs.is_empty() {
                return Err(Refused::Tool(
                    "`action: \"remove_page\"` requires `docs`.".into(),
                ));
            }
            if args.get("parent").is_some()
                || args.get("position").is_some()
                || args.get("after").is_some()
                || args.get("before").is_some()
                || args.get("order").is_some()
            {
                return Err(Refused::Tool(
                    "`action: \"remove_page\"` takes only `docs`; use `add_page` or `reorder_pages` for placement.".into(),
                ));
            }
            let mapped = json!({ "doc": doc_arg(&docs) });
            page_doc(paths, &mapped)
        }
        "reorder_pages" => {
            if text(args, "parent").is_none() || args.get("order").is_none() {
                return Err(Refused::Tool(
                    "`action: \"reorder_pages\"` requires `parent` and `order`.".into(),
                ));
            }
            let mut mapped = map_without(args, &["action", "parent"]);
            mapped.insert("page_of".into(), args["parent"].clone());
            page_doc(paths, &Value::Object(mapped))
        }
        "link_translation" => {
            let docs = one_doc(args, "link_translation")?;
            let Some(parent) = text(args, "parent") else {
                return Err(Refused::Tool(
                    "`action: \"link_translation\"` requires `parent`.".into(),
                ));
            };
            let mut mapped = map_without(args, &["action", "docs", "parent"]);
            mapped.insert("doc".into(), json!(docs));
            mapped.insert("of".into(), json!(parent));
            tongue_doc(paths, &Value::Object(mapped))
        }
        "unlink_translation" => {
            let docs = one_doc(args, "unlink_translation")?;
            if args.get("parent").is_some() {
                return Err(Refused::Tool(
                    "`action: \"unlink_translation\"` takes no `parent`.".into(),
                ));
            }
            tongue_doc(paths, &json!({ "doc": docs }))
        }
        _ => unreachable!(),
    }
}

fn doc_annotate(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let action = require_enum(
        args,
        "action",
        &["archive", "unarchive", "flag", "summarize"],
    )?;
    match action {
        "archive" | "unarchive" => {
            let docs = strings(args, "docs")?;
            if docs.is_empty() {
                return Err(Refused::Tool(format!(
                    "`action: \"{action}\"` requires `docs` with at least one document ID."
                )));
            }
            let away = action == "archive";
            let target_folder = text(args, "folder");
            // Validate every target before committing any op so partial batch archiving cannot occur.
            {
                let (state, _) = opened(paths)?;
                for one in &docs {
                    check_archive(paths, &state, one, away, target_folder.as_deref())?;
                }
            }
            let mut answers = Vec::with_capacity(docs.len());
            for one in docs {
                let mut mapped = map_without(args, &["action", "docs"]);
                mapped.insert("doc".into(), json!(one));
                mapped.insert("archived".into(), json!(away));
                answers.push(archive_doc(paths, &Value::Object(mapped))?);
            }
            Ok(merge_answers(answers))
        }
        "flag" => {
            let docs = strings(args, "docs")?;
            if docs.is_empty() {
                return Err(Refused::Tool(
                    "`action: \"flag\"` requires `docs` with at least one document or idea ID."
                        .into(),
                ));
            }
            let mut mapped = map_without(args, &["action", "docs"]);
            mapped.insert("doc".into(), json!(docs));
            flag_doc(paths, &Value::Object(mapped))
        }
        "summarize" => {
            let doc = one_doc(args, "summarize")?;
            let mut mapped = map_without(args, &["action", "docs"]);
            mapped.insert("doc".into(), json!(doc));
            sum_up(paths, &Value::Object(mapped))
        }
        _ => unreachable!(),
    }
}

fn file_transfer(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let action = require_enum(args, "action", &["attach", "import_doc", "export_doc"])?;
    match action {
        "attach" => attach(paths, &Value::Object(map_without(args, &["action"]))),
        "import_doc" => import_doc(paths, &Value::Object(map_without(args, &["action"]))),
        "export_doc" => {
            let Some(path) = args
                .get("path")
                .filter(|v| v.as_str().is_some_and(|s| !s.trim().is_empty()))
            else {
                return Err(Refused::Tool(
                    "`file_transfer` (`action: \"export_doc\"`) requires `path`, the destination folder."
                        .into(),
                ));
            };
            let mut mapped = map_without(args, &["action", "path"]);
            mapped.insert("into".into(), path.clone());
            export_doc(paths, &Value::Object(mapped))
        }
        _ => unreachable!(),
    }
}

fn doc_arg(docs: &[String]) -> Value {
    match docs {
        [one] => json!(one),
        many => json!(many),
    }
}

fn one_doc(args: &Value, action: &str) -> Result<String, Refused> {
    let docs = strings(args, "docs")?;
    match docs.as_slice() {
        [one] => Ok(one.clone()),
        [] => Err(Refused::Tool(format!(
            "`action: \"{action}\"` requires `docs` with one document ID."
        ))),
        _ => Err(Refused::Tool(format!(
            "`action: \"{action}\"` operates on one document at a time in `docs`."
        ))),
    }
}
