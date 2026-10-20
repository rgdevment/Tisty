use crate::attach::{leaf_of, lower_hex, stamped_by};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Copy {
    pub at: String,
    pub links: usize,
    pub shelved: bool,
    pub since: i64,
}

pub fn keeper(copies: &[Copy]) -> usize {
    (0..copies.len())
        .min_by_key(|&one| {
            let copy = &copies[one];
            let leaf = leaf_of(&copy.at).unwrap_or(&copy.at);
            (
                copy.shelved,
                repeats_stamp(leaf),
                std::cmp::Reverse(words(leaf)),
                std::cmp::Reverse(copy.links),
                copy.since,
                copy.at.clone(),
            )
        })
        .unwrap_or(0)
}

fn parts(leaf: &str) -> impl Iterator<Item = &str> {
    leaf.split('.').next().unwrap_or_default().split('-')
}

fn words(leaf: &str) -> usize {
    parts(leaf)
        .filter(|part| !part.is_empty() && !(part.len() >= 8 && lower_hex(part)))
        .count()
}

// A copy kept again before names carried their stamp once says it over and over.
fn repeats_stamp(leaf: &str) -> bool {
    let stamp = stamped_by(leaf);
    parts(leaf).filter(|part| *part == stamp).count() > 1
}

/// Every copy that goes, named in the body, pointed at the one that stays, with how many links moved.
pub fn relinked_all(body: &str, going: &[(String, String)]) -> Option<(String, usize)> {
    let mut now: Option<(String, usize)> = None;
    for (from, to) in going {
        let was = now.as_ref().map_or(body, |(text, _)| text.as_str());
        if let Some((changed, moved)) = relinked(was, from, to) {
            let before = now.map_or(0, |(_, count)| count);
            now = Some((changed, before + moved));
        }
    }
    now
}

pub fn relinked(body: &str, from: &str, to: &str) -> Option<(String, usize)> {
    let code = crate::beside::written_as_code(body);
    let mut out = String::with_capacity(body.len());
    let mut last = 0;
    let mut changed = 0;
    for (at, _) in body.match_indices(from) {
        let named = !code.get(at).copied().unwrap_or(false)
            && body[..at]
                .chars()
                .next_back()
                .is_none_or(|one| !in_a_name(one))
            && ends_a_name(&body[at + from.len()..])
            && a_link_leads_to(&body[..at]);
        if named {
            out.push_str(&body[last..at]);
            out.push_str(to);
            last = at + from.len();
            changed += 1;
        }
    }
    out.push_str(&body[last..]);
    (changed > 0).then_some((out, changed))
}

// Only a link's target is a name to change, never the same words said in a sentence.
fn a_link_leads_to(before: &str) -> bool {
    let mut lead = before;
    if let Some(cut) = lead.rfind(crate::attach::SHED) {
        let path = &lead[cut + crate::attach::SHED.len()..];
        if path.starts_with('/')
            && !path.contains(char::is_whitespace)
            && path.matches('/').count() <= 2
        {
            lead = &lead[..cut];
        }
    }
    lead.ends_with(['(', '<', '"', '\'', '=']) || lead.trim_end_matches(' ').ends_with("]:")
}

fn in_a_name(one: char) -> bool {
    one.is_ascii_alphanumeric() || matches!(one, '-' | '_' | '.' | '%')
}

// A dot may close a sentence after a name, but one that goes on is a longer name.
fn ends_a_name(after: &str) -> bool {
    let mut next = after.chars();
    match next.next() {
        None => true,
        Some('.') => next.next().is_none_or(|one| !one.is_ascii_alphanumeric()),
        Some(one) => !in_a_name(one),
    }
}

pub fn shelved(body: &str, leaf: &str) -> bool {
    let tail = format!("/{leaf}");
    body.match_indices(&tail).any(|(at, _)| {
        let before = &body[..at];
        before
            .rfind('/')
            .is_some_and(|cut| before[..cut].ends_with(crate::attach::SHED) && cut + 1 < at)
    })
}

#[cfg(test)]
#[path = "joining_test.rs"]
mod tests;
