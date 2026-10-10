use std::path::Path;

use ed25519_dalek::VerifyingKey;

use crate::event::DeviceId;
use crate::seal;
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

    let last = found
        .iter()
        .filter_map(|one| numbered(one.file_name()?.to_str()?))
        .max()
        .unwrap_or(0);
    let mut walk = Walk {
        device,
        by,
        tip: from.tip,
        held: from,
        sealed: false,
        deferred: None,
        last_closed: None,
        live: last + 1,
    };
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
        walk.segment(one, named, number, &bytes, &|| {
            crate::store::left_over(&found, one, &bytes)
        })?;
    }
    walk.end()
}

struct Walk<'a> {
    device: &'a DeviceId,
    by: &'a VerifyingKey,
    tip: [u8; 32],
    held: Reached,
    sealed: bool,
    deferred: Option<Adrift>,
    last_closed: Option<(u32, [u8; 32])>,
    live: u32,
}

impl Walk<'_> {
    fn segment(
        &mut self,
        one: &Path,
        named: &str,
        number: Option<u32>,
        bytes: &[u8],
        spent: &dyn Fn() -> bool,
    ) -> Result<(), Adrift> {
        let scan = Scan::of(bytes, self.tip);
        let outcome = match &scan.last {
            Some(last) => self.sealed(named, number, &scan, last),
            None => self.unsealed(one, named, number, bytes, &scan),
        };
        match outcome {
            Ok(()) => {}
            Err(_) if spent() => return Ok(()),
            // Answered for by the first seal that follows, if one does: its tip covers these bytes.
            Err(why) if scan.last.is_none() && !self.sealed && !written_sealed(&scan) => {
                self.deferred.get_or_insert(why);
            }
            Err(why) => return Err(why),
        }
        self.tip = scan.tip;
        if let Some(n) = number {
            self.last_closed = Some((n, self.tip));
        }
        Ok(())
    }

    fn sealed(
        &mut self,
        named: &str,
        number: Option<u32>,
        scan: &Scan,
        last: &Last,
    ) -> Result<(), Adrift> {
        if scan.broken {
            return Err(Adrift::Unreadable(named.to_string()));
        }
        let said = &last.read.seal;
        let answers = said.seg == number.unwrap_or(self.live)
            && said.at == last.at
            && said.tip == last.tip
            && said.n == last.n
            && seal::holds(self.by, &self.device.0, &last.read);
        if !answers {
            return Err(Adrift::Disowned(named.to_string()));
        }
        if scan.after || (number.is_some() && !said.closed) {
            return Err(Adrift::Unreadable(named.to_string()));
        }
        self.sealed = true;
        self.deferred = None;
        self.held.signing = true;
        let reached = match number {
            Some(n) => Some((n, scan.tip)),
            None => self.last_closed,
        };
        if let Some((n, tip)) = reached {
            self.held.segment = n;
            self.held.tip = tip;
        }
        Ok(())
    }

    fn unsealed(
        &mut self,
        one: &Path,
        named: &str,
        number: Option<u32>,
        bytes: &[u8],
        scan: &Scan,
    ) -> Result<(), Adrift> {
        if self.sealed || written_sealed(scan) {
            return Err(match number {
                Some(_) => Adrift::Disowned(named.to_string()),
                None => Adrift::Unreadable(named.to_string()),
            });
        }
        let about = signing::About {
            device: &self.device.0,
            segment: named,
        };
        let signed = answered(self.by, &about, one, bytes, self.tip, self.held.signing)?;
        self.held.signing |= signed;
        if signed
            && self.deferred.is_none()
            && let Some(n) = number
        {
            self.held.segment = n;
            self.held.tip = scan.tip;
        }
        Ok(())
    }

    fn end(self) -> Result<Reached, Adrift> {
        match self.deferred {
            Some(why) => Err(why),
            None => Ok(self.held),
        }
    }
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

fn written_sealed(scan: &Scan) -> bool {
    scan.newest.is_some_and(|v| v >= crate::event::SEALED_FROM)
}

struct Last {
    read: seal::Read,
    at: u64,
    tip: [u8; 32],
    n: u64,
}

struct Scan {
    tip: [u8; 32],
    last: Option<Last>,
    after: bool,
    broken: bool,
    newest: Option<u32>,
}

impl Scan {
    fn of(bytes: &[u8], from: [u8; 32]) -> Self {
        let mut scan = Scan {
            tip: from,
            last: None,
            after: false,
            broken: false,
            newest: None,
        };
        let (mut at, mut events, mut latest) = (0u64, 0u64, None);
        for line in bytes.split_inclusive(|one| *one == b'\n') {
            let whole = line.last() == Some(&b'\n');
            let blank = line.iter().all(u8::is_ascii_whitespace);
            match (whole, seal::read(line)) {
                (true, seal::Line::Seal(read)) => {
                    scan.last = Some(Last {
                        read: *read,
                        at,
                        tip: scan.tip,
                        n: events,
                    });
                    scan.after = false;
                }
                (true, seal::Line::Broken) => scan.broken = true,
                _ if blank => {}
                _ => {
                    scan.after = true;
                    if whole {
                        events += 1;
                        latest = Some(line);
                    }
                }
            }
            scan.tip = signing::tip_of(scan.tip, line);
            at += line.len() as u64;
        }
        if scan.last.is_none() {
            scan.newest = latest.and_then(written_at);
        }
        scan
    }
}

fn written_at(line: &[u8]) -> Option<u32> {
    #[derive(serde::Deserialize)]
    struct Written {
        v: u32,
    }
    serde_json::from_slice::<Written>(line)
        .ok()
        .map(|one| one.v)
}

fn numbered(named: &str) -> Option<u32> {
    crate::store::is_closed(named)
        .then(|| named.strip_suffix(".tisty")?.parse().ok())
        .flatten()
}

#[cfg(test)]
#[path = "answering_test.rs"]
mod tests;
