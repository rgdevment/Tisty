use crate::html::attribute;
use crate::{Mark, Span};

// The tags Tisty wrote before it had words of its own: read only so they can be turned into them.
pub fn fold(spans: &mut Vec<Span>, text: &str) {
    paired(spans, text, "<span", "</span>", |tag| {
        let inside = tag.strip_prefix("<span")?.strip_suffix('>')?;
        let name = attribute(inside, "data-ico")?;
        Some(Mark::Icon {
            name,
            hue: attribute(inside, "data-hue"),
        })
    });
    paired(spans, text, "<mark", "</mark>", |tag| {
        if tag.eq_ignore_ascii_case("<mark>") {
            return Some(Mark::Pen { color: None });
        }
        let inside = tag.strip_prefix("<mark")?.strip_suffix('>')?;
        let color = attribute(inside, "data-pen")?;
        Some(Mark::Pen {
            color: (color != crate::vocab::YELLOW).then_some(color),
        })
    });
    paired(spans, text, "<u>", "</u>", |tag| {
        tag.eq_ignore_ascii_case("<u>").then_some(Mark::Underline)
    });
}

// Another of the same tag opening first means this one was never closed, and nothing is folded.
fn paired(
    spans: &mut Vec<Span>,
    text: &str,
    opens: &str,
    shuts: &str,
    mark_of: impl Fn(&str) -> Option<Mark>,
) {
    let said = |one: &Span| text[one.at.clone()].to_ascii_lowercase();
    let mut at = 0;
    while at < spans.len() {
        let mark = (spans[at].mark == Mark::Html)
            .then(|| mark_of(&text[spans[at].at.clone()]))
            .flatten();
        let Some(mark) = mark else {
            at += 1;
            continue;
        };
        let next = spans[at + 1..]
            .iter()
            .position(|one| {
                one.mark == Mark::Html && {
                    let tag = said(one);
                    tag == shuts || tag.starts_with(opens)
                }
            })
            .map(|step| at + 1 + step);
        if let Some(shut) = next.filter(|shut| said(&spans[*shut]) == shuts) {
            let whole = spans[at].at.start..spans[shut].at.end;
            spans.remove(shut);
            spans[at] = Span { mark, at: whole };
        }
        at += 1;
    }
}
