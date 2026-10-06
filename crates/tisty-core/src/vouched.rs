use std::path::Path;

use crate::event::DeviceId;

const KEPT: &str = ".keys-confirmed";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Confirmed {
    pub key: String,
    pub when: u64,
    /// True when nobody here compared it: the key a machine already held here published itself.
    pub carried: bool,
}

/// What a machine published is a claim; this is what the person at this machine accepted. The
/// two are kept apart because the log decides the first and only a person can decide the second.
pub fn confirmed(data: &Path, who: &DeviceId) -> Option<Confirmed> {
    lines(data)
        .iter()
        .filter_map(|line| read_line(line))
        .find(|(whose, _)| whose == who)
        .map(|(_, one)| one)
}

pub fn all_confirmed(data: &Path) -> std::collections::BTreeMap<DeviceId, Confirmed> {
    lines(data)
        .iter()
        .filter_map(|line| read_line(line))
        .collect()
}

/// Kept once and never moved: a key confirmed is the one that machine answers for from then on,
/// and letting a later write replace it would give back the door this exists to close.
pub fn confirm(data: &Path, who: &DeviceId, said: &str) -> bool {
    kept(data, who, said, "")
}

/// The first key of a machine this store already held from before signing, taken without asking.
pub fn carried(data: &Path, who: &DeviceId, said: &str) -> bool {
    kept(data, who, said, "\tcarried")
}

fn kept(data: &Path, who: &DeviceId, said: &str, how: &str) -> bool {
    if crate::signing::read(said).is_none() || !crate::store::is_device_name(&who.0) {
        return false;
    }
    if let Some(stood) = confirmed(data, who) {
        return stood.key == said;
    }
    let mut kept = vec![format!("{}\t{said}\t{}{how}", who.0, crate::lately::now())];
    kept.extend(lines(data));
    crate::store::write_atomic(&data.join(KEPT), kept.join("\n").as_bytes()).is_ok()
}

pub fn confirm_each(data: &Path, keys: &std::collections::BTreeMap<DeviceId, String>) -> usize {
    keys.iter()
        .filter(|(who, said)| confirm(data, who, said))
        .count()
}

/// A line that does not read whole is no confirmation, so a key nobody can parse never stands in
/// for one a person answered for.
fn read_line(line: &str) -> Option<(DeviceId, Confirmed)> {
    let mut said = line.split('\t');
    let whose = said.next()?;
    let key = said.next()?.trim();
    let when = said.next()?.trim().parse().ok()?;
    let carried = said.next().is_some_and(|how| how.trim() == "carried");
    (crate::store::is_device_name(whose) && crate::signing::read(key).is_some()).then(|| {
        (
            DeviceId(whose.to_string()),
            Confirmed {
                key: key.to_string(),
                when,
                carried,
            },
        )
    })
}

fn lines(data: &Path) -> Vec<String> {
    std::fs::read_to_string(data.join(KEPT))
        .map(|said| {
            said.lines()
                .map(|one| one.trim_end_matches('\r').to_string())
                .filter(|one| !one.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "vouched_test.rs"]
mod tests;
