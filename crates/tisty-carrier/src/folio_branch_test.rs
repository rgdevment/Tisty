use std::cell::Cell;

use super::{brought, keep, kept, kept_at};

const KEY: &str = include_str!("folio_signed/hosts/key.pub");
const HOSTS: &[u8] = include_bytes!("folio_signed/hosts/hosts.json");
const SIGNED: &[u8] = include_bytes!("folio_signed/hosts/hosts.json.minisig");
const OLDER_SIGNED: &[u8] = include_bytes!("folio_signed/hosts/older.json.minisig");
const URL: &str = "https://example.test/domains/hosts.json";
const NAMED: &str = "domains/hosts.json";

#[test]
fn a_signature_caught_mid_publish_is_asked_for_once_more() {
    let signatures = Cell::new(0);
    let serving = |url: &str| match url {
        URL => Some(HOSTS.to_vec()),
        _ => {
            signatures.set(signatures.get() + 1);
            Some(
                if signatures.get() == 1 {
                    OLDER_SIGNED
                } else {
                    SIGNED
                }
                .to_vec(),
            )
        }
    };
    let signed = brought(&serving, URL, NAMED, KEY.trim()).expect("the second pair holds");
    assert_eq!(signed.said, HOSTS);
    assert_eq!(signatures.get(), 2);
}

#[test]
fn a_pair_that_never_holds_is_asked_for_twice_and_refused() {
    let asked = Cell::new(0);
    let serving = |url: &str| {
        asked.set(asked.get() + 1);
        Some(if url == URL { HOSTS } else { OLDER_SIGNED }.to_vec())
    };
    assert!(brought(&serving, URL, NAMED, KEY.trim()).is_none());
    assert_eq!(asked.get(), 4);
}

#[test]
fn nothing_reached_is_not_asked_for_again() {
    let asked = Cell::new(0);
    let serving = |_: &str| {
        asked.set(asked.get() + 1);
        None
    };
    assert!(brought(&serving, URL, NAMED, KEY.trim()).is_none());
    assert_eq!(asked.get(), 1);
}

#[test]
fn what_is_kept_holds_only_under_its_name() {
    let home = tempfile::tempdir().unwrap();
    let (said_at, signature_at) = kept_at(home.path(), "hosts.json");
    let serving = |url: &str| Some(if url == URL { HOSTS } else { SIGNED }.to_vec());
    keep(
        &said_at,
        &signature_at,
        &brought(&serving, URL, NAMED, KEY.trim()).unwrap(),
    );
    assert_eq!(
        kept(&said_at, &signature_at, NAMED, KEY.trim()).as_deref(),
        Some(HOSTS)
    );
    assert!(kept(&said_at, &signature_at, "domains/75.json", KEY.trim()).is_none());
}
