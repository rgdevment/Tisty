use std::path::Path;

use crate::event::{DeviceId, Event, Op, SIGNED_FROM};

// Read whole and in order, so a renamed segment or one slipped in ahead cannot pass as ours.
pub fn first_key_past(mine: &Path, theirs: &Path, who: &DeviceId) -> Option<String> {
    let held = whole(mine)?;
    if held.is_empty() || !held.split(|b| *b == b'\n').any(from_before_signing) {
        return None;
    }
    let folder = whole(theirs)?;
    let past = folder.strip_prefix(held.as_slice())?;
    past.split(|b| *b == b'\n')
        .filter_map(|line| serde_json::from_slice::<Event>(line).ok())
        .filter(|one| &one.device == who)
        .find_map(|one| match one.op {
            Op::DeviceKey { d, p } | Op::DeviceJoin { d, p: Some(p), .. }
                if &d == who && crate::signing::read(&p).is_some() =>
            {
                Some(p)
            }
            _ => None,
        })
}

fn from_before_signing(line: &[u8]) -> bool {
    serde_json::from_slice::<super::Written>(line).is_ok_and(|one| one.v < SIGNED_FROM)
}

fn whole(dir: &Path) -> Option<Vec<u8>> {
    let mut all = Vec::new();
    for segment in super::segments_in(dir).ok()? {
        all.extend(std::fs::read(segment).ok()?);
    }
    Some(all)
}

#[cfg(test)]
#[path = "before_test.rs"]
mod tests;
