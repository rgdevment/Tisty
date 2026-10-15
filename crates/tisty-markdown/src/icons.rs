use crate::html::attribute;
use crate::{Mark, Span};

pub fn fold(spans: &mut Vec<Span>, text: &str) {
    let mut at = 0;
    while at < spans.len() {
        let Some((name, hue)) = opened(&spans[at], text) else {
            at += 1;
            continue;
        };
        if let Some(shut) = closing(spans, at, text) {
            let whole = spans[at].at.start..spans[shut].at.end;
            spans.splice(
                at..=shut,
                [Span {
                    mark: Mark::Icon { name, hue },
                    at: whole,
                }],
            );
        }
        at += 1;
    }
}

// Another span opening first means this one was never closed, and nothing is folded then.
fn closing(spans: &[Span], from: usize, text: &str) -> Option<usize> {
    let next = spans[from + 1..]
        .iter()
        .position(|one| {
            let said = &text[one.at.clone()];
            one.mark == Mark::Html && (said == "</span>" || said.starts_with("<span"))
        })
        .map(|step| from + 1 + step)?;
    (&text[spans[next].at.clone()] == "</span>").then_some(next)
}

fn opened(span: &Span, text: &str) -> Option<(String, Option<String>)> {
    if span.mark != Mark::Html {
        return None;
    }
    let inside = text[span.at.clone()]
        .strip_prefix("<span")?
        .strip_suffix('>')?;
    let name = attribute(inside, "data-ico")?;
    Some((name, attribute(inside, "data-hue")))
}
