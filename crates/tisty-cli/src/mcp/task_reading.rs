use serde_json::{Value, json};
use tisty_core::model::{LogEntry, StepId};

use super::{Refused, when};

const NOTES_SHOWN: usize = 10;
const NOTES_AT_LEAST: usize = 3;
const JOURNAL_SHOWN_UP_TO: usize = 3_000;
const NOTE_CLIPPED_AT: usize = 1_000;
// Asked by count, a note comes as whole as `note` lets one be written.
const NOTE_ASKED_UP_TO: usize = super::chores::NOTE_AT_MOST;
// A task's page is read at every pickup, so it comes whole only while it is short.
const PAGE_WHOLE_UP_TO: usize = 4_000;
const NOTES_TO_COMPACT: usize = 8_000;
const LATEST_NOTE_AT_MOST: usize = 3_000;
// The digest is the task's state, so it arrives whole, with room for its signature.
const DIGEST_SHOWN_UP_TO: usize = super::task_notes::DIGEST_AT_MOST + 200;

pub(super) struct Page {
    pub(super) file: String,
    pub(super) body: String,
    pub(super) boxed: Vec<StepId>,
    outlined: bool,
    digest: Option<(String, bool)>,
    latest: Option<(String, bool)>,
    notes_held: usize,
    yours: bool,
}

impl Page {
    /// `digest` is what the last digest wrote, as kept on this machine to undo it.
    pub(super) fn new(
        file: String,
        body: String,
        boxed: Vec<StepId>,
        yours: bool,
        digest: Option<String>,
    ) -> Self {
        let outlined = body.chars().count() > PAGE_WHOLE_UP_TO;
        let notes = tisty_core::assistant_notes::section(&body);
        let notes_held = notes.map_or(0, |words| words.trim().chars().count());
        let shown = notes.filter(|_| outlined);
        let heading = digest.filter(|said| {
            !said.trim().is_empty()
                && shown.is_some_and(|words| words.trim_start().starts_with(said.trim()))
        });
        let latest = shown
            .and_then(tisty_core::assistant_notes::latest)
            .filter(|one| heading.as_deref().is_none_or(|said| said.trim() != *one))
            .map(|one| clipped(one, LATEST_NOTE_AT_MOST));
        Self {
            file,
            body,
            boxed,
            outlined,
            digest: heading.map(|said| clipped(said.trim(), DIGEST_SHOWN_UP_TO)),
            latest,
            notes_held,
            yours,
        }
    }

    fn wants_compacting(&self) -> bool {
        self.yours && self.notes_held > NOTES_TO_COMPACT
    }

    /// The page as the answer shows it whole, so what it holds need not be said again beside it.
    pub(super) fn whole(&self) -> Option<&Self> {
        (!self.outlined).then_some(self)
    }

    pub(super) fn structured(&self) -> Value {
        let mut kept = match self.outlined {
            false => json!({ "file": self.file, "body": self.body }),
            true => json!({
                "file": self.file,
                "whole": false,
                "outline": tisty_core::docs::outlined(&self.body),
            }),
        };
        if let Some((said, _)) = &self.digest {
            kept["digest"] = json!(said);
        }
        if let Some((said, _)) = &self.latest {
            kept["latest_note"] = json!(said);
        }
        if self.wants_compacting() {
            kept["notes_to_compact"] = json!(self.notes_held);
        }
        kept
    }

    pub(super) fn told(&self) -> String {
        let mut told = match self.outlined {
            false => format!("\n\nIts page:\n{}", self.body.trim_end()),
            true => format!(
                "\n\nIts page holds {} characters, so here are its sections rather than the whole \
                 of it: `read_doc` {:?} with a `section` brings one.\n{}",
                self.body.chars().count(),
                self.file,
                tisty_core::docs::outlined(&self.body)
                    .iter()
                    .map(|one| serde_json::to_string(one).unwrap_or_default())
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        };
        if let Some((said, cut)) = &self.digest {
            told.push_str("\n\nThe digest that opens the assistant's notes:\n");
            told.push_str(said);
            if *cut {
                told.push_str("\n(cut; the section brings it whole)");
            }
        }
        if let Some((said, cut)) = &self.latest {
            told.push_str("\n\nThe latest of the assistant's notes on it:\n");
            told.push_str(said);
            if *cut {
                told.push_str(&format!(
                    "\n(cut at {LATEST_NOTE_AT_MOST} characters; the section brings it whole)"
                ));
            }
        }
        if self.wants_compacting() {
            told.push_str(&format!(
                "\n\nThe assistant's notes on it hold {} characters: compact them when you leave \
                 off, as `rules` «tasks» says.",
                self.notes_held
            ));
        }
        told
    }
}

pub(super) struct Shown<'a> {
    one: &'a LogEntry,
    kept: String,
    cut: bool,
    by: Option<String>,
}

pub(super) struct Window<'a> {
    notes: Vec<Shown<'a>>,
    earlier: usize,
    hid: bool,
    asked: bool,
}

pub(super) fn widened(args: &Value) -> Result<Option<u64>, Refused> {
    match args.get("notes") {
        None | Some(Value::Null) => Ok(None),
        Some(said) => said.as_u64().map(Some).ok_or_else(|| {
            Refused::Tool(format!(
                "`notes` is how many of the latest journal notes to bring, or 0 for all of them, \
                 not {said}."
            ))
        }),
    }
}

pub(super) fn window<'a>(
    journal: &[&'a LogEntry],
    widened: Option<u64>,
    kept_here: impl Fn(&str) -> String,
    by: impl Fn(&LogEntry) -> Option<String>,
) -> Window<'a> {
    let earlier = match widened {
        Some(0) => 0,
        Some(latest) => journal.len().saturating_sub(latest as usize),
        None => journal.len() - within_the_window(journal),
    };
    let mut hid = false;
    let notes = journal[earlier..]
        .iter()
        .map(|one| {
            let kept = kept_here(&one.body);
            hid |= kept != one.body;
            let (kept, cut) = match widened {
                None => clipped(&kept, NOTE_CLIPPED_AT),
                Some(0) => (kept, false),
                Some(_) => clipped(&kept, NOTE_ASKED_UP_TO),
            };
            Shown {
                one,
                kept,
                cut,
                by: by(one),
            }
        })
        .collect();
    Window {
        notes,
        earlier,
        hid,
        asked: widened.is_some(),
    }
}

impl Window<'_> {
    pub(super) fn structured(&self, whole: &mut Value, unpathed: &str) {
        if self.notes.is_empty() {
            return;
        }
        whole["journal"] = json!(
            self.notes
                .iter()
                .map(|shown| {
                    let mut note = json!({
                        "id": shown.one.id.to_string(),
                        "at": when(shown.one.at),
                        "body": shown.kept,
                    });
                    if let Some(by) = &shown.by {
                        note["by"] = json!(by);
                    }
                    if shown.cut {
                        note["cut"] = json!(true);
                    }
                    note
                })
                .collect::<Vec<_>>()
        );
        if self.earlier > 0 {
            whole["earlier"] = json!(self.earlier);
        }
        if self.hid {
            whole["unpathed"] = json!(unpathed);
        }
    }

    pub(super) fn told(&self, unpathed: &str) -> String {
        let mut told = String::new();
        if self.earlier > 0 && !self.notes.is_empty() {
            told.push_str(&format!(
                "\n\n{} earlier note(s) left out; `notes` 0 brings them all.",
                self.earlier
            ));
        }
        for shown in &self.notes {
            let by = shown
                .by
                .as_ref()
                .map(|by| format!(", by {by}"))
                .unwrap_or_default();
            told.push_str(&format!(
                "\n\n({}, note {}{by}) {}",
                when(shown.one.at),
                shown.one.id,
                shown.kept
            ));
        }
        let past_the_cap = self
            .notes
            .iter()
            .any(|shown| shown.cut && shown.one.body.chars().count() > NOTE_ASKED_UP_TO);
        if self.notes.iter().any(|shown| shown.cut) {
            told.push_str(&match (self.asked, past_the_cap) {
                (false, false) => format!(
                    "\n\nNotes past {NOTE_CLIPPED_AT} characters are cut; `notes` with a count \
                     brings them whole."
                ),
                (false, true) => format!(
                    "\n\nNotes past {NOTE_CLIPPED_AT} characters are cut; `notes` with a count \
                     brings them up to {NOTE_ASKED_UP_TO} characters, and `notes` 0 brings every \
                     note whole."
                ),
                (true, _) => format!(
                    "\n\nNotes past {NOTE_ASKED_UP_TO} characters are cut; `notes` 0 brings \
                     every note whole."
                ),
            });
        }
        if self.hid {
            told.push_str(&format!("\n\n{unpathed}"));
        }
        told
    }
}

// The window is what a pickup pays for, so it is counted in characters as well as notes.
fn within_the_window(journal: &[&LogEntry]) -> usize {
    let mut spent = 0;
    let mut taken = 0;
    for one in journal.iter().rev().take(NOTES_SHOWN) {
        let size = one.body.chars().count().min(NOTE_CLIPPED_AT);
        if taken >= NOTES_AT_LEAST && spent + size > JOURNAL_SHOWN_UP_TO {
            break;
        }
        spent += size;
        taken += 1;
    }
    taken
}

fn clipped(said: &str, most: usize) -> (String, bool) {
    match said.chars().count() > most {
        true => (
            format!("{}…", said.chars().take(most).collect::<String>()),
            true,
        ),
        false => (said.to_string(), false),
    }
}
