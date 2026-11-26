use tisty_markdown::front;

use super::Refused;
use super::trigger::{self, Trigger};
use crate::event::DeviceId;

pub const KEY: &str = front::AGENT;
pub const GLOBAL: &str = "global";
const BOM: char = '\u{feff}';
const MARKDOWN: [&str; 3] = ["md", "markdown", "mdc"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Rule,
    Skill,
    Memory,
    Hook,
}

impl Kind {
    pub fn word(self) -> &'static str {
        match self {
            Self::Rule => "rule",
            Self::Skill => "skill",
            Self::Memory => "memory",
            Self::Hook => "hook",
        }
    }

    fn read(word: &str) -> Option<Self> {
        [Self::Rule, Self::Skill, Self::Memory, Self::Hook]
            .into_iter()
            .find(|one| one.word() == word)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Endings {
    #[default]
    Lf,
    Crlf,
    AsIs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub level: String,
    pub kind: Kind,
    pub name: String,
    pub endings: Endings,
    pub bom: bool,
    pub open_end: bool,
    /// The one computer it is for; without it, the card is for every computer.
    pub computer: Option<DeviceId>,
    pub trigger: Option<Trigger>,
}

impl Card {
    pub fn of(body: &str) -> Option<Self> {
        let fields = front::fields(body);
        let field = fields
            .iter()
            .find(|one| one.key.eq_ignore_ascii_case(KEY))?;
        let (address, flags) = field.values.split_first()?;
        let (level, kind, name) = front::agent_address(address)?;
        let mut card = Self {
            level: level.to_string(),
            kind: Kind::read(kind)?,
            name: name.to_string(),
            endings: Endings::Lf,
            bom: false,
            open_end: false,
            computer: None,
            trigger: None,
        };
        if card.kind == Kind::Hook {
            let fits = card.level == GLOBAL && trigger::hook_name(&card.name);
            card.trigger = Some(trigger::read(&fields).filter(|_| fits)?);
        }
        for flag in flags {
            match flag.as_str() {
                "crlf" => card.endings = Endings::Crlf,
                "as-is" => card.endings = Endings::AsIs,
                "bom" => card.bom = true,
                "open-end" => card.open_end = true,
                one if card.computer.is_none() && front::agent_computer(one).is_some() => {
                    card.computer = front::agent_computer(one).map(|id| DeviceId(id.to_string()));
                }
                // A flag this Tisty does not know would be bytes it cannot promise.
                _ => return None,
            }
        }
        Some(card)
    }

    pub fn address(&self) -> String {
        format!("{}/{}/{}", self.level, self.kind.word(), self.name)
    }

    pub fn code(&self) -> bool {
        !MARKDOWN.contains(&extension(&self.name).as_str())
    }

    fn line(&self) -> String {
        let on = self
            .computer
            .as_ref()
            .map(|one| format!("{}{}", front::AGENT_ON, one.0));
        let flags: Vec<&str> = [
            (self.endings == Endings::Crlf).then_some("crlf"),
            (self.endings == Endings::AsIs).then_some("as-is"),
            self.bom.then_some("bom"),
            self.open_end.then_some("open-end"),
            on.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect();
        match flags.is_empty() {
            true => format!("{KEY}: {}", self.address()),
            false => format!("{KEY}: [{}, {}]", self.address(), flags.join(", ")),
        }
    }
}

fn extension(name: &str) -> String {
    let leaf = name.rsplit('/').next().unwrap_or(name);
    match leaf.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => ext.to_ascii_lowercase(),
        _ => String::new(),
    }
}

pub fn doc_of(level: &str, kind: Kind, name: &str, file: &[u8]) -> Result<String, Refused> {
    if kind == Kind::Hook || !front::agent_level(level) || !front::agent_name(name) {
        return Err(Refused::Name);
    }
    let text = std::str::from_utf8(file).map_err(|_| Refused::NotText)?;
    let (bom, text) = match text.strip_prefix(BOM) {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let card = Card {
        level: level.to_string(),
        kind,
        name: name.to_string(),
        endings: endings_of(text),
        bom,
        open_end: !text.is_empty() && !text.ends_with('\n'),
        computer: None,
        trigger: None,
    };
    let title = format!("title: {level} · {name}");
    Ok(format!(
        "---\n{}\n{title}\n---\n{}",
        card.line(),
        held(&card, text)
    ))
}

// A hook is made only here, with its trigger, so nothing brought from disk becomes one.
pub fn hook_doc(
    name: &str,
    computer: Option<&DeviceId>,
    trigger: &Trigger,
    script: &str,
) -> Result<String, Refused> {
    if !trigger::hook_name(name) {
        return Err(Refused::Name);
    }
    let card = Card {
        level: GLOBAL.to_string(),
        kind: Kind::Hook,
        name: name.to_string(),
        endings: endings_of(script),
        bom: false,
        open_end: !script.is_empty() && !script.ends_with('\n'),
        computer: computer.cloned(),
        trigger: Some(trigger.clone()),
    };
    let body = format!(
        "---\n{}\ntitle: {name}\n{}---\n{}",
        card.line(),
        trigger.lines(),
        held(&card, script)
    );
    Card::of(&body).map(|_| body).ok_or(Refused::NoCard)
}

pub fn brought(body: &str, file: &[u8]) -> Result<String, Refused> {
    let was = Card::of(body).ok_or(Refused::NoCard)?;
    let span = front::span(body).ok_or(Refused::NoCard)?;
    let fresh = doc_of(&was.level, was.kind, &was.name, file)?;
    let card = Card {
        computer: was.computer,
        ..Card::of(&fresh).ok_or(Refused::NoCard)?
    };
    let fresh_span = front::span(&fresh).ok_or(Refused::NoCard)?;
    Ok(format!(
        "{}{}",
        lined(&body[span], &card),
        &fresh[fresh_span.end..]
    ))
}

fn lined(block: &str, card: &Card) -> String {
    block
        .split_inclusive('\n')
        .map(|line| match keyed(line) {
            true => format!("{}\n", card.line()),
            false => line.to_string(),
        })
        .collect()
}

/// The same document for one computer, or for every one: only its `agent:` line changes.
pub fn meant_for(body: &str, computer: Option<&DeviceId>) -> Option<String> {
    let card = Card {
        computer: computer.cloned(),
        ..Card::of(body)?
    };
    let span = front::span(body)?;
    Some(format!(
        "{}{}",
        lined(&body[span.clone()], &card),
        &body[span.end..]
    ))
}

fn keyed(line: &str) -> bool {
    !line.starts_with([' ', '\t'])
        && line
            .split_once(':')
            .is_some_and(|(key, _)| key.trim().eq_ignore_ascii_case(KEY))
}

pub fn rendered(body: &str) -> Result<Vec<u8>, Refused> {
    let card = Card::of(body).ok_or(Refused::NoCard)?;
    let span = front::span(body).ok_or(Refused::NoCard)?;
    let after = &body[span.end..];
    let inner = match card.code() {
        true => unfenced(after).ok_or(Refused::BrokenFence)?,
        false => after,
    };
    let inner = match card.open_end {
        true => inner.strip_suffix('\n').unwrap_or(inner),
        false => inner,
    };
    let mut text = match card.endings {
        Endings::Lf => inner.replace("\r\n", "\n"),
        Endings::Crlf => inner.replace("\r\n", "\n").replace('\n', "\r\n"),
        Endings::AsIs => inner.to_string(),
    };
    let newline = match card.endings {
        Endings::Crlf => "\r\n",
        _ => "\n",
    };
    if !card.open_end && !text.is_empty() && !text.ends_with('\n') {
        text.push_str(newline);
    }
    if card.bom {
        text.insert(0, BOM);
    }
    Ok(text.into_bytes())
}

fn endings_of(text: &str) -> Endings {
    let lines = text.matches('\n').count();
    let crlf = text.matches("\r\n").count();
    let lone_cr = text.matches('\r').count() != crlf;
    match (crlf, lone_cr) {
        (_, true) => Endings::AsIs,
        (0, false) => Endings::Lf,
        (n, false) if n == lines => Endings::Crlf,
        _ => Endings::AsIs,
    }
}

fn held(card: &Card, text: &str) -> String {
    if !card.code() {
        return text.to_string();
    }
    let longest = text
        .lines()
        .map(|line| {
            line.trim_start()
                .chars()
                .take_while(|one| *one == '`')
                .count()
        })
        .max()
        .unwrap_or(0);
    let fence = "`".repeat(longest.max(2) + 1);
    let info = match extension(&card.name) {
        ext if !ext.is_empty() && ext.bytes().all(|b| b.is_ascii_alphanumeric()) => ext,
        _ => "text".to_string(),
    };
    let closing = match text.is_empty() || text.ends_with('\n') {
        true => "",
        false => "\n",
    };
    format!("{fence}{info}\n{text}{closing}{fence}\n")
}

fn unfenced(after: &str) -> Option<&str> {
    let (first, rest) = after.split_once('\n')?;
    let fence: String = first.chars().take_while(|one| *one == '`').collect();
    if fence.len() < 3 {
        return None;
    }
    let trimmed = rest.trim_end_matches(['\n', '\r', ' ', '\t']);
    let inner = trimmed.strip_suffix(fence.as_str())?;
    if !(inner.is_empty() || inner.ends_with('\n')) {
        return None;
    }
    Some(inner)
}

#[cfg(test)]
#[path = "card_test.rs"]
mod tests;
