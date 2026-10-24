use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use jiff::Timestamp;
use tisty_share_proto::VerifyingKey;
use tisty_share_proto::identity::{Certificate, Person, Revocation, ShareKey, Vouched, public_key};
use tisty_share_proto::seed::Seed;

use crate::event::{DeviceId, Event, Op};
use crate::state::State;

pub const KEEP: &str = ".share-key";

/// Only what holds is kept, so a forged line never reaches the cache nor any answer.
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Persons {
    /// Every key a machine named, in the order it first named them: the one it signs with is last.
    pub keys: BTreeMap<DeviceId, Vec<String>>,
    pub certificates: Vec<Certificate>,
    pub revocations: Vec<Revocation>,
    /// The keys of machines the person took back, apart from keys a machine merely stopped using.
    #[serde(default)]
    pub taken: BTreeSet<String>,
}

impl Persons {
    /// Assistants and machines the log never seated never speak for the person here.
    pub(crate) fn took(
        &mut self,
        event: &Event,
        assistants: &BTreeSet<DeviceId>,
        devices: &BTreeSet<DeviceId>,
    ) {
        let seated = !assistants.contains(&event.device) && devices.contains(&event.device);
        match &event.op {
            // A key said again is a line replayed, never a step back to it.
            Op::ShareKey { p }
                if seated
                    && public_key(p).is_some()
                    && !self.named_by_another(&event.device, p) =>
            {
                let said = self.keys.entry(event.device.clone()).or_default();
                if !said.contains(p) {
                    said.push(p.clone());
                }
            }
            // The root's signature is the authority here, whoever carried the line.
            Op::ShareCertified { c } if c.vouches().is_ok() && !self.certificates.contains(c) => {
                self.certificates.push(c.clone());
            }
            Op::ShareRevoked { r, machine } if r.holds().is_ok() => {
                if *machine && !assistants.contains(&event.device) {
                    self.taken.insert(r.share.clone());
                }
                if !self.revocations.contains(r) {
                    self.revocations.push(r.clone());
                }
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
        public_key(self.keys.get(device)?.last()?)
    }

    fn named_by(&self, device: &DeviceId) -> Vec<VerifyingKey> {
        self.keys
            .get(device)
            .map(|keys| keys.iter().filter_map(|one| public_key(one)).collect())
            .unwrap_or_default()
    }

    // `took` checked every signature once; asking again on each question would only cost time.
    fn vouches(&self) -> impl Iterator<Item = Vouched> + '_ {
        self.certificates.iter().filter_map(Certificate::unchecked)
    }

    fn vouched_for(&self, key: &VerifyingKey, root: &Person) -> bool {
        self.vouches()
            .any(|one| one.person == *root && one.share == *key)
    }

    fn key_taken_back_by(&self, key: &VerifyingKey, root: &Person) -> bool {
        self.revocations
            .iter()
            .filter_map(Revocation::unchecked)
            .any(|(person, share)| person == *root && share == *key)
    }

    fn newest(&self, device: &DeviceId, root: Option<&Person>) -> Option<Vouched> {
        let key = self.current(device)?;
        self.vouches()
            .filter(|one| one.share == key && root.is_none_or(|root| one.person == *root))
            .filter(|one| !self.key_taken_back_by(&one.share, &one.person))
            .max_by_key(|one| one.since)
    }

    /// Two roots on one key only happen when a person began again; the later signature is the one asked.
    pub fn person(&self, device: &DeviceId) -> Option<Person> {
        self.newest(device, None).map(|one| one.person)
    }

    pub fn vouched_by(&self, device: &DeviceId, root: &Person) -> Option<Vouched> {
        self.newest(device, Some(root))
    }

    /// Clocks of two machines never agree, so no date decides: only the phrase typed there lets it back.
    fn taken_back_by(&self, device: &DeviceId, root: &Person) -> bool {
        self.machine_keys_taken_back(device, root).next().is_some()
            && self.vouched_by(device, root).is_none()
    }

    fn machine_keys_taken_back<'a>(
        &'a self,
        device: &DeviceId,
        root: &'a Person,
    ) -> impl Iterator<Item = VerifyingKey> + 'a {
        self.keys
            .get(device)
            .into_iter()
            .flatten()
            .filter(|said| self.taken.contains(*said))
            .filter_map(|said| public_key(said))
            .filter(move |key| self.key_taken_back_by(key, root))
    }

    /// The log names a person once a certificate holds that nobody took back.
    pub fn names_a_person(&self) -> bool {
        self.vouches()
            .any(|one| !self.key_taken_back_by(&one.share, &one.person))
    }

    /// Whether a machine holding the seed has anything to do: a key nobody vouched for, or a machine gone.
    pub fn awaits_the_seed(&self, machines: &BTreeSet<DeviceId>) -> bool {
        self.keys
            .keys()
            .any(|device| match machines.contains(device) {
                true => self
                    .current(device)
                    .is_some_and(|key| !self.vouches().any(|one| one.share == key)),
                false => self.person(device).is_some(),
            })
    }

    /// Whether this root is a person the log names, so a seed for another one never speaks here.
    pub fn names(&self, root: &Person) -> bool {
        self.vouches()
            .any(|one| one.person == *root && !self.key_taken_back_by(&one.share, &one.person))
    }

    pub fn waiting(
        &self,
        root: &Person,
        machines: &BTreeSet<DeviceId>,
    ) -> Vec<(DeviceId, VerifyingKey)> {
        machines
            .iter()
            .filter(|device| !self.taken_back_by(device, root))
            .filter_map(|device| Some((device.clone(), self.current(device)?)))
            .filter(|(_, key)| !self.vouched_for(key, root))
            .collect()
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

/// A machine removed from the store, or an assistant, answers for nobody whatever a certificate says.
pub fn person_of(state: &State, device: &DeviceId) -> Option<Person> {
    machines(state)
        .contains(device)
        .then(|| state.persons.person(device))
        .flatten()
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
    let person = root.person();
    let key = usable(paths, device, persons, Some(&person))?;
    let mut ops: Vec<Op> = announced(&key, device, persons).into_iter().collect();
    let vouched = persons
        .vouched_by(device, &person)
        .is_some_and(|one| one.share == key.public());
    if !vouched {
        ops.push(Op::ShareCertified {
            c: root.certify(&key.public(), now),
        });
        ops.extend(retired(seed, persons, device, &key.public(), now));
    }
    Some(ops)
}

/// The key a machine can still say: one taken back, named by another or left behind is never said again.
fn usable(
    paths: &crate::Paths,
    device: &DeviceId,
    persons: &Persons,
    root: Option<&Person>,
) -> Option<ShareKey> {
    let key = mine(paths, device)?;
    let shown = key.shown();
    let left_behind = persons
        .keys
        .get(device)
        .is_some_and(|said| said.contains(&shown) && said.last() != Some(&shown));
    let taken = root.is_some_and(|root| persons.key_taken_back_by(&key.public(), root));
    match taken || left_behind || persons.named_by_another(device, &shown) {
        true => replaced(paths, device),
        false => Some(key),
    }
}

/// The keys a machine left behind that this root vouched for, so an old copy of one signs nothing.
fn retired<'a>(
    seed: &'a Seed,
    persons: &'a Persons,
    device: &DeviceId,
    key: &'a VerifyingKey,
    now: Timestamp,
) -> impl Iterator<Item = Op> + 'a {
    let root = seed.root();
    let person = root.person();
    persons
        .named_by(device)
        .into_iter()
        .filter(move |old| old != key && persons.vouched_for(old, &person))
        .filter(move |old| !persons.key_taken_back_by(old, &person))
        .map(move |old| Op::ShareRevoked {
            r: root.revoke(&old, now),
            machine: false,
        })
}

/// A machine says its key only once the log names a person, so nobody else ever makes one.
pub fn introduced(paths: &crate::Paths, device: &DeviceId, persons: &Persons) -> Option<Op> {
    if !persons.names_a_person() {
        return None;
    }
    announced(&usable(paths, device, persons, None)?, device, persons)
}

fn announced(key: &ShareKey, device: &DeviceId, persons: &Persons) -> Option<Op> {
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

/// A machine's new key is vouched for and the ones it left behind are retired, so an old copy signs nothing.
pub fn certified(
    seed: &Seed,
    persons: &Persons,
    machines: &BTreeSet<DeviceId>,
    now: Timestamp,
) -> Vec<Op> {
    let root = seed.root();
    let mut ops = Vec::new();
    for (device, key) in persons.waiting(&root.person(), machines) {
        ops.push(Op::ShareCertified {
            c: root.certify(&key, now),
        });
        ops.extend(retired(seed, persons, &device, &key, now));
    }
    ops
}

/// Machines removed from the list from a machine without the seed are taken back by the one that has it.
pub fn left(
    seed: &Seed,
    persons: &Persons,
    machines: &BTreeSet<DeviceId>,
    now: Timestamp,
) -> Vec<Op> {
    let person = seed.root().person();
    persons
        .keys
        .keys()
        .filter(|device| !machines.contains(*device))
        .filter(|device| {
            persons
                .named_by(device)
                .iter()
                .any(|key| persons.vouched_for(key, &person))
        })
        .flat_map(|device| revoked(seed, persons, device, now))
        .collect()
}

/// Every key the machine ever named, vouched for yet or not, so it cannot slip in before the root speaks.
pub fn revoked(seed: &Seed, persons: &Persons, device: &DeviceId, now: Timestamp) -> Vec<Op> {
    let root = seed.root();
    let person = root.person();
    let gone: Vec<VerifyingKey> = persons.machine_keys_taken_back(device, &person).collect();
    persons
        .named_by(device)
        .into_iter()
        .filter(|key| !gone.contains(key))
        .map(|key| Op::ShareRevoked {
            r: root.revoke(&key, now),
            machine: true,
        })
        .collect()
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

/// Called only right after reading the key that cannot be used, so the file holds that one.
fn replaced(paths: &crate::Paths, device: &DeviceId) -> Option<ShareKey> {
    let at = kept_at(paths, device)?;
    if let Err(e) = std::fs::remove_file(&at)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        crate::witness::warn(
            crate::witness::channel::STORE,
            "a share key that cannot be used could not be thrown away, so no new one was made",
            &[("why", crate::witness::Fact::Why(e.to_string()))],
        );
        return None;
    }
    mine(paths, device)
}

#[cfg(test)]
#[path = "person_test.rs"]
mod tests;
