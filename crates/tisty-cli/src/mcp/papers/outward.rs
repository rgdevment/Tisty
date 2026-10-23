use serde_json::{Map, Value, json};

use super::super::Refused;
use super::super::asked::text;
use super::super::jsonrpc::told;

// Past this a document leaves as files with `export_doc`, not through the conversation.
const AT_MOST: usize = 100_000;
const PARTS: [&str; 5] = ["section", "from", "to", "chars", "cursor"];

#[derive(Clone, Copy)]
pub(super) enum Shape {
    Markdown,
    Html,
}

impl Shape {
    fn said(self) -> &'static str {
        match self {
            Shape::Markdown => "markdown",
            Shape::Html => "html",
        }
    }
}

pub(super) fn asked(args: &Value) -> Result<Option<Shape>, Refused> {
    let shape = match text(args, "as").as_deref() {
        None => return Ok(None),
        Some("markdown") => Shape::Markdown,
        Some("html") => Shape::Html,
        Some(other) => {
            return Err(Refused::Tool(format!(
                "`as` takes \"markdown\" or \"html\", not {other:?}. Leave it out to read the \
                 document in Tisty's own words."
            )));
        }
    };
    if let Some(part) = PARTS
        .into_iter()
        .find(|key| args.get(*key).is_some_and(|one| !one.is_null()))
    {
        return Err(Refused::Tool(format!(
            "`as` brings the whole document, ready to carry outside Tisty, so it takes no \
             `{part}`. Read a part without `as`, in Tisty's own words."
        )));
    }
    Ok(Some(shape))
}

/// The document turned for another place, with what does not make the journey said out loud.
pub(super) fn carried(
    body: &str,
    shape: Shape,
    pages: usize,
    mut kept_of: Map<String, Value>,
) -> Result<Value, Refused> {
    let turned = match shape {
        Shape::Markdown => tisty_markdown::to_standard(body, &|_| None),
        Shape::Html => tisty_markdown::html(&tisty_markdown::read(body)),
    };
    if turned.chars().count() > AT_MOST {
        return Err(Refused::Tool(format!(
            "turned into {}, this document is past the {AT_MOST} characters a conversation \
             carries well. `export_doc` writes it out as files instead.",
            shape.said()
        )));
    }
    let lost = lost(body);
    let mut warned = lost.words(shape);
    if pages > 0 {
        warned.push(format!(
            "Its {pages} page(s) are not in this: read each one with `as` too."
        ));
    }
    if !lost.is_empty() {
        kept_of.insert("left_out".into(), lost.told());
    }
    kept_of.insert("as".into(), json!(shape.said()));
    kept_of.insert("body".into(), json!(turned));
    let head = match warned.is_empty() {
        true => String::new(),
        false => format!("{}\n\n", warned.join(" ")),
    };
    Ok(told(
        format!(
            "{} as {}, to carry outside Tisty. {head}{turned}",
            tisty_core::docs::titled(body),
            shape.said()
        ),
        Value::Object(kept_of),
    ))
}

#[derive(Default)]
struct Lost {
    files: Vec<Value>,
    diagrams: usize,
    named: usize,
}

impl Lost {
    fn is_empty(&self) -> bool {
        self.files.is_empty() && self.diagrams == 0 && self.named == 0
    }

    fn told(&self) -> Value {
        json!({ "files": self.files, "diagrams": self.diagrams, "named": self.named })
    }

    fn words(&self, shape: Shape) -> Vec<String> {
        let mut said = Vec::new();
        if !self.files.is_empty() {
            said.push(format!(
                "{} attached file(s) are named by their place in Tisty and do not travel with the \
                 text, listed in `left_out.files`: `export_doc` writes the document out with them \
                 into a folder, and from there the other place's own tool can upload each.",
                self.files.len()
            ));
        }
        if self.diagrams > 0 {
            said.push(match shape {
                Shape::Markdown => format!(
                    "{} diagram(s) stay as mermaid fences: GitHub, GitLab and Notion draw them, \
                     Confluence and Jira show them as code unless a Mermaid macro is used.",
                    self.diagrams
                ),
                Shape::Html => format!(
                    "{} diagram(s) are drawn as data: images, which Confluence and many editors \
                     drop.",
                    self.diagrams
                ),
            });
        }
        if self.named > 0 {
            said.push(format!(
                "{} link(s) to other documents or tasks in Tisty became plain words.",
                self.named
            ));
        }
        said
    }
}

// Answers never carry a path on this disk, so a file is named by its place in Tisty.
fn lost(body: &str) -> Lost {
    let mut lost = Lost::default();
    let mut seen = std::collections::BTreeSet::new();
    for one in tisty_core::refs::extract(body) {
        if one.target.starts_with("attachments/") && seen.insert(one.target.clone()) {
            lost.files.push(json!(one.target));
        }
    }
    for block in tisty_markdown::read(body).blocks() {
        counted(block, &mut lost);
    }
    lost
}

fn counted(block: &tisty_markdown::Block, lost: &mut Lost) {
    if let tisty_markdown::Kind::Code {
        tongue: Some(tongue),
        ..
    } = &block.kind
        && matches!(tongue.as_str(), "mermaid" | "mmd")
    {
        lost.diagrams += 1;
    }
    for span in &block.spans {
        let elsewhere = |to: &str| {
            to.starts_with(tisty_markdown::refs::DOC) || to.starts_with(tisty_markdown::refs::TASK)
        };
        match &span.mark {
            tisty_markdown::Mark::Ref { .. } => lost.named += 1,
            tisty_markdown::Mark::Link { to } | tisty_markdown::Mark::Image { src: to }
                if elsewhere(to) =>
            {
                lost.named += 1;
            }
            _ => {}
        }
    }
    for inner in &block.inner {
        counted(inner, lost);
    }
}
