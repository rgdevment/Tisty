use serde_json::{Map, Value, json};
use tisty_core::history::Machines;
use tisty_core::model::Kept;
use tisty_core::{DeviceId, Paths, State, Store};

use super::super::asked::text;
use super::super::jsonrpc::told;
use super::super::mistaken::named_task;
use super::super::resuming::signed_by_another;
use super::super::{Refused, doc_named, hitch, title_of, when, written_title};

const LISTED_AT_ONCE: usize = 40;
const SHORT: usize = 12;
const READ_BY_PARTS: [&str; 4] = ["section", "from", "to", "chars"];

pub(super) enum Asked {
    List,
    One(String),
}

impl Asked {
    pub(super) fn of(args: &Value) -> Result<Option<Self>, Refused> {
        let sent = |key: &str| args.get(key).is_some_and(|one| !one.is_null());
        let list = args.get("versions").and_then(Value::as_bool) == Some(true);
        let asked = match (list, text(args, "version")) {
            (true, Some(_)) => {
                return Err(Refused::Tool(
                    "`versions` lists what a document said before and `version` reads one of \
                     them: send one of the two."
                        .into(),
                ));
            }
            (true, None) => Self::List,
            (false, Some(one)) => Self::One(one),
            (false, None) => return Ok(None),
        };
        if sent("as") {
            return Err(Refused::Tool(
                "`view: \"markdown\"` carries the document as it reads now, so it takes no \
                 `versions` or `version`. Read the version with `view: \"content\"`, or go back \
                 to it with `doc_edit` (`mode: \"restore\"`) and carry the document then."
                    .into(),
            ));
        }
        if let (Self::List, Some(part)) = (&asked, READ_BY_PARTS.into_iter().find(|key| sent(key)))
        {
            return Err(Refused::Tool(format!(
                "`versions` lists what the document said before, so it takes no `{part}`: only \
                 `cursor`, for the older ones. Read one of them by parts with `version`."
            )));
        }
        Ok(Some(asked))
    }
}

/// A task's id stands for its page here, as it does for `restore_doc`.
pub(super) fn page_or_doc(state: &State, which: &str) -> Result<String, Refused> {
    let Some(task) = named_task(state, which) else {
        return Ok(which.to_string());
    };
    let owner = tisty_core::task_docs::owner(state, task.id);
    match tisty_core::task_docs::of(state, owner) {
        Some(page) => Ok(page.file.clone()),
        None => Err(Refused::Tool(format!(
            "{:?} has no page yet, so nothing it said was ever written over.",
            task.title
        ))),
    }
}

// What the person took out of their diary stays out of an assistant's reach, open or not.
pub(super) fn not_the_diary(paths: &Paths, state: &State, kept: &Kept) -> Result<(), Refused> {
    match super::super::diary::in_the_diary(paths, state, &kept.file) {
        false => Ok(()),
        true => Err(Refused::Tool(
            "the diary is the person's own, and so is what it said before: an assistant reads \
             what it says now and no version of it. `workspace_read` without `versions` or \
             `version` reads it."
                .into(),
        )),
    }
}

struct Past {
    all: Vec<tisty_core::history::Row>,
    days: u32,
    says: Option<String>,
    machines: Machines,
}

fn past(paths: &Paths, state: &State, store: &Store, kept: &Kept) -> Result<Past, Refused> {
    let config = tisty_core::Config::load_or_init(paths).map_err(hitch)?;
    let days = config.history_days();
    let log = store.read_all().map_err(hitch)?;
    let says = tisty_core::docs::read(&paths.docs(), &kept.file)
        .ok()
        .map(|body| tisty_core::docs::print_of_text(&body));
    let machines = Machines::of(state, &config.device_id, config.agent_id.as_ref()).told_by(&log);
    let all = tisty_core::history::rows(
        paths.data(),
        &kept.file,
        &tisty_core::history::logged(&log, kept.id),
        days,
        jiff::Timestamp::now(),
        |device| machines.elsewhere(device),
    );
    Ok(Past {
        all,
        days,
        says,
        machines,
    })
}

fn gone_back_at(paths: &Paths, doc: &str) -> Option<std::path::PathBuf> {
    tisty_core::docs::resolve(&paths.data().join(tisty_core::history::GONE_BACK), doc).ok()
}

pub(super) fn went_back(paths: &Paths, doc: &str, from: &str, to: &str) {
    let Some(at) = gone_back_at(paths, doc) else {
        return;
    };
    let prints = format!(
        "{}\n{}\n",
        tisty_core::docs::print_of_text(from),
        tisty_core::docs::print_of_text(to)
    );
    let kept = at
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .map_err(tisty_core::Error::Io)
        .and_then(|()| tisty_core::store::write_atomic(&at, prints.as_bytes()));
    if let Err(e) = kept {
        tisty_core::witness::warn(
            tisty_core::witness::channel::DOOR,
            "a going back to a version could not be noted, so its undo may be taken for another",
            &[("why", tisty_core::witness::Fact::Why(e.to_string()))],
        );
    }
}

// A page write is settled after the call, so it is noted once its final body is known.
pub(super) fn noted_if_gone_back(paths: &Paths, doc: &str, before: &str, said: &Value) {
    if said.pointer("/structuredContent/went_back_to").is_some()
        && let Ok(now) = tisty_core::docs::read(&paths.docs(), doc)
    {
        went_back(paths, doc, before, &now);
    }
}

// Only the two bodies a going back moved between, the one standing and the one a step back, are its undo.
pub(super) fn undoes_a_going_back(
    paths: &Paths,
    doc: &str,
    now: &str,
) -> Option<tisty_core::history::Before> {
    let marked: Vec<String> = gone_back_at(paths, doc)
        .and_then(|at| std::fs::read_to_string(at).ok())?
        .lines()
        .map(str::to_string)
        .collect();
    let back = tisty_core::history::back(paths.data(), doc)?;
    let (standing, behind) = (
        tisty_core::docs::print_of_text(now),
        tisty_core::docs::print_of_text(&back.body),
    );
    let pair = marked == [standing.as_str(), behind.as_str()]
        || marked == [behind.as_str(), standing.as_str()];
    (standing != behind && pair && back.left.as_deref() == Some(standing.as_str())).then_some(back)
}

fn short(print: &str) -> &str {
    print.get(..SHORT).unwrap_or(print)
}

fn hand(state: &State, by: &DeviceId, via: Option<&str>, signed: Option<&str>) -> String {
    signed_by_another(state, by, signed).unwrap_or_else(|| {
        match (via, state.assistants.contains(by)) {
            (Some(via), _) => tisty_core::agent::client_named(via),
            (None, true) => "an assistant".to_string(),
            (None, false) => "the person".to_string(),
        }
    })
}

fn machine_named(state: &State, machines: &Machines, device: &DeviceId) -> String {
    machines
        .machine_of(device)
        .and_then(|one| state.named.get(&one))
        .map(|named| named.name.trim())
        .filter(|name| !name.is_empty())
        .map_or_else(|| "another machine".to_string(), str::to_string)
}

struct Written {
    at: String,
    by: String,
}

/// One version as an answer tells it: its hours already said as the person reads them.
pub(super) struct Row {
    version: String,
    written: Option<Written>,
    until: String,
    bytes: u64,
    elsewhere: Option<String>,
}

impl Row {
    fn of(state: &State, machines: &Machines, one: &tisty_core::history::Row) -> Self {
        Self {
            version: short(&one.print).to_string(),
            written: one.wrote.zip(one.device.as_ref()).map(|(at, by)| Written {
                at: when(at),
                by: hand(state, by, one.via.as_deref(), one.signed.as_deref()),
            }),
            until: when(one.replaced),
            bytes: one.bytes,
            elsewhere: (!one.here).then(|| {
                one.kept_by.as_ref().map_or_else(
                    || "another machine".to_string(),
                    |by| machine_named(state, machines, by),
                )
            }),
        }
    }

    pub(super) fn told_into(&self, into: &mut Map<String, Value>) {
        into.insert("version".into(), json!(self.version));
        if let Some(written) = &self.written {
            into.insert("written".into(), json!(written.at));
            into.insert("by".into(), json!(written.by));
        }
        into.insert("until".into(), json!(self.until));
        into.insert("bytes".into(), json!(self.bytes));
        if let Some(machine) = &self.elsewhere {
            into.insert("elsewhere".into(), json!(machine));
        }
    }

    fn told(&self) -> Value {
        let mut row = Map::new();
        self.told_into(&mut row);
        Value::Object(row)
    }
}

pub(super) fn listed(
    paths: &Paths,
    state: &State,
    store: &Store,
    kept: &Kept,
    args: &Value,
) -> Result<Value, Refused> {
    let Past {
        all,
        days,
        machines,
        ..
    } = past(paths, state, store, kept)?;
    let named = doc_named(state, &kept.file);
    let from = args
        .get("cursor")
        .and_then(Value::as_u64)
        .map_or(0, |at| at as usize);
    let rows: Vec<Value> = all
        .iter()
        .skip(from)
        .take(LISTED_AT_ONCE)
        .map(|one| Row::of(state, &machines, one).told())
        .collect();
    let away = all.iter().filter(|one| !one.here).count();
    let here = all.len() - away;
    let next = (from + rows.len() < all.len()).then_some(from + rows.len());

    let mut kept_of = Map::new();
    kept_of.insert("doc".into(), json!(kept.file));
    kept_of.insert(
        "title".into(),
        json!(title_of(
            state,
            &kept.file,
            kept.title.clone().unwrap_or_default()
        )),
    );
    kept_of.insert("kept_here".into(), json!(here));
    if away > 0 {
        kept_of.insert("kept_elsewhere".into(), json!(away));
    }
    kept_of.insert("kept_days".into(), json!(days));
    kept_of.insert("versions".into(), json!(rows));
    if let Some(next) = next {
        kept_of.insert("next".into(), json!(next));
    }

    let elsewhere = format!(
        "{away} marked `elsewhere` were written over on another of the person's machines, which \
         alone may keep them: their text is not here, so from this one they are neither read nor \
         gone back to"
    );
    let older = match next {
        Some(next) => format!(" Older ones: `versions` again with `cursor` {next}."),
        None => String::new(),
    };
    let said = match (all.len(), rows.is_empty(), here) {
        (0, _, _) => format!(
            "Nothing of what {named} said before is kept on this machine. A text becomes a \
             version when it is written over after half an hour untouched, or by someone else, \
             and stays {days} days; versions never leave the machine that kept them."
        ),
        (every, true, _) => format!(
            "No version of {named} is left past `cursor` {from}: the list holds {every} in all. \
             Ask `versions` again without `cursor` for the newest."
        ),
        (_, false, 0) => format!(
            "Nothing of what {named} said before is kept on this machine; {elsewhere}. Versions \
             never leave the machine that kept them.{older}"
        ),
        (_, false, _) => format!(
            "{named} keeps {here} past version(s) on this machine, newest first{}. \
             `workspace_read` with `version` reads one, and `doc_edit` (`mode: \"restore\"`) with \
             `version` goes back to it. Versions never leave the machine that kept them, and stay \
             {days} days.{older}",
            match away {
                0 => String::new(),
                _ => format!("; {elsewhere}"),
            }
        ),
    };
    Ok(told(said, Value::Object(kept_of)))
}

pub(super) fn read(
    paths: &Paths,
    state: &State,
    store: &Store,
    kept: &Kept,
    asked: &str,
    args: &Value,
) -> Result<Value, Refused> {
    let version = kept_here(paths, state, store, kept, asked)?;
    let notice = version.notice(&doc_named(state, &kept.file));
    let title = written_title(state, &kept.file, &version.body);
    let mut kept_of = Map::new();
    kept_of.insert("doc".into(), json!(kept.file));
    kept_of.insert("title".into(), json!(title));
    version.row.told_into(&mut kept_of);
    kept_of.insert("notice".into(), json!(notice));
    kept_of.insert("chars".into(), json!(version.body.chars().count()));
    kept_of.insert("lines".into(), json!(version.body.lines().count()));
    // A version never changes under a cursor, so its parts need no print sent back to join them.
    let mut args = args.clone();
    args["print"] = json!(tisty_core::attach::printed(version.body.as_bytes()));
    super::reading::by_parts(
        super::reading::Reading {
            body: &version.body,
            title,
            warned: format!("({notice})\n\n"),
            past: true,
        },
        &args,
        kept_of,
    )
}

pub(super) struct Version {
    pub body: String,
    pub row: Row,
}

impl Version {
    pub(super) fn short(&self) -> &str {
        &self.row.version
    }

    fn notice(&self, named: &str) -> String {
        format!(
            "This is a past version of {named}, not what it says now: what it said until {}{}. \
             `doc_edit` (`mode: \"restore\"`) with this `version` goes back to it.",
            self.row.until,
            match &self.row.written {
                Some(written) => format!(", written by {} on {}", written.by, written.at),
                None => String::new(),
            }
        )
    }
}

/// The one version `asked` names, when this machine keeps its text; otherwise why it cannot be had.
pub(super) fn kept_here(
    paths: &Paths,
    state: &State,
    store: &Store,
    kept: &Kept,
    asked: &str,
) -> Result<Version, Refused> {
    let named = doc_named(state, &kept.file);
    let Past {
        all,
        says,
        machines,
        ..
    } = past(paths, state, store, kept)?;
    let wanted = asked.trim().to_lowercase();
    let mut alike = all.iter().filter(|one| one.print.starts_with(&wanted));
    let one = match (alike.next(), alike.next()) {
        (Some(one), None) => one,
        (Some(_), Some(_)) => {
            return Err(Refused::Tool(format!(
                "{asked:?} begins more than one version of {named}. Send the `version` whole, \
                 as `read` with `versions` gives it."
            )));
        }
        (None, _) if says.is_some_and(|now| now.starts_with(&wanted)) => {
            return Err(Refused::Tool(format!(
                "{named} already says what version {asked:?} says, so it is no past version: \
                 `read` without `version` reads it, and there is nothing to go back to."
            )));
        }
        (None, _) => {
            return Err(Refused::Tool(format!(
                "{named} keeps no version {asked:?}. `read` with `versions` lists the ones \
                 it keeps; one past its days, or emptied by the person, is gone."
            )));
        }
    };
    let row = Row::of(state, &machines, one);
    if let Some(machine) = &row.elsewhere {
        return Err(Refused::Tool(format!(
            "that version of {named} was written over on {machine}, not on this machine, and \
             versions never leave the machine that kept them: its text is not here, so from \
             here it is neither read nor gone back to. Nothing was changed. Tell the person \
             which machine may keep it; `read` with `versions` shows the ones this machine \
             keeps, without `elsewhere`."
        )));
    }
    let Some(body) = tisty_core::history::read(paths.data(), &kept.file, &one.print) else {
        return Err(Refused::Tool(format!(
            "version {} of {named} could not be read back from this machine's history: it was \
             taken out a moment ago, or it is not text.",
            row.version
        )));
    };
    Ok(Version { body, row })
}
