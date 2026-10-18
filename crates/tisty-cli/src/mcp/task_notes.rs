use serde_json::{Value, json};
use tisty_core::model::TaskId;
use tisty_core::{Paths, State, Store};

use super::jsonrpc::{speaking_through, told};
use super::{Refused, hitch};

/// A task's page answers to its file and to the task's own id, which is what `read` hands out.
pub(super) fn task_named(state: &State, which: &str) -> Option<TaskId> {
    if let Some(kept) = state.docs.values().find(|one| one.file == which) {
        kept.task_of?;
        return tisty_core::task_docs::behind(state).get(which).copied();
    }
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
    let elsewhere = match under {
        Some(_) => {
            " It went under the assistant's notes: a task's page keeps the rest for the person."
        }
        None => "",
    };
    let written = tisty_core::task_docs::wrote(paths, store, id, |was, _| {
        Some(tisty_core::assistant_notes::noted(was, &note, spanish))
    });
    if written.is_ok() {
        return Ok(told(
            format!("Added under the assistant's notes on {title:?}'s page.{elsewhere}"),
            json!({ "task": id.to_string(), "title": title }),
        ));
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
