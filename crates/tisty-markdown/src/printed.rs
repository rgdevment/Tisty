use serde_json::{Value, json};

use crate::inline::{alt, atom, code_words, holds, outside, paired, walls};
use crate::{Block, Callout, Column, Doc, Kind, Lean, Mark, Span};

pub fn printed(doc: &Doc) -> Value {
    let content: Vec<Value> = doc
        .blocks()
        .iter()
        .filter_map(|one| node(one, doc.text()))
        .collect();
    json!({ "type": "doc", "content": content })
}

fn node(block: &Block, text: &str) -> Option<Value> {
    let inner = |block: &Block| -> Vec<Value> {
        block
            .inner
            .iter()
            .filter_map(|one| node(one, text))
            .collect()
    };
    Some(match &block.kind {
        Kind::Heading(level) => json!({
            "type": "heading",
            "attrs": { "level": level },
            "content": runs(&block.spans, text),
        }),
        Kind::Paragraph => json!({ "type": "paragraph", "content": runs(&block.spans, text) }),
        Kind::Quote => json!({ "type": "blockquote", "content": inner(block) }),
        Kind::Callout(kind) => json!({
            "type": "callout",
            "attrs": { "kind": called(*kind) },
            "content": inner(block),
        }),
        Kind::List { start } => listed(block, *start, text),
        Kind::Item { .. } => return None,
        Kind::Code { tongue, .. } => {
            let said: String = block
                .spans
                .iter()
                .map(|one| &text[one.at.clone()])
                .collect();
            json!({
                "type": "codeBlock",
                "attrs": { "language": tongue },
                "content": [{ "type": "text", "text": said }],
            })
        }
        Kind::Table { columns } => tabled(block, columns, text),
        Kind::Rule => json!({ "type": "horizontalRule" }),
        Kind::Details { summary, .. } => {
            let mut content = vec![json!({
                "type": "paragraph",
                "content": [{ "type": "text", "text": summary, "marks": [{ "type": "bold" }] }],
            })];
            content.extend(inner(block));
            json!({ "type": "blockquote", "content": content })
        }
        Kind::Note { label } => {
            let mut content = vec![json!({
                "type": "paragraph",
                "content": [{ "type": "text", "text": label, "marks": [{ "type": "bold" }] }],
            })];
            content.extend(inner(block));
            json!({ "type": "blockquote", "content": content })
        }
        Kind::Html | Kind::Row { .. } | Kind::Cell => return None,
    })
}

fn called(kind: Callout) -> &'static str {
    match kind {
        Callout::Note => "note",
        Callout::Tip => "tip",
        Callout::Important => "important",
        Callout::Warning => "warning",
        Callout::Caution => "caution",
    }
}

fn listed(block: &Block, start: Option<u64>, text: &str) -> Value {
    let ticked = block
        .inner
        .iter()
        .any(|one| matches!(one.kind, Kind::Item { done: Some(_) }));
    let items: Vec<Value> = block
        .inner
        .iter()
        .map(|item| {
            let done = match item.kind {
                Kind::Item { done } => done,
                _ => None,
            };
            let mut content = Vec::new();
            if !item.spans.is_empty() {
                content.push(json!({ "type": "paragraph", "content": runs(&item.spans, text) }));
            }
            content.extend(item.inner.iter().filter_map(|one| node(one, text)));
            if ticked {
                json!({ "type": "taskItem", "attrs": { "checked": done == Some(true) }, "content": content })
            } else {
                json!({ "type": "listItem", "content": content })
            }
        })
        .collect();
    match (ticked, start) {
        (true, _) => json!({ "type": "taskList", "content": items }),
        (false, Some(first)) => {
            json!({ "type": "orderedList", "attrs": { "start": first }, "content": items })
        }
        (false, None) => json!({ "type": "bulletList", "content": items }),
    }
}

fn tabled(block: &Block, columns: &[Column], text: &str) -> Value {
    let rows: Vec<Value> =
        block
            .inner
            .iter()
            .map(|row| {
                let cells: Vec<Value> = row
                .inner
                .iter()
                .enumerate()
                .map(|(at, cell)| {
                    let lean = columns.get(at).and_then(|one| one.lean).map(|lean| match lean {
                        Lean::Left => "left",
                        Lean::Center => "center",
                        Lean::Right => "right",
                    });
                    json!({
                        "type": "tableCell",
                        "attrs": { "textAlign": lean },
                        "content": [{ "type": "paragraph", "content": runs(&cell.spans, text) }],
                    })
                })
                .collect();
                json!({ "type": "tableRow", "content": cells })
            })
            .collect();
    json!({ "type": "table", "content": rows })
}

fn marked(mark: &Mark) -> Option<Value> {
    match mark {
        Mark::Strong => Some(json!({ "type": "bold" })),
        Mark::Emphasis => Some(json!({ "type": "italic" })),
        Mark::Strike => Some(json!({ "type": "strike" })),
        Mark::Underline => Some(json!({ "type": "underline" })),
        Mark::Link { to } if crate::nowhere(to) => None,
        Mark::Link { to } => Some(json!({ "type": "link", "attrs": { "href": to } })),
        Mark::Pen { color } => Some(json!({
            "type": "highlight",
            "attrs": { "color": color.as_deref().unwrap_or("yellow") },
        })),
        _ => None,
    }
}

fn runs(spans: &[Span], text: &str) -> Vec<Value> {
    let atoms: Vec<&Span> = spans.iter().filter(|one| atom(&one.mark)).collect();
    let mut silent: Vec<std::ops::Range<usize>> = atoms.iter().map(|one| one.at.clone()).collect();
    silent.extend(walls(spans, text));
    let mut items: Vec<(usize, Option<&Span>, std::ops::Range<usize>)> = atoms
        .iter()
        .map(|one| (one.at.start, Some(*one), one.at.clone()))
        .collect();
    for span in spans.iter().filter(|one| one.mark == Mark::Text) {
        for piece in outside(span.at.clone(), &silent) {
            items.push((piece.start, None, piece));
        }
    }
    items.sort_by_key(|(at, ..)| *at);
    let kept = paired(spans, text);
    let mut out = Vec::new();
    for (at, one, piece) in items {
        let marks: Vec<Value> = spans
            .iter()
            .filter(|span| holds(&span.mark) && span.at.start <= at && at < span.at.end)
            .filter_map(|span| marked(&span.mark))
            .collect();
        let said = &text[piece];
        let Some(span) = one else {
            out.push(json!({ "type": "text", "text": decoded(said), "marks": marks }));
            continue;
        };
        let (words, code) = match &span.mark {
            Mark::Code => (code_words(said), true),
            Mark::Break => {
                out.push(json!({ "type": "hardBreak" }));
                continue;
            }
            Mark::Image { src } => {
                out.push(json!({
                    "type": "image",
                    "attrs": { "src": src, "alt": alt(span, spans, text) },
                }));
                continue;
            }
            Mark::Ref { name } => (name.clone(), false),
            Mark::NoteRef { label } => (format!("[{label}]"), false),
            Mark::Html if kept.contains(&at) => {
                if said.to_ascii_lowercase().starts_with("<br") {
                    out.push(json!({ "type": "hardBreak" }));
                }
                continue;
            }
            Mark::Html if said.starts_with("<!--") => continue,
            Mark::Icon { .. } => continue,
            _ => (said.to_string(), false),
        };
        if words.is_empty() {
            continue;
        }
        let mut marks = marks;
        if code {
            marks.push(json!({ "type": "code" }));
        }
        out.push(json!({ "type": "text", "text": words, "marks": marks }));
    }
    out
}

// An entity reaches the sheet as its own piece of source; the window decodes it, paper must too.
fn decoded(said: &str) -> String {
    let Some(inner) = said
        .strip_prefix('&')
        .and_then(|rest| rest.strip_suffix(';'))
    else {
        return said.to_string();
    };
    let number = match inner.strip_prefix('#') {
        Some(hex) if hex.starts_with(['x', 'X']) => u32::from_str_radix(&hex[1..], 16).ok(),
        Some(digits) => digits.parse().ok(),
        None => None,
    };
    if let Some(one) = number.and_then(char::from_u32) {
        return one.to_string();
    }
    match inner {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        "nbsp" => "\u{a0}",
        "copy" => "©",
        "reg" => "®",
        "trade" => "™",
        "mdash" => "—",
        "ndash" => "–",
        "hellip" => "…",
        "laquo" => "«",
        "raquo" => "»",
        "middot" => "·",
        "deg" => "°",
        "euro" => "€",
        _ => said,
    }
    .to_string()
}
