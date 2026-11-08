use tisty_core::{DeviceId, Paths, State};
use tisty_share_proto::identity::{Certificate, Person};

use crate::signer;

pub fn certified_by(certificate: &Certificate, root: &str) -> Option<Person> {
    let wanted = Person::read(root)?;
    (Person::read(&certificate.root) == Some(wanted)).then_some(wanted)
}

// Only a machine key certified by that root proves it, never an alias or a root the log merely names.
pub fn signs_for(paths: &Paths, state: &State, device: &DeviceId, root: &str) -> Option<Person> {
    signer(paths, state, device).and_then(|(_, certificate)| certified_by(&certificate, root))
}
