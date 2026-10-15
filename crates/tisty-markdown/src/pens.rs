use crate::{Mark, Span};

/// `==words==` is the yellow pen; every other colour is written as `<mark data-pen="…">`.
pub fn fold(spans: &mut Vec<Span>, text: &str) {
    tagged(spans, text);
    doubled(spans, text);
    spans.sort_by_key(|one| (one.at.start, std::cmp::Reverse(one.at.end)));
}

fn tagged(spans: &mut Vec<Span>, text: &str) {
    let mut at = 0;
    while at < spans.len() {
        let Some(color) = opened(&spans[at], text) else {
            at += 1;
            continue;
        };
        let shut = spans[at + 1..]
            .iter()
            .position(|one| one.mark == Mark::Html && &text[one.at.clone()] == "</mark>")
            .map(|step| at + 1 + step);
        if let Some(shut) = shut {
            let whole = spans[at].at.start..spans[shut].at.end;
            spans.remove(shut);
            spans[at] = Span {
                mark: Mark::Pen { color },
                at: whole,
            };
        }
        at += 1;
    }
}

fn opened(span: &Span, text: &str) -> Option<Option<String>> {
    if span.mark != Mark::Html {
        return None;
    }
    let tag = &text[span.at.clone()];
    if tag == "<mark>" {
        return Some(None);
    }
    let pen = tag.strip_prefix("<mark data-pen=\"")?.strip_suffix("\">")?;
    Some(Some(pen.to_string()))
}

fn doubled(spans: &mut Vec<Span>, text: &str) {
    let walls = walls(spans, text);
    let mut at = 0;
    while at + 1 < walls.len() {
        let (open, shut) = (walls[at], walls[at + 1]);
        let inside = &text[open + 2..shut];
        if inside.is_empty()
            || inside.starts_with(char::is_whitespace)
            || inside.ends_with(char::is_whitespace)
        {
            at += 1;
            continue;
        }
        spans.push(Span {
            mark: Mark::Pen { color: None },
            at: open..shut + 2,
        });
        at += 2;
    }
}

// Only a pair of `=` on its own counts: `===` and longer are something else being written.
fn walls(spans: &[Span], text: &str) -> Vec<usize> {
    let mut found = Vec::new();
    for span in spans.iter().filter(|one| one.mark == Mark::Text) {
        let said = &text[span.at.clone()];
        let bytes = said.as_bytes();
        let mut at = 0;
        while at + 1 < bytes.len() {
            let run = bytes[at..].iter().take_while(|b| **b == b'=').count();
            if run == 2 {
                found.push(span.at.start + at);
            }
            at += run.max(1);
        }
    }
    found
}
