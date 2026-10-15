use std::fmt::Write as _;
use std::ops::Range;

use crate::{Mark, Span};

pub fn inline(spans: &[Span], text: &str, out: &mut String) {
    let atoms: Vec<&Span> = spans.iter().filter(|one| atom(&one.mark)).collect();
    let mut items: Vec<(usize, Item)> = atoms
        .iter()
        .map(|one| (one.at.start, Item::Atom(one)))
        .collect();
    let mut silent: Vec<Range<usize>> = atoms.iter().map(|one| one.at.clone()).collect();
    silent.extend(walls(spans, text));
    for span in spans.iter().filter(|one| one.mark == Mark::Text) {
        for piece in outside(span.at.clone(), &silent) {
            items.push((piece.start, Item::Words(piece)));
        }
    }
    items.sort_by_key(|(at, _)| *at);
    let containers: Vec<&Span> = spans.iter().filter(|one| holds(&one.mark)).collect();
    let mut open: Vec<&Span> = Vec::new();
    let mut next = 0;
    for (at, item) in items {
        while open.last().is_some_and(|one| one.at.end <= at) {
            shut(open.pop().map(|one| &one.mark), out);
        }
        while next < containers.len() && containers[next].at.start <= at {
            if containers[next].at.end > at {
                opening(&containers[next].mark, out);
                open.push(containers[next]);
            }
            next += 1;
        }
        match item {
            Item::Words(piece) => words(&text[piece], out),
            Item::Atom(one) => atomic(one, spans, text, out),
        }
    }
    while let Some(one) = open.pop() {
        shut(Some(&one.mark), out);
    }
}

enum Item<'a> {
    Words(Range<usize>),
    Atom(&'a Span),
}

fn atom(mark: &Mark) -> bool {
    !matches!(mark, Mark::Text) && !holds(mark)
}

fn holds(mark: &Mark) -> bool {
    matches!(
        mark,
        Mark::Emphasis | Mark::Strong | Mark::Strike | Mark::Link { .. } | Mark::Pen { .. }
    )
}

// The yellow pen is written between double equals, which mark it and are never shown.
fn walls(spans: &[Span], text: &str) -> Vec<Range<usize>> {
    spans
        .iter()
        .filter(|one| one.mark == Mark::Pen { color: None })
        .filter(|one| text[one.at.clone()].starts_with("=="))
        .flat_map(|one| [one.at.start..one.at.start + 2, one.at.end - 2..one.at.end])
        .collect()
}

fn outside(within: Range<usize>, silent: &[Range<usize>]) -> Vec<Range<usize>> {
    let mut pieces = vec![within];
    for one in silent {
        pieces = pieces
            .into_iter()
            .flat_map(|piece| {
                let before = piece.start..one.start.clamp(piece.start, piece.end);
                let after = one.end.clamp(piece.start, piece.end)..piece.end;
                [before, after]
            })
            .filter(|piece| !piece.is_empty())
            .collect();
    }
    pieces
}

fn opening(mark: &Mark, out: &mut String) {
    match mark {
        Mark::Emphasis => out.push_str("<em>"),
        Mark::Strong => out.push_str("<strong>"),
        Mark::Strike => out.push_str("<s>"),
        Mark::Link { to } => {
            let _ = write!(out, "<a href=\"{}\">", escaped(&safe(to)));
        }
        Mark::Pen { color } => match color {
            Some(pen) => {
                let _ = write!(out, "<mark data-pen=\"{}\">", escaped(pen));
            }
            None => out.push_str("<mark>"),
        },
        _ => {}
    }
}

fn shut(mark: Option<&Mark>, out: &mut String) {
    out.push_str(match mark {
        Some(Mark::Emphasis) => "</em>",
        Some(Mark::Strong) => "</strong>",
        Some(Mark::Strike) => "</s>",
        Some(Mark::Link { .. }) => "</a>",
        Some(Mark::Pen { .. }) => "</mark>",
        _ => "",
    });
}

fn atomic(one: &Span, spans: &[Span], text: &str, out: &mut String) {
    let said = &text[one.at.clone()];
    match &one.mark {
        Mark::Code => {
            let _ = write!(out, "<code>{}</code>", escaped(code_words(said)));
        }
        Mark::Image { src } => pictured(src, &alt(one, spans, text), out),
        Mark::Icon { name, hue } => {
            let hue = hue.as_deref().map(escaped).unwrap_or_default();
            let _ = write!(
                out,
                "<span class=\"ico\" data-ico=\"{0}\" data-hue=\"{hue}\" title=\"{0}\"></span>",
                escaped(name)
            );
        }
        Mark::Math => formula(said.trim_matches('$'), out),
        Mark::Emoji { glyph } => out.push_str(&escaped(glyph)),
        Mark::Tag { name } => {
            let _ = write!(
                out,
                "<span class=\"tag\" data-tag=\"{}\">{}</span>",
                escaped(name),
                escaped(said)
            );
        }
        Mark::Ref { name } => {
            let _ = write!(
                out,
                "<span class=\"ref\" data-ref=\"{0}\">{0}</span>",
                escaped(name)
            );
        }
        Mark::Break => out.push_str("<br>"),
        Mark::Html => out.push_str(&kept_tag(said)),
        _ => {}
    }
}

// An entity arrives as its own piece, already valid html; everything else is escaped.
fn words(said: &str, out: &mut String) {
    let entity = said.len() > 2
        && said.starts_with('&')
        && said.ends_with(';')
        && said[1..said.len() - 1]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '#');
    if entity {
        out.push_str(said);
    } else {
        out.push_str(&escaped(said));
    }
}

fn code_words(said: &str) -> &str {
    let wall = said.chars().take_while(|c| *c == '`').count();
    let inner = &said[wall..said.len() - wall.min(said.len() - wall)];
    match inner
        .strip_prefix(' ')
        .and_then(|one| one.strip_suffix(' '))
    {
        Some(trimmed) if !inner.trim().is_empty() => trimmed,
        _ => inner,
    }
}

fn alt(image: &Span, spans: &[Span], text: &str) -> String {
    spans
        .iter()
        .filter(|one| {
            one.mark == Mark::Text && image.at.start <= one.at.start && one.at.end <= image.at.end
        })
        .map(|one| &text[one.at.clone()])
        .collect()
}

fn pictured(src: &str, alt: &str, out: &mut String) {
    match src.strip_prefix(crate::refs::DOC) {
        Some(doc) => {
            let _ = write!(
                out,
                "<a class=\"card\" data-doc=\"{}\">{}</a>",
                escaped(doc),
                escaped(alt)
            );
        }
        None => {
            let _ = write!(
                out,
                "<img src=\"{}\" alt=\"{}\" loading=\"lazy\">",
                escaped(&safe(src)),
                escaped(alt)
            );
        }
    }
}

fn formula(latex: &str, out: &mut String) {
    #[cfg(feature = "math")]
    if let Some(mathml) = crate::math::mathml(latex, false) {
        out.push_str(&mathml);
        return;
    }
    let _ = write!(out, "<code class=\"math\">{}</code>", escaped(latex));
}

// Only plain inline tags are kept; anything with attributes or that can run is shown as words.
fn kept_tag(said: &str) -> String {
    const KEPT: &[&str] = &[
        "<kbd>", "</kbd>", "<u>", "</u>", "<sub>", "</sub>", "<sup>", "</sup>", "<br>", "<br/>",
        "<br />", "<mark>", "</mark>", "<small>", "</small>",
    ];
    if KEPT.contains(&said.to_ascii_lowercase().as_str()) {
        said.to_ascii_lowercase()
    } else {
        escaped(said)
    }
}

pub fn escaped(said: &str) -> String {
    said.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// A link may only go where a reader can safely follow it: the web, mail, Tisty or a file beside it.
pub fn safe(to: &str) -> String {
    let lowered = to.trim().to_ascii_lowercase();
    let scheme = lowered.split_once(':').map(|(head, _)| head);
    let allowed = match scheme {
        None => true,
        Some(head) if head.contains(['/', '?', '#']) => true,
        Some(head) => ["http", "https", "mailto", "tisty"].contains(&head),
    };
    if allowed {
        to.trim().to_string()
    } else {
        "#".to_string()
    }
}
