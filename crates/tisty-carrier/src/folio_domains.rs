use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::policy;

const SHARDS_URL: &str =
    "https://raw.githubusercontent.com/rgdevment/Folio-by-Tisty/main/docs/domains/";
const KEPT: &str = "folio-domains";
const KEPT_FOR: Duration = Duration::from_secs(24 * 60 * 60);
const PROOF_CHARACTERS: usize = 32;
const NAME_AT_MOST: usize = 253;
const LABEL_AT_MOST: usize = 63;
const PUNYCODE: &str = "xn--";

pub fn proof_of(root: &str, domain: &str) -> String {
    tisty_core::attach::printed(format!("{root}{domain}").as_bytes())[..PROOF_CHARACTERS].to_owned()
}

pub fn shard_of(proof: &str) -> &str {
    &proof[..2]
}

fn a_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= LABEL_AT_MOST
        && label
            .bytes()
            .all(|one| one.is_ascii_lowercase() || one.is_ascii_digit() || one == b'-')
        && !label.starts_with('-')
        && !label.ends_with('-')
}

fn a_top_label(label: &str) -> bool {
    match label.strip_prefix(PUNYCODE) {
        Some(rest) => {
            (1..=59).contains(&rest.len())
                && rest
                    .bytes()
                    .all(|one| one.is_ascii_lowercase() || one.is_ascii_digit() || one == b'-')
        }
        None => {
            (2..=63).contains(&label.len()) && label.bytes().all(|one| one.is_ascii_lowercase())
        }
    }
}

// The same shape Folio checks before a domain goes on its list: lowercase ASCII, punycode allowed.
pub fn a_domain(said: &str) -> bool {
    let labels: Vec<&str> = said.split('.').collect();
    let Some((top, rest)) = labels.split_last() else {
        return false;
    };
    (1..=NAME_AT_MOST).contains(&said.len())
        && !rest.is_empty()
        && rest.iter().all(|one| a_label(one))
        && a_top_label(top)
}

pub fn checked(
    home: &Path,
    root: &str,
    domain: &str,
    bring: impl Fn(&str) -> Option<Vec<u8>>,
) -> bool {
    checked_under(home, root, domain, &bring, crate::folio_limits::KEY)
}

fn checked_under(
    home: &Path,
    root: &str,
    domain: &str,
    bring: &impl Fn(&str) -> Option<Vec<u8>>,
    key: &str,
) -> bool {
    if !a_domain(domain) {
        return false;
    }
    let proof = proof_of(root, domain);
    let shard = shard_of(&proof);
    let holds = |held: Vec<String>| held.contains(&proof);
    kept_proofs(home, shard, key).is_some_and(holds)
        || fetched_proofs(home, shard, bring, key).is_some_and(holds)
}

fn kept_at(home: &Path, shard: &str) -> (PathBuf, PathBuf) {
    let dir = home.join(KEPT);
    (
        dir.join(format!("{shard}.json")),
        dir.join(format!("{shard}.json.minisig")),
    )
}

fn fresh(at: &Path) -> bool {
    std::fs::metadata(at)
        .and_then(|one| one.modified())
        .ok()
        .and_then(|when| when.elapsed().ok())
        .is_some_and(|since| since < KEPT_FOR)
}

// Read again under Folio's signature even from the copy kept here: a day's copy grants nothing on its own.
fn kept_proofs(home: &Path, shard: &str, key: &str) -> Option<Vec<String>> {
    let (said_at, signature_at) = kept_at(home, shard);
    if !fresh(&said_at) {
        return None;
    }
    accepted(
        shard,
        &std::fs::read(&said_at).ok()?,
        &std::fs::read(&signature_at).ok()?,
        key,
    )
}

fn fetched_proofs(
    home: &Path,
    shard: &str,
    bring: &impl Fn(&str) -> Option<Vec<u8>>,
    key: &str,
) -> Option<Vec<String>> {
    let (said_at, signature_at) = kept_at(home, shard);
    let said = bring(&format!("{SHARDS_URL}{shard}.json"))?;
    let signature = bring(&format!("{SHARDS_URL}{shard}.json.minisig"))?;
    let held = accepted(shard, &said, &signature, key)?;
    if let Some(dir) = said_at.parent() {
        let _ = std::fs::create_dir_all(dir);
        let _ = tisty_core::store::write_atomic(&said_at, &said);
        let _ = tisty_core::store::write_atomic(&signature_at, &signature);
    }
    Some(held)
}

fn accepted(shard: &str, said: &[u8], signature: &[u8], key: &str) -> Option<Vec<String>> {
    let named = format!("domains/{shard}.json");
    policy::signed_as(said, &String::from_utf8_lossy(signature), key, &named)
        .then(|| serde_json::from_slice(said).ok())
        .flatten()
}

#[cfg(test)]
#[path = "folio_domains_test.rs"]
mod tests;
