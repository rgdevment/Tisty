use serde_json::{Value, json};
use tisty_core::Paths;

use tisty_core::State;
use tisty_core::model::{Priority, Tag, Task, TaskId};
use tisty_core::standing::Standing;

use super::asked::{strings, text};
use super::jsonrpc::{mirrored, told};
use super::mistaken::{named_document, no_document_called, not_a_task_id};
use super::resuming::{
    Asked, MOVED_SHOWN, Since, ended_task, last_task, left_off, looks_now, moved_since, seen_whole,
    whose,
};
use super::task_reading::{Page, widened, window};
use super::unpathing::{fenced_in, unpathed_body};
use super::{
    Refused, alike, already, doc_named, hitch, named, named_doc, opened, said, scoped, trail, when,
};

pub(super) fn find(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;
    let today = jiff::Zoned::now().date();

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
            json!({ "found": held.map(|task| brief(task, &state, today)) }),
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
             `open_to_agents`, `part_of`, `from_source`, `from` and `to` to sift by."
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
    let mut hits: Vec<&Task> = match &query {
        Some(query) => state.searching(query, scope, usize::MAX).0,
        None => {
            let mut all: Vec<&Task> = state
                .tasks
                .values()
                .filter(|one| scope.keeps(one))
                .collect();
            all.sort_by_key(|one| (one.date.as_ref().map(|d| d.date()), one.id));
            all
        }
    };
    // One walk through the documents serves both the documents found and the tasks found by theirs.
    let seen = query
        .as_deref()
        .map(|query| sightings(paths, query))
        .unwrap_or_default();
    let mut lined = std::collections::HashMap::new();
    if query.is_some() {
        let mut held: std::collections::HashSet<_> = hits.iter().map(|hit| hit.id).collect();
        for (task, line) in documented(&state, &seen) {
            let Some(one) = state.tasks.get(&task) else {
                continue;
            };
            if scope.keeps(one) && held.insert(task) {
                hits.push(one);
                lined.insert(task, line);
            }
        }
    }
    let hits: Vec<&Task> = hits
        .into_iter()
        .filter(|one| !one.folded() && sifted.keeps(one, &state))
        .collect();
    let all = hits.len();
    let hits: Vec<&Task> = hits.into_iter().skip(past).take(most).collect();
    let mut hid = false;
    let wheres: Vec<Option<(&str, String)>> = hits
        .iter()
        .map(|task| {
            let (kind, line, kept) = match lined.get(&task.id) {
                Some(line) => ("document", line.clone(), kept_here(line)),
                None => {
                    let (kind, line) = tisty_core::view::mentioned_where(task, query.as_deref()?)?;
                    let kept = kept_where(task, kind, &line);
                    (kind, line, kept)
                }
            };
            hid |= kept != line;
            Some((kind, kept))
        })
        .collect();
    let found: Vec<Value> = hits
        .iter()
        .zip(&wheres)
        .map(|(task, said)| {
            let mut one = brief(task, &state, today);
            if let Some((kind, line)) = said {
                one["in"] = json!(kind);
                one["line"] = json!(line);
            }
            one
        })
        .collect();
    // `after` walks the tasks only — paging past them would empty this list without saying why.
    let within = match text(args, "folder") {
        Some(said) => Some(super::folder_found(&state, &said)?),
        None => None,
    };
    let papers = match (&query, sifted.none()) {
        (Some(_), true) => papers_matching(&state, &seen, scope, within),
        _ => Vec::new(),
    };
    let papers_all = papers.len();
    let papers: Vec<Value> = papers.into_iter().take(most).collect();
    let mut lines: Vec<String> = hits
        .iter()
        .zip(&wheres)
        .map(|(task, said)| match said {
            Some((kind, line)) => format!(
                "{:?} {} ({}) [{}]\n  {kind}: {line}",
                task.title,
                placed(&state, task),
                standing(task),
                task.id
            ),
            None => format!(
                "{:?} {} ({}) [{}]",
                task.title,
                placed(&state, task),
                standing(task),
                task.id
            ),
        })
        .collect();
    lines.extend(papers.iter().map(|one| {
        let put_away = if one["archived"] == json!(true) {
            ", put away"
        } else {
            ""
        };
        let what = match one["page_of"].as_str() {
            Some(up) => format!("a page of {}", super::doc_titled(&state, up)),
            None => "document".into(),
        };
        let marked = match one["marked"].as_str() {
            Some(how) => format!(", marked as {how}, waiting for the person"),
            None => String::new(),
        };
        let line = match said(one, "line") {
            "" => String::new(),
            line => format!("\n  line: {line}"),
        };
        format!(
            "{:?} ({what}{put_away}{marked}) [{}]{line}",
            said(one, "title"),
            said(one, "doc")
        )
    }));
    let asked = query.clone().unwrap_or_else(|| sifted.said());
    if hid {
        lines.push(UNPATHED.to_string());
    }
    let mut whole = json!({
        "matches": found,
        "total": all,
        "docs": papers,
        "docsTotal": papers_all,
    });
    if hid {
        whole["unpathed"] = json!(UNPATHED);
    }
    Ok(mirrored(
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
        whole,
    ))
}

struct Sifted {
    tag: Option<String>,
    list: Option<String>,
    by_agent: Option<bool>,
    said_done: Option<bool>,
    open_to_agents: Option<bool>,
    part_of: Option<TaskId>,
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
    // A hidden task's words stay out of sight here as they do in `read`.
    let seen = state
        .docs
        .values()
        .find(|one| one.file == which)
        .filter(|one| state.doc_in_sight(one));
    if seen.is_none() {
        return Err(Refused::Tool(no_document_called(state, which)));
    }
    let named = doc_named(state, which);
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
    let reach = args
        .get("around")
        .and_then(Value::as_u64)
        .map_or(AROUND_A_HIT, |asked| (asked as usize).min(AROUND_AT_MOST));
    let found: Vec<Value> = at
        .iter()
        .take(most)
        .map(|n| {
            let first = n.saturating_sub(reach);
            let last = (n + reach).min(lines.len().saturating_sub(1));
            let mut hit = serde_json::Map::new();
            hit.insert("line".into(), json!(n + 1));
            hit.insert("text".into(), json!(lines[*n]));
            if (first..=last).any(|at| at != *n && !lines[at].trim().is_empty()) {
                let around: Vec<String> = (first..=last)
                    .map(|at| tisty_core::view::clipped(lines[at]))
                    .collect();
                hit.insert("around".into(), json!(around.join("\n")));
            }
            if let Some(section) = section_of(n + 1) {
                hit.insert("section".into(), section);
            }
            Value::Object(hit)
        })
        .collect();

    let said = match all {
        0 => format!("Nothing in {named} says {query:?}."),
        _ => format!(
            "{all} line(s) of {named} say {query:?}; showing {}.\n{}",
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
    Ok(mirrored(
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
    let today = jiff::Zoned::now().date();
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("reading needs a `task` id.".into()));
    };
    let (state, store) = opened(paths)?;
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(not_a_task_id(&state, &said)));
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
    if let Some(odd) = asked
        .iter()
        .find(|one| !READ_FIELDS.contains(&one.as_str()))
    {
        return Err(Refused::Tool(format!(
            "`fields` has no {odd:?}: it takes {}. The page is `document`, the notes are `journal`.",
            READ_FIELDS.join(", ")
        )));
    }
    let wants = |key: &str| asked.is_empty() || asked.iter().any(|one| one == key);

    let mut whole = match asked.is_empty() {
        true => brief(task, &state, today),
        false => {
            let brief = brief(task, &state, today);
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
    let owner = tisty_core::task_docs::owner(&state, task.id);
    let page = wants("document")
        .then(|| {
            let kept = tisty_core::task_docs::of(&state, owner)?;
            let body = tisty_core::docs::read(&paths.docs(), &kept.file).ok()?;
            let (body, boxed) =
                tisty_core::task_front::dressed_boxes(&body, task, list_of(&state, task));
            Some(Page::new(kept.file.clone(), body, boxed))
        })
        .flatten();
    let named = |key: &str| asked.iter().any(|one| one == key);
    let shown_page = page.as_ref().and_then(Page::whole);
    let description = task.description.as_deref().filter(|one| {
        named("description")
            || (wants("description")
                && !shown_page
                    .is_some_and(|page| tisty_core::task_docs::holds_description(&page.body, one)))
    });
    let steps_apart = task
        .steps
        .iter()
        .any(|one| !shown_page.is_some_and(|page| page.boxed.contains(&one.id)));
    if let Some(body) = description {
        whole["description"] = json!(body);
    }
    if wants("steps") && !task.steps.is_empty() {
        whole["steps"] = json!(
            task.steps
                .iter()
                .map(|one| {
                    let mut step = json!({ "text": one.text, "done": one.done });
                    if one.by_agent {
                        step["by_agent"] = json!(true);
                    }
                    if one.said {
                        step["said_done"] = json!(true);
                    }
                    step
                })
                .collect::<Vec<_>>()
        );
    }
    if let Some(page) = &page {
        whole["document"] = page.structured();
    }
    let parts: Vec<&Task> = state
        .parts_of(task.id)
        .filter(|one| !one.folded())
        .collect();
    if wants("parts") && !parts.is_empty() {
        whole["parts"] = json!(
            parts
                .iter()
                .map(|one| json!({ "id": one.id.to_string(), "title": one.title, "status": one.status }))
                .collect::<Vec<_>>()
        );
    }
    let journal: Vec<&tisty_core::model::LogEntry> = task.journal().collect();
    let notes = window(&journal, widened(args)?, kept_here, |one| {
        one.by
            .as_ref()
            .and_then(|by| whose(&state, by, one.via.as_deref(), store.device()))
    });
    if wants("journal") {
        notes.structured(&mut whole, UNPATHED);
    }
    if wants("kept") && !task.references().is_empty() {
        whole["kept"] = json!(
            task.references()
                .iter()
                .map(|one| json!({ "target": one.target, "label": one.label }))
                .collect::<Vec<_>>()
        );
    }

    let mut plainly = named_task(&state, task);
    if !task.is_open() {
        let notice = format!(
            "{} — {}: it reads as it ended, and nothing on it changes",
            standing(task),
            how_it_ended(task)
        );
        plainly.push_str(&format!(" ({notice})"));
        whole["notice"] = json!(notice);
    } else if task.open_to_agents {
        let notice =
            "open to agents: yours to describe, plan, tick, mend and say done or not doing";
        plainly.push_str(&format!(" ({notice})"));
        if wants("notice") {
            whole["notice"] = json!(notice);
        }
    }
    if let Some(body) = description {
        plainly.push_str("\n\n");
        plainly.push_str(body);
    }
    if wants("steps") && steps_apart {
        for one in &task.steps {
            plainly.push_str(&format!(
                "\n[{}] {}",
                if one.done { 'x' } else { ' ' },
                one.text
            ));
        }
    }
    if let Some(page) = &page {
        plainly.push_str(&page.told());
    }
    if wants("parts") {
        for one in &parts {
            plainly.push_str(&format!(
                "\npart [{}] {} — {}",
                if one.is_open() { ' ' } else { 'x' },
                one.id,
                one.title
            ));
        }
    }
    if wants("journal") {
        plainly.push_str(&notes.told(UNPATHED));
    }
    Ok(mirrored(plainly, whole))
}

pub(super) fn catch_up(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (state, store) = opened(paths)?;
    let today = jiff::Zoned::now().date();

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

    let tags = counted_tags(&state);

    let log = store.read_all().map_err(hitch)?;
    let head = log.iter().map(|one| one.timestamp).max();

    let mut kept = serde_json::Map::new();
    kept.insert("lists".into(), json!(named));
    if !folders.is_empty() {
        kept.insert(
            "folders".into(),
            json!(folders.iter().take(AT_A_GLANCE).collect::<Vec<_>>()),
        );
    }
    if folders.len() > AT_A_GLANCE {
        kept.insert("folders_more".into(), json!(folders.len() - AT_A_GLANCE));
    }
    if !tags.is_empty() {
        kept.insert(
            "tags".into(),
            json!(
                tags.iter()
                    .take(TAGS_AT_A_GLANCE)
                    .map(|(one, times)| json!({ "tag": one, "times": times }))
                    .collect::<Vec<_>>()
            ),
        );
    }
    if tags.len() > TAGS_AT_A_GLANCE {
        kept.insert("tags_more".into(), json!(tags.len() - TAGS_AT_A_GLANCE));
    }
    let open = state
        .tasks
        .values()
        .filter(|one| one.is_open() && !one.folded())
        .count();
    let papers = state.papers().filter(|one| !state.held_away(one)).count();
    let pressing = pressing(&state, today);
    let mut counts = json!({ "open": open, "docs": papers });
    for (how, many) in &pressing {
        counts[how] = json!(many);
    }
    kept.insert("counts".into(), counts);
    if let Some(head) = head {
        kept.insert("cursor".into(), json!(head.to_string()));
    }

    let left = left_off(
        paths,
        &state,
        log.iter().rev().filter(|one| one.device == *store.device()),
    );
    let mut said = Vec::new();
    if let Some(task) = last_task(&state, &left) {
        let on = left
            .task_at
            .map(|at| format!(", {}", when(at)))
            .unwrap_or_default();
        said.push(format!(
            "You last worked on {}{on}: `read` it.",
            named_task(&state, task)
        ));
        let mut last = json!({ "id": task.id.to_string(), "title": task.title });
        if let Some(list) = list_of(&state, task) {
            last["list"] = json!(list);
        }
        if let Some(at) = left.task_at {
            last["at"] = json!(when(at));
        }
        kept.insert("last".into(), last);
    } else if let Some(task) = ended_task(&state, &left) {
        let how = match &task.resolved {
            Some(said) if said.drop => "you said you are not doing it, and it waits for the person",
            Some(_) => "you said it is done, and it waits for the person",
            None if task.hidden => "the person put it out of sight",
            None if task.is_dropped() => "the person dropped it",
            None => "it is closed",
        };
        said.push(format!(
            "The task you were last on, {}, is not open to pick up: {how}.",
            named_task(&state, task)
        ));
    }
    let asked = match text(args, "since") {
        None => None,
        Some(since) => Some(since.parse::<jiff::Timestamp>().map_err(|_| {
            Refused::Tool(format!(
                "`since` has to be a `cursor` a previous `catch_up` handed back, not {since:?}."
            ))
        })?),
    };
    let since = asked
        .map(Since::At)
        .or_else(|| left.seen.clone().map(Since::Seen))
        .or(left.at.map(Since::At));
    let mut seen = seen_whole(&log);
    let mut whole = true;
    if let Some(since) = &since {
        let moved = moved_since(
            &state,
            &log,
            Asked {
                since,
                mine_too: asked.is_some(),
                seen: left.seen.clone().unwrap_or_default(),
            },
            store.device(),
            today,
        );
        let by = match moved.by.is_empty() {
            true => String::new(),
            false => format!(
                ", by {}",
                moved.by.iter().cloned().collect::<Vec<_>>().join(" and ")
            ),
        };
        let from = match (asked, left.looked_at.or_else(|| since.latest())) {
            (Some(_), _) => "then".to_string(),
            (None, Some(at)) => format!("you were last here, {}", when(at)),
            (None, None) => "you were last here".to_string(),
        };
        let cut = moved.cut();
        whole = !cut;
        said.push(format!(
            "{} task(s) and {} document(s) moved since {from}{by}.{}",
            moved.tasks_all,
            moved.docs_all,
            match (cut, asked.is_some()) {
                (false, _) => String::new(),
                (true, false) => format!(
                    " Here are the first that moved, at most {MOVED_SHOWN} of each kind; the \
                     next `catch_up` without `since` brings the rest."
                ),
                (true, true) => format!(
                    " Here are the first that moved, at most {MOVED_SHOWN} of each kind; ask \
                     again with this `cursor` for the rest."
                ),
            }
        ));
        if cut
            && asked.is_some()
            && let Some(resume) = moved.resume
        {
            kept.insert("cursor".into(), json!(resume.to_string()));
        }
        if asked.is_none()
            && let Some(at) = since.latest()
        {
            kept.insert("since".into(), json!(at.to_string()));
        }
        for (key, all, shown) in [
            ("tasks", moved.tasks_all, moved.tasks),
            ("docs", moved.docs_all, moved.docs),
        ] {
            if all > shown.len() {
                kept.insert(format!("{key}_more"), json!(all - shown.len()));
            }
            if !shown.is_empty() {
                kept.insert(key.into(), json!(shown));
            }
        }
        seen = moved.seen;
    }
    if asked.is_none() {
        let newest = newest_docs(&state);
        if !newest.is_empty() {
            kept.insert("newest".into(), json!(newest));
        }
        said.push(format!(
            "{} list(s), {} folder(s), {} tag(s), {open} open task(s){}, {papers} document(s) \
             not put away, pages included.",
            named.len(),
            folders.len(),
            tags.len(),
            pressed(&pressing),
        ));
        if folders.len() > AT_A_GLANCE {
            said.push(format!(
                "Here are the first {AT_A_GLANCE} folders; `docs` names every one."
            ));
        }
    }
    if since.is_none() {
        said.push("Next time, what moved since this call comes with it as well.".to_string());
    }
    if asked.is_none() {
        looks_now(paths, &seen, whole);
    }
    Ok(told(said.join(" "), Value::Object(kept)))
}

fn counted_tags(state: &State) -> Vec<(&str, usize)> {
    let mut how: std::collections::BTreeMap<&str, usize> = Default::default();
    for one in state
        .tasks
        .values()
        .flat_map(|task| &task.tags)
        .chain(state.papers().flat_map(|one| &one.tags))
    {
        *how.entry(one.as_str()).or_default() += 1;
    }
    let mut tags: Vec<(&str, usize)> = how.into_iter().collect();
    tags.sort_by(|(one, mine), (other, theirs)| theirs.cmp(mine).then(one.cmp(other)));
    tags
}

fn newest_docs(state: &State) -> Vec<Value> {
    let mut newest: Vec<&tisty_core::model::Kept> =
        state.papers().filter(|one| !state.held_away(one)).collect();
    newest.sort_by_key(|one| std::cmp::Reverse((one.wrote, one.id)));
    newest
        .iter()
        .take(NEWEST_SHOWN)
        .map(|one| json!({ "doc": one.file, "title": one.title }))
        .collect()
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

// The same text is published, drawn for people and whole for assistants, so an answer can point a person there.
pub(super) const GUIDE: &str = "https://markdown.tisty.app/";

pub(super) fn tags(paths: &Paths) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;
    let named = counted_tags(&state);

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
    if named_document(&state, &which).is_none() {
        return Err(Refused::Tool(no_document_called(&state, &which)));
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
const AROUND_AT_MOST: usize = 10;
pub(super) const NEWEST_SHOWN: usize = 8;

const TAGS_SHOWN: usize = 300;
const TAGS_AT_A_GLANCE: usize = 40;
const AT_A_GLANCE: usize = 30;
const READ_FIELDS: [&str; 25] = [
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
    "via",
    "said_done",
    "said_not_doing",
    "open_to_agents",
    "part_of",
    "parts",
    "description",
    "steps",
    "document",
    "journal",
    "kept",
];

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

/// Attaching records where a file came from, and those paths are the person's disk. The agent
/// needs the card, not the shape of their home directory.
fn kept_here(body: &str) -> String {
    unpathed_body(body)
}

const UNPATHED: &str =
    "A path on this machine reads as [path] here; the person sees it whole in the window.";

/// A line `find` quotes is read as it sits where it came from: inside a fence it is an example.
fn kept_where(task: &Task, kind: &str, line: &str) -> String {
    let fenced = match kind {
        "journal" => task.journal().any(|one| fenced_in(&one.body, line)),
        "description" => task
            .description
            .as_deref()
            .is_some_and(|body| fenced_in(body, line)),
        _ => false,
    };
    if fenced {
        line.to_string()
    } else {
        kept_here(line)
    }
}

/// Documents are searched by the same engine the window uses; a document the agent cannot find
/// again is a document it wrote into the dark.
fn papers_matching(
    state: &State,
    seen: &[tisty_core::docs::Sighting],
    scope: tisty_core::view::Scope,
    within: Option<tisty_core::model::FolderId>,
) -> Vec<Value> {
    let inside = |one: &tisty_core::model::Kept| {
        within.is_none_or(|at| {
            one.folder
                .is_some_and(|it| it == at || super::placing::beneath(state, it, at))
        })
    };
    let here: std::collections::HashMap<String, (bool, Option<String>, Option<String>)> = state
        .papers()
        .filter(|one| state.doc_kept_by(scope, one) && inside(one))
        .map(|one| {
            let marked = one
                .flagged
                .as_ref()
                .filter(|_| !state.held_away(one))
                .map(|how| {
                    super::papers::flagging::reason_said(
                        state,
                        how.reason.as_deref(),
                        how.instead.as_deref(),
                    )
                });
            (
                one.file.clone(),
                (
                    state.held_away(one),
                    one.page_of.and_then(|up| named_doc(state, up)),
                    marked,
                ),
            )
        })
        .collect();

    // A title that says it comes before a body that only mentions it, and what was marked old after both.
    let mut seen: Vec<&tisty_core::docs::Sighting> = seen
        .iter()
        .filter(|one| here.contains_key(&one.id))
        .collect();
    seen.sort_by_key(|one| {
        let marked = here.get(&one.id).is_some_and(|(_, _, how)| how.is_some());
        (marked, !one.line.is_empty())
    });
    seen.into_iter()
        .map(|one| {
            let (archived, page_of, marked) =
                here.get(&one.id).cloned().unwrap_or((false, None, None));
            let mut hit = json!({
                "doc": one.id,
                "title": one.title,
                "line": one.line,
                "page_of": page_of,
                "archived": archived,
                "flagged": marked.is_some(),
            });
            if let Some(how) = marked {
                hit["marked"] = json!(how);
            }
            hit
        })
        .collect()
}

fn sightings(paths: &Paths, query: &str) -> Vec<tisty_core::docs::Sighting> {
    let held = tisty_core::cache::Cache::open(paths.cache()).ok().flatten();
    tisty_core::docs::sighted(&paths.docs(), held.as_ref(), query, usize::MAX, |_| true)
        .unwrap_or_else(|| {
            tisty_core::docs::Corpus::default()
                .searching(&paths.docs(), query, usize::MAX, |_| true)
        })
}

// A task found by its document's words answers as the task, with the line that said it.
fn documented(
    state: &State,
    seen: &[tisty_core::docs::Sighting],
) -> Vec<(tisty_core::model::TaskId, String)> {
    let behind = tisty_core::task_docs::behind(state);
    seen.iter()
        .filter_map(|one| Some((*behind.get(&one.id)?, one.line.clone())))
        .collect()
}

/// A field that says nothing still costs the reader a line, so it is left out.
pub(super) fn brief(task: &Task, state: &State, today: jiff::civil::Date) -> Value {
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
    put("standing", json!(standing_of(task, today)));
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
    put("list", json!(list_of(state, task)));
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
    put(
        "part_of",
        json!(task.part_of.map(|whole| whole.to_string())),
    );
    let (closed, all) = state
        .parts_of(task.id)
        .filter(|part| !part.folded())
        .fold((0, 0), |(closed, all), part| {
            (closed + usize::from(!part.is_open()), all + 1)
        });
    if all > 0 {
        put("parts", json!(format!("{closed}/{all}")));
    }
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
            part_of: match text(args, "part_of") {
                None => None,
                Some(said) => Some(said.parse::<TaskId>().map_err(|_| {
                    Refused::Tool(format!(
                        "`part_of` takes a task id, and {said:?} is not one."
                    ))
                })?),
            },
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
            && self.part_of.is_none()
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
        if let Some(one) = &self.part_of {
            all.push(format!("parts of {one}"));
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
            let named = list_of(state, task).map(str::to_lowercase);
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
        if let Some(want) = self.part_of
            && task.part_of != Some(want)
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

/// A closed task's dates no longer press, so it is never handed a standing that would read as urgent.
pub(super) fn standing_of(task: &Task, today: jiff::civil::Date) -> Option<Standing> {
    let how = tisty_core::standing::standing(task, today);
    (task.is_open() && how != Standing::Undated).then_some(how)
}

/// Every count goes out even at zero, so a missing key never has to be read as «none».
pub(super) fn pressing(state: &State, today: jiff::civil::Date) -> Vec<(String, usize)> {
    let mut many = [
        (Standing::Overdue, 0),
        (Standing::DueSoon, 0),
        (Standing::Late, 0),
    ];
    for task in state.tasks.values().filter(|one| !one.folded()) {
        if let Some(how) = standing_of(task, today)
            && let Some((_, count)) = many.iter_mut().find(|(one, _)| *one == how)
        {
            *count += 1;
        }
    }
    many.into_iter()
        .map(|(how, count)| (json!(how).as_str().unwrap_or_default().to_string(), count))
        .collect()
}

fn pressed(pressing: &[(String, usize)]) -> String {
    let told: Vec<String> = pressing
        .iter()
        .filter(|(_, many)| *many > 0)
        .map(|(how, many)| format!("{many} {}", how.replace('_', " ")))
        .collect();
    if told.is_empty() {
        return String::new();
    }
    format!(" ({})", told.join(", "))
}

pub(super) fn list_of<'a>(state: &'a State, task: &Task) -> Option<&'a str> {
    task.list
        .and_then(|at| state.lists.get(&at))
        .map(|one| one.name.as_str())
}

pub(super) fn placed(state: &State, task: &Task) -> String {
    list_of(state, task).map_or_else(|| "in the inbox".to_string(), |one| format!("in {one}"))
}

pub(super) fn named_task(state: &State, task: &Task) -> String {
    format!("{:?} {} [{}]", task.title, placed(state, task), task.id)
}

#[cfg(test)]
#[path = "looking_test.rs"]
mod tests;
