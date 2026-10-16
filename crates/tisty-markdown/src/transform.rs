use std::ops::Range;

use crate::{Block, Mark, Span, kept, vocab};

/// Tisty's own words for what the old tags said, every other byte left as it was written.
pub fn to_tisty(text: &str) -> String {
    let doc = crate::reading::read_old(text);
    let mut edits = Vec::new();
    for block in doc.blocks() {
        gather(block, text, &mut edits);
    }
    edits.sort_by_key(|(at, _)| std::cmp::Reverse(at.start));
    let mut out = text.to_string();
    for (at, said) in edits {
        out.replace_range(at, &said);
    }
    out
}

fn gather(block: &Block, text: &str, edits: &mut Vec<(Range<usize>, String)>) {
    for span in &block.spans {
        let said = &text[span.at.clone()];
        let tagged = said.starts_with('<');
        match &span.mark {
            Mark::Link { to: target } | Mark::Image { src: target } => {
                linked(span, text, target, edits)
            }
            _ if !tagged => {}
            // A colour the palette lacks has no Tisty word, so its tag stays and the page still paints it.
            Mark::Pen { color } if color.as_deref().is_none_or(vocab::pen) => {
                let shut = match color {
                    Some(color) => format!("=={{{color}}}"),
                    None => "==".to_string(),
                };
                walled(span, text, "==", &shut, edits);
            }
            Mark::Underline if !inside_a_word(span, text, '+') => {
                walled(span, text, "++", "++", edits)
            }
            Mark::Icon { name, hue } if vocab::icon(name) => {
                let said = match hue.as_deref().filter(|one| vocab::hue(one)) {
                    Some(hue) => format!("::{name} {hue}::"),
                    None => format!("::{name}::"),
                };
                let before = text[..span.at.start].chars().next_back();
                let after = text[span.at.end..].chars().next();
                let room = |at: Option<char>| match at.is_some_and(char::is_alphanumeric) {
                    true => " ",
                    false => "",
                };
                edits.push((
                    span.at.clone(),
                    format!("{}{said}{}", room(before), room(after)),
                ));
            }
            _ => {}
        }
    }
    for inner in &block.inner {
        gather(inner, text, edits);
    }
}

// An underline the short walls cannot hold, mid-word or around a `++`, keeps the tag the page still reads.
fn inside_a_word(span: &Span, text: &str, wall: char) -> bool {
    let whole = &text[span.at.clone()];
    let held = |at: Option<char>| at.is_some_and(|c| c.is_alphanumeric() || c == wall);
    whole.contains("++")
        || held(text[..span.at.start].chars().next_back())
        || held(text[span.at.end..].chars().next())
}

// The new walls cannot touch a space on their inner side, so any space the old tags held moves out.
fn walled(
    span: &Span,
    text: &str,
    opens: &str,
    shuts: &str,
    edits: &mut Vec<(Range<usize>, String)>,
) {
    let whole = &text[span.at.clone()];
    let (Some(open), Some(close)) = (whole.find('>'), whole.rfind("</")) else {
        return;
    };
    let inner = &whole[open + 1..close];
    let words = inner.trim();
    let start = span.at.start;
    if words.is_empty() {
        edits.push((span.at.clone(), inner.to_string()));
        return;
    }
    let lead = inner.len() - inner.trim_start().len();
    let trail = inner.len() - inner.trim_end().len();
    edits.push((
        start..start + open + 1 + lead,
        format!("{}{opens}", &inner[..lead]),
    ));
    edits.push((
        start + close - trail..span.at.end,
        format!("{shuts}{}", &inner[inner.len() - trail..]),
    ));
}

// An attachment is named by its stamped name alone, wherever an older body said it lay.
fn linked(span: &Span, text: &str, target: &str, edits: &mut Vec<(Range<usize>, String)>) {
    let Some(leaf) = kept::leaf_of(target).filter(|leaf| *leaf != target) else {
        return;
    };
    let said = &text[span.at.clone()];
    let Some(open) = said.rfind("](") else {
        return;
    };
    let start = span.at.start + open + 2;
    let rest = &text[start..span.at.end];
    let written = match rest.strip_prefix('<') {
        Some(inner) => inner.find('>').map(|shut| shut + 2),
        None => rest.find([' ', ')']),
    };
    if let Some(written) = written {
        edits.push((start..start + written, leaf.to_string()));
    }
}

/// An underline goes, since Markdown has no word for it and the tag is what a stricter reader drops.
pub fn to_plain(text: &str) -> String {
    let doc = crate::reading::read(text);
    let mut edits = Vec::new();
    for block in doc.blocks() {
        outward(block, text, &mut edits);
    }
    edits.sort_by_key(|(at, _)| std::cmp::Reverse(at.start));
    let mut out = text.to_string();
    for (at, said) in edits {
        out.replace_range(at, &said);
    }
    out
}

fn outward(block: &Block, text: &str, edits: &mut Vec<(Range<usize>, String)>) {
    for span in &block.spans {
        let said = &text[span.at.clone()];
        let (start, end) = (span.at.start, span.at.end);
        match &span.mark {
            Mark::Underline if said.starts_with("++") => {
                edits.push((start..start + 2, String::new()));
                edits.push((end - 2..end, String::new()));
            }
            Mark::Pen { color } if said.starts_with("==") => {
                let Some(shut) = said.rfind("==") else {
                    continue;
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
            _ => {}
        }
    }
    for inner in &block.inner {
        outward(inner, text, edits);
    }
}
