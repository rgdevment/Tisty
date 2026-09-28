use serde_json::{Value, json};

use super::writing::{beside_the_file, said_of, where_it_lands, write_doc};

use super::reachable_or;
use tisty_core::Paths;

use super::super::asked::{body_at_most, short_and_plain, text};
use super::super::jsonrpc::told;
use super::super::{Refused, opened};

pub(in crate::mcp) fn import_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "path") else {
        return Err(Refused::Tool(
            "importing needs a `path` to a markdown file on this machine.".into(),
        ));
    };
    let asked = std::path::Path::new(&said);
    let at = reachable_or(paths, &said, asked, "take files from")?;
    if !at.is_file() {
        return Err(Refused::Tool(format!(
            "{said:?} is not a file. `import_doc` takes one markdown file at a time; call it once per file when you are bringing a whole export across."
        )));
    }
    if !at
        .extension()
        .is_some_and(|one| one.eq_ignore_ascii_case("md") || one.eq_ignore_ascii_case("markdown"))
    {
        return Err(Refused::Tool(format!(
            "{said:?} is not markdown. A document is markdown; anything else goes in with `attach`, which keeps it beside a document or a task."
        )));
    }
    let big = std::fs::metadata(&at).map(|one| one.len()).unwrap_or(0);
    if big > tisty_core::docs::BODY_AT_MOST {
        return Err(Refused::Tool(format!(
            "{said:?} is {big} bytes, past the {} a file may be to be read at all. Split it \
             before bringing it in.",
            tisty_core::docs::BODY_AT_MOST
        )));
    }
    let raw = std::fs::read(&at)
        .map_err(|why| Refused::Tool(format!("{said:?} could not be read: {why}.")))?;
    let raw = String::from_utf8(raw).map_err(|_| {
        Refused::Tool(format!(
            "{said:?} is not text this can read. Tisty keeps documents as UTF-8."
        ))
    })?;

    let looks =
        tisty_core::agent::secret_in(raw.as_bytes()).map(|one| match one.named.is_empty() {
            true => one.why.to_string(),
            false => one.named,
        });

    let made = tisty_core::arriving::tidied(&raw);
    tisty_core::docs::survives(&made.body).map_err(|eats| {
        Refused::Tool(format!(
            "{said:?} still holds {eats} after being tidied, so it would be destroyed the first time the person opens it. Nothing was written."
        ))
    })?;
    short_and_plain(&json!({ "body": made.body }))?;
    // The files beside it are copied in below, and a document turned away after that would
    // leave them on the person's disk with nothing pointing at them.
    where_it_lands(&opened(paths)?.0, args)?;
    let (whole, brought) = beside_the_file(paths, &at, &made.body);
    let made = tisty_core::arriving::Tidied {
        body: whole,
        changed: made.changed,
    };

    let headed = made
        .body
        .lines()
        .find(|one| !one.trim().is_empty())
        .is_some_and(|one| one.starts_with("# "));
    let body = match (text(args, "title"), headed) {
        (Some(said), _) => format!(
            "# {said}

{}",
            made.body.trim_start()
        ),
        (None, true) => made.body.clone(),
        (None, false) => {
            let named = at.file_stem().unwrap_or_default().to_string_lossy();
            format!(
                "# {named}

{}",
                made.body.trim_start()
            )
        }
    };

    let mut asked_again = args.clone();
    if let Some(one) = asked_again.as_object_mut() {
        one.remove("path");
        one.remove("title");
        one.insert("body".into(), json!(body));
    }
    let long = body.chars().count();
    if long > body_at_most() {
        return Err(Refused::Tool(format!(
            "{said:?} reads as {long} characters once it is tidied, and a document is kept up \
             to {}. Split it before bringing it in.",
            body_at_most()
        )));
    }
    short_and_plain(&asked_again)?;
    let written = write_doc(paths, &asked_again)?;

    let changed = made.changed.join(", ");
    Ok(told(
        format!(
            "{}{}{}{}{}{}",
            said_of(&written),
            match made.changed.is_empty() {
                true => " Nothing had to be changed on the way in.".to_string(),
                false => format!(
                    " On the way in it was tidied: {changed}. What the file said is still on disk, untouched."
                ),
            },
            match brought.kept {
                0 => String::new(),
                one => format!(
                    " {one} file(s) beside it came in too, and the text now points at the copies Tisty keeps."
                ),
            },
            match brought.missed.is_empty() {
                true => String::new(),
                false => format!(
                    " These could not come in, so their links were taken out rather than left pointing outside Tisty — the words that named them are still in the text: {}.",
                    brought.missed.join("; ")
                ),
            },
            match &looks {
                None => String::new(),
                Some(said) => format!(
                    " Nothing was held back: what reads like a live credential ({said}) came in as written, under a warning the person sees when they open it."
                ),
            },
            match brought.papers.is_empty() {
                true => String::new(),
                false => format!(
                    " It also names {} other markdown file(s) — those are documents, not files to keep: import each one and the links will still read as text until you tie them together.",
                    brought.papers.len()
                ),
            }
        ),
        match written["structuredContent"].clone() {
            Value::Object(mut one) => {
                one.insert("from".into(), json!(at.display().to_string()));
                one.insert("changed".into(), json!(made.changed));
                one.insert("files".into(), json!(brought.kept));
                one.insert("left_behind".into(), json!(brought.missed));
                one.insert("names_markdown".into(), json!(brought.papers));
                one.insert("reads_like_a_credential".into(), json!(looks));
                Value::Object(one)
            }
            other => other,
        },
    ))
}
