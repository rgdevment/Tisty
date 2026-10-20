use crate::attach::{leaf_of, lower_hex, stamped_by};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub at: String,
    pub links: usize,
    pub shelved: bool,
    pub since: i64,
}

pub fn keeper(copies: &[Candidate]) -> usize {
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

// Kept again, a name grew its own stamp once more, maybe longer: the one shares the other's start.
fn a_stamp(part: &str, own: &str) -> bool {
    (8..=16).contains(&part.len())
        && lower_hex(part)
        && (part.starts_with(own) || own.starts_with(part))
}

fn words(leaf: &str) -> usize {
    let own = stamped_by(leaf);
    parts(leaf)
        .filter(|part| !part.is_empty() && !a_stamp(part, own))
        .count()
}

// A copy kept again before names carried their stamp once says one, or two of other lengths, over and over.
fn repeats_stamp(leaf: &str) -> bool {
    let own = stamped_by(leaf);
    parts(leaf).filter(|part| a_stamp(part, own)).count() > 1
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
    // An example in code shows a link, it is not one, and stays as it was written.
    let code = crate::beside::written_as_code(body);
    let mut out = String::with_capacity(body.len());
    let mut last = 0;
    let mut changed = 0;
    for (at, _) in body.match_indices(from) {
        let alone = !code.get(at).copied().unwrap_or(false)
            && body[..at]
                .chars()
                .next_back()
                .is_none_or(|one| !in_a_name(one))
            && ends_a_name(&body[at + from.len()..]);
        let Some(start) = alone
            .then(|| link_start(&body[..at]))
            .flatten()
            .filter(|start| *start >= last)
        else {
            continue;
        };
        out.push_str(&body[last..start]);
        // Through a folder or an old shelf, it is written flat, as every link to an attachment now is.
        if start < at {
            out.push_str(crate::attach::SHED);
            out.push('/');
        }
        out.push_str(to);
        last = at + from.len();
        changed += 1;
    }
    out.push_str(&body[last..]);
    (changed > 0).then_some((out, changed))
}

// Only a link's target is a name to change, never the same words said in a sentence.
fn link_start(before: &str) -> Option<usize> {
    // Only the folder right before the name counts: `attachments/`, or one folder under it like an old shelf.
    let folder = |path: &str| {
        path == "/"
            || path
                .strip_prefix('/')
                .and_then(|rest| rest.strip_suffix('/'))
                .is_some_and(|one| {
                    !one.is_empty()
                        && one
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
                })
    };
    let start = before
        .rfind(crate::attach::SHED)
        .filter(|cut| folder(&before[cut + crate::attach::SHED.len()..]))
        .unwrap_or(before.len());
    let lead = &before[..start];
    let lead = lead.strip_suffix('<').unwrap_or(lead);
    let ends = |text: &str, one: &str| {
        let tail = text.as_bytes();
        tail.len() >= one.len()
            && tail[tail.len() - one.len()..].eq_ignore_ascii_case(one.as_bytes())
    };
    // A space may follow a bracket, never an attribute's equals sign.
    let spaced = lead.trim_end_matches(' ');
    let markdown = spaced.ends_with("](") || spaced.ends_with("[[") || defined(spaced);
    let html = ["src=\"", "src='", "href=\"", "href='", "src=", "href="]
        .iter()
        .any(|one| ends(lead, one));
    (markdown || html).then_some(start)
}

// A reference definition opens its own line: `[label]: target`.
fn defined(lead: &str) -> bool {
    let Some(line) = lead.strip_suffix("]:") else {
        return false;
    };
    let line = line.rsplit('\n').next().unwrap_or_default().trim_start();
    line.starts_with('[') && !line[1..].contains(']')
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
