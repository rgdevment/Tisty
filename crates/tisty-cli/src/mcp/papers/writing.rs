use serde_json::{Value, json};

use tisty_core::beside::{Carried, written_as_code};
use tisty_core::{Op, Paths, State, Store};
use ulid::Ulid;

use super::super::asked::text;
use super::super::jsonrpc::told;
use super::super::{
    Refused, UNSETTLED, WHOLE_UP_TO, by_how_much, doc_titled, folder_named, grew, hitch,
    left_loose, loose_words, named_doc, named_twice, opened, retold, room_for_a_notice, trail,
    twice_words, up_named, with_echo, wrapped, wrote,
};

pub(in crate::mcp) fn write_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    written_doc(paths, args, None)
}

/// An import has judged the subject already, and the files it copied are told with the document.
pub(super) fn written_doc(
    paths: &Paths,
    args: &Value,
    imported: Option<Vec<Op>>,
) -> Result<Value, Refused> {
    let Some(body) = text(args, "body") else {
        return Err(Refused::Tool("a document needs a `body`.".into()));
    };
    let body = tisty_core::speaking::turned(&body);
    let (state, mut store) = opened(paths)?;

    tisty_core::docs::survives(&body).map_err(|eats| Refused::Tool(not_spoken(eats)))?;

    let body = match warned(paths, &body).and_then(|notice| warned_into("", &body, &notice)) {
        Some(made) => made,
        None => body,
    };

    if let Some(which) = text(args, "doc") {
        if super::compacting::compacting(args) {
            return super::compacting::compacted(paths, args, &state, &mut store, &which, &body);
        }
        return over_again(paths, args, &state, &mut store, &which, &body);
    }
    super::compacting::not_alone(args)?;
    if text(args, "print").is_some() {
        return Err(Refused::Tool(
            "`print` says which body you mean to replace, so it needs the `doc` it belongs to. Without one, `write_doc` writes a new document."
                .into(),
        ));
    }

    let (folder, page_of) = where_it_lands(&state, args)?;
    if imported.is_none() {
        super::compacting::subject_free(&state, args, &body)?;
    }
    let made = tisty_core::docs::create(&paths.docs(), store.device(), &body).map_err(|e| match e {
        tisty_core::Error::DocumentTooBig { limit, .. } => Refused::Tool(format!(
            "that body is past the {limit} bytes Tisty can open. Send a shorter document, or split it into pages."
        )),
        other => hitch(other),
    })?;
    let order = tisty_core::order::last_of(
        state
            .docs
            .values()
            .filter(|one| one.page_of == page_of && (page_of.is_some() || one.folder == folder))
            .map(|one| one.order.as_str()),
    );
    let id = Ulid::generate();
    let add = Op::DocAdd {
        id,
        d: tisty_core::event::DocAdd {
            wrote: None,
            guest: false,
            made: None,
            by: state.signed.alias.clone(),
            file: made.id.clone(),
            order,
            said: Some(tisty_core::event::Said::of(&body)),
            folder,
            page_of,
            task_of: None,
        },
    };
    let added = match imported {
        Some(mut ops) => {
            ops.push(add);
            store.append_batch(ops).map(|_| ())
        }
        None => store.append(add).map(|_| ()),
    };
    if let Err(e) = added
        && !wrote(&mut store, id)
    {
        let _ = tisty_core::docs::remove(&paths.docs(), &made.id);
        return Err(hitch(e));
    }

    let mut named_there = None;
    if let Some(up) = page_of
        .and_then(|up| state.docs.get(&up))
        .map(|up| up.file.clone())
    {
        let said =
            tisty_core::docs::name_at_end(&paths.docs(), paths.data(), &up, &[made.id.as_str()]);
        if let Ok(tisty_core::docs::Naming::Wrote { whole, .. }) = &said {
            // The page and its card are already written; refusing now would have a retry write
            // both a second time, and where they sit settles by itself on the next write or open.
            if let Ok(events) = store.read_all() {
                let _ = retold(&tisty_core::State::replay(&events), &mut store, &up, whole);
            }
        }
        named_there = said.ok();
    }
    let named_there = named_there.unwrap_or(tisty_core::docs::Naming::Nothing);

    let where_at = folder.map(|at| trail(&state, at));
    let under = page_of.and_then(|up| named_doc(&state, up));
    let held_by = page_of.and_then(|up| up_named(&state, up));
    Ok(told(
        match (&held_by, &where_at) {
            (Some(named), _) if matches!(named_there, tisty_core::docs::Naming::Wrote { .. }) => {
                format!(
                    "Wrote the document {:?}, a page of {named}, and named it at the end of that \
                     document. Where a page is named is where it sits. [{}]",
                    made.title, made.id
                )
            }
            (Some(named), _) => format!(
                "Wrote the document {:?} [{}], a page of {named}, but no line naming it was \
                 written, so it is loose: {}. Name it with `page_doc` once that is dealt with.",
                made.title,
                made.id,
                match named_there {
                    tisty_core::docs::Naming::Fenced =>
                        "that document ends inside a fence, and a line in code is not a way in",
                    tisty_core::docs::Naming::WouldRename =>
                        "that document says nothing a title could come from yet, so the line \
                         would have become its title",
                    _ => "that document could not be read back",
                }
            ),
            (None, Some(named)) => format!(
                "Wrote the document {:?} in the folder {named}. [{}]",
                made.title, made.id
            ),
            (None, None) => format!(
                "Wrote the document {:?}.{} [{}]",
                made.title,
                super::super::placing::loose_doc(&state, &made.title),
                made.id
            ),
        },
        json!({
            "for_the_person": match (under.as_deref(), &where_at) {
                (Some(up), _) => format!(
                    "the document {:?}, a page of {}",
                    made.title,
                    doc_titled(&state, up)
                ),
                (None, Some(named)) => format!("the document {:?}, in the folder {named}", made.title),
                (None, None) => format!("the document {:?}, outside any folder", made.title),
            },
            "doc": made.id,
            "title": made.title,
            "folder": where_at,
            "page_of": under,
            "print": tisty_core::docs::read(&paths.docs(), &made.id)
                .map(|one| tisty_core::attach::printed(one.as_bytes()))
                .unwrap_or_default(),
        }),
    ))
}

/// Every write keeps what the document said before it, and until now nothing could reach it.
/// Restoring keeps the body it replaces in turn, so this undoes itself when it was the wrong call.
pub(in crate::mcp) fn restore_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(which) = text(args, "doc") else {
        return Err(Refused::Tool(
            "going back needs the `doc` name of the document to put back.".into(),
        ));
    };
    let (which, a_task_s) = super::compacting::page_to_restore(paths, &which)?;
    let Some(was) = tisty_core::docs::read_before(paths.data(), &which) else {
        return Err(Refused::Tool(format!(
            "nothing is kept beside {which:?} to go back to. Only a write that replaced a body \
             is held, and adding to the end — `append_doc`, or a file kept with `attach` — \
             replaces nothing."
        )));
    };
    let now = tisty_core::docs::read(&paths.docs(), &which).map_err(hitch)?;
    if a_task_s {
        super::compacting::only_the_notes(&now, &was)?;
    }
    if tisty_core::docs::unchanged(&now, &was) {
        return Err(Refused::Tool(format!(
            "{which:?} already says what is kept beside it, so there is nothing to go back to."
        )));
    }
    // Adding to the end keeps nothing, and neither does the window's own save. Either one leaves
    // the kept body further back than one step, and putting it back would take their writing too.
    let print = tisty_core::attach::printed(now.as_bytes());
    let left = tisty_core::docs::before_left_at(paths.data(), &which);
    let more = left.as_deref() != Some(print.as_str());
    let asked = args
        .get("even_if_more")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if more && !asked {
        return Err(Refused::Tool(format!(
            "{which:?} has been written since the body kept beside it was set aside, so going \
             back would undo that writing as well and not only the one write. Nothing was \
             changed. Read it and edit the part that is wrong instead — or send \
             `even_if_more` to go back over all of it, which is itself kept and can be undone \
             the same way."
        )));
    }

    let (state, mut store) = opened(paths)?;
    let mut said = over_again(
        paths,
        &json!({ "print": print }),
        &state,
        &mut store,
        &which,
        &was,
    )?;
    if let Some(kept) = state.docs.values().find(|one| one.file == which)
        && kept
            .compacted
            .as_ref()
            .is_some_and(|one| one.undone_by(&now, &was))
    {
        let _ = store.append(Op::DocUncompacted { id: kept.id });
    }
    if more {
        if let Some(text) = said["content"][0]["text"].as_str() {
            said["content"][0]["text"] = json!(format!(
                "{text} That went back over writing done since, which was what `even_if_more` \
                 asked for: what it wrote over is kept in turn, so calling this again puts it \
                 back."
            ));
        }
        said["structuredContent"]["over_more_than_one_write"] = json!(true);
    }
    Ok(said)
}

pub(in crate::mcp) fn append_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(which) = text(args, "doc") else {
        return Err(Refused::Tool(
            "adding to a document needs its `doc` name.".into(),
        ));
    };
    let Some(body) = text(args, "body") else {
        return Err(Refused::Tool("adding needs a `body` to add.".into()));
    };
    let body = tisty_core::speaking::turned(&body);
    let (state, mut store) = opened(paths)?;
    if let Some(task) = super::super::task_notes::task_named(&state, &which) {
        tisty_core::docs::survives(&body).map_err(|eats| Refused::Tool(not_spoken(eats)))?;
        let under = text(args, "under");
        let replace = args
            .get("replace")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        return super::super::task_notes::noted_on_page(
            paths,
            &state,
            &mut store,
            task,
            &body,
            under.as_deref(),
            replace,
        );
    }
    if args.get("replace").and_then(Value::as_bool) == Some(true) {
        return Err(Refused::Tool(
            "`replace` writes the assistant's notes on a task's page again, so it needs a task's \
             id in `doc`. A document's passage is changed with `edit_doc`."
                .into(),
        ));
    }
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        )));
    };
    if state.held_away(kept) {
        return Err(Refused::Tool(format!(
            "{which:?} is put away, so nothing more goes into it. Write a new document instead."
        )));
    }
    if state.shut(kept.id) {
        return Err(Refused::Tool(format!(
            "{which:?} is locked. The person shut it so nothing writes in it — not the \
             window, not you. Ask them to unlock it if it truly has to change."
        )));
    }
    tisty_core::docs::survives(&body).map_err(|eats| Refused::Tool(not_spoken(eats)))?;

    let before = tisty_core::docs::read(&paths.docs(), &which).unwrap_or_default();
    let body = match warned(paths, &body).and_then(|notice| warned_into(&before, &body, &notice)) {
        Some(made) => made,
        None => body,
    };

    let under = text(args, "under");
    let mut why = None;
    let whole =
        match &under {
            None => tisty_core::docs::append(&paths.docs(), &which, &body),
            Some(under) => tisty_core::docs::amend(&paths.docs(), paths.data(), &which, |was| {
                match under_a_heading(was, under, &body) {
                    Ok(whole) => Some(whole),
                    Err(said) => {
                        why = Some(said);
                        None
                    }
                }
            })
            .map(|made| made.unwrap_or_default()),
        }
        .map_err(|e| match e {
            tisty_core::Error::DocumentTooBig { limit, .. } => Refused::Tool(format!(
                "that would take the document past the {limit} bytes Tisty can open. Write a new \
             document instead of growing this one."
            )),
            other => hitch(other),
        })?;
    if let Some(said) = why {
        return Err(Refused::Tool(said));
    }

    // The text is already written: refusing here would have a dutiful retry add it twice.
    let settled = retold(&state, &mut store, &which, &whole).is_ok();
    let twice = named_twice(&state, &which, &whole);

    Ok(told(
        format!(
            "Added {} {:?}, {}. Nothing that was there changed.{}{}{}",
            match &under {
                Some(under) => format!("under {under:?} in"),
                None => "to the end of".into(),
            },
            tisty_core::docs::titled(&whole),
            by_how_much(&before, &whole),
            twice_words(&state, &twice),
            if settled { "" } else { UNSETTLED },
            wrapped(&body)
        ),
        with_echo(
            json!({
                "doc": which,
                "title": tisty_core::docs::titled(&whole),
                "chars": whole.chars().count(),
                "lines": whole.lines().count(),
                "grew": grew(&before, &whole),
                "print": tisty_core::attach::printed(whole.as_bytes()),
            }),
            args,
            &before,
            &whole,
        ),
    ))
}

pub(super) fn over_again(
    paths: &Paths,
    args: &Value,
    state: &tisty_core::State,
    store: &mut tisty_core::Store,
    which: &str,
    body: &str,
) -> Result<Value, Refused> {
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        )));
    };
    if state.held_away(kept) {
        return Err(Refused::Tool(format!(
            "{which:?} is put away, so nothing is written over it."
        )));
    }
    if state.shut(kept.id) {
        return Err(Refused::Tool(format!(
            "{which:?} is locked. The person shut it so nothing writes in it — not the \
             window, not you. Ask them to unlock it if it truly has to change."
        )));
    }
    if text(args, "folder").is_some() || text(args, "page_of").is_some() {
        return Err(Refused::Tool(format!(
            "replacing the body of {which:?} does not move it. `file_doc` puts a document in a folder and `page_doc` makes it a page."
        )));
    }
    let Some(print) = text(args, "print") else {
        return Err(Refused::Tool(format!(
            "replacing the body of {which:?} needs the `print` you read it at, which `read_doc` hands back beside the text. Read it, then send that print with the new body."
        )));
    };

    let named = tisty_core::refs::papers(body);
    let loose: Vec<&tisty_core::model::Kept> = state
        .pages_of(kept.id)
        .into_iter()
        .filter(|one| !named.contains(&one.file))
        .collect();
    let walled_off = tisty_core::docs::ends_fenced(body);
    let adrift: Vec<String> = match walled_off {
        true => loose.iter().map(|one| one.file.clone()).collect(),
        false => Vec::new(),
    };
    let loose: Vec<&tisty_core::model::Kept> = match walled_off {
        true => Vec::new(),
        false => loose,
    };
    let body = &match loose.is_empty() {
        true => body.to_string(),
        false => {
            let cards = loose
                .iter()
                .map(|one| {
                    let title = tisty_core::docs::read(&paths.docs(), &one.file)
                        .map(|body| tisty_core::docs::titled(&body))
                        .unwrap_or_default();
                    tisty_core::refs::card(&one.file, &title)
                })
                .collect::<Vec<_>>()
                .join(
                    "
",
                );
            format!(
                "{}

{cards}
",
                body.trim_end()
            )
        }
    };
    let kept_back: Vec<String> = loose.iter().map(|one| one.file.clone()).collect();

    let made = tisty_core::docs::rewrite(&paths.docs(), paths.data(), which, body, &print)
        .map_err(hitch)?;
    match made {
        tisty_core::docs::Rewrite::Moved => Err(Refused::Tool({
            let now = tisty_core::docs::read(&paths.docs(), which).unwrap_or_default();
            let shown = match now.chars().count() > WHOLE_UP_TO {
                true => format!(
                    "what is in it now:

{}",
                    said_outline(&now)
                ),
                false => format!(
                    "what it says now:

{now}"
                ),
            };
            format!(
                "{which:?} does not read as it did when you took that print — the person, or another agent, wrote in it since. Nothing was changed, and nothing of theirs was lost. Here is {shown}

print: {}",
                tisty_core::attach::printed(now.as_bytes())
            )
        })),
        tisty_core::docs::Rewrite::Made { whole, .. } => {
            let settled = retold(state, store, which, &whole).is_ok();
            let twice = named_twice(state, which, &whole);
            Ok(told(
                format!(
                    "Wrote {:?} again, whole. {}{}{}{}{}",
                    tisty_core::docs::titled(&whole),
                    "What it said before is kept beside the documents.",
                    twice_words(state, &twice),
                    match kept_back.is_empty() {
                        true => String::new(),
                        false => format!(
                            " The body you sent named none of {}, which are pages of it, so their lines were put back at the end rather than left with nothing pointing at them. Move them with `edit_doc` if they belong somewhere else.",
                            kept_back.join(", ")
                        ),
                    },
                    match walled_off {
                        false => String::new(),
                        true => format!(
                            " It ends inside a fence, so a line naming a page would have been written as code rather than as a way in: {} are pages of it that nothing now names. Close the fence and write those lines where they belong.",
                            adrift.join(", ")
                        ),
                    },
                    if settled { "" } else { UNSETTLED }
                ),
                json!({
                "doc": which,
                "title": tisty_core::docs::titled(&whole),
                "chars": whole.chars().count(),
                "lines": whole.lines().count(),
                "print": tisty_core::attach::printed(whole.as_bytes()),
                "pages_kept_back": kept_back,
                }),
            ))
        }
    }
}

fn warned(paths: &Paths, body: &str) -> Option<String> {
    let told = tisty_core::agent::secrets_in(body.as_bytes());
    if told.is_empty() {
        return None;
    }
    let lang = crate::i18n::Lang::detect(
        tisty_core::Config::load_or_init(paths)
            .ok()
            .and_then(|one| one.locale)
            .as_deref(),
    );
    let mut named: Vec<&str> = Vec::new();
    for one in told.iter().map(|one| one.named.as_str()) {
        let one = &one[..one.len().min(A_NAME_AT_MOST)];
        if !one.is_empty() && !named.contains(&one) {
            named.push(one);
        }
    }
    let over = named.len().saturating_sub(NAMES_IN_A_WARNING);
    named.truncate(NAMES_IN_A_WARNING);
    let what = match (named.is_empty(), over) {
        (true, _) => lang.get("secret-a-key").to_string(),
        (false, 0) => named.join(", "),
        (false, more) => format!("{} +{more}", named.join(", ")),
    };
    Some(format!(
        "> [!CAUTION]\n> {}",
        lang.fill("secret-warning", &[("what", &what)])
    ))
}

fn warned_into(before: &str, body: &str, notice: &str) -> Option<String> {
    if already_warned(before) || already_warned(body) {
        return None;
    }
    let whole = format!("{before}{body}");
    let code = written_as_code(&whole);
    let inside = |at: usize| code.get(before.len() + at).copied().unwrap_or(false);

    let at = room_for_a_notice(body);
    let at = match inside(at) {
        false => at,
        true => body.len(),
    };
    if inside(at) || inside(at.saturating_sub(1)) {
        return None;
    }
    let made = put_in(body, at, notice);
    let made = match tisty_core::docs::titled(&made) == tisty_core::docs::titled(body) {
        true => made,
        false => put_in(body, body.len(), notice),
    };
    tisty_core::docs::survives(&made).is_ok().then_some(made)
}

/// Every reason a new document is turned away, worked out before anything is written or copied:
/// a folder that does not exist must not be paid for with an orphan on disk.
pub(super) fn where_it_lands(
    state: &State,
    args: &Value,
) -> Result<
    (
        Option<tisty_core::model::FolderId>,
        Option<tisty_core::model::DocId>,
    ),
    Refused,
> {
    // An append-only store keeps every one of these forever, and the window replays them all.
    if state.papers().count() >= DOCS_AT_MOST {
        return Err(Refused::Tool(format!(
            "there are already {DOCS_AT_MOST} documents here. Add to a task's journal instead, or ask the person to clear some."
        )));
    }
    let folder = match text(args, "folder") {
        Some(said) => Some(folder_named(state, &said)?),
        None => None,
    };
    let page_of = match text(args, "page_of") {
        None => None,
        Some(said) => {
            let Some(up) = state.docs.values().find(|one| one.file == said) else {
                return Err(Refused::Tool(format!(
                    "no document here is called {said:?}. `docs` lists them all."
                )));
            };
            if up.page_of.is_some() {
                return Err(Refused::Tool(format!(
                    "{said} is a page itself, and a page holds no pages. Name the document it \
                     belongs to."
                )));
            }
            if state.held_away(up) {
                return Err(Refused::Tool(format!(
                    "{said} is put away, and a page of it would be put away unread. Write a \
                     document of its own instead."
                )));
            }
            if state.shut(up.id) {
                return Err(Refused::Tool(format!(
                    "{said} is locked, and hanging a page off it writes the line that names \
                     it in its body. Ask the person to unlock it first."
                )));
            }
            Some(up.id)
        }
    };
    Ok(match page_of.and_then(|up| state.docs.get(&up)) {
        Some(up) => (up.folder, page_of),
        None => (folder, page_of),
    })
}

/// Added under a heading means at the end of what that heading holds, not just below its line.
fn under_a_heading(was: &str, under: &str, body: &str) -> Result<String, String> {
    let all = tisty_core::docs::headings(was);
    let same = |one: &str| one.trim().eq_ignore_ascii_case(under.trim());
    let hit: Vec<usize> = all
        .iter()
        .enumerate()
        .filter(|(_, (_, _, title))| same(title))
        .map(|(at, _)| at)
        .collect();

    let at = match hit.as_slice() {
        [one] => *one,
        [] => {
            return Err(match all.is_empty() {
                true => format!(
                    "this document has no headings at all, so there is no {under:?} to add under. \
                     Leave `under` out and it goes at the end."
                ),
                false => format!(
                    "no heading here reads {under:?}. These do:\n\n{}",
                    said_outline(was)
                ),
            });
        }
        many => {
            return Err(format!(
                "{} headings read {under:?}, and Tisty will not choose for you: lines {}. Name a \
                 `section` with `edit_doc` instead.",
                many.len(),
                many.iter()
                    .map(|at| all[*at].0.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    };

    let (_, to) =
        tisty_core::docs::section_lines(was, at).ok_or("that heading went missing".to_string())?;
    let last = was.lines().count().max(1);
    let held = tisty_core::docs::lines_between(was, 1, to);
    let rest = tisty_core::docs::lines_between(was, to + 1, last);
    if !holds_together(was, &held, &rest) {
        return Err(
            "that heading could not be cut out cleanly, so nothing was added. Read the document \
             again and add under a heading `outline_doc` names."
                .to_string(),
        );
    }
    let ending = match was.contains("\r\n") {
        true => "\r\n",
        false => "\n",
    };
    let body = body
        .trim_end_matches('\n')
        .replace("\r\n", "\n")
        .replace('\n', ending);
    Ok(format!("{held}{ending}{body}{ending}{rest}"))
}

pub(super) fn where_over(body: &str, old: &str) -> Vec<usize> {
    let first = match old.lines().find(|one| !one.trim().is_empty()) {
        Some(one) => one,
        None => return Vec::new(),
    };
    body.lines()
        .enumerate()
        .filter(|(_, line)| line.contains(first))
        .map(|(n, _)| n + 1)
        .collect()
}

/// An edit that names where rather than what has no text to anchor on, so the print stands in.
pub(super) fn in_its_place(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    which: &str,
    args: &Value,
    new: &str,
) -> Result<Value, Refused> {
    let Some(print) = text(args, "print") else {
        return Err(Refused::Tool(
            "an edit that names a place rather than a passage needs the `print` the document read \
             at, since there is no text to recognise it by. `outline_doc` hands it back."
                .into(),
        ));
    };
    let body = tisty_core::docs::read(&paths.docs(), which).map_err(hitch)?;
    let last = body.lines().count().max(1);

    let (from, to) = match args.get("section").and_then(Value::as_u64) {
        Some(at) => tisty_core::docs::section_lines(&body, at as usize).ok_or_else(|| {
            Refused::Tool(format!(
                "this document has no section {at}. `outline_doc` numbers them from 0."
            ))
        })?,
        None => {
            let from = args
                .get("from")
                .and_then(Value::as_u64)
                .map(|one| one as usize)
                .unwrap_or(1)
                .max(1);
            let to = args
                .get("to")
                .and_then(Value::as_u64)
                .map(|one| one as usize)
                .unwrap_or(last)
                .min(last);
            if from > to {
                return Err(Refused::Tool(format!(
                    "`from` is line {from} and `to` is line {to}, so there is nothing between them."
                )));
            }
            (from, to)
        }
    };

    if from <= 1 && to >= last {
        return Err(Refused::Tool(format!(
            "that names the whole of {which:?}, which `edit_doc` will not take: a passage it \
             cannot tell from the document is a rewrite wearing an edit's clothes. Nothing was \
             changed. Name the part that differs, or replace the body with `write_doc`."
        )));
    }

    let new = tisty_core::speaking::turned(&new.replace('\r', ""));
    let tail = match new.is_empty() || new.ends_with('\n') {
        true => new,
        false => format!("{new}\n"),
    };
    let (head, rest) = (
        tisty_core::docs::lines_between(&body, 1, from - 1),
        tisty_core::docs::lines_between(&body, to + 1, last),
    );
    if !holds_together(&body, &head, &rest) {
        return Err(Refused::Tool(format!(
            "lines {from} to {to} could not be cut out of {which:?} cleanly, so nothing was \
             written. Read it again and name the passage."
        )));
    }
    let whole = format!("{head}{tail}{rest}");

    tisty_core::docs::survives(&tail).map_err(|eats| Refused::Tool(not_spoken(eats)))?;

    match tisty_core::docs::rewrite(&paths.docs(), paths.data(), which, &whole, &print)
        .map_err(hitch)?
    {
        tisty_core::docs::Rewrite::Moved => Err(Refused::Tool(format!(
            "{which:?} does not read as it did when you took that print, so line {from} is no \
             longer where you left it and nothing was changed. Ask `outline_doc` where the \
             passage sits now."
        ))),
        tisty_core::docs::Rewrite::Made { whole, .. } => {
            let loose = left_loose(state, which, &body, &whole);
            let twice = named_twice(state, which, &whole);
            let settled = retold(state, store, which, &whole).is_ok();
            Ok(told(
                format!(
                    "Changed lines {from} to {to} of {:?}. What it was is kept beside the \
                     documents.{}{}{}",
                    tisty_core::docs::titled(&whole),
                    loose_words(state, &loose),
                    twice_words(state, &twice),
                    if settled { "" } else { UNSETTLED }
                ),
                with_echo(
                    json!({
                        "doc": which,
                        "title": tisty_core::docs::titled(&whole),
                        "chars": whole.chars().count(),
                        "lines": whole.lines().count(),
                        "grew": grew(&body, &whole),
                        "loose": loose,
                        "print": tisty_core::attach::printed(whole.as_bytes()),
                    }),
                    args,
                    &body,
                    &whole,
                ),
            ))
        }
    }
}

pub(super) fn beside_the_file(
    paths: &Paths,
    from: &std::path::Path,
    body: &str,
) -> (String, Carried) {
    let mut reach = |at: &std::path::Path| {
        tisty_core::agent::may_reach(at, paths)
            .map_err(|_| "it is not somewhere an assistant may take files from".to_string())
    };
    tisty_core::beside::brought(
        paths,
        from,
        body,
        &mut tisty_core::beside::Way {
            reach: &mut reach,
            outside: ", or bring the file in on its own afterwards with `attach`",
            notes: true,
        },
    )
}

pub(super) fn said_of(written: &Value) -> String {
    written["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

const DOCS_AT_MOST: usize = 500;

const NAMES_IN_A_WARNING: usize = 6;

const A_NAME_AT_MOST: usize = 40;

fn already_warned(body: &str) -> bool {
    body.lines().any(|one| {
        one.trim_start()
            .trim_start_matches('>')
            .trim_start()
            .starts_with("[!CAUTION]")
    })
}

fn put_in(body: &str, at: usize, notice: &str) -> String {
    let (head, tail) = body.split_at(at);
    let head = head.trim_end_matches('\n');
    let tail = tail.trim_start_matches('\n');
    match (head.is_empty(), tail.is_empty()) {
        (true, true) => notice.to_string(),
        (true, false) => format!("{notice}\n\n{tail}"),
        (false, true) => format!("{head}\n\n{notice}\n"),
        (false, false) => format!("{head}\n\n{notice}\n\n{tail}"),
    }
}

/// The head and the tail of a splice are cut from the same document, so together they can never
/// be longer than it was. They were, once, and the document came back with a second copy of itself
/// pasted behind the edit.
fn holds_together(body: &str, head: &str, tail: &str) -> bool {
    head.len() + tail.len() <= body.len()
}

/// The outline as a person would read it aloud, not as JSON.
fn said_outline(body: &str) -> String {
    tisty_core::docs::outlined(body)
        .iter()
        .map(|one| {
            format!(
                "{}{}  ·  lines {}-{}, {} chars",
                "  ".repeat(one.level - 1),
                one.title,
                one.line,
                one.to,
                one.chars
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn not_spoken(eats: &str) -> String {
    format!(
        "Tisty does not keep {eats} in its own words, so nothing was written. Send Tisty markdown: headings, lists, emphasis, links inline or by reference, footnotes, tables (aligned columns and all), fenced code with its language, and GitHub alerts written as a quote that opens with [!NOTE], [!TIP], [!IMPORTANT], [!WARNING] or [!CAUTION]. Underline is ++words++; a pen is ==words==, or ==words=={{green}} for another colour of the palette (red, orange, amber, green, teal, blue, indigo, purple, pink, brown, gray); an icon is ::name:: or ::name hue::, named as `language` lists them; an attachment is named by its stamped name. An older Tisty's <u>, <mark> and icon span are turned into these on their way in; any other tag is refused. Maths goes between $ or $$, or in a fence saying `math`. A fence carries its language and, if you want, a name: ```rust title=\"src/walk.rs\", and the same for `mermaid`, `math` and `widget`, which the window draws with that name above them. A `widget` holds plain HTML, CSS and script, drawn in a sealed frame with no network and the window's colours; its classes `grid`, `stat`, `card`, `head`, `badge`, `list`, `box` and `primary` buttons already look like Tisty, and a link in it can only open a `tisty:doc/` document or a web page. A page too large to write inline can be attached instead: `attach` an `.html` file to the document and it is drawn the same way."
    )
}
