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
    pub moved: Option<[u8; 32]>,
}

impl Default for Reached {
    fn default() -> Self {
        Self {
            segment: 0,
            tip: signing::NOTHING_BEFORE,
            signing: false,
            moved: None,
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
            moved: None,
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
    answers_trusting(theirs, device, &[*by], from, ours_already)
}

/// The first key is the one answered for now; any after it is one it was rotated from.
pub fn answers_trusting(
    theirs: &Path,
    device: &DeviceId,
    trusted: &[VerifyingKey],
    from: Reached,
    ours_already: &dyn Fn(&str) -> bool,
) -> Result<Reached, Adrift> {
    let Some(first) = trusted.first() else {
        return Err(Adrift::Unreadable(device.0.clone()));
    };
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
        by: *first,
        trusted: trusted.to_vec(),
        key_now: None,
        rotating: None,
        signer: None,
        signed_in: None,
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
    by: VerifyingKey,
    trusted: Vec<VerifyingKey>,
    key_now: Option<VerifyingKey>,
    rotating: Option<VerifyingKey>,
    signer: Option<VerifyingKey>,
    signed_in: Option<String>,
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
        let scan = Scan::of(bytes, self.tip, self.device);
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
        let seg = number.unwrap_or(self.live);
        let mut last_checked = false;
        for mark in &scan.marks {
            match mark {
                Mark::Key(said) => {
                    self.key_now.get_or_insert(*said);
                }
                Mark::Rotate(next) => self.rotating = Some(*next),
                Mark::Seal(boundary) => {
                    let Some(next) = self.rotating.take() else {
                        continue;
                    };
                    // The batch that names a new key is the last one its old key seals.
                    let by = self.key_now.unwrap_or(self.by);
                    self.checked(named, seg, boundary, &by)?;
                    if self.trusted.contains(&by) && !self.trusted.contains(&next) {
                        self.trusted.push(next);
                    }
                    self.key_now = Some(next);
                    last_checked = boundary.at == last.at;
                }
            }
        }
        if !last_checked {
            let by = self.key_now.unwrap_or(self.by);
            self.checked(named, seg, last, &by)?;
        }
        if scan.after || (number.is_some() && !last.read.seal.closed) {
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
        let signed = answered(&self.by, &about, one, bytes, self.tip, self.held.signing)?;
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

    fn checked(
        &mut self,
        named: &str,
        seg: u32,
        last: &Last,
        by: &VerifyingKey,
    ) -> Result<(), Adrift> {
        let said = &last.read.seal;
        let answers = said.seg == seg
            && said.at == last.at
            && said.tip == last.tip
            && said.n == last.n
            && seal::holds(by, &self.device.0, &last.read);
        if !answers {
            return Err(Adrift::Disowned(named.to_string()));
        }
        self.signer = Some(*by);
        self.signed_in = Some(named.to_string());
        Ok(())
    }

    /// What answers is a seal by a key trusted here, or by one a trusted key rotated to.
    fn end(self) -> Result<Reached, Adrift> {
        if let Some(why) = self.deferred {
            return Err(why);
        }
        let named = self.signed_in.clone().unwrap_or_default();
        if let Some(signer) = self.signer
            && !self.trusted.contains(&signer)
        {
            return Err(
                match self.key_now.is_some_and(|now| self.trusted.contains(&now)) {
                    true => Adrift::Unreadable(named),
                    false => Adrift::Disowned(named),
                },
            );
        }
        let mut held = self.held;
        held.moved = self
            .key_now
            .filter(|now| *now != self.by && self.trusted.contains(now) && self.signer.is_some())
            .map(|now| now.to_bytes());
        Ok(held)
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

#[derive(Clone)]
struct Last {
    read: seal::Read,
    at: u64,
    tip: [u8; 32],
    n: u64,
}

enum Mark {
    Key(VerifyingKey),
    Rotate(VerifyingKey),
    Seal(Last),
}

struct Scan {
    tip: [u8; 32],
    last: Option<Last>,
    marks: Vec<Mark>,
    after: bool,
    broken: bool,
    newest: Option<u32>,
}

impl Scan {
    fn of(bytes: &[u8], from: [u8; 32], device: &DeviceId) -> Self {
        let mut scan = Scan {
            tip: from,
            last: None,
            marks: Vec::new(),
            after: false,
            broken: false,
            newest: None,
        };
        let (mut at, mut events, mut latest, mut rotating) = (0u64, 0u64, None, false);
        for line in bytes.split_inclusive(|one| *one == b'\n') {
            let whole = line.last() == Some(&b'\n');
            let blank = line.iter().all(u8::is_ascii_whitespace);
            match (whole, seal::read(line)) {
                (true, seal::Line::Seal(read)) => {
                    let last = Last {
                        read: *read,
                        at,
                        tip: scan.tip,
                        n: events,
                    };
                    if std::mem::take(&mut rotating) {
                        scan.marks.push(Mark::Seal(last.clone()));
                    }
                    scan.last = Some(last);
                    scan.after = false;
                }
                (true, seal::Line::Broken) => scan.broken = true,
                _ if blank => {}
                _ => {
                    scan.after = true;
                    if whole {
                        events += 1;
                        latest = Some(line);
                        if let Some(mark) = said_about_its_key(line, device) {
                            rotating |= matches!(mark, Mark::Rotate(_));
                            scan.marks.push(mark);
                        }
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

fn said_about_its_key(line: &[u8], device: &DeviceId) -> Option<Mark> {
    use crate::event::{Event, Op};

    const DEVICE: &[u8] = b"\"op\":\"device.";
    if !line.windows(DEVICE.len()).any(|one| one == DEVICE) {
        return None;
    }
    let event: Event = serde_json::from_slice(line).ok()?;
    if &event.device != device {
        return None;
    }
    match event.op {
        Op::DeviceKey { d, p } | Op::DeviceJoin { d, p: Some(p), .. } if &d == device => {
            signing::read(&p).map(Mark::Key)
        }
        Op::DeviceRotate { d, p } if &d == device => signing::read(&p).map(Mark::Rotate),
        _ => None,
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
