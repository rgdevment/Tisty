use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use crate::folio_domains::a_domain;
use crate::policy;

const HOSTS_URL: &str = from_folio!("domains/hosts.json");
const SIGNATURE_URL: &str = from_folio!("domains/hosts.json.minisig");
const SIGNED_AS: &str = "domains/hosts.json";
const KEPT: &str = "folio-hosts.json";
const SIGNATURE_KEPT: &str = "folio-hosts.json.minisig";
const ASKED: &str = "folio-hosts-asked";
const ASKED_EVERY: Duration = Duration::from_secs(48 * 60 * 60);
pub const FIRST_PRIVATE: &str = "tisty.cl";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct Hosts {
    serial: u64,
    private: String,
}

fn read(said: &[u8], signature: &[u8], key: &str) -> Option<Hosts> {
    let signature = String::from_utf8_lossy(signature);
    if !policy::signed_as(said, &signature, key, SIGNED_AS) {
        return None;
    }
    serde_json::from_slice::<Hosts>(said)
        .ok()
        .filter(|hosts| a_domain(&hosts.private))
}

// Where every token goes is decided here, so even the copy kept on disk is read again under Folio's signature.
fn kept(home: &Path, key: &str) -> Option<Hosts> {
    read(
        &std::fs::read(home.join(KEPT)).ok()?,
        &std::fs::read(home.join(SIGNATURE_KEPT)).ok()?,
        key,
    )
}

fn fetched(
    home: &Path,
    bring: &impl Fn(&str) -> Option<Vec<u8>>,
    key: &str,
    kept_serial: Option<u64>,
) -> Option<Hosts> {
    let said = bring(HOSTS_URL)?;
    let signature = bring(SIGNATURE_URL)?;
    let hosts = read(&said, &signature, key)?;
    if kept_serial.is_some_and(|serial| hosts.serial < serial) {
        return None;
    }
    let _ = std::fs::create_dir_all(home);
    let _ = tisty_core::store::write_atomic(&home.join(KEPT), &said);
    let _ = tisty_core::store::write_atomic(&home.join(SIGNATURE_KEPT), &signature);
    Some(hosts)
}

pub fn private_kept(home: &Path) -> String {
    private_kept_under(home, crate::folio_limits::KEY)
}

fn private_kept_under(home: &Path, key: &str) -> String {
    kept(home, key).map_or_else(|| FIRST_PRIVATE.to_owned(), |hosts| hosts.private)
}

pub fn private(home: &Path, bring: impl Fn(&str) -> Option<Vec<u8>>) -> String {
    private_under(home, &bring, crate::folio_limits::KEY)
}

fn private_under(home: &Path, bring: &impl Fn(&str) -> Option<Vec<u8>>, key: &str) -> String {
    let held = kept(home, key);
    let brought = policy::ask_now(&home.join(ASKED), ASKED_EVERY)
        .then(|| fetched(home, bring, key, held.as_ref().map(|hosts| hosts.serial)))
        .flatten();
    brought
        .or(held)
        .map_or_else(|| FIRST_PRIVATE.to_owned(), |hosts| hosts.private)
}

#[cfg(test)]
#[path = "folio_hosts_test.rs"]
mod tests;
