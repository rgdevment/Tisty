use tisty_core::model::Kept;
use tisty_core::untouched::SOWN_BY;
use tisty_core::{DeviceId, Paths, State};
use tisty_share_proto::identity::Person;

use crate::signer;

pub fn sown_by_tisty(kept: &Kept) -> bool {
    kept.guest && kept.by.as_deref() == Some(SOWN_BY)
}

// Only a machine key certified by that root proves it, never an alias or a root the log merely names.
pub fn signs_for(paths: &Paths, state: &State, device: &DeviceId, root: &str) -> bool {
    let Some(wanted) = Person::read(root) else {
        return false;
    };
    signer(paths, state, device)
        .is_some_and(|(_, certificate)| Person::read(&certificate.root) == Some(wanted))
}
