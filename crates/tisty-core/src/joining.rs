use crate::attach::{leaf_of, lower_hex, stamped_by};

/// One copy of an attachment kept twice, as the person is shown it before the two become one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Copy {
    pub at: String,
    pub links: usize,
    pub shelved: bool,
    pub since: i64,
}

/// The copy that stays, best first: no link through an old shelf, a readable name, the most links,
/// the oldest.
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

/// The body with every link to `from` pointing at `to` instead, in whatever form it was written;
/// `None` when nothing in it names `from`.
pub fn relinked(body: &str, from: &str, to: &str) -> Option<String> {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    let mut changed = false;
    while let Some(at) = rest.find(from) {
        let before = rest[..at].chars().next_back();
        let after = &rest[at + from.len()..];
        out.push_str(&rest[..at]);
        if before.is_none_or(|one| !in_a_name(one)) && ends_a_name(after) {
            out.push_str(to);
            changed = true;
        } else {
            out.push_str(from);
        }
        rest = after;
    }
    out.push_str(rest);
    changed.then_some(out)
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

/// Whether the body reaches `leaf` through an old shelf, `attachments/xx/<leaf>`.
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
