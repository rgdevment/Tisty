use std::io::Write;
use std::path::Path;

use tisty_share_proto::cloud::{CloudKey, Mark, Unsealed, mark, mark_says, open, seal};
use tisty_sync::{MARKER, NAMED, STORE};

use crate::{Changes, Expect, Hitch, Limits, Remote, Seen, Watch};

pub(crate) const INSIDE: &str = "sealed";

pub(crate) fn reading(root: &str) -> Option<&str> {
    root.strip_prefix(INSIDE)?.strip_prefix(':')
}

pub(crate) fn stays_plain(name: &str, folds: bool) -> bool {
    let same = |one: &str| match folds {
        true => one.to_lowercase() == name.to_lowercase(),
        false => one == name,
    };
    [
        NAMED,
        tisty_core::moved::NAMED,
        &format!("{STORE}/{MARKER}"),
    ]
    .into_iter()
    .any(same)
}
const HEADER: u64 = 24;
const TAG: u64 = 16;
const BLOCK: u64 = 64 * 1024;

pub struct Sealed<R> {
    inner: R,
    key: CloudKey,
    cloud: String,
    root: String,
    marked: std::sync::atomic::AtomicBool,
}

fn broke(e: impl ToString) -> Hitch {
    Hitch::Broke(e.to_string())
}

const MARK: &str = "sealed.mark";
const MARK_AT_MOST: u64 = 512;

fn named_back(hitch: Hitch, inner: &str, name: &str) -> Hitch {
    match hitch {
        Hitch::Missing(one) if one == inner => Hitch::Missing(name.to_string()),
        Hitch::Changed(one) if one == inner => Hitch::Changed(name.to_string()),
        other => other,
    }
}

#[cfg(test)]
pub(crate) fn sealed_len(plain: u64) -> u64 {
    HEADER + plain + TAG * plain.div_ceil(BLOCK).max(1)
}

fn plain_room(sealed: u64) -> u64 {
    let body = sealed.saturating_sub(HEADER);
    (body / (BLOCK + TAG)) * BLOCK + (body % (BLOCK + TAG)).saturating_sub(TAG)
}

pub(crate) fn plain_len(sealed: u64) -> Option<u64> {
    let body = sealed.checked_sub(HEADER)?;
    let (full, rest) = (body / (BLOCK + TAG), body % (BLOCK + TAG));
    match (full, rest) {
        (0, rest) if rest < TAG => None,
        (_, 0) => Some(full * BLOCK),
        (_, rest) if rest < TAG => None,
        (_, rest) => Some(full * BLOCK + rest - TAG),
    }
}

impl<R: Remote> Sealed<R> {
    pub fn new(inner: R, key: CloudKey, cloud: impl Into<String>) -> Self {
        let cloud = cloud.into();
        Self {
            inner,
            key,
            root: format!("{INSIDE}:{cloud}"),
            cloud,
            marked: std::sync::atomic::AtomicBool::new(false),
        }
    }

    #[cfg(test)]
    pub(crate) fn inner(&self) -> &R {
        &self.inner
    }

    pub fn marked(&self) -> Result<Option<Mark>, Hitch> {
        let at = format!("{INSIDE}/{MARK}");
        match self.inner.about(&at)? {
            None => return Ok(None),
            Some(there) if there.bytes > MARK_AT_MOST => return Ok(Some(Mark::Damaged)),
            Some(_) => {}
        }
        let mut said = Vec::new();
        match self.inner.fetch(&at, 0, &mut said) {
            Ok(_) => Ok(Some(mark_says(&self.key, &self.cloud, &said))),
            Err(Hitch::Missing(_)) => Ok(None),
            Err(other) => Err(other),
        }
    }

    pub fn mark_it(&self) -> Result<(), Hitch> {
        match self.marked()? {
            None => {}
            said => return self.mark_holds(said),
        }
        let said = mark(&self.key, &self.cloud).ok_or_else(|| broke("no way to mark the cloud"))?;
        let held = crate::temp::Temp::new();
        let mut file = held.create().map_err(broke)?;
        file.write_all(&said).map_err(broke)?;
        drop(file);
        self.inner
            .put(&format!("{INSIDE}/{MARK}"), &held.0, Expect::Absent)?;
        match self.marked()? {
            None => Err(broke("the mark could not be read back")),
            said => self.mark_holds(said),
        }
    }

    fn mark_holds(&self, said: Option<Mark>) -> Result<(), Hitch> {
        let at = || self.root.clone();
        match said {
            Some(Mark::Holds) => Ok(()),
            Some(Mark::OtherPhrase) => Err(Hitch::Barred(tisty_sync::Trouble::OtherPhrase(at()))),
            Some(Mark::Newer) => Err(Hitch::Barred(tisty_sync::Trouble::Shape(at()))),
            _ => Err(broke("the phrase mark of this cloud is damaged")),
        }
    }

    fn folds(&self) -> bool {
        self.inner.limits().folds_case
    }

    fn same(&self, one: &str, other: &str) -> bool {
        match self.folds() {
            true => one.to_lowercase() == other.to_lowercase(),
            false => one == other,
        }
    }

    fn is_clear(&self, name: &str) -> bool {
        stays_plain(name, self.folds())
    }

    fn reserved(&self, name: &str) -> Result<(), Hitch> {
        match self.same(name, MARK) {
            true => Err(broke("that name is the mark of a sealed cloud")),
            false => Ok(()),
        }
    }

    fn placed(&self, name: &str) -> String {
        match self.is_clear(name) {
            true => name.to_string(),
            false => format!("{INSIDE}/{name}"),
        }
    }

    fn bound(&self, name: &str) -> String {
        match self.folds() {
            true => name.to_lowercase(),
            false => name.to_string(),
        }
    }

    fn shown(&self, name: &str) -> Option<String> {
        if self.is_clear(name) {
            return Some(name.to_ascii_lowercase());
        }
        let (top, inside) = name.split_once('/')?;
        (self.same(top, INSIDE)
            && !inside.is_empty()
            && !self.same(inside, MARK)
            && !self.is_clear(inside))
        .then(|| inside.to_string())
    }

    fn held_as(&self, one: &Seen, asked: &str) -> String {
        self.shown(&one.name).unwrap_or_else(|| asked.to_string())
    }

    fn seen(&self, one: Seen, name: &str, hash: String) -> Seen {
        match self.is_clear(name) {
            true => Seen {
                name: name.to_string(),
                ..one
            },
            false => Seen {
                name: name.to_string(),
                bytes: plain_len(one.bytes).unwrap_or(one.bytes),
                hash,
                revision: one.revision,
            },
        }
    }

    fn listed(&self, many: Vec<Seen>) -> Vec<Seen> {
        many.into_iter()
            .filter_map(|one| {
                let name = self.shown(&one.name)?;
                Some(self.seen(one, &name, String::new()))
            })
            .collect()
    }

    fn within(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        let under = under.trim_matches('/');
        if under.is_empty() {
            return Ok(self.listed(self.inner.list("")?));
        }
        let mut seen = self.listed(self.inner.list(&format!("{INSIDE}/{under}"))?);
        if under == STORE {
            let id = format!("{STORE}/{MARKER}");
            seen.extend(self.inner.about(&id)?.map(|one| {
                let name = self.held_as(&one, &id);
                self.seen(one, &name, String::new())
            }));
        }
        Ok(seen)
    }
}

impl<R: Remote> Remote for Sealed<R> {
    fn limits(&self) -> Limits {
        let mut limits = self.inner.limits();
        limits.most_per_file = plain_room(limits.most_per_file);
        limits
    }

    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        self.within(under)
    }

    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        if self.is_clear(name) {
            return self.inner.fetch(name, from, into);
        }
        self.reserved(name)
            .map_err(|_| Hitch::Missing(name.to_string()))?;
        let inner = self.placed(name);
        let (cipher, plain) = (crate::temp::Temp::new(), crate::temp::Temp::new());
        let mut to = cipher.create().map_err(broke)?;
        let got = self.inner.fetch(&inner, 0, &mut to);
        drop(to);
        let got = got.map_err(|e| named_back(e, &inner, name))?;
        if plain_len(got.bytes).is_some_and(|len| from > len) {
            return Err(broke(format!("{name} was asked for past its end")));
        }
        let mut from_cipher = std::fs::File::open(&cipher.0).map_err(broke)?;
        let mut to_plain = plain.create().map_err(broke)?;
        open(
            &self.key,
            &self.cloud,
            &self.bound(name),
            &mut from_cipher,
            &mut to_plain,
        )
        .map_err(|e: Unsealed| broke(format!("{name} does not open: {e}")))?;
        drop(to_plain);
        let hash = self.inner.hash_of(&plain.0).map_err(broke)?;
        let mut read = std::fs::File::open(&plain.0).map_err(broke)?;
        std::io::Seek::seek(&mut read, std::io::SeekFrom::Start(from)).map_err(broke)?;
        std::io::copy(&mut read, into).map_err(broke)?;
        let there = self.held_as(&got, name);
        Ok(self.seen(got, &there, hash))
    }

    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch> {
        if self.is_clear(name) {
            return self.inner.put(name, from, expect);
        }
        self.reserved(name)?;
        let inner = self.placed(name);
        let (cipher, snapshot) = (crate::temp::Temp::new(), crate::temp::Temp::new());
        let mut source = std::fs::File::open(from).map_err(broke)?;
        let mut copy = snapshot.create().map_err(broke)?;
        std::io::copy(&mut source, &mut copy).map_err(broke)?;
        drop(copy);
        let mut plain = std::fs::File::open(&snapshot.0).map_err(broke)?;
        let mut to = cipher.create().map_err(broke)?;
        seal(
            &self.key,
            &self.cloud,
            &self.bound(name),
            &mut plain,
            &mut to,
        )
        .map_err(broke)?;
        drop(to);
        let there = self
            .inner
            .put(&inner, &cipher.0, expect)
            .map_err(|e| named_back(e, &inner, name))?;
        if !there.hash.is_empty() && self.inner.hash_of(&cipher.0).map_err(broke)? != there.hash {
            return Err(broke(format!(
                "{name} did not reach the provider as sealed"
            )));
        }
        let hash = self.inner.hash_of(&snapshot.0).map_err(broke)?;
        let held = self.held_as(&there, name);
        Ok(self.seen(there, &held, hash))
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        self.reserved(name)?;
        let inner = self.placed(name);
        self.inner
            .delete(&inner, expect)
            .map_err(|e| named_back(e, &inner, name))
    }

    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        self.inner.hash_of(local)
    }

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        if self.reserved(name).is_err() {
            return Ok(None);
        }
        let inner = self.placed(name);
        Ok(self
            .inner
            .about(&inner)
            .map_err(|e| named_back(e, &inner, name))?
            .map(|one| {
                let held = self.held_as(&one, name);
                self.seen(one, &held, String::new())
            }))
    }

    fn changes(&self, since: Option<&str>) -> Result<Changes, Hitch> {
        Ok(match self.inner.changes(since)? {
            Changes::Whole { seen, cursor } => Changes::Whole {
                seen: self.listed(seen),
                cursor,
            },
            Changes::Since {
                changed,
                gone,
                cursor,
            } => Changes::Since {
                changed: self.listed(changed),
                gone: gone.iter().filter_map(|name| self.shown(name)).collect(),
                cursor,
            },
        })
    }

    fn hears(&self, since: &str) -> Option<Box<dyn Watch>> {
        self.inner.hears(since)
    }

    fn room(&self) -> Result<Option<u64>, Hitch> {
        self.inner.room()
    }

    fn spent_beyond(&self) -> u64 {
        self.inner.spent_beyond()
    }

    fn moved_on(&self) {
        self.inner.moved_on();
    }

    fn root(&self) -> &str {
        &self.root
    }

    // Asked once for each carrier, so a machine with another phrase never sends into this cloud.
    fn mark_forgotten(&self) {
        self.marked
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    fn mark_up(&self) -> Result<(), Hitch> {
        use std::sync::atomic::Ordering;
        if !self.marked.load(Ordering::Relaxed) {
            self.mark_it()?;
            self.marked.store(true, Ordering::Relaxed);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "sealed_test.rs"]
mod tests;
