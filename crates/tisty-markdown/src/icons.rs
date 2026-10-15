use crate::{Mark, Span};

/// Folds `<span data-ico="…">…</span>` into one icon, the way the editor has always written it.
pub fn fold(spans: &mut Vec<Span>, text: &str) {
    let mut at = 0;
    while at < spans.len() {
        let Some((name, hue)) = opened(&spans[at], text) else {
            at += 1;
            continue;
        };
        let shut = spans[at + 1..]
            .iter()
            .position(|one| one.mark == Mark::Html && &text[one.at.clone()] == "</span>")
            .map(|step| at + 1 + step);
        if let Some(shut) = shut {
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

fn opened(span: &Span, text: &str) -> Option<(String, Option<String>)> {
    if span.mark != Mark::Html {
        return None;
    }
    let tag = text[span.at.clone()]
        .strip_prefix("<span ")?
        .strip_suffix('>')?;
    let name = attribute(tag, "data-ico")?;
    Some((name, attribute(tag, "data-hue")))
}

fn attribute(tag: &str, name: &str) -> Option<String> {
    let open = format!("{name}=\"");
    let from = tag.find(&open)? + open.len();
    let to = from + tag[from..].find('"')?;
    Some(unquoted(&tag[from..to]))
}

fn unquoted(said: &str) -> String {
    said.replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}
