use std::io::{Read, Seek};
use std::path::Path;

use ed25519_dalek::VerifyingKey;

use crate::event::DeviceId;
use crate::seal::{self, Line};

const WINDOW: u64 = 64 * 1024;

pub(super) fn answered_at(
    one: &Path,
    device: &DeviceId,
    by: &VerifyingKey,
) -> Option<(u64, [u8; 32])> {
    sealed_at(one, device, by).or_else(|| signed_at(one, device, by))
}

pub(super) fn holds_a_seal(one: &Path) -> bool {
    tail_of(one).is_some_and(|(_, tail)| {
        tail.split_inclusive(|one| *one == b'\n')
            .any(|line| line.last() == Some(&b'\n') && matches!(seal::read(line), Line::Seal(_)))
    })
}

fn sealed_at(one: &Path, device: &DeviceId, by: &VerifyingKey) -> Option<(u64, [u8; 32])> {
    let (mut at, tail) = tail_of(one)?;
    let mut bytes = tail.as_slice();
    if at > 0 {
        let cut = bytes.iter().position(|one| *one == b'\n')? + 1;
        at += cut as u64;
        bytes = &bytes[cut..];
    }
    let mut last = None;
    for line in bytes.split_inclusive(|one| *one == b'\n') {
        if line.last() == Some(&b'\n')
            && let Line::Seal(read) = seal::read(line)
        {
            last = Some((at, read));
        }
        at += line.len() as u64;
    }
    let (offset, read) = last?;
    (read.seal.at == offset && seal::holds(by, &device.0, &read)).then_some((offset, read.seal.tip))
}

fn signed_at(one: &Path, device: &DeviceId, by: &VerifyingKey) -> Option<(u64, [u8; 32])> {
    let named = one.file_name()?.to_str()?;
    let said = std::fs::read_to_string(one.with_extension(crate::signing::SIG)).ok()?;
    let about = crate::signing::About {
        device: &device.0,
        segment: named,
    };
    let crate::signing::Holds::Covers(held) = crate::signing::holds(by, &about, &said) else {
        return None;
    };
    let weighs = std::fs::metadata(one).ok()?.len();
    // A signature over a prefix leaves the rest of a closed segment out of the chain.
    let fits = match super::is_closed(named) {
        true => held.at == weighs,
        false => held.at <= weighs,
    };
    fits.then_some((held.at, held.tip))
}

fn tail_of(one: &Path) -> Option<(u64, Vec<u8>)> {
    let mut file = std::fs::File::open(one).ok()?;
    crate::counting::opened();
    let weighs = file.metadata().ok()?.len();
    let start = weighs.saturating_sub(WINDOW);
    file.seek(std::io::SeekFrom::Start(start)).ok()?;
    let mut tail = Vec::new();
    file.read_to_end(&mut tail).ok()?;
    Some((start, tail))
}
