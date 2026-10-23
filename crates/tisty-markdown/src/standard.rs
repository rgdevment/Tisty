use std::cmp::Reverse;
use std::ops::Range;

use crate::{Block, Callout, Kind, Mark, refs};

/// Where a link to another document lands outside Tisty; nothing found leaves the words alone.
pub enum Toward<'a> {
    Doc(&'a str),
    Ref(&'a str),
}

type Edits = Vec<(Range<usize>, String)>;

/// The Tisty language as GitHub and CommonMark read it, with the tags any reader passes over for the rest.
pub fn to_standard(text: &str, toward: &dyn Fn(Toward) -> Option<String>) -> String {
    let doc = crate::reading::read(text);
    let mut edits = Vec::new();
    for block in doc.blocks() {
        outward(block, text, toward, &mut edits);
    }
    edits.sort_by_key(|(at, _)| (Reverse(at.start), Reverse(at.end)));
    let mut out = text.to_string();
    for (at, said) in edits {
        out.replace_range(at, &said);
    }
    fronted(text, &out)
}

fn outward(
    block: &Block,
    text: &str,
    toward: &dyn Fn(Toward) -> Option<String>,
    edits: &mut Edits,
) {
    // A block followed by more words carries its line end, and the fence that closes it sits before.
    let said = text[block.at.clone()].trim_end_matches(['\r', '\n']);
    let start = block.at.start;
    match &block.kind {
        Kind::Callout(kind) if said.starts_with(":::") => quoted(said, start, *kind, edits),
        Kind::Details { summary, .. } if said.starts_with(":::") => {
            let first = said.find('\n').unwrap_or(said.len());
            let last = said.rfind('\n').map_or(said.len(), |at| at + 1);
            edits.push((
                start..start + first,
                format!("<details>\n<summary>{}</summary>", escaped(summary)),
            ));
            edits.push((start + last..start + said.len(), "</details>".to_string()));
        }
        Kind::Callout(_) => headed(said, start, edits),
        // Tisty draws `mmd` too; GitHub and GitLab draw only a fence that says `mermaid`.
        Kind::Code {
            tongue: Some(tongue),
            ..
        } if tongue == "mmd" => {
            let line = &said[..said.find('\n').unwrap_or(said.len())];
            let fence = line.len() - line.trim_start_matches([' ', '`', '~']).len();
            if let Some(at) = line[fence..].find("mmd") {
                let at = start + fence + at;
                edits.push((at..at + 3, "mermaid".to_string()));
            }
        }
        _ => {}
    }
    for span in &block.spans {
        spanned(span, text, toward, edits);
    }
    for inner in &block.inner {
        outward(inner, text, toward, edits);
    }
}

fn spanned(
    span: &crate::Span,
    text: &str,
    toward: &dyn Fn(Toward) -> Option<String>,
    edits: &mut Edits,
) {
    let said = &text[span.at.clone()];
    let (start, end) = (span.at.start, span.at.end);
    match &span.mark {
        Mark::Underline if said.starts_with("++") => {
            edits.push((start..start + 2, "<ins>".to_string()));
            edits.push((end - 2..end, "</ins>".to_string()));
        }
        Mark::Pen { color } if said.starts_with("==") => {
            let Some(shut) = said.rfind("==") else {
                return;
            };
            let open = match color {
                Some(color) => format!("<mark data-pen=\"{color}\">"),
                None => "<mark>".to_string(),
            };
            edits.push((start..start + 2, open));
            edits.push((start + shut..end, "</mark>".to_string()));
        }
        Mark::Icon { name, hue } if said.starts_with("::") => {
            let tint = hue
                .as_deref()
                .map(|hue| format!(" data-hue=\"{hue}\""))
                .unwrap_or_default();
            edits.push((
                span.at.clone(),
                format!("<span data-ico=\"{name}\"{tint}>:{name}:</span>"),
            ));
        }
        Mark::Emoji { glyph } => edits.push((span.at.clone(), glyph.clone())),
        // A single line break is a break in Tisty, and CommonMark would join the two lines.
        Mark::Break if !said.starts_with(['\\', ' ']) => edits.push((start..start, "\\".into())),
        Mark::Ref { name } if said.starts_with("[[") => {
            let words = match toward(Toward::Ref(name)) {
                Some(at) => format!("[{name}](<{at}>)"),
                None => name.clone(),
            };
            edits.push((span.at.clone(), words));
        }
        Mark::Link { to } | Mark::Image { src: to } => {
            if let Some(file) = to.strip_prefix(refs::DOC) {
                carded(span, said, toward(Toward::Doc(file)), edits);
            } else if to.starts_with(refs::TASK) {
                carded(span, said, None, edits);
            }
        }
        _ => {}
    }
}

// Only the brackets and the target change, so whatever is marked inside the words keeps its own edits.
fn carded(span: &crate::Span, said: &str, at: Option<String>, edits: &mut Edits) {
    let (start, end) = (span.at.start, span.at.end);
    let Some(close) = said.rfind("](") else {
        return;
    };
    let pictured = usize::from(said.starts_with('!'));
    match at {
        Some(at) => {
            edits.push((start..start + pictured, String::new()));
            edits.push((start + close + 2..end - 1, format!("<{at}>")));
        }
        None => {
            edits.push((start..start + pictured + 1, String::new()));
            edits.push((start + close..end, String::new()));
        }
    }
}

fn quoted(said: &str, start: usize, kind: Callout, edits: &mut Edits) {
    let first = said.find('\n').unwrap_or(said.len());
    let last = said.rfind('\n').unwrap_or(said.len());
    let name = match kind {
        Callout::Note => "NOTE",
        Callout::Tip => "TIP",
        Callout::Important => "IMPORTANT",
        Callout::Warning => "WARNING",
        Callout::Caution => "CAUTION",
    };
    edits.push((start..start + first, format!("> [!{name}]")));
    for (at, _) in said.match_indices('\n').filter(|(at, _)| *at < last) {
        let line = at + 1;
        let blank = said[line..].starts_with('\n');
        edits.push((
            start + line..start + line,
            if blank { ">" } else { "> " }.into(),
        ));
    }
    edits.push((start + last..start + said.len(), String::new()));
}

// GitHub reads a callout only when its marker stands alone, so the words after it go to the next line.
fn headed(said: &str, start: usize, edits: &mut Edits) {
    let line = said.split('\n').next().unwrap_or_default();
    let Some(open) = line.find("[!") else {
        return;
    };
    let Some(shut) = line[open..].find(']').map(|at| open + at + 1) else {
        return;
    };
    let rest = &line[shut..];
    let words = rest.trim_start();
    if words.trim().is_empty() {
        return;
    }
    let gap = rest.len() - words.len();
    edits.push((start + shut..start + shut + gap, "\n> ".into()));
}

fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// A document's tags are its front matter's and its hashtags together, and a reader outside sees them in one place.
fn fronted(text: &str, out: &str) -> String {
    let tags = crate::tags::tags_in(text);
    if tags.is_empty() {
        return out.to_string();
    }
    let listed = format!("tags: [{}]", tags.join(", "));
    let Some(at) = crate::front::span(out) else {
        return format!("---\n{listed}\n---\n\n{out}");
    };
    let block = &out[at.clone()];
    let mut kept: Vec<&str> = Vec::new();
    let mut tagging = false;
    let lines: Vec<&str> = block.lines().collect();
    let shut = lines.len().saturating_sub(1);
    for (n, line) in lines.iter().enumerate() {
        if n == shut {
            break;
        }
        let key = line.split_once(':').map(|(key, _)| key.trim());
        if !line.starts_with([' ', '\t', '-']) {
            tagging = key.is_some_and(crate::front::tagging);
        }
        if !tagging {
            kept.push(line);
        }
    }
    kept.push(&listed);
    kept.push(lines.get(shut).copied().unwrap_or("---"));
    let ending = if block.ends_with('\n') { "\n" } else { "" };
    format!(
        "{}{}{ending}{}",
        &out[..at.start],
        kept.join("\n"),
        &out[at.end..]
    )
}
