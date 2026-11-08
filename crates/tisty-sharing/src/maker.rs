use tisty_core::{DeviceId, Paths, State};
use tisty_share_proto::identity::{Certificate, Person};

use crate::signer;

pub fn certified_by(certificate: &Certificate, root: &str) -> bool {
    Person::read(root).is_some_and(|wanted| Person::read(&certificate.root) == Some(wanted))
}

// Only a machine key certified by that root proves it, never an alias or a root the log merely names.
pub fn signs_for(paths: &Paths, state: &State, device: &DeviceId, root: &str) -> bool {
    signer(paths, state, device).is_some_and(|(_, certificate)| certified_by(&certificate, root))
}
