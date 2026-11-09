use std::cell::Cell;
use std::path::Path;
use std::time::{Duration, SystemTime};

use super::{FIRST_PRIVATE, KEPT, MISSED, private_kept_under, private_under};

const KEY: &str = include_str!("folio_signed/hosts/key.pub");
const HOSTS: &[u8] = include_bytes!("folio_signed/hosts/hosts.json");
const SIGNED: &[u8] = include_bytes!("folio_signed/hosts/hosts.json.minisig");
const OLDER: &[u8] = include_bytes!("folio_signed/hosts/older.json");
const OLDER_SIGNED: &[u8] = include_bytes!("folio_signed/hosts/older.json.minisig");
const UNSHAPED: &[u8] = include_bytes!("folio_signed/hosts/unshaped.json");
const UNSHAPED_SIGNED: &[u8] = include_bytes!("folio_signed/hosts/unshaped.json.minisig");
const SIGNED_ELSEWHERE: &[u8] = include_bytes!("folio_signed/hosts/elsewhere.json.minisig");

fn serving(said: &'static [u8], signature: &'static [u8]) -> impl Fn(&str) -> Option<Vec<u8>> {
    move |url: &str| match url {
        from_folio!("domains/hosts.json") => Some(said.to_vec()),
        from_folio!("domains/hosts.json.minisig") => Some(signature.to_vec()),
        _ => None,
    }
}

fn nothing(_: &str) -> Option<Vec<u8>> {
    None
}

fn an_hour_later(home: &Path) {
    std::fs::remove_file(home.join(MISSED)).unwrap();
}

fn two_days_later(home: &Path) {
    let long_ago = SystemTime::now() - Duration::from_secs(49 * 60 * 60);
    std::fs::File::options()
        .write(true)
        .open(home.join(KEPT))
        .and_then(|kept| kept.set_modified(long_ago))
        .unwrap();
}

#[test]
fn with_nothing_signed_yet_the_private_host_is_the_first_one() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(
        private_under(home.path(), &nothing, KEY.trim()),
        FIRST_PRIVATE
    );
    assert_eq!(private_kept_under(home.path(), KEY.trim()), FIRST_PRIVATE);
}

#[test]
fn a_host_folio_signed_is_taken_and_kept() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(
        private_under(home.path(), &serving(HOSTS, SIGNED), KEY.trim()),
        "tisty.dev"
    );
    assert_eq!(private_kept_under(home.path(), KEY.trim()), "tisty.dev");
    assert_eq!(
        private_under(home.path(), &nothing, KEY.trim()),
        "tisty.dev"
    );
}

#[test]
fn a_copy_that_holds_is_asked_for_once_in_two_days() {
    let home = tempfile::tempdir().unwrap();
    let asked = Cell::new(0);
    let counting = |url: &str| {
        asked.set(asked.get() + 1);
        serving(HOSTS, SIGNED)(url)
    };
    private_under(home.path(), &counting, KEY.trim());
    private_under(home.path(), &counting, KEY.trim());
    assert_eq!(asked.get(), 2, "the file and its signature, once");
    two_days_later(home.path());
    an_hour_later(home.path());
    private_under(home.path(), &counting, KEY.trim());
    assert_eq!(asked.get(), 4, "asked again once the two days are over");
}

#[test]
fn a_look_that_brought_nothing_tries_again_within_the_hour_not_in_two_days() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(
        private_under(home.path(), &nothing, KEY.trim()),
        FIRST_PRIVATE
    );
    assert_eq!(
        private_under(home.path(), &serving(HOSTS, SIGNED), KEY.trim()),
        FIRST_PRIVATE
    );
    an_hour_later(home.path());
    assert_eq!(
        private_under(home.path(), &serving(HOSTS, SIGNED), KEY.trim()),
        "tisty.dev"
    );
}

#[test]
fn what_folio_did_not_sign_or_signed_as_another_file_is_refused() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(
        private_under(home.path(), &serving(HOSTS, SIGNED_ELSEWHERE), KEY.trim()),
        FIRST_PRIVATE
    );
    an_hour_later(home.path());
    assert_eq!(
        private_under(home.path(), &serving(OLDER, SIGNED), KEY.trim()),
        FIRST_PRIVATE
    );
    assert!(!home.path().join(KEPT).exists());
}

#[test]
fn a_host_that_is_not_a_bare_domain_is_refused_even_signed() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(
        private_under(home.path(), &serving(UNSHAPED, UNSHAPED_SIGNED), KEY.trim()),
        FIRST_PRIVATE
    );
}

#[test]
fn an_older_serial_never_takes_back_a_newer_host() {
    let home = tempfile::tempdir().unwrap();
    private_under(home.path(), &serving(HOSTS, SIGNED), KEY.trim());
    two_days_later(home.path());
    assert_eq!(
        private_under(home.path(), &serving(OLDER, OLDER_SIGNED), KEY.trim()),
        "tisty.dev"
    );
}

#[test]
fn a_kept_copy_changed_on_disk_counts_for_nothing() {
    let home = tempfile::tempdir().unwrap();
    private_under(home.path(), &serving(HOSTS, SIGNED), KEY.trim());
    std::fs::write(
        home.path().join(KEPT),
        br#"{"serial":9,"private":"evil.example"}"#,
    )
    .unwrap();
    assert_eq!(private_kept_under(home.path(), KEY.trim()), FIRST_PRIVATE);
}
