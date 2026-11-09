use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::folio_branch;

const SHARDS_URL: &str = from_folio!("domains/");
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
    folio_branch::kept_at(&home.join(KEPT), &format!("{shard}.json"))
}

fn named(shard: &str) -> String {
    format!("domains/{shard}.json")
}

fn proofs(said: &[u8]) -> Option<Vec<String>> {
    serde_json::from_slice(said).ok()
}

fn kept_proofs(home: &Path, shard: &str, key: &str) -> Option<Vec<String>> {
    let (said_at, signature_at) = kept_at(home, shard);
    if !folio_branch::fresh(&said_at, KEPT_FOR) {
        return None;
    }
    proofs(&folio_branch::kept(
        &said_at,
        &signature_at,
        &named(shard),
        key,
    )?)
}

fn fetched_proofs(
    home: &Path,
    shard: &str,
    bring: &impl Fn(&str) -> Option<Vec<u8>>,
    key: &str,
) -> Option<Vec<String>> {
    let (said_at, signature_at) = kept_at(home, shard);
    let signed = folio_branch::brought(
        bring,
        &format!("{SHARDS_URL}{shard}.json"),
        &named(shard),
        key,
    )?;
    let held = proofs(&signed.said)?;
    folio_branch::keep(&said_at, &signature_at, &signed);
    Some(held)
}

#[cfg(test)]
#[path = "folio_domains_test.rs"]
mod tests;
