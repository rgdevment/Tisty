use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::identity::{self, Certificate, ShareKey, Signed, Verdict};

const ABOUT: &str = "language-review";
pub const REVIEWED: &str = "reviewed";

pub struct Kept<'a> {
    pub document: &'a str,
    pub lang: &'a str,
    pub title: Option<&'a str>,
    pub key: &'a [u8],
    pub print: &'a [u8],
}

pub struct Said<'a> {
    pub anchor: &'a str,
    pub of: &'a [u8],
    pub seq: u64,
    pub kept: Option<Kept<'a>>,
}

fn sized(out: &mut Vec<u8>, said: &[u8]) {
    out.extend_from_slice(&(said.len() as u64).to_be_bytes());
    out.extend_from_slice(said);
}

fn bytes(said: &Said) -> Vec<u8> {
    let mut out = Vec::new();
    sized(&mut out, said.anchor.as_bytes());
    out.extend_from_slice(said.of);
    out.extend_from_slice(&said.seq.to_be_bytes());
    out.push(u8::from(said.kept.is_some()));
    if let Some(kept) = &said.kept {
        sized(&mut out, kept.document.as_bytes());
        sized(&mut out, kept.lang.as_bytes());
        sized(&mut out, kept.title.unwrap_or_default().as_bytes());
        out.extend_from_slice(kept.key);
        out.extend_from_slice(kept.print);
    }
    out
}

/// Keyed by the anchor's link key: an id is short enough to be guessed back from a bare hash.
pub fn of(anchor_key: &[u8], id: &str) -> Option<Vec<u8>> {
    let mut of = <Hmac<Sha256> as KeyInit>::new_from_slice(anchor_key).ok()?;
    of.update(b"tisty-language-review\0");
    of.update(id.as_bytes());
    Some(of.finalize().into_bytes().to_vec())
}

/// What a review's signature covers as a whole, which the owner's word on all of them hashes.
pub fn row(root: &[u8], machine: &[u8], said: &Said) -> Vec<u8> {
    let mut out = identity::SIGNED.to_vec();
    out.extend_from_slice(root);
    out.extend_from_slice(machine);
    sized(&mut out, ABOUT.as_bytes());
    out.extend_from_slice(&bytes(said));
    out
}

pub fn reviewed(anchor: &str, seq: u64, reviews: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    sized(&mut out, anchor.as_bytes());
    out.extend_from_slice(&seq.to_be_bytes());
    out.extend_from_slice(reviews);
    out
}

pub fn holds(said: &Said, signed: &Signed) -> bool {
    matches!(
        identity::check(signed, ABOUT, &bytes(said), &[]),
        Verdict::Holds { .. }
    )
}

pub fn signed(key: &ShareKey, certificate: &Certificate, said: &Said) -> Option<Signed> {
    key.sign(certificate, ABOUT, &bytes(said))
}
