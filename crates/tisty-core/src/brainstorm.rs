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

pub const COLOURS: [&str; 12] = tisty_markdown::IDEA_HUES;

pub fn jotted(idea: &str, at: jiff::civil::DateTime, colour: &str) -> String {
    stuck(
        &format!("{} · {colour}", at.strftime("%Y-%m-%d %H:%M")),
        idea,
    )
}

fn stuck(head: &str, idea: &str) -> String {
    let mut lines = vec![format!("{OPENER} {}", head.trim())];
    lines.extend(
        idea.trim()
            .lines()
            .map(|line| format!("> {line}").trim_end().to_string()),
    );
    lines.join("\n")
}

const OPENER: &str = "> [!IDEA]";

fn lines(body: &str) -> impl Iterator<Item = &str> {
    body.split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
}

// A marker right under a quoted line stays in that quote, as every Markdown reader joins them.
#[derive(Default)]
struct Openers {
    fenced: bool,
    fenced_in_quote: bool,
    quoting: bool,
}

impl Openers {
    fn opens(&mut self, line: &str) -> bool {
        let quoted = is_quoted(line);
        // Markdown ends a fence opened inside a quote where the quote ends, closed or not.
        if self.fenced && self.fenced_in_quote && !quoted {
            self.fenced = false;
        }
        let opens = !self.fenced && !self.quoting && tisty_markdown::idea_words(line).is_some();
        if fences(line) {
            self.fenced_in_quote = !self.fenced && quoted;
            self.fenced = !self.fenced;
        }
        self.quoting = quoted;
        opens
    }
}

fn is_quoted(line: &str) -> bool {
    line.trim_start().starts_with('>')
}

fn unquoted(line: &str) -> &str {
    line.trim_start()
        .strip_prefix('>')
        .map_or(line, |rest| rest.strip_prefix(' ').unwrap_or(rest))
}

fn fences(line: &str) -> bool {
    let bare = line.trim_start_matches(['>', ' ', '\t']);
    bare.starts_with("```") || bare.starts_with("~~~")
}

pub fn coloured(body: &str, roll: u32) -> &'static str {
    picked(last_hue(body), roll)
}

fn last_hue(body: &str) -> Option<&'static str> {
    let mut openers = Openers::default();
    lines(body)
        .filter(|line| openers.opens(line))
        .last()
        .and_then(|line| tisty_markdown::idea_head(line).hue)
}

fn picked(last: Option<&str>, roll: u32) -> &'static str {
    let many = COLOURS.len();
    let mut at = roll as usize % many;
    if last == Some(COLOURS[at]) {
        at = (at + 1 + (roll as usize / many) % (many - 1)) % many;
    }
    COLOURS[at]
}

pub fn filed(idea: &str, heading: &str) -> String {
    let heading = heading.trim();
    let head = match tisty_markdown::idea_parts(heading).side {
        _ if heading.is_empty() => "right".to_string(),
        Some(_) => heading.to_string(),
        None => format!("{heading} · right"),
    };
    stuck(&head, idea)
}

pub fn ideas(body: &str) -> usize {
    let mut openers = Openers::default();
    lines(body).filter(|line| openers.opens(line)).count()
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
    let mut openers = Openers::default();
    for line in lines(body) {
        let opens = openers.opens(line);
        match sheet.blocks.last_mut() {
            _ if opens => sheet.blocks.push(vec![line]),
            Some(block) => block.push(line),
            None => sheet.head.push(line),
        }
    }
    sheet
}

fn idea_of(block: &[&str]) -> Idea {
    let said: Vec<&str> = block[1..].iter().map(|line| unquoted(line)).collect();
    Idea {
        heading: tisty_markdown::idea_words(block[0])
            .unwrap_or_default()
            .to_string(),
        text: said.join("\n").trim().to_string(),
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
    tisty_markdown::idea_head(line).when.is_some()
}

pub fn quoted(note: &str) -> Option<String> {
    let mut lines = note.trim().lines();
    if !filed_on(lines.next()?) {
        return None;
    }
    let said: Option<Vec<&str>> = lines
        .map(|line| is_quoted(line).then(|| unquoted(line)))
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
    let mut last = last_hue(was);
    let added: Vec<String> = each
        .iter()
        .map(|idea| {
            let colour = picked(last, roll());
            last = Some(colour);
            jotted(idea, at, colour)
        })
        .collect();
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
