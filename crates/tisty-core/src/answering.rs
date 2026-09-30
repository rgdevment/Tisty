use std::path::Path;

use ed25519_dalek::VerifyingKey;

use crate::event::DeviceId;
use crate::signing;

/// The last closed segment whose signature answered, and the chain tip after it. Kept so the
/// next round folds what arrived since rather than the machine's whole history over again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reached {
    pub segment: u32,
    pub tip: [u8; 32],
}

impl Default for Reached {
    fn default() -> Self {
        Self {
            segment: 0,
            tip: signing::NOTHING_BEFORE,
        }
    }
}

/// Every segment of theirs that carries a signature, recomputed from the bytes that are there
/// and checked against the key that machine published. One carrying none is not refused: asking
/// that there be one is a later rule, and every history written before signing has none.
///
/// The error is the segment that did not answer.
pub fn answers(
    dir: &Path,
    device: &DeviceId,
    by: &VerifyingKey,
    from: Reached,
) -> Result<Reached, String> {
    let Ok(found) = crate::store::segments_in(dir) else {
        return Ok(from);
    };
    if !found
        .iter()
        .any(|one| one.with_extension(signing::SIG).is_file())
    {
        return Ok(from);
    }

    let mut tip = from.tip;
    let mut held = from;
    for one in &found {
        let Some(named) = one.file_name().and_then(|one| one.to_str()) else {
            continue;
        };
        let number = numbered(named);
        if number.is_some_and(|n| n <= from.segment) {
            continue;
        }
        let Ok(bytes) = std::fs::read(one) else {
            return Err(named.to_string());
        };
        let about = signing::About {
            device: &device.0,
            segment: named,
        };
        let answered = match std::fs::read_to_string(one.with_extension(signing::SIG)) {
            Ok(said) => {
                let Some(covers) = signing::holds(by, &about, &said) else {
                    return Err(named.to_string());
                };
                let Ok(reach) = usize::try_from(covers.at) else {
                    return Err(named.to_string());
                };
                let Some(under) = bytes.get(..reach) else {
                    return Err(named.to_string());
                };
                if signing::tip_of(tip, under) != covers.tip {
                    return Err(named.to_string());
                }
                true
            }
            Err(_) => false,
        };
        tip = signing::tip_of(tip, &bytes);
        // Only ever the tip of a closed segment: the one still being written grows, and a memo
        // that counted it in would fold those bytes a second time on the next round.
        if answered && let Some(n) = number {
            held = Reached { segment: n, tip };
        }
    }
    Ok(held)
}

fn numbered(named: &str) -> Option<u32> {
    crate::store::is_closed(named)
        .then(|| named.strip_suffix(".tisty")?.parse().ok())
        .flatten()
}

#[cfg(test)]
#[path = "answering_test.rs"]
mod tests;
