use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tisty_core::agents::{self, Bundled, Ledger};

use super::agents_sync::{Rounded, gated_with, refused, rounded, word};
use crate::wiring::agents::{self as places, Homes, Placed};
use crate::{Answer, Refusal, Session, held};

const SKILLS: &str = "resources/skills";
const LOCK: &str = include_str!("../../skills.lock");

fn skills_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let folder = exe.parent()?;
    Some(match cfg!(target_os = "macos") {
        true => folder.join("../Resources").join(SKILLS),
        false => folder.join(SKILLS),
    })
}

pub fn bundled_here() -> Vec<Bundled> {
    skills_dir()
        .map(|dir| agents::bundled(&dir, &agents::locked(LOCK)))
        .unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSaid {
    pub name: String,
    pub label: String,
    pub about: Option<String>,
    pub files: Vec<String>,
    pub yours: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Landing {
    pub assistant: String,
    pub at: String,
    pub standing: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Leftover {
    pub assistant: String,
    pub at: String,
    pub changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TistySkills {
    pub version: String,
    pub on: bool,
    pub skills: Vec<SkillSaid>,
    pub landing: Vec<Landing>,
    pub plugged: Vec<String>,
    pub leftover: Vec<Leftover>,
}

fn here() -> String {
    tisty_core::machine::here().unwrap_or_default()
}

fn homes() -> Answer<Homes> {
    Homes::here().ok_or_else(|| Refusal::about("cannotWrite", "no home folder"))
}

fn skills_said(bundled: &[Bundled], gate: &agents::Gate) -> Vec<SkillSaid> {
    let mut said: Vec<SkillSaid> = Vec::new();
    for one in bundled {
        let at = match said.iter().position(|was| was.name == one.skill) {
            Some(at) => at,
            None => {
                said.push(SkillSaid {
                    name: one.skill.clone(),
                    label: one.skill.clone(),
                    about: None,
                    files: Vec::new(),
                    yours: gate.skill_named(&one.skill),
                });
                said.len() - 1
            }
        };
        let skill = &mut said[at];
        skill.files.push(one.name.clone());
        if one.name.ends_with("/SKILL.md") {
            skill.label = agents::shown(&one.body).map_or(skill.label.clone(), |seen| seen.label);
            skill.about = agents::own_front(&one.body).about;
        }
    }
    said
}

/// A file found where one of Tisty's skills goes that holds its very bytes is Tisty's, whatever the ledger kept.
/// Without the bytes, its name and size tell it, as the first look reads nothing more.
pub(crate) fn shipped_alike(
    found: &agents::Found,
    bundled: &[Bundled],
    bytes: Option<&[u8]>,
) -> bool {
    let size = || std::fs::metadata(&found.at).map(|meta| meta.len()).ok();
    found.kind == agents::Kind::Skill
        && bundled.iter().any(|one| {
            one.name.eq_ignore_ascii_case(&found.name)
                && agents::rendered(&one.body).is_ok_and(|raw| match bytes {
                    Some(bytes) => bytes == raw.as_slice(),
                    None => size() == Some(raw.len() as u64),
                })
        })
}

// Agents' documents are read only with Agents' yes; without it the gate holds Tisty's skills alone.
fn shipped_gate(session: &Session, ledger: &Ledger, bundled: &[Bundled]) -> agents::Gate {
    if ledger.consented() {
        return gated_with(session, ledger, || bundled.to_vec());
    }
    let mut gate = agents::Gate::default();
    gate.admit_builtins(ledger, bundled);
    gate
}

fn plugged_here(homes: &Homes, ledger: &Ledger) -> BTreeSet<&'static str> {
    match ledger.tisty_skills() || ledger.consented() {
        true => homes.plugged(),
        false => BTreeSet::new(),
    }
}

// Windows and macOS keep `Tisty` and `tisty` in one file.
fn same_file(one: &Path, other: &Path) -> bool {
    match cfg!(any(windows, target_os = "macos")) {
        true => {
            let tidy = |at: &Path| at.to_string_lossy().replace('\\', "/").to_lowercase();
            tidy(one) == tidy(other)
        }
        false => one == other,
    }
}

fn placed_over(placed: &[Placed], at: &Path) -> bool {
    placed.iter().any(|one| same_file(&one.target.at, at))
}

const READ_AT_MOST: u64 = 512 * 1024;

// Only a plain file of a skill's size is opened; a link or anything larger reads as changed and is never followed.
fn changed_now(ledger: &Ledger, at: &Path) -> bool {
    let plain = std::fs::symlink_metadata(at)
        .is_ok_and(|meta| meta.is_file() && meta.len() <= READ_AT_MOST);
    !plain
        || std::fs::read(at).map_or(true, |now| {
            ledger.written(at) != Some(tisty_core::attach::printed(&now).as_str())
        })
}

// Copies Tisty wrote that nothing places now: the plugin arrived, one changed stayed when they went off, or its agent left.
// Without either yes their words are never opened. A path a document places is never one.
fn leftovers(
    homes: &Homes,
    ledger: &Ledger,
    placed: &[Placed],
) -> Vec<(Leftover, PathBuf, &'static str)> {
    let looks = ledger.tisty_skills() || ledger.consented();
    ledger
        .written_files()
        .filter(|(at, _)| agents::shipped_copy(ledger, at) && !placed_over(placed, at))
        .filter(|(at, _)| std::fs::symlink_metadata(at).is_ok())
        .filter_map(|(at, _)| {
            let (assistant, root) = places::owner_any(homes, at)?;
            let one = Leftover {
                assistant: assistant.to_string(),
                at: at.to_string_lossy().into_owned(),
                changed: !looks || changed_now(ledger, at),
            };
            Some((one, root, assistant))
        })
        .collect()
}

// Before either yes nothing of the agents is read: only which agents' folders exist, as Settings › Agents already shows.
pub fn summed(session: &Session, homes: &Homes, ledger: &Ledger) -> TistySkills {
    summed_from(session, homes, ledger, &bundled_here())
}

pub(crate) fn summed_from(
    session: &Session,
    homes: &Homes,
    ledger: &Ledger,
    bundled: &[Bundled],
) -> TistySkills {
    let on = ledger.tisty_skills();
    let mut looked = ledger.clone();
    looked.take_tisty_skills(true);
    let gate = shipped_gate(session, &looked, bundled);
    let plugged = plugged_here(homes, ledger);
    let placed = places::placed(homes, &gate, &looked, &plugged);
    let landing = placed
        .iter()
        .filter(|one| one.target.shipped())
        .map(|one| Landing {
            assistant: one.target.assistant.clone(),
            at: one.target.at.to_string_lossy().into_owned(),
            standing: match on {
                true => word(&agents::standing(ledger, &one.target)),
                false => "write",
            },
        })
        .collect();
    let placing: Vec<Placed> = placed
        .iter()
        .filter(|one| on || !one.target.shipped())
        .cloned()
        .collect();
    TistySkills {
        version: env!("CARGO_PKG_VERSION").to_string(),
        on,
        skills: skills_said(bundled, &gate),
        landing,
        plugged: plugged.iter().map(|id| id.to_string()).collect(),
        leftover: leftovers(homes, ledger, &placing)
            .into_iter()
            .map(|(one, _, _)| one)
            .collect(),
    }
}

/// Tisty's own skills alone, through the same gate and round as Agents and never a document: what opening the app runs.
pub(crate) fn shipped_round(
    session: &Session,
    homes: &Homes,
    ledger: &mut Ledger,
    bundled: &[Bundled],
) -> Rounded {
    if !ledger.tisty_skills() {
        return Rounded::default();
    }
    let gate = shipped_gate(session, ledger, bundled).shipped_only();
    let placed = places::targets(homes, &gate, ledger);
    rounded(session.paths.data(), ledger, &placed)
}

fn skill_folder_of(at: &Path) -> PathBuf {
    at.ancestors()
        .find(|one| {
            one.parent()
                .and_then(Path::file_name)
                .is_some_and(|name| name == "skills")
        })
        .unwrap_or(at)
        .to_path_buf()
}

// Turning them off goes skill by skill, as writing does: one with a copy changed, or that a document places, stays whole in the card.
fn unwritten(session: &Session, homes: &Homes, ledger: &mut Ledger, placed: &[Placed]) {
    let data = session.paths.data();
    let mut skills: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for (at, _) in ledger
        .written_files()
        .filter(|(at, _)| agents::shipped_copy(ledger, at))
    {
        skills
            .entry(skill_folder_of(at))
            .or_default()
            .push(at.to_path_buf());
    }
    for copies in skills.into_values() {
        let kept = copies.iter().any(|at| {
            placed_over(placed, at)
                || (std::fs::symlink_metadata(at).is_ok() && changed_now(ledger, at))
        });
        if kept {
            continue;
        }
        for at in copies {
            let Some((assistant, root)) = places::owner_any(homes, &at) else {
                continue;
            };
            let target = agents::Gate::default().target(ledger, root, at, assistant);
            if let Err(why) = agents::unshipped(data, ledger, &target) {
                tisty_core::witness::warn(
                    tisty_core::witness::channel::AGENT,
                    "a copy of Tisty's own skill could not be taken away, so its card lists it",
                    &[("why", tisty_core::witness::Fact::Why(why.to_string()))],
                );
            }
        }
    }
}

// What documents place, read only with Agents' yes.
fn documents_placed(session: &Session, homes: &Homes, ledger: &Ledger) -> Vec<Placed> {
    match ledger.consented() {
        true => places::placed(
            homes,
            &gated_with(session, ledger, Vec::new),
            ledger,
            &BTreeSet::new(),
        ),
        false => Vec::new(),
    }
}

pub fn taken(session: &Session, homes: &Homes, here: &str, on: bool) -> Answer<TistySkills> {
    taken_from(session, homes, here, on, &bundled_here())
}

pub(crate) fn taken_from(
    session: &Session,
    homes: &Homes,
    here: &str,
    on: bool,
    bundled: &[Bundled],
) -> Answer<TistySkills> {
    let data = session.paths.data();
    let mut ledger = Ledger::open(data, here);
    if !ledger.named() {
        return Err(refused(agents::Refused::NoConsent));
    }
    ledger.take_tisty_skills(on);
    ledger.keep(data).map_err(refused)?;
    match on {
        true => drop(shipped_round(session, homes, &mut ledger, bundled)),
        false => {
            let placed = documents_placed(session, homes, &ledger);
            unwritten(session, homes, &mut ledger, &placed);
        }
    }
    Ok(summed_from(session, homes, &ledger, bundled))
}

// The path comes from the ledger, never from the window, and only a copy still left over goes, with a backup.
pub fn left_out(
    session: &Session,
    homes: &Homes,
    here: &str,
    at: &str,
    bundled: &[Bundled],
) -> Answer<TistySkills> {
    let data = session.paths.data();
    let mut ledger = Ledger::open(data, here);
    let gate = shipped_gate(session, &ledger, bundled);
    let placed = places::placed(homes, &gate, &ledger, &plugged_here(homes, &ledger));
    let Some((_, root, assistant)) = leftovers(homes, &ledger, &placed)
        .into_iter()
        .find(|(one, _, _)| one.at == at)
    else {
        return Err(refused(agents::Refused::Held));
    };
    let target = agents::Gate::default().target(&ledger, root, PathBuf::from(at), assistant);
    agents::removed(data, &mut ledger, &target).map_err(refused)?;
    Ok(summed_from(session, homes, &ledger, bundled))
}

#[tauri::command(async)]
pub fn agents_tisty_skills(session: tauri::State<'_, Mutex<Session>>) -> Answer<TistySkills> {
    let homes = homes()?;
    let session = held(&session);
    let ledger = Ledger::open(session.paths.data(), &here());
    Ok(summed(&session, &homes, &ledger))
}

#[tauri::command(async)]
pub fn agents_tisty_round(session: tauri::State<'_, Mutex<Session>>) -> Answer<TistySkills> {
    let homes = homes()?;
    let session = held(&session);
    let mut ledger = Ledger::open(session.paths.data(), &here());
    shipped_round(&session, &homes, &mut ledger, &bundled_here());
    Ok(summed(&session, &homes, &ledger))
}

#[tauri::command(async)]
pub fn agents_tisty_skills_leftover(
    session: tauri::State<'_, Mutex<Session>>,
    at: String,
) -> Answer<TistySkills> {
    let homes = homes()?;
    let session = held(&session);
    left_out(&session, &homes, &here(), &at, &bundled_here())
}

#[tauri::command(async)]
pub fn agents_tisty_skills_take(
    session: tauri::State<'_, Mutex<Session>>,
    on: bool,
) -> Answer<TistySkills> {
    let homes = homes()?;
    let session = held(&session);
    taken(&session, &homes, &here(), on)
}

#[cfg(test)]
#[path = "tisty_skills_test.rs"]
mod tests;
