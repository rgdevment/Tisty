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
            true => Err(Adrift::Disowned(String::new())),
            false => Ok(from),
        };
    }

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
        let answered = match std::fs::read_to_string(one.with_extension(signing::SIG)) {
            Ok(said) => match signing::holds(by, &about, &said) {
                // The whole segment or none of it: one answering for a prefix would let a line
                // appended past it in, and the next thing that machine writes would sign it.
                signing::Holds::Covers(covers) if covers.at != bytes.len() as u64 => {
                    return Err(Adrift::Unreadable(named.to_string()));
                }
                signing::Holds::Covers(covers) if signing::tip_of(tip, &bytes) != covers.tip => {
                    return Err(Adrift::Disowned(named.to_string()));
                }
                signing::Holds::Covers(_) => true,
                signing::Holds::Refused => return Err(Adrift::Disowned(named.to_string())),
                signing::Holds::Unreadable => {
                    return Err(Adrift::Unreadable(named.to_string()));
                }
            },
            Err(_) if held.signing => return Err(Adrift::Disowned(named.to_string())),
            Err(_) => false,
        };
        tip = signing::tip_of(tip, &bytes);
        held.signing |= answered;
        if answered && let Some(n) = number {
            held.segment = n;
            held.tip = tip;
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
