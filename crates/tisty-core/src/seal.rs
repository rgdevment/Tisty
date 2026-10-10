use ed25519_dalek::{SigningKey, VerifyingKey};

use crate::event::{SCHEMA_VERSION, SEALED_FROM};
use crate::signing::{hexed, unhexed};

pub const OP: &str = "seal";
const MARK: &[u8] = b"\"op\":\"seal\"";
const SIG: &[u8] = b",\"sig\":\"";
const CONTEXT: &str = "tisty.seal";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seal {
    pub seg: u32,
    pub at: u64,
    pub tip: [u8; 32],
    pub n: u64,
    pub closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Read {
    pub seal: Seal,
    signed: Vec<u8>,
    sig: [u8; 64],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    Other,
    Seal(Box<Read>),
    Broken,
}

#[derive(serde::Deserialize)]
struct Stamped {
    v: u32,
    #[serde(default)]
    op: String,
}

#[derive(serde::Deserialize)]
struct Said {
    seg: u32,
    at: u64,
    tip: String,
    n: u64,
    #[serde(default)]
    closed: bool,
    sig: String,
}

pub fn is_seal(v: u32, op: &str) -> bool {
    op == OP && v >= SEALED_FROM
}

pub fn line(key: &SigningKey, device: &str, seal: &Seal) -> String {
    use ed25519_dalek::Signer;

    let mut said = format!(
        "{{\"v\":{SCHEMA_VERSION},\"op\":\"{OP}\",\"seg\":{},\"at\":{},\"tip\":\"{}\",\"n\":{}",
        seal.seg,
        seal.at,
        hexed(&seal.tip),
        seal.n
    );
    if seal.closed {
        said.push_str(",\"closed\":true");
    }
    let sig = key.sign(&over(device, said.as_bytes()));
    said.push_str(",\"sig\":\"");
    said.push_str(&hexed(&sig.to_bytes()));
    said.push_str("\"}\n");
    said
}

fn over(device: &str, signed: &[u8]) -> Vec<u8> {
    let mut said = format!("{CONTEXT}\u{0}{device}\u{0}").into_bytes();
    said.extend_from_slice(signed);
    said
}

/// A seal answers for the exact bytes in front of it, so they are checked as they stand.
pub fn read(line: &[u8]) -> Line {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    if !line.windows(MARK.len()).any(|one| one == MARK) {
        return Line::Other;
    }
    match serde_json::from_slice::<Stamped>(line) {
        Ok(one) if is_seal(one.v, &one.op) => {}
        Ok(_) => return Line::Other,
        Err(_) => return Line::Broken,
    }
    match parsed(line) {
        Some(read) => Line::Seal(Box::new(read)),
        None => Line::Broken,
    }
}

fn parsed(line: &[u8]) -> Option<Read> {
    let said: Said = serde_json::from_slice(line).ok()?;
    let cut = line.windows(SIG.len()).rposition(|one| one == SIG)?;
    let tail = line.get(cut + SIG.len()..)?.strip_suffix(b"\"}")?;
    if tail != said.sig.as_bytes() {
        return None;
    }
    Some(Read {
        seal: Seal {
            seg: said.seg,
            at: said.at,
            tip: unhexed::<32>(&said.tip)?,
            n: said.n,
            closed: said.closed,
        },
        signed: line[..cut].to_vec(),
        sig: unhexed::<64>(&said.sig)?,
    })
}

pub fn holds(by: &VerifyingKey, device: &str, read: &Read) -> bool {
    use ed25519_dalek::Verifier;

    let sig = ed25519_dalek::Signature::from_bytes(&read.sig);
    by.verify(&over(device, &read.signed), &sig).is_ok()
}

#[cfg(test)]
#[path = "seal_test.rs"]
mod tests;
