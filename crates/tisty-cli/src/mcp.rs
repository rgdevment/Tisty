mod asked;
mod attaching;
mod catalogue;
mod chores;
mod door;
mod jsonrpc;
mod looking;
mod papers;

use papers::editing::edit_doc;
use papers::filing::{archive_doc, file_doc, flag_doc, folder};
use papers::importing::import_doc;
use papers::paging::page_doc;
use papers::reading::{export_doc, outline_doc, read_doc};
use papers::writing::{append_doc, restore_doc, write_doc};

use asked::{only_what_it_takes, short_and_plain, text};

use attaching::attach;
use catalogue::{instructions, tools};
use chores::{describe, note, plan, propose, remind, reschedule, say_done, tick};
pub(crate) use door::turn;
use looking::{NEWEST_SHOWN, catch_up, find, gone, how_it_ended, lists, read, sum_up, tags};

use jsonrpc::{
    discovered, fault, introduced, legacy_greeting, named_tool, reply, speaking_through, told,
    wrong,
};

use std::io::{BufRead, Write};
use std::process::ExitCode;

use serde_json::{Value, json};
use tisty_core::{
    Op, Paths, State, Store, Task, TaskId,
    model::FOLDER_NAME_AT_MOST,
    order,
    witness::{self, Fact},
};
use ulid::Ulid;

const VERSIONS: [&str; 3] = ["2026-07-28", "2025-11-25", "2025-06-18"];
const TOOLS_STAY_FRESH: i64 = 3_600_000;
const INBOX_TAG: &str = tisty_core::model::AGENT_TAG;
const LISTED_AT_MOST: usize = 200;

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

fn moved(task: &Task) -> Refused {
    Refused::Tool(format!(
        "{:?} moved while you were writing — the person, or another agent. `read` it again \
         before saying anything.",
        task.title
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

fn named(status: tisty_core::model::Status) -> &'static str {
    match status {
        tisty_core::model::Status::Open => "open",
        tisty_core::model::Status::Done => "done",
        tisty_core::model::Status::Dropped => "dropped",
    }
}

fn ended(task: &Task) -> String {
    task.completed_at
        .map(|at| format!(" on {}", when(at)))
        .unwrap_or_default()
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
    told: Vec<tisty_core::Op>,
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

#[cfg(test)]
#[path = "mcp_test.rs"]
mod tests;
