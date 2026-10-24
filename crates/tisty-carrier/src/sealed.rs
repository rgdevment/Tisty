use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tisty_share_proto::cloud::{CloudKey, Unsealed, open, seal};
use tisty_sync::{MARKER, NAMED, STORE};

use crate::{Changes, Expect, Hitch, Limits, Remote, Seen, Watch};

const INSIDE: &str = "sealed";
const HEADER: u64 = 24;
const TAG: u64 = 16;
const BLOCK: u64 = 64 * 1024;

static TEMPS: AtomicU64 = AtomicU64::new(0);

pub struct Sealed<R> {
    inner: R,
    key: CloudKey,
    cloud: String,
}

struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        let n = TEMPS.fetch_add(1, Ordering::Relaxed);
        let at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        Self(std::env::temp_dir().join(format!("tisty-sealed-{}-{n}-{at:x}", std::process::id())))
    }

    fn create(&self) -> std::io::Result<std::fs::File> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        options.open(&self.0)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn broke(e: impl ToString) -> Hitch {
    Hitch::Broke(e.to_string())
}

const MARK: &str = "sealed.mark";

fn clear(name: &str) -> bool {
    name == NAMED || name == tisty_core::moved::NAMED || name == format!("{STORE}/{MARKER}")
}

fn placed(name: &str) -> String {
    match clear(name) {
        true => name.to_string(),
        false => format!("{INSIDE}/{name}"),
    }
}

fn named_back(hitch: Hitch, inner: &str, name: &str) -> Hitch {
    match hitch {
        Hitch::Missing(one) if one == inner => Hitch::Missing(name.to_string()),
        Hitch::Changed(one) if one == inner => Hitch::Changed(name.to_string()),
        other => other,
    }
}

fn overhead(plain: u64) -> u64 {
    HEADER.saturating_add(TAG.saturating_mul(plain.div_ceil(BLOCK).max(1)))
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
        Self {
            inner,
            key,
            cloud: cloud.into(),
        }
    }

    fn folds(&self) -> bool {
        self.inner.limits().folds_case
    }

    fn same(&self, one: &str, other: &str) -> bool {
        match self.folds() {
            true => one.eq_ignore_ascii_case(other),
            false => one == other,
        }
    }

    fn is_clear(&self, name: &str) -> bool {
        [
            NAMED,
            tisty_core::moved::NAMED,
            &format!("{STORE}/{MARKER}"),
        ]
        .into_iter()
        .any(|one| self.same(one, name))
    }

    fn reserved(&self, name: &str) -> Result<(), Hitch> {
        match self.same(name, MARK) {
            true => Err(broke("that name is the mark of a sealed cloud")),
            false => Ok(()),
        }
    }

    fn shown(&self, name: &str) -> Option<String> {
        if self.is_clear(name) {
            return Some(name.to_ascii_lowercase());
        }
        let (top, inside) = name.split_once('/')?;
        (self.same(top, INSIDE) && !inside.is_empty() && !self.same(inside, MARK))
            .then(|| inside.to_string())
    }

    fn seen(&self, one: Seen, name: &str, hash: String) -> Seen {
        match clear(name) {
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
            seen.extend(
                self.inner
                    .about(&id)?
                    .map(|one| self.seen(one, &id, String::new())),
            );
        }
        Ok(seen)
    }
}

impl<R: Remote> Remote for Sealed<R> {
    fn limits(&self) -> Limits {
        let mut limits = self.inner.limits();
        limits.most_per_file = limits
            .most_per_file
            .saturating_sub(overhead(limits.most_per_file));
        limits
    }

    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        self.within(under)
    }

    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        if clear(name) {
            return self.inner.fetch(name, from, into);
        }
        self.reserved(name)
            .map_err(|_| Hitch::Missing(name.to_string()))?;
        let inner = placed(name);
        let (cipher, plain) = (Temp::new(), Temp::new());
        let mut to = cipher.create().map_err(broke)?;
        let got = self.inner.fetch(&inner, 0, &mut to);
        drop(to);
        let got = got.map_err(|e| named_back(e, &inner, name))?;
        let mut from_cipher = std::fs::File::open(&cipher.0).map_err(broke)?;
        let mut to_plain = plain.create().map_err(broke)?;
        open(
            &self.key,
            &self.cloud,
            name,
            &mut from_cipher,
            &mut to_plain,
        )
        .map_err(|e: Unsealed| broke(format!("{name} does not open: {e}")))?;
        drop(to_plain);
        let hash = self.inner.hash_of(&plain.0).map_err(broke)?;
        let mut read = std::fs::File::open(&plain.0).map_err(broke)?;
        std::io::Seek::seek(&mut read, std::io::SeekFrom::Start(from)).map_err(broke)?;
        std::io::copy(&mut read, into).map_err(broke)?;
        Ok(self.seen(got, name, hash))
    }

    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch> {
        if clear(name) {
            return self.inner.put(name, from, expect);
        }
        self.reserved(name)?;
        let inner = placed(name);
        let (cipher, snapshot) = (Temp::new(), Temp::new());
        std::fs::copy(from, &snapshot.0).map_err(broke)?;
        let mut plain = std::fs::File::open(&snapshot.0).map_err(broke)?;
        let mut to = cipher.create().map_err(broke)?;
        seal(&self.key, &self.cloud, name, &mut plain, &mut to).map_err(broke)?;
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
        Ok(self.seen(there, name, hash))
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        self.reserved(name)?;
        let inner = placed(name);
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
        let inner = placed(name);
        Ok(self
            .inner
            .about(&inner)
            .map_err(|e| named_back(e, &inner, name))?
            .map(|one| self.seen(one, name, String::new())))
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
}

#[cfg(test)]
#[path = "sealed_test.rs"]
mod tests;
