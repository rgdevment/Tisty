use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use crate::folio_branch;
use crate::folio_domains::a_domain;
use crate::policy;

const HOSTS_URL: &str = from_folio!("domains/hosts.json");
const NAMED: &str = "domains/hosts.json";
const KEPT: &str = "folio-hosts.json";
const KEPT_FOR: Duration = Duration::from_secs(48 * 60 * 60);
const MISSED: &str = "folio-hosts-missed";
const MISSED_EVERY: Duration = Duration::from_secs(60 * 60);
pub const FIRST_PRIVATE: &str = "tisty.cl";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct Hosts {
    serial: u64,
    private: String,
}

fn read(said: &[u8]) -> Option<Hosts> {
    serde_json::from_slice::<Hosts>(said)
        .ok()
        .filter(|hosts| a_domain(&hosts.private))
}

fn kept(home: &Path, key: &str) -> Option<Hosts> {
    let (said_at, signature_at) = folio_branch::kept_at(home, KEPT);
    read(&folio_branch::kept(&said_at, &signature_at, NAMED, key)?)
}

fn fetched(
    home: &Path,
    bring: &impl Fn(&str) -> Option<Vec<u8>>,
    key: &str,
    kept_serial: Option<u64>,
) -> Option<Hosts> {
    let signed = folio_branch::brought(bring, HOSTS_URL, NAMED, key)?;
    let hosts = read(&signed.said)?;
    if kept_serial.is_some_and(|serial| hosts.serial < serial) {
        return None;
    }
    let (said_at, signature_at) = folio_branch::kept_at(home, KEPT);
    folio_branch::keep(&said_at, &signature_at, &signed);
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

// A copy that holds is trusted for two days, while a look that brought nothing tries again within the hour.
fn private_under(home: &Path, bring: &impl Fn(&str) -> Option<Vec<u8>>, key: &str) -> String {
    let held = kept(home, key);
    let (said_at, _) = folio_branch::kept_at(home, KEPT);
    let settled = held.is_some() && folio_branch::fresh(&said_at, KEPT_FOR);
    let brought = (!settled && policy::ask_now(&home.join(MISSED), MISSED_EVERY))
        .then(|| fetched(home, bring, key, held.as_ref().map(|hosts| hosts.serial)))
        .flatten();
    brought
        .or(held)
        .map_or_else(|| FIRST_PRIVATE.to_owned(), |hosts| hosts.private)
}

#[cfg(test)]
#[path = "folio_hosts_test.rs"]
mod tests;
