use serde_json::{Value, json};
use tisty_core::model::{LogEntry, LogId, StepCount, Task, TaskId};
use tisty_core::task_state::{STATE, STATE_AT_MOST, Since};
use tisty_core::{Op, Paths, State};

use super::asked::{strings, text};
use super::chores::{already_said_done, reaching};
use super::jsonrpc::told;
use super::task_notes::assistant;
use super::task_reading::{Page, Window, clipped};
use super::{Refused, hitch, opened, when};

const CLOSING_SHOWN_UP_TO: usize = 1_000;
// Room for the signature over what `tick` takes, so only a state written by other means is cut.
const STATE_SHOWN_UP_TO: usize = STATE_AT_MOST + 200;

pub(super) fn tick(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(words) = text(args, "state") else {
        return super::chores::tick(paths, args);
    };
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("ticking needs a `task` id.".into()));
    };
    let (held, _) = opened(paths)?;
    let (id, task) = reaching(&held, &said)?;
    let title = task.title.clone();
    let (spanish, page) = writable(paths, &held, task, &words)?;
    let named_steps = text(args, "step").is_some() || !strings(args, "steps")?.is_empty();
    let ticked = match named_steps {
        true => Some(super::chores::tick(paths, args)?),
        false => None,
    };
    let named = match written(paths, id, &words, spanish, page) {
        Ok(named) => named,
        Err(why) => {
            return Err(Refused::Tool(match ticked {
                Some(_) => format!(
                    "the steps were ticked on {title:?}, but its state could not be written just now \
                 ({why}). Send `tick` again with only `state`."
                ),
                None => format!(
                    "the state of {title:?} could not be written just now ({why}). Nothing changed."
                ),
            }));
        }
    };
    let said = format!("Left its state on its page, under «{named}».");
    Ok(match ticked {
        Some(answer) => stated_too(answer, &said),
        None => {
            let left: Vec<&str> = task
                .steps
                .iter()
                .filter(|step| !step.done && !step.said)
                .map(|step| step.text.as_str())
                .collect();
            told(
                format!("{said} {} step(s) of {title:?} still unticked.", left.len()),
                json!({
                    "id": id.to_string(),
                    "title": title,
                    "stated": true,
                    "left": left.len(),
                    "next": left.first(),
                }),
            )
        }
    })
}

fn heading(spanish: bool) -> &'static str {
    STATE[usize::from(spanish)].trim_start_matches("## ")
}

fn stated_too(mut answer: Value, said: &str) -> Value {
    for at in ["/content/0/text", "/structuredContent/said"] {
        if let Some(Value::String(text)) = answer.pointer_mut(at) {
            text.push(' ');
            text.push_str(said);
        }
    }
    answer["structuredContent"]["stated"] = json!(true);
    answer
}

// Judged before any step is ticked, so a state refused leaves the steps as they were.
fn writable(
    paths: &Paths,
    state: &State,
    task: &Task,
    words: &str,
) -> Result<(bool, Option<String>), Refused> {
    let title = &task.title;
    if let Some(refusal) = already_said_done(task, "leaving its state") {
        return Err(refusal);
    }
    if task.repeat.is_some() || task.after.is_some() {
        return Err(Refused::Tool(format!(
            "{title:?} comes back as a routine, and every round shares one page, so it keeps no \
             state: `tick` without `state`, and what was different this round goes in `note`."
        )));
    }
    let length = words.chars().count();
    if length > STATE_AT_MOST {
        return Err(Refused::Tool(format!(
            "a state says what stands now — what is done, what is left, the decisions in force, \
             what was ruled out — in at most {STATE_AT_MOST} characters, and this one has \
             {length}. Nothing was ticked or written. Put the long part on the task's page with \
             `append_doc`, or in a document of its own, and say in the state where it is."
        )));
    }
    let owner = tisty_core::task_docs::owner(state, task.id);
    let page = tisty_core::task_docs::of(state, owner);
    if page.is_some_and(|kept| state.shut(kept.id)) {
        return Err(Refused::Tool(format!(
            "{title:?}'s page is locked. The person shut it so nothing writes in it; nothing was \
             ticked. `tick` without `state`, and tell the person what stands."
        )));
    }
    let body = page.and_then(|kept| {
        let body = tisty_core::docs::read(&paths.docs(), &kept.file).ok()?;
        Some((kept.file.clone(), body))
    });
    if let Some((file, body)) = &body {
        let theirs = left_out(paths, file, body, words);
        if !theirs.is_empty() {
            return Err(Refused::Tool(format!(
                "the state on {title:?}'s page holds words no assistant wrote, likely the \
                 person's: {}. A new state would write over them, so nothing was ticked or \
                 written. Carry each of those lines into `state` word for word, or leave it and \
                 tell the person.",
                theirs
                    .iter()
                    .take(5)
                    .map(|line| format!("«{}»", line.chars().take(120).collect::<String>()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
    }
    let named = body
        .as_ref()
        .and_then(|(_, body)| tisty_core::task_state::heading_in(body));
    let file = body.map(|(file, _)| file);
    if let Some(named) = named {
        return Ok((named == heading(true), file));
    }
    let config = tisty_core::Config::load_or_init(paths).map_err(hitch)?;
    let spanish = tisty_core::model::spoken(config.locale.as_deref())
        .to_lowercase()
        .starts_with("es");
    Ok((spanish, file))
}

fn written(
    paths: &Paths,
    id: TaskId,
    words: &str,
    spanish: bool,
    page: Option<String>,
) -> Result<&'static str, String> {
    let (_, mut store) = opened(paths).map_err(|_| "the store could not be opened".to_string())?;
    let signed =
        tisty_core::assistant_notes::signed_by(&assistant(), &jiff::ZonedDateTime::now(), words);
    let mut sides = None;
    let mut written_over = false;
    tisty_core::task_docs::noted_beside(paths, &mut store, id, |was, _| {
        // The person may have written in the state since it was judged, so it is judged again.
        written_over = page
            .as_deref()
            .is_some_and(|file| !left_out(paths, file, was, words).is_empty());
        if written_over {
            return None;
        }
        let now = tisty_core::task_state::stated(was, &signed, spanish);
        sides = Some((was.to_string(), now.clone()));
        Some(now)
    })
    .map_err(|e| e.to_string())?;
    if written_over {
        return Err("the person wrote in it meanwhile; `read` it and carry their lines".into());
    }
    let Some((before, after)) = sides else {
        return Err("the task has no page to write on".into());
    };
    let named = tisty_core::task_state::heading_in(&after).unwrap_or(heading(spanish));
    let mut ops: Vec<Op> = tisty_core::paged::paged(id, &before, &after)
        .into_iter()
        .filter(
            |op| matches!(op, Op::TaskPaged { d, .. } if tisty_core::task_state::named(&d.section)),
        )
        .collect();
    // The same words again still say the state stands as of now, so the trail hears it every time.
    if ops.is_empty() {
        ops.push(Op::TaskPaged {
            id,
            d: tisty_core::event::Paged {
                section: named.to_string(),
                change: tisty_core::event::PageChange::Written,
            },
        });
    }
    store.append_batch(ops).map_err(|e| e.to_string())?;
    let file = page.or_else(|| {
        let (held, _) = opened(paths).ok()?;
        let owner = tisty_core::task_docs::owner(&held, id);
        tisty_core::task_docs::of(&held, owner).map(|kept| kept.file.clone())
    });
    if let Some(file) = file {
        left_as_assistant(paths, &file, &before, &after);
    }
    Ok(named)
}

fn left_out(paths: &Paths, file: &str, page: &str, words: &str) -> Vec<String> {
    let Some(was) = tisty_core::task_state::section(page) else {
        return Vec::new();
    };
    let left = tisty_core::docs::read_state_left(paths.data(), file);
    tisty_core::assistant_notes::foreign(was, left.as_deref())
        .into_iter()
        .filter(|line| !words.lines().any(|one| one.trim() == line))
        .collect()
}

pub(super) fn left_as_assistant(paths: &Paths, file: &str, was: &str, now: &str) {
    let Some(words) = tisty_core::task_state::section(now) else {
        return;
    };
    let known = tisty_core::docs::read_state_left(paths.data(), file);
    let left = tisty_core::assistant_notes::kept_by_assistants(
        tisty_core::task_state::section(was),
        words,
        known.as_deref(),
    );
    kept_left(paths, file, &left);
}

fn kept_left(paths: &Paths, file: &str, words: &str) {
    if let Err(e) = tisty_core::docs::keep_state_left(paths.data(), file, words) {
        tisty_core::witness::warn(
            tisty_core::witness::channel::DOOR,
            "the state an assistant left could not be kept aside",
            &[("code", tisty_core::witness::Fact::Code(e.coded()))],
        );
    }
}

pub(super) fn replacing(
    state: &State,
    task: &Task,
    args: &Value,
) -> Result<Option<LogId>, Refused> {
    let Some(said) = text(args, "replaces") else {
        return Ok(None);
    };
    let title = &task.title;
    let old = said
        .parse::<LogId>()
        .ok()
        .and_then(|id| task.entry(id))
        .filter(|one| !one.body.trim().is_empty());
    let Some(old) = old else {
        return Err(Refused::Tool(format!(
            "no note of {title:?} has the id {said}; nothing was noted. `read` gives each note's \
             id, and `notes` 0 brings them all."
        )));
    };
    let on = when(old.at);
    if task.replaced().contains(&old.id) {
        return Err(Refused::Tool(format!(
            "the note of {on} on {title:?} was replaced already; nothing was noted. Replace the \
             note that took its place."
        )));
    }
    if task
        .resolved
        .as_ref()
        .is_some_and(|said| said.entry == old.id)
    {
        return Err(Refused::Tool(format!(
            "the note of {on} is the one that said {title:?} is settled, and it stays while the \
             person decides; nothing was noted. `note` what changed without `replaces`."
        )));
    }
    if old
        .by
        .as_ref()
        .is_some_and(|by| !state.assistants.contains(by))
    {
        return Err(Refused::Tool(format!(
            "the note of {on} on {title:?} is the person's, and taking it back is theirs; nothing \
             was noted. `note` what you found without `replaces`, and tell them."
        )));
    }
    Ok(Some(old.id))
}

struct Written {
    body: Option<(String, bool)>,
    heading: &'static str,
    at: Option<jiff::Timestamp>,
    since: Option<Since>,
}

pub(super) struct Stated<'a> {
    task: &'a Task,
    written: Option<Written>,
    closing: Option<(&'a LogEntry, String, bool)>,
}

impl<'a> Stated<'a> {
    pub(super) fn of(paths: &Paths, state: &State, task: &'a Task, page: Option<&Page>) -> Self {
        let shown_whole = page.and_then(Page::whole).is_some();
        let read_apart = || {
            let owner = tisty_core::task_docs::owner(state, task.id);
            let kept = tisty_core::task_docs::of(state, owner)?;
            tisty_core::docs::read(&paths.docs(), &kept.file).ok()
        };
        let body = match page {
            Some(page) => Some(std::borrow::Cow::Borrowed(page.body.as_str())),
            None => read_apart().map(std::borrow::Cow::Owned),
        };
        let written = body.as_deref().and_then(|body| {
            let said = tisty_core::task_state::said(body)?;
            Some(Written {
                body: (!shown_whole).then(|| clipped(said, STATE_SHOWN_UP_TO)),
                heading: tisty_core::task_state::heading_in(body)?,
                at: task.stated_at,
                since: tisty_core::task_state::since(task),
            })
        });
        let closing = task
            .resolved
            .as_ref()
            .and_then(|said| task.entry(said.entry))
            .filter(|one| !one.body.trim().is_empty())
            .map(|one| {
                let (body, cut) = clipped(one.body.trim(), CLOSING_SHOWN_UP_TO);
                (one, body, cut)
            });
        Self {
            task,
            written,
            closing,
        }
    }

    pub(super) fn after(&self) -> Option<jiff::Timestamp> {
        self.written.as_ref().and_then(|one| one.at)
    }

    fn old(&self) -> Option<String> {
        let since = self.written.as_ref()?.since?;
        if !since.old() {
            return None;
        }
        Some(match (since.steps, since.notes) {
            (true, 0) => "older than the last change to its steps".into(),
            (false, n) => format!("older than {n} later note(s)"),
            (true, n) => format!("older than the last change to its steps and {n} later note(s)"),
        })
    }

    pub(super) fn structured(&self, whole: &mut Value, notes: &Window) {
        let mut kept = serde_json::Map::new();
        match &self.written {
            Some(one) => {
                if let Some(at) = one.at {
                    kept.insert("written".into(), json!(when(at)));
                }
                if let Some((body, cut)) = &one.body {
                    kept.insert("body".into(), json!(body));
                    if *cut {
                        kept.insert("cut".into(), json!(true));
                    }
                }
                if let Some(old) = self.old() {
                    kept.insert("old".into(), json!(old));
                }
            }
            None => {
                let count = self.task.step_count();
                if count.all > 0 {
                    kept.insert("built".into(), json!(true));
                    kept.insert("done".into(), json!(count.done));
                    kept.insert("of".into(), json!(count.all));
                }
                if count.said > 0 {
                    kept.insert("said_done".into(), json!(count.said));
                }
            }
        }
        if let Some((one, body, cut)) = &self.closing
            && !notes.holds(one.id)
        {
            let mut closing = json!({ "id": one.id.to_string(), "at": when(one.at), "body": body });
            if *cut {
                closing["cut"] = json!(true);
            }
            kept.insert("closing".into(), closing);
            if self.written.is_none() {
                kept.entry("built").or_insert(json!(true));
            }
        }
        if !kept.is_empty() {
            whole["state"] = Value::Object(kept);
        }
    }

    pub(super) fn told(&self, notes: &Window) -> String {
        let mut told = match &self.written {
            Some(one) => {
                let at = one
                    .at
                    .map(|at| format!(", written {}", when(at)))
                    .unwrap_or_default();
                let old = self
                    .old()
                    .map(|old| format!(" It is {old}."))
                    .unwrap_or_default();
                match &one.body {
                    Some((body, cut)) => format!(
                        "\n\nIts state{at}:{old}\n{body}{}",
                        if *cut {
                            "\n(cut; its section on the page brings it whole)"
                        } else {
                            ""
                        }
                    ),
                    None => format!(
                        "\n\nIts state is on its page, under «{}»{at}.{old}",
                        one.heading
                    ),
                }
            }
            None => match self.task.step_count() {
                StepCount { all: 0, .. } => String::new(),
                StepCount { done, said, all } => format!(
                    "\n\nNo state is written yet. Built from it: {done} of {all} steps done{}.",
                    super::brain::project::word_waiting(said)
                ),
            },
        };
        if let Some((one, body, cut)) = &self.closing
            && !notes.holds(one.id)
        {
            told.push_str(&format!(
                "\n\nThe note that said it settled ({}): {body}{}",
                when(one.at),
                if *cut { " (cut)" } else { "" }
            ));
        }
        told
    }
}

#[cfg(test)]
#[path = "task_state_test.rs"]
mod tests;
