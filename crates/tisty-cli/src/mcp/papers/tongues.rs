use std::collections::BTreeMap;

use serde_json::{Map, Value, json};
use tisty_core::model::{DocId, Kept};
use tisty_core::{Paths, State};

use super::writing::written_doc;

use super::super::asked::text;
use super::super::jsonrpc::told;
use super::super::{Refused, doc_named, hitch, opened, retold};

/// The languages a document is told in, for an assistant to read the one it needs.
pub(in crate::mcp) fn tongued_into(state: &State, kept: &Kept, into: &mut Map<String, Value>) {
    said_into(kept, &state.tongues_of(kept.id), into);
}

/// Every group's languages as a listing names them, worked out once for the whole list.
pub(in crate::mcp) fn spoken(state: &State) -> BTreeMap<DocId, Vec<&Kept>> {
    state
        .families()
        .into_values()
        .flat_map(|tongues| {
            tongues
                .iter()
                .map(|one| (one.id, tongues.clone()))
                .collect::<Vec<_>>()
        })
        .collect()
}

pub(in crate::mcp) fn said_into(kept: &Kept, tongues: &[&Kept], into: &mut Map<String, Value>) {
    if let Some(lang) = &kept.lang {
        into.insert("lang".into(), json!(lang));
    }
    if !tongues.is_empty() {
        let listed: Vec<Value> = tongues
            .iter()
            .map(|one| json!({ "lang": one.lang, "doc": one.file }))
            .collect();
        into.insert("languages".into(), json!(listed));
    }
}

fn found<'a>(state: &'a State, which: &str) -> Result<&'a Kept, Refused> {
    let kept = state
        .papers()
        .find(|one| one.file == which)
        .ok_or_else(|| Refused::Tool(format!("no document here is named {which:?}.")))?;
    if kept.page_of.is_some() {
        return Err(Refused::Tool(format!(
            "{} is a page, and a page is in the language of its document: join the document instead.",
            doc_named(state, which)
        )));
    }
    if state.held_away(kept) {
        return Err(super::put_away(state, kept, &doc_named(state, which)));
    }
    Ok(kept)
}

fn language(args: &Value) -> Result<Option<String>, Refused> {
    match text(args, "lang") {
        None => Ok(None),
        Some(said) => tisty_markdown::front::tagged(&said)
            .map(Some)
            .ok_or_else(|| {
                Refused::Tool(format!(
                    "{said:?} is not a language tag. Send one like `es`, `en` or `pt-BR`."
                ))
            }),
    }
}

fn unsaid(state: &State, which: &str) -> Refused {
    Refused::Tool(format!(
        "{} says no language yet. Write a `lang:` line in its properties with `edit_doc` first.",
        doc_named(state, which)
    ))
}

fn taken(state: &State, which: &str, lang: &str) -> Refused {
    Refused::Tool(format!(
        "{} already has a document in {lang}. Change that one instead, by its own `doc`.",
        doc_named(state, which)
    ))
}

/// A document written in another language of one already here, joined to it.
pub(in crate::mcp) fn translation_written(
    paths: &Paths,
    args: &Value,
    of: &str,
) -> Result<Value, Refused> {
    if text(args, "page_of").is_some() {
        return Err(Refused::Tool(
            "a translation is a document of its own, never a page: leave `page_of` out, and its pages are translated one by one."
                .into(),
        ));
    }
    let (state, _) = opened(paths)?;
    let up = found(&state, of)?;
    // Written here it would be the person's own, so someone else's words would go out under their name.
    if !state.mine(up) {
        return Err(Refused::Tool(format!(
            "{} is someone else's writing, so a translation of it would still be theirs and is not written as the person's own. The person can copy it in another language from the window.",
            doc_named(&state, of)
        )));
    }
    let Some(body) = text(args, "body") else {
        return Err(Refused::Tool("a translation needs a `body`.".into()));
    };
    let body = match language(args)? {
        Some(lang) => tisty_markdown::front::with_field(&body, "lang", &lang),
        None => body,
    };
    let Some(lang) = tisty_markdown::front::lang(&body) else {
        return Err(Refused::Tool(
            "say which language the translation is in, with `lang` or a `lang:` line in its properties."
                .into(),
        ));
    };
    if up.lang.is_none() {
        return Err(unsaid(&state, of));
    }
    if state.tongue_taken(up.id, &lang) {
        return Err(taken(&state, of, &lang));
    }
    let mut asked = args.clone();
    if let Some(one) = asked.as_object_mut() {
        one.remove("translation_of");
        one.remove("lang");
        one.insert("body".into(), json!(body));
        match up.folder {
            Some(at) => one.insert("folder".into(), json!(at.to_string())),
            None => one.remove("folder"),
        };
    }
    // The same subject in another language is the point, so the one-subject check is not asked.
    let written = written_doc(paths, &asked, Some(Vec::new()))?;
    let file = written["structuredContent"]["doc"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let (state, mut store) = opened(paths)?;
    let (Some(kept), Some(up)) = (
        state.papers().find(|one| one.file == file),
        state.papers().find(|one| one.file == of),
    ) else {
        return Ok(written);
    };
    // Already written, so a join that fails says so rather than inviting a second copy.
    if let Err(why) = store.append_batch(state.joining(kept.id, up.id)) {
        return Err(Refused::Tool(format!(
            "{} was written, but it could not be joined to {} ({why}). Join it with `tongue_doc`; do not write it again.",
            doc_named(&state, &file),
            doc_named(&state, of)
        )));
    }
    Ok(told(
        format!(
            "Wrote {} in {lang}, one of the languages of {}. The person sees them as one document and switches between them.",
            doc_named(&state, &file),
            doc_named(&state, of)
        ),
        json!({ "doc": file, "lang": lang, "translation_of": of }),
    ))
}

pub(in crate::mcp) fn tongue_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(which) = text(args, "doc") else {
        return Err(Refused::Tool("name the document in `doc`.".into()));
    };
    let lang = language(args)?;
    let (state, mut store) = opened(paths)?;
    let kept = found(&state, &which)?.clone();
    let Some(of) = text(args, "of") else {
        let ops = state.parting(kept.id);
        if !ops.is_empty() {
            store.append_batch(ops).map_err(hitch)?;
        }
        return Ok(told(
            format!(
                "{} stands on its own now; its other languages, if any, stay together.",
                doc_named(&state, &which)
            ),
            json!({ "doc": which }),
        ));
    };
    let up = found(&state, &of)?.clone();
    if let Some(sheet) = [&which, &of]
        .into_iter()
        .find(|one| super::super::ideas::sheet_named(paths, &state, one).is_some())
    {
        return Err(super::super::ideas::sheet_stays_put(&state, sheet));
    }
    if up.lang.is_none() {
        return Err(unsaid(&state, &of));
    }
    // Judged whole before anything is written, so a refusal leaves the document as it was.
    let (said, unwritten) = match (lang, kept.lang.clone()) {
        (Some(lang), was) if was.as_deref() != Some(lang.as_str()) => (lang, true),
        (Some(lang), _) => (lang, false),
        (None, Some(was)) => (was, false),
        (None, None) => {
            return Err(Refused::Tool(format!(
                "{} says no language. Send `lang` with the one it is in.",
                doc_named(&state, &which)
            )));
        }
    };
    if state.tongue_taken(up.id, &said) {
        return Err(taken(&state, &of, &said));
    }
    if unwritten {
        if state.shut(kept.id) {
            return Err(Refused::Tool(format!(
                "{} is locked, so its language cannot be written.",
                doc_named(&state, &which)
            )));
        }
        let whole = tisty_core::docs::amend(&paths.docs(), paths.data(), &which, |was| {
            Some(tisty_markdown::front::with_field(was, "lang", &said))
        })
        .map_err(hitch)?;
        if let Some(whole) = whole {
            retold(&state, &mut store, &which, &whole)?;
        }
    }
    let ops = state.joining(kept.id, up.id);
    if !ops.is_empty() {
        store.append_batch(ops).map_err(hitch)?;
    }
    Ok(told(
        format!(
            "{} is now the {said} of {}, in the same folder. The person sees them as one document and switches between them.",
            doc_named(&state, &which),
            doc_named(&state, &of)
        ),
        json!({ "doc": which, "lang": said, "of": of }),
    ))
}
