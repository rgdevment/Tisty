use serde_json::{Value, json};
use tisty_core::model::TaskId;
use tisty_core::{Paths, State, Store};

use super::jsonrpc::{speaking_through, told};
use super::{Refused, hitch};

// The door turns a task's file away before any tool runs, so its page is named here by the task's id.
pub(super) fn task_named(state: &State, which: &str) -> Option<TaskId> {
    which
        .parse::<TaskId>()
        .ok()
        .filter(|id| state.tasks.contains_key(id))
}

pub(super) fn noted_on_page(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    id: TaskId,
    body: &str,
    under: Option<&str>,
    replace: bool,
) -> Result<Value, Refused> {
    let Some(task) = state.tasks.get(&id) else {
        return Err(Refused::Tool("that task is not here any more.".into()));
    };
    if task.hidden {
        return Err(Refused::Tool(format!(
            "no task here has the id {id}. Look it up again with `find`."
        )));
    }
    let title = task.title.clone();
    if !task.is_open() {
        return Err(Refused::Tool(format!(
            "{title:?} is closed, and its page is put away with it. If the work came back, \
             propose it anew."
        )));
    }
    let owner = tisty_core::task_docs::owner(state, id);
    if tisty_core::task_docs::of(state, owner).is_some_and(|kept| state.shut(kept.id)) {
        return Err(Refused::Tool(format!(
            "{title:?}'s page is locked. The person shut it so nothing writes in it."
        )));
    }
    let config = tisty_core::Config::load_or_init(paths).map_err(hitch)?;
    let spanish = tisty_core::model::spoken(config.locale.as_deref())
        .to_lowercase()
        .starts_with("es");
    let day = jiff::Zoned::now().strftime("%Y-%m-%d %H:%M");
    let who = speaking_through()
        .map(|client| tisty_core::agent::client_named(&client))
        .unwrap_or_else(|| "assistant".to_string());
    let note = format!("**{day}** · {who}\n{}", body.trim());
    let page = tisty_core::task_docs::of(state, owner).map(|kept| kept.file.clone());
    let folded = match replace {
        true => folding(paths, page.as_deref(), &title, body, &note, spanish)?,
        false => None,
    };
    if let (Some(was), Some(file)) = (&folded, &page) {
        tisty_core::docs::keep_notes_before(paths.data(), file, &was.slot).map_err(hitch)?;
    }
    let elsewhere = match under {
        Some(_) => {
            " It went under the assistant's notes: a task's page keeps the rest for the person."
        }
        None => "",
    };
    let known = page
        .as_deref()
        .and_then(|file| tisty_core::docs::read_notes_left(paths.data(), file));
    let mut left = None;
    let written = tisty_core::task_docs::wrote(paths, store, id, |was, _| {
        let now = match replace {
            true => tisty_core::assistant_notes::rewritten(was, &note, spanish),
            false => tisty_core::assistant_notes::noted(was, &note, spanish),
        };
        left = tisty_core::assistant_notes::left_by_assistants(was, &now, known.as_deref());
        Some(now)
    });
    if written.is_ok() {
        notes_left(paths, owner, page.as_deref(), left.as_deref());
        let mut kept = json!({ "task": id.to_string(), "title": title, "replaced": replace });
        let said = match &folded {
            Some(was) => {
                let before = was.slot.before.chars().count();
                kept["before"] = json!(before);
                kept["after"] = json!(note.chars().count());
                kept["folded"] = json!(was.by.len());
                kept["by"] = json!(was.by.iter().collect::<std::collections::BTreeSet<_>>());
                format!(
                    "Wrote the assistant's notes on {title:?}'s page again as one digest, from \
                     {before} to {} characters, folding {} dated entries by {}; the rest of the \
                     page is as it was. While the digest heads the notes, `restore_doc` with the \
                     task's id puts back the notes it replaced, even after later ticks and \
                     notes.{elsewhere}",
                    note.chars().count(),
                    was.by.len(),
                    named_hands(&was.by)
                )
            }
            None if replace => format!(
                "There were no assistant's notes on {title:?}'s page yet, so this one opens them.{elsewhere}"
            ),
            None => format!("Added under the assistant's notes on {title:?}'s page.{elsewhere}"),
        };
        return Ok(told(said, kept));
    }
    // A page that cannot take it right now leaves the note in the History, never nowhere.
    let length = super::chores::note_length(body);
    if length > super::chores::NOTE_AT_MOST {
        return Err(Refused::Tool(format!(
            "{title:?}'s page could not be written just now, and at {length} characters this \
             is too long for its History, so nothing was written. Try `append_doc` again in \
             a moment, or keep it in a document of its own with `write_doc`."
        )));
    }
    let zone = jiff::tz::TimeZone::system().iana_name().map(str::to_string);
    let d = tisty_core::event::LogAdd::new(ulid::Ulid::generate(), body.trim()).in_zone(zone);
    store
        .append(tisty_core::Op::TaskLog { id, d })
        .map_err(hitch)?;
    Ok(told(
        format!(
            "{title:?}'s page could not be written just now, so the note is in its History, \
             signed, instead. Nothing needs doing again."
        ),
        json!({ "task": id.to_string(), "title": title, "in": "history" }),
    ))
}

pub(super) const DIGEST_AT_MOST: usize = 6_000;
const THEIRS_SHOWN: usize = 5;

struct Folded {
    slot: tisty_core::docs::NotesBefore,
    by: Vec<String>,
}

fn folding(
    paths: &Paths,
    page: Option<&str>,
    title: &str,
    digest: &str,
    note: &str,
    spanish: bool,
) -> Result<Option<Folded>, Refused> {
    let length = digest.trim().chars().count();
    if length > DIGEST_AT_MOST {
        return Err(Refused::Tool(format!(
            "a digest is the state, the decisions, what was ruled out and the references, in at \
             most {DIGEST_AT_MOST} characters, and this one has {length}. Nothing was written. \
             Fold only what still matters; a long analysis worth keeping goes in a document of \
             its own."
        )));
    }
    let Some(file) = page else {
        return Ok(None);
    };
    let Ok(body) = tisty_core::docs::read(&paths.docs(), file) else {
        return Ok(None);
    };
    let Some(words) = tisty_core::assistant_notes::section(&body) else {
        return Ok(None);
    };
    let left = tisty_core::docs::read_notes_left(paths.data(), file);
    let theirs: Vec<String> = tisty_core::assistant_notes::foreign(words, left.as_deref())
        .into_iter()
        .filter(|line| !digest.contains(line.as_str()))
        .collect();
    if !theirs.is_empty() {
        return Err(Refused::Tool(format!(
            "the assistant's notes on {title:?}'s page hold words no assistant wrote, likely the \
             person's: {}. A digest would write over them, so nothing was written. Carry each of \
             those lines into the digest word for word, or leave the notes as they are and tell \
             the person.",
            quoted(&theirs)
        )));
    }
    let after = tisty_core::assistant_notes::rewritten(&body, note, spanish);
    Ok(Some(Folded {
        slot: tisty_core::docs::NotesBefore {
            before: words.to_string(),
            after: tisty_core::assistant_notes::section(&after)
                .unwrap_or_default()
                .to_string(),
            undone: false,
        },
        by: tisty_core::assistant_notes::signed(words),
    }))
}

fn quoted(lines: &[String]) -> String {
    let shown: Vec<String> = lines
        .iter()
        .take(THEIRS_SHOWN)
        .map(|line| format!("«{}»", line.chars().take(120).collect::<String>()))
        .collect();
    match lines.len().saturating_sub(THEIRS_SHOWN) {
        0 => shown.join(", "),
        more => format!("{} and {more} more", shown.join(", ")),
    }
}

// What an assistant left is what tells the person's words from theirs at the next digest.
fn notes_left(paths: &Paths, owner: TaskId, page: Option<&str>, words: Option<&str>) {
    let file = page.map(str::to_string).or_else(|| {
        let (state, _) = super::opened(paths).ok()?;
        tisty_core::task_docs::of(&state, owner).map(|kept| kept.file.clone())
    });
    if let (Some(file), Some(words)) = (file, words) {
        kept_left(paths, &file, words);
    }
}

pub(super) fn left_as_assistants(paths: &Paths, file: &str, was: &str, now: &str) {
    let known = tisty_core::docs::read_notes_left(paths.data(), file);
    if let Some(words) = tisty_core::assistant_notes::left_by_assistants(was, now, known.as_deref())
    {
        kept_left(paths, file, &words);
    }
}

fn kept_left(paths: &Paths, file: &str, words: &str) {
    if let Err(e) = tisty_core::docs::keep_notes_left(paths.data(), file, words) {
        tisty_core::witness::warn(
            tisty_core::witness::channel::DOOR,
            "the notes an assistant left could not be kept aside",
            &[("code", tisty_core::witness::Fact::Code(e.coded()))],
        );
    }
}

fn named_hands(by: &[String]) -> String {
    let hands: std::collections::BTreeSet<&str> = by.iter().map(String::as_str).collect();
    match hands.len() {
        0 => "nobody".to_string(),
        _ => hands.into_iter().collect::<Vec<_>>().join(" and "),
    }
}
