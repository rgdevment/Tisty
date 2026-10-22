use std::collections::HashSet;
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
    let silent = Nearby::of(silent);
    for span in spans.iter().filter(|one| one.mark == Mark::Text) {
        for piece in outside(span.at.clone(), silent.near(&span.at)) {
            items.push((piece.start, Item::Words(piece)));
        }
    }
    items.sort_by_key(|(at, _)| *at);
    let kept = paired(spans, text);
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
            Item::Atom(one) => atomic(one, spans, text, &kept, out),
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

pub(crate) fn atom(mark: &Mark) -> bool {
    !matches!(mark, Mark::Text) && !holds(mark)
}

pub(crate) fn holds(mark: &Mark) -> bool {
    matches!(
        mark,
        Mark::Emphasis
            | Mark::Strong
            | Mark::Strike
            | Mark::Link { .. }
            | Mark::Pen { .. }
            | Mark::Underline
    )
}

// The walls of a pen or an underline, and the colour named after a pen, mark it and are never shown.
pub(crate) fn walls(spans: &[Span], text: &str) -> Vec<Range<usize>> {
    spans
        .iter()
        .filter_map(|one| {
            let said = &text[one.at.clone()];
            let (start, end) = (one.at.start, one.at.end);
            match one.mark {
                Mark::Pen { .. } if said.starts_with("==") => {
                    let shut = start + said.rfind("==")?;
                    Some([start..start + 2, shut..end])
                }
                Mark::Underline if said.starts_with("++") => Some([start..start + 2, end - 2..end]),
                _ => None,
            }
        })
        .flatten()
        .collect()
}

/// Ranges sorted once, so each piece of a long paragraph asks only those around it instead of all of them.
pub(crate) struct Nearby {
    ranges: Vec<Range<usize>>,
    reach: Vec<usize>,
}

impl Nearby {
    pub(crate) fn of(mut ranges: Vec<Range<usize>>) -> Self {
        ranges.sort_by_key(|one| one.start);
        let reach = ranges
            .iter()
            .scan(0, |far, one| {
                *far = (*far).max(one.end);
                Some(*far)
            })
            .collect();
        Self { ranges, reach }
    }

    /// Every range that can touch `within`, and perhaps a few between them that do not.
    pub(crate) fn near(&self, within: &Range<usize>) -> &[Range<usize>] {
        let from = self.reach.partition_point(|far| *far <= within.start);
        let to = self.ranges.partition_point(|one| one.start < within.end);
        &self.ranges[from..to.max(from)]
    }
}

pub(crate) fn outside(within: Range<usize>, silent: &[Range<usize>]) -> Vec<Range<usize>> {
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
        Mark::Underline => out.push_str("<u>"),
        Mark::Link { to } if crate::nowhere(to) => out.push_str("<span>"),
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
        Some(Mark::Underline) => "</u>",
        Some(Mark::Link { to }) if crate::nowhere(to) => "</span>",
        Some(Mark::Link { .. }) => "</a>",
        Some(Mark::Pen { .. }) => "</mark>",
        _ => "",
    });
}

fn atomic(one: &Span, spans: &[Span], text: &str, kept: &HashSet<usize>, out: &mut String) {
    let said = &text[one.at.clone()];
    match &one.mark {
        Mark::Code => {
            let _ = write!(out, "<code>{}</code>", escaped(&code_words(said)));
        }
        Mark::Image { src } => pictured(src, &alt(one, spans, text), filed(said), out),
        Mark::Icon { name, hue } => {
            let hue = hue.as_deref().map(escaped).unwrap_or_default();
            let _ = write!(
                out,
                "<span class=\"ico\" data-ico=\"{0}\" data-hue=\"{hue}\" title=\"{0}\"></span>",
                escaped(name)
            );
        }
        Mark::Math => {
            let wall = if said.len() >= 4 && said.starts_with("$$") {
                2
            } else {
                1
            };
            formula(
                said.get(wall..said.len().saturating_sub(wall))
                    .unwrap_or(said),
                wall == 2,
                out,
            );
        }
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
        Mark::NoteRef { label } => {
            let _ = write!(
                out,
                "<sup class=\"fnref\"><a href=\"#fn-{0}\">{1}</a></sup>",
                escaped(&label.to_lowercase()),
                escaped(label)
            );
        }
        Mark::Break => out.push_str("<br>"),
        Mark::Html if kept.contains(&one.at.start) => out.push_str(&said.to_ascii_lowercase()),
        Mark::Html if said.starts_with("<!--") => {}
        Mark::Html => out.push_str(&escaped(said)),
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

pub(crate) fn code_words(said: &str) -> String {
    let wall = said.chars().take_while(|c| *c == '`').count();
    let inner = &said[wall..said.len() - wall.min(said.len() - wall)];
    let joined = inner
        .split('\n')
        .enumerate()
        .map(|(index, line)| {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if index == 0 {
                line
            } else {
                crate::unquoted(line)
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    match joined
        .strip_prefix(' ')
        .and_then(|one| one.strip_suffix(' '))
    {
        Some(trimmed) if !joined.trim().is_empty() => trimmed.to_string(),
        _ => joined,
    }
}

pub(crate) fn alt(image: &Span, spans: &[Span], text: &str) -> String {
    spans
        .iter()
        .filter(|one| {
            one.mark == Mark::Text && image.at.start <= one.at.start && one.at.end <= image.at.end
        })
        .map(|one| &text[one.at.clone()])
        .collect()
}

// A title of "file" asks for the attachment as a file, where a page would otherwise be drawn live.
fn filed(said: &str) -> bool {
    let said = said.trim_end().trim_end_matches(')').trim_end();
    ["\"file\"", "'file'", "(file"]
        .iter()
        .any(|one| said.ends_with(one))
}

fn pictured(src: &str, alt: &str, filed: bool, out: &mut String) {
    if let Some(task) = src.strip_prefix(crate::refs::TASK) {
        let _ = write!(
            out,
            "<a class=\"card\" data-task=\"{}\">{}</a>",
            escaped(task),
            escaped(alt)
        );
        return;
    }
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
                "<img src=\"{}\" alt=\"{}\" loading=\"lazy\"{}>",
                escaped(&safe(src)),
                escaped(alt),
                if filed { " data-as=\"file\"" } else { "" }
            );
        }
    }
}

#[cfg_attr(not(feature = "math"), allow(unused_variables))]
fn formula(latex: &str, block: bool, out: &mut String) {
    #[cfg(feature = "math")]
    if let Some(mathml) = crate::math::mathml(latex, block) {
        out.push_str(&mathml);
        return;
    }
    let _ = write!(out, "<code class=\"math\">{}</code>", escaped(latex));
}

// A tag left open would carry on through every block after it, so only matched pairs are kept.
pub(crate) fn paired(spans: &[Span], text: &str) -> HashSet<usize> {
    const PAIRS: &[&str] = &["kbd", "u", "sub", "sup", "mark", "small"];
    const BREAKS: &[&str] = &["<br>", "<br/>", "<br />"];
    let mut kept = HashSet::new();
    let mut open: Vec<(&str, usize)> = Vec::new();
    for one in spans.iter().filter(|one| one.mark == Mark::Html) {
        let said = text[one.at.clone()].to_ascii_lowercase();
        if BREAKS.contains(&said.as_str()) {
            kept.insert(one.at.start);
        } else if let Some(name) = said
            .strip_prefix("</")
            .and_then(|rest| rest.strip_suffix('>'))
        {
            if let Some(at) = open.iter().rposition(|(tag, _)| *tag == name) {
                kept.insert(open[at].1);
                kept.insert(one.at.start);
                open.truncate(at);
            }
        } else if let Some(name) = said
            .strip_prefix('<')
            .and_then(|rest| rest.strip_suffix('>'))
            && let Some(tag) = PAIRS.iter().find(|tag| **tag == name)
        {
            open.push((tag, one.at.start));
        }
    }
    kept
}

pub fn escaped(said: &str) -> String {
    said.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub const SCHEMES: &[&str] = &["http", "https", "mailto", "tisty"];

pub fn safe(to: &str) -> String {
    let lowered = to.trim().to_ascii_lowercase();
    let scheme = lowered.split_once(':').map(|(head, _)| head);
    let allowed = match scheme {
        None => true,
        Some(head) if head.contains(['/', '?', '#']) => true,
        Some(head) if head.len() == 1 && head.chars().all(|c| c.is_ascii_alphabetic()) => true,
        Some(head) => SCHEMES.contains(&head),
    };
    if allowed {
        to.trim().to_string()
    } else {
        "#".to_string()
    }
}

pub(crate) fn holder(spans: &[Span], wall: usize) -> Option<Range<usize>> {
    spans
        .iter()
        .filter(|one| !matches!(one.mark, Mark::Text | Mark::Break | Mark::Html))
        .filter(|one| one.at.start < wall && wall < one.at.end)
        .min_by_key(|one| one.at.end - one.at.start)
        .map(|one| one.at.clone())
}
