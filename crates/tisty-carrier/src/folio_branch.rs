use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::policy;

pub(crate) struct Signed {
    pub said: Vec<u8>,
    pub signature: Vec<u8>,
}

pub(crate) fn kept_at(dir: &Path, name: &str) -> (PathBuf, PathBuf) {
    (dir.join(name), dir.join(format!("{name}.minisig")))
}

pub(crate) fn fresh(at: &Path, for_as_long: Duration) -> bool {
    std::fs::metadata(at)
        .and_then(|one| one.modified())
        .ok()
        .and_then(|when| when.elapsed().ok())
        .is_some_and(|since| since < for_as_long)
}

fn holds(said: &[u8], signature: &[u8], named: &str, key: &str) -> bool {
    policy::signed_as(said, &String::from_utf8_lossy(signature), key, named)
}

pub(crate) fn kept(said_at: &Path, signature_at: &Path, named: &str, key: &str) -> Option<Vec<u8>> {
    let said = std::fs::read(said_at).ok()?;
    let signature = std::fs::read(signature_at).ok()?;
    holds(&said, &signature, named, key).then_some(said)
}

pub(crate) fn brought(
    bring: &impl Fn(&str) -> Option<Vec<u8>>,
    url: &str,
    named: &str,
    key: &str,
) -> Option<Signed> {
    let asked = || {
        let said = bring(url)?;
        let signature = bring(&format!("{url}.minisig"))?;
        Some(Signed { said, signature })
    };
    let first = asked()?;
    if holds(&first.said, &first.signature, named, key) {
        return Some(first);
    }
    // The two files are asked for apart, so a publish between them looks like a bad signature.
    asked().filter(|again| holds(&again.said, &again.signature, named, key))
}

pub(crate) fn keep(said_at: &Path, signature_at: &Path, signed: &Signed) {
    if let Some(dir) = said_at.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = tisty_core::store::write_atomic(signature_at, &signed.signature);
    let _ = tisty_core::store::write_atomic(said_at, &signed.said);
}

#[cfg(test)]
#[path = "folio_branch_test.rs"]
mod tests;
