use std::path::{Path, PathBuf};

use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::{Error, Result, event::DeviceId, store::write_atomic};

const EXTENSION: &str = "md";
const DIGITS: usize = 4;
const MOST_DIGITS: u64 = 999_999_999_999;
const TITLE_AT_MOST: u64 = 4 * 1024;
pub const BODY_AT_MOST: u64 = 500 * 1024;
pub const BODY_ROOMY: u64 = 300 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Doc {
    pub id: String,
    pub title: String,
}

/// The body a write replaced, kept with the print of the body that write left behind. Not every
/// way of writing a document keeps one — the window's own save does not, nor does adding to the
/// end — so the print is what says whether this is still the step back it looks like.
pub fn kept_before(data: &Path, id: &str, body: &str, left: &str) -> Result<()> {
    let at = data.join("originals");
    std::fs::create_dir_all(&at)?;
    let _ = crate::paths::ours_alone(&at);
    let into = resolve(&at, id)?;
    write_atomic(&into, body.as_bytes())?;
    let _ = crate::paths::ours_alone(&into);

    let marked = data.join("originals-at");
    std::fs::create_dir_all(&marked)?;
    let _ = crate::paths::ours_alone(&marked);
    let into = resolve(&marked, id)?;
    // What reaches the disk is the settled body, so hashing what was handed in would leave the
    // print of a text that was never written and go back on nothing.
    write_atomic(
        &into,
        crate::attach::printed(settled(left).as_bytes()).as_bytes(),
    )?;
    let _ = crate::paths::ours_alone(&into);
    Ok(())
}

/// What the document read at when what is kept beside it was set aside.
pub fn before_left_at(data: &Path, id: &str) -> Option<String> {
    let at = resolve(&data.join("originals-at"), id).ok()?;
    std::fs::read_to_string(at).ok()
}

fn base(data: &Path) -> PathBuf {
    data.join("carried")
}

pub fn keep_carried(data: &Path, id: &str, body: &str) -> Result<()> {
    let at = base(data);
    std::fs::create_dir_all(&at)?;
    let _ = crate::paths::ours_alone(&at);
    let into = resolve(&at, id)?;
    write_atomic(&into, body.as_bytes())?;
    let _ = crate::paths::ours_alone(&into);
    Ok(())
}

pub fn carried_print(data: &Path, id: &str) -> Option<String> {
    resolve(&base(data), id)
        .ok()
        .and_then(|at| print_of(&at).ok()?)
}

pub fn read_carried(data: &Path, id: &str) -> Option<String> {
    let at = resolve(&base(data), id).ok()?;
    std::fs::read_to_string(at).ok()
}

pub fn forget_carried(data: &Path, id: &str) {
    if let Ok(at) = resolve(&base(data), id) {
        let _ = std::fs::remove_file(at);
    }
}

pub fn read_before(data: &Path, id: &str) -> Option<String> {
    let at = resolve(&data.join("originals"), id).ok()?;
    std::fs::read_to_string(at).ok()
}

pub fn print_of(at: &Path) -> std::io::Result<Option<String>> {
    match std::fs::metadata(at) {
        Ok(one) if one.len() > BODY_AT_MOST => {
            return Err(std::io::Error::other("a body past the ceiling"));
        }
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    }
    match std::fs::read(at) {
        Ok(bytes) => Ok(Some(crate::attach::printed(&bytes))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Carried(std::collections::BTreeMap<String, String>);

impl Carried {
    pub fn read(data: &Path) -> Self {
        std::fs::read_to_string(ledger(data))
            .ok()
            .and_then(|said| serde_json::from_str(&said).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, data: &Path) -> Result<()> {
        let said = serde_json::to_string(self).map_err(|e| Error::Io(std::io::Error::other(e)))?;
        write_atomic(&ledger(data), said.as_bytes())?;
        let _ = crate::paths::ours_alone(&ledger(data));
        Ok(())
    }

    pub fn of(&self, id: &str) -> Option<&str> {
        self.0.get(id).map(String::as_str)
    }

    pub fn keep(&mut self, id: &str, print: &str) {
        self.0.insert(id.to_string(), print.to_string());
    }

    pub fn forget(&mut self, id: &str) {
        self.0.remove(id);
    }
}

fn ledger(data: &Path) -> PathBuf {
    data.join("carried.json")
}

pub fn forget_what_was_carried(data: &Path) {
    let _ = std::fs::remove_file(ledger(data));
    let _ = std::fs::remove_dir_all(base(data));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Nothing,
    Bring,
    Send,
    TheyDecide,
}

pub fn moved(base: Option<&str>, here: Option<&str>, there: Option<&str>) -> Move {
    match (here, there) {
        (None, None) => Move::Nothing,
        (Some(_), None) => Move::Send,
        (None, Some(_)) => Move::Bring,
        (Some(here), Some(there)) if here == there => Move::Nothing,
        (Some(here), Some(there)) => match base {
            Some(base) if base == here => Move::Bring,
            Some(base) if base == there => Move::Send,
            _ => Move::TheyDecide,
        },
    }
}

/// Steps over fenced blocks line by line, for anything that reads a body and must not read code.
pub fn fencing() -> impl FnMut(&str) -> bool {
    let mut fence = Fencing::default();
    move |line| fence.inside(line)
}

pub fn ends_fenced(text: &str) -> bool {
    let mut fence = Fencing::default();
    for line in text.lines() {
        fence.inside(line);
    }
    fence.open.is_some()
}

pub(crate) fn fenced_spans(text: &str) -> Vec<(usize, usize)> {
    let mut fence = Fencing::default();
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let end = at + line.len();
        if fence.inside(line.trim_end_matches(['\n', '\r'])) {
            match out.last_mut() {
                Some(last) if last.1 == at => last.1 = end,
                _ => out.push((at, end)),
            }
        }
        at = end;
    }
    out
}

/// A fence opens on three or more of one marker and closes on the same, at least as long.
/// The quote prefix comes off first, or a fence written inside a quote is never seen.
#[derive(Default)]
struct Fencing {
    open: Option<(char, usize, usize, usize)>,
    base: usize,
    told: bool,
}

impl Fencing {
    fn inside(&mut self, line: &str) -> bool {
        let (deep, wide, said) = quoted(line);
        if self.open.is_none() {
            self.base = listed(self.base, wide, said);
        }
        let (wide, said) = match bullet(said) {
            Some(after) => (wide + after, &said[after..]),
            None => (wide, said),
        };
        self.told = false;
        let mut marker = None;
        for mark in ['`', '~'] {
            let many = said.chars().take_while(|c| *c == mark).count();
            if many < 3 || wide >= self.base + 4 {
                continue;
            }
            let after = &said[many.min(said.len())..];
            if mark == '`' && after.contains('`') {
                self.told = self.open.is_none();
                continue;
            }
            marker = Some((mark, many));
            break;
        }

        if let Some((open, was, held, room)) = self.open {
            if said.is_empty() {
                return true;
            }
            if deep >= held && wide >= room {
                if let Some((mark, many)) = marker
                    && mark == open
                    && many >= was
                    && deep == held
                {
                    self.open = None;
                }
                return true;
            }
            self.open = None;
        }

        match marker {
            Some((mark, many)) => {
                self.told = nameless(&said[many..]).split_whitespace().count() > 1;
                self.open = Some((mark, many, deep, wide));
                true
            }
            None => false,
        }
    }
}

fn bullet(said: &str) -> Option<usize> {
    let bytes = said.as_bytes();
    let mut at = 0;
    if matches!(bytes.first(), Some(b'-' | b'*' | b'+')) {
        at = 1;
    } else {
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        if at == 0 || at > 9 || !matches!(bytes.get(at), Some(b'.' | b')')) {
            return None;
        }
        at += 1;
    }
    let gap = said[at..]
        .chars()
        .take_while(|c| matches!(c, ' ' | '\t'))
        .count();
    (gap > 0).then_some(at + gap)
}

fn listed(base: usize, wide: usize, said: &str) -> usize {
    if said.is_empty() {
        return base;
    }
    match bullet(said) {
        Some(after) if wide <= base => wide + after,
        _ if wide < base => 0,
        _ => base,
    }
}

pub(crate) fn nameless(said: &str) -> String {
    let mut from = 0;
    while let Some(found) = said[from..].find("title=\"") {
        let at = from + found;
        from = at + 7;
        if said[..at]
            .chars()
            .next_back()
            .is_some_and(|one| one.is_alphanumeric() || one == '_')
        {
            continue;
        }
        let rest = &said[from..];
        let bytes = rest.as_bytes();
        let mut over = 0;
        let end = loop {
            match bytes.get(over) {
                None => return said.to_string(),
                Some(b'\\') => over += 2,
                Some(b'"') => break over,
                _ => over += 1,
            }
        };
        return format!("{} {}", &said[..at], &rest[end + 1..]);
    }
    said.to_string()
}

fn spacing(said: &str) -> usize {
    said.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

fn quoted(line: &str) -> (usize, usize, &str) {
    let mut said = line;
    let mut deep = 0;
    let mut wide = spacing(said);

    while wide < 4 {
        let Some(rest) = said.trim_start().strip_prefix('>') else {
            break;
        };
        said = rest.strip_prefix(' ').unwrap_or(rest);
        deep += 1;
        wide = spacing(said);
    }
    (deep, wide, said.trim())
}

fn quoteless(line: &str) -> &str {
    quoted(line).2
}

pub fn titled(body: &str) -> String {
    let body = body.trim_start_matches('\u{feff}');
    let mut said: Vec<&str> = body.lines().collect();
    if let Some(start) = said.iter().position(|one| !one.trim().is_empty())
        && said[start].trim() == "---"
        && let Some(shuts) = said
            .iter()
            .skip(start + 1)
            .position(|one| one.trim() == "---")
    {
        said.drain(..start + shuts + 2);
    }
    let mut fence = Fencing::default();
    let first = said
        .iter()
        .find_map(|one| {
            if fence.inside(one) {
                return None;
            }
            let flat = one.trim();
            if flat.is_empty() {
                return None;
            }
            let opened = quoteless(one).trim_start_matches('#').trim_start();
            let said = match flat.starts_with('>') {
                true => crate::refs::alerted(opened).unwrap_or(opened),
                false => opened,
            };
            (!wordless(said)).then_some(said)
        })
        .unwrap_or_default();
    crate::text::plainly(unspanned(first).trim())
}

pub fn marked(body: &str, said: &str) -> String {
    let bare = body.trim_start_matches('\u{feff}');
    let mut seen = 0;
    let mut out: Vec<String> = Vec::new();
    let mut done = false;
    let mut fence = Fencing::default();

    for line in bare.lines() {
        let trimmed = line.trim();
        if done || trimmed.is_empty() {
            out.push(line.to_string());
            continue;
        }
        if trimmed == "---"
            && seen == 0
            && bare.lines().filter(|one| one.trim() == "---").count() > 1
        {
            seen = 1;
            out.push(line.to_string());
            continue;
        }
        if seen == 1 {
            if trimmed == "---" {
                seen = 2;
            }
            out.push(line.to_string());
            continue;
        }
        if fence.inside(line) || wordless(trimmed) {
            out.push(line.to_string());
            continue;
        }
        let after = quoteless(line);
        // A row of a table, and a marker with nothing after it, are lines a name would break.
        let bare_line = trimmed.starts_with('|')
            || matches!(crate::refs::alerted(after), Some(rest) if rest.is_empty());
        match bare_line {
            false => {
                out.push(format!("{} ({said})", line.trim_end()));
                done = true;
            }
            true => out.push(line.to_string()),
        }
    }

    if !done {
        return format!("# {said}\n\n{bare}");
    }
    let mut whole = out.join("\n");
    if bare.ends_with('\n') {
        whole.push('\n');
    }
    whole
}

pub fn create(root: &Path, device: &DeviceId, body: &str) -> Result<Doc> {
    std::fs::create_dir_all(root)?;
    let _ = crate::paths::ours_alone(root);
    let mut number = next(root, device);
    loop {
        if number > MOST_DIGITS {
            return Err(Error::OutsideTheStore(format!("{}-{number}", stem(device))));
        }
        let id = format!("{}-{number:0width$}", stem(device), width = DIGITS);
        let at = resolve(root, &id)?;
        match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&at)
        {
            Ok(_) => {
                // `create_new` already won this name against everyone, and taking the lock here
                // would queue creations that never contend for the same body.
                if let Err(e) = written(root, &id, body) {
                    // The name was won before the body was weighed; an empty file must not outlive
                    // the refusal, and the number is spent anyway because it was handed out once.
                    let _ = std::fs::remove_file(&at);
                    spend(root, device, number);
                    return Err(e);
                }
                spend(root, device, number);
                return Ok(Doc {
                    title: titled(body),
                    id,
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => number += 1,
            Err(e) => return Err(Error::Io(e)),
        }
    }
}

pub fn settled(body: &str) -> String {
    if body.is_empty() || body.ends_with('\n') {
        return body.to_string();
    }
    format!("{body}\n")
}

/// The editor hands back what it loaded with its own line endings and without the last newline,
/// and neither is a change somebody made.
pub fn unchanged(was: &str, now: &str) -> bool {
    let plain = |said: &str| settled(&said.replace("\r\n", "\n"));
    plain(was) == plain(now)
}

/// One door for every writer: a body read by one is never written under another.
pub fn write(root: &Path, id: &str, body: &str) -> Result<()> {
    alone(root, || written(root, id, body))
}

fn written(root: &Path, id: &str, body: &str) -> Result<()> {
    let whole = settled(body);
    let bytes = whole.len() as u64;
    if bytes > BODY_AT_MOST {
        return Err(Error::DocumentTooBig {
            bytes,
            limit: BODY_AT_MOST,
        });
    }
    let at = resolve(root, id)?;
    std::fs::create_dir_all(root)?;
    let _ = crate::paths::ours_alone(root);
    write_atomic(&at, whole.as_bytes())
}

/// What happened when a book was asked to name pages at its end. One answer for every door, so
/// the window and the assistant cannot come to differ about when a line is written.
#[derive(Debug, PartialEq, Eq)]
pub enum Naming {
    Wrote { named: Vec<String>, whole: String },
    Fenced,
    WouldRename,
    Nothing,
}

pub fn name_at_end(root: &Path, data: &Path, parent: &str, which: &[&str]) -> Result<Naming> {
    alone(root, || {
        let body = read(root, parent)?;
        let told: Vec<String> = crate::refs::paper_lines(&body)
            .into_iter()
            .map(|(one, _)| one)
            .collect();
        let mut cards: Vec<String> = Vec::new();
        let mut named: Vec<String> = Vec::new();
        for one in which {
            if told.iter().any(|said| said == one) || named.iter().any(|said| said == one) {
                continue;
            }
            // The lock is held over this and all that is wanted is a name: the opening few
            // thousand bytes hold it, and a chapter can run to half a megabyte.
            let title = resolve(root, one)
                .map(|at| opening(&at))
                .unwrap_or_default();
            cards.push(crate::refs::card(one, &title));
            named.push((*one).to_string());
        }
        if cards.is_empty() {
            return Ok(Naming::Nothing);
        }
        if ends_fenced(&body) {
            return Ok(Naming::Fenced);
        }
        let whole = named_after(&body, &cards);
        if titled(&body) != titled(&whole) {
            return Ok(Naming::WouldRename);
        }
        written(root, parent, &whole)?;
        kept_still(data, parent, &body, &whole)?;
        Ok(Naming::Wrote {
            named,
            whole: settled(&whole),
        })
    })
}

/// A line naming a page is not the person's writing, so it does not take their one step back with
/// it: the body kept beside the document stays, and only the print of what it stands against moves.
/// A step back already spent stays spent — carrying that one forward would offer to undo whatever
/// spent it, which is somebody's writing.
fn kept_still(data: &Path, id: &str, was: &str, left: &str) -> Result<()> {
    let stood = crate::attach::printed(settled(was).as_bytes());
    if before_left_at(data, id).as_deref() != Some(stood.as_str()) {
        return Ok(());
    }
    let Ok(into) = resolve(&data.join("originals-at"), id) else {
        return Ok(());
    };
    write_atomic(
        &into,
        crate::attach::printed(settled(left).as_bytes()).as_bytes(),
    )
}

fn named_after(body: &str, cards: &[String]) -> String {
    let ending = match body.contains("\r\n") {
        true => "\r\n",
        false => "\n",
    };
    let mut lines: Vec<String> = body.lines().map(str::to_string).collect();
    while lines.last().is_some_and(|one| one.trim().is_empty()) {
        lines.pop();
    }
    for card in cards {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(card.clone());
    }
    let mut out = lines.join(ending);
    if !out.ends_with('\n') {
        out.push_str(ending);
    }
    out
}

pub fn append(root: &Path, id: &str, body: &str) -> Result<String> {
    alone(root, || {
        let was = read(root, id)?;
        let ending = match was.contains("\r\n") {
            true => "\r\n",
            false => "\n",
        };
        let flat = settled(body.trim_start_matches(['\n', '\r'])).replace("\r\n", "\n");
        let added = match ending {
            "\r\n" => flat.replace('\n', "\r\n"),
            _ => flat,
        };
        let whole = match was.trim_end().is_empty() {
            true => added,
            false => format!("{}{ending}{ending}{added}", was.trim_end()),
        };
        written(root, id, &whole)?;
        Ok(whole)
    })
}

#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    Made { was: String, whole: String },
    Missing,
    Twice(usize),
    TheLot,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Rewrite {
    Made { was: String, whole: String },
    Moved,
}

/// What it said before is kept first: a rewrite that cannot be undone is not written at all.
pub fn rewrite(root: &Path, data: &Path, id: &str, body: &str, print: &str) -> Result<Rewrite> {
    alone(root, || {
        let at = resolve(root, id)?;
        if print_of(&at)?.as_deref() != Some(print) {
            return Ok(Rewrite::Moved);
        }
        let was = read(root, id)?;
        kept_before(data, id, &was, body)?;
        written(root, id, body)?;
        Ok(Rewrite::Made {
            was,
            whole: settled(body),
        })
    })
}

pub fn edit(root: &Path, data: &Path, id: &str, old: &str, new: &str) -> Result<Change> {
    if old.is_empty() {
        return Ok(Change::Missing);
    }
    alone(root, || {
        let was = read(root, id)?;
        let (old, new) = as_written(&was, old, new);
        // A rewrite wearing an edit's clothes: no tool hands a document a new body.
        if was.trim() == old.trim() {
            return Ok(Change::TheLot);
        }
        match was.matches(old.as_str()).count() {
            0 => Ok(Change::Missing),
            1 => {
                let whole = was.replacen(old.as_str(), new.as_str(), 1);
                kept_before(data, id, &was, &whole)?;
                written(root, id, &whole)?;
                Ok(Change::Made { was, whole })
            }
            many => Ok(Change::Twice(many)),
        }
    })
}

/// Work out the new body under the same lock that writes it, so nothing slips in between.
pub fn amend(
    root: &Path,
    data: &Path,
    id: &str,
    make: impl FnOnce(&str) -> Option<String>,
) -> Result<Option<String>> {
    alone(root, || {
        let was = read(root, id)?;
        let Some(whole) = make(&was) else {
            return Ok(None);
        };
        kept_before(data, id, &was, &whole)?;
        written(root, id, &whole)?;
        Ok(Some(settled(&whole)))
    })
}

/// A heading inside a fence is code, not a title: skipping the fences is what keeps a shell
/// prompt from becoming a section.
pub fn headings(body: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut fenced = false;
    for (n, line) in body.lines().enumerate() {
        let bare = line.trim_start();
        if bare.starts_with("```") || bare.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let deep = bare.chars().take_while(|one| *one == '#').count();
        if deep == 0 || deep > 3 || !bare[deep..].starts_with(' ') {
            continue;
        }
        out.push((n + 1, deep, bare[deep + 1..].trim().to_string()));
    }
    out
}

/// Where a section ends: the next heading no deeper than its own, or the end of the document.
/// The blank lines before the next heading separate the two, so they belong to neither.
pub fn section_lines(body: &str, at: usize) -> Option<(usize, usize)> {
    let all = headings(body);
    let lines: Vec<&str> = body.lines().collect();
    section_ends(&all, &lines, at)
}

fn section_ends(
    all: &[(usize, usize, String)],
    lines: &[&str],
    at: usize,
) -> Option<(usize, usize)> {
    let (line, deep, _) = all.get(at)?;
    let mut last = all
        .iter()
        .skip(at + 1)
        .find(|(_, other, _)| other <= deep)
        .map(|(next, _, _)| next - 1)
        .unwrap_or(lines.len());
    while last > *line && lines.get(last - 1).is_some_and(|one| one.trim().is_empty()) {
        last -= 1;
    }
    Some((*line, last))
}

pub fn outlined(body: &str) -> Vec<Heading> {
    let all = headings(body);
    let lines: Vec<&str> = body.lines().collect();
    all.iter()
        .enumerate()
        .map(|(at, (line, level, title))| {
            let (_, to) = section_ends(&all, &lines, at).unwrap_or((*line, *line));
            let chars = lines[line - 1..to]
                .iter()
                .map(|one| one.chars().count() + 1)
                .sum();
            Heading {
                at,
                line: *line,
                level: *level,
                title: title.clone(),
                to,
                chars,
            }
        })
        .collect()
}

/// Cut where the lines really end, so a body that ended in a newline still does. A run that ends
/// before it begins, or begins past the last line, is nothing at all: callers splice a head and a
/// tail around an edit, and a tail that answered with the whole body would duplicate it.
pub fn lines_between(body: &str, from: usize, to: usize) -> String {
    if to < from {
        return String::new();
    }
    let mut start = None;
    let mut end = body.len();
    let mut at = 0usize;
    for (n, line) in body.split_inclusive('\n').enumerate() {
        if n + 1 == from {
            start = Some(at);
        }
        at += line.len();
        if n + 1 == to {
            end = at;
            break;
        }
    }
    let Some(start) = start else {
        return String::new();
    };
    body.get(start..end).unwrap_or_default().to_string()
}

/// What can be worked out from a body without anybody writing it down, and so can never be
/// stale: every field here is read back out of the text each time the file changes.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Card {
    pub print: String,
    pub title: String,
    pub chars: usize,
    pub lines: usize,
    pub words: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outline: Vec<Heading>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    #[serde(default, skip_serializing_if = "none_at_all")]
    pub pictures: usize,
    #[serde(default, skip_serializing_if = "none_at_all")]
    pub links: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Heading {
    pub at: usize,
    pub line: usize,
    pub level: usize,
    pub title: String,
    #[serde(default)]
    pub to: usize,
    #[serde(default)]
    pub chars: usize,
}

fn none_at_all(many: &usize) -> bool {
    *many == 0
}

/// What a card cannot work out because nobody can: somebody read the document and said what it
/// was about. It is kept beside the card, on this machine only, and carries the print of the
/// body it was written against — so a reader can be told it is describing an older text.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Gist {
    pub print: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    pub at: jiff::Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
}

pub const SUMMARY_AT_MOST: usize = 2_000;
pub const NOTES_AT_MOST: usize = 4_000;

const KEYWORDS_AT_MOST: usize = 12;
const A_WORD_AT_LEAST: usize = 4;

impl Card {
    pub fn read_from(body: &str) -> Self {
        let outline = outlined(body);
        let (pictures, links) = pointed_at(body);
        Self {
            print: crate::attach::printed(body.as_bytes()),
            title: titled(body),
            chars: body.chars().count(),
            lines: body.lines().count(),
            words: body.split_whitespace().count(),
            keywords: standing_out(body),
            outline,
            pictures,
            links,
        }
    }
}

/// Searching without opening anything: the cards already hold the text, folded the same way a
/// query is, so the database names the few documents worth reading and only those are read.
/// Without a cache there is nothing to ask, and the caller falls back to walking the files.
pub fn sighted(
    root: &Path,
    cache: Option<&crate::cache::Cache>,
    query: &str,
    most: usize,
    wanted: impl Fn(&str) -> bool,
) -> Option<Vec<Sighting>> {
    let terms = crate::text::terms(query);
    if terms.is_empty() {
        return Some(Vec::new());
    }
    let cache = cache?;
    // A card missing for a file that is there means the text was never read into the database,
    // and answering from what it holds would quietly leave that document out of every search.
    let all = all(root);
    for one in &all {
        card_of(root, Some(cache), &one.id)?;
    }

    let mut found = cache.holding(&terms)?;
    found.sort();
    let mut out = Vec::new();
    for id in found {
        if out.len() >= most || !wanted(&id) {
            continue;
        }
        let Ok(body) = read(root, &id) else { continue };
        let title = titled(&body);
        let line = match crate::text::folded(&title)
            .split_whitespace()
            .collect::<String>()
            .is_empty()
        {
            _ if terms
                .iter()
                .all(|term| crate::text::folded(&title).contains(term.as_str())) =>
            {
                String::new()
            }
            _ => match shown_around(&body, &terms) {
                Some(line) => line,
                None => continue,
            },
        };
        out.push(Sighting { id, title, line });
    }
    Some(out)
}

/// Worked out once per version of a file and remembered locally, because reading two hundred
/// bodies to answer "which of these is about the roof" is a cost nobody should pay twice.
pub fn card_of(root: &Path, cache: Option<&crate::cache::Cache>, id: &str) -> Option<Card> {
    let at = resolve(root, id).ok()?;
    let stamp = stamped(&at)?;
    if let Some(cache) = cache
        && let Some(card) = cache.card(id, stamp)
    {
        return Some(card);
    }
    let body = read(root, id).ok()?;
    let card = Card::read_from(&body);
    if let Some(cache) = cache {
        cache.note_card(id, stamp, &card, &crate::text::folded(&bared(&body)));
    }
    Some(card)
}

/// The cards of many, in one pass. It forgets nothing: the caller asks for a page at a time,
/// and throwing away every card outside that page would leave the cache colder each time.
pub fn cards_of(
    root: &Path,
    cache: Option<&crate::cache::Cache>,
    ids: &[String],
) -> std::collections::BTreeMap<String, Card> {
    ids.iter()
        .filter_map(|id| card_of(root, cache, id).map(|card| (id.clone(), card)))
        .collect()
}

/// What is remembered about documents that are no longer on disk, weighed against the files
/// themselves rather than against whatever somebody happened to ask for.
pub fn forget_stray_cards(root: &Path, cache: Option<&crate::cache::Cache>) {
    let Some(cache) = cache else {
        return;
    };
    cache.forget_cards(&all(root).into_iter().map(|one| one.id).collect());
}

/// A picture is drawn where a link is followed, and an agent choosing a document wants to know
/// which of the two it is walking into.
fn pointed_at(body: &str) -> (usize, usize) {
    let (mut drawn, mut followed) = (0, 0);
    let bytes = body.as_bytes();
    for (at, _) in body.match_indices('[') {
        let Some(shut) = crate::refs::shuts(&body[at + 1..]) else {
            continue;
        };
        if bytes.get(at + 1 + shut + 1) != Some(&b'(') {
            continue;
        }
        match at > 0 && bytes[at - 1] == b'!' {
            true => drawn += 1,
            false => followed += 1,
        }
    }
    (drawn, followed)
}

/// The tags a body carries first, since somebody meant those; then the words it leans on, which
/// nobody meant but which say what it is about all the same.
fn standing_out(body: &str) -> Vec<String> {
    let mut out: Vec<String> = crate::tagging::tags_in(body)
        .iter()
        .map(|one| one.as_str().to_string())
        .collect();

    let mut times: std::collections::HashMap<String, usize> = Default::default();
    for word in crate::text::terms(body) {
        if word.chars().count() < A_WORD_AT_LEAST {
            continue;
        }
        *times.entry(word).or_default() += 1;
    }
    let mut said: Vec<(String, usize)> = times.into_iter().filter(|(_, n)| *n > 1).collect();
    said.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for (word, _) in said {
        if out.len() >= KEYWORDS_AT_MOST {
            break;
        }
        if !out.contains(&word) {
            out.push(word);
        }
    }
    out.truncate(KEYWORDS_AT_MOST);
    out
}

/// A document imported from Windows keeps its line endings, and nobody types those.
fn as_written(was: &str, old: &str, new: &str) -> (String, String) {
    if was.contains(old) || !was.contains("\r\n") {
        return (old.to_string(), new.to_string());
    }
    let crlf = |said: &str| said.replace("\r\n", "\n").replace('\n', "\r\n");
    (crlf(old), crlf(new))
}

const LOCK: &str = ".lock";
/// Waiting beats refusing: the writers that queue here are a saving editor, a sync round and an
/// agent, and every one of them holds it for a write, not for a session.
const LOCK_WAIT_MS: u64 = 2_000;
const LOCK_POLL_MS: u64 = 5;

pub struct Alone(std::fs::File);

impl Drop for Alone {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

/// For a writer that would rather carry on unheld than not write at all.
pub fn hold(root: &Path) -> Option<Alone> {
    std::fs::create_dir_all(root).ok()?;

    let mut waited = 0;
    loop {
        let taken = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(root.join(LOCK))
            .ok()
            .filter(|file| file.try_lock().is_ok());
        if let Some(file) = taken {
            return Some(Alone(file));
        }
        if waited >= LOCK_WAIT_MS {
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(LOCK_POLL_MS));
        waited += LOCK_POLL_MS;
    }
}

fn alone<T>(root: &Path, work: impl FnOnce() -> Result<T>) -> Result<T> {
    let held = hold(root).ok_or(Error::AlreadyRunning)?;
    let out = work();
    drop(held);
    out
}

/// What a watcher compares to tell that a body moved: a body writes no event to notice it by.
pub fn print(root: &Path) -> String {
    let Ok(entries) = std::fs::read_dir(root) else {
        return String::new();
    };
    let mut parts: Vec<String> = entries
        .filter_map(|one| one.ok())
        .filter(|one| one.file_type().map(|kind| kind.is_file()).unwrap_or(false))
        .filter_map(|one| {
            let at = one.path();
            let id = named(&at)?;
            let told = one.metadata().ok()?;
            let when = told
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?;
            Some(format!("{id}:{}:{}", told.len(), when.as_nanos()))
        })
        .collect();
    parts.sort();
    parts.join("|")
}

pub const IMPORTS: [&str; 3] = ["md", "markdown", "txt"];

pub fn importable(at: &Path) -> bool {
    match at.extension().and_then(|one| one.to_str()) {
        None => at.file_name().is_some(),
        Some(one) => IMPORTS.contains(&one.to_lowercase().as_str()),
    }
}

pub fn read_outside(at: &Path) -> Result<String> {
    if !importable(at) {
        return Err(Error::OutsideTheStore(at.display().to_string()));
    }
    let file = std::fs::File::open(at)?;
    if !file.metadata()?.is_file() {
        return Err(Error::OutsideTheStore(at.display().to_string()));
    }
    let mut body = String::new();
    let read = file.take(BODY_AT_MOST + 1).read_to_string(&mut body)? as u64;
    if read > BODY_AT_MOST {
        return Err(Error::DocumentTooBig {
            bytes: read,
            limit: BODY_AT_MOST,
        });
    }
    Ok(body)
}

pub fn read(root: &Path, id: &str) -> Result<String> {
    let at = resolve(root, id)?;
    let file = std::fs::File::open(&at)?;
    if !file.metadata()?.is_file() {
        return Err(Error::OutsideTheStore(id.to_string()));
    }
    let mut body = String::new();
    let read = file.take(BODY_AT_MOST + 1).read_to_string(&mut body)? as u64;
    if read > BODY_AT_MOST {
        return Err(Error::DocumentTooBig {
            bytes: read,
            limit: BODY_AT_MOST,
        });
    }
    Ok(body)
}

/// What came out, and what could not: a page missing from disk is left behind, and saying so
/// is the only way the person learns their book came out a chapter short.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Taken {
    pub files: usize,
    pub missed: usize,
    pub left: Vec<String>,
}

pub fn exported(data: &Path, id: &str, into: &Path) -> Result<Taken> {
    with_pages(data, id, &[], into, None)
}

/// The pages travel with the document: a book exported by its cover alone is not the book.
pub fn with_pages(
    data: &Path,
    id: &str,
    pages: &[String],
    into: &Path,
    also: Option<&Path>,
) -> Result<Taken> {
    laid_out_as(data, id, pages, into, None, also)
}

pub fn laid_out_as(
    data: &Path,
    id: &str,
    pages: &[String],
    into: &Path,
    called: Option<&str>,
    also: Option<&Path>,
) -> Result<Taken> {
    if into.starts_with(data) || data.starts_with(into) {
        return Err(Error::OutsideTheStore(into.display().to_string()));
    }
    let body = read(&data.join("docs"), id)?;

    let named = match called {
        Some(one) => one.to_string(),
        None => {
            let named = titled(&body);
            spelled(if named.is_empty() { id } else { &named })
        }
    };
    let folder = into.join(&named);
    std::fs::create_dir_all(into)?;
    if folder.exists() {
        return Err(Error::AlreadyTakenOut(named.clone()));
    }
    std::fs::create_dir(&folder)?;

    let wide = pages.len().to_string().len().max(2);
    let mut missed = 0;
    let mut written: Vec<(String, String, String)> = Vec::new();
    for (n, page) in pages.iter().enumerate() {
        let Ok(body) = read(&data.join("docs"), page) else {
            missed += 1;
            continue;
        };
        let title = titled(&body);
        let title = spelled(if title.is_empty() { page } else { &title });
        let at = format!("{:0wide$} {title}.{EXTENSION}", n + 1);
        written.push((page.clone(), at, body));
    }

    let beside = |body: &str| {
        written
            .iter()
            .fold(body.to_string(), |body, (file, at, _)| {
                let named = format!("{}{file}", crate::refs::DOC);
                unpictured(
                    &body
                        .replace(&format!("](<{named}>)"), &format!("](<{at}>)"))
                        .replace(&format!("]({named})"), &format!("](<{at}>)")),
                    at,
                )
            })
    };

    let mut taken = laid_out(
        data,
        &beside(&body),
        &folder,
        &format!("{named}.{EXTENSION}"),
        also,
    )?;
    for (_, at, body) in &written {
        let more = laid_out(data, &beside(body), &folder, at, also)?;
        taken.files += more.files;
        for one in more.left {
            left_behind(&mut taken.left, one);
        }
    }
    Ok(Taken {
        files: taken.files,
        missed,
        left: taken.left,
    })
}

fn unpictured(body: &str, at: &str) -> String {
    let shut = format!("](<{at}>)");
    let mut said = String::with_capacity(body.len());
    let mut from = 0;
    while let Some(found) = body[from..].find(&shut).map(|n| from + n) {
        let opened = began(&body[..found]);
        let cut = match opened {
            Some(open) if body[..open].ends_with('!') => open - 1,
            _ => found,
        };
        said.push_str(&body[from..cut]);
        if cut != found {
            said.push_str(&body[cut + 1..found]);
        }
        said.push_str(&shut);
        from = found + shut.len();
    }
    said.push_str(&body[from..]);
    said
}

fn began(before: &str) -> Option<usize> {
    let mut escaped = false;
    let mut open = None;
    for (at, c) in before.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' => escaped = true,
            '[' => open = Some(at),
            _ => {}
        }
    }
    open
}

fn left_behind(left: &mut Vec<String>, one: String) {
    if !left.contains(&one) {
        left.push(one);
    }
}

fn shelved<'a>(from: &'a Path, held: &Path, also: Option<&Path>) -> Option<&'a Path> {
    from.strip_prefix(held)
        .ok()
        .or_else(|| also.and_then(|beside| from.strip_prefix(beside.join("attachments")).ok()))
}

fn laid_out(
    data: &Path,
    body: &str,
    folder: &Path,
    named: &str,
    also: Option<&Path>,
) -> Result<Taken> {
    write_atomic(&folder.join(named), body.as_bytes())?;

    let held = data.join("attachments");
    let mut taken = Taken::default();
    for one in crate::refs::extract(body).into_iter().map(|one| one.target) {
        if !one.starts_with("attachments/") {
            continue;
        }
        let Ok(from) = crate::attach::found(&one, data, also) else {
            left_behind(&mut taken.left, one);
            continue;
        };
        let Some(rest) = shelved(&from, &held, also) else {
            continue;
        };
        if !from.is_file() {
            left_behind(&mut taken.left, one);
            continue;
        }
        let at = folder.join("attachments").join(rest);
        if let Some(under) = at.parent() {
            std::fs::create_dir_all(under)?;
        }
        if std::fs::copy(&from, &at).is_ok() {
            taken.files += 1;
        } else {
            left_behind(&mut taken.left, one);
        }
    }
    Ok(taken)
}

pub fn spelled(said: &str) -> String {
    let flat: String = said
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let flat = flat.trim().replace(' ', "-");
    let flat: String = flat.chars().take(60).collect();
    let flat = flat.trim_matches('-').to_string();
    if flat.is_empty() || crate::attach::reserved(&flat) {
        "documento".into()
    } else {
        flat
    }
}

pub fn referenced(root: &Path) -> Vec<String> {
    all(root)
        .iter()
        .filter_map(|doc| match read(root, &doc.id) {
            Ok(body) => Some(body),
            Err(e) => {
                crate::witness::warn(
                    crate::witness::channel::ATTACH,
                    "a document could not be read while counting what is still named",
                    &[
                        ("id", crate::witness::Fact::Id(doc.id.clone())),
                        ("why", crate::witness::Fact::Why(e.to_string())),
                    ],
                );
                None
            }
        })
        .flat_map(|body| crate::refs::extract(&body))
        .map(|one| one.target)
        .collect()
}

pub fn remove(root: &Path, id: &str) -> Result<()> {
    let at = resolve(root, id)?;
    match std::fs::remove_file(at) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::Io(e)),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stray {
    pub file: String,
    pub title: String,
    pub bytes: u64,
    pub when: i64,
}

pub fn strayed(root: &Path, alive: &[String]) -> Vec<Stray> {
    loose(root, alive)
        .into_iter()
        .map(|one| {
            let at = resolve(root, &one.id).ok();
            let told = at.as_ref().and_then(|at| std::fs::metadata(at).ok());
            Stray {
                title: one.title,
                file: one.id,
                bytes: told.as_ref().map(|one| one.len()).unwrap_or(0),
                when: told
                    .and_then(|one| one.modified().ok())
                    .and_then(|one| one.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|one| one.as_secs() as i64)
                    .unwrap_or(0),
            }
        })
        .collect()
}

pub fn missing(root: &Path, alive: &[String]) -> Vec<String> {
    alive
        .iter()
        .filter(|id| !resolve(root, id).is_ok_and(|at| at.exists()))
        .cloned()
        .collect()
}

pub fn loose(root: &Path, alive: &[String]) -> Vec<Doc> {
    let held: std::collections::BTreeSet<&str> = alive.iter().map(String::as_str).collect();
    all(root)
        .into_iter()
        .filter(|one| !held.contains(one.id.as_str()))
        .collect()
}

pub fn sweep(root: &Path, shed: &std::collections::BTreeSet<String>) -> usize {
    let mut gone = 0;
    for id in shed {
        let Ok(at) = resolve(root, id) else {
            continue;
        };
        match std::fs::remove_file(&at) {
            Ok(()) => gone += 1,
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
            Err(why) => crate::witness::warn(
                crate::witness::channel::STORE,
                "a deleted document kept its file, and it is swept again at the next opening",
                &[
                    ("at", crate::witness::Fact::Id(id.clone())),
                    ("why", crate::witness::Fact::Why(why.to_string())),
                ],
            ),
        }
        forget_carried(root.parent().unwrap_or(root), id);
    }
    if gone > 0 {
        crate::witness::note(
            crate::witness::channel::STORE,
            "documents deleted elsewhere had their files taken out here",
            &[("count", crate::witness::Fact::Count(gone))],
        );
    }
    gone
}

pub fn names(root: &Path) -> std::collections::BTreeSet<String> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return std::collections::BTreeSet::new();
    };
    entries
        .filter_map(|one| one.ok())
        .filter(|one| one.file_type().map(|kind| kind.is_file()).unwrap_or(false))
        .filter_map(|one| named(&one.path()))
        .collect()
}

pub fn title_of(root: &Path, id: &str) -> Option<String> {
    let at = resolve(root, id).ok()?;
    at.is_file().then(|| opening(&at))
}

pub fn all(root: &Path) -> Vec<Doc> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut found: Vec<Doc> = entries
        .filter_map(|one| one.ok())
        .filter(|one| one.file_type().map(|kind| kind.is_file()).unwrap_or(false))
        .filter_map(|one| {
            let at = one.path();
            let id = named(&at)?;
            Some(Doc {
                title: opening(&at),
                id,
            })
        })
        .collect();
    found.sort_by(|a, b| a.id.cmp(&b.id));
    found
}

pub fn resolve(root: &Path, id: &str) -> Result<PathBuf> {
    if !well_formed(id) {
        return Err(Error::OutsideTheStore(id.to_string()));
    }
    Ok(root.join(format!("{id}.{EXTENSION}")))
}

fn well_formed(id: &str) -> bool {
    let Some((device, number)) = id.rsplit_once('-') else {
        return false;
    };
    crate::store::is_device_name(device)
        && !number.is_empty()
        && number.len() <= 12
        && number.chars().all(|c| c.is_ascii_digit())
}

fn named(at: &Path) -> Option<String> {
    if at.extension()? != EXTENSION {
        return None;
    }
    let id = at.file_stem()?.to_str()?.to_string();
    well_formed(&id).then_some(id)
}

fn spent(root: &Path, device: &DeviceId) -> PathBuf {
    root.join(format!(".spent-{}", stem(device)))
}

fn spend(root: &Path, device: &DeviceId, number: u64) {
    let at = spent(root, device);
    if let Err(e) = write_atomic(&at, number.to_string().as_bytes()) {
        crate::witness::warn(
            crate::witness::channel::STORE,
            "the highest document name given out could not be kept, so a deleted one could come back",
            &[("why", crate::witness::Fact::Why(e.to_string()))],
        );
    }
}

fn next(root: &Path, device: &DeviceId) -> u64 {
    let mine = format!("{}-", stem(device));
    let on_disk = all(root)
        .iter()
        .filter_map(|doc| doc.id.strip_prefix(&mine))
        .filter_map(|number| number.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    let given = std::fs::read_to_string(spent(root, device))
        .ok()
        .and_then(|said| said.trim().parse::<u64>().ok())
        .unwrap_or(0);
    on_disk.max(given) + 1
}

fn stem(device: &DeviceId) -> String {
    let plain = device.0.strip_prefix("dev_").unwrap_or(&device.0);
    let kept: String = plain
        .chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| {
            if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(48)
        .collect();
    if kept.is_empty() {
        "device".to_string()
    } else {
        kept
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sighting {
    pub id: String,
    pub title: String,
    pub line: String,
}

pub const SAID_AT_MOST: usize = 160;

fn skipped(chars: &mut std::iter::Peekable<std::str::Chars>, opens: char, shuts: char) {
    let mut depth = 1;
    for one in chars.by_ref() {
        if one == opens {
            depth += 1;
        } else if one == shuts {
            depth -= 1;
            if depth == 0 {
                return;
            }
        }
    }
}

fn unspanned(line: &str) -> String {
    if !line.contains("<span data-ico=") {
        return line.to_string();
    }
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(at) = rest.find("<span data-ico=") {
        out.push_str(&rest[..at]);
        let after = &rest[at..];
        let Some(shut) = after.find('>') else {
            return out + after;
        };
        let inner = &after[shut + 1..];
        match inner.find("</span>") {
            Some(ends) => {
                out.push_str(&inner[..ends]);
                rest = &inner[ends + "</span>".len()..];
            }
            None => return out + inner,
        }
    }
    out.push_str(rest);
    out
}

pub fn bare(line: &str) -> String {
    let line = &unspanned(line);
    let flat = line.trim();
    let quoted = flat.starts_with('>');
    let said = flat
        .trim_start_matches(['>', '#', ' '])
        .trim_start()
        .trim_start_matches(['-', '*', '+'])
        .trim_start();
    let said = said
        .strip_prefix("[ ] ")
        .or(said.strip_prefix("[x] "))
        .unwrap_or(said);
    let said = match quoted {
        true => crate::refs::alerted(said).unwrap_or(said),
        false => said,
    };

    let mut out = String::with_capacity(said.len());
    let mut chars = said.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => out.extend(chars.next()),
            '`' | '*' => {}
            '~' if chars.peek() == Some(&'~') => {
                chars.next();
            }
            '!' if chars.peek() == Some(&'[') => {
                chars.next();
            }
            '[' => {}
            ']' => match chars.peek() {
                Some('(') => {
                    chars.next();
                    skipped(&mut chars, '(', ')');
                }
                Some('[') => {
                    chars.next();
                    skipped(&mut chars, '[', ']');
                }
                _ => {}
            },
            '|' => out.push(' '),
            _ => out.push(c),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn wordless(said: &str) -> bool {
    said.is_empty()
        || !said
            .chars()
            .any(|c| c.is_alphanumeric() || matches!(c, '¿' | '?' | '¡' | '!'))
}

fn bared(body: &str) -> String {
    body.lines().map(bare).collect::<Vec<_>>().join("\n")
}

fn cut(said: String) -> String {
    said.chars().take(SAID_AT_MOST).collect()
}

fn shown_around(body: &str, terms: &[String]) -> Option<String> {
    let mut best: Option<(usize, String)> = None;
    let mut backup: Option<(usize, String)> = None;
    let mut fence = Fencing::default();

    for line in body.lines() {
        let drawn = fence.inside(line);
        let said = bare(line);
        let flat = crate::text::folded(&said);
        let held = terms.iter().filter(|term| flat.contains(*term)).count();
        if held == 0 {
            continue;
        }
        let slot = if drawn || wordless(&said) {
            &mut backup
        } else {
            &mut best
        };
        if slot.as_ref().is_none_or(|(had, _)| held > *had) {
            *slot = Some((held, said));
        }
        if best.as_ref().is_some_and(|(had, _)| *had == terms.len()) {
            break;
        }
    }
    best.or(backup).map(|(_, said)| cut(said))
}

pub const CORPUS_AT_MOST: usize = 64 * 1024 * 1024;

struct Held {
    stamp: (u64, u64),
    flat: String,
}

pub struct Corpus {
    kept: std::collections::HashMap<String, Held>,
    bytes: usize,
    room: usize,
}

impl Default for Corpus {
    fn default() -> Self {
        Self::holding(CORPUS_AT_MOST)
    }
}

fn stamped(at: &Path) -> Option<(u64, u64)> {
    let told = std::fs::metadata(at).ok()?;
    let when = told
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;
    Some((told.len(), when.as_nanos() as u64))
}

impl Corpus {
    pub fn holding(room: usize) -> Self {
        Self {
            kept: std::collections::HashMap::new(),
            bytes: 0,
            room,
        }
    }

    pub fn forget(&mut self, id: &str) {
        if let Some(gone) = self.kept.remove(id) {
            self.bytes -= gone.flat.len();
        }
    }

    pub fn held(&self) -> usize {
        self.bytes
    }

    fn flattened(&mut self, root: &Path, id: &str) -> Option<&str> {
        let at = resolve(root, id).ok()?;
        let stamp = stamped(&at)?;
        if !self.kept.get(id).is_some_and(|one| one.stamp == stamp) {
            self.forget(id);
            let flat = crate::text::folded(&bared(&read(root, id).ok()?));
            if self.bytes + flat.len() > self.room {
                return None;
            }
            self.bytes += flat.len();
            self.kept.insert(id.to_string(), Held { stamp, flat });
        }
        self.kept.get(id).map(|one| one.flat.as_str())
    }

    pub fn searching(
        &mut self,
        root: &Path,
        query: &str,
        most: usize,
        wanted: impl Fn(&str) -> bool,
    ) -> Vec<Sighting> {
        let terms = crate::text::terms(query);
        if terms.is_empty() {
            return Vec::new();
        }

        let mut found = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for doc in all(root) {
            seen.insert(doc.id.clone());
            if found.len() >= most || !wanted(&doc.id) {
                continue;
            }
            let title = doc.title.clone();
            let flat = crate::text::folded(&title);
            let missing: Vec<&String> = terms
                .iter()
                .filter(|term| !flat.contains(term.as_str()))
                .collect();
            if missing.is_empty() {
                found.push(Sighting {
                    id: doc.id,
                    title,
                    line: String::new(),
                });
                continue;
            }
            let sighted = match self.flattened(root, &doc.id) {
                Some(flat) => missing.iter().all(|term| flat.contains(term.as_str())),
                None => read(root, &doc.id)
                    .map(|body| {
                        let flat = crate::text::folded(&bared(&body));
                        missing.iter().all(|term| flat.contains(term.as_str()))
                    })
                    .unwrap_or(false),
            };
            if !sighted {
                continue;
            }
            let Ok(body) = read(root, &doc.id) else {
                continue;
            };
            if let Some(line) = shown_around(&body, &terms) {
                found.push(Sighting {
                    id: doc.id,
                    title,
                    line,
                });
            }
        }
        self.kept.retain(|id, _| seen.contains(id));
        self.bytes = self.kept.values().map(|one| one.flat.len()).sum();
        found
    }
}

fn opening(at: &Path) -> String {
    let Ok(file) = std::fs::File::open(at) else {
        return String::new();
    };
    let mut head = Vec::new();
    let _ = file.take(TITLE_AT_MOST).read_to_end(&mut head);
    titled(&String::from_utf8_lossy(&head))
}

#[cfg(test)]
#[path = "docs_naming.rs"]
mod naming;

#[cfg(test)]
#[path = "docs_test.rs"]
mod tests;

/// What the window's editor destroys the first time somebody opens a document. It rewrites the
/// whole file, so anything it cannot represent is gone on the first keystroke.
fn ruled(rest: &str) -> bool {
    let Some(mark) = rest.chars().find(|c| !c.is_whitespace()) else {
        return false;
    };
    if !matches!(mark, '*' | '_' | '-') {
        return false;
    }
    let mut many = 0;
    for one in rest.chars() {
        if one == mark {
            many += 1;
        } else if one != ' ' && one != '\t' {
            return false;
        }
    }
    many >= 3
}

fn dashed(line: &str) -> bool {
    let said = quoteless(line);
    said.starts_with('|')
        && said.contains('-')
        && said.chars().all(|one| matches!(one, '|' | '-' | ':' | ' '))
}

fn blocked(line: &str, next: &str) -> bool {
    let (_, wide, said) = quoted(line);
    if wide >= 4 || ruled(said) {
        return false;
    }
    let Some(after) = bullet(said) else {
        return false;
    };
    let mark = said[..after].trim_end().len();
    let gap: usize = said[mark..after]
        .chars()
        .map(|one| if one == '\t' { 4 } else { 1 })
        .sum();
    if gap >= 5 {
        return true;
    }
    let rest = &said[after..];
    if rest.starts_with('>')
        || rest.starts_with("```")
        || rest.starts_with("~~~")
        || rest.starts_with("![")
        || bullet(rest).is_some()
        || ruled(rest)
    {
        return true;
    }
    let hashed = rest.chars().take_while(|c| *c == '#').count();
    let after_hash = &rest[hashed..];
    if (1..=6).contains(&hashed) && (after_hash.is_empty() || after_hash.starts_with([' ', '\t'])) {
        return true;
    }
    rest.starts_with('|') && dashed(next)
}

pub fn survives(body: &str) -> std::result::Result<(), &'static str> {
    let body = body.trim_start_matches('\u{feff}');
    if fronted(body) {
        return Err("YAML frontmatter");
    }
    let mut fence = Fencing::default();

    let said: Vec<&str> = body.lines().collect();
    for (at, line) in said.iter().enumerate() {
        let coded = fence.open.is_some();
        let within = fence.inside(line);
        if fence.told {
            return Err("what a fence says after its language");
        }
        if !coded && blocked(line, said.get(at + 1).unwrap_or(&"")) {
            return Err("a list item that opens on a block");
        }
        if within {
            continue;
        }
        if line.starts_with("    ") || line.starts_with('\t') {
            continue;
        }

        let plain = outside_code_spans(line);
        if let Some(why) = markup(&plain) {
            return Err(why);
        }
        let flat = plain.trim();
        if dollared(flat) {
            return Err("maths written between dollars");
        }
        if noted(flat) {
            return Err("footnotes");
        }
        if linked(flat, said.get(at + 1).unwrap_or(&"")) {
            return Err("reference links");
        }
    }
    Ok(())
}

fn fronted(body: &str) -> bool {
    let mut lines = body.trim_start_matches('\u{feff}').lines();
    if lines.next().map(str::trim) != Some("---") {
        return false;
    }
    lines.any(|line| line.trim() == "---")
}

fn outside_code_spans(line: &str) -> String {
    let bytes = line.as_bytes();
    let ticks = |from: usize| bytes[from..].iter().take_while(|c| **c == b'`').count();
    let mut out = String::with_capacity(line.len());
    let mut at = 0;

    while at < bytes.len() {
        if bytes[at] != b'`' {
            let one = line[at..].chars().next().unwrap_or('\0');
            out.push(one);
            at += one.len_utf8();
            continue;
        }
        let open = ticks(at);
        let mut scan = at + open;
        let mut shut = None;
        while scan < bytes.len() {
            if bytes[scan] != b'`' {
                scan += 1;
                continue;
            }
            let many = ticks(scan);
            if many == open {
                shut = Some(scan + many);
                break;
            }
            scan += many;
        }
        match shut {
            Some(end) => at = end,
            None => {
                out.push_str(&line[at..at + open]);
                at += open;
            }
        }
    }
    out
}

fn dollared(line: &str) -> bool {
    let said = line.trim();
    if said.starts_with("$$") {
        return true;
    }
    let bytes = said.as_bytes();
    let mut at = 0;
    while let Some(found) = said[at..].find("$$") {
        let start = at + found;
        if start == 0 || bytes[start - 1] != b'\\' {
            return true;
        }
        at = start + 2;
    }
    false
}

fn noted(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut at = 0;
    while let Some(found) = line[at..].find("[^") {
        let start = at + found;
        at = start + 2;
        if start > 0 && bytes[start - 1] == b'\\' {
            continue;
        }
        let Some(end) = line[at..].find(']') else {
            return false;
        };
        if end > 0 && !matches!(bytes.get(at + end + 1), Some(b'(') | Some(b'[')) {
            return true;
        }
        at += end + 1;
    }
    false
}

fn labelled(rest: &str) -> Option<(&str, &str)> {
    let bytes = rest.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 2,
            b']' if bytes.get(at + 1) == Some(&b':') => {
                return Some((&rest[..at], &rest[at + 2..]));
            }
            b']' => return None,
            _ => at += 1,
        }
    }
    None
}

fn linked(line: &str, next: &str) -> bool {
    let Some(rest) = line.strip_prefix('[') else {
        return false;
    };
    let Some((label, told)) = labelled(rest) else {
        return false;
    };
    !label.is_empty()
        && !label.starts_with('^')
        && (!told.trim().is_empty() || !next.trim().is_empty())
}

/// A `<` that opens a tag, wherever it sits on the line. An autolink is markdown and comes
/// back untouched, so it is not markup.
pub(crate) fn markup(line: &str) -> Option<&'static str> {
    let bytes = line.as_bytes();
    let mut shut = 0;
    for at in 0..bytes.len() {
        if bytes[at] == b'&' && entity(&line[at..]) {
            return Some("HTML entities");
        }
        if bytes[at] != b'<' {
            continue;
        }
        let rest = &line[at + 1..];
        if rest.starts_with("!--") {
            return Some("HTML comments");
        }
        if !rest.starts_with(|c: char| c.is_ascii_alphabetic() || c == '/' || c == '!' || c == '?')
        {
            continue;
        }
        if shut <= at {
            shut = match rest.find('>') {
                Some(end) => at + 1 + end,
                None => line.len(),
            };
        }
        let inner = &line[at + 1..shut];
        if inner.contains("://") || (inner.contains('@') && !inner.contains(' ')) {
            continue;
        }
        // `](<…>)` is where markdown puts a target that has spaces or brackets in it, and it is
        // what an attachment is written as.
        if line[..at].ends_with("](") && !inner.contains('<') && anchored(&line[..at - 2]) {
            continue;
        }
        if kept(inner) {
            continue;
        }
        return Some("HTML");
    }
    None
}

fn quotedly<'a>(said: &'a str, name: &str, plain: fn(char) -> bool) -> Option<&'a str> {
    let rest = said.strip_prefix(name)?.strip_prefix("=\"")?;
    let (value, after) = rest.split_once('"')?;
    if value.is_empty() || !value.chars().all(plain) {
        return None;
    }
    Some(after.trim_start())
}

fn named_or_marked(said: &str) -> Option<&str> {
    let rest = said.strip_prefix("data-ico")?.strip_prefix("=\"")?;
    let (value, after) = rest.split_once('"')?;
    if value.is_empty() {
        return None;
    }
    let plain = value
        .chars()
        .all(|one| one.is_ascii_alphanumeric() || one == '-');
    (plain || crate::model::icon::a_mark(value)).then(|| after.trim_start())
}

fn iconed(inner: &str) -> bool {
    let Some(rest) = inner
        .get(..5)
        .filter(|one| one.eq_ignore_ascii_case("span "))
        .map(|_| inner[5..].trim())
    else {
        return false;
    };
    let Some(after) = named_or_marked(rest) else {
        return false;
    };
    if after.is_empty() {
        return true;
    }
    quotedly(after, "data-hue", |one| one.is_ascii_alphabetic()).is_some_and(str::is_empty)
}

pub(crate) fn kept(inner: &str) -> bool {
    for one in ["u", "/u", "mark", "/mark", "/span"] {
        if inner.eq_ignore_ascii_case(one) {
            return true;
        }
    }
    if iconed(inner) {
        return true;
    }
    let Some(pen) = inner
        .get(..5)
        .filter(|one| one.eq_ignore_ascii_case("mark "))
        .map(|_| inner[5..].trim())
    else {
        return false;
    };
    let Some(said) = pen
        .strip_prefix("data-pen=\"")
        .and_then(|one| one.strip_suffix('"'))
    else {
        return false;
    };
    !said.is_empty() && said.chars().all(|one| one.is_ascii_alphabetic())
}

fn anchored(said: &str) -> bool {
    match said.rfind('[') {
        Some(open) => !said[open + 1..].contains(']'),
        None => false,
    }
}

fn entity(from: &str) -> bool {
    let Some(shut) = from.find(';') else {
        return false;
    };
    let name = &from[1..shut];
    name.len() > 1 && name.len() < 12 && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '#')
}

#[cfg(test)]
#[path = "docs_survival.rs"]
mod survival;

#[cfg(test)]
#[path = "docs_cards.rs"]
mod cards;
