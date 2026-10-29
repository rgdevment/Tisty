use std::net::IpAddr;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use tisty_share_proto::work::{self, Challenge, Proof};

use crate::share::{Policy, Refused};
use crate::store::Store;

const HOUR: i64 = 60 * 60;
const DAY: i64 = 24 * HOUR;
const CHALLENGE: &[u8] = b"tisty-share.challenge.v1\0";
const WHO_TODAY: &[u8] = b"tisty-share.who.day.v1\0";
const WHO: &[u8] = b"tisty-share.who.v1\0";

/// What a caller is about to do, each with its own hourly allowance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    Reserve,
    Upload,
    Read,
    Report,
}

impl Act {
    fn name(self) -> &'static str {
        match self {
            Act::Reserve => "reserve",
            Act::Upload => "upload",
            Act::Read => "read",
            Act::Report => "report",
        }
    }
}

/// How many of each a caller may do in an hour, how much it may store in a day, and how long it waits when it overdoes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allowance {
    pub reserves: u64,
    pub uploads: u64,
    pub reads: u64,
    pub reports: u64,
    pub documents_a_day: u64,
    pub bytes_a_day: u64,
    /// The first time it is too much, the second and every one after, within a week.
    pub waits: [i64; 3],
}

impl Default for Allowance {
    /// Placeholders until the numbers are measured; each server sets its own.
    fn default() -> Self {
        Self {
            reserves: 20,
            uploads: 60,
            reads: 600,
            reports: 10,
            documents_a_day: 50,
            bytes_a_day: 50 * 1024 * 1024,
            waits: [HOUR, DAY, 7 * DAY],
        }
    }
}

impl Allowance {
    fn per_hour(&self, act: Act) -> u64 {
        match act {
            Act::Reserve => self.reserves,
            Act::Upload => self.uploads,
            Act::Read => self.reads,
            Act::Report => self.reports,
        }
    }
}

/// How much work each step costs, in leading zero bits of a hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Work {
    pub reserve: u8,
    pub upload: u8,
    /// Added for every megabyte an upload carries, so a heavy one costs more than a note (decision 33).
    pub per_megabyte: u8,
    pub report: u8,
    pub lasts: i64,
}

impl Default for Work {
    fn default() -> Self {
        Self {
            reserve: 16,
            upload: 14,
            per_megabyte: 2,
            report: 18,
            lasts: 10 * 60,
        }
    }
}

impl Work {
    pub fn for_upload(&self, bytes: u64) -> u8 {
        let megabytes = bytes.div_ceil(1024 * 1024);
        let added = megabytes.saturating_mul(u64::from(self.per_megabyte));
        let bits = u64::from(self.upload).saturating_add(added);
        bits.min(u64::from(work::BITS_AT_MOST)) as u8
    }
}

/// A seed only this server could have signed, bound to what it is for, so no proof answers for something else.
pub fn challenge(
    secret: &[u8; 32],
    bound: &[u8],
    salt: [u8; 16],
    bits: u8,
    expires: i64,
) -> Challenge {
    let salt = URL_SAFE_NO_PAD.encode(salt);
    Challenge {
        seed: seed(secret, bound, &salt, bits, expires),
        salt,
        bits,
        expires,
    }
}

pub fn checked(
    secret: &[u8; 32],
    bound: &[u8],
    proof: &Proof,
    needed: u8,
    now: i64,
) -> Result<(), Refused> {
    let asked = &proof.challenge;
    let signed = seed(secret, bound, &asked.salt, asked.bits, asked.expires) == asked.seed;
    if !signed || asked.expires < now || asked.bits < needed {
        return Err(Refused::WorkRefused);
    }
    if !work::meets(asked, proof.nonce) {
        return Err(Refused::WorkRefused);
    }
    Ok(())
}

fn seed(secret: &[u8; 32], bound: &[u8], salt: &str, bits: u8, expires: i64) -> String {
    // Each part says its length, so no bound and salt run together into another pair.
    let lengths = [bound.len() as u64, salt.len() as u64].map(u64::to_be_bytes);
    let said = keyed(
        secret,
        &[
            CHALLENGE,
            &lengths[0],
            bound,
            &lengths[1],
            salt.as_bytes(),
            &[bits],
            &expires.to_be_bytes(),
        ],
    );
    URL_SAFE_NO_PAD.encode(said)
}

/// An IPv6 caller is one /64, so a machine cannot step around a limit by changing its last bits.
fn network(ip: IpAddr) -> Vec<u8> {
    match ip {
        IpAddr::V4(four) => four.octets().to_vec(),
        IpAddr::V6(six) => match six.to_ipv4_mapped() {
            Some(four) => four.octets().to_vec(),
            None => six.octets()[..8].to_vec(),
        },
    }
}

/// Today's name for a caller: counters a day apart can never be told to be the same person.
pub fn today(secret: &[u8; 32], ip: IpAddr, now: i64) -> String {
    let day = now.div_euclid(DAY);
    hex(&keyed(
        secret,
        &[WHO_TODAY, &day.to_be_bytes(), &network(ip)],
    ))
}

/// A name that holds across days, kept only while a block or a report needs it (decisions 41 and 42).
pub fn lasting(secret: &[u8; 32], ip: IpAddr) -> String {
    hex(&keyed(secret, &[WHO, &network(ip)]))
}

/// Refused while the caller waits out a block, and a step past its allowance earns the next, longer wait.
pub async fn admit<S: Store>(
    store: &S,
    policy: &Policy,
    secret: &[u8; 32],
    ip: IpAddr,
    act: Act,
    now: i64,
) -> Result<(), Refused> {
    let who = lasting(secret, ip);
    if let Some(until) = store.held(&format!("wait:{who}"), now).await? {
        return Err(Refused::Slow { until });
    }
    let hour = now.div_euclid(HOUR);
    let key = format!("rate:{}:{}:{hour}", act.name(), today(secret, ip, now));
    let done = store.count(&key, 1, (hour + 1) * HOUR, now).await?;
    if done > policy.allowance.per_hour(act) {
        return Err(overdone(store, policy, &who, now).await?);
    }
    Ok(())
}

/// What a caller stores in a day: documents it reserves and bytes it uploads.
pub async fn spend<S: Store>(
    store: &S,
    policy: &Policy,
    secret: &[u8; 32],
    ip: IpAddr,
    documents: u64,
    bytes: u64,
    now: i64,
) -> Result<(), Refused> {
    let day = now.div_euclid(DAY);
    let until = (day + 1) * DAY;
    let who = today(secret, ip, now);
    let allowance = &policy.allowance;
    let made = store
        .count(&format!("documents:{who}:{day}"), documents, until, now)
        .await?;
    let sent = store
        .count(&format!("bytes:{who}:{day}"), bytes, until, now)
        .await?;
    if made > allowance.documents_a_day || sent > allowance.bytes_a_day {
        return Err(overdone(store, policy, &lasting(secret, ip), now).await?);
    }
    Ok(())
}

async fn overdone<S: Store>(
    store: &S,
    policy: &Policy,
    who: &str,
    now: i64,
) -> Result<Refused, Refused> {
    let strikes = store
        .count(&format!("strikes:{who}"), 1, now + 7 * DAY, now)
        .await?;
    let waits = &policy.allowance.waits;
    let wait = waits[usize::try_from(strikes - 1)
        .unwrap_or(usize::MAX)
        .min(waits.len() - 1)];
    let until = now + wait;
    store.hold(&format!("wait:{who}"), until).await?;
    Ok(Refused::Slow { until })
}

/// HMAC pads a key shorter than its block with zeros, so padding it here builds one that cannot fail.
fn keyed(secret: &[u8; 32], parts: &[&[u8]]) -> [u8; 32] {
    let mut block = hmac::digest::Key::<Hmac<Sha256>>::default();
    block[..secret.len()].copy_from_slice(secret);
    let mut over = <Hmac<Sha256> as KeyInit>::new(&block);
    parts.iter().for_each(|part| over.update(part));
    over.finalize().into_bytes().into()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
#[path = "guard_test.rs"]
mod tests;
