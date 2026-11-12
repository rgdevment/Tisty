use std::path::Path;

use crate::State;
use crate::event::Idea;
use crate::model::{FlaggedIdea, Kept};

pub fn opening(title: &str) -> String {
    format!(
        "---\nview: notes\nbrainstorm: on\n---\n\n# {}\n",
        title.trim()
    )
}

pub const IDEA_AT_MOST: usize = 140;

pub const COLOURS: [&str; 12] = [
    "pink", "coral", "orange", "yellow", "lime", "green", "teal", "cyan", "blue", "lavender",
    "violet", "magenta",
];

pub fn jotted(idea: &str, at: jiff::civil::DateTime, colour: &str) -> String {
    format!(
        "## {} · {colour}\n\n{}",
        at.strftime("%Y-%m-%d %H:%M"),
        idea.trim()
    )
}

pub fn coloured(body: &str, roll: u32) -> &'static str {
    let last = body
        .lines()
        .rev()
        .find(|line| line.starts_with("## "))
        .and_then(|line| line.rsplit_once(" · "))
        .map(|(_, colour)| colour.trim());
    let many = COLOURS.len();
    let mut at = roll as usize % many;
    if last == Some(COLOURS[at]) {
        at = (at + 1 + (roll as usize / many) % (many - 1)) % many;
    }
    COLOURS[at]
}

pub fn filed(idea: &str, from: &str, on: jiff::civil::Date) -> String {
    let quoted: Vec<String> = idea
        .trim()
        .lines()
        .map(|line| format!("> {line}").trim_end().to_string())
        .collect();
    format!(
        "> [!NOTE] {} · {}\n{}",
        from.trim(),
        on.strftime("%Y-%m-%d"),
        quoted.join("\n")
    )
}

pub fn ideas(body: &str) -> usize {
    let mut fenced = false;
    body.lines()
        .filter(|line| {
            if line.starts_with("```") || line.starts_with("~~~") {
                fenced = !fenced;
                return false;
            }
            !fenced && line.starts_with("## ")
        })
        .count()
}

struct Sheet<'a> {
    ending: &'static str,
    head: Vec<&'a str>,
    blocks: Vec<Vec<&'a str>>,
}

fn cut(body: &str) -> Sheet<'_> {
    let mut sheet = Sheet {
        ending: if body.contains("\r\n") { "\r\n" } else { "\n" },
        head: Vec::new(),
        blocks: Vec::new(),
    };
    let mut fenced = false;
    for line in body
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
    {
        let opens = !fenced && line.starts_with("## ");
        if line.starts_with("```") || line.starts_with("~~~") {
            fenced = !fenced;
        }
        match sheet.blocks.last_mut() {
            _ if opens => sheet.blocks.push(vec![line]),
            Some(block) => block.push(line),
            None => sheet.head.push(line),
        }
    }
    sheet
}

fn idea_of(block: &[&str]) -> Idea {
    Idea {
        heading: block[0].trim_start_matches("## ").trim().to_string(),
        text: block[1..].join("\n").trim().to_string(),
    }
}

// Untouched ideas go back byte for byte, as the window sews them, so one change never rewrites the file in sync.
fn sewn(sheet: &Sheet) -> String {
    let last = sheet.blocks.len().saturating_sub(1);
    let mut lines: Vec<&str> = sheet.head.clone();
    for (at, block) in sheet.blocks.iter().enumerate() {
        lines.extend(block);
        if at < last && block.last() != Some(&"") {
            lines.push("");
        }
    }
    while lines.last() == Some(&"") {
        lines.pop();
    }
    format!("{}{}", lines.join(sheet.ending), sheet.ending)
}

pub fn ideas_on(body: &str) -> Vec<Idea> {
    cut(body)
        .blocks
        .iter()
        .map(|block| idea_of(block))
        .collect()
}

pub fn without(body: &str, idea: &Idea) -> Option<String> {
    let mut sheet = cut(body);
    let at = sheet
        .blocks
        .iter()
        .position(|block| idea_of(block) == *idea)?;
    sheet.blocks.remove(at);
    Some(sewn(&sheet))
}

pub fn marked<'a>(kept: &'a Kept, body: &str) -> Vec<&'a FlaggedIdea> {
    let on = ideas_on(body);
    kept.ideas_flagged
        .iter()
        .filter(|one| on.contains(&one.idea))
        .collect()
}

pub fn files_an_idea(note: &str) -> bool {
    note.trim().lines().next().is_some_and(filed_on)
}

fn filed_on(line: &str) -> bool {
    line.strip_prefix("> [!NOTE] ")
        .and_then(|rest| rest.rsplit_once(" · "))
        .is_some_and(|(_, on)| jiff::civil::Date::strptime("%Y-%m-%d", on.trim()).is_ok())
}

pub fn quoted(note: &str) -> Option<String> {
    let mut lines = note.trim().lines();
    if !filed_on(lines.next()?) {
        return None;
    }
    let said: Option<Vec<&str>> = lines
        .map(|line| {
            line.strip_prefix('>')
                .map(|rest| rest.strip_prefix(' ').unwrap_or(rest))
        })
        .collect();
    let text = said?.join("\n").trim().to_string();
    (!text.is_empty()).then_some(text)
}

pub fn jotted_all(
    was: &str,
    each: &[String],
    at: jiff::civil::DateTime,
    mut roll: impl FnMut() -> u32,
) -> String {
    let mut sheet = was.to_string();
    let mut added = Vec::new();
    for idea in each {
        let one = jotted(idea, at, coloured(&sheet, roll()));
        sheet.push_str("\n\n");
        sheet.push_str(&one);
        added.push(one);
    }
    added.join("\n\n")
}

pub fn roll() -> u32 {
    use rand_core::TryRngCore;
    rand_core::OsRng
        .try_next_u32()
        .unwrap_or_else(|_| jiff::Timestamp::now().subsec_nanosecond().unsigned_abs())
}

// The oldest wins, so two machines that each made one before they met keep dropping ideas in the same.
pub fn find<'a>(root: &Path, state: &'a State, last: Option<&str>) -> Option<(&'a Kept, String)> {
    let mut open: Vec<&Kept> = state
        .docs
        .values()
        .filter(|one| one.page_of.is_none() && !state.held_away(one))
        .collect();
    open.sort_by_key(|one| one.id);
    let holding = |one: &'a Kept| {
        crate::docs::read(root, &one.file)
            .ok()
            .filter(|body| tisty_markdown::front::brainstorm(body))
            .map(|body| (one, body))
    };
    open.iter()
        .find(|one| last.is_some_and(|file| one.file == file))
        .and_then(|one| holding(one))
        .or_else(|| open.into_iter().find_map(holding))
}

pub fn said_sheet<'a>(root: &Path, state: &'a State) -> Option<(&'a Kept, String)> {
    let mut told: Vec<&Kept> = state
        .docs
        .values()
        .filter(|one| one.brainstorm && one.page_of.is_none() && !state.held_away(one))
        .collect();
    told.sort_by_key(|one| one.id);
    told.into_iter().find_map(|one| sheet_of(root, one))
}

pub fn sheet_of<'a>(root: &Path, kept: &'a Kept) -> Option<(&'a Kept, String)> {
    crate::docs::read(root, &kept.file)
        .ok()
        .filter(|body| tisty_markdown::front::brainstorm(body))
        .map(|body| (kept, body))
}

#[cfg(test)]
#[path = "brainstorm_test.rs"]
mod tests;
