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
    let title = task.title.clone();
    if !task.is_open() {
        return Err(Refused::Tool(format!(
            "{title:?} is closed, and its page is put away with it. If the work came back, \
             propose it anew."
        )));
    }
    if !state.attended_by_agents(task) {
        return Err(Refused::Tool(format!(
            "{title:?} is the person's own: its page takes your words only once they open it to \
             agents. `note` keeps what you found in its History, signed."
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
        true => folding(paths, page.as_deref(), &title, body)?,
        false => None,
    };
    let elsewhere = match under {
        Some(_) => {
            " It went under the assistant's notes: a task's page keeps the rest for the person."
        }
        None => "",
    };
    let written = tisty_core::task_docs::wrote(paths, store, id, |was, _| {
        Some(match replace {
            true => tisty_core::assistant_notes::rewritten(was, &note, spanish),
            false => tisty_core::assistant_notes::noted(was, &note, spanish),
        })
    });
    if written.is_ok() {
        let mut kept = json!({ "task": id.to_string(), "title": title, "replaced": replace });
        let said = match (&folded, &page) {
            (Some(was), Some(file)) => {
                tisty_core::docs::keep_notes_before(paths.data(), file, &was.words)
                    .map_err(hitch)?;
                kept["before"] = json!(was.words.chars().count());
                kept["after"] = json!(note.chars().count());
                kept["folded"] = json!(was.by.len());
                kept["by"] = json!(was.by.iter().collect::<std::collections::BTreeSet<_>>());
                format!(
                    "Wrote the assistant's notes on {title:?}'s page again as one digest, from {} \
                     to {} characters, folding {} dated entries by {}; the rest of the page is as \
                     it was. `restore_doc` with the task's id puts the notes back as they were, \
                     even after later ticks and notes.{elsewhere}",
                    was.words.chars().count(),
                    note.chars().count(),
                    was.by.len(),
                    named_hands(&was.by)
                )
            }
            _ if replace => format!(
                "There were no assistant's notes on {title:?}'s page yet, so this one opens them.{elsewhere}"
            ),
            _ => format!("Added under the assistant's notes on {title:?}'s page.{elsewhere}"),
        };
        return Ok(told(said, kept));
    }
    // A page that cannot take it right now leaves the note in the History, never nowhere.
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

const DIGEST_AT_MOST: usize = 6_000;

struct Folded {
    words: String,
    by: Vec<String>,
}

fn folding(
    paths: &Paths,
    page: Option<&str>,
    title: &str,
    digest: &str,
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
    let (by, unsigned) = tisty_core::assistant_notes::signed(words);
    if unsigned {
        return Err(Refused::Tool(format!(
            "the assistant's notes on {title:?}'s page start with words no assistant signed, \
             likely the person's. A digest would write over them, so nothing was written. Leave \
             the notes as they are and tell the person, or `append_doc` without `replace`."
        )));
    }
    Ok(Some(Folded {
        words: words.to_string(),
        by,
    }))
}

fn named_hands(by: &[String]) -> String {
    let hands: std::collections::BTreeSet<&str> = by.iter().map(String::as_str).collect();
    match hands.len() {
        0 => "nobody".to_string(),
        _ => hands.into_iter().collect::<Vec<_>>().join(" and "),
    }
}
