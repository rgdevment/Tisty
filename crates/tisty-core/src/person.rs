use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use jiff::Timestamp;
use tisty_share_proto::VerifyingKey;
use tisty_share_proto::identity::{Certificate, Person, Revocation, ShareKey, Vouched, share_key};
use tisty_share_proto::seed::Seed;

use crate::event::{DeviceId, Event, Op};
use crate::state::State;

pub const KEEP: &str = ".share-key";

/// Only what holds is kept, so a forged line never reaches the cache nor any answer.
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Persons {
    /// Every key a machine named, the one it signs with now last.
    pub keys: BTreeMap<DeviceId, Vec<String>>,
    pub certificates: Vec<Certificate>,
    pub revocations: Vec<Revocation>,
}

impl Persons {
    /// Assistants never speak for the person here, whatever they sign.
    pub(crate) fn took(&mut self, event: &Event, assistants: &BTreeSet<DeviceId>) {
        if assistants.contains(&event.device) {
            return;
        }
        match &event.op {
            Op::ShareKey { p }
                if share_key(p).is_some() && !self.named_by_another(&event.device, p) =>
            {
                let said = self.keys.entry(event.device.clone()).or_default();
                said.retain(|one| one != p);
                said.push(p.clone());
            }
            Op::ShareCertified { c } if c.vouches().is_ok() && !self.certificates.contains(c) => {
                self.certificates.push(c.clone());
            }
            Op::ShareRevoked { r } if r.holds().is_ok() && !self.revocations.contains(r) => {
                self.revocations.push(r.clone());
            }
            _ => {}
        }
    }

    // One key answers for one machine, or taking one back would take back the other.
    fn named_by_another(&self, device: &DeviceId, said: &str) -> bool {
        self.keys
            .iter()
            .any(|(one, keys)| one != device && keys.iter().any(|key| key == said))
    }

    fn current(&self, device: &DeviceId) -> Option<VerifyingKey> {
        share_key(self.keys.get(device)?.last()?)
    }

    fn vouches(&self) -> impl Iterator<Item = Vouched> + '_ {
        self.certificates
            .iter()
            .filter_map(|one| one.vouches().ok())
    }

    fn taken_back(&self, vouched: &Vouched) -> bool {
        self.revocations.iter().any(|one| one.takes_back(vouched))
    }

    /// The person whose word holds for a machine's key now; the newest when two roots vouched.
    pub fn person(&self, device: &DeviceId) -> Option<Person> {
        let key = self.current(device)?;
        self.vouches()
            .filter(|one| one.share == key && !self.taken_back(one))
            .max_by_key(|one| one.since)
            .map(|one| one.person)
    }

    pub fn vouched_by(&self, device: &DeviceId, root: &Person) -> Option<Vouched> {
        let key = self.current(device)?;
        self.vouches()
            .filter(|one| one.share == key && one.person == *root && !self.taken_back(one))
            .max_by_key(|one| one.since)
    }

    fn named_by(&self, device: &DeviceId) -> Vec<VerifyingKey> {
        self.keys
            .get(device)
            .map(|keys| keys.iter().filter_map(|one| share_key(one)).collect())
            .unwrap_or_default()
    }

    fn key_taken_back_by(&self, key: &VerifyingKey, root: &Person) -> bool {
        self.revocations
            .iter()
            .filter_map(|one| one.holds().ok())
            .any(|(person, share)| person == *root && share == *key)
    }

    /// The root's last word on a machine counts: taken back, it stays out until the phrase is typed there.
    fn taken_back_by(&self, device: &DeviceId, root: &Person) -> bool {
        let keys = self.named_by(device);
        let vouched = self
            .vouches()
            .filter(|one| one.person == *root && keys.contains(&one.share))
            .map(|one| one.since)
            .max();
        let taken = self
            .revocations
            .iter()
            .filter(|one| {
                one.holds()
                    .is_ok_and(|(person, share)| person == *root && keys.contains(&share))
            })
            .map(|one| one.at)
            .max();
        match (vouched, taken) {
            (_, None) => false,
            (None, Some(_)) => true,
            (Some(vouched), Some(taken)) => taken >= vouched,
        }
    }

    pub fn waiting(&self, root: &Person, machines: &BTreeSet<DeviceId>) -> Vec<VerifyingKey> {
        let mut waiting: Vec<VerifyingKey> = machines
            .iter()
            .filter(|device| !self.taken_back_by(device, root))
            .filter_map(|device| self.current(device))
            .filter(|key| {
                !self
                    .vouches()
                    .any(|one| one.person == *root && one.share == *key)
            })
            .collect();
        waiting.sort_by_key(|key| *key.as_bytes());
        waiting.dedup();
        waiting
    }
}

pub fn machines(state: &State) -> BTreeSet<DeviceId> {
    state
        .devices
        .iter()
        .filter(|one| !state.assistants.contains(*one))
        .cloned()
        .collect()
}

/// Typing the phrase on a machine is the person's own word for it, even one they took back before.
pub fn begun(
    paths: &crate::Paths,
    device: &DeviceId,
    seed: &Seed,
    persons: &Persons,
    now: Timestamp,
) -> Option<Vec<Op>> {
    let root = seed.root();
    let mut key = mine(paths, device)?;
    if persons.key_taken_back_by(&key.public(), &root.person()) {
        key = replaced(paths, device, &key)?;
    }
    let mut ops = Vec::new();
    if persons.current(device) != Some(key.public()) {
        ops.push(Op::ShareKey { p: key.shown() });
    }
    let vouched = persons
        .vouched_by(device, &root.person())
        .is_some_and(|one| one.share == key.public());
    if !vouched {
        ops.push(Op::ShareCertified {
            c: root.certify(&key.public(), now),
        });
    }
    Some(ops)
}

/// A machine says its key only once the log names a person, so nobody else ever makes one.
pub fn introduced(paths: &crate::Paths, device: &DeviceId, persons: &Persons) -> Option<Op> {
    if persons.certificates.is_empty() {
        return None;
    }
    let key = mine(paths, device)?;
    let shown = key.shown();
    if persons.named_by_another(device, &shown) {
        crate::witness::warn(
            crate::witness::channel::STORE,
            "another machine named this machine's share key first, so it is not said again",
            &[("by", crate::witness::Fact::Id(device.0.clone()))],
        );
        return None;
    }
    (persons.current(device) != Some(key.public())).then_some(Op::ShareKey { p: shown })
}

pub fn certified(
    seed: &Seed,
    persons: &Persons,
    machines: &BTreeSet<DeviceId>,
    now: Timestamp,
) -> Vec<Op> {
    let root = seed.root();
    persons
        .waiting(&root.person(), machines)
        .into_iter()
        .map(|key| Op::ShareCertified {
            c: root.certify(&key, now),
        })
        .collect()
}

/// Every key the machine ever had that this root vouched for, so an old copy of one signs nothing.
pub fn revoked(seed: &Seed, persons: &Persons, device: &DeviceId, now: Timestamp) -> Vec<Op> {
    let root = seed.root();
    let person = root.person();
    persons
        .named_by(device)
        .into_iter()
        .filter(|key| {
            persons
                .vouches()
                .any(|one| one.person == person && one.share == *key)
        })
        .filter(|key| !persons.key_taken_back_by(key, &person))
        .map(|key| Op::ShareRevoked {
            r: root.revoke(&key, now),
        })
        .collect()
}

/// A machine removed from the store answers for nobody, whatever its certificate still says.
pub fn person_of(state: &State, device: &DeviceId) -> Option<Person> {
    state
        .devices
        .contains(device)
        .then(|| state.persons.person(device))
        .flatten()
}

pub fn kept_at(paths: &crate::Paths, device: &DeviceId) -> Option<PathBuf> {
    crate::store::identity::device_file(paths, device, KEEP)
}

pub fn mine(paths: &crate::Paths, device: &DeviceId) -> Option<ShareKey> {
    let at = kept_at(paths, device)?;
    let one = crate::store::identity::kept_or_minted(
        paths,
        &at,
        "what was kept as this machine's share key is not one",
    )?;
    Some(ShareKey::from_bytes(&one))
}

/// Only the key that was taken back is thrown away, never one minted after it.
fn replaced(paths: &crate::Paths, device: &DeviceId, old: &ShareKey) -> Option<ShareKey> {
    let at = kept_at(paths, device)?;
    if mine(paths, device)?.public() == old.public()
        && let Err(e) = std::fs::remove_file(&at)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        crate::witness::warn(
            crate::witness::channel::STORE,
            "the share key taken back could not be thrown away, so no new one was made",
            &[("why", crate::witness::Fact::Why(e.to_string()))],
        );
        return None;
    }
    mine(paths, device)
}

#[cfg(test)]
#[path = "person_test.rs"]
mod tests;
