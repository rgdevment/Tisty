use serde_json::{Value, json};
use tisty_core::Paths;

use tisty_core::State;
use tisty_core::model::{Priority, Step, Tag, Task, TaskId};
use tisty_core::standing::Standing;

use super::asked::{strings, text};
use super::dropped::{self, decided_not_to_do};
use super::jsonrpc::{mirrored, told};
use super::lists;
use super::mistaken::not_a_task_id;
use super::pressing::{name_pressing, pressed, pressing, tallied};
use super::resuming::{
    Asked, MOVED_SHOWN, Since, ended_task, last_task, left_off, looks_now, moved_since, seen_whole,
    whose,
};
use super::task_reading::{Page, widened, window};
use super::task_state::Stated;
use super::unpathing::unpathed_body;
use super::{Refused, hitch, named, opened, trail, when};

const TASK_ONLY: [&str; 2] = ["fields", "notes"];
const WAITING: &str = "waiting on the person";
const SAID_WAITING: &str = "said done, waiting on the person";
const DOC_ONLY: [&str; 9] = [
    "section", "from", "to", "chars", "cursor", "as", "marks", "versions", "version",
];

pub(super) fn read(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (only, other) = match (text(args, "task"), text(args, "doc")) {
        (Some(_), Some(_)) => {
            return Err(Refused::Tool(
                "`read` reads one thing at a time: a `task` or a `doc`, not both.".into(),
            ));
        }
        (None, None) => {
            return Err(Refused::Tool(
                "`read` needs a `task` id or a `doc` id; `brain` finds either.".into(),
            ));
        }
        (Some(_), None) => (&DOC_ONLY[..], "a `doc`"),
        (None, Some(_)) => (&TASK_ONLY[..], "a `task`"),
    };
    if let Some(stray) = only.iter().find(|key| args.get(**key).is_some()) {
        return Err(Refused::Tool(format!(
            "`{stray}` is for reading {other}, so it goes with {other} id. Nothing was read."
        )));
    }
    match text(args, "task") {
        Some(said) => read_task(paths, args, said),
        None => super::papers::reading::read_doc(paths, args),
    }
}

fn read_task(paths: &Paths, args: &Value, said: String) -> Result<Value, Refused> {
    let today = jiff::ZonedDateTime::now().date();
    let (state, store) = opened(paths)?;
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(not_a_task_id(&state, &said)));
    };
    if let Some(task) = state.tasks.get(&id).filter(|one| decided_not_to_do(one)) {
        let card = dropped::card(&state, task);
        return Ok(told(
            format!("{} {}", dropped::told(&card), dropped::ONLY_THIS),
            card,
        ));
    }
    // Hidden is the person taking something out of sight. An agent reading its journal whole
    // would undo that decision on the one task most likely to deserve it.
    let Some(task) = state.tasks.get(&id).filter(|one| !one.folded()) else {
        if state.is_erased(id) {
            return Err(gone(&state, &said));
        }
        return Err(Refused::Tool(format!(
            "no task here has the id {said}. Look it up again with `brain`."
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
    let (page, page_print) = wants("document")
        .then(|| {
            let kept = tisty_core::task_docs::of(&state, owner)?;
            let body = tisty_core::docs::read(&paths.docs(), &kept.file).ok()?;
            let print = tisty_core::attach::printed(body.as_bytes());
            // The page comes back as it is stored, so it can be written back as it came.
            let (body, boxed) = tisty_core::stages::ticked_boxes(task, &body);
            let yours = task.is_open();
            let digest = tisty_core::docs::read_notes_before(paths.data(), &kept.file)
                .filter(|slot| !slot.undone)
                .map(|slot| slot.after);
            Some((
                Page::new(kept.file.clone(), body, boxed, yours, digest),
                print,
            ))
        })
        .flatten()
        .unzip();
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
    let waiting = |one: &Step| task.is_open() && one.said && !one.done;
    if wants("steps") && !task.steps.is_empty() {
        whole["steps"] = json!(
            task.steps
                .iter()
                .map(|one| {
                    let mut step = json!({ "text": one.text, "done": one.done });
                    if one.by_agent {
                        step["by_agent"] = json!(true);
                    }
                    if waiting(one) {
                        step["said_done"] = json!(true);
                    }
                    step
                })
                .collect::<Vec<_>>()
        );
    }
    // The page no longer copies the journal, so the answer says how many notes it holds, as the window does.
    let noted = task.journal().count();
    if let Some(page) = &page {
        whole["document"] = page.structured();
        whole["document"]["print"] = json!(page_print);
        if noted > 0 {
            whole["document"]["journal"] = json!(noted);
        }
    } else if noted > 0 && wants("journal") {
        whole["journal_notes"] = json!(noted);
    }
    let parts: Vec<&Task> = state
        .parts_of(task.id)
        .filter(|one| !one.folded())
        .collect();
    if wants("parts") && !parts.is_empty() {
        whole["parts"] = json!(
            parts
                .iter()
                .map(|one| {
                    let mut part = json!({ "id": one.id.to_string(), "title": one.title, "status": one.status });
                    if waiting_word(one).is_some() {
                        part["stage"] = json!("waiting");
                    }
                    part
                })
                .collect::<Vec<_>>()
        );
    }
    let stated = (wants("state") || wants("journal"))
        .then(|| Stated::of(paths, &state, task, page.as_ref()));
    let journal: Vec<&tisty_core::model::LogEntry> = task.journal().collect();
    let after = stated.as_ref().and_then(Stated::after);
    let notes = window(&journal, widened(args)?, after, kept_here, |one| {
        one.by
            .as_ref()
            .and_then(|by| whose(&state, by, one.via.as_deref(), store.device()))
    });
    if wants("journal") {
        notes.structured(&mut whole);
        notes.counted(&mut whole, UNPATHED);
    }
    if let Some(stated) = stated.as_ref().filter(|_| wants("state")) {
        stated.structured(&mut whole, &notes);
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
    }
    if let Some(stated) = stated.as_ref().filter(|_| wants("state")) {
        plainly.push_str(&stated.told(&notes));
    }
    if let Some(body) = description {
        plainly.push_str("\n\n");
        plainly.push_str(body);
    }
    if wants("steps") && steps_apart {
        for one in &task.steps {
            plainly.push_str(&match (one.done, waiting(one)) {
                (true, _) => format!("\n[x] {}", one.text),
                (false, true) => format!("\n[~] {} ({SAID_WAITING})", one.text),
                (false, false) => format!("\n[ ] {}", one.text),
            });
        }
    }
    if let Some(page) = &page {
        plainly.push_str(&page.told());
    }
    // The page comes back as stored, so a box an agent said done is told after it.
    if wants("steps") && !steps_apart && task.steps.iter().any(waiting) {
        plainly.push_str("\n\nOn its page, steps an agent said done:");
        for one in task.steps.iter().filter(|one| waiting(one)) {
            plainly.push_str(&format!("\n[~] {} ({SAID_WAITING})", one.text));
        }
    }
    if wants("parts") {
        for one in &parts {
            let (mark, waiting) = match (one.is_open(), waiting_word(one)) {
                (false, _) => ('x', String::new()),
                (true, Some(word)) => ('~', format!(" ({word}, {WAITING})")),
                (true, None) => (' ', String::new()),
            };
            plainly.push_str(&format!(
                "\npart [{mark}] {} — {}{waiting}",
                one.id, one.title
            ));
        }
    }
    if wants("journal") {
        plainly.push_str(&notes.told(UNPATHED));
    }
    super::each::said_once(&mut whole, &["journal", "parts"]);
    let print = answer_print(&plainly, &whole);
    if let Some(since) = text(args, "print").and_then(|held| held_since(&held, &print)) {
        return Ok(told(
            format!(
                "Nothing on {} changed since you read it{since}: what you hold still stands. \
                 Without `print`, `read` brings it whole again.",
                named_task(&state, task)
            ),
            json!({ "id": task.id.to_string(), "status": task.status, "unchanged": true, "print": text(args, "print") }),
        ));
    }
    whole["print"] = json!(format!("{print}.{}", jiff::Timestamp::now().as_second()));
    Ok(mirrored(plainly, whole))
}

fn answer_print(plainly: &str, whole: &Value) -> String {
    let shown = format!("{plainly}\n{whole}");
    tisty_core::attach::printed(shown.as_bytes())[..PRINT_HELD].to_string()
}

// The print carries the second it was read at, so «unchanged since» needs no memory of the call.
fn held_since(held: &str, print: &str) -> Option<String> {
    let (said, at) = held.split_once('.').unwrap_or((held, ""));
    if said != print {
        return None;
    }
    let since = at
        .parse::<i64>()
        .ok()
        .and_then(|second| jiff::Timestamp::from_second(second).ok())
        .map(|at| format!(", {}", when(at)))
        .unwrap_or_default();
    Some(since)
}

const PRINT_HELD: usize = 16;

pub(super) fn catch_up(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (state, store) = opened(paths)?;
    let today = jiff::ZonedDateTime::now().date();

    let listed = lists::open_by_name(&state);
    let named: Vec<&str> = listed.iter().map(|(name, _)| *name).collect();

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
    let tally = tallied(&pressing);
    let mut counts = json!({ "open": open, "docs": papers });
    for (how, many) in &tally {
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
        let progress = super::brain::project::progress(paths, &state, task);
        said.push(format!(
            "You last worked on {}{on}: {}. `read` it.",
            named_task(&state, task),
            progress.told()
        ));
        let mut last = json!({ "id": task.id.to_string(), "title": task.title });
        progress.put(&mut last);
        last["said"] = json!(progress.how_far());
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
        let named = match task.hidden {
            true => String::new(),
            false => format!(", {},", named_task(&state, task)),
        };
        said.push(format!(
            "The task you were last on{named} is not open to pick up: {how}."
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
            pressed(&tally),
        ));
        let inbox = (lists::inbox_named(&state), lists::open_in_the_inbox(&state));
        said.push(open_by_list(&listed, inbox));
        kept.insert(
            "open_by_list".into(),
            listed
                .iter()
                .chain([&inbox])
                .map(|(name, open)| (name.to_string(), json!(open)))
                .collect::<serde_json::Map<_, _>>()
                .into(),
        );
        if let Some(named) = name_pressing(&state, &pressing, open, today, &mut kept) {
            said.push(named);
        }
        if folders.len() > AT_A_GLANCE {
            said.push(format!(
                "Here are the first {AT_A_GLANCE} folders; `brain` with nothing names every one."
            ));
        }
        if let Some((glanced, sheet)) = super::ideas::glance(paths, &state) {
            said.push(glanced);
            kept.insert("ideas".into(), sheet);
        }
        if let Some((due, review)) = super::reviewing::glance(paths, &state, today) {
            said.push(due);
            kept.insert("review".into(), review);
        }
    }
    if since.is_none() {
        said.push("Next time, what moved since this call comes with it as well.".to_string());
    }
    if asked.is_none() {
        looks_now(paths, &seen, whole);
    }
    let mut kept = Value::Object(kept);
    super::each::said_once(&mut kept, &["tasks", "docs", "pressing"]);
    Ok(told(said.join(" "), kept))
}

fn open_by_list(listed: &[(&str, usize)], (named, inbox): (&str, usize)) -> String {
    let all: String = listed
        .iter()
        .map(|(name, open)| format!("{name} {open}, "))
        .collect();
    format!(
        "Open by list: {all}and {inbox} in no list, which `brain` with `list` «{named}» brings."
    )
}

pub(super) fn counted_tags(state: &State) -> Vec<(&str, usize)> {
    let mut how: std::collections::BTreeMap<&str, usize> = Default::default();
    for one in state
        .tasks
        .values()
        .filter(|task| !task.hidden)
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

// The same text is published, drawn for people and whole for assistants, so an answer can point a person there.
pub(super) const GUIDE: &str = "https://markdown.tisty.app/";

pub(super) const NEWEST_SHOWN: usize = 8;

const TAGS_AT_A_GLANCE: usize = 40;
const AT_A_GLANCE: usize = 30;
const READ_FIELDS: [&str; 26] = [
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

/// The tombstone knows: what the person erased is said to be erased, not merely missing.
pub(super) fn gone(state: &State, said: &str) -> Refused {
    match said.parse::<TaskId>() {
        Ok(id) if state.is_erased(id) => Refused::Tool(format!(
            "{said} was erased by the person, and stays erased. If the same work has come back, \
             propose it anew."
        )),
        Ok(id) if let Some(refused) = dropped::refused(state, id) => refused,
        _ => Refused::Tool(format!(
            "no task here has the id {said}. It may have been deleted."
        )),
    }
}

// An agent's word leaves a part open until the person decides, as `brain`'s project view says.
fn waiting_word(part: &Task) -> Option<&'static str> {
    let word = part.resolved.as_ref().filter(|_| part.is_open())?;
    Some(if word.drop {
        "said not doing"
    } else {
        "said done"
    })
}

pub(super) fn standing(task: &Task) -> String {
    match task.completed_at.filter(|_| !task.is_open()) {
        Some(at) => format!("{} {}", named(task.status), when(at)),
        None => named(task.status).to_string(),
    }
}

/// Done and dropped are not the same thing to tell the person who asks; and what they hid,
/// `read` does not reach.
pub(super) fn how_it_ended(task: &Task) -> String {
    let mut said = match task.status {
        tisty_core::model::Status::Dropped => {
            "The person dropped it — they decided not to do it — and it is history".to_string()
        }
        _ => "It is done, and history".to_string(),
    };
    if task.hidden {
        said.push_str(", put away by the person where `read` does not reach");
    }
    said
}

/// Attaching records where a file came from, and those paths are the person's disk. The agent
/// needs the card, not the shape of their home directory.
pub(super) fn kept_here(body: &str) -> String {
    unpathed_body(body)
}

pub(super) const UNPATHED: &str =
    "A path on this machine reads as [path] here; the person sees it whole in the window.";

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

/// A closed task's dates no longer press, so it is never handed a standing that would read as urgent.
pub(super) fn standing_of(task: &Task, today: jiff::civil::Date) -> Option<Standing> {
    let how = tisty_core::standing::standing(task, today);
    (task.is_open() && how != Standing::Undated).then_some(how)
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
