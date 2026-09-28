use std::sync::Mutex;

use tisty_core::witness::{self, Fact, channel};

/// Enough for every heavy file a shared folder holds, and a ceiling so a store that churns
/// through them cannot grow this without end.
const VOUCHED_AT_MOST: usize = 4096;

/// The sync has always made that folder answer for its bytes; opening one asks the same, once per
/// file and again only when it changes size or date.
type Vouched = std::collections::HashMap<String, bool>;

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
pub fn vouched_before(asked: &str) -> Option<bool> {
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
    one.seen.get(asked).copied()
}

pub fn vouching_kept(asked: String, said: bool) {
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

pub fn vouches(at: &std::path::Path, reference: &str) -> bool {
    let mut parts = reference.rsplit('/');
    let (Some(leaf), Some(shelf)) = (parts.next(), parts.next()) else {
        return false;
    };
    let Ok(told) = std::fs::metadata(at) else {
        return false;
    };
    let when = told
        .modified()
        .ok()
        .and_then(|one| one.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|one| one.as_secs())
        .unwrap_or(0);
    let asked = format!("{}|{}|{when}", at.display(), told.len());

    if let Some(held) = vouched_before(&asked) {
        return held;
    }
    // Not while the lock is held: reading the file takes seconds, and every other window
    // command that touches an attachment would wait behind it.
    let said = tisty_core::attach::hashed(at)
        .is_ok_and(|(sha256, _)| tisty_core::attach::vouched(shelf, leaf, &sha256));
    vouching_kept(asked, said);
    said
}
