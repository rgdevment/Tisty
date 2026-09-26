mod asked;
mod catalogue;
mod jsonrpc;
mod papers;

use papers::editing::edit_doc;
use papers::filing::{archive_doc, file_doc, flag_doc, folder};
use papers::importing::import_doc;
use papers::paging::page_doc;
use papers::reading::{export_doc, outline_doc, read_doc};
use papers::writing::{append_doc, restore_doc, write_doc};

use asked::{day, in_order, moments, only_what_it_takes, ranked, short_and_plain, strings, text};

use catalogue::{instructions, tools};

use jsonrpc::{
    discovered, fault, introduced, legacy_greeting, named_tool, reply, speaking_through, told,
    wrong,
};

use std::io::{BufRead, Write};
use std::process::ExitCode;

use serde_json::{Value, json};
use tisty_core::{
    Op, Paths, State, Store, Task, TaskId,
    capture::{Draft, Rejected},
    event::{Body, LogAdd, Resolve, StepAdd, StepRef, TaskPatch},
    model::{DateSpec, FOLDER_NAME_AT_MOST, Priority, Tag},
    order,
    witness::{self, Fact},
};
use ulid::Ulid;

const VERSIONS: [&str; 3] = ["2026-07-28", "2025-11-25", "2025-06-18"];
const TOOLS_STAY_FRESH: i64 = 3_600_000;
const INBOX_TAG: &str = tisty_core::model::AGENT_TAG;
const LISTED_AT_MOST: usize = 200;

pub fn turn(paths: &Paths, on: Option<bool>, lang: crate::i18n::Lang) -> anyhow::Result<ExitCode> {
    let config = tisty_core::Config::load_or_init(paths)?;
    let named = |who: &tisty_core::DeviceId| tisty_core::config::nicknamed(&who.0);

    match (on, config.agent_id.clone()) {
        (None, Some(who)) => {
            println!(
                "  {}",
                lang.fill("agent-already", &[("name", &named(&who))])
            );
            println!("  {}", crate::style::dim(lang.get("agent-how")));
        }
        (None, None) => println!("  {}", crate::style::dim(lang.get("agent-none"))),
        (Some(true), Some(who)) => {
            println!(
                "  {}",
                lang.fill("agent-already", &[("name", &named(&who))])
            );
            println!("  {}", crate::style::dim(lang.get("agent-how")));
        }
        (Some(true), None) => {
            match let_in(crate::typist::at_the_persons_store(paths), at_a_terminal()) {
                Door::Asks if !agreed(lang) => {
                    println!("  {}", crate::style::dim(lang.get("agent-none")));
                    return Ok(ExitCode::SUCCESS);
                }
                Door::NoTerminal => {
                    eprintln!(
                        "{}: {}",
                        crate::style::paint(crate::style::RED, lang.get("error")),
                        lang.get("agent-needs-terminal")
                    );
                    return Ok(ExitCode::from(crate::EXIT_ERROR));
                }
                Door::Asks | Door::Open => {}
            }
            let who = tisty_core::agent::register(paths)?;
            println!("  {}", lang.fill("agent-on", &[("name", &named(&who))]));
            println!("  {}", crate::style::dim(lang.get("agent-how")));
        }
        (Some(false), Some(_)) => {
            tisty_core::agent::retire(paths)?;
            println!("  {}", lang.get("agent-off"));
        }
        (Some(false), None) => println!("  {}", crate::style::dim(lang.get("agent-not-on"))),
    }
    Ok(ExitCode::SUCCESS)
}

#[derive(Debug, PartialEq, Eq)]
enum Door {
    Open,
    Asks,
    NoTerminal,
}

/// Letting an assistant in is the person's act, so the person is asked — on the terminal
/// itself, which a shell an assistant drives does not have and a piped answer never reaches.
fn let_in(at_the_persons_store: bool, at_a_terminal: bool) -> Door {
    match (at_the_persons_store, at_a_terminal) {
        (false, _) => Door::Open,
        (true, true) => Door::Asks,
        (true, false) => Door::NoTerminal,
    }
}

fn at_a_terminal() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal() && std::io::stderr().is_terminal()
}

fn agreed(lang: crate::i18n::Lang) -> bool {
    dialoguer::Confirm::new()
        .with_prompt(lang.get("agent-sure"))
        .default(false)
        .interact()
        .unwrap_or(false)
}

pub fn serve(paths: Paths) -> anyhow::Result<ExitCode> {
    let mut stdin = std::io::stdin().lock();
    let mut out = std::io::stdout().lock();

    // Bytes, not lines: one stray non-UTF-8 byte would end the session and every request behind.
    let mut raw = Vec::new();
    while stdin.read_until(b'\n', &mut raw)? > 0 {
        let line = String::from_utf8_lossy(&raw).into_owned();
        raw.clear();
        if line.trim().is_empty() {
            continue;
        }
        let Some(said) = answer(&paths, &line) else {
            continue;
        };
        writeln!(out, "{said}")?;
        out.flush()?;
    }
    Ok(ExitCode::SUCCESS)
}

/// What the client called itself, kept for the session — one process serves one client — so
/// every event written from here says which hand spoke. Nothing decides on it.
fn answer(paths: &Paths, line: &str) -> Option<String> {
    let asked: Value = match serde_json::from_str(line) {
        Ok(asked) => asked,
        Err(why) => return Some(fault(Value::Null, -32700, &format!("parse error: {why}"))),
    };
    if asked.is_array() {
        return Some(fault(
            Value::Null,
            -32600,
            "one message per line, not a batch",
        ));
    }
    let id = asked.get("id").cloned();
    let method = asked.get("method").and_then(Value::as_str).unwrap_or("");

    // A notification has no id and takes no answer, whatever it says.
    let id = id?;
    let params = asked.get("params").cloned().unwrap_or(json!({}));

    if matches!(method, "server/discover" | "initialize") {
        introduced(&params);
    }

    Some(match method {
        "server/discover" => reply(id, discovered()),
        "initialize" => reply(id, legacy_greeting(&params)),
        "ping" => reply(id, json!({})),
        "tools/list" => reply(
            id,
            json!({
                "resultType": "complete",
                "tools": tools(),
                "ttlMs": TOOLS_STAY_FRESH,
                "cacheScope": "public",
            }),
        ),
        "tools/call" => match called(paths, &params) {
            Ok(said) => {
                witness::trace(
                    witness::channel::AGENT,
                    "the door answered a tool",
                    &[("tool", Fact::Id(named_tool(&params)))],
                );
                reply(id, said)
            }
            Err(Refused::Protocol(code, why)) => {
                witness::warn(
                    witness::channel::AGENT,
                    "the door was spoken to in a way it does not know",
                    &[
                        ("tool", Fact::Id(named_tool(&params))),
                        ("code", Fact::Count(code.unsigned_abs() as usize)),
                    ],
                );
                fault(id, code, &why)
            }
            Err(Refused::Tool(why)) => {
                witness::warn(
                    witness::channel::AGENT,
                    "the door turned a tool away",
                    &[
                        ("tool", Fact::Id(named_tool(&params))),
                        ("why", Fact::Why(why.clone())),
                    ],
                );
                reply(id, wrong(&why))
            }
        },
        _ => fault(id, -32601, &format!("unknown method: {method}")),
    })
}

enum Refused {
    Protocol(i32, String),
    Tool(String),
}

fn called(paths: &Paths, params: &Value) -> Result<Value, Refused> {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(json!({}));
    only_what_it_takes(name, &args)?;
    short_and_plain(&args)?;

    match name {
        "propose" => propose(paths, &args),
        "remind" => remind(paths, &args),
        "note" => note(paths, &args),
        "describe" => describe(paths, &args),
        "plan" => plan(paths, &args),
        "tick" => tick(paths, &args),
        "find" => find(paths, &args),
        "read" => read(paths, &args),
        "write_doc" => write_doc(paths, &args),
        "append_doc" => append_doc(paths, &args),
        "restore_doc" => restore_doc(paths, &args),
        "edit_doc" => edit_doc(paths, &args),
        "catch_up" => catch_up(paths, &args),
        "sum_up" => sum_up(paths, &args),
        "reschedule" => reschedule(paths, &args),
        "say_done" => say_done(paths, &args),
        "read_doc" => read_doc(paths, &args),
        "outline_doc" => outline_doc(paths, &args),
        "docs" => papers(paths, &args),
        "archive_doc" => archive_doc(paths, &args),
        "flag_doc" => flag_doc(paths, &args),
        "export_doc" => export_doc(paths, &args),
        "import_doc" => import_doc(paths, &args),
        "file_doc" => file_doc(paths, &args),
        "page_doc" => page_doc(paths, &args),
        "folder" => folder(paths, &args),
        "lists" => lists(paths),
        "tags" => tags(paths),
        "attach" => attach(paths, &args),
        "" => Err(Refused::Protocol(-32602, "a call needs a name".into())),
        other => Err(Refused::Protocol(
            -32602,
            match pointed(other) {
                Some(way) => format!("unknown tool: {other}. {way}"),
                None => format!("unknown tool: {other}"),
            },
        )),
    }
}

fn pointed(name: &str) -> Option<&'static str> {
    const ERASING_A_DOCUMENT: &str = "There is no deleting a document here, on purpose. Mark it \
         with `flag_doc`, saying what makes it old and how you know: the person sees the mark when \
         they open it and deletes it from the window in one click. Putting it away instead is \
         `archive_doc`.";
    const SHELVING: &str = "Putting a document away and bringing it back are both `archive_doc` — \
         with `archived` false it comes back. (`restore_doc` is another thing entirely: it takes \
         back an edit.)";
    const FINISHING: &str = "Finishing is the person's, and there is no tool for it. Say what you \
         did with `say_done` and the task stays open, marked, until they close it. What you only \
         learnt goes in `note`.";
    const DROPPING_A_TASK: &str = "Closing, dropping and erasing a task are the person's alone, \
         and no tool here does any of them. Record what you found with `note`.";
    const LOOKING_BACK: &str = "What is finished is not somewhere else: `find` and `docs` reach \
         it with `scope` set to `archive`, or `either` for both at once. A task the person closed \
         comes back from `find` like any other, and `read` says when and how it ended.";
    const LISTING: &str = "`find` searches the tasks and the documents — by text, or by the \
         sifting fields alone — and `docs` lists what is written with the folder each one sits \
         in. `catch_up` is the one to ask first when you arrive.";

    Some(match name {
        "delete_doc" | "remove_doc" | "drop_doc" | "erase_doc" | "trash_doc"
        | "delete_document" | "doc_delete" | "rm_doc" => ERASING_A_DOCUMENT,
        "unarchive_doc" | "unarchive" | "archive" | "shelve_doc" | "restore_archive"
        | "put_away" | "bring_back" => SHELVING,
        "complete" | "complete_task" | "close_task" | "finish" | "finish_task" | "mark_done"
        | "task_done" | "resolve" | "resolve_task" | "check_off" => FINISHING,
        "delete_task" | "remove_task" | "drop_task" | "erase_task" | "task_delete" | "close"
        | "done" | "drop" | "rm" => DROPPING_A_TASK,
        "archived" | "archived_tasks" | "archived_docs" | "list_archived" | "archive_list"
        | "completed" | "completed_tasks" | "done_tasks" | "closed_tasks" | "history" => {
            LOOKING_BACK
        }
        "list_tasks" | "tasks" | "list_docs" | "documents" | "search" | "query" | "list" => LISTING,
        _ => return None,
    })
}

/// A misspelt argument would otherwise be dropped in silence, teaching the model nothing.
fn one_of_many(paths: &Paths, one: &Value) -> Result<Value, Refused> {
    only_what_it_takes("propose", one)?;
    short_and_plain(one)?;
    if one.get("tasks").is_some() {
        return Err(Refused::Tool(
            "a task inside `tasks` cannot carry `tasks` of its own.".into(),
        ));
    }
    proposed(paths, one)
}

fn opened(paths: &Paths) -> Result<(State, Store), Refused> {
    let config = tisty_core::Config::load_or_init(paths).map_err(hitch)?;
    let Some(agent) = config.agent_id.clone() else {
        return Err(Refused::Tool(
            "no agent is registered on this machine. The person turns one on in Tisty's settings, \
             under Agents."
                .into(),
        ));
    };
    let state = tisty_core::cache::project(&paths.store(), paths.cache()).map_err(hitch)?;
    // The log says who may write, not the settings file an agent could edit itself.
    if !state.agents.contains(&agent) {
        return Err(Refused::Tool(
            "no agent is registered on this machine. The person turns one on in Tisty's settings, \
             under Agents."
                .into(),
        ));
    }
    let store = Store::open(paths.store(), agent)
        .map_err(hitch)?
        .speaking_through(speaking_through());
    Ok((state, store))
}

fn wrote(store: &mut Store, id: tisty_core::model::DocId) -> bool {
    store.read_all().is_ok_and(|told| {
        told.iter()
            .any(|one| matches!(&one.op, Op::DocAdd { id: which, .. } if which == &id))
    })
}

const UNSETTLED: &str = " Where its pages sit could not be settled just now — it settles by \
                         itself the next time the document is written or opened. Do not send \
                         this again.";

fn retold(state: &State, store: &mut Store, doc: &str, body: &str) -> Result<(), Refused> {
    let mut told = state.settling(doc, body);
    if let Some(kept) = state.docs.values().find(|one| one.file == doc) {
        let said = tisty_core::event::Said::of(body).by(state.signed.alias.clone());
        if said.news_for(kept) {
            told.push(Op::DocSaid {
                id: kept.id,
                d: said,
            });
        }
    }
    if told.is_empty() {
        return Ok(());
    }
    store.append_batch(told).map(|_| ()).map_err(hitch)
}

fn hitch(e: tisty_core::Error) -> Refused {
    Refused::Tool(match e {
        tisty_core::Error::AlreadyRunning => {
            "Tisty is being written to right now. Try the same call again.".into()
        }
        tisty_core::Error::UnsupportedVersion(_) => {
            "a newer Tisty updated the person's data, and this one cannot read it. Nothing was \
             read or written: tell the person to update Tisty on this machine, and stop."
                .into()
        }
        other => format!("Tisty could not be read or written: {other}"),
    })
}

const DRAFTS_AT_MOST: usize = 32;

/// Eight tasks in one call instead of eight calls: what the door costs an agent is mostly the
/// conversation it has to send again each time, not the writing.
fn propose(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(many) = args.get("tasks") else {
        return proposed(paths, args);
    };
    let Some(many) = many.as_array() else {
        return Err(Refused::Tool(
            "`tasks` is a list of what you want to propose. One on its own needs no list at all."
                .into(),
        ));
    };
    if many.is_empty() {
        return Err(Refused::Tool(
            "`tasks` came empty, so nothing was written.".into(),
        ));
    }
    if many.len() > DRAFTS_AT_MOST {
        return Err(Refused::Tool(format!(
            "{} is more than the {DRAFTS_AT_MOST} tasks one call takes. Send them in rounds.",
            many.len()
        )));
    }
    if many.iter().any(|one| !one.is_object()) {
        return Err(Refused::Tool(
            "every entry in `tasks` is a task of its own, written the same way a single one is."
                .into(),
        ));
    }

    let mut done: Vec<Value> = Vec::with_capacity(many.len());
    let (mut written, mut already, mut turned) = (0, 0, 0);
    for one in many {
        match one_of_many(paths, one) {
            Ok(said) => {
                let kept = said
                    .get("structuredContent")
                    .cloned()
                    .unwrap_or(Value::Null);
                match kept.get("proposed") == Some(&json!(true)) {
                    true => written += 1,
                    false => already += 1,
                }
                done.push(kept);
            }
            Err(Refused::Tool(said)) => {
                turned += 1;
                done.push(json!({
                    "title": text(one, "title"),
                    "proposed": false,
                    "refused": said,
                }));
            }
            Err(other) => return Err(other),
        }
    }

    let stepped = done
        .iter()
        .filter(|one| {
            one.get("steps")
                .and_then(Value::as_u64)
                .is_some_and(|n| n > 0)
        })
        .count();
    Ok(told(
        format!(
            "{written} written, {already} already there, {turned} turned away.\n{}{}",
            done.iter()
                .map(|one| match one.get("refused").and_then(Value::as_str) {
                    Some(why) => format!("{} — {why}", said(one, "title")),
                    None => format!("{} — {}", said(one, "id"), said(one, "title")),
                })
                .collect::<Vec<_>>()
                .join("\n"),
            match stepped {
                0 => String::new(),
                n => format!(
                    "\n{n} of them carry steps: `tick` each as you do it, `say_done` when all are."
                ),
            }
        ),
        json!({ "tasks": done, "written": written, "refused": turned }),
    ))
}

fn proposed(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(title) = text(args, "title") else {
        return Err(Refused::Tool("a task needs a `title`.".into()));
    };
    let (state, mut store) = opened(paths)?;

    let again = args.get("again").and_then(Value::as_bool).unwrap_or(false);
    if let Some(source) = text(args, "source")
        && let Some(held) = already(&state, &source)
        && state.is_erased(held)
        && !again
    {
        return Ok(told(
            "Already proposed from that source, and erased since: the person let it go. Nothing \
             was written. Only if they want it back, propose it once more with `again` set."
                .into(),
            json!({ "id": held.to_string(), "proposed": false, "erased": true }),
        ));
    }
    if let Some(source) = text(args, "source")
        && let Some(held) = already(&state, &source)
        && let Some(task) = state.tasks.get(&held)
        && !(again && !task.is_open())
    {
        let (said, kept) = match task.completed_at.filter(|_| !task.is_open()) {
            Some(at) => (
                format!(
                    "Already proposed from that source, and closed on {}: {:?}. {}. If the \
                     person wants it done again, propose it once more with `again` set, saying \
                     in the description how this one ended. Nothing was written.",
                    when(at),
                    task.title,
                    how_it_ended(task)
                ),
                json!({ "id": task.id.to_string(), "title": task.title, "proposed": false, "closed": when(at) }),
            ),
            None => (
                format!(
                    "Already proposed from that source: {:?}. Nothing was written.",
                    task.title
                ),
                json!({ "id": task.id.to_string(), "title": task.title, "proposed": false }),
            ),
        };
        return Ok(told(said, kept));
    }

    let on = day(args, "date")?;
    let owed = day(args, "deadline")?;
    in_order(on.as_ref(), owed.as_ref())?;

    let mut tags: Vec<Tag> = Vec::new();
    for one in strings(args, "tags")?
        .iter()
        .filter_map(|said| Tag::written(said).ok())
    {
        if !tags.contains(&one) {
            tags.push(one);
        }
    }
    if let Ok(mine) = Tag::new(INBOX_TAG)
        && !tags.contains(&mine)
    {
        tags.push(mine);
    }

    let draft = Draft {
        title: title.clone(),
        date: on,
        deadline: owed,
        priority: ranked(args)?,
        filing: text(args, "list").map(tisty_core::capture::Filing::Named),
        tags,
        repeat: None,
        source: text(args, "source"),
    };
    let plan =
        tisty_core::capture::plan(&state, draft).map_err(|e| with_the_names(refused(e), &state))?;
    let id = plan.task;
    let mut ops = plan.ops;
    if let Some(body) = text(args, "description") {
        ops.push(Op::TaskDescribe {
            id,
            d: Body { body: Some(body) },
        });
    }
    let bells = moments(args, "remind")?;
    if !bells.is_empty() {
        ops.push(Op::TaskUpdate {
            id,
            d: TaskPatch {
                reminders: Some(bells),
                ..Default::default()
            },
        });
    }
    let mut step = order::first();
    let mut planned = 0;
    for one in strings(args, "steps")? {
        planned += 1;
        ops.push(Op::StepAdd {
            id,
            d: StepAdd {
                step: Ulid::generate(),
                text: one,
                order: step.clone(),
            },
        });
        step = order::after(&step);
    }

    let source = text(args, "source");
    let taken = source.clone();
    // Checked again under the lock: two agents reading the same thread at once must not
    // both get through. A closed task from that source stands aside only when `again` says so.
    let written = store
        .append_batch_unless(ops, move |events| match &taken {
            None => false,
            Some(one) => {
                let held = State::replay(events);
                already(&held, one).is_some_and(|id| {
                    held.is_erased(id) && !again
                        || held
                            .tasks
                            .get(&id)
                            .is_some_and(|task| task.is_open() || !again)
                })
            }
        })
        .map_err(hitch)?;

    let Some(_) = written else {
        let held = State::replay(&store.read_all().map_err(hitch)?);
        let task = source
            .as_deref()
            .and_then(|one| already(&held, one))
            .and_then(|id| held.tasks.get(&id));
        return Ok(told(
            match task {
                Some(task) => format!(
                    "Already proposed from that source, as {}: {:?}. Nothing was written.",
                    task.id, task.title
                ),
                None => "Already proposed from that source. Nothing was written.".into(),
            },
            json!({
                "id": task.map(|one| one.id.to_string()),
                "title": task.map(|one| one.title.clone()),
                "proposed": false,
            }),
        ));
    };
    let landed = text(args, "list");
    let where_at = match &landed {
        Some(name) => format!("in {name}"),
        None => "in the inbox".to_string(),
    };
    let mut kept = serde_json::Map::new();
    kept.insert("id".into(), json!(id.to_string()));
    kept.insert("title".into(), json!(title));
    if let Some(landed) = &landed {
        kept.insert("list".into(), json!(landed));
    }
    kept.insert("proposed".into(), json!(true));
    if planned > 0 {
        kept.insert("steps".into(), json!(planned));
    }
    Ok(told(
        match planned {
            0 => format!("Proposed {title:?} as {id} {where_at}, tagged #{INBOX_TAG}."),
            n => format!(
                "Proposed {title:?} as {id} {where_at}, tagged #{INBOX_TAG}, with {n} step(s): \
                 `tick` each as you do it, `say_done` when all are."
            ),
        },
        Value::Object(kept),
    ))
}

fn remind(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("a reminder needs a `task` id.".into()));
    };
    let bells = moments(args, "at")?;
    if bells.is_empty() {
        return Err(Refused::Tool(
            "a reminder needs `at`, a day and an hour like 2026-08-31T09:00.".into(),
        ));
    }
    let (state, mut store) = opened(paths)?;
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(format!(
            "{said:?} is not a task id. Use the `id` that `find` or `propose` gave you."
        )));
    };
    let Some(task) = state.tasks.get(&id).filter(|one| !one.folded()) else {
        return Err(gone(&state, &said));
    };
    if !task.is_open() {
        return Err(history(task));
    }
    let mut all = task.reminders.clone();
    let mut added = 0;
    for one in bells {
        if !all.iter().any(|kept| kept.at == one.at) {
            all.push(one);
            added += 1;
        }
    }
    if added == 0 {
        return Ok(told(
            format!("{:?} was already set to ring then.", task.title),
            json!({ "id": id.to_string(), "title": task.title, "added": false }),
        ));
    }
    all.sort_by_key(|one| one.at);
    store
        .append(Op::TaskUpdate {
            id,
            d: TaskPatch {
                reminders: Some(all.clone()),
                ..Default::default()
            },
        })
        .map_err(hitch)?;
    Ok(told(
        format!("{:?} will ring {added} time(s) more.", task.title),
        json!({
            "id": id.to_string(),
            "title": task.title,
            "added": true,
            "reminders": all.iter().map(|one| one.at.to_string()).collect::<Vec<_>>(),
        }),
    ))
}

fn reschedule(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("moving a day needs a `task` id.".into()));
    };
    let on = day(args, "date")?;
    let owed = day(args, "deadline")?;
    let clears = |key: &str| args.get(key).is_some_and(Value::is_null);
    if on.is_none() && owed.is_none() && !clears("date") && !clears("deadline") {
        return Err(Refused::Tool(
            "moving a day needs a `date` or a `deadline`. Send null to take one off.".into(),
        ));
    }
    let (state, mut store) = opened(paths)?;
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(format!(
            "{said:?} is not a task id. Use the `id` that `find` or `propose` gave you."
        )));
    };
    let Some(task) = state.tasks.get(&id) else {
        return Err(gone(&state, &said));
    };
    if !state.filed_by_agents(task) {
        return Err(Refused::Tool(format!(
            "{:?} is the person's own, so its day is theirs to move — opened to agents or not. \
             You can only move what an agent filed. Say what you have learnt with `note` and \
             leave the day alone.",
            task.title
        )));
    }
    if !task.is_open() {
        return Err(history(task));
    }
    if let Some(why) = already_said_done(task, "moving its day") {
        return Err(why);
    }
    let standing =
        |given: &Option<DateSpec>, key: &str, held: &Option<DateSpec>| match (given, clears(key)) {
            (Some(one), _) => Some(one.clone()),
            (None, true) => None,
            (None, false) => held.clone(),
        };
    in_order(on.as_ref(), owed.as_ref())?;

    let was = (
        task.date.as_ref().map(|one| one.date().to_string()),
        task.deadline.as_ref().map(|one| one.date().to_string()),
    );
    let patch = TaskPatch {
        date: match (on.clone(), clears("date")) {
            (Some(one), _) => Some(Some(one)),
            (None, true) => Some(None),
            (None, false) => None,
        },
        deadline: match (owed.clone(), clears("deadline")) {
            (Some(one), _) => Some(Some(one)),
            (None, true) => Some(None),
            (None, false) => None,
        },
        ..Default::default()
    };
    store
        .append(Op::TaskUpdate { id, d: patch })
        .map_err(hitch)?;

    let now = (
        on.as_ref().map(|one| one.date().to_string()),
        owed.as_ref().map(|one| one.date().to_string()),
    );
    let moved = |what: &str, was: &Option<String>, now: &Option<String>, cleared: bool| match (
        now, cleared,
    ) {
        (Some(one), _) => Some(match was {
            Some(was) => format!("{what} {was} \u{2192} {one}"),
            None => format!("{what} {one}"),
        }),
        (None, true) => Some(format!("{what} taken off")),
        (None, false) => None,
    };
    let said = [
        moved("on", &was.0, &now.0, clears("date")),
        moved("owed", &was.1, &now.1, clears("deadline")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(", ");

    Ok(told(
        format!("Moved {:?}: {said}.", task.title),
        json!({
            "id": id.to_string(),
            "title": task.title,
            "date": standing(&on, "date", &task.date).map(|one| one.date().to_string()),
            "deadline": standing(&owed, "deadline", &task.deadline)
                .map(|one| one.date().to_string()),
        }),
    ))
}

/// What an assistant may fill in: a task it filed, or one the person opened to agents — open,
/// and not folded away. The refusal says which of the three it is not.
fn filling<'a>(state: &'a State, store: &Store, said: &str) -> Result<(TaskId, &'a Task), Refused> {
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(format!(
            "{said:?} is not a task id. Use the `id` that `find` or `propose` gave you."
        )));
    };
    let Some(task) = state.tasks.get(&id).filter(|one| !one.folded()) else {
        return Err(gone(state, said));
    };
    if !task.is_open() {
        return Err(history(task));
    }
    if !state.attended_by_agents(task) {
        return Err(Refused::Tool(format!(
            "{:?} is the person's own, and they have not opened it to you. You fill in what an \
             agent filed, or what they opened to agents; on the rest, say what you have learnt \
             with `note`.",
            task.title
        )));
    }
    let _ = store;
    Ok((id, task))
}

fn describe(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("describing needs a `task` id.".into()));
    };
    let Some(body) = text(args, "body") else {
        return Err(Refused::Tool(
            "describing needs a `body`, in markdown.".into(),
        ));
    };
    let (state, mut store) = opened(paths)?;
    let (id, task) = filling(&state, &store, &said)?;
    if let Some(why) = already_said_done(task, "writing a description") {
        return Err(why);
    }
    if task
        .description
        .as_deref()
        .is_some_and(|had| !had.trim().is_empty())
    {
        return Err(Refused::Tool(format!(
            "{:?} is already described, and a description is not yours to write over. Add what \
             you have learnt with `note`.",
            task.title
        )));
    }
    let written = store
        .append_batch_unless(
            vec![Op::TaskDescribe {
                id,
                d: Body { body: Some(body) },
            }],
            |events| {
                let held = State::replay(events);
                held.tasks.get(&id).is_none_or(|now| {
                    !still_filling(&held, now)
                        || now
                            .description
                            .as_deref()
                            .is_some_and(|had| !had.trim().is_empty())
                })
            },
        )
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }
    Ok(told(
        format!("Described {:?}.", task.title),
        json!({ "id": id.to_string(), "title": task.title }),
    ))
}

/// Still open, in sight, and open to this hand: what every fill-in checks again under the lock.
fn still_filling(held: &State, now: &Task) -> bool {
    now.is_open() && !now.folded() && held.attended_by_agents(now)
}

fn moved(task: &Task) -> Refused {
    Refused::Tool(format!(
        "{:?} moved while you were writing — the person, or another agent. `read` it again \
         before saying anything.",
        task.title
    ))
}

fn plan(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("planning needs a `task` id.".into()));
    };
    let steps = strings(args, "steps")?;
    if steps.is_empty() {
        return Err(Refused::Tool(
            "planning needs `steps`, the checklist to add, one string each.".into(),
        ));
    }
    let (state, mut store) = opened(paths)?;
    let (id, task) = filling(&state, &store, &said)?;
    if task.resolved.is_some() {
        return Err(Refused::Tool(format!(
            "{:?} has already been said done, and a step added now would stand unticked under \
             that mark — which is the very thing `say_done` refuses to do. Say what is still \
             left with `note` and leave the task to the person.",
            task.title
        )));
    }
    let mut ops = Vec::with_capacity(steps.len());
    let mut order = state.step_order_between(id, task.steps.last().map(|s| s.id), None);
    for one in &steps {
        ops.push(Op::StepAdd {
            id,
            d: StepAdd {
                step: Ulid::generate(),
                text: one.clone(),
                order: order.clone(),
            },
        });
        order = order::after(&order);
    }
    let written = store
        .append_batch_unless(ops, |events| {
            let held = State::replay(events);
            held.tasks
                .get(&id)
                .is_none_or(|now| !still_filling(&held, now) || now.resolved.is_some())
        })
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }
    Ok(told(
        format!("Planned {} step(s) on {:?}.", steps.len(), task.title),
        json!({ "id": id.to_string(), "title": task.title, "added": steps.len() }),
    ))
}

fn tick(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("ticking needs a `task` id.".into()));
    };
    let mut wanted = strings(args, "steps")?;
    if let Some(one) = text(args, "step") {
        wanted.push(one);
    }
    if wanted.is_empty() {
        return Err(Refused::Tool(
            "ticking needs `steps`: the text of each step you did, as `read` shows it \
             (`step` for one)."
                .into(),
        ));
    }
    let (state, mut store) = opened(paths)?;
    let (id, task) = filling(&state, &store, &said)?;
    let mut chosen: Vec<&tisty_core::model::Step> = Vec::new();
    for one in &wanted {
        let sought = tisty_core::text::folded(one.trim());
        let alike: Vec<&tisty_core::model::Step> = task
            .steps
            .iter()
            .filter(|step| tisty_core::text::folded(step.text.trim()) == sought)
            .collect();
        // Two steps written alike are a list with a line repeated: naming it ticks the next
        // one still open, so the checklist is walked down rather than stuck on the name.
        let Some(step) = alike
            .iter()
            .find(|step| !step.done && !chosen.iter().any(|had| had.id == step.id))
            .or_else(|| alike.first())
            .copied()
        else {
            return Err(Refused::Tool(format!(
                "no step of {:?} reads {one:?}; nothing was ticked. `read` shows them as they \
                 are written.",
                task.title
            )));
        };
        if !chosen.iter().any(|had| had.id == step.id) {
            chosen.push(step);
        }
    }
    let (already, fresh): (Vec<&tisty_core::model::Step>, Vec<&tisty_core::model::Step>) =
        chosen.iter().partition(|step| step.done);
    if !fresh.is_empty() {
        let ids: Vec<tisty_core::model::StepId> = fresh.iter().map(|step| step.id).collect();
        let ops = ids
            .iter()
            .map(|step| Op::StepDone {
                id,
                d: StepRef { step: *step },
            })
            .collect();
        // Judged again under the lock: a step the person ticked meanwhile is not ticked twice.
        let written = store
            .append_batch_unless(ops, |events| {
                let held = State::replay(events);
                held.tasks.get(&id).is_none_or(|now| {
                    !still_filling(&held, now)
                        || now
                            .steps
                            .iter()
                            .any(|step| step.done && ids.contains(&step.id))
                })
            })
            .map_err(hitch)?;
        if written.is_none() {
            return Err(moved(task));
        }
    }
    let left = task.steps.iter().filter(|step| !step.done).count() - fresh.len();
    let mut said = match fresh.len() {
        0 => format!("Nothing new ticked on {:?}: ", task.title),
        n => format!("Ticked {n} step(s) on {:?}. ", task.title),
    };
    if !already.is_empty() {
        said.push_str(&format!("{} ticked already. ", already.len()));
    }
    said.push_str(&match left {
        0 => "None left: `say_done` when the work is done.".to_string(),
        n => format!("{n} still unticked."),
    });
    Ok(told(
        said,
        json!({
            "id": id.to_string(),
            "title": task.title,
            "ticked": fresh.len(),
            "already": already.len(),
            "left": left,
        }),
    ))
}

fn note(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(body) = text(args, "body") else {
        return Err(Refused::Tool("a note needs a `body`.".into()));
    };
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("a note needs a `task` id.".into()));
    };
    let (state, mut store) = opened(paths)?;
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(format!(
            "{said:?} is not a task id. Use the `id` that `find` or `propose` gave you."
        )));
    };
    let Some(task) = state.tasks.get(&id) else {
        return Err(gone(&state, &said));
    };
    if !task.is_open() {
        return Err(history(task));
    }

    let zone = jiff::tz::TimeZone::system();
    store
        .append(Op::TaskLog {
            id,
            d: LogAdd::new(Ulid::generate(), body).in_zone(zone.iana_name().map(str::to_string)),
        })
        .map_err(hitch)?;
    Ok(told(
        format!("Noted on {:?}.", task.title),
        json!({ "id": id.to_string(), "title": task.title }),
    ))
}

fn when(at: jiff::Timestamp) -> String {
    at.to_zoned(jiff::tz::TimeZone::system())
        .strftime("%Y-%m-%d %H:%M")
        .to_string()
}

const UNTICKED_NAMED: usize = 5;

/// A mark beside an unticked checklist reads as work nobody did.
fn unticked(task: &Task) -> Option<Refused> {
    let left: Vec<&str> = task
        .steps
        .iter()
        .filter(|step| !step.done)
        .map(|step| step.text.trim())
        .collect();
    if left.is_empty() {
        return None;
    }
    let mut named: Vec<String> = left
        .iter()
        .take(UNTICKED_NAMED)
        .map(|one| format!("{:?}", one.chars().take(80).collect::<String>()))
        .collect();
    if left.len() > UNTICKED_NAMED {
        named.push(format!("and {} more", left.len() - UNTICKED_NAMED));
    }
    Some(Refused::Tool(format!(
        "{:?} still has {} step(s) unticked: {}. `tick` the ones you did; if one no longer \
         applies, say so with `note` and leave the task open — the person decides.",
        task.title,
        left.len(),
        named.join(", ")
    )))
}

fn say_done(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool(
            "saying a task is done needs the `task` id.".into(),
        ));
    };
    let Some(body) = text(args, "body") else {
        return Err(Refused::Tool(
            "saying a task is done needs a `body`: what you did and how you know it holds. \
             Without it the person has only your word and nothing to check it against."
                .into(),
        ));
    };
    let (state, mut store) = opened(paths)?;
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(format!(
            "{said:?} is not a task id. Use the `id` that `find` or `propose` gave you."
        )));
    };
    let Some(task) = state.tasks.get(&id) else {
        return Err(gone(&state, &said));
    };
    let me = store.device().clone();
    if !state.attended_by_agents(task) {
        return Err(Refused::Tool(format!(
            "{:?} is the person's own, and they have not opened it to you: whether it is done \
             is theirs to say. You can only speak for what an agent filed, or for what they \
             opened to agents. Say what you have learnt with `note` instead.",
            task.title
        )));
    }
    if !task.is_open() {
        return Err(history(task));
    }
    if task.folded() {
        return Err(Refused::Tool(format!(
            "{:?} was put out of sight by the person, so it is not yours to speak for. Say what \
             you have learnt with `note` instead.",
            task.title
        )));
    }
    if let Some(already) = &task.resolved {
        let who = match already.by == me && already.via == speaking_through() {
            true => "you".to_string(),
            false => already
                .via
                .as_deref()
                .map(tisty_core::agent::client_named)
                .unwrap_or_else(|| "an assistant".to_string()),
        };
        return Err(Refused::Tool(format!(
            "{who} already said {:?} was done on {}, and the person has not looked yet. Saying \
             it again would only stack another entry on the journal — add what is new with \
             `note`.",
            task.title,
            when(already.at)
        )));
    }
    if let Some(refusal) = unticked(task) {
        return Err(refusal);
    }

    let zone = jiff::tz::TimeZone::system();
    let entry = Ulid::generate();
    // Judged again under the lock: the mark must not land beside a step the person just
    // unticked, nor on top of one another agent just left.
    let written = store
        .append_batch_unless(
            vec![
                Op::TaskLog {
                    id,
                    d: LogAdd::new(entry, body).in_zone(zone.iana_name().map(str::to_string)),
                },
                Op::TaskResolve {
                    id,
                    d: Resolve::new(entry).said_by(jiff::Timestamp::now(), me.clone()),
                },
            ],
            |events| {
                let held = State::replay(events);
                held.tasks.get(&id).is_none_or(|now| {
                    !still_filling(&held, now)
                        || now.resolved.is_some()
                        || now.steps.iter().any(|step| !step.done)
                })
            },
        )
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }

    Ok(told(
        format!(
            "Said done: {:?}. It stays open until the person finishes it.",
            task.title
        ),
        json!({ "id": id.to_string(), "title": task.title, "open": true }),
    ))
}

enum Beside {
    Task(TaskId),
    Doc(String),
}

fn beside(state: &State, args: &Value) -> Result<Beside, Refused> {
    match (text(args, "task"), text(args, "doc")) {
        (Some(_), Some(_)) => Err(Refused::Tool(
            "attaching takes a `task` or a `doc`, not both: a file is kept in one place.".into(),
        )),
        (None, None) => Err(Refused::Tool(
            "attaching needs the `task` it belongs to, or the `doc` it goes in.".into(),
        )),
        (Some(who), None) => {
            let Ok(id) = who.parse::<TaskId>() else {
                return Err(Refused::Tool(format!(
                    "{who:?} is not a task id. Use the `id` that `find` or `propose` gave you."
                )));
            };
            match state.tasks.get(&id).filter(|one| !one.folded()) {
                Some(task) if !task.is_open() => Err(history(task)),
                Some(_) => Ok(Beside::Task(id)),
                None => Err(Refused::Tool(format!("no task here has the id {who}."))),
            }
        }
        (None, Some(which)) => {
            let Some(kept) = state.docs.values().find(|one| one.file == which) else {
                return Err(Refused::Tool(format!(
                    "no document here is called {which:?}. `docs` lists them all."
                )));
            };
            if state.shut(kept.id) {
                return Err(Refused::Tool(format!(
                    "{which:?} is locked. The person shut it so nothing writes in it — not the \
                     window, not you. Ask them to unlock it if the file truly has to go there."
                )));
            }
            match state.held_away(kept) {
                true => Err(Refused::Tool(format!(
                    "{which:?} is put away, so nothing more goes into it. Keep the file with a \
                     task, or in a document that is still open."
                ))),
                false => Ok(Beside::Doc(which)),
            }
        }
    }
}

fn room(paths: &Paths, which: &str, named: &str) -> Result<(), Refused> {
    let full = || {
        Refused::Tool(format!(
            "{which:?} has no room left for the line that names a file. Keep it with a task, or \
             in a new document."
        ))
    };
    let body = match tisty_core::docs::read(&paths.docs(), which) {
        Ok(body) => body,
        Err(tisty_core::Error::DocumentTooBig { .. }) => return Err(full()),
        Err(other) => return Err(hitch(other)),
    };
    match tisty_core::attach::fits(&body, named) {
        Ok(()) => Ok(()),
        Err(tisty_core::attach::NoRoom::Full) => Err(full()),
        Err(tisty_core::attach::NoRoom::Crowded(held)) => Err(Refused::Tool(format!(
            "{which:?} already carries {held} files, which is as many as a document is read with. \
             Keep this one with a task, or in a document of its own."
        ))),
    }
}

fn attach(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "path") else {
        return Err(Refused::Tool(
            "attaching needs a `path` to a file on this machine.".into(),
        ));
    };
    let (state, mut store) = opened(paths)?;
    let with = beside(&state, args)?;

    let asked = std::path::Path::new(&said);
    let at = &tisty_core::agent::may_attach(asked, paths).map_err(|why| {
        Refused::Tool(match why {
            tisty_core::Error::NotForAnAgent(_) => format!(
                "{said:?} does not hold what its name says it does — a .png whose bytes are not a \
                 PNG, say — so what came back out of Tisty would not open. Any kind of \
                 file is kept; this one is turned away only because its name lies."
            ),
            _ => format!(
                "{said:?} is not somewhere an assistant may take files from. Those are: {}.",
                tisty_core::agent::reachable()
                    .iter()
                    .map(|one| one.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        })
    })?;
    let named = tisty_core::attach::called(at, text(args, "label"));
    if let Beside::Doc(which) = &with {
        room(paths, which, &named)?;
    }
    let config = tisty_core::Config::load_or_init(paths).map_err(hitch)?;
    let limit = match &with {
        Beside::Task(_) => config.copies_up_to(),
        Beside::Doc(_) => config.copies_in_a_doc(),
    };
    let kept = tisty_core::attach::keep(at, paths.data(), limit).map_err(|e| match e {
        tisty_core::Error::AttachmentTooBig { bytes, .. } => Refused::Tool(format!(
            "that file is {} MB and this machine copies at most {} MB {}.",
            bytes / 1_000_000,
            limit / 1_000_000,
            match with {
                Beside::Task(_) => "onto a task",
                Beside::Doc(_) => "into a document",
            }
        )),
        _ => Refused::Tool(format!("{said:?} could not be read from this machine.")),
    })?;

    match with {
        Beside::Task(id) => {
            let lang = crate::i18n::Lang::detect(config.locale.as_deref());
            let body = tisty_core::attach::journalled(&kept, &named, at, lang.get("attached-from"));
            let zone = jiff::tz::TimeZone::system();
            store
                .append(Op::TaskLog {
                    id,
                    d: LogAdd::new(Ulid::generate(), body)
                        .in_zone(zone.iana_name().map(str::to_string)),
                })
                .map_err(hitch)?;
            let title = state.tasks[&id].title.clone();
            Ok(told(
                format!("Kept {named:?} with {title:?}."),
                json!({ "id": id.to_string(), "at": kept.at, "label": named }),
            ))
        }
        Beside::Doc(which) => {
            let whole = tisty_core::docs::append(&paths.docs(), &which, &kept.written(&named))
                .map_err(|e| match e {
                    tisty_core::Error::DocumentTooBig { limit, .. } => Refused::Tool(format!(
                        "that would take the document past the {limit} bytes Tisty can open. \
                         Keep the file with a task, or in a new document."
                    )),
                    other => hitch(other),
                })?;
            let title = tisty_core::docs::titled(&whole);
            Ok(told(
                format!("Kept {named:?} at the end of {title:?}."),
                json!({ "doc": which, "title": title, "at": kept.at, "label": named }),
            ))
        }
    }
}

fn said<'a>(one: &'a Value, key: &str) -> &'a str {
    one[key].as_str().unwrap_or_default()
}

fn scoped(args: &Value) -> Result<tisty_core::view::Scope, Refused> {
    match text(args, "scope").as_deref() {
        Some("open") => Ok(tisty_core::view::Scope::Open),
        Some("archive") => Ok(tisty_core::view::Scope::Archived),
        None | Some("either") => Ok(tisty_core::view::Scope::Either),
        Some(said) => Err(Refused::Tool(format!(
            "`scope` is open, archive or either — not {said:?}."
        ))),
    }
}

fn find(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;

    if let Some(source) = text(args, "source") {
        if text(args, "query").is_some() {
            return Err(Refused::Tool(
                "`find` takes a `source` or a `query`, not both: one asks whether this exact thing was already filed, the other searches. Send one."
                    .into(),
            ));
        }
        let filed = already(&state, &source);
        if let Some(id) = filed.filter(|id| state.is_erased(*id)) {
            return Ok(told(
                "Already proposed from that source, and erased since: the person let it go. A \
                 new filing from this source takes `again`, and only if they want it back."
                    .into(),
                json!({ "found": { "id": id.to_string(), "erased": true } }),
            ));
        }
        let held = filed.and_then(|id| state.tasks.get(&id));
        return Ok(told(
            match held {
                Some(task) if !task.is_open() => format!(
                    "Already proposed from that source, and closed since ({}): {:?}. {}; a new \
                     filing from this source takes `again`.",
                    standing(task),
                    task.title,
                    how_it_ended(task)
                ),
                Some(task) => format!("Already proposed from that source: {:?}", task.title),
                None => "Nothing here came from that source.".into(),
            },
            json!({ "found": held.map(|task| brief(task, &state)) }),
        ));
    }

    if let Some(which) = text(args, "doc") {
        return inside_a_doc(paths, &state, &which, args);
    }

    let sifted = Sifted::asked(args)?;
    let query = text(args, "query");
    if query.is_none() && sifted.none() {
        return Err(Refused::Tool(
            "`find` needs a `query`, a `source` to check whether it was proposed already, a \
             `doc` to look inside, or one of `tag`, `list`, `by_agent`, `said_done`, \
             `open_to_agents`, `from_source`, `from` and `to` to sift by."
                .into(),
        ));
    }
    let scope = scoped(args)?;
    let most = args
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(20)
        .clamp(1, 100) as usize;
    let past = args.get("after").and_then(Value::as_u64).unwrap_or(0) as usize;

    // Counting what it cannot see would still say the thing exists.
    let hits: Vec<&Task> = match &query {
        Some(query) => state.searching(query, scope, usize::MAX).0,
        None => {
            let mut all: Vec<&Task> = state
                .tasks
                .values()
                .filter(|one| match scope {
                    tisty_core::view::Scope::Open => one.is_open(),
                    tisty_core::view::Scope::Archived => one.is_archived(),
                    tisty_core::view::Scope::Either => true,
                })
                .collect();
            all.sort_by_key(|one| (one.date.as_ref().map(|d| d.date()), one.id));
            all
        }
    };
    let hits: Vec<&Task> = hits
        .into_iter()
        .filter(|one| !one.folded() && sifted.keeps(one, &state))
        .collect();
    let all = hits.len();
    let hits: Vec<&Task> = hits.into_iter().skip(past).take(most).collect();
    let found: Vec<Value> = hits.iter().map(|task| brief(task, &state)).collect();
    // `after` walks the tasks only — paging past them would empty this list without saying why.
    let papers = match (&query, sifted.none()) {
        (Some(query), true) => papers_matching(paths, &state, query, scope, usize::MAX),
        _ => Vec::new(),
    };
    let papers_all = papers.len();
    let papers: Vec<Value> = papers.into_iter().take(most).collect();
    let mut lines: Vec<String> = hits
        .iter()
        .map(|task| format!("{} — {} ({})", task.id, task.title, standing(task)))
        .collect();
    lines.extend(papers.iter().map(|one| {
        let put_away = if one["archived"] == json!(true) {
            ", put away"
        } else {
            ""
        };
        let what = match one["page_of"].as_str() {
            Some(up) => format!("page of {up}"),
            None => "document".into(),
        };
        format!(
            "{} — {} ({what}{put_away})",
            said(one, "doc"),
            said(one, "title")
        )
    }));
    let asked = query.clone().unwrap_or_else(|| sifted.said());
    Ok(told(
        format!(
            "{all} task(s) and {papers_all} document(s) match {asked:?}; showing {} and {}.
{}",
            found.len(),
            papers.len(),
            lines.join(
                "
"
            )
        ),
        json!({
            "matches": found,
            "total": all,
            "docs": papers,
            "docsTotal": papers_all,
        }),
    ))
}

struct Sifted {
    tag: Option<String>,
    list: Option<String>,
    by_agent: Option<bool>,
    said_done: Option<bool>,
    open_to_agents: Option<bool>,
    from_source: Option<String>,
    from: Option<jiff::civil::Date>,
    to: Option<jiff::civil::Date>,
}

impl Sifted {
    fn asked(args: &Value) -> Result<Self, Refused> {
        let on = |key: &str| -> Result<Option<jiff::civil::Date>, Refused> {
            let Some(said) = text(args, key) else {
                return Ok(None);
            };
            said.parse::<jiff::civil::Date>().map(Some).map_err(|_| {
                Refused::Tool(format!(
                    "`{key}` has to be a plain date like 2026-08-31, not {said:?}."
                ))
            })
        };
        Ok(Self {
            tag: text(args, "tag").map(|one| one.trim_start_matches('#').to_lowercase()),
            list: text(args, "list").map(|one| one.to_lowercase()),
            by_agent: args.get("by_agent").and_then(Value::as_bool),
            said_done: args.get("said_done").and_then(Value::as_bool),
            open_to_agents: args.get("open_to_agents").and_then(Value::as_bool),
            from_source: text(args, "from_source").map(|one| alike(&one)),
            from: on("from")?,
            to: on("to")?,
        })
    }

    fn none(&self) -> bool {
        self.tag.is_none()
            && self.list.is_none()
            && self.by_agent.is_none()
            && self.said_done.is_none()
            && self.open_to_agents.is_none()
            && self.from_source.is_none()
            && self.from.is_none()
            && self.to.is_none()
    }

    fn said(&self) -> String {
        let mut all = Vec::new();
        if let Some(one) = &self.tag {
            all.push(format!("#{one}"));
        }
        if let Some(one) = &self.list {
            all.push(format!("in {one}"));
        }
        match self.by_agent {
            Some(true) => all.push("filed by an agent".into()),
            Some(false) => all.push("written by the person".into()),
            None => {}
        }
        match self.said_done {
            Some(true) => all.push("already said done".into()),
            Some(false) => all.push("not said done yet".into()),
            None => {}
        }
        match self.open_to_agents {
            Some(true) => all.push("opened to agents".into()),
            Some(false) => all.push("kept to the person".into()),
            None => {}
        }
        if let Some(one) = &self.from_source {
            all.push(format!("out of {one}"));
        }
        if let (Some(from), Some(to)) = (self.from, self.to) {
            all.push(format!("{from} to {to}"));
        } else if let Some(from) = self.from {
            all.push(format!("from {from}"));
        } else if let Some(to) = self.to {
            all.push(format!("up to {to}"));
        }
        all.join(", ")
    }

    fn keeps(&self, task: &Task, state: &State) -> bool {
        if let Some(want) = &self.tag
            && !task
                .tags
                .iter()
                .any(|one| one.as_str().to_lowercase() == *want)
        {
            return false;
        }
        if let Some(want) = &self.list {
            let named = task
                .list
                .as_ref()
                .and_then(|id| state.lists.get(id))
                .map(|one| one.name.to_lowercase());
            if named.as_deref() != Some(want.as_str()) {
                return false;
            }
        }
        if let Some(want) = self.by_agent {
            let by = task
                .created_by
                .as_ref()
                .is_some_and(|who| state.assistants.contains(who));
            if by != want {
                return false;
            }
        }
        if let Some(want) = self.said_done
            && task.resolved.is_some() != want
        {
            return false;
        }
        // False is what the person kept to themselves: their own, unopened. What an agent filed
        // is neither, and answers to `by_agent`.
        if let Some(want) = self.open_to_agents
            && (task.open_to_agents != want
                || (!want
                    && task
                        .created_by
                        .as_ref()
                        .is_some_and(|who| state.assistants.contains(who))))
        {
            return false;
        }
        if let Some(want) = &self.from_source {
            let came = task.source.as_deref().map(alike);
            if !came.is_some_and(|one| one.starts_with(want.as_str())) {
                return false;
            }
        }
        if self.from.is_some() || self.to.is_some() {
            let days: Vec<jiff::civil::Date> = [task.date.as_ref(), task.deadline.as_ref()]
                .into_iter()
                .flatten()
                .map(|one| one.date())
                .collect();
            if days.is_empty() {
                return false;
            }
            let within = days.iter().any(|one| {
                self.from.is_none_or(|first| *one >= first)
                    && self.to.is_none_or(|last| *one <= last)
            });
            if !within {
                return false;
            }
        }
        true
    }
}

const AROUND_A_HIT: usize = 1;

fn inside_a_doc(paths: &Paths, state: &State, which: &str, args: &Value) -> Result<Value, Refused> {
    let Some(query) = text(args, "query") else {
        return Err(Refused::Tool(
            "looking inside a document needs a `query` to look for.".into(),
        ));
    };
    if state.docs.values().all(|one| one.file != which) {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        )));
    }
    let body = tisty_core::docs::read(&paths.docs(), which).map_err(hitch)?;
    let most = args
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(20)
        .clamp(1, 100) as usize;

    let terms = tisty_core::text::terms(&query);
    if terms.is_empty() {
        return Err(Refused::Tool(
            "looking inside a document needs a `query` with a word in it.".into(),
        ));
    }
    let lines: Vec<&str> = body.lines().collect();
    let at: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, one)| {
            let flat = tisty_core::text::folded(one);
            terms.iter().all(|term| flat.contains(term.as_str()))
        })
        .map(|(n, _)| n)
        .collect();
    let all = at.len();

    let outline = tisty_core::docs::outlined(&body);
    let section_of = |line: usize| {
        outline
            .iter()
            .rev()
            .find(|one| one.line <= line && line <= one.to)
            .map(|one| json!({ "at": one.at, "title": one.title }))
    };
    let found: Vec<Value> = at
        .iter()
        .take(most)
        .map(|n| {
            let first = n.saturating_sub(AROUND_A_HIT);
            let last = (n + AROUND_A_HIT).min(lines.len().saturating_sub(1));
            let mut hit = serde_json::Map::new();
            hit.insert("line".into(), json!(n + 1));
            hit.insert("text".into(), json!(lines[*n]));
            hit.insert("around".into(), json!(lines[first..=last].join("\n")));
            if let Some(section) = section_of(n + 1) {
                hit.insert("section".into(), section);
            }
            Value::Object(hit)
        })
        .collect();

    let said = match all {
        0 => format!("Nothing in {which:?} says {query:?}."),
        _ => format!(
            "{all} line(s) of {which:?} say {query:?}; showing {}.\n{}",
            found.len(),
            found
                .iter()
                .map(|one| match one["section"].is_object() {
                    true => format!(
                        "line {} (section {}, {}) — {}",
                        one["line"],
                        one["section"]["at"],
                        said(&one["section"], "title"),
                        said(one, "text")
                    ),
                    false => format!("line {} — {}", one["line"], said(one, "text")),
                })
                .collect::<Vec<_>>()
                .join("\n")
        ),
    };
    Ok(told(
        said,
        json!({
            "doc": which,
            "lines": found,
            "total": all,
            "print": tisty_core::attach::printed(body.as_bytes()),
        }),
    ))
}

fn named(status: tisty_core::model::Status) -> &'static str {
    match status {
        tisty_core::model::Status::Open => "open",
        tisty_core::model::Status::Done => "done",
        tisty_core::model::Status::Dropped => "dropped",
    }
}

/// The tombstone knows: what the person erased is said to be erased, not merely missing.
fn gone(state: &State, said: &str) -> Refused {
    match said.parse::<TaskId>() {
        Ok(id) if state.is_erased(id) => Refused::Tool(format!(
            "{said} was erased by the person, and stays erased. If the same work has come back, \
             propose it anew."
        )),
        _ => Refused::Tool(format!(
            "no task here has the id {said}. It may have been deleted."
        )),
    }
}

fn standing(task: &Task) -> String {
    match task.completed_at.filter(|_| !task.is_open()) {
        Some(at) => format!("{} {}", named(task.status), when(at)),
        None => named(task.status).to_string(),
    }
}

fn ended(task: &Task) -> String {
    task.completed_at
        .map(|at| format!(" on {}", when(at)))
        .unwrap_or_default()
}

/// Done and dropped are not the same thing to tell the person who asks; and what they put
/// away, `read` does not reach.
fn how_it_ended(task: &Task) -> String {
    let mut said = match task.status {
        tisty_core::model::Status::Dropped => {
            "The person dropped it — they decided not to do it — and it is history".to_string()
        }
        _ => "It is done, and history".to_string(),
    };
    if task.folded() {
        said.push_str(", put away by the person where `read` does not reach");
    }
    said
}

fn already_said_done(task: &Task, doing: &str) -> Option<Refused> {
    task.resolved.as_ref().map(|_| {
        Refused::Tool(format!(
            "{:?} has already been said done, so {doing} now would speak over a mark nobody has \
             looked at yet. Say what you have learnt with `note` and leave the task to the \
             person.",
            task.title
        ))
    })
}

fn history(task: &Task) -> Refused {
    let ended = ended(task);
    let how = match task.status {
        tisty_core::model::Status::Dropped => {
            "was dropped — the person decided not to do it —".to_string()
        }
        _ => format!("was closed{ended} and"),
    };
    let look = match task.folded() {
        true => "it is put away, so `read` does not reach it; if the same work has come back",
        false => "if the same work has come back, `read` how this one ended and",
    };
    Refused::Tool(format!(
        "{:?} {how} is history now: it reads as it ended, and nothing on it changes — not its \
         day, not its journal, not a mark saying it is done. And {look} propose a new task whose \
         description says so, naming this one by its title and its id {}, with a source of its \
         own.",
        task.title, task.id
    ))
}

fn read(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("reading needs a `task` id.".into()));
    };
    let (state, _) = opened(paths)?;
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(format!(
            "{said:?} is not a task id. Use the `id` that `find` gave you."
        )));
    };
    // Hidden is the person taking something out of sight. An agent reading its journal whole
    // would undo that decision on the one task most likely to deserve it.
    let Some(task) = state.tasks.get(&id).filter(|one| !one.folded()) else {
        if state.is_erased(id) {
            return Err(gone(&state, &said));
        }
        return Err(Refused::Tool(format!(
            "no task here has the id {said}. Look it up again with `find`."
        )));
    };

    let asked = strings(args, "fields")?;
    let wants = |key: &str| asked.is_empty() || asked.iter().any(|one| one == key);

    let mut whole = match asked.is_empty() {
        true => brief(task, &state),
        false => {
            let brief = brief(task, &state);
            let mut kept = serde_json::Map::new();
            kept.insert("id".into(), json!(task.id.to_string()));
            for key in &asked {
                if let Some(one) = brief.get(key.as_str()) {
                    kept.insert(key.clone(), one.clone());
                }
            }
            Value::Object(kept)
        }
    };
    if wants("description") && task.description.is_some() {
        whole["description"] = json!(task.description);
    }
    if wants("steps") && !task.steps.is_empty() {
        whole["steps"] = json!(
            task.steps
                .iter()
                .map(|one| json!({ "text": one.text, "done": one.done }))
                .collect::<Vec<_>>()
        );
    }
    if wants("journal") && !task.log.is_empty() {
        whole["journal"] = json!(
            task.log
                .iter()
                .map(|one| json!({ "at": one.at.to_string(), "body": kept_here(&one.body) }))
                .collect::<Vec<_>>()
        );
    }
    if wants("kept") && !task.references().is_empty() {
        whole["kept"] = json!(
            task.references()
                .iter()
                .map(|one| json!({ "target": one.target, "label": one.label }))
                .collect::<Vec<_>>()
        );
    }

    let mut plainly = format!("{} — {}", task.id, task.title);
    if !task.is_open() {
        let notice = format!(
            "{} — {}: it reads as it ended, and nothing on it changes",
            standing(task),
            how_it_ended(task)
        );
        plainly.push_str(&format!(" ({notice})"));
        whole["notice"] = json!(notice);
    } else if task.open_to_agents {
        plainly.push_str(" (open to agents: yours to describe, plan, tick and say done)");
    }
    if let Some(body) = &task.description
        && wants("description")
    {
        plainly.push_str("\n\n");
        plainly.push_str(body);
    }
    if wants("steps") {
        for one in &task.steps {
            plainly.push_str(&format!(
                "\n[{}] {}",
                if one.done { 'x' } else { ' ' },
                one.text
            ));
        }
    }
    if wants("journal") {
        for one in &task.log {
            plainly.push_str(&format!("\n\n({}) {}", one.at, kept_here(&one.body)));
        }
    }
    Ok(told(plainly, whole))
}

fn room_for_a_notice(body: &str) -> usize {
    let bom = body.len() - body.trim_start_matches('\u{feff}').len();
    let mut walk = bom;
    let mut started = false;
    for line in body[bom..].split_inclusive('\n') {
        let flat = line.trim();
        match (started, flat.is_empty()) {
            (false, true) => walk += line.len(),
            (false, false) if flat.starts_with('#') => return walk + line.len(),
            (false, false) => {
                started = true;
                walk += line.len();
            }
            (true, true) => return walk,
            (true, false) => walk += line.len(),
        }
    }
    body.len()
}

const WRAPPED: &str = " What came in reads as though it was wrapped to a fixed width. Tisty \
keeps one paragraph on one line, and every break inside one is kept as a break the person has to \
take out by hand. Send each paragraph on a single line.";

/// Three prose lines in a row, all about the width a formatter would pick, and none of them a
/// list, a heading, a table, a quote or code. Warned about rather than joined up: which breaks
/// were meant is not something this can know.
fn looks_wrapped(body: &str) -> bool {
    let mut run = 0usize;
    let mut fenced = false;
    for line in body.lines() {
        let bare = line.trim_start();
        if bare.starts_with("```") || bare.starts_with("~~~") {
            fenced = !fenced;
            run = 0;
            continue;
        }
        let prose = !fenced
            && !bare.is_empty()
            && !bare.starts_with(['#', '|', '>', '-', '*', '+'])
            && !line.starts_with("    ")
            && !bare.chars().next().is_some_and(|one| one.is_ascii_digit());
        run = match prose && (55..=90).contains(&line.chars().count()) {
            true => run + 1,
            false => 0,
        };
        if run >= 3 {
            return true;
        }
    }
    false
}

fn wrapped(body: &str) -> &'static str {
    match looks_wrapped(body) {
        true => WRAPPED,
        false => "",
    }
}

const AROUND: usize = 3;

/// Where two bodies first stop agreeing. An edit is named three different ways and lands in one
/// place, and this finds it without any of them having to say where.
fn first_apart(was: &str, whole: &str) -> usize {
    was.lines()
        .zip(whole.lines())
        .position(|(one, other)| one != other)
        .unwrap_or_else(|| was.lines().count().min(whole.lines().count()))
        + 1
}

/// What the document reads like around a change, so seeing it does not cost a whole `read_doc`.
fn echoed(args: &Value, was: &str, whole: &str) -> Option<Value> {
    if !args.get("echo").and_then(Value::as_bool).unwrap_or(false) {
        return None;
    }
    let at = first_apart(was, whole);
    let last = whole.lines().count().max(1);
    let from = at.saturating_sub(AROUND).max(1);
    let to = (at + AROUND).min(last);
    Some(json!({
        "from": from,
        "to": to,
        "body": tisty_core::docs::lines_between(whole, from, to),
    }))
}

/// Adds the echo to an answer only when one was asked for, so nothing else grows.
fn with_echo(mut kept: Value, args: &Value, was: &str, whole: &str) -> Value {
    if let Some(around) = echoed(args, was, whole) {
        kept["around"] = around;
    }
    kept
}

fn grew(was: &str, whole: &str) -> i64 {
    whole.chars().count() as i64 - was.chars().count() as i64
}

/// Said out loud on every write, so a splice that quietly doubled a document cannot read like a
/// small change to whoever asked for one.
fn by_how_much(was: &str, whole: &str) -> String {
    match grew(was, whole) {
        0 => "the same length".into(),
        by if by > 0 => format!("{by} characters longer"),
        by => format!("{} characters shorter", -by),
    }
}

/// `old` and `new` are matched byte for byte, so trimming them would be trimming the document.
fn raw<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

/// A refusal that says only "not there" sends the agent back to read the whole document.
fn nearest(body: &str, old: &str) -> Option<(usize, String)> {
    let first = old.lines().find(|one| !one.trim().is_empty())?.trim();
    if first.chars().count() < 4 {
        return None;
    }
    let mut most = None;
    for (n, line) in body.lines().enumerate() {
        let shared = line
            .trim()
            .chars()
            .zip(first.chars())
            .take_while(|(a, b)| a.eq_ignore_ascii_case(b))
            .count();
        if shared >= 4 && most.as_ref().is_none_or(|(_, _, was)| shared > *was) {
            most = Some((n + 1, line.to_string(), shared));
        }
    }
    most.map(|(line, said, _)| (line, said))
}

fn left_loose(state: &State, which: &str, was: &str, whole: &str) -> Vec<String> {
    let before = tisty_core::refs::papers(was);
    let after = tisty_core::refs::papers(whole);
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return Vec::new();
    };
    state
        .pages_of(kept.id)
        .into_iter()
        .map(|one| one.file.clone())
        .filter(|file| before.contains(file) && !after.contains(file))
        .collect()
}

fn named_twice(state: &State, which: &str, whole: &str) -> Vec<String> {
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return Vec::new();
    };
    let lines = tisty_core::refs::paper_lines(whole);
    state
        .pages_of(kept.id)
        .into_iter()
        .map(|one| one.file.clone())
        .filter(|file| lines.iter().filter(|(one, _)| one == file).count() > 1)
        .collect()
}

fn twice_words(state: &State, twice: &[String]) -> String {
    if twice.is_empty() {
        return String::new();
    }
    let (names, reads) = match twice.len() {
        1 => ("the line naming", "it is read where it is named first"),
        _ => ("the lines naming", "each is read where it is named first"),
    };
    format!(
        " Now {names} {} stands in more than one place, and {reads}, so the later one draws a \
         way in that leads nowhere new. Take it out with another `edit_doc`.",
        named_all(state, twice)
    )
}

fn loose_words(state: &State, loose: &[String]) -> String {
    if loose.is_empty() {
        return String::new();
    }
    let (took, still, put) = match loose.len() {
        1 => (
            "the line that named",
            "it is still a page",
            "its line where it belongs",
        ),
        _ => (
            "the lines that named",
            "they are still pages",
            "their lines where they belong",
        ),
    };
    format!(
        " It also took out {took} {}: nothing in this document points there now, though {still} \
         of it. If you are moving {}, write {put} with another `edit_doc`, or put {} in place \
         with `page_doc` and `after`. If the cut was a mistake, `restore_doc` puts the passage \
         back as it was.",
        named_all(state, loose),
        match loose.len() {
            1 => "it",
            _ => "them",
        },
        match loose.len() {
            1 => "it",
            _ => "each",
        }
    )
}

fn trail(state: &State, at: tisty_core::model::FolderId) -> String {
    let mut named = Vec::new();
    let mut walk = Some(at);
    while let Some(one) = walk {
        let Some(folder) = state.folders.get(&one) else {
            break;
        };
        named.push(folder.name.as_str());
        walk = folder.parent;
        if named.len() > tisty_core::model::DEEPEST {
            break;
        }
    }
    named.reverse();
    named.join(" / ")
}

fn as_path(said: &str) -> String {
    said.split('/')
        .map(str::trim)
        .filter(|one| !one.is_empty())
        .map(tisty_core::text::folded)
        .collect::<Vec<_>>()
        .join("/")
}

fn folder_named(state: &State, said: &str) -> Result<tisty_core::model::FolderId, Refused> {
    let found = folder_found(state, said)?;
    match state.folder_away(found) {
        true => Err(Refused::Tool(format!(
            "{said:?} is in the archive, so nothing new goes into it. The person brings it back from the window when it is meant to be used again."
        ))),
        false => Ok(found),
    }
}

fn folder_found(state: &State, said: &str) -> Result<tisty_core::model::FolderId, Refused> {
    if let Ok(id) = said.parse::<Ulid>()
        && state.folders.contains_key(&id)
    {
        return Ok(id);
    }
    let wanted = tisty_core::text::folded(said);
    let mut hit: Vec<&tisty_core::model::Folder> = state
        .folders
        .values()
        .filter(|one| tisty_core::text::folded(&one.name) == wanted)
        .collect();

    if hit.len() != 1 {
        let path = as_path(said);
        let walked: Vec<&tisty_core::model::Folder> = state
            .folders
            .values()
            .filter(|one| as_path(&trail(state, one.id)) == path)
            .collect();
        if walked.len() == 1 {
            hit = walked;
        }
    }

    match hit.as_slice() {
        [one] => Ok(one.id),
        [] => Err(Refused::Tool({
            let mut all: Vec<String> = state
                .folders
                .keys()
                .filter(|id| !state.folder_away(**id))
                .map(|id| trail(state, *id))
                .collect();
            all.sort();
            all.dedup();
            match all.is_empty() {
                true => format!(
                    "no folder here is called {said:?}, and there are none at all yet. `folder` \
                     makes one."
                ),
                false => format!(
                    "no folder here is called {said:?}. These exist, and each one answers to its \
                     own name or to the whole path: {}. `folder` makes a new one.",
                    all.join(", ")
                ),
            }
        })),
        many => Err(Refused::Tool(format!(
            "{said:?} is the name of {} folders. Send the whole path, or the id, of the one you \
             mean instead: {}.",
            many.len(),
            many.iter()
                .map(|one| format!("{} ({})", one.id, trail(state, one.id)))
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

fn papers(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;
    let scope = scoped(args)?;
    let within = match text(args, "folder") {
        Some(said) => Some(folder_found(&state, &said)?),
        None => match args.get("folder").is_some_and(|one| !one.is_null()) {
            true => {
                return Err(Refused::Tool(
                    "`folder` names one folder to list. Leave it out to list them all.".into(),
                ));
            }
            false => None,
        },
    };
    if args.get("page_of").is_some() && text(args, "page_of").is_none() {
        return Err(Refused::Tool(
            "`page_of` names one document whose pages to list. Leave it out to list them all."
                .into(),
        ));
    }
    let under = match text(args, "page_of") {
        Some(said) => Some(
            state
                .docs
                .values()
                .find(|one| one.file == said)
                .map(|one| one.id)
                .ok_or_else(|| {
                    Refused::Tool(format!(
                        "no document here is called {said:?}, so nothing hangs from it. `docs` \
                         lists them all."
                    ))
                })?,
        ),
        None => None,
    };
    let most = args
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(50)
        .clamp(1, LISTED_AT_MOST as u64) as usize;
    let past = args.get("after").and_then(Value::as_u64).unwrap_or(0) as usize;

    let reading = under.is_some();
    let mut kept: Vec<&tisty_core::model::Kept> = state
        .docs
        .values()
        .filter(|one| match scope {
            tisty_core::view::Scope::Open => !state.held_away(one),
            tisty_core::view::Scope::Archived => state.held_away(one),
            tisty_core::view::Scope::Either => true,
        })
        .filter(|one| within.is_none_or(|at| one.folder == Some(at) && one.page_of.is_none()))
        .filter(|one| under.is_none_or(|up| one.page_of == Some(up)))
        .collect();
    match reading {
        // Pages answer in the order they are read, which is the order they are named in.
        true => kept.sort_by(|a, b| a.order.cmp(&b.order).then(a.id.cmp(&b.id))),
        // What moved last, not what was made last: an agent coming back asks what has changed.
        false => kept.sort_by_key(|one| std::cmp::Reverse((one.wrote, one.id))),
    }

    let all = kept.len();
    let shown_of: Vec<&tisty_core::model::Kept> =
        kept.iter().skip(past).take(most).copied().collect();
    let held = tisty_core::cache::Cache::open(paths.cache()).ok().flatten();
    // Listing them is the one moment that already knows which files are really there.
    tisty_core::docs::forget_stray_cards(&paths.docs(), held.as_ref());
    let cards = tisty_core::docs::cards_of(
        &paths.docs(),
        held.as_ref(),
        &shown_of
            .iter()
            .map(|one| one.file.clone())
            .collect::<Vec<_>>(),
    );

    let shown: Vec<Value> = shown_of
        .iter()
        .map(|one| {
            let card = cards.get(&one.file);
            let mut kept_of = serde_json::Map::new();
            kept_of.insert("doc".into(), json!(one.file));
            kept_of.insert(
                "title".into(),
                json!(card.map(|one| one.title.clone()).unwrap_or_default()),
            );
            if let Some(at) = one.folder {
                kept_of.insert("folder".into(), json!(trail(&state, at)));
            }
            if let Some(up) = one.page_of.and_then(|up| named_doc(&state, up)) {
                kept_of.insert("page_of".into(), json!(up));
            }
            let pages = state.pages_of(one.id);
            if !pages.is_empty() {
                kept_of.insert("pages".into(), json!(pages.len()));
                let away = pages.iter().filter(|page| page.archived).count();
                if away > 0 {
                    kept_of.insert("pages_archived".into(), json!(away));
                }
                let marked = pages
                    .iter()
                    .filter(|page| page.flagged.is_some() && !state.held_away(page))
                    .count();
                if marked > 0 {
                    kept_of.insert("pages_flagged".into(), json!(marked));
                }
            }
            if state.held_away(one) {
                kept_of.insert("archived".into(), json!(true));
            }
            if one.archived {
                kept_of.insert("apart".into(), json!(true));
            }
            if state.shut(one.id) {
                kept_of.insert("locked".into(), json!(true));
            }
            if one.flagged.is_some() && !state.held_away(one) {
                kept_of.insert("flagged".into(), json!(true));
            }
            if let Some(card) = card {
                kept_of.insert("words".into(), json!(card.words));
                if !card.outline.is_empty() {
                    kept_of.insert("sections".into(), json!(card.outline.len()));
                }
                if !card.keywords.is_empty() {
                    kept_of.insert("about".into(), json!(card.keywords));
                }
                if card.pictures > 0 {
                    kept_of.insert("pictures".into(), json!(card.pictures));
                }
            }
            if let Some(at) = one.wrote {
                kept_of.insert("wrote".into(), json!(at.to_string()));
            }
            if let Some(card) = card
                && let Some(said) = gist_of(held.as_ref(), &one.file, &card.print)
            {
                kept_of.insert("gist".into(), json!(shortened(said)));
            }
            Value::Object(kept_of)
        })
        .collect();

    let mut folders: Vec<Value> = state
        .folders
        .values()
        .map(|one| {
            json!({
                "folder": one.name,
                "id": one.id.to_string(),
                "path": trail(&state, one.id),
                "icon": one.icon,
                "docs": state
                    .docs
                    .values()
                    .filter(|kept| kept.folder == Some(one.id) && kept.page_of.is_none())
                    .count(),
            })
        })
        .collect();
    folders.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));

    let mut lines: Vec<String> = shown
        .iter()
        .map(|one| {
            let where_at = match one["page_of"].as_str() {
                Some(up) => format!("page of {up}"),
                None => one["folder"].as_str().unwrap_or("no folder").to_string(),
            };
            let holds = match one["pages"].as_u64().unwrap_or(0) {
                0 => String::new(),
                1 => ", 1 page".into(),
                many => format!(", {many} pages"),
            };
            let put_away = if one["archived"] == json!(true) {
                ", put away"
            } else {
                ""
            };
            let shut = if one["locked"] == json!(true) {
                ", locked"
            } else {
                ""
            };
            let marked = if one["flagged"] == json!(true) {
                ", marked as one that has had its day"
            } else {
                ""
            };
            format!(
                "{} — {} ({where_at}{holds}{put_away}{shut}{marked})",
                said(one, "doc"),
                said(one, "title")
            )
        })
        .collect();
    lines.push(match folders.is_empty() {
        true => "No folders here yet.".into(),
        false => format!(
            "Folders: {}.",
            folders
                .iter()
                .filter_map(|one| one["path"].as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    });

    let mut kept = json!({ "docs": shown, "total": all });
    if args
        .get("folders")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        kept["folders"] = json!(folders);
    }

    Ok(told(
        format!(
            "{all} document(s); showing {}.\n{}",
            shown.len(),
            lines.join("\n")
        ),
        kept,
    ))
}

#[derive(Default)]
struct Carried {
    kept: usize,
    missed: Vec<String>,
    papers: Vec<String>,
}

fn unescaped(target: &str) -> String {
    let bytes = target.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'%'
            && at + 2 < bytes.len()
            && let Ok(one) = u8::from_str_radix(&target[at + 1..at + 3], 16)
        {
            out.push(one);
            at += 3;
            continue;
        }
        out.push(bytes[at]);
        at += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| target.to_string())
}

fn opening(body: &str, at: usize) -> Option<usize> {
    let bytes = body.as_bytes();
    let line = body[..at].rfind('\n').map(|one| one + 1).unwrap_or(0);
    let mut deep = 0;
    let mut walk = at;
    while walk > line {
        walk -= 1;
        match bytes[walk] {
            b']' => deep += 1,
            b'[' if deep == 0 => return Some(walk),
            b'[' => deep -= 1,
            _ => {}
        }
    }
    None
}

fn written_as_code(body: &str) -> Vec<bool> {
    let mut out = vec![false; body.len()];
    let mut fence: Option<String> = None;
    let mut at = 0;
    for line in body.split_inclusive('\n') {
        let bare = line.trim_end_matches('\n');
        let opens = bare
            .trim_start()
            .chars()
            .take_while(|one| *one == '`' || *one == '~')
            .count();
        let mark = bare.trim_start().chars().next().unwrap_or(' ');
        match &fence {
            Some(open) => {
                for one in out.iter_mut().skip(at).take(line.len()) {
                    *one = true;
                }
                if opens >= open.len() && bare.trim_start().starts_with(open.as_str()) {
                    fence = None;
                }
            }
            None if opens >= 3 => {
                fence = Some(mark.to_string().repeat(opens));
                for one in out.iter_mut().skip(at).take(line.len()) {
                    *one = true;
                }
            }
            None => {
                let mut walk = 0;
                for (span, part) in tisty_core::arriving::spans(bare) {
                    if span {
                        for one in out.iter_mut().skip(at + walk).take(part.len()) {
                            *one = true;
                        }
                    }
                    walk += part.len();
                }
            }
        }
        at += line.len();
    }
    out
}

fn retargeted(
    body: &str,
    with: &mut impl FnMut(&str, &str, &str) -> Option<(String, String)>,
) -> String {
    let coded = written_as_code(body);
    let bytes = body.as_bytes();
    let mut out = String::with_capacity(body.len());
    let mut at = 0;
    let mut from = 0;
    while at < bytes.len() {
        if bytes[at] != b']' || bytes.get(at + 1) != Some(&b'(') || coded[at] {
            at += 1;
            continue;
        }
        let Some(open) = opening(body, at) else {
            at += 1;
            continue;
        };
        let mut walk = at + 2;
        let mut deep = 1;
        while walk < bytes.len() && deep > 0 {
            match bytes[walk] {
                b'(' => deep += 1,
                b')' => deep -= 1,
                _ => {}
            }
            walk += 1;
        }
        if deep > 0 {
            at += 1;
            continue;
        }
        let inner = &body[at + 2..walk - 1];
        let (target, title) = split_target(inner);
        let label = &body[open + 1..at];
        let pictured = open > 0 && bytes[open - 1] == b'!';
        let cut = match pictured {
            true => open - 1,
            false => open,
        };
        out.push_str(&body[from..cut]);
        match with(label, &target, &title) {
            Some((now, title)) => {
                let shown = match title.is_empty() {
                    true => format!("<{now}>"),
                    false => format!("<{now}> {title}"),
                };
                let mark = if pictured { "!" } else { "" };
                out.push_str(&format!("{mark}[{label}]({shown})"));
            }
            None => out.push_str(label),
        }
        from = walk;
        at = walk;
    }
    out.push_str(&body[from..]);
    out
}

fn split_target(inner: &str) -> (String, String) {
    let said = inner.trim();
    if let Some(rest) = said.strip_prefix('<')
        && let Some(shut) = rest.find('>')
    {
        return (
            rest[..shut].to_string(),
            rest[shut + 1..].trim().to_string(),
        );
    }
    match said.find(char::is_whitespace) {
        Some(gap) => {
            let rest = said[gap..].trim();
            match rest.starts_with(['"', '\'', '(']) {
                true => (said[..gap].to_string(), rest.to_string()),
                false => (said.to_string(), String::new()),
            }
        }
        None => (said.to_string(), String::new()),
    }
}

fn line_of(lines: &[String], id: &str) -> Option<usize> {
    tisty_core::refs::paper_lines(&lines.join("\n"))
        .into_iter()
        .find(|(one, _)| one == id)
        .map(|(_, at)| at)
}

fn card_alone(line: &str, id: &str) -> bool {
    card_any(line) && tisty_core::refs::papers(line.trim()) == vec![id.to_string()]
}

fn blank(lines: &[String], at: usize) -> bool {
    lines.get(at).is_none_or(|one| one.trim().is_empty())
}

enum Held {
    After(String),
    Before(String),
    At(Spot<'static>),
}

impl Held {
    fn spot(&self) -> Spot<'_> {
        match self {
            Held::After(one) => Spot::After(one),
            Held::Before(one) => Spot::Before(one),
            Held::At(one) => *one,
        }
    }
}

#[derive(Clone, Copy)]
enum Spot<'a> {
    After(&'a str),
    Before(&'a str),
    First,
    Last,
}

impl Spot<'_> {
    fn anchor(&self) -> Option<&str> {
        match self {
            Spot::After(one) | Spot::Before(one) => Some(one),
            Spot::First | Spot::Last => None,
        }
    }

    fn said(&self) -> &'static str {
        match self {
            Spot::After(_) => "after",
            Spot::Before(_) => "before",
            Spot::First => "first",
            Spot::Last => "last",
        }
    }
}

fn cards_in(lines: &[String]) -> Vec<usize> {
    let mut at: Vec<usize> = tisty_core::refs::paper_lines(&lines.join("\n"))
        .into_iter()
        .map(|(_, at)| at)
        .collect();
    at.dedup();
    at
}

fn card_any(line: &str) -> bool {
    let said = line.trim();
    said.starts_with("![")
        && said.ends_with(')')
        && tisty_core::refs::extract(said).len() == 1
        && tisty_core::refs::papers(said).len() == 1
}

fn card_moved(body: &str, which: &str, title: &str, spot: Spot) -> Option<String> {
    let ending = match body.contains("\r\n") {
        true => "\r\n",
        false => "\n",
    };
    let mut lines: Vec<String> = body.lines().map(str::to_string).collect();
    if let Some(at) = line_of(&lines, which) {
        lines.remove(at);
        if at > 0 && at < lines.len() && blank(&lines, at) && blank(&lines, at - 1) {
            lines.remove(at);
        }
    }
    let at = match spot {
        Spot::After(anchor) => line_of(&lines, anchor)? + 1,
        Spot::Before(anchor) => line_of(&lines, anchor)?,
        Spot::First => cards_in(&lines).first().copied().unwrap_or(lines.len()),
        Spot::Last => match cards_in(&lines).last() {
            Some(one) => one + 1,
            None => {
                while lines.last().is_some_and(|one| one.trim().is_empty()) {
                    lines.pop();
                }
                lines.len()
            }
        },
    };
    lines.insert(at, tisty_core::refs::card(which, title));
    if !blank(&lines, at + 1) {
        lines.insert(at + 1, String::new());
    }
    if at > 0 && !blank(&lines, at - 1) {
        lines.insert(at, String::new());
    }
    let mut out = lines.join(ending);
    if !out.ends_with('\n') {
        out.push_str(ending);
    }
    Some(out)
}

fn carried_off(state: &State, parent: &str, which: &str, at: usize, line: &str) -> Refused {
    Refused::Tool(format!(
        "line {} of {} names {} and says other things besides, so moving that line would carry \
         them off with it: {:?}. Nothing was moved. Put the line on its own with `edit_doc` \
         first, or move it there yourself.",
        at + 1,
        doc_named(state, parent),
        doc_named(state, which),
        line.trim()
    ))
}

fn named_doc(state: &State, id: tisty_core::model::DocId) -> Option<String> {
    state.docs.get(&id).map(|one| one.file.clone())
}

fn named_all(state: &State, files: &[String]) -> String {
    files
        .iter()
        .map(|one| doc_named(state, one))
        .collect::<Vec<_>>()
        .join(", ")
}

fn doc_named(state: &State, which: &str) -> String {
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return which.to_string();
    };
    match kept
        .title
        .as_deref()
        .map(str::trim)
        .filter(|one| !one.is_empty())
    {
        Some(title) => format!("{title:?} ({which})"),
        None => which.to_string(),
    }
}

fn up_named(state: &State, id: tisty_core::model::DocId) -> Option<String> {
    let which = named_doc(state, id)?;
    Some(doc_named(state, &which))
}

const NEWEST_SHOWN: usize = 8;

/// Every op carries its own id under the same name, so reading it back as JSON keeps this from
/// having to know each one — and from going quiet the day another is added.
fn what_it_touched(event: &tisty_core::event::Event) -> Option<(String, String)> {
    let said = serde_json::to_value(&event.op).ok()?;
    let named = said.get("op")?.as_str()?.to_string();
    let id = said.get("id")?.as_str()?.to_string();
    Some((named, id))
}

fn catch_up(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (state, store) = opened(paths)?;

    let mut named: Vec<&str> = state
        .lists
        .values()
        .filter(|one| !one.archived)
        .map(|one| one.name.as_str())
        .collect();
    named.sort_unstable();

    let mut folders: Vec<String> = state
        .folders
        .keys()
        .filter(|id| !state.folder_away(**id))
        .map(|id| trail(&state, *id))
        .collect();
    folders.sort();

    let mut how: std::collections::BTreeMap<&str, usize> = Default::default();
    for one in state
        .tasks
        .values()
        .flat_map(|task| &task.tags)
        .chain(state.docs.values().flat_map(|one| &one.tags))
    {
        *how.entry(one.as_str()).or_default() += 1;
    }
    let mut tags: Vec<(&str, usize)> = how.into_iter().collect();
    tags.sort_by(|(one, mine), (other, theirs)| theirs.cmp(mine).then(one.cmp(other)));

    let log = store.read_all().map_err(hitch)?;
    let head = log
        .iter()
        .map(|one| one.timestamp)
        .max()
        .map(|one| one.to_string());

    let mut kept = serde_json::Map::new();
    kept.insert("lists".into(), json!(named));
    if !folders.is_empty() {
        kept.insert("folders".into(), json!(folders));
    }
    if !tags.is_empty() {
        kept.insert(
            "tags".into(),
            json!(
                tags.iter()
                    .take(TAGS_SHOWN)
                    .map(|(one, times)| json!({ "tag": one, "times": times }))
                    .collect::<Vec<_>>()
            ),
        );
    }
    kept.insert(
        "counts".into(),
        json!({
            "open": state.tasks.values().filter(|one| one.is_open() && !one.folded()).count(),
            "docs": state.docs.values().filter(|one| !state.held_away(one)).count(),
        }),
    );
    if let Some(head) = &head {
        kept.insert("cursor".into(), json!(head));
    }

    let since = text(args, "since");
    let said = match &since {
        None => {
            let mut newest: Vec<&tisty_core::model::Kept> = state
                .docs
                .values()
                .filter(|one| !state.held_away(one))
                .collect();
            newest.sort_by_key(|one| std::cmp::Reverse((one.wrote, one.id)));
            let newest: Vec<String> = newest
                .iter()
                .take(NEWEST_SHOWN)
                .map(|one| one.file.clone())
                .collect();
            if !newest.is_empty() {
                kept.insert("newest".into(), json!(newest));
            }
            format!(
                "{} list(s), {} folder(s), {} tag(s), {} open task(s), {} document(s). Send the \
                 `cursor` back next time and what moved since comes with it as well.",
                named.len(),
                folders.len(),
                tags.len(),
                state
                    .tasks
                    .values()
                    .filter(|one| one.is_open() && !one.folded())
                    .count(),
                state
                    .docs
                    .values()
                    .filter(|one| !state.held_away(one))
                    .count()
            )
        }
        Some(since) => {
            let since: jiff::Timestamp = since.parse().map_err(|_| {
                Refused::Tool(format!(
                    "`since` has to be a `cursor` a previous `catch_up` handed back, not {since:?}."
                ))
            })?;
            let mut tasks: Vec<String> = Vec::new();
            let mut papers: Vec<String> = Vec::new();
            // The log refuses two events at the very same instant, so what the cursor named is
            // behind us and nothing is skipped by leaving it out.
            for one in log.iter().filter(|one| one.timestamp > since) {
                let Some((named, id)) = what_it_touched(one) else {
                    continue;
                };
                if named.starts_with("task.") && !tasks.contains(&id) {
                    tasks.push(id);
                } else if named.starts_with("doc.") && !papers.contains(&id) {
                    papers.push(id);
                }
            }
            let moved: Vec<Value> = tasks
                .iter()
                .filter_map(|id| id.parse::<TaskId>().ok())
                .filter_map(|id| state.tasks.get(&id))
                .filter(|one| !one.folded())
                .map(|one| brief(one, &state))
                .collect();
            let written: Vec<Value> = papers
                .iter()
                .filter_map(|id| {
                    let id = id.parse::<Ulid>().ok()?;
                    let kept = state.docs.values().find(|one| one.id == id)?;
                    Some(json!({ "doc": kept.file, "title": kept.title }))
                })
                .collect();
            let said = format!(
                "{} task(s) and {} document(s) moved since then.",
                moved.len(),
                written.len()
            );
            if !moved.is_empty() {
                kept.insert("tasks".into(), json!(moved));
            }
            if !written.is_empty() {
                kept.insert("docs".into(), json!(written));
            }
            said
        }
    };
    Ok(told(said, Value::Object(kept)))
}

fn lists(paths: &Paths) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;
    let mut named: Vec<&str> = state
        .lists
        .values()
        .filter(|one| !one.archived)
        .map(|one| one.name.as_str())
        .collect();
    named.sort_unstable();

    let text = if named.is_empty() {
        "No lists here yet. What you propose lands in the inbox.".to_string()
    } else {
        format!("{}; anything else lands in the inbox.", named.join(", "))
    };
    Ok(told(text, json!({ "lists": named })))
}

const TAGS_SHOWN: usize = 300;

fn tags(paths: &Paths) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;
    let mut how: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for one in state
        .tasks
        .values()
        .flat_map(|task| &task.tags)
        .chain(state.docs.values().flat_map(|one| &one.tags))
    {
        *how.entry(one.as_str()).or_default() += 1;
    }
    let mut named: Vec<(&str, usize)> = how.into_iter().collect();
    named.sort_by(|(one, mine), (other, theirs)| theirs.cmp(mine).then(one.cmp(other)));

    let text = match named.len() {
        0 => "No tags here yet. Write the word the person would use, not one of your own.".into(),
        many => {
            let shown = named
                .iter()
                .take(TAGS_SHOWN)
                .map(|(one, times)| format!("#{one} ({times})"))
                .collect::<Vec<_>>()
                .join(", ");
            match many > TAGS_SHOWN {
                true => format!(
                    "{shown}, and {} more that came back with this answer.",
                    many - TAGS_SHOWN
                ),
                false => shown,
            }
        }
    };
    Ok(told(
        text,
        json!({
            "tags": named
                .iter()
                .map(|(one, times)| json!({ "tag": one, "times": times }))
                .collect::<Vec<_>>()
        }),
    ))
}

/// A summary is the one thing about a document that cannot be worked out from it, so an agent
/// that has read one leaves what it learnt here rather than making the next one read it again.
fn sum_up(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(which) = text(args, "doc") else {
        return Err(Refused::Tool(
            "saying what a document is about needs its `doc` name.".into(),
        ));
    };
    let summary = text(args, "summary").unwrap_or_default();
    let notes = text(args, "notes").unwrap_or_default();
    if summary.is_empty() && notes.is_empty() {
        return Err(Refused::Tool(
            "this takes a `summary` of what the document says, `notes` for what the next agent \
             should know about it, or both."
                .into(),
        ));
    }
    if summary.chars().count() > tisty_core::docs::SUMMARY_AT_MOST {
        return Err(Refused::Tool(format!(
            "a summary past {} characters is not a summary. Say what the document is for and \
             what a reader would come to it for.",
            tisty_core::docs::SUMMARY_AT_MOST
        )));
    }
    if notes.chars().count() > tisty_core::docs::NOTES_AT_MOST {
        return Err(Refused::Tool(format!(
            "`notes` runs past {} characters. What does not fit belongs in the document itself, \
             where the person can see it.",
            tisty_core::docs::NOTES_AT_MOST
        )));
    }

    let (state, store) = opened(paths)?;
    if state.docs.values().all(|one| one.file != which) {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        )));
    }
    let Some(cache) = tisty_core::cache::Cache::open(paths.cache()).ok().flatten() else {
        return Err(Refused::Tool(
            "this machine has nowhere to keep what you read, so nothing was written. It is not \
             worth another try: read the document when you need it."
                .into(),
        ));
    };
    let Some(card) = tisty_core::docs::card_of(&paths.docs(), Some(&cache), &which) else {
        return Err(Refused::Tool(format!(
            "{which:?} is named in the log but its text is not on this machine yet."
        )));
    };

    let held = tisty_core::docs::Gist {
        print: card.print.clone(),
        summary: summary.clone(),
        notes: notes.clone(),
        at: jiff::Timestamp::now(),
        by: Some(store.device().0.clone()),
    };
    if !cache.note_gist(&which, &held) {
        return Err(Refused::Tool(
            "what you read could not be kept on this machine, so nothing was written.".into(),
        ));
    }

    Ok(told(
        format!(
            "Kept what {:?} is about, against the text as it reads now. It stays on this \
             machine, and the next reader is told if the document has moved since.",
            card.title
        ),
        json!({
            "doc": which,
            "title": card.title,
            "print": card.print,
            "kept": true,
        }),
    ))
}

const A_SUMMARY_IN_A_LIST: usize = 300;

/// In a list of fifty, the summary is there to choose by, not to read.
fn shortened(said: Value) -> Value {
    let Value::Object(mut kept) = said else {
        return said;
    };
    kept.remove("notes");
    if let Some(summary) = kept.get("summary").and_then(Value::as_str)
        && summary.chars().count() > A_SUMMARY_IN_A_LIST
    {
        let cut: String = summary.chars().take(A_SUMMARY_IN_A_LIST).collect();
        kept.insert("summary".into(), json!(format!("{cut}…")));
        kept.insert("more".into(), json!(true));
    }
    Value::Object(kept)
}

/// What was written about a document, and whether it still describes the text that is there.
fn gist_of(cache: Option<&tisty_core::cache::Cache>, which: &str, print: &str) -> Option<Value> {
    let held = cache?.gist(which)?;
    let mut kept = serde_json::Map::new();
    if !held.summary.is_empty() {
        kept.insert("summary".into(), json!(held.summary));
    }
    if !held.notes.is_empty() {
        kept.insert("notes".into(), json!(held.notes));
    }
    kept.insert("said_at".into(), json!(held.at.to_string()));
    kept.insert("by_agent".into(), json!(true));
    if held.print != print {
        kept.insert("stale".into(), json!(true));
    }
    Some(Value::Object(kept))
}

/// Long enough that reading it whole is a decision, not an accident.
const WHOLE_UP_TO: usize = 12_000;

enum Part {
    Outline,
    Held {
        body: String,
        from: usize,
        to: usize,
        next: Option<usize>,
    },
}

/// The last line that fits in a budget of characters, counting from `start`.
fn as_far_as(body: &str, start: usize, most: usize, last: usize) -> usize {
    let mut room = 0usize;
    let mut at = start;
    for line in body.lines().skip(start.saturating_sub(1)) {
        let next = room + line.chars().count() + 1;
        if next > most && at > start {
            break;
        }
        room = next;
        at += 1;
    }
    (at - 1).max(start).min(last)
}

fn part_asked(body: &str, args: &Value) -> Result<Part, Refused> {
    let last = body.lines().count().max(1);
    let held = |from: usize, to: usize| Part::Held {
        body: tisty_core::docs::lines_between(body, from, to),
        from,
        to,
        next: None,
    };

    if let Some(at) = args.get("section").and_then(Value::as_u64) {
        let Some((from, to)) = tisty_core::docs::section_lines(body, at as usize) else {
            return Err(Refused::Tool(format!(
                "this document has no section {at}. `outline_doc` numbers them from 0."
            )));
        };
        let fits = as_far_as(body, from, WHOLE_UP_TO, last).min(to);
        return Ok(Part::Held {
            body: tisty_core::docs::lines_between(body, from, fits),
            from,
            to: fits,
            next: (fits < to).then_some(fits + 1),
        });
    }

    let from = args
        .get("from")
        .and_then(Value::as_u64)
        .map(|one| one as usize);
    let to = args
        .get("to")
        .and_then(Value::as_u64)
        .map(|one| one as usize);
    if from.is_some() || to.is_some() {
        let from = from.unwrap_or(1).max(1);
        let to = to.unwrap_or(last).min(last);
        if from > to {
            return Err(Refused::Tool(format!(
                "`from` is line {from} and `to` is line {to}, so there is nothing between them."
            )));
        }
        // Naming a run wide enough to cover the document was the way round the budget every other
        // way of reading one is held to.
        let fits = as_far_as(body, from, WHOLE_UP_TO, last).min(to);
        return Ok(Part::Held {
            body: tisty_core::docs::lines_between(body, from, fits),
            from,
            to: fits,
            next: (fits < to).then_some(fits + 1),
        });
    }

    if let Some(most) = args
        .get("chars")
        .and_then(Value::as_u64)
        .map(|one| one as usize)
    {
        let start = args
            .get("cursor")
            .and_then(Value::as_u64)
            .map(|one| one as usize)
            .unwrap_or(1)
            .max(1);
        if start > 1 {
            let now = tisty_core::attach::printed(body.as_bytes());
            match text(args, "print") {
                Some(then) if then == now => {}
                Some(_) => {
                    return Err(Refused::Tool(format!(
                        "this document was written since you read the part before it, so carrying \
                         on from line {start} would join two different versions. Read it again \
                         from the top, or ask `outline_doc` what is in it now."
                    )));
                }
                None => {
                    return Err(Refused::Tool(
                        "carrying on from a `cursor` needs the `print` the part before it came \
                         with, so that the two halves are known to be the same document."
                            .into(),
                    ));
                }
            }
        }
        let to = as_far_as(body, start, most, last);
        return Ok(Part::Held {
            body: tisty_core::docs::lines_between(body, start, to),
            from: start,
            to,
            next: (to < last).then_some(to + 1),
        });
    }

    match body.chars().count() > WHOLE_UP_TO {
        true => Ok(Part::Outline),
        false => Ok(held(1, last)),
    }
}

/// Attaching records where a file came from, and those paths are the person's disk. The agent
/// needs the card, not the shape of their home directory.
fn kept_here(body: &str) -> String {
    body.lines()
        .map(|line| match absolute(line) {
            Some(at) => format!("{}…", &line[..at]),
            None => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join(
            "
",
        )
}

fn absolute(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    (0..bytes.len()).find(|&at| {
        // A drive is one letter: without this, the "s" of "https://" reads as one and the rest
        // of the line goes with it.
        let alone = at == 0 || !bytes[at - 1].is_ascii_alphanumeric();
        let drive = alone
            && at + 2 < bytes.len()
            && bytes[at].is_ascii_alphabetic()
            && bytes[at + 1] == b':'
            && (bytes[at + 2] == b'/' || bytes[at + 2] == b'\\');
        let rooted = bytes[at] == b'/' && at > 0 && bytes[at - 1] == b' ';
        drive || rooted
    })
}

/// Documents are searched by the same engine the window uses; a document the agent cannot find
/// again is a document it wrote into the dark.
fn papers_matching(
    paths: &Paths,
    state: &State,
    query: &str,
    scope: tisty_core::view::Scope,
    most: usize,
) -> Vec<Value> {
    let here: std::collections::HashMap<String, (bool, Option<String>, bool)> = state
        .docs
        .values()
        .filter(|one| match scope {
            tisty_core::view::Scope::Open => !state.held_away(one),
            tisty_core::view::Scope::Archived => state.held_away(one),
            tisty_core::view::Scope::Either => true,
        })
        .map(|one| {
            (
                one.file.clone(),
                (
                    state.held_away(one),
                    one.page_of.and_then(|up| named_doc(state, up)),
                    one.flagged.is_some() && !state.held_away(one),
                ),
            )
        })
        .collect();

    let held = tisty_core::cache::Cache::open(paths.cache()).ok().flatten();
    tisty_core::docs::sighted(&paths.docs(), held.as_ref(), query, most, |id| {
        here.contains_key(id)
    })
    .unwrap_or_else(|| {
        tisty_core::docs::Corpus::default()
            .searching(&paths.docs(), query, most, |id| here.contains_key(id))
    })
    .into_iter()
    .map(|one| {
        let (archived, page_of, flagged) =
            here.get(&one.id).cloned().unwrap_or((false, None, false));
        json!({
            "doc": one.id,
            "title": one.title,
            "line": one.line,
            "page_of": page_of,
            "archived": archived,
            "flagged": flagged,
        })
    })
    .collect()
}

/// «sereno#1», «sereno: #1» and «Sereno #1» name the same message, and a second filing of one
/// is a duplicate however it was written.
fn alike(source: &str) -> String {
    let one = source.trim().to_lowercase();
    let mut out = String::with_capacity(one.len());
    let mut space = false;
    for c in one.chars() {
        match c.is_whitespace() {
            true => space = !out.is_empty(),
            false => {
                if c == '#' {
                    while out.ends_with(':') || out.ends_with(' ') {
                        out.pop();
                    }
                } else if space {
                    out.push(' ');
                }
                space = false;
                out.push(c);
            }
        }
    }
    out
}

fn already(state: &State, source: &str) -> Option<TaskId> {
    let want = alike(source);
    state
        .sourced
        .iter()
        .find(|(one, _)| alike(one) == want)
        .map(|(_, id)| *id)
}

/// A field that says nothing still costs the reader a line, so it is left out.
fn brief(task: &Task, state: &State) -> Value {
    let mut kept = serde_json::Map::new();
    let mut put = |key: &str, one: Value| {
        let empty = one.is_null()
            || one.as_str() == Some("")
            || one.as_array().is_some_and(|all| all.is_empty());
        if !empty {
            kept.insert(key.into(), one);
        }
    };
    put("id", json!(task.id.to_string()));
    put("title", json!(task.title));
    put("status", json!(task.status));
    put(
        "closed",
        json!(task.completed_at.filter(|_| !task.is_open()).map(when)),
    );
    put(
        "date",
        json!(task.date.as_ref().map(|d| d.date().to_string())),
    );
    put(
        "deadline",
        json!(task.deadline.as_ref().map(|d| d.date().to_string())),
    );
    put(
        "reminders",
        json!(
            task.reminders
                .iter()
                .map(|one| one.at.to_string())
                .collect::<Vec<_>>()
        ),
    );
    put(
        "tags",
        json!(task.tags.iter().map(Tag::as_str).collect::<Vec<_>>()),
    );
    put("source", json!(task.source));
    // Where it ended up and how the person ranked it: reading them is how an agent sees a
    // decision it cannot make itself.
    put(
        "list",
        json!(
            task.list
                .as_ref()
                .and_then(|id| state.lists.get(id))
                .map(|one| &one.name)
        ),
    );
    put(
        "priority",
        json!((task.priority != Priority::Unset).then_some(task.priority)),
    );
    let by_agent = task
        .created_by
        .as_ref()
        .is_some_and(|who| state.assistants.contains(who));
    put("by_agent", json!(by_agent));
    put(
        "via",
        json!(
            task.created_via
                .as_deref()
                .filter(|_| by_agent)
                .map(tisty_core::agent::client_named)
        ),
    );
    put(
        "said_done",
        json!(task.resolved.as_ref().map(|one| when(one.at))),
    );
    put("open_to_agents", json!(task.open_to_agents.then_some(true)));
    Value::Object(kept)
}

/// A refusal that sends the agent to another call for a handful of names costs it a whole turn.
fn with_the_names(said: Refused, state: &State) -> Refused {
    let Refused::Tool(said) = said else {
        return said;
    };
    if !said.contains("list") {
        return Refused::Tool(said);
    }
    let mut all: Vec<&str> = state
        .lists
        .values()
        .filter(|one| !one.archived)
        .map(|one| one.name.as_str())
        .collect();
    all.sort_unstable();
    Refused::Tool(match all.is_empty() {
        true => format!("{said} There are no lists at all yet."),
        false => format!("{said} These exist: {}.", all.join(", ")),
    })
}

fn refused(e: Rejected) -> Refused {
    match e {
        Rejected::Untitled => Refused::Tool("a task needs a title.".into()),
        Rejected::NoSuchList(said) => Refused::Tool(format!(
            "there is no list called {said:?}, and you cannot make one. Leave `list` out and it \
             lands in the inbox."
        )),
        Rejected::AmbiguousList(said) => Refused::Tool(format!(
            "{said:?} matches more than one list. Send the whole name."
        )),

        Rejected::ArchivedList(said) => Refused::Tool(format!(
            "the list {said:?} is put away, so nothing new goes in it. Leave `list` out and it \
             lands in the inbox."
        )),
        other => Refused::Protocol(-32603, format!("{other:?}")),
    }
}

#[cfg(test)]
#[path = "mcp_test.rs"]
mod tests;
