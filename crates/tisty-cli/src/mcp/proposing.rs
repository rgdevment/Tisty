use serde_json::{Value, json};
use tisty_core::agents::{self, Unfit};
use tisty_core::event::{Op, Proposal};
use tisty_core::model::Kept;
use tisty_core::{Paths, State};

use super::asked::text;
use super::jsonrpc::told;
use super::{Refused, hitch, opened};

const TAKES: [(&str, &[&str]); 3] = [
    ("write_doc", &["doc", "body", "print"]),
    (
        "edit_doc",
        &[
            "doc", "old", "new", "section", "from", "to", "print", "echo",
        ],
    ),
    ("append_doc", &["doc", "body", "echo"]),
];

pub(super) fn takes(name: &str, args: &Value) -> bool {
    let Some((_, keys)) = TAKES.iter().find(|(tool, _)| *tool == name) else {
        return false;
    };
    args.as_object()
        .is_some_and(|all| all.keys().all(|key| keys.contains(&key.as_str())))
}

struct Asked<'a> {
    name: &'a str,
    args: &'a Value,
    named: String,
    approved: String,
    pending: Option<&'a tisty_core::model::Proposed>,
}

impl Asked<'_> {
    fn base(&self) -> &str {
        self.pending
            .map_or(self.approved.as_str(), |one| one.body.as_str())
    }

    fn printed(&self) -> String {
        tisty_core::docs::print_of_text(self.base())
    }
}

pub(super) fn proposed(
    paths: &Paths,
    name: &str,
    args: &Value,
    file: &str,
) -> Result<Value, Refused> {
    let (state, mut store) = opened(paths)?;
    let Some(kept) = state.docs.values().find(|one| one.file == file) else {
        return Err(Refused::Tool(super::mistaken::no_document_called(
            &state, file,
        )));
    };
    let named = super::doc_named(&state, file);
    if state.held_away(kept) || state.shut(kept.id) {
        return Err(Refused::Tool(format!(
            "{named} is put away or locked, so it takes no proposal. Tell the person what you \
             would change."
        )));
    }
    let approved = tisty_core::docs::read(&paths.docs(), file).map_err(hitch)?;
    let asked = Asked {
        name,
        args,
        named,
        approved,
        pending: state.proposals.get(&kept.id),
    };
    let proposal = made(&asked)?;
    agents::proposable(&asked.approved, &proposal).map_err(|why| unfit(&asked.named, why))?;
    let print = tisty_core::docs::print_of_text(&proposal);
    let fresh = proposal != asked.approved && asked.pending.is_none_or(|one| one.print != print);
    if fresh {
        store
            .append(Op::DocProposal {
                id: kept.id,
                d: Proposal {
                    body: Some(proposal.clone()),
                    base: Some(tisty_core::docs::print_of_text(&asked.approved)),
                    settles: None,
                },
            })
            .map_err(hitch)?;
    }
    Ok(answered(&state, kept, &asked, &proposal, print))
}

fn made(asked: &Asked) -> Result<String, Refused> {
    let base = asked.base();
    match asked.name {
        "write_doc" => {
            printed_as(asked)?;
            Ok(raw(asked.args, "body").unwrap_or_default().to_string())
        }
        "append_doc" => {
            let added = raw(asked.args, "body").unwrap_or_default();
            let kept = base.trim_end_matches('\n');
            Ok(format!("{kept}\n\n{}\n", added.trim_end_matches('\n')))
        }
        _ if raw(asked.args, "old").is_some() => passage_replaced(asked, base),
        _ => lines_replaced(asked, base),
    }
}

// A body Tisty stores with plain newlines matches a passage copied out of Windows once its returns go.
fn as_in(base: &str, said: &str) -> String {
    match base.contains('\r') {
        true => said.to_string(),
        false => said.replace('\r', ""),
    }
}

fn passage_replaced(asked: &Asked, base: &str) -> Result<String, Refused> {
    let old = as_in(base, raw(asked.args, "old").unwrap_or_default());
    let new = as_in(base, raw(asked.args, "new").unwrap_or_default());
    match base.matches(old.as_str()).count() {
        1 if !old.is_empty() => Ok(base.replacen(old.as_str(), &new, 1)),
        0 | 1 => Err(Refused::Tool(format!(
            "nothing in {} reads exactly like that `old`, so nothing was proposed. {}",
            asked.named,
            where_to_read(asked)
        ))),
        many => Err(Refused::Tool(format!(
            "that `old` fits {many} places in {}, so nothing was proposed. Send more of the \
             lines around the one you mean.",
            asked.named
        ))),
    }
}

fn lines_replaced(asked: &Asked, base: &str) -> Result<String, Refused> {
    printed_as(asked)?;
    let last = base.lines().count().max(1);
    let number = |key| {
        asked
            .args
            .get(key)
            .and_then(Value::as_u64)
            .map(|one| one as usize)
    };
    let (from, to) = match number("section") {
        Some(at) => tisty_core::docs::section_lines(base, at)
            .ok_or_else(|| Refused::Tool(format!("{} has no section {at}.", asked.named)))?,
        None => (
            number("from").unwrap_or(1).max(1),
            number("to").unwrap_or(last).min(last),
        ),
    };
    if from > to {
        return Err(Refused::Tool(format!(
            "`from` is line {from} and `to` is line {to}, so there is nothing between them."
        )));
    }
    let new = as_in(base, raw(asked.args, "new").unwrap_or_default());
    let tail = match new.is_empty() || new.ends_with('\n') {
        true => new,
        false => format!("{new}\n"),
    };
    Ok(format!(
        "{}{tail}{}",
        tisty_core::docs::lines_between(base, 1, from - 1),
        tisty_core::docs::lines_between(base, to + 1, last)
    ))
}

fn printed_as(asked: &Asked) -> Result<(), Refused> {
    let said = text(asked.args, "print").unwrap_or_default();
    if said == asked.printed() {
        return Ok(());
    }
    let told = match asked.pending {
        Some(one) if said == tisty_core::docs::print_of_text(&asked.approved) => format!(
            "that `print` is the approved text of {}, but a proposal already waits on top of it, \
             with the print {}. Build on that proposal: `read` with this `doc` hands it back.",
            asked.named, one.print
        ),
        _ => format!(
            "{} does not read as it did at that `print`, so nothing was proposed. {}",
            asked.named,
            where_to_read(asked)
        ),
    };
    Err(Refused::Tool(told))
}

fn where_to_read(asked: &Asked) -> &'static str {
    match asked.pending {
        Some(_) => {
            "A proposal already waits on it, and your change goes on top of that: `read` with this `doc` hands both back."
        }
        None => "Read it again with `read`.",
    }
}

fn unfit(named: &str, why: Unfit) -> Refused {
    let said = match why {
        Unfit::NoCard => "carries no `agent:` line, so it is no assistant's file",
        Unfit::Memory => "is an assistant's memory, which that assistant writes in its own file",
        Unfit::Retargeted => "would get another `agent:` line, which only the person changes",
        Unfit::TooLong => "would grow past what a proposal holds",
        Unfit::TooManyLines => "has more lines than the person can compare at once on its card",
        Unfit::NotWritable => "would no longer make a whole file: its code block must stay closed",
    };
    Refused::Tool(format!(
        "{named} {said}, so nothing was proposed. Tell the person what you would change and why."
    ))
}

pub(super) const WAITS: &str = "An assistant's change waits beside this approved text for the \
                                 person to approve or discard: `proposal` holds it, and your next \
                                 write to this document goes on top of it, at its `print`.";

pub(super) fn shown(waiting: &tisty_core::model::Proposed) -> Value {
    let chars = waiting.body.chars().count();
    let mut shown = json!({
        "print": waiting.print,
        "at": waiting.at.to_string(),
        "via": waiting.via,
        "chars": chars,
    });
    match chars <= super::WHOLE_UP_TO {
        true => shown["body"] = json!(waiting.body),
        false => shown["whole"] = json!(false),
    }
    shown
}

fn raw<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

fn answered(state: &State, kept: &Kept, asked: &Asked, proposal: &str, print: String) -> Value {
    let unchanged = proposal == asked.approved;
    let said = match unchanged {
        true => format!(
            "{} already reads like that, so nothing waits for the person.",
            asked.named
        ),
        false => format!(
            "Proposed that change to {}. It waits beside the approved text for the person, who \
             approves or discards it on the document's card in Tisty; no assistant's file \
             changes until they approve it. Your next write to this document goes on top of \
             this proposal, at this `print`.",
            asked.named
        ),
    };
    told(
        said,
        json!({
            "doc": kept.file,
            "title": super::title_of(state, &kept.file, kept.title.clone().unwrap_or_default()),
            "proposed": !unchanged,
            "print": print,
            "chars": proposal.chars().count(),
            "grew": super::grew(&asked.approved, proposal),
        }),
    )
}
