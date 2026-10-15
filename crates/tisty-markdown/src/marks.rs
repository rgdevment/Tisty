use crate::{Mark, Span};

#[cfg(feature = "tags")]
// The core reads a tag off the whole line, so the line decides what the hash belongs to.
pub fn tagged(spans: &mut Vec<Span>, text: &str) {
    let mut found = Vec::new();
    for span in spans.iter().filter(|one| one.mark == Mark::Text) {
        let from = text[..span.at.start].rfind('\n').map_or(0, |at| at + 1);
        let to = text[span.at.end..]
            .find('\n')
            .map_or(text.len(), |at| span.at.end + at);
        for (at, name) in crate::tags::spots(&text[from..to]) {
            let at = from + at.start..from + at.end;
            if span.at.start <= at.start && at.end <= span.at.end {
                found.push(Span {
                    mark: Mark::Tag { name },
                    at,
                });
            }
        }
    }
    spans.extend(found);
}

#[cfg(feature = "refs")]
// The tokenizer splits `[[name]]` around its brackets, so the pair is found in the bytes.
pub fn named(spans: &mut Vec<Span>, text: &str) {
    let mut found = Vec::new();
    let inside = |at: usize| {
        spans
            .iter()
            .any(|one| one.mark == Mark::Text && one.at.start <= at && at < one.at.end)
    };
    let (Some(from), Some(to)) = (
        spans.iter().map(|one| one.at.start).min(),
        spans.iter().map(|one| one.at.end).max(),
    ) else {
        return;
    };
    let mut at = from;
    while let Some(open) = text[at..to].find("[[").map(|step| at + step) {
        let Some(shut) = text[open + 2..to].find("]]").map(|step| open + 2 + step) else {
            break;
        };
        let name = text[open + 2..shut].trim();
        if !name.is_empty() && !name.contains(['\n', '[']) && inside(open) && inside(shut + 1) {
            found.push(Span {
                mark: Mark::Ref {
                    name: name.to_string(),
                },
                at: open..shut + 2,
            });
            at = shut + 2;
        } else {
            at = open + 2;
        }
    }
    spans.extend(found);
}
