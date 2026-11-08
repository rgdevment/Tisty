use std::cell::Cell;

use super::{a_domain, checked_under, kept_at, proof_of, shard_of};

const KEY: &str = include_str!("folio_signed/domains/key.pub");
const SHARD: &[u8] = include_bytes!("folio_signed/domains/75.json");
const SIGNED: &[u8] = include_bytes!("folio_signed/domains/75.json.minisig");
const SIGNED_ELSEWHERE: &[u8] = include_bytes!("folio_signed/domains/elsewhere.json.minisig");
const ROOT: &str = "AfnGoRnNecrr5RMnprcTbdZdk7_LABXtpqchY-L0b0s";
const DOMAIN: &str = "rgdevment.dev";

fn serving(signature: &'static [u8]) -> impl Fn(&str) -> Option<Vec<u8>> {
    move |url: &str| match url {
        _ if url.ends_with("/domains/75.json") => Some(SHARD.to_vec()),
        _ if url.ends_with("/domains/75.json.minisig") => Some(signature.to_vec()),
        _ => None,
    }
}

#[test]
fn a_proof_is_the_start_of_sha256_over_the_root_and_the_domain_in_the_shard_of_its_first_two() {
    let proof = proof_of(ROOT, DOMAIN);
    assert_eq!(proof, "7532a62348e8a81b78e532d9e81c87e0");
    assert_eq!(shard_of(&proof), "75");
}

#[test]
fn only_a_lowercase_ascii_domain_is_looked_up() {
    for good in [
        "rgdevment.dev",
        "ana.cl",
        "a-b.example.com",
        "xn--bcher-kva.example",
        "x.xn--p1ai",
    ] {
        assert!(a_domain(good), "{good}");
    }
    for bad in [
        "rgdevment",
        "Ana.dev",
        "ana..dev",
        "-ana.dev",
        "ana-.dev",
        "ana.d",
        "ana.d3v",
        "ánа.dev",
        "",
        "ana dev.cl",
    ] {
        assert!(!a_domain(bad), "{bad}");
    }
}

#[test]
fn a_domain_is_checked_when_its_proof_is_in_a_shard_folio_signed() {
    let home = tempfile::tempdir().unwrap();
    assert!(checked_under(
        home.path(),
        ROOT,
        DOMAIN,
        &serving(SIGNED),
        KEY.trim()
    ));
    assert!(!checked_under(
        home.path(),
        &"B".repeat(43),
        DOMAIN,
        &serving(SIGNED),
        KEY.trim()
    ));
    assert!(!checked_under(
        home.path(),
        ROOT,
        "otro.dev",
        &serving(SIGNED),
        KEY.trim()
    ));
}

#[test]
fn a_shard_signed_for_another_name_a_missing_one_or_another_key_checks_nothing() {
    let home = tempfile::tempdir().unwrap();
    assert!(!checked_under(
        home.path(),
        ROOT,
        DOMAIN,
        &serving(SIGNED_ELSEWHERE),
        KEY.trim()
    ));
    assert!(!checked_under(
        home.path(),
        ROOT,
        DOMAIN,
        &|_: &str| None,
        KEY.trim()
    ));
    assert!(!checked_under(
        home.path(),
        ROOT,
        DOMAIN,
        &serving(SIGNED),
        crate::folio_limits::KEY
    ));
}

#[test]
fn a_shard_is_kept_a_day_and_read_again_under_its_signature() {
    let home = tempfile::tempdir().unwrap();
    let asked = Cell::new(0);
    let counting = |url: &str| {
        asked.set(asked.get() + 1);
        serving(SIGNED)(url)
    };
    assert!(checked_under(
        home.path(),
        ROOT,
        DOMAIN,
        &counting,
        KEY.trim()
    ));
    assert!(checked_under(
        home.path(),
        ROOT,
        DOMAIN,
        &counting,
        KEY.trim()
    ));
    assert_eq!(asked.get(), 2, "the second look reads the copy kept here");

    let (said_at, _) = kept_at(home.path(), "75");
    std::fs::write(&said_at, b"[\"7532a62348e8a81b78e532d9e81c87e0\"]\n").unwrap();
    assert!(!checked_under(
        home.path(),
        ROOT,
        DOMAIN,
        &|_: &str| None,
        KEY.trim()
    ));
}
