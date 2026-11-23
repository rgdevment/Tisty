use std::collections::BTreeMap;
use std::path::Path;

use super::{GLOBAL, Kind, doc_of};

// A file key no document can have, so nothing in the store ever stands for Tisty's own skills.
pub const BUILTIN: &str = "tisty:skill/";
const FILE_AT_MOST: u64 = 256 * 1024;
const FILES_AT_MOST: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bundled {
    pub skill: String,
    pub name: String,
    pub body: String,
}

impl Bundled {
    pub fn file(&self) -> String {
        format!("{BUILTIN}{}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locked {
    pub name: String,
    pub print: String,
}

pub fn builtin(file: &str) -> bool {
    file.starts_with(BUILTIN)
}

pub fn skill_of(name: &str) -> &str {
    name.split_once('/').map_or(name, |(folder, _)| folder)
}

/// The files a release ships, as `file <sha256> <skill>/<file>` lines of `skills.lock`.
pub fn locked(lock: &str) -> Vec<Locked> {
    lock.lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let (Some("file"), Some(print), Some(name), None) =
                (words.next(), words.next(), words.next(), words.next())
            else {
                return None;
            };
            Some(Locked {
                name: name.to_string(),
                print: print.to_ascii_lowercase(),
            })
        })
        .take(FILES_AT_MOST)
        .collect()
}

// Only bytes the lock names go out, and a skill missing one of its files, its licence among them, goes out whole or not at all.
pub fn bundled(dir: &Path, lock: &[Locked]) -> Vec<Bundled> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut skills: BTreeMap<&str, Option<Vec<Bundled>>> = BTreeMap::new();
    for one in lock {
        let skill = skill_of(&one.name);
        let read = read_locked(dir, one, skill);
        let kept = skills.entry(skill).or_insert_with(|| Some(Vec::new()));
        match (kept.as_mut(), read) {
            (Some(all), Some(read)) => all.push(read),
            _ => *kept = None,
        }
    }
    let mut found: Vec<Bundled> = Vec::new();
    for (skill, all) in skills {
        match all {
            Some(all) => found.extend(all),
            None => crate::witness::warn(
                crate::witness::channel::AGENT,
                "one of Tisty's own skills is not as this release ships it, so it is left out",
                &[("skill", crate::witness::Fact::Id(skill.to_string()))],
            ),
        }
    }
    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}

fn read_locked(dir: &Path, one: &Locked, skill: &str) -> Option<Bundled> {
    if skill == one.name || !tisty_markdown::front::agent_name(&one.name) {
        return None;
    }
    let at = dir.join(&one.name);
    let meta = std::fs::symlink_metadata(&at).ok()?;
    if !meta.is_file() || meta.len() > FILE_AT_MOST {
        return None;
    }
    let bytes = std::fs::read(&at).ok()?;
    if crate::attach::printed(&bytes) != one.print {
        return None;
    }
    let body = doc_of(GLOBAL, Kind::Skill, &one.name, &bytes).ok()?;
    Some(Bundled {
        skill: skill.to_string(),
        name: one.name.clone(),
        body,
    })
}

#[cfg(test)]
#[path = "builtin_test.rs"]
mod tests;
