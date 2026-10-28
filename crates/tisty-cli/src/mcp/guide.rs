use serde_json::{Value, json};

use super::jsonrpc::told;
use super::looking::GUIDE;

// The reference's sections in reading order, named the same in both languages so one topic reads either.
const TOPICS: [&str; 25] = [
    "glance",
    "properties",
    "title",
    "headings",
    "paragraphs",
    "emphasis",
    "underline",
    "pens",
    "links",
    "lists",
    "callouts",
    "collapsible",
    "tags",
    "icons",
    "emoji",
    "documents",
    "attachments",
    "references",
    "footnotes",
    "code",
    "diagrams",
    "tables",
    "page-breaks",
    "comments",
    "outside",
];

// What Tisty adds to Markdown or reads its own way; the rest reads as Markdown does anywhere.
const ADDED: &[&str] = &[
    "properties",
    "paragraphs",
    "underline",
    "pens",
    "lists",
    "callouts",
    "collapsible",
    "tags",
    "icons",
    "documents",
    "attachments",
    "references",
    "diagrams",
    "tables",
    "page-breaks",
];

pub(super) fn topics() -> Vec<&'static str> {
    TOPICS.iter().copied().chain(["all"]).collect()
}

fn sections(reference: &str) -> Vec<(String, String)> {
    tisty_core::docs::outlined(reference)
        .into_iter()
        .filter(|one| one.level == 2)
        .map(|one| {
            let body = tisty_core::docs::lines_between(reference, one.line, one.to);
            (one.title, body.trim_end().to_string())
        })
        .collect()
}

pub(super) fn language(args: &Value) -> Value {
    let spanish = args["lang"].as_str() == Some("es");
    let drawn = match spanish {
        true => format!("{GUIDE}es/"),
        false => GUIDE.to_string(),
    };
    let reference = tisty_markdown::reference(spanish).trim_end();
    let found = sections(reference);
    let asked = args["topic"].as_str();
    let said = match asked {
        Some("all") => reference.to_string(),
        Some(topic) => match TOPICS.iter().position(|one| *one == topic) {
            Some(at) if at < found.len() => found[at].1.clone(),
            _ => format!("No topic is called {topic:?}.\n\n{}", indexed(&found)),
        },
        None => indexed(&found),
    };
    told(
        format!("{said}\n\nThe guide for people, with every example drawn: {drawn}\n"),
        json!({
            "language": said,
            "topic": asked.unwrap_or("index"),
            "lang": if spanish { "es" } else { "en" },
            "guide": drawn,
        }),
    )
}

fn indexed(found: &[(String, String)]) -> String {
    let row = |at: usize| -> Option<String> {
        let title = &found.get(at)?.0;
        Some(format!("- {}: {title}", TOPICS[at]))
    };
    let pick = |added: bool| -> String {
        (1..TOPICS.len())
            .filter(|at| ADDED.contains(&TOPICS[*at]) == added)
            .filter_map(row)
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "Plain Markdown — CommonMark with GitHub's additions — is taken as it is and turned into the \
         Tisty language on the way in, so none of this is needed to write. Ask `language` with a \
         `topic` for one of these when you want it, or «all» for the whole guide.\n\n\
         What Tisty adds, or reads its own way:\n{}\n\nMarkdown as anywhere:\n{}",
        pick(true),
        pick(false)
    )
}

#[cfg(test)]
#[path = "guide_test.rs"]
mod tests;
