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

/// A provider that holds only sealed files, under `sealed/`, beside the three that must stay readable by every build.
pub struct Sealed<R> {
    inner: R,
    key: CloudKey,
    cloud: String,
}

struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        let n = TEMPS.fetch_add(1, Ordering::Relaxed);
        Self(std::env::temp_dir().join(format!("tisty-sealed-{}-{n}", std::process::id())))
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

fn clear(name: &str) -> bool {
    name == NAMED || name == tisty_core::moved::NAMED || name == format!("{STORE}/{MARKER}")
}

fn placed(name: &str) -> String {
    match clear(name) {
        true => name.to_string(),
        false => format!("{INSIDE}/{name}"),
    }
}

fn shown(name: &str) -> Option<String> {
    if clear(name) {
        return Some(name.to_string());
    }
    let inside = name.strip_prefix(INSIDE)?.strip_prefix('/')?;
    (!inside.is_empty() && inside != "sealed.mark").then(|| inside.to_string())
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

    fn seen(&self, one: Seen, name: &str, hash: String) -> Seen {
        match clear(name) {
            true => one,
            false => Seen {
                name: name.to_string(),
                bytes: plain_len(one.bytes).unwrap_or(0),
                hash,
                revision: one.revision,
            },
        }
    }

    fn listed(&self, many: Vec<Seen>) -> Vec<Seen> {
        many.into_iter()
            .filter_map(|one| {
                let name = shown(&one.name)?;
                Some(self.seen(one, &name, String::new()))
            })
            .collect()
    }

    fn within(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        let top = match under.is_empty() {
            true => INSIDE.to_string(),
            false => format!("{INSIDE}/{under}"),
        };
        let mut seen = self.listed(self.inner.list(&top)?);
        if under.is_empty() || under == STORE {
            let beside = self.inner.list(under)?;
            seen.extend(beside.into_iter().filter(|one| clear(&one.name)));
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
        if from > 0 {
            return Err(broke("a sealed file is never fetched from the middle"));
        }
        let (cipher, plain) = (Temp::new(), Temp::new());
        let mut to = std::fs::File::create(&cipher.0).map_err(broke)?;
        let got = self.inner.fetch(&placed(name), 0, &mut to);
        drop(to);
        let got = got?;
        let mut from_cipher = std::fs::File::open(&cipher.0).map_err(broke)?;
        let mut to_plain = std::fs::File::create(&plain.0).map_err(broke)?;
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
        std::io::copy(&mut read, into).map_err(broke)?;
        Ok(self.seen(got, name, hash))
    }

    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch> {
        if clear(name) {
            return self.inner.put(name, from, expect);
        }
        let cipher = Temp::new();
        let mut plain = std::fs::File::open(from).map_err(broke)?;
        let mut to = std::fs::File::create(&cipher.0).map_err(broke)?;
        seal(&self.key, &self.cloud, name, &mut plain, &mut to).map_err(broke)?;
        drop(to);
        let there = self.inner.put(&placed(name), &cipher.0, expect)?;
        if !there.hash.is_empty() && self.inner.hash_of(&cipher.0).map_err(broke)? != there.hash {
            return Err(broke(format!(
                "{name} did not reach the provider as sealed"
            )));
        }
        let hash = self.inner.hash_of(from).map_err(broke)?;
        Ok(self.seen(there, name, hash))
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        self.inner.delete(&placed(name), expect)
    }

    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        self.inner.hash_of(local)
    }

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        Ok(self
            .inner
            .about(&placed(name))?
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
                gone: gone.iter().filter_map(|name| shown(name)).collect(),
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
