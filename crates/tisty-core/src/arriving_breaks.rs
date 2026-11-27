use std::ops::Range;

use tisty_markdown::{BLANK, Block, Kind, Mark};

type Edits = Vec<(Range<usize>, String)>;

/// Line breaks other editors write as `<br>`: alone on a line they keep that many blank lines, among words they break the line.
pub fn unbroken(text: &str) -> String {
    let doc = tisty_markdown::read(text);
    let mut edits = Edits::new();
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

fn gather(block: &Block, text: &str, edits: &mut Edits) {
    let alone = breaks(text[block.at.clone()].trim()).is_some();
    match &block.kind {
        Kind::Html | Kind::Paragraph if alone => return blank_lines(block, text, edits),
        Kind::Html => return,
        Kind::Table { .. } => return celled(block, text, edits),
        // Code and properties say what they say.
        Kind::Code { .. } | Kind::Front => return,
        _ => {}
    }
    for span in &block.spans {
        if span.mark == Mark::Html && breaks(&text[span.at.clone()]) == Some(1) {
            edits.push((span.at.clone(), broken_here(span.at.end, block, text)));
        }
    }
    for inner in &block.inner {
        gather(inner, text, edits);
    }
}

// A cell holds one line, so a break in it parts its words with a space rather than gluing them.
fn celled(block: &Block, text: &str, edits: &mut Edits) {
    for span in &block.spans {
        if span.mark == Mark::Html && breaks(&text[span.at.clone()]).is_some() {
            edits.push((span.at.clone(), " ".to_string()));
        }
    }
    for inner in &block.inner {
        celled(inner, text, edits);
    }
}

fn blank_lines(block: &Block, text: &str, edits: &mut Edits) {
    let said = text[block.at.clone()].trim_end();
    if let Some(many) = breaks(said) {
        let at = block.at.start..block.at.start + said.len();
        edits.push((at, vec![BLANK; many].join("\n\n")));
    }
}

// A break that already ends its line only marks that end; one that ends the paragraph breaks nothing.
fn broken_here(end: usize, block: &Block, text: &str) -> String {
    let rest = &text[end..block.at.end];
    if rest.trim().is_empty() {
        String::new()
    } else if rest.starts_with(['\r', '\n']) {
        "\\".to_string()
    } else {
        "\\\n".to_string()
    }
}

/// How many `<br>` the words hold, when they hold nothing else.
fn breaks(said: &str) -> Option<usize> {
    let mut rest = said.trim();
    let mut many = 0;
    while !rest.is_empty() {
        let shut = rest.find('>')?;
        if !one_break(&rest[..=shut]) {
            return None;
        }
        many += 1;
        rest = rest[shut + 1..].trim_start();
    }
    (many > 0).then_some(many)
}

fn one_break(tag: &str) -> bool {
    let inner = tag
        .strip_prefix('<')
        .and_then(|one| one.strip_suffix('>'))
        .map(|one| one.trim_end().trim_end_matches('/').trim_end());
    inner.is_some_and(|name| name.eq_ignore_ascii_case("br"))
}
