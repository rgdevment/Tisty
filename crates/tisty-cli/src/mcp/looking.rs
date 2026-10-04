use serde_json::{Value, json};
use tisty_core::Paths;

use tisty_core::State;
use tisty_core::model::{Priority, Tag, Task, TaskId};
use ulid::Ulid;

use super::asked::{strings, text};
use super::jsonrpc::told;
use super::{
    Refused, alike, already, hitch, named, named_doc, opened, said, scoped, trail, unpathed, when,
};

pub(super) fn find(paths: &Paths, args: &Value) -> Result<Value, Refused> {
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
    let wheres: Vec<Option<(&str, String)>> = hits
        .iter()
        .map(|task| {
            query
                .as_deref()
                .and_then(|query| tisty_core::view::mentioned_where(task, query))
                .map(|(kind, line)| (kind, kept_here(&line)))
        })
        .collect();
    let found: Vec<Value> = hits
        .iter()
        .zip(&wheres)
        .map(|(task, said)| {
            let mut one = brief(task, &state);
            if let Some((kind, line)) = said {
                one["in"] = json!(kind);
                one["line"] = json!(line);
            }
            one
        })
        .collect();
    // `after` walks the tasks only — paging past them would empty this list without saying why.
    let papers = match (&query, sifted.none()) {
        (Some(query), true) => papers_matching(paths, &state, query, scope, usize::MAX),
        _ => Vec::new(),
    };
    let papers_all = papers.len();
    let papers: Vec<Value> = papers.into_iter().take(most).collect();
    let mut lines: Vec<String> = hits
        .iter()
        .zip(&wheres)
        .map(|(task, said)| match said {
            Some((kind, line)) => format!(
                "{} — {} ({})\n    {kind}: {line}",
                task.id,
                task.title,
                standing(task)
            ),
            None => format!("{} — {} ({})", task.id, task.title, standing(task)),
        })
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

pub(super) fn inside_a_doc(
    paths: &Paths,
    state: &State,
    which: &str,
    args: &Value,
) -> Result<Value, Refused> {
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

pub(super) fn read(paths: &Paths, args: &Value) -> Result<Value, Refused> {
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
                .map(|one| match one.by_agent {
                    true => json!({ "text": one.text, "done": one.done, "by_agent": true }),
                    false => json!({ "text": one.text, "done": one.done }),
                })
                .collect::<Vec<_>>()
        );
    }
    if wants("journal") && task.journal().next().is_some() {
        whole["journal"] = json!(
            task.journal()
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
        plainly.push_str(
            " (open to agents: yours to describe, plan, tick, untick and say done or not doing)",
        );
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
        for one in task.journal() {
            plainly.push_str(&format!("\n\n({}) {}", one.at, kept_here(&one.body)));
        }
    }
    Ok(told(plainly, whole))
}

pub(super) fn catch_up(paths: &Paths, args: &Value) -> Result<Value, Refused> {
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

pub(super) fn lists(paths: &Paths) -> Result<Value, Refused> {
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

pub(super) fn tags(paths: &Paths) -> Result<Value, Refused> {
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
pub(super) fn sum_up(paths: &Paths, args: &Value) -> Result<Value, Refused> {
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

const AROUND_A_HIT: usize = 1;

pub(super) const NEWEST_SHOWN: usize = 8;

const TAGS_SHOWN: usize = 300;

/// The tombstone knows: what the person erased is said to be erased, not merely missing.
pub(super) fn gone(state: &State, said: &str) -> Refused {
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

pub(super) fn standing(task: &Task) -> String {
    match task.completed_at.filter(|_| !task.is_open()) {
        Some(at) => format!("{} {}", named(task.status), when(at)),
        None => named(task.status).to_string(),
    }
}

/// Done and dropped are not the same thing to tell the person who asks; and what they put
/// away, `read` does not reach.
pub(super) fn how_it_ended(task: &Task) -> String {
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

/// Every op carries its own id under the same name, so reading it back as JSON keeps this from
/// having to know each one — and from going quiet the day another is added.
fn what_it_touched(event: &tisty_core::event::Event) -> Option<(String, String)> {
    let said = serde_json::to_value(&event.op).ok()?;
    let named = said.get("op")?.as_str()?.to_string();
    let id = said.get("id")?.as_str()?.to_string();
    Some((named, id))
}

/// Attaching records where a file came from, and those paths are the person's disk. The agent
/// needs the card, not the shape of their home directory.
fn kept_here(body: &str) -> String {
    body.lines().map(unpathed).collect::<Vec<_>>().join(
        "
",
    )
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

/// A field that says nothing still costs the reader a line, so it is left out.
pub(super) fn brief(task: &Task, state: &State) -> Value {
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
        json!(
            task.resolved
                .as_ref()
                .filter(|one| !one.drop)
                .map(|one| when(one.at))
        ),
    );
    put(
        "said_not_doing",
        json!(
            task.resolved
                .as_ref()
                .filter(|one| one.drop)
                .map(|one| when(one.at))
        ),
    );
    put("open_to_agents", json!(task.open_to_agents.then_some(true)));
    Value::Object(kept)
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
            && (task.resolved.is_some() && task.is_open()) != want
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
