use std::ops::Range;

pub const THIS_TIME: [&str; 2] = ["## This time", "## Esta vez"];
pub const WHEN_IT_CLOSES: [&str; 2] = ["## When it closes", "## Al cerrarla"];

fn level(line: &str) -> Option<usize> {
    let bare = line.trim_end();
    let marks = bare.chars().take_while(|one| *one == '#').count();
    ((1..=6).contains(&marks) && bare[marks..].starts_with([' ', '\t'])).then_some(marks)
}

fn hint(line: &str) -> bool {
    let bare = line.trim();
    bare.starts_with("<!--") && bare.ends_with("-->")
}

// What a section holds runs to the next heading as high as its own, code fences aside.
pub(crate) fn held(body: &str, heads: &[&str]) -> Option<(Range<usize>, Range<usize>)> {
    let mut at = 0;
    let mut fenced = tisty_markdown::fences::fencing();
    let mut found: Option<(usize, usize, usize)> = None;
    for line in body.split_inclusive('\n') {
        let code = fenced(line.trim_end_matches(['\n', '\r']));
        let marks = (!code).then(|| level(line)).flatten();
        match (found, marks) {
            (Some((_, from, own)), Some(marks)) if marks <= own => {
                return found.map(|(head, _, _)| (head..from, from..at));
            }
            (None, Some(marks)) if heads.contains(&line.trim_end()) => {
                found = Some((at, at + line.len(), marks));
            }
            _ => {}
        }
        at += line.len();
    }
    found.map(|(head, from, _)| (head..from, from..body.len()))
}

pub fn said_in(body: &str, heads: &[&str]) -> String {
    let Some((_, words)) = held(body, heads) else {
        return String::new();
    };
    body[words]
        .lines()
        .filter(|line| !hint(line))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// A round's own words leave with it: the section keeps its hint and waits for the next one.
pub fn this_time(body: &str) -> Option<(String, String)> {
    let said = said_in(body, &THIS_TIME);
    let (_, words) = held(body, &THIS_TIME).filter(|_| !said.is_empty())?;
    let line = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let hints: Vec<&str> = body[words.clone()]
        .lines()
        .filter(|one| hint(one))
        .collect();
    let mut kept = hints.join(line);
    if !kept.is_empty() {
        kept.push_str(line);
    }
    if words.end < body.len() {
        kept.push_str(line);
    }
    let cleared = format!("{}{kept}{}", &body[..words.start], &body[words.end..]);
    Some((said, cleared))
}

/// How it went goes first in the section kept for it, or closes the page when there is none.
pub fn summed(body: &str, summary: &str) -> String {
    let summary = summary.trim();
    // Closing again after a refusal or an undo says the same words once.
    let first = said_in(body, &WHEN_IT_CLOSES);
    if first.split("\n\n").next().map(str::trim) == Some(summary) {
        return body.to_string();
    }
    let line = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let Some((_, words)) = held(body, &WHEN_IT_CLOSES) else {
        let kept = body.trim_end_matches(['\n', '\r']);
        if kept.rsplit("\n\n").next().map(str::trim) == Some(summary) {
            return body.to_string();
        }
        return format!("{kept}{line}{line}{summary}{line}");
    };
    let section = &body[words.clone()];
    let hints = section
        .split_inclusive('\n')
        .take_while(|one| hint(one))
        .map(str::len)
        .sum::<usize>();
    let after = words.start + hints;
    let rest = body[after..words.end].trim_start_matches(['\n', '\r']);
    let rest = if rest.trim().is_empty() { "" } else { rest };
    let more = words.end < body.len();
    let gap = if !rest.is_empty() || more { line } else { "" };
    format!(
        "{}{line}{summary}{line}{gap}{rest}{}",
        &body[..after],
        &body[words.end..]
    )
}

#[cfg(test)]
#[path = "closing_test.rs"]
mod tests;
