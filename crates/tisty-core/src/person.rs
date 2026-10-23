use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use jiff::Timestamp;
use tisty_share_proto::VerifyingKey;
use tisty_share_proto::identity::{Certificate, Person, Revocation, ShareKey, Vouched, share_key};
use tisty_share_proto::seed::Seed;

use crate::event::{DeviceId, Event, Op};
use crate::state::State;

pub const KEEP: &str = ".share-key";

/// What the log says of the person: each machine's share key and what the root said about it.
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Persons {
    pub keys: BTreeMap<DeviceId, String>,
    pub certificates: Vec<Certificate>,
    pub revocations: Vec<Revocation>,
}

impl Persons {
    pub(crate) fn took(&mut self, event: &Event) {
        match &event.op {
            // The newest key a machine names stands: only that machine writes its history.
            Op::ShareKey { p } if share_key(p).is_some() => {
                self.keys.insert(event.device.clone(), p.clone());
            }
            Op::ShareCertified { c } => self.certificates.push(c.clone()),
            Op::ShareRevoked { r } => self.revocations.push(r.clone()),
            _ => {}
        }
    }

    fn vouches(&self) -> impl Iterator<Item = Vouched> + '_ {
        self.certificates
            .iter()
            .filter_map(|one| one.vouches().ok())
    }

    fn taken_back(&self, vouched: &Vouched) -> bool {
        self.revocations.iter().any(|one| one.takes_back(vouched))
    }

    /// The certificate that holds for a machine's key now; the newest when the root said it twice.
    pub fn vouched(&self, device: &DeviceId) -> Option<Vouched> {
        let key = share_key(self.keys.get(device)?)?;
        self.vouches()
            .filter(|one| one.share == key && !self.taken_back(one))
            .max_by_key(|one| one.since)
    }

    pub fn person(&self, device: &DeviceId) -> Option<Person> {
        self.vouched(device).map(|one| one.person)
    }

    /// Machines whose key this person never vouched for. A key taken back is not waiting: it stays out.
    pub fn waiting(
        &self,
        root: &Person,
        machines: &BTreeSet<DeviceId>,
    ) -> Vec<(DeviceId, VerifyingKey)> {
        machines
            .iter()
            .filter_map(|device| {
                let key = share_key(self.keys.get(device)?)?;
                let known = self
                    .vouches()
                    .any(|one| one.person == *root && one.share == key);
                (!known).then(|| (device.clone(), key))
            })
            .collect()
    }

    fn key_taken_back(&self, key: &VerifyingKey) -> bool {
        self.vouches()
            .any(|one| one.share == *key && self.taken_back(&one))
    }
}

/// The machines of the log, the ones a person sits at: assistants never get a key of their own.
pub fn machines(state: &State) -> BTreeSet<DeviceId> {
    state
        .devices
        .iter()
        .filter(|one| !state.agents.contains(*one) && !state.assistants.contains(*one))
        .cloned()
        .collect()
}

/// The first steps of a person on the machine where the seed was born: its key, vouched for.
pub fn begun(
    paths: &crate::Paths,
    device: &DeviceId,
    seed: &Seed,
    now: Timestamp,
) -> Option<Vec<Op>> {
    let key = mine(paths, device)?;
    let certificate = seed.root().certify(&key.public(), now);
    Some(vec![
        Op::ShareKey { p: key.shown() },
        Op::ShareCertified { c: certificate },
    ])
}

/// Once the log names a person, a machine says the key it would sign with, for the root to vouch for.
/// A key taken back is never used again: the machine makes another.
pub fn introduced(paths: &crate::Paths, device: &DeviceId, persons: &Persons) -> Option<Op> {
    if persons.certificates.is_empty() {
        return None;
    }
    let published = persons.keys.get(device).and_then(|said| share_key(said));
    let key = match published {
        Some(key) if persons.key_taken_back(&key) => fresh(paths, device)?,
        _ => mine(paths, device)?,
    };
    (published != Some(key.public())).then(|| Op::ShareKey { p: key.shown() })
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
        .map(|(_, key)| Op::ShareCertified {
            c: root.certify(&key, now),
        })
        .collect()
}

/// None when that machine has no key this person vouched for.
pub fn revoked(seed: &Seed, persons: &Persons, device: &DeviceId, now: Timestamp) -> Option<Op> {
    let root = seed.root();
    let vouched = persons
        .vouched(device)
        .filter(|one| one.person == root.person())?;
    Some(Op::ShareRevoked {
        r: root.revoke(&vouched.share, now),
    })
}

pub fn kept_at(paths: &crate::Paths, device: &DeviceId) -> Option<PathBuf> {
    crate::store::is_device_name(&device.0)
        .then(|| paths.private().join(format!("{}{KEEP}", device.0)))
}

pub fn mine(paths: &crate::Paths, device: &DeviceId) -> Option<ShareKey> {
    use crate::store::identity::{Kept, kept, minted};

    let at = kept_at(paths, device)?;
    let one = match kept(
        paths,
        &at,
        "what was kept as this machine's share key is not one",
    ) {
        Kept::Good(one) => Some(one),
        Kept::Gone => minted(paths, &at),
        Kept::Blocked => None,
    }?;
    Some(ShareKey::from_bytes(&one))
}

fn fresh(paths: &crate::Paths, device: &DeviceId) -> Option<ShareKey> {
    let at = kept_at(paths, device)?;
    match std::fs::remove_file(&at) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return None,
    }
    mine(paths, device)
}

#[cfg(test)]
#[path = "person_test.rs"]
mod tests;
