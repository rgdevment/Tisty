use std::path::Path;

use crate::event::DeviceId;

const KEPT: &str = ".keys-confirmed";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Confirmed {
    pub key: String,
    pub when: u64,
    /// True when nobody here compared it: the key a machine already held here published itself.
    pub carried: bool,
    /// The machine whose word this was taken on: an agent's host, already confirmed here.
    pub host: Option<DeviceId>,
    /// The key this one was rotated from, so what the person compared stays told apart.
    pub was: Option<String>,
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
    let mut all = std::collections::BTreeMap::new();
    for (who, one) in lines(data).iter().filter_map(|line| read_line(line)) {
        all.entry(who).or_insert(one);
    }
    all
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

/// The key a confirmed one rotated to, sealed by it: replaced only while that key still stands.
pub fn rotated(data: &Path, who: &DeviceId, now: &str, was: &str) -> bool {
    let stands = confirmed(data, who).is_some_and(|stood| stood.key == was);
    if !stands || crate::signing::read(now).is_none() {
        return false;
    }
    let mut kept = vec![format!(
        "{}\t{now}\t{}\trotated:{was}",
        who.0,
        crate::lately::now()
    )];
    kept.extend(lines(data));
    crate::store::write_atomic(&data.join(KEPT), kept.join("\n").as_bytes()).is_ok()
}

/// An agent's key taken on its host's word, the host being confirmed here already.
pub fn through(data: &Path, who: &DeviceId, said: &str, host: &DeviceId) -> bool {
    crate::store::is_device_name(&host.0) && kept(data, who, said, &format!("\thost:{}", host.0))
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
    let how = said.next().map(str::trim).unwrap_or_default();
    let carried = how == "carried";
    let host = how
        .strip_prefix("host:")
        .filter(|host| crate::store::is_device_name(host))
        .map(|host| DeviceId(host.to_string()));
    let was = how
        .strip_prefix("rotated:")
        .filter(|was| crate::signing::read(was).is_some())
        .map(str::to_string);
    (crate::store::is_device_name(whose) && crate::signing::read(key).is_some()).then(|| {
        (
            DeviceId(whose.to_string()),
            Confirmed {
                key: key.to_string(),
                when,
                carried,
                host,
                was,
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
