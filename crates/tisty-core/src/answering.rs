use std::path::Path;

use ed25519_dalek::VerifyingKey;

use crate::event::DeviceId;
use crate::signing;

/// The last closed segment whose signature answered, and the chain tip after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reached {
    pub segment: u32,
    pub tip: [u8; 32],
    /// Whether this machine has ever shown a signature. Signing only ever starts: once it has,
    /// a segment arriving without one is a signature taken away, not a history from before.
    pub signing: bool,
}

impl Default for Reached {
    fn default() -> Self {
        Self {
            segment: 0,
            tip: signing::NOTHING_BEFORE,
            signing: false,
        }
    }
}

impl Reached {
    pub fn said(&self) -> String {
        let signing = u8::from(self.signing);
        format!("{} {signing} {}", self.segment, signing::hexed(&self.tip))
    }

    pub fn read(said: &str) -> Option<Self> {
        let mut apart = said.split(' ');
        let (Some(segment), Some(signing), Some(tip), None) =
            (apart.next(), apart.next(), apart.next(), apart.next())
        else {
            return None;
        };
        Some(Self {
            segment: segment.parse().ok()?,
            tip: signing::unhexed::<32>(tip)?,
            signing: signing == "1",
        })
    }
}

/// Why a history was not taken in. What cannot be read now may read tomorrow, and a round that
/// says so heals itself; a chain that does not add up is somebody's hand and never heals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Adrift {
    Unreadable(String),
    Disowned(String),
}

/// Every segment of theirs checked against the key it is handed for that machine, recomputed from
/// the bytes that are there. A closed segment is passed over only where it is byte for byte the copy
/// we already answered for — `ours_already` — so what the far side changed is read again however
/// old it is, and the round pays no second reading for asking.
pub fn answers(
    theirs: &Path,
    device: &DeviceId,
    by: &VerifyingKey,
    from: Reached,
    ours_already: &dyn Fn(&str) -> bool,
) -> Result<Reached, Adrift> {
    let found = match crate::store::segments_in(theirs) {
        Ok(found) => found,
        Err(e) => return Err(Adrift::Unreadable(e.to_string())),
    };
    if found.is_empty() {
        return Ok(from);
    }
    if !found
        .iter()
        .any(|one| one.with_extension(signing::SIG).is_file())
    {
        return match from.signing {
            true => Err(Adrift::Disowned(device.0.clone())),
            false => Ok(from),
        };
    }

    // `from.tip` already holds every segment up to `from.segment`, so one of those read again
    // would be folded in twice. Where that is going to happen the memo is no use and the whole
    // history is answered for from its first line; the latch is what must not be given up.
    let from = match found.iter().any(|one| {
        one.file_name()
            .and_then(|one| one.to_str())
            .is_some_and(|named| {
                numbered(named).is_some_and(|n| n <= from.segment) && !ours_already(named)
            })
    }) {
        true => Reached {
            signing: from.signing,
            ..Default::default()
        },
        false => from,
    };

    let mut tip = from.tip;
    let mut held = from;
    for one in &found {
        let Some(named) = one.file_name().and_then(|one| one.to_str()) else {
            continue;
        };
        let number = numbered(named);
        if number.is_some_and(|n| n <= from.segment) && ours_already(named) {
            continue;
        }
        let Ok(bytes) = std::fs::read(one) else {
            return Err(Adrift::Unreadable(named.to_string()));
        };
        let about = signing::About {
            device: &device.0,
            segment: named,
        };
        let signed = match answered(by, &about, one, &bytes, tip, held.signing) {
            Ok(signed) => signed,
            Err(_) if crate::store::left_over(&found, one, &bytes) => continue,
            Err(why) => return Err(why),
        };
        tip = signing::tip_of(tip, &bytes);
        held.signing |= signed;
        if signed && let Some(n) = number {
            held.segment = n;
            held.tip = tip;
        }
    }
    Ok(held)
}

fn answered(
    by: &VerifyingKey,
    about: &signing::About,
    one: &Path,
    bytes: &[u8],
    tip: [u8; 32],
    signing_before: bool,
) -> Result<bool, Adrift> {
    let named = about.segment;
    match std::fs::read(one.with_extension(signing::SIG)) {
        Ok(said) => match String::from_utf8(said) {
            Ok(said) => match signing::holds(by, about, &said) {
                // The whole segment or none of it: a signature over a prefix would let a later line in.
                signing::Holds::Covers(covers) if covers.at != bytes.len() as u64 => {
                    Err(Adrift::Unreadable(named.to_string()))
                }
                signing::Holds::Covers(covers) if signing::tip_of(tip, bytes) != covers.tip => {
                    Err(Adrift::Disowned(named.to_string()))
                }
                signing::Holds::Covers(_) => Ok(true),
                signing::Holds::Refused => Err(Adrift::Disowned(named.to_string())),
                signing::Holds::Unreadable => Err(Adrift::Unreadable(named.to_string())),
            },
            // Bytes that are not text are a signature that will not read, not one taken away.
            Err(_) => Err(Adrift::Unreadable(named.to_string())),
        },
        // Only a sidecar that is not there is taken away; one that will not open is read again.
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(Adrift::Unreadable(named.to_string()))
        }
        Err(_) if signing_before => Err(Adrift::Disowned(named.to_string())),
        Err(_) => Ok(false),
    }
}

fn numbered(named: &str) -> Option<u32> {
    crate::store::is_closed(named)
        .then(|| named.strip_suffix(".tisty")?.parse().ok())
        .flatten()
}

#[cfg(test)]
#[path = "answering_test.rs"]
mod tests;
