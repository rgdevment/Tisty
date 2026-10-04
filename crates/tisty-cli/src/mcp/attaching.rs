use serde_json::{Value, json};
use tisty_core::Paths;
use tisty_core::State;
use tisty_core::event::{LogAdd, Op};
use tisty_core::model::TaskId;
use ulid::Ulid;

use super::asked::text;
use super::jsonrpc::told;
use super::{Refused, history, hitch, opened};

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

pub(super) fn attach(paths: &Paths, args: &Value) -> Result<Value, Refused> {
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
            store.append(kept.told()).map_err(hitch)?;
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
            if let Err(why) = store.append(kept.told()) {
                tisty_core::witness::warn(
                    tisty_core::witness::channel::ATTACH,
                    "what an attachment holds could not be written down",
                    &[("why", tisty_core::witness::Fact::Why(why.to_string()))],
                );
            }
            if super::retold(&state, &mut store, &which, &whole).is_err() {
                tisty_core::witness::warn(
                    tisty_core::witness::channel::ATTACH,
                    "what the document now holds could not be written down",
                    &[],
                );
            }
            let title = tisty_core::docs::titled(&whole);
            Ok(told(
                format!("Kept {named:?} at the end of {title:?}."),
                json!({ "doc": which, "title": title, "at": kept.at, "label": named }),
            ))
        }
    }
}
