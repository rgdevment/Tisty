use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use tisty_core::agents::{self, GLOBAL, Gate, Kind, Ledger};

use super::agents_sync::Told;
use super::tisty_skills::changed_now;
use crate::wiring::agents::{self as places, Homes};

const SETTLED: [&str; 2] = ["wrote", "upToDate"];

fn global_skill(gate: &Gate, files: &BTreeSet<String>) -> bool {
    !files.is_empty()
        && files.iter().all(|file| {
            gate.card_here(file)
                .is_some_and(|card| card.level == GLOBAL && card.kind == Kind::Skill)
        })
}

fn shipped(files: &BTreeSet<String>) -> bool {
    !files.is_empty() && files.iter().all(|file| agents::builtin(file))
}

fn intact(ledger: &Ledger, at: &Path) -> bool {
    std::fs::symlink_metadata(at).is_err() || !changed_now(ledger, at)
}

// The next round takes it away on its own, so it is never asked about; one changed since stays for the person.
pub(crate) fn goes_alone(
    homes: &Homes,
    gate: &Gate,
    ledger: &Ledger,
    at: &Path,
    files: &BTreeSet<String>,
) -> bool {
    places::cursor_twin(homes, at).is_some() && global_skill(gate, files) && intact(ledger, at)
}

struct Written {
    at: PathBuf,
    shipped: bool,
}

// A skill goes whole, and only once its twin in the shared folder stands as Tisty wrote it in this very round.
pub(crate) fn retired(
    data: &Path,
    homes: &Homes,
    gate: &Gate,
    ledger: &mut Ledger,
    placing: &BTreeSet<String>,
    told: &[Told],
) {
    let Some(root) = homes.root("cursor") else {
        return;
    };
    let settled: BTreeSet<String> = told
        .iter()
        .filter(|one| SETTLED.contains(&one.standing))
        .map(|one| agents::path_key(Path::new(&one.at)))
        .collect();
    let mut skills: BTreeMap<PathBuf, Vec<Written>> = BTreeMap::new();
    let mut kept: BTreeSet<PathBuf> = BTreeSet::new();
    for (at, files) in ledger.written_files() {
        let Some((folder, twin)) = places::cursor_twin(homes, at) else {
            continue;
        };
        let shipped = shipped(files);
        let goes = (shipped || (ledger.syncs("cursor") && global_skill(gate, files)))
            && !placing.contains(&agents::path_key(at))
            && settled.contains(&agents::path_key(&twin))
            && intact(ledger, at);
        if !goes {
            kept.insert(folder.clone());
        }
        skills.entry(folder).or_default().push(Written {
            at: at.to_path_buf(),
            shipped,
        });
    }
    for (_, copies) in skills
        .into_iter()
        .filter(|(folder, _)| !kept.contains(folder))
    {
        for one in copies {
            let target = Gate::default().target(ledger, root.clone(), one.at, "cursor");
            let gone = match one.shipped {
                true => agents::unshipped(data, ledger, &target),
                false => agents::removed_as_written(data, ledger, &target),
            };
            if let Err(why) = gone {
                tisty_core::witness::warn(
                    tisty_core::witness::channel::AGENT,
                    "an old copy of a skill in Cursor's own folder could not be taken away",
                    &[("why", tisty_core::witness::Fact::Why(why.to_string()))],
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "cursor_copies_test.rs"]
mod tests;
