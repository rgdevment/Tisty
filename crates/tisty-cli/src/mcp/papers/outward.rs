use serde_json::{Map, Value, json};

use super::super::Refused;
use super::super::jsonrpc::told;

// A document carried whole still has to fit a conversation; past this it leaves as files.
const AT_MOST: usize = 40_000;
const PARTS: [&str; 4] = ["section", "from", "to", "chars"];

/// Whether this read is one to carry outside Tisty: only standard Markdown, which other tools turn into their own.
pub(super) fn asked(args: &Value) -> Result<bool, Refused> {
    let Some(shape) = args.get("as").filter(|one| !one.is_null()) else {
        return Ok(false);
    };
    if shape.as_str() != Some("markdown") {
        return Err(Refused::Tool(format!(
            "`as` takes \"markdown\", not {shape}. Leave it out to read the document in Tisty's \
             own words."
        )));
    }
    if let Some(part) = PARTS
        .into_iter()
        .find(|key| args.get(*key).is_some_and(|one| !one.is_null()))
    {
        return Err(Refused::Tool(format!(
            "`as` brings the whole document, ready to carry outside Tisty, so it takes no \
             `{part}`. Read a part without `as`, in Tisty's own words."
        )));
    }
    Ok(true)
}

/// The document in standard Markdown, with what does not make the journey said out loud.
pub(super) fn carried(
    body: &str,
    standing: Option<String>,
    pages: usize,
    mut kept_of: Map<String, Value>,
) -> Result<Value, Refused> {
    let turned = tisty_markdown::to_standard(body, &|_| None);
    if turned.chars().count() > AT_MOST {
        return Err(Refused::Tool(format!(
            "in standard Markdown this document is past the {AT_MOST} characters a conversation \
             carries well. `export_doc` writes it out as files instead."
        )));
    }
    let lost = lost(body);
    let mut warned = standing
        .map(|said| {
            vec![format!(
                "{said}. Ask the person before carrying it anywhere."
            )]
        })
        .unwrap_or_default();
    warned.extend(lost.words());
    if pages > 0 {
        warned.push(format!(
            "Its {pages} page(s) are not in this: read each one with `as` too."
        ));
    }
    if !lost.is_empty() {
        kept_of.insert("left_out".into(), lost.told());
    }
    kept_of.insert("as".into(), json!("markdown"));
    kept_of.insert("body".into(), json!(turned));
    let head = match warned.is_empty() {
        true => String::new(),
        false => format!("{}\n\n", warned.join(" ")),
    };
    Ok(told(
        format!(
            "{} as markdown, to carry outside Tisty. {head}{turned}",
            tisty_core::docs::titled(body)
        ),
        Value::Object(kept_of),
    ))
}

#[derive(Default)]
struct Lost {
    files: Vec<String>,
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

    fn words(&self) -> Vec<String> {
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
            said.push(format!(
                "{} diagram(s) stay as mermaid fences: GitHub, GitLab and Notion draw them, \
                 Confluence and Jira show them as code unless a Mermaid macro is used.",
                self.diagrams
            ));
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

// Read from the parsed document, so a link shown inside code is never taken for one; answers carry no paths.
fn lost(body: &str) -> Lost {
    let mut lost = Lost::default();
    for block in tisty_markdown::read(body).blocks() {
        counted(block, &mut lost);
    }
    lost.files.sort();
    lost.files.dedup();
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
        match &span.mark {
            tisty_markdown::Mark::Ref { .. } => lost.named += 1,
            tisty_markdown::Mark::Link { to } | tisty_markdown::Mark::Image { src: to } => {
                if to.starts_with(tisty_markdown::refs::DOC)
                    || to.starts_with(tisty_markdown::refs::TASK)
                {
                    lost.named += 1;
                } else if let Some(at) = tisty_core::attach::canonical(to)
                    && tisty_core::attach::names_an_attachment(&at)
                {
                    lost.files.push(at);
                }
            }
            _ => {}
        }
    }
    for inner in &block.inner {
        counted(inner, lost);
    }
}
