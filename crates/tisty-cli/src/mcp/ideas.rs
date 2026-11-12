use serde_json::{Value, json};
use tisty_core::brainstorm::{self, IDEA_AT_MOST};
use tisty_core::event::{Flag, Idea};
use tisty_core::model::Kept;
use tisty_core::{Op, Paths, State, Store};

use super::asked::text;
use super::jsonrpc::{speaking_through, told};
use super::{Refused, doc_named, hitch, opened, retold};

const IDEAS_AT_ONCE: usize = 50;

pub(super) fn sheet<'a>(paths: &Paths, state: &'a State) -> Option<(&'a Kept, String)> {
    brainstorm::find(&paths.docs(), state, None)
}

pub(super) fn is_sheet(body: &str) -> bool {
    tisty_markdown::front::brainstorm(body)
}

pub(super) fn glance(paths: &Paths, state: &State) -> Option<(String, Value)> {
    let (kept, body) = sheet(paths, state)?;
    let waiting = brainstorm::ideas(&body);
    if waiting == 0 {
        return None;
    }
    let marked = brainstorm::marked(kept, &body).len();
    let marks = match marked {
        0 => String::new(),
        n => format!(", {n} of them marked and waiting for the person"),
    };
    Some((
        format!(
            "Brainstorm, {}, holds {waiting} idea(s) not yet sorted{marks}: `rules` «ideas» says \
             how to work them.",
            doc_named(state, &kept.file)
        ),
        json!({
            "doc": kept.file,
            "title": tisty_core::docs::titled(&body),
            "ideas": waiting,
            "marked": marked,
        }),
    ))
}

pub(super) fn about_sheet(
    kept: &Kept,
    body: &str,
    into: &mut serde_json::Map<String, Value>,
) -> String {
    let marked = brainstorm::marked(kept, body);
    let mut said = format!(
        "This is Brainstorm: {} idea(s), oldest first, each under the minute it came and its \
         colour. An idea is named `{}#` and its heading, without the hashes. `rules` «ideas» \
         says how to sort them.",
        brainstorm::ideas(body),
        kept.file
    );
    if !marked.is_empty() {
        said.push_str(&format!(
            " Marked and waiting for the person: {}.",
            marked
                .iter()
                .map(|one| format!("«{}»", one.idea.text))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        into.insert(
            "marked".into(),
            json!(
                marked
                    .iter()
                    .map(|one| json!({
                        "idea": one.idea.heading,
                        "reason": one.mark.reason,
                        "instead": one.mark.instead,
                        "said": one.mark.body,
                    }))
                    .collect::<Vec<_>>()
            ),
        );
    }
    into.insert("brainstorm".into(), json!(true));
    said
}

fn named_idea(which: &str) -> Option<(&str, &str)> {
    which
        .split_once('#')
        .map(|(doc, heading)| (doc.trim(), heading.trim().trim_start_matches("## ").trim()))
}

pub(super) fn names_an_idea(which: &str) -> bool {
    named_idea(which).is_some()
}

fn ideas_in(said: &str) -> Result<Vec<String>, Refused> {
    let mut each = Vec::new();
    for paragraph in said.replace('\r', "").split("\n\n") {
        let lines: Vec<&str> = paragraph
            .lines()
            .map(str::trim)
            .filter(|one| !one.is_empty())
            .collect();
        if lines.iter().any(|line| line.starts_with('#')) {
            return Err(Refused::Tool(
                "a heading on Brainstorm is the minute an idea came, and Tisty writes it. Send the \
                 ideas alone: one per paragraph, or one per item of a list. Nothing was written."
                    .into(),
            ));
        }
        match lines.iter().all(|line| listed(line).is_some()) {
            true => each.extend(
                lines
                    .iter()
                    .filter_map(|line| listed(line))
                    .map(str::to_string),
            ),
            false if !lines.is_empty() => each.push(lines.join("\n")),
            false => {}
        }
    }
    each.retain(|one| !one.is_empty());
    if each.is_empty() {
        return Err(Refused::Tool(
            "there is no idea in that `body`. Nothing was written.".into(),
        ));
    }
    if each.len() > IDEAS_AT_ONCE {
        return Err(Refused::Tool(format!(
            "{} ideas is more than the {IDEAS_AT_ONCE} one call takes. Send them in rounds. \
             Nothing was written.",
            each.len()
        )));
    }
    let long: Vec<String> = each
        .iter()
        .filter(|one| one.chars().count() > IDEA_AT_MOST)
        .map(|one| {
            format!(
                "«{}…» ({})",
                one.chars().take(40).collect::<String>(),
                one.chars().count()
            )
        })
        .collect();
    if !long.is_empty() {
        return Err(Refused::Tool(format!(
            "an idea fits in {IDEA_AT_MOST} characters, as in the window, and these are longer: \
             {}. Say each in fewer words, or keep the longer thought in a document of its own. \
             Nothing was written.",
            long.join(", ")
        )));
    }
    Ok(each)
}

fn listed(line: &str) -> Option<&str> {
    let rest = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .or_else(|| line.strip_prefix("+ "))
        .or_else(|| {
            let (number, rest) = line.split_once(". ")?;
            number.chars().all(|c| c.is_ascii_digit()).then_some(rest)
        })?;
    Some(rest.trim_start_matches("[ ] ").trim())
}

pub(super) fn jotted_on(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    kept: &Kept,
    said: &str,
    args: &Value,
) -> Result<Value, Refused> {
    let named = doc_named(state, &kept.file);
    if text(args, "under").is_some() {
        return Err(Refused::Tool(format!(
            "{named} is Brainstorm, where every idea goes at the end, dated as it comes. Send \
             them without `under`."
        )));
    }
    let each = ideas_in(said)?;
    let before = tisty_core::docs::read(&paths.docs(), &kept.file).map_err(hitch)?;
    let at = jiff::Zoned::now().datetime();
    let added = brainstorm::jotted_all(&before, &each, at, brainstorm::roll);
    let whole = tisty_core::docs::append(&paths.docs(), &kept.file, &added).map_err(hitch)?;
    // The ideas are already on the sheet: refusing here would have a dutiful retry add them twice.
    let _ = retold(state, store, &kept.file, &whole);
    Ok(told(
        format!(
            "Jotted {} idea(s) on {named}, dated {}, each on a note of its own colour. Nothing \
             that was there changed.",
            each.len(),
            at.strftime("%Y-%m-%d %H:%M")
        ),
        json!({
            "doc": kept.file,
            "jotted": each.len(),
            "ideas": brainstorm::ideas(&whole),
            "print": tisty_core::attach::printed(whole.as_bytes()),
        }),
    ))
}

fn taken_off(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    kept: &Kept,
    idea: &Idea,
) -> Option<()> {
    let whole = tisty_core::docs::amend(&paths.docs(), paths.data(), &kept.file, |was| {
        brainstorm::without(was, idea)
    })
    .ok()??;
    let _ = retold(state, store, &kept.file, &whole);
    Some(())
}

// Filing moves the idea, as the window's «File in a document» does, so the sheet holds only what waits.
pub(super) fn filed_off(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    said: &str,
    into: &str,
) -> Option<String> {
    let quoted = brainstorm::quoted(said)?;
    let (kept, body) = sheet(paths, state)?;
    if kept.file == into {
        return None;
    }
    let idea = brainstorm::ideas_on(&body)
        .into_iter()
        .find(|one| one.text == quoted)?;
    taken_off(paths, state, store, kept, &idea)?;
    Some(format!(
        " The idea it quotes left {}, as filing it from the window does.",
        doc_named(state, &kept.file)
    ))
}

// The idea becomes the task, as the window's «Turn into a task» makes it, so it waits nowhere twice.
pub(super) fn made_a_task(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    source: &str,
) -> Option<String> {
    let (doc, heading) = named_idea(source)?;
    let (kept, body) = sheet(paths, state)?;
    if kept.file != doc {
        return None;
    }
    let named = doc_named(state, &kept.file);
    let found: Vec<Idea> = brainstorm::ideas_on(&body)
        .into_iter()
        .filter(|one| one.heading == heading)
        .collect();
    match found.as_slice() {
        [idea] => taken_off(paths, state, store, kept, idea).map(|()| {
            format!(" The idea it came from left {named}, as turning it into a task from the window does.")
        }),
        [] => None,
        _ => Some(format!(
            " {} ideas on {named} carry that heading, so none was taken off: mark the one this \
             task came from with `flag_doc`, `moved` to it.",
            found.len()
        )),
    }
}

pub(super) fn keeps_every_idea(
    state: &State,
    which: &str,
    before: &str,
    after: &str,
) -> Result<(), Refused> {
    if !is_sheet(before) {
        return Ok(());
    }
    let named = doc_named(state, which);
    if !is_sheet(after) {
        return Err(Refused::Tool(format!(
            "{named} is the person's Brainstorm, and `brainstorm: on` in its properties is what \
             makes it so. Keep that line. Nothing was written."
        )));
    }
    let mut left: Vec<Idea> = brainstorm::ideas_on(after);
    let mut lost: Vec<Idea> = Vec::new();
    for one in brainstorm::ideas_on(before) {
        match left.iter().position(|kept| kept.heading == one.heading) {
            Some(at) => {
                left.remove(at);
            }
            None => lost.push(one),
        }
    }
    if let Some(gone) = lost.first() {
        return Err(Refused::Tool(format!(
            "{named} is the person's Brainstorm, and this write would take {} idea(s) off it, \
             «{}» first. Taking an idea off is theirs: mark it with `flag_doc`, `doc` \
             \"{which}#{}\", or file it in a document or a task, which takes it off as the \
             window does. Nothing was written.",
            lost.len(),
            gone.text,
            gone.heading
        )));
    }
    let now = brainstorm::ideas_on(after);
    if let Some(long) = now
        .iter()
        .find(|one| one.text.chars().count() > IDEA_AT_MOST)
    {
        return Err(Refused::Tool(format!(
            "an idea fits in {IDEA_AT_MOST} characters, as in the window, and «{}» would hold {}. \
             Say it in fewer words. Nothing was written.",
            long.heading,
            long.text.chars().count()
        )));
    }
    Ok(())
}

pub(super) fn whole_sheet_refused(state: &State, which: &str) -> Refused {
    Refused::Tool(format!(
        "{} is Brainstorm, where the person's ideas wait: marking it would let one click delete \
         them all. Mark the ideas that had their day instead, each as `{which}#` and its \
         heading. Archiving or deleting the whole sheet is the person's, from the window.",
        doc_named(state, which)
    ))
}

pub(super) fn flag_ideas(
    paths: &Paths,
    args: &Value,
    many: &[String],
    listed: bool,
    body: String,
    reason: Option<String>,
) -> Result<Value, Refused> {
    let (state, mut store) = opened(paths)?;
    let Some((kept, sheet_body)) = sheet(paths, &state) else {
        return Err(Refused::Tool(format!(
            "there is no Brainstorm here, so {:?} names no idea. A document is marked by its id \
             alone. Nothing was marked.",
            many[0]
        )));
    };
    let named = doc_named(&state, &kept.file);
    let on = brainstorm::ideas_on(&sheet_body);
    let mut marking: Vec<Idea> = Vec::new();
    let mut already: Vec<Idea> = Vec::new();
    for which in many {
        let idea = idea_on(&on, &kept.file, &named, which)?;
        match kept.ideas_flagged.iter().find(|one| one.idea == idea) {
            Some(had) if !listed => {
                return Err(Refused::Tool(format!(
                    "«{}» on {named} was already marked on {}, and the person has not looked yet. \
                     Marking it again would only say the same thing twice.",
                    idea.text,
                    super::when(had.mark.at)
                )));
            }
            Some(_) => already.push(idea),
            None if !marking.contains(&idea) => marking.push(idea),
            None => {}
        }
    }
    let instead = instead_of(&state, args, reason.as_deref(), &on, kept, &named, many)?;
    let at = jiff::Timestamp::now();
    let me = store.device().clone();
    let ops: Vec<Op> = marking
        .iter()
        .map(|idea| Op::IdeaFlag {
            id: kept.id,
            idea: idea.clone(),
            d: Flag::new(body.clone())
                .said_by(at, me.clone())
                .through(speaking_through())
                .because(reason.clone(), instead.clone()),
        })
        .collect();
    if !ops.is_empty() {
        store.append_batch(ops).map_err(hitch)?;
    }
    let quoted = |ideas: &[Idea]| {
        ideas
            .iter()
            .map(|one| format!("«{}»", one.text))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let skipped = match already.is_empty() {
        true => String::new(),
        false => format!(
            " Already marked and waiting for the person, so left as they were: {}.",
            quoted(&already)
        ),
    };
    let said = match marking.is_empty() {
        true => format!("Nothing new was marked.{skipped}"),
        false => format!(
            "Marked {} on {named} as {}. Untouched, each still reads the same: the person sees \
             the mark on the note, and deleting it or taking the mark off are theirs{}.{skipped}",
            quoted(&marking),
            reason_said(
                &state,
                &on,
                &kept.file,
                reason.as_deref(),
                instead.as_deref()
            ),
            match marking.len() {
                1 => "",
                _ => " — one click settles them all",
            }
        ),
    };
    Ok(told(
        said,
        json!({
            "doc": kept.file,
            "ideas": marking.iter().map(|one| &one.heading).collect::<Vec<_>>(),
            "flagged": !marking.is_empty(),
            "reason": reason,
            "instead": instead,
            "skipped": already.iter().map(|one| &one.heading).collect::<Vec<_>>(),
        }),
    ))
}

fn idea_on(on: &[Idea], sheet: &str, named: &str, which: &str) -> Result<Idea, Refused> {
    let Some((doc, heading)) = named_idea(which) else {
        return Err(Refused::Tool(format!(
            "{which:?} is a document, and this call marks ideas: mark documents in a call of \
             their own. Nothing was marked."
        )));
    };
    if doc != sheet {
        return Err(Refused::Tool(format!(
            "{which:?} names an idea of {doc:?}, but ideas live on Brainstorm, {named}: name each \
             as `{sheet}#` and its heading. Nothing was marked."
        )));
    }
    let found: Vec<&Idea> = on.iter().filter(|one| one.heading == heading).collect();
    match found.as_slice() {
        [one] => Ok((*one).clone()),
        [] => Err(Refused::Tool(format!(
            "no idea on {named} is headed {heading:?}. `read_doc` it: an idea's heading is the \
             line above its words, without the hashes. Nothing was marked."
        ))),
        _ => Err(Refused::Tool(format!(
            "{} ideas on {named} are headed {heading:?}, so a mark would be a guess. Ask the \
             person which one they mean. Nothing was marked.",
            found.len()
        ))),
    }
}

fn instead_of(
    state: &State,
    args: &Value,
    reason: Option<&str>,
    on: &[Idea],
    kept: &Kept,
    named: &str,
    many: &[String],
) -> Result<Option<String>, Refused> {
    let Some(instead) = text(args, "instead") else {
        return match reason.is_some_and(tisty_core::model::reason_points_elsewhere) {
            true => Err(Refused::Tool(format!(
                "an idea marked {} points at what holds it now: send `instead`, the id of that \
                 document or task, or `{}#` and the heading of the idea it repeats.",
                reason.unwrap_or_default(),
                kept.file
            ))),
            false => Ok(None),
        };
    };
    if names_an_idea(&instead) {
        if many.contains(&instead) {
            return Err(Refused::Tool(
                "`instead` names one of the ideas being marked; it has to be the one that stays."
                    .into(),
            ));
        }
        idea_on(on, &kept.file, named, &instead)?;
        return Ok(Some(instead));
    }
    super::papers::flagging::instead_of(state, args, reason, many).map(|_| Some(instead))
}

fn reason_said(
    state: &State,
    on: &[Idea],
    sheet: &str,
    reason: Option<&str>,
    instead: Option<&str>,
) -> String {
    let idea = instead
        .and_then(named_idea)
        .filter(|(doc, _)| *doc == sheet)
        .and_then(|(_, heading)| on.iter().find(|one| one.heading == heading));
    match (reason, idea) {
        (Some("duplicate_of"), Some(one)) => format!("a duplicate of the idea «{}»", one.text),
        _ => super::papers::flagging::reason_said(state, reason, instead),
    }
}
