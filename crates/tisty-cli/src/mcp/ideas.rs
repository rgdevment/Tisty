use serde_json::{Value, json};
use tisty_core::brainstorm::{self, IDEA_AT_MOST};
use tisty_core::event::{Flag, Idea};
use tisty_core::model::Kept;
use tisty_core::{Op, Paths, State, Store};

use super::asked::text;
use super::jsonrpc::{speaking_through, told};
use super::{Refused, UNSETTLED, doc_named, hitch, opened, retold};

const IDEAS_AT_ONCE: usize = 50;

pub(super) fn sheet<'a>(paths: &Paths, state: &'a State) -> Option<(&'a Kept, String)> {
    brainstorm::said_sheet(&paths.docs(), state)
}

pub(super) fn sheet_named<'a>(
    paths: &Paths,
    state: &'a State,
    file: &str,
) -> Option<(&'a Kept, String)> {
    let kept = state.docs.values().find(|one| one.file == file)?;
    brainstorm::sheet_of(&paths.docs(), kept)
}

pub(super) fn holds_ideas(state: &State, file: &str) -> bool {
    state
        .docs
        .values()
        .any(|one| one.file == file && one.brainstorm)
}

pub(super) fn no_pages_on_a_sheet(state: &State, file: &str) -> Refused {
    Refused::Tool(format!(
        "{} is Brainstorm, a sheet of ideas the window keeps pinned on its own: it holds no pages \
         and is no page. Hang the page under the document of its subject.",
        doc_named(state, file)
    ))
}

pub(super) fn sheet_may_start(paths: &Paths, state: &State, placed: bool) -> Result<(), Refused> {
    if let Some((kept, _)) = sheet(paths, state) {
        return Err(Refused::Tool(format!(
            "the person's Brainstorm is already {}, and there is only ever one: add these ideas \
             to it with `append_doc`. Nothing was written.",
            doc_named(state, &kept.file)
        )));
    }
    if placed {
        return Err(Refused::Tool(
            "Brainstorm is pinned at the top of the Library on its own: send it without `folder` \
             or `page_of`. Nothing was written."
                .into(),
        ));
    }
    Ok(())
}

pub(super) fn sheet_stays_put(state: &State, file: &str) -> Refused {
    Refused::Tool(format!(
        "{} is Brainstorm, pinned at the top of the Library on its own: it goes in no folder and \
         joins no other document's languages, and only the person puts it away, from the window, \
         since every idea on it would go with it. Nothing was changed.",
        doc_named(state, file)
    ))
}

pub(super) fn nothing_but_ideas(state: &State, file: &str) -> Refused {
    Refused::Tool(format!(
        "{} is Brainstorm, and everything on it is an idea of a few words: a file added at its \
         end would be glued onto the last idea. Attach it to the document or the task its \
         subject belongs to. Nothing was copied.",
        doc_named(state, file)
    ))
}

pub(super) fn is_sheet(body: &str) -> bool {
    tisty_markdown::front::brainstorm(body)
}

pub(super) fn glance(paths: &Paths, state: &State) -> Option<(String, Value)> {
    let (kept, body) = sheet(paths, state)?;
    let waiting = brainstorm::ideas(&body);
    let marked = brainstorm::marked(kept, &body).len();
    let marks = match marked {
        0 => String::new(),
        n => format!(", {n} of them marked and waiting for the person"),
    };
    let named = doc_named(state, &kept.file);
    Some((
        match waiting {
            0 => format!(
                "Brainstorm, {named}, holds no idea now: new ones are jotted on it, as `rules` \
                 «ideas» says."
            ),
            _ => format!(
                "Brainstorm, {named}, holds {waiting} idea(s) not yet sorted{marks}: `rules` \
                 «ideas» says how to work them."
            ),
        },
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
        "This is Brainstorm: {} idea(s), oldest first, each a `> [!IDEA]` note headed by the \
         minute it came and its colour. An idea is named `{}#` and that heading. `rules` «ideas» \
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
        .map(|(doc, heading)| (doc.trim(), bare_heading(heading)))
}

fn bare_heading(heading: &str) -> &str {
    let heading = heading.trim();
    tisty_markdown::idea_words(heading).unwrap_or(heading)
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
        if lines
            .iter()
            .any(|line| headed(line) || tisty_markdown::idea_words(line).is_some())
        {
            return Err(Refused::Tool(
                "each idea's `> [!IDEA]` line, with the minute it came and its colour, is Tisty's \
                 to write, and a heading has no place on Brainstorm. Send the ideas alone: one per \
                 paragraph, or one per item of a list. Nothing was written."
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

fn headed(line: &str) -> bool {
    let hashes = line.chars().take_while(|one| *one == '#').count();
    (1..=6).contains(&hashes)
        && line[hashes..]
            .chars()
            .next()
            .is_none_or(char::is_whitespace)
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
    let settled = retold(state, store, &kept.file, &whole).is_ok();
    Ok(told(
        format!(
            "Jotted {} idea(s) on {named}, dated from {} on, each headed by a minute and colour \
             no other idea there shares. Nothing that was there changed.{}",
            each.len(),
            at.strftime("%Y-%m-%d %H:%M"),
            if settled { "" } else { UNSETTLED }
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

pub(super) struct Filed {
    pub(super) said: String,
    pub(super) off: bool,
}

fn still_waits(named: &str, idea: &Idea, into: &str) -> Filed {
    Filed {
        said: format!(
            " «{}» could not be taken off {named} just now, so it still waits there: mark it \
             `moved` with `flag_doc`, `instead` {into}.",
            idea.text
        ),
        off: false,
    }
}

// Filing moves the idea, as the window's «File in a document» does, so the sheet holds only what waits.
pub(super) fn filed_off(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    said: &str,
    into: &str,
) -> Option<Filed> {
    if !brainstorm::files_an_idea(said) {
        return None;
    }
    let (kept, body) = sheet(paths, state)?;
    if kept.file == into {
        return None;
    }
    let named = doc_named(state, &kept.file);
    let on = brainstorm::ideas_on(&body);
    let quoted = brainstorm::quoted(said);
    let same_words = |one: &Idea| {
        quoted
            .as_deref()
            .is_some_and(|words| one.text.split_whitespace().eq(words.split_whitespace()))
    };
    let head = tisty_markdown::idea_head(said.trim_start());
    let headed_alike = |one: &Idea| {
        let theirs = tisty_markdown::idea_parts(&one.heading);
        theirs.when == head.when && (head.hue.is_none() || theirs.hue == head.hue)
    };
    let found = on
        .iter()
        .find(|one| headed_alike(one) && same_words(one))
        .or_else(|| on.iter().find(|one| same_words(one)));
    // A note headed like no idea on the sheet is a sticky note of its own, not a filing.
    let Some(idea) = found.cloned() else {
        let named_one = on.iter().find(|one| headed_alike(one))?;
        return Some(Filed {
            said: format!(
                " This note is headed as «{}» on {named}, but its words are not that idea's, so \
                 the sheet still holds it: a filed idea goes alone, its words as the sheet has \
                 them. If this note files it, mark that idea `moved` with `flag_doc`, `instead` \
                 \"{into}\", rather than adding the note again.",
                named_one.text
            ),
            off: false,
        });
    };
    Some(match taken_off(paths, state, store, kept, &idea) {
        Some(()) => Filed {
            said: format!(" The idea it quotes left {named}, as filing it from the window does."),
            off: true,
        },
        None => still_waits(&named, &idea, &format!("\"{into}\"")),
    })
}

// The idea becomes the task, as the window's «Turn into a task» makes it, so it waits nowhere twice.
pub(super) fn made_a_task(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    source: &str,
) -> Option<Filed> {
    let (doc, heading) = named_idea(source)?;
    let (kept, body) = sheet_named(paths, state, doc)?;
    let named = doc_named(state, &kept.file);
    let found: Vec<Idea> = brainstorm::ideas_on(&body)
        .into_iter()
        .filter(|one| one.heading == heading)
        .collect();
    Some(match found.as_slice() {
        [idea] => match taken_off(paths, state, store, kept, idea) {
            Some(()) => Filed {
                said: format!(
                    " The idea it came from left {named}, as turning it into a task from the \
                     window does."
                ),
                off: true,
            },
            None => still_waits(&named, idea, "this task"),
        },
        [] => Filed {
            said: format!(
                " No idea on {named} is headed {heading:?}, so the one this task came from still \
                 waits there: mark it `moved` with `flag_doc`, `instead` this task."
            ),
            off: false,
        },
        _ => Filed {
            said: format!(
                " {} ideas on {named} carry that heading, so none was taken off: mark the one \
                 this task came from with `flag_doc`, `moved` to it.",
                found.len()
            ),
            off: false,
        },
    })
}

pub(super) fn keeps_every_idea(
    state: &State,
    which: &str,
    before: &str,
    after: &str,
) -> Result<(), Refused> {
    if !is_sheet(before) {
        return match is_sheet(after) {
            true => Err(Refused::Tool(format!(
                "`brainstorm: on` would turn {} into a Brainstorm sheet, and a sheet is only ever \
                 made new with `write_doc`, as `rules` «ideas» says. Leave that line out. Nothing \
                 was written.",
                doc_named(state, which)
            ))),
            false => Ok(()),
        };
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
        let kept_whole = left.iter().position(|kept| *kept == one);
        let reworded = || {
            left.iter().position(|kept| {
                kept.heading == one.heading && (one.text.is_empty() || !kept.text.is_empty())
            })
        };
        match kept_whole.or_else(reworded) {
            Some(at) => {
                left.remove(at);
            }
            None => lost.push(one),
        }
    }
    if let Some(gone) = lost.first() {
        return Err(Refused::Tool(format!(
            "{named} is the person's Brainstorm, and this write would take {} idea(s) off it or \
             leave them without their words, «{}» first. Taking an idea off is theirs: mark it \
             with `flag_doc`, `doc` \"{which}#{}\", or file it in a document or a task, which \
             takes it off as the window does. Nothing was written.",
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
    let first = many
        .iter()
        .find_map(|one| named_idea(one))
        .map_or_else(String::new, |(doc, _)| doc.to_string());
    let Some((kept, sheet_body)) = sheet_named(paths, &state, &first) else {
        return Err(Refused::Tool(format!(
            "{first:?} is not Brainstorm, so `{first}#` and a heading names no idea. Ideas are \
             named by the sheet's id: `catch_up` gives it. A document is marked by its id alone. \
             Nothing was marked."
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
             words after `[!IDEA]` on its first line. Nothing was marked."
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

pub(super) fn filed_too(mut answer: Value, filed: Option<Filed>) -> Value {
    let Some(filed) = filed else {
        return answer;
    };
    for at in ["/content/0/text", "/structuredContent/said"] {
        if let Some(Value::String(said)) = answer.pointer_mut(at) {
            said.push_str(&filed.said);
        }
    }
    if let Some(Value::Object(kept)) = answer.pointer_mut("/structuredContent") {
        kept.insert("idea_filed".into(), json!(filed.off));
    }
    answer
}
