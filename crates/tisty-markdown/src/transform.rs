use std::ops::Range;

use crate::{Block, Mark, Span, vocab};

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
        if !said.starts_with('<') {
            continue;
        }
        match &span.mark {
            Mark::Pen { color } => {
                let shut = match color {
                    Some(color) => format!("=={{{color}}}"),
                    None => "==".to_string(),
                };
                walled(span, text, "==", &shut, edits);
            }
            Mark::Underline => walled(span, text, "++", "++", edits),
            Mark::Icon { name, hue } if vocab::icon(name) => {
                let said = match hue.as_deref().filter(|one| vocab::hue(one)) {
                    Some(hue) => format!("::{name} {hue}::"),
                    None => format!("::{name}::"),
                };
                edits.push((span.at.clone(), said));
            }
            _ => {}
        }
    }
    for inner in &block.inner {
        gather(inner, text, edits);
    }
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
