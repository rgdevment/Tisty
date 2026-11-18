use serde_json::{Map, Value, json};
use tisty_core::history::{Shown, Wrote};
use tisty_core::model::Kept;
use tisty_core::{Paths, State, Store};

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
                "`as` carries the document as it reads now, so it takes no `versions` or \
                 `version`. Read the version without `as`, or go back to it with `restore_doc` \
                 and carry the document then."
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
             what it says now and no version of it. `read_doc` without `versions` or `version` \
             reads it."
                .into(),
        )),
    }
}

struct Past {
    all: Vec<Shown>,
    days: u32,
}

fn past(paths: &Paths, state: &State, store: &Store, kept: &Kept) -> Result<Past, Refused> {
    let config = tisty_core::Config::load_or_init(paths).map_err(hitch)?;
    let days = config.history_days();
    let since = jiff::Timestamp::now()
        .checked_sub(jiff::SignedDuration::from_hours(24 * i64::from(days)))
        .unwrap_or(jiff::Timestamp::MIN);
    let log = store.read_all().map_err(hitch)?;
    let says = tisty_core::docs::read(&paths.docs(), &kept.file)
        .ok()
        .map(|body| tisty_core::attach::printed(body.as_bytes()));
    let all = tisty_core::history::shown(
        &tisty_core::history::versions(paths.data(), &kept.file),
        &tisty_core::history::wrote(&log, kept.id),
        |one| tisty_core::history::machine(state, one),
        &tisty_core::history::Here {
            machine: &config.device_id,
            says: says.as_deref(),
            since,
        },
    );
    Ok(Past { all, days })
}

fn short(print: &str) -> &str {
    print.get(..SHORT).unwrap_or(print)
}

fn hand(state: &State, wrote: &Wrote) -> String {
    let signed = signed_by_another(state, &wrote.by, wrote.signed.as_deref());
    signed.unwrap_or_else(
        || match (&wrote.via, state.assistants.contains(&wrote.by)) {
            (Some(via), _) => tisty_core::agent::client_named(via),
            (None, true) => "an assistant".to_string(),
            (None, false) => "the person".to_string(),
        },
    )
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
    bytes: Option<u64>,
    elsewhere: Option<String>,
}

impl Row {
    fn of(state: &State, one: &Shown) -> Self {
        Self {
            version: short(&one.print).to_string(),
            written: one.wrote.as_ref().map(|wrote| Written {
                at: when(wrote.at),
                by: hand(state, wrote),
            }),
            until: when(one.replaced),
            bytes: one.bytes,
            elsewhere: one.elsewhere.as_ref().map(|machine| {
                state
                    .named
                    .get(machine)
                    .map(|named| named.name.trim())
                    .filter(|name| !name.is_empty())
                    .map_or_else(|| "another machine".to_string(), str::to_string)
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
        if let Some(bytes) = self.bytes {
            into.insert("bytes".into(), json!(bytes));
        }
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
    let Past { all, days } = past(paths, state, store, kept)?;
    let named = doc_named(state, &kept.file);
    let from = args
        .get("cursor")
        .and_then(Value::as_u64)
        .map_or(0, |at| at as usize);
    let rows: Vec<Value> = all
        .iter()
        .skip(from)
        .take(LISTED_AT_ONCE)
        .map(|one| Row::of(state, one).told())
        .collect();
    let away = all.iter().filter(|one| one.elsewhere.is_some()).count();
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

    let said = match (all.len(), rows.is_empty()) {
        (0, _) => format!(
            "Nothing of what {named} said before is kept on this machine. A text becomes a \
             version when it is written over after half an hour untouched, or by someone else, \
             and stays {days} days; versions never leave the machine that kept them."
        ),
        (every, true) => format!(
            "No version of {named} is left past `cursor` {from}: the list holds {every} in all. \
             Ask `versions` again without `cursor` for the newest."
        ),
        (_, false) => format!(
            "{named} keeps {here} past version(s) on this machine, newest first{}. `read_doc` \
             with `version` reads one, and `restore_doc` with `version` goes back to it. \
             Versions never leave the machine that kept them, and stay {days} days.{}",
            match away {
                0 => String::new(),
                _ => format!(
                    "; another {away}, marked `elsewhere`, live on another machine alone: that \
                     text is not here, so it is neither read nor gone back to from this one"
                ),
            },
            match next {
                Some(next) => format!(" Older ones: `versions` again with `cursor` {next}."),
                None => String::new(),
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
    args["print"] = json!(version.print);
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
    pub print: String,
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
             `restore_doc` with this `version` goes back to it.",
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
    let Past { all, .. } = past(paths, state, store, kept)?;
    let wanted = asked.trim().to_lowercase();
    let mut alike = all.iter().filter(|one| one.print.starts_with(&wanted));
    let one = match (alike.next(), alike.next()) {
        (Some(one), None) => one,
        (Some(_), Some(_)) => {
            return Err(Refused::Tool(format!(
                "{asked:?} begins more than one version of {named}. Send the `version` whole, \
                 as `read_doc` with `versions` gives it."
            )));
        }
        (None, _) => {
            return Err(Refused::Tool(format!(
                "{named} keeps no version {asked:?}. `read_doc` with `versions` lists the ones \
                 it keeps; one past its days, or emptied by the person, is gone."
            )));
        }
    };
    let row = Row::of(state, one);
    if let Some(machine) = &row.elsewhere {
        return Err(Refused::Tool(format!(
            "that version of {named} is kept on {machine}, not on this machine, and versions \
             never leave the machine that kept them: its text is not here, so from here it is \
             neither read nor gone back to. Nothing was changed. Tell the person which machine \
             has it; `read_doc` with `versions` shows the ones this machine keeps, without \
             `elsewhere`."
        )));
    }
    let Some(body) = tisty_core::history::read(paths.data(), &kept.file, &one.print) else {
        return Err(Refused::Tool(format!(
            "version {} of {named} could not be read back from this machine's history: it was \
             taken out a moment ago, or it is not text.",
            row.version
        )));
    };
    Ok(Version {
        print: one.print.clone(),
        body,
        row,
    })
}
