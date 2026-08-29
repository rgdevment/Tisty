use std::io::{BufRead, Write};
use std::process::ExitCode;

use serde_json::{Value, json};
use tisty_core::{
    Op, Paths, State, Store, Task, TaskId,
    capture::{Draft, Rejected},
    event::{Body, LogAdd, StepAdd},
    model::{DateSpec, Priority, Tag},
    order,
};
use ulid::Ulid;

const VERSIONS: [&str; 3] = ["2026-07-28", "2025-11-25", "2025-06-18"];
const INBOX_TAG: &str = "agent";
const DOCS_AT_MOST: usize = 500;

fn instructions(today: jiff::civil::Date) -> String {
    format!("Today is {today}.\n\n{TAUGHT}")
}

const TAUGHT: &str = "\
Tisty is one person's task list on this machine. You propose work for it; you never close, \
drop, delete or edit what the person wrote. There is no tool for those, on purpose.

Always pass `source` when you have one: a message id, a thread link, anything stable \
enough to recognise the same thing twice. Tisty refuses a second filing from the same \
source and hands back the task that already exists, so you cannot duplicate by mistake. \
Without a source, `find` by text before you propose.

Everything you propose lands in the inbox tagged #agent, for the person to place. You cannot \
choose a list. Dates are plain ISO (2026-08-31), never words: work out which day someone \
means by \"Monday\" from the date above, before calling.

Fill in what you actually know. A title alone is a fine task; inventing a deadline nobody \
gave you is worse than leaving it empty. Put what you read in `description`. Write titles \
and notes in the language the person writes in.

A document is for what is worth keeping and is not work to do — a summary, a note, something to consult. Writing one creates no task: if something has to happen, propose it.

`note` appends to a task's journal, including tasks the person wrote themselves. Use it \
when something new turns up about work that already exists, rather than filing a duplicate. \
`read` gives you one whole task — description, steps, journal, what it keeps — so ask for it \
before adding a note and you will not write down what is already written.";

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

    Some(match method {
        "server/discover" => reply(id, discovered()),
        "initialize" => reply(id, legacy_greeting(&params)),
        "ping" => reply(id, json!({})),
        "tools/list" => reply(id, json!({ "resultType": "complete", "tools": tools() })),
        "tools/call" => match called(paths, &params) {
            Ok(said) => reply(id, said),
            Err(Refused::Protocol(code, why)) => fault(id, code, &why),
            Err(Refused::Tool(why)) => reply(id, wrong(&why)),
        },
        _ => fault(id, -32601, &format!("unknown method: {method}")),
    })
}

fn discovered() -> Value {
    json!({
        "resultType": "complete",
        "supportedVersions": VERSIONS,
        "capabilities": { "tools": {} },
        "instructions": instructions(jiff::Zoned::now().date()),
        "_meta": { "io.modelcontextprotocol/serverInfo": who() },
    })
}

fn legacy_greeting(params: &Value) -> Value {
    let asked = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or(VERSIONS[0]);
    let speaking = if VERSIONS.contains(&asked) {
        asked
    } else {
        VERSIONS[0]
    };
    json!({
        "protocolVersion": speaking,
        "capabilities": { "tools": {} },
        "serverInfo": who(),
        "instructions": instructions(jiff::Zoned::now().date()),
    })
}

fn who() -> Value {
    json!({ "name": "tisty", "version": env!("CARGO_PKG_VERSION") })
}

fn reply(id: Value, result: Value) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string()
}

fn fault(id: Value, code: i32, message: &str) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }).to_string()
}

fn wrong(why: &str) -> Value {
    json!({
        "resultType": "complete",
        "content": [{ "type": "text", "text": why }],
        "isError": true,
    })
}

fn told(text: String, structured: Value) -> Value {
    json!({
        "resultType": "complete",
        "content": [{ "type": "text", "text": text }],
        "structuredContent": structured,
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
        "note" => note(paths, &args),
        "find" => find(paths, &args),
        "read" => read(paths, &args),
        "write_doc" => write_doc(paths, &args),
        "read_doc" => read_doc(paths, &args),
        "attach" => attach(paths, &args),
        "" => Err(Refused::Protocol(-32602, "a call needs a name".into())),
        other => Err(Refused::Protocol(-32602, format!("unknown tool: {other}"))),
    }
}

/// A misspelt argument would otherwise be dropped in silence, teaching the model nothing.
fn only_what_it_takes(name: &str, args: &Value) -> Result<(), Refused> {
    let Some(said) = args.as_object() else {
        return Ok(());
    };
    let tools = tools();
    let Some(taken) = tools
        .as_array()
        .and_then(|all| all.iter().find(|one| one["name"] == name))
        .and_then(|one| one["inputSchema"]["properties"].as_object())
    else {
        return Ok(());
    };
    if let Some(stray) = said.keys().find(|key| !taken.contains_key(*key)) {
        let mut known: Vec<&str> = taken.keys().map(String::as_str).collect();
        known.sort_unstable();
        return Err(Refused::Tool(format!(
            "`{stray}` is not something `{name}` takes. It takes: {}.",
            known.join(", ")
        )));
    }
    Ok(())
}

const AT_MOST: &[(&str, usize)] = &[
    ("title", 500),
    ("description", 64_000),
    ("body", 64_000),
    ("source", 512),
    ("label", 200),
];
const MANY_AT_MOST: &[(&str, usize)] = &[("tags", 32), ("steps", 200)];

/// An append-only log rereads a ten-megabyte title forever, and control characters in one
/// would rewrite the terminal it prints on.
fn short_and_plain(args: &Value) -> Result<(), Refused> {
    let Some(said) = args.as_object() else {
        return Ok(());
    };
    for (key, most) in AT_MOST {
        let Some(one) = said.get(*key).and_then(Value::as_str) else {
            continue;
        };
        if one.chars().count() > *most {
            return Err(Refused::Tool(format!(
                "`{key}` is longer than the {most} characters Tisty keeps. Shorten it."
            )));
        }
        if one
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
        {
            return Err(Refused::Tool(format!(
                "`{key}` carries control characters. Send plain text."
            )));
        }
    }
    for (key, most) in MANY_AT_MOST {
        if said
            .get(*key)
            .and_then(Value::as_array)
            .is_some_and(|all| all.len() > *most)
        {
            return Err(Refused::Tool(format!("`{key}` takes at most {most}.")));
        }
    }
    Ok(())
}

fn text(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|said| !said.is_empty())
        .map(ToString::to_string)
}

fn day(args: &Value, key: &str) -> Result<Option<DateSpec>, Refused> {
    let Some(said) = text(args, key) else {
        return Ok(None);
    };
    let zone = jiff::tz::TimeZone::system();
    let named = zone.iana_name().unwrap_or("UTC").to_string();
    said.parse::<jiff::civil::Date>()
        .map(|on| Some(DateSpec::all_day(on, named)))
        .map_err(|_| {
            Refused::Tool(format!(
                "`{key}` has to be a plain date like 2026-08-31, not {said:?}. Work out the day \
                 yourself before calling."
            ))
        })
}

fn ranked(args: &Value) -> Result<Option<Priority>, Refused> {
    let Some(said) = text(args, "priority") else {
        return Ok(None);
    };
    said.parse::<Priority>().map(Some).map_err(|_| {
        Refused::Tool(format!(
            "`priority` is do, decide, delegate or minor — not {said:?}. Leave it out if nobody \
             said which."
        ))
    })
}

fn listed(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(Value::as_array)
        .map(|all| {
            all.iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|said| !said.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
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
            "no agent is registered on this machine. The person turns one on in Tisty's settings,              under Agents."
                .into(),
        ));
    }
    let store = Store::open(paths.store(), agent).map_err(hitch)?;
    Ok((state, store))
}

fn hitch(e: tisty_core::Error) -> Refused {
    Refused::Tool(match e {
        tisty_core::Error::AlreadyRunning => {
            "Tisty is being written to right now. Try the same call again.".into()
        }
        other => format!("Tisty could not be read or written: {other}"),
    })
}

fn propose(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(title) = text(args, "title") else {
        return Err(Refused::Tool("a task needs a `title`.".into()));
    };
    let (state, mut store) = opened(paths)?;

    if let Some(source) = text(args, "source")
        && let Some(held) = state.sourced.get(&source)
        && let Some(task) = state.tasks.get(held)
    {
        return Ok(told(
            format!(
                "Already proposed from that source: {:?}. Nothing was written.",
                task.title
            ),
            json!({ "id": task.id.to_string(), "title": task.title, "proposed": false }),
        ));
    }

    let mut tags: Vec<Tag> = listed(args, "tags")
        .iter()
        .filter_map(|said| Tag::new(said).ok())
        .collect();
    if let Ok(mine) = Tag::new(INBOX_TAG)
        && !tags.contains(&mine)
    {
        tags.push(mine);
    }

    let draft = Draft {
        title: title.clone(),
        date: day(args, "date")?,
        deadline: day(args, "deadline")?,
        priority: ranked(args)?,
        tags,
        filing: None,
        repeat: None,
        source: text(args, "source"),
    };
    let plan = tisty_core::capture::plan(&state, draft).map_err(refused)?;
    let id = plan.task;
    let mut ops = plan.ops;
    if let Some(body) = text(args, "description") {
        ops.push(Op::TaskDescribe {
            id,
            d: Body { body: Some(body) },
        });
    }
    let mut step = order::first();
    for one in listed(args, "steps") {
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
    let written = store
        .append_batch_unless(ops, move |events| match &taken {
            None => false,
            Some(one) => State::replay(events).sourced.contains_key(one),
        })
        .map_err(hitch)?;

    let Some(_) = written else {
        let held = State::replay(&store.read_all().map_err(hitch)?);
        let task = source
            .as_deref()
            .and_then(|one| held.sourced.get(one))
            .and_then(|id| held.tasks.get(id));
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
    Ok(told(
        format!("Proposed {title:?} as {id} in the inbox, tagged #{INBOX_TAG}."),
        json!({ "id": id.to_string(), "title": title, "proposed": true }),
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
        return Err(Refused::Tool(format!(
            "no task here has the id {said}. It may have been deleted."
        )));
    };

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

fn attach(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "path") else {
        return Err(Refused::Tool(
            "attaching needs a `path` to a file on this machine.".into(),
        ));
    };
    let Some(who) = text(args, "task") else {
        return Err(Refused::Tool("attaching needs a `task` id.".into()));
    };
    let (state, mut store) = opened(paths)?;
    let Ok(id) = who.parse::<TaskId>() else {
        return Err(Refused::Tool(format!(
            "{who:?} is not a task id. Use the `id` that `find` or `propose` gave you."
        )));
    };
    let Some(task) = state.tasks.get(&id) else {
        return Err(Refused::Tool(format!("no task here has the id {who}.")));
    };

    let asked = std::path::Path::new(&said);
    let at = &tisty_core::agent::may_attach(asked, paths).map_err(|_| {
        Refused::Tool(format!(
            "{said:?} is not somewhere an assistant may take files from. Those are: {}.",
            tisty_core::agent::reachable()
                .iter()
                .map(|one| one.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    })?;
    let named = tisty_core::attach::called(at, text(args, "label"));
    let limit = tisty_core::Config::load_or_init(paths)
        .map_err(hitch)?
        .copies_up_to();
    let kept = tisty_core::attach::keep(at, paths.data(), limit).map_err(|e| match e {
        tisty_core::Error::AttachmentTooBig { bytes, .. } => Refused::Tool(format!(
            "that file is {} MB and this machine copies at most {} MB.",
            bytes / 1_000_000,
            limit / 1_000_000
        )),
        _ => Refused::Tool(format!("{said:?} could not be read from this machine.")),
    })?;

    let lang = crate::i18n::Lang::detect(
        tisty_core::Config::load_or_init(paths)
            .map_err(hitch)?
            .locale
            .as_deref(),
    );
    let body = tisty_core::attach::journalled(&kept, &named, at, lang.get("attached-from"));
    let zone = jiff::tz::TimeZone::system();
    store
        .append(Op::TaskLog {
            id,
            d: LogAdd::new(Ulid::generate(), body).in_zone(zone.iana_name().map(str::to_string)),
        })
        .map_err(hitch)?;

    Ok(told(
        format!("Kept {named:?} with {:?}.", task.title),
        json!({ "id": id.to_string(), "at": kept.at, "label": named }),
    ))
}

fn find(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;

    if let Some(source) = text(args, "source") {
        let held = state
            .sourced
            .get(&source)
            .and_then(|id| state.tasks.get(id));
        return Ok(told(
            match held {
                Some(task) => format!("Already proposed from that source: {:?}", task.title),
                None => "Nothing here came from that source.".into(),
            },
            json!({ "found": held.map(|task| brief(task, &state)) }),
        ));
    }

    let Some(query) = text(args, "query") else {
        return Err(Refused::Tool(
            "`find` needs a `query`, or a `source` to check whether it was proposed already."
                .into(),
        ));
    };
    let scope = match text(args, "scope").as_deref() {
        Some("open") => tisty_core::view::Scope::Open,
        Some("archive") => tisty_core::view::Scope::Archived,
        None | Some("either") => tisty_core::view::Scope::Either,
        Some(said) => {
            return Err(Refused::Tool(format!(
                "`scope` is open, archive or either — not {said:?}."
            )));
        }
    };
    let most = args
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(20)
        .clamp(1, 100) as usize;

    // Counting what it cannot see would still say the thing exists.
    let (hits, _) = state.searching(&query.to_lowercase(), scope, usize::MAX);
    let hits: Vec<&Task> = hits.into_iter().filter(|one| !one.folded()).collect();
    let all = hits.len();
    let hits: Vec<&Task> = hits.into_iter().take(most).collect();
    let found: Vec<Value> = hits.iter().map(|task| brief(task, &state)).collect();
    let lines: Vec<String> = hits
        .iter()
        .map(|task| format!("{} — {} ({})", task.id, task.title, named(task.status)))
        .collect();
    Ok(told(
        format!(
            "{all} match {query:?}; showing {}.
{}",
            found.len(),
            lines.join(
                "
"
            )
        ),
        json!({ "matches": found, "total": all }),
    ))
}

fn named(status: tisty_core::model::Status) -> &'static str {
    match status {
        tisty_core::model::Status::Open => "open",
        tisty_core::model::Status::Done => "done",
        tisty_core::model::Status::Dropped => "dropped",
    }
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
        return Err(Refused::Tool(format!(
            "no task here has the id {said}. Look it up again with `find`."
        )));
    };

    let mut whole = brief(task, &state);
    whole["description"] = json!(task.description);
    whole["steps"] = json!(
        task.steps
            .iter()
            .map(|one| json!({ "text": one.text, "done": one.done }))
            .collect::<Vec<_>>()
    );
    whole["journal"] = json!(
        task.log
            .iter()
            .map(|one| json!({ "at": one.at.to_string(), "body": kept_here(&one.body) }))
            .collect::<Vec<_>>()
    );
    whole["kept"] = json!(
        task.references()
            .iter()
            .map(|one| json!({ "target": one.target, "label": one.label }))
            .collect::<Vec<_>>()
    );

    let mut plainly = format!("{} — {}", task.id, task.title);
    if let Some(body) = &task.description {
        plainly.push_str("\n\n");
        plainly.push_str(body);
    }
    for one in &task.steps {
        plainly.push_str(&format!(
            "\n[{}] {}",
            if one.done { 'x' } else { ' ' },
            one.text
        ));
    }
    for one in &task.log {
        plainly.push_str(&format!("\n\n({}) {}", one.at, kept_here(&one.body)));
    }
    Ok(told(plainly, whole))
}

fn write_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(body) = text(args, "body") else {
        return Err(Refused::Tool("a document needs a `body`.".into()));
    };
    let (state, mut store) = opened(paths)?;

    tisty_core::docs::survives(&body).map_err(|eats| {
        Refused::Tool(format!(
            "Tisty's editor cannot keep {eats}, and would destroy it the first time the person              opens the document. Send plain markdown: headings, lists, emphasis, inline links."
        ))
    })?;

    // An append-only store keeps every one of these forever, and the window replays them all.
    if state.docs.len() >= DOCS_AT_MOST {
        return Err(Refused::Tool(format!(
            "there are already {DOCS_AT_MOST} documents here. Add to a task's journal instead, or              ask the person to clear some."
        )));
    }
    let made = tisty_core::docs::create(&paths.docs(), store.device(), &body).map_err(hitch)?;
    let order = tisty_core::order::last_of(
        state
            .docs
            .values()
            .filter(|one| one.folder.is_none())
            .map(|one| one.order.as_str()),
    );
    store
        .append(Op::DocAdd {
            id: Ulid::generate(),
            d: tisty_core::event::DocAdd {
                file: made.id.clone(),
                order,
                folder: None,
            },
        })
        .map_err(hitch)?;

    Ok(told(
        format!("Wrote {:?} as {}.", made.title, made.id),
        json!({ "doc": made.id, "title": made.title }),
    ))
}

fn read_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(which) = text(args, "doc") else {
        return Err(Refused::Tool(
            "reading a document needs its `doc` name.".into(),
        ));
    };
    let (state, _) = opened(paths)?;
    if !state.docs.values().any(|one| one.file == which) {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}."
        )));
    }
    let body = tisty_core::docs::read(&paths.docs(), &which).map_err(hitch)?;
    Ok(told(
        body.clone(),
        json!({ "doc": which, "title": tisty_core::docs::titled(&body) }),
    ))
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
        let drive = at + 2 < bytes.len()
            && bytes[at].is_ascii_alphabetic()
            && bytes[at + 1] == b':'
            && (bytes[at + 2] == b'/' || bytes[at + 2] == b'\\');
        let rooted = bytes[at] == b'/' && at > 0 && bytes[at - 1] == b' ';
        drive || rooted
    })
}

fn brief(task: &Task, state: &State) -> Value {
    json!({
        "id": task.id.to_string(),
        "title": task.title,
        "status": task.status,
        "date": task.date.as_ref().map(|d| d.date().to_string()),
        "deadline": task.deadline.as_ref().map(|d| d.date().to_string()),
        "tags": task.tags.iter().map(Tag::as_str).collect::<Vec<_>>(),
        "source": task.source,
        "by_agent": task
            .created_by
            .as_ref()
            .is_some_and(|who| state.agents.contains(who)),
    })
}

fn refused(e: Rejected) -> Refused {
    match e {
        Rejected::Untitled => Refused::Tool("a task needs a title.".into()),
        other => Refused::Protocol(-32603, format!("{other:?}")),
    }
}

fn tools() -> Value {
    json!([
        {
            "name": "propose",
            "title": "Propose a task",
            "description": "Propose a task for the person's inbox. Check `find` with the same \
                            `source` first: if it comes back with a task, this was proposed already. \
                            Fill in only what you were actually told.",
            "inputSchema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "title": { "type": "string", "description": "What has to be done, in a line" },
                    "description": {
                        "type": "string",
                        "description": "What you read, in markdown. Where the detail goes"
                    },
                    "date": {
                        "type": "string",
                        "description": "The day it is meant to be done, as 2026-08-31"
                    },
                    "deadline": {
                        "type": "string",
                        "description": "The day it actually runs out, as 2026-08-31"
                    },
                    "priority": {
                        "type": "string",
                        "enum": ["do", "decide", "delegate", "minor"],
                        "description": "Only if someone said so. Leave it out otherwise"
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Lowercase words; spaces become dashes"
                    },
                    "steps": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "A checklist, if the thing has parts"
                    },
                    "source": {
                        "type": "string",
                        "description": "Where you read it — a message id, a thread link. Stable \
                                        enough to recognise the same thing twice"
                    }
                },
                "required": ["title"]
            }
        },
        {
            "name": "note",
            "title": "Add to a task's journal",
            "description": "Append to what a task has recorded. Works on tasks the person wrote \
                            too. Use it when something new turns up about work that already \
                            exists, instead of filing a duplicate.",
            "inputSchema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "task": { "type": "string", "description": "The task id" },
                    "body": { "type": "string", "description": "What to record, in markdown" }
                },
                "required": ["task", "body"]
            }
        },
        {
            "name": "attach",
            "title": "Keep a file with a task",
            "description": "Copy a file from this machine into Tisty and record it on a task's journal. The file is copied, not linked, and where it came from is written down beside it. Only attach what you were asked to.",
            "inputSchema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "task": { "type": "string", "description": "The task id" },
                    "path": {
                        "type": "string",
                        "description": "A path to a file on this machine"
                    },
                    "label": {
                        "type": "string",
                        "description": "What to call it. Defaults to the file's own name"
                    }
                },
                "required": ["task", "path"]
            }
        },
        {
            "name": "write_doc",
            "title": "Write a document",
            "description": "Write something down that is not work to do: a note, a summary,                             something to keep. Plain markdown only — headings, lists, emphasis,                             inline links. Documents do not create tasks.",
            "inputSchema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "body": {
                        "type": "string",
                        "description": "The whole document. Its first line becomes its title"
                    }
                },
                "required": ["body"]
            }
        },
        {
            "name": "read_doc",
            "title": "Read a document",
            "description": "The whole text of a document that already exists. You can write new                             ones and read any of them, but never rewrite one: two sides editing                             the same text cannot be merged, so one would be lost.",
            "inputSchema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "doc": { "type": "string", "description": "The document's name" }
                },
                "required": ["doc"]
            }
        },
        {
            "name": "read",
            "title": "Read a whole task",
            "description": "Everything one task holds: its description, its steps, its journal                             and what it keeps. Ask for it before adding a note, so you do not                             write down something already written.",
            "inputSchema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "task": { "type": "string", "description": "The task id" }
                },
                "required": ["task"]
            }
        },
        {
            "name": "find",
            "title": "Search the list and the archive",
            "description": "Search by text, or pass `source` alone to check whether something was \
                            already came from it. Do that before proposing.",
            "inputSchema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "query": { "type": "string", "description": "Words to look for" },
                    "source": {
                        "type": "string",
                        "description": "Ask whether this exact source was proposed already"
                    },
                    "scope": {
                        "type": "string",
                        "enum": ["open", "archive", "either"],
                        "description": "Defaults to either"
                    },
                    "limit": { "type": "integer", "description": "At most 100, 20 by default" }
                }
            }
        }
    ])
}
