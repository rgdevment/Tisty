use tisty_core::person::Persons;
use tisty_core::{DeviceId, Paths};
use tisty_share_proto::identity::{Certificate, Person};

use crate::signer_among;

pub fn certified_by(certificate: &Certificate, root: &str) -> Option<Person> {
    let wanted = Person::read(root)?;
    (Person::read(&certificate.root) == Some(wanted)).then_some(wanted)
}

// Only a machine key certified by that root proves it, never an alias or a root the log merely names.
pub fn signs_for(
    paths: &Paths,
    persons: &Persons,
    device: &DeviceId,
    root: &str,
) -> Option<Person> {
    signer_among(paths, persons, device)
        .and_then(|(_, certificate)| certified_by(&certificate, root))
}
