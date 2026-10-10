use std::sync::Mutex;

use tisty_core::witness::{self, Fact, channel};

/// Enough for every heavy file a shared folder holds, and a ceiling so a store that churns
/// through them cannot grow this without end.
const VOUCHED_AT_MOST: usize = 4096;

/// The sync has always made that folder answer for its bytes; opening one asks the same, once per
/// file and again only when it changes size, date or identity. What is kept is the digest, so a
/// print the log learns later is measured against it without reading the file again.
type Vouched = std::collections::HashMap<String, String>;

static VOUCHING: std::sync::OnceLock<Mutex<Vouching>> = std::sync::OnceLock::new();

#[derive(Default)]
pub struct Vouching {
    at: Option<std::path::PathBuf>,
    seen: Vouched,
    read: bool,
}

pub fn vouching() -> &'static Mutex<Vouching> {
    VOUCHING.get_or_init(Default::default)
}

/// Reading half a gigabyte to answer for its name is worth doing once, not once per launch: the
/// answers are kept beside the cache, where losing them costs a re-read and nothing else.
pub fn vouching_kept_at(at: std::path::PathBuf) {
    if let Ok(mut one) = vouching().lock() {
        one.at = Some(at);
        one.seen.clear();
        one.read = false;
    }
}

/// What was written down last time, read from disk once and then held.
pub fn vouched_before(asked: &str) -> Option<String> {
    let mut one = vouching().lock().ok()?;
    if !one.read {
        one.read = true;
        if let Some(said) = one
            .at
            .as_ref()
            .and_then(|at| std::fs::read_to_string(at).ok())
            .and_then(|said| serde_json::from_str::<Vouched>(&said).ok())
        {
            one.seen = said;
        }
    }
    one.seen.get(asked).cloned()
}

pub fn vouching_kept(asked: String, said: String) {
    let Ok(mut one) = vouching().lock() else {
        return;
    };
    // A file that changed keeps its path with a new size or date, so the old row is dead weight;
    // dropping the lot is simpler than tracking which, and costs one re-read each.
    if one.seen.len() >= VOUCHED_AT_MOST {
        one.seen.clear();
    }
    one.seen.insert(asked, said);
    let Some(at) = one.at.clone() else {
        return;
    };
    let Ok(body) = serde_json::to_vec(&one.seen) else {
        return;
    };
    drop(one);
    if let Some(parent) = at.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = tisty_core::store::write_atomic(&at, &body) {
        witness::warn(
            channel::ATTACH,
            "what answered for its name could not be written down",
            &[("at", Fact::Path(at)), ("why", Fact::Why(e.to_string()))],
        );
    }
}

/// The whole digest the log wrote down is what answers for a file when there is one; the name's
/// few bits are all that is left for one no machine ever brought home.
pub fn vouches(at: &std::path::Path, reference: &str, avowed: Option<&str>) -> Option<bool> {
    let mut parts = reference.rsplit('/');
    let (Some(leaf), Some(shelf)) = (parts.next(), parts.next()) else {
        return Some(false);
    };
    let told = std::fs::metadata(at).ok()?;
    let asked = format!(
        "{}|{}|{}|{}",
        at.display(),
        told.len(),
        nanos(told.modified()),
        identity(at, &told)
    );

    let sha256 = match vouched_before(&asked) {
        Some(held) => held,
        None => {
            // Not while the lock is held: reading the file takes seconds, and every other window
            // command that touches an attachment would wait behind it.
            let (sha256, _) = tisty_core::attach::hashed(at).ok()?;
            vouching_kept(asked, sha256.clone());
            sha256
        }
    };
    Some(
        tisty_core::attach::vouched(shelf, leaf, &sha256)
            && avowed.is_none_or(|avowed| avowed == sha256),
    )
}

fn nanos(when: std::io::Result<std::time::SystemTime>) -> u128 {
    when.ok()
        .and_then(|one| one.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |one| one.as_nanos())
}

/// What the system moves on every write and every rename, and no file time a person or a cloud
/// client sets puts back: a body rewritten under its old size and date still gets a new key.
#[cfg(unix)]
fn identity(_: &std::path::Path, told: &std::fs::Metadata) -> String {
    use std::os::unix::fs::MetadataExt;
    format!("{}.{}", told.ctime(), told.ctime_nsec())
}

#[cfg(windows)]
fn identity(at: &std::path::Path, told: &std::fs::Metadata) -> String {
    format!(
        "{}.{}",
        nanos(told.created()),
        crate::desktop::changed(at).unwrap_or(0)
    )
}

#[cfg(not(any(unix, windows)))]
fn identity(_: &std::path::Path, told: &std::fs::Metadata) -> String {
    nanos(told.created()).to_string()
}
