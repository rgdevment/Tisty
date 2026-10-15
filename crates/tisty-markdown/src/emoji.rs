use crate::{Mark, Span};

/// `:tada:` is read as 🎉; on disk the dialect keeps the character itself.
pub fn fold(spans: &mut Vec<Span>, text: &str) {
    let mut found = Vec::new();
    for span in spans.iter().filter(|one| one.mark == Mark::Text) {
        named(text, span.at.clone(), &mut found);
    }
    spans.extend(found);
}

fn named(text: &str, within: std::ops::Range<usize>, found: &mut Vec<Span>) {
    let said = &text[within.clone()];
    let mut from = 0;
    while let Some(open) = said[from..].find(':').map(|at| from + at) {
        let rest = &said[open + 1..];
        let wide = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || "_+-".contains(c)))
            .unwrap_or(rest.len());
        let shut = open + 1 + wide;
        let alone = |at: Option<char>| at.is_none_or(|c| !c.is_alphanumeric());
        let fits = wide > 0
            && said[shut..].starts_with(':')
            && alone(text[..within.start + open].chars().next_back())
            && alone(text[within.start + shut + 1..].chars().next());
        match fits
            .then(|| emojis::get_by_shortcode(&rest[..wide]))
            .flatten()
        {
            Some(emoji) => {
                found.push(Span {
                    mark: Mark::Emoji {
                        glyph: emoji.as_str().to_string(),
                    },
                    at: within.start + open..within.start + shut + 1,
                });
                from = shut + 1;
            }
            None => from = open + 1,
        }
    }
}
