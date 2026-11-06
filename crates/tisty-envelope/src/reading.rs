use std::collections::BTreeSet;
use std::io::Read as _;

use aes_gcm::Aes256Gcm;
use aes_gcm::Nonce;
use aes_gcm::aead::{Aead, Payload};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use flate2::read::DeflateDecoder;
use jiff::Timestamp;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use tisty_contracts::{
    EnvelopeKind, EnvelopeReader, Opening, Orientation, Paper, Served, VERSION, Verdict, check,
    check_html, pictures_in, references_in,
};
use tisty_share_proto::identity::{self, Certificate, Revocation, Signed};

use crate::sealing::{
    ABOUT, MANIFEST_AT_MOST, MARK, OWNER_ABOUT, SIGNATURE_AT_MOST, cipher_of, commitment_holds,
    name_tag, nonce, owner_said,
};

// This reader's own, and no number of the contract's: each reader knows what it can hold.
const OPENS_AT_MOST: u64 = 64 * 1024 * 1024;
const PNG_MARK: &[u8] = b"\x89PNG\r\n\x1a\n";
const JPEG_MARK: &[u8] = &[0xFF, 0xD8, 0xFF];
const FIRST_SHARED: (u64, u64) = (1, 2);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Unopened {
    #[error("it was written under a newer version: update to read it")]
    Newer,
    #[error("it was written under a version too old for this reader")]
    TooOld,
    #[error("it was written under a draft of the contract")]
    Draft,
    #[error("its signature does not let its words be shown: {0:?}")]
    Signature(Verdict),
    #[error("it is damaged: {0}")]
    Damaged(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    pub manifest: Vec<u8>,
    pub index: Vec<u8>,
    pub parts: Vec<Vec<u8>>,
    pub verdict: Verdict,
    pub signer: String,
    pub owner: Option<String>,
}

// It stays a `Value`: a later minor version adds what this one leaves aside, and a paper it does not know is no fault.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub document: Value,
    pub pictures: Vec<Vec<u8>>,
    pub verdict: Verdict,
    pub signer: String,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Laid {
    pub paper: Paper,
    pub on: bool,
    pub orientation: Orientation,
}

const fn damaged<T>(why: &'static str) -> Result<T, Unopened> {
    Err(Unopened::Damaged(why))
}

// One spelling for a number, as every reader has to agree on which versions it opens.
fn number_in(said: &str) -> Option<u64> {
    said.parse()
        .ok()
        .filter(|one: &u64| one.to_string() == said)
}

fn major_of(version: &str) -> Option<u64> {
    number_in(version.split('.').next()?)
}

fn minor_of(said: &str) -> Option<u64> {
    number_in(said.split('.').nth(1)?)
}

fn under_a_later_minor(bytes: &[u8]) -> bool {
    let size = bytes
        .get(4..8)
        .and_then(|one| <[u8; 4]>::try_from(one).ok())
        .map(|one| u32::from_be_bytes(one) as usize)
        .filter(|one| *one <= MANIFEST_AT_MOST);
    let manifest = size.and_then(|one| bytes.get(8..8 + one));
    let manifest = manifest.and_then(|one| serde_json::from_slice::<Value>(one).ok());
    let said = manifest.as_ref().and_then(|one| one["contract"].as_str());
    said.filter(|one| !one.contains('-')).is_some_and(|one| {
        major_of(one) == major_of(VERSION)
            && minor_of(one)
                .zip(minor_of(VERSION))
                .is_some_and(|(said, here)| said > here)
    })
}

// What a later minor may add, this reader cannot tell from what is broken, and it shows neither.
fn told_apart<T>(bytes: &[u8], result: Result<T, Unopened>) -> Result<T, Unopened> {
    result.map_err(|wrong| match wrong {
        Unopened::Damaged(_) | Unopened::Signature(_) if under_a_later_minor(bytes) => {
            Unopened::Newer
        }
        other => other,
    })
}

// Before anything else: a newer version may say what this one would call damaged.
fn opens_under(said: &str) -> Result<(), Unopened> {
    opens_under_this(said, VERSION)
}

fn opens_under_this(said: &str, version: &str) -> Result<(), Unopened> {
    if said.contains('-') || version.contains('-') {
        return if said == version {
            Ok(())
        } else {
            Err(Unopened::Draft)
        };
    }
    match major_of(said).zip(major_of(version)) {
        Some((major, here)) if major == here => match minor_of(said) {
            None => damaged("its version is no version"),
            Some(minor) if (major, minor) < FIRST_SHARED => Err(Unopened::TooOld),
            Some(_) => Ok(()),
        },
        Some((major, here)) if major > here => Err(Unopened::Newer),
        Some(_) => Err(Unopened::TooOld),
        None => damaged("its version is no version"),
    }
}

struct Pieces<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Pieces<'a> {
    fn take(&mut self, size: usize) -> Result<&'a [u8], Unopened> {
        let cut = Unopened::Damaged("it was cut");
        let end = self.at.checked_add(size).ok_or(cut)?;
        let piece = self.bytes.get(self.at..end).ok_or(cut)?;
        self.at = end;
        Ok(piece)
    }

    fn length(&mut self, at_most: usize, past: &'static str) -> Result<usize, Unopened> {
        let said =
            <[u8; 4]>::try_from(self.take(4)?).map_err(|_| Unopened::Damaged("it was cut"))?;
        let said = u32::from_be_bytes(said) as usize;
        if said > at_most {
            damaged(past)
        } else {
            Ok(said)
        }
    }

    fn all_taken(&self) -> bool {
        self.at == self.bytes.len()
    }
}

// What JavaScript can tell of a number: an integer, however it was written.
fn whole(said: &Value) -> Option<u64> {
    said.as_u64().or_else(|| {
        said.as_f64()
            .filter(|one| one.fract() == 0.0 && *one >= 0.0)
            .map(|one| one as u64)
    })
}

fn word_of(kind: EnvelopeKind) -> &'static str {
    match kind {
        EnvelopeKind::Main => "main",
        EnvelopeKind::Files => "files",
    }
}

fn parts_of(manifest: &Value) -> &[Value] {
    manifest["parts"].as_array().map_or(&[], Vec::as_slice)
}

fn parts_in_their_order(manifest: &Value, kind: EnvelopeKind) -> bool {
    let names: Vec<&str> = parts_of(manifest)
        .iter()
        .filter_map(|one| one["name"].as_str())
        .collect();
    match kind {
        EnvelopeKind::Main => {
            names.len() >= 2
                && names[0] == "first"
                && names[1] == "rest"
                && names[2..].iter().all(|name| *name == "picture")
        }
        EnvelopeKind::Files => names.iter().all(|name| *name == "file"),
    }
}

fn manifest_says(manifest: &Value, opening: &Opening, has_key: bool) -> Result<(), Unopened> {
    if !check("envelope", manifest).is_empty() {
        return damaged("its manifest is outside the contract");
    }
    if manifest["id"] != opening.id.as_str() || manifest["kind"] != word_of(opening.kind) {
        return damaged("it is not what was asked for");
    }
    if whole(&manifest["from"]).is_some_and(|from| Some(from) >= whole(&manifest["version"])) {
        return damaged("it starts from a version that is not earlier");
    }
    let squeezes = |part: &Value| {
        part["compressed"] != true || part["name"] == "first" || part["name"] == "rest"
    };
    if !parts_of(manifest).iter().all(squeezes) {
        return damaged("a picture or a file says it is compressed");
    }
    if !parts_in_their_order(manifest, opening.kind) {
        return damaged("its parts are not in their order");
    }
    if manifest["sealed"] != true && has_key {
        return damaged("a key answers only for what is sealed");
    }
    Ok(())
}

// Any other counts for nothing, so what names another key or says no moment is left aside before weighing.
fn weighed(root: &str, machine: &str, served: &Served) -> Vec<Revocation> {
    served
        .revocations
        .iter()
        .flatten()
        .filter(|one| one.root == root && one.machine == machine)
        .filter_map(|one| {
            Some(Revocation {
                root: one.root.clone(),
                share: one.machine.clone(),
                at: one.at.parse().ok()?,
                sig: one.signed.clone(),
            })
        })
        .collect()
}

fn verdict_on(signature: &Value, about: &str, said: &[u8], served: &Served) -> Verdict {
    let text = |name: &str| signature[name].as_str().unwrap_or_default().to_owned();
    let Ok(since) = text("since").parse::<Timestamp>() else {
        return Verdict::Unreadable;
    };
    let signed = Signed {
        certificate: Certificate {
            root: text("root"),
            share: text("machine"),
            since,
            sig: text("certificate"),
        },
        sig: text("signed"),
    };
    let revoked = weighed(&text("root"), &text("machine"), served);
    match identity::check(&signed, about, said, &revoked) {
        identity::Verdict::Holds { .. } => Verdict::Holds,
        identity::Verdict::Refused => Verdict::Refused,
        identity::Verdict::Unreadable => Verdict::Unreadable,
        identity::Verdict::Revoked { at, .. } => {
            let received = served
                .received
                .as_deref()
                .and_then(|one| one.parse::<Timestamp>().ok());
            match received {
                Some(received) if received < at => Verdict::Retired,
                _ => Verdict::Doubtful,
            }
        }
    }
}

fn said_in<'a>(value: &'a Value, name: &str) -> &'a str {
    value[name].as_str().unwrap_or_default()
}

fn worse(one: Verdict, other: Verdict) -> Verdict {
    match (one, other) {
        (Verdict::Doubtful, _) | (_, Verdict::Doubtful) => Verdict::Doubtful,
        (Verdict::Retired, _) | (_, Verdict::Retired) => Verdict::Retired,
        _ => one,
    }
}

// The one rule of who may sign a version: its owner, or whoever holds the token its owner allowed.
fn owner_answers(
    manifest: &Value,
    signature: &Value,
    said: &[u8],
    served: &Served,
) -> Result<Verdict, Unopened> {
    let owner = &manifest["owner"];
    let decoded = |name: &str| URL_SAFE_NO_PAD.decode(owner[name].as_str().unwrap_or_default());
    let permit = owner["permit"]
        .is_string()
        .then(|| decoded("permit"))
        .transpose()
        .map_err(|_| Unopened::Signature(Verdict::Unreadable))?;
    let tag = decoded("name_tag").map_err(|_| Unopened::Signature(Verdict::Unreadable))?;
    let id = manifest["id"].as_str().unwrap_or_default();
    let owned = owner_said(id, permit.as_deref(), &tag);
    let verdict = verdict_on(owner, OWNER_ABOUT, &owned, served);
    if matches!(verdict, Verdict::Refused | Verdict::Unreadable) {
        return Err(Unopened::Signature(verdict));
    }
    if said_in(owner, "root") == said_in(signature, "root") {
        return Ok(verdict);
    }
    let allowed = identity::permit_holds(
        said_in(owner, "permit"),
        said_in(signature, "permit_signed"),
        said_in(signature, "root"),
        said_in(signature, "machine"),
        said,
    );
    if allowed {
        Ok(verdict)
    } else {
        Err(Unopened::Signature(Verdict::Refused))
    }
}

fn owner_name_holds(manifest: &Value, first: &[u8], key: &[u8]) -> Result<(), Unopened> {
    let name = serde_json::from_slice::<Value>(first).unwrap_or_default();
    let name = name["owner_name"].as_str().unwrap_or_default();
    let id = manifest["id"].as_str().unwrap_or_default();
    let tag = name_tag(key, id, name).map_err(|_| Unopened::Damaged("its key is not one"))?;
    if manifest["owner"]["name_tag"].as_str() == Some(URL_SAFE_NO_PAD.encode(tag).as_str()) {
        Ok(())
    } else {
        damaged("its owner's name is not the one its owner signed")
    }
}

fn key_given(said: Option<&str>) -> Result<Option<Vec<u8>>, Unopened> {
    said.map(|one| {
        URL_SAFE_NO_PAD
            .decode(one)
            .ok()
            .filter(|key| key.len() == 32)
            .ok_or(Unopened::Damaged("its key is not one"))
    })
    .transpose()
}

fn cipher_for(manifest: &Value, key: Option<&[u8]>) -> Result<Option<Aes256Gcm>, Unopened> {
    if manifest["sealed"] != true {
        return Ok(None);
    }
    let key = key.ok_or(Unopened::Damaged("what is sealed asks for its key"))?;
    let decoded = |name: &str| URL_SAFE_NO_PAD.decode(manifest[name].as_str().unwrap_or_default());
    let head = decoded("head")
        .ok()
        .and_then(|one| <[u8; 32]>::try_from(one).ok());
    let head = head.ok_or(Unopened::Damaged("its head is not one"))?;
    let commitment = decoded("commitment").unwrap_or_default();
    if !commitment_holds(key, &head, &commitment) {
        return damaged("it was sealed with another key");
    }
    cipher_of(key, &head)
        .map(Some)
        .map_err(|_| Unopened::Damaged("its key is not one"))
}

fn inflated(squeezed: &[u8], open_size: u64) -> Result<Vec<u8>, Unopened> {
    if open_size > OPENS_AT_MOST {
        return damaged("it says more than this reader holds");
    }
    let mut out = Vec::new();
    let mut decoder = DeflateDecoder::new(squeezed);
    (&mut decoder)
        .take(open_size + 1)
        .read_to_end(&mut out)
        .map_err(|_| Unopened::Damaged("it does not inflate"))?;
    if decoder.total_in() != squeezed.len() as u64 {
        return damaged("something follows what it squeezed");
    }
    Ok(out)
}

fn part_opened(
    part: &Value,
    number: usize,
    travelled: &[u8],
    cipher: Option<&Aes256Gcm>,
    manifest: &[u8],
) -> Result<Vec<u8>, Unopened> {
    let held = match cipher {
        Some(cipher) => {
            let nonce = nonce(number).map_err(|_| Unopened::Damaged("it holds too many parts"))?;
            let payload = Payload {
                msg: travelled,
                aad: manifest,
            };
            cipher
                .decrypt(&Nonce::from(nonce), payload)
                .map_err(|_| Unopened::Damaged("it does not open"))?
        }
        None => travelled.to_vec(),
    };
    let open_size = whole(&part["open_size"]).unwrap_or_default();
    let held = if part["compressed"] == true {
        inflated(&held, open_size)?
    } else {
        held
    };
    if held.len() as u64 != open_size {
        return damaged("a part gives another size than it says");
    }
    Ok(held)
}

pub fn opened(bytes: &[u8], opening: &Opening, served: &Served) -> Result<Opened, Unopened> {
    told_apart(bytes, opened_as_said(bytes, opening, served))
}

fn opened_as_said(bytes: &[u8], opening: &Opening, served: &Served) -> Result<Opened, Unopened> {
    if bytes.get(..MARK.len()) != Some(MARK.as_slice()) {
        return damaged("it has no mark of an envelope");
    }
    let mut pieces = Pieces {
        bytes,
        at: MARK.len(),
    };
    let size = pieces.length(MANIFEST_AT_MOST, "its manifest is past its size")?;
    let manifest_bytes = pieces.take(size)?;
    let Ok(manifest) = serde_json::from_slice::<Value>(manifest_bytes) else {
        return damaged("its manifest is no JSON");
    };
    let Some(said) = manifest["contract"].as_str() else {
        return damaged("its manifest says no version");
    };
    opens_under(said)?;
    let key = key_given(opening.link_key.as_deref())?;
    manifest_says(&manifest, opening, key.is_some())?;
    let size = pieces.length(SIGNATURE_AT_MOST, "its signature is past its size")?;
    let Ok(signature) = serde_json::from_slice::<Value>(pieces.take(size)?) else {
        return damaged("its signature is no JSON");
    };
    if !check("signature", &signature).is_empty() {
        return damaged("its signature is outside the contract");
    }
    let parts = parts_of(&manifest);
    let index = pieces.take(32 * parts.len())?;
    let mut said = Sha256::digest(manifest_bytes).to_vec();
    said.extend_from_slice(index);
    let mut verdict = verdict_on(&signature, ABOUT, &said, served);
    if matches!(verdict, Verdict::Refused | Verdict::Unreadable) {
        return Err(Unopened::Signature(verdict));
    }
    let sealed = manifest["sealed"] == true;
    if sealed {
        verdict = worse(
            verdict,
            owner_answers(&manifest, &signature, &said, served)?,
        );
    }
    let cipher = cipher_for(&manifest, key.as_deref())?;
    let mut out = Vec::with_capacity(parts.len());
    for (number, part) in parts.iter().enumerate() {
        let size = whole(&part["sealed_size"]).and_then(|one| usize::try_from(one).ok());
        let travelled = pieces.take(size.ok_or(Unopened::Damaged("it says no size"))?)?;
        if Sha256::digest(travelled).as_slice() != &index[32 * number..32 * number + 32] {
            return damaged("a part is not the one signed");
        }
        out.push(part_opened(
            part,
            number,
            travelled,
            cipher.as_ref(),
            manifest_bytes,
        )?);
    }
    if !pieces.all_taken() {
        return damaged("something follows its parts");
    }
    if let (true, EnvelopeKind::Main, Some(first), Some(key)) =
        (sealed, opening.kind, out.first(), key.as_deref())
    {
        owner_name_holds(&manifest, first, key)?;
    }
    Ok(Opened {
        signer: said_in(&signature, "root").to_owned(),
        owner: manifest["owner"]["root"].as_str().map(str::to_owned),
        manifest: manifest_bytes.to_vec(),
        index: index.to_vec(),
        parts: out,
        verdict,
    })
}

fn words(part: &[u8]) -> Result<Map<String, Value>, Unopened> {
    match serde_json::from_slice(part) {
        Ok(Value::Object(fields)) => Ok(fields),
        _ => damaged("its words are not an object"),
    }
}

fn ats_of(list: &Value) -> Vec<u64> {
    list.as_array()
        .into_iter()
        .flatten()
        .filter_map(|one| whole(&one["at"]))
        .collect()
}

fn references_hold(html: &str, held: &Value) -> Result<(), Unopened> {
    if !check_html(html).is_empty() {
        return damaged("its HTML is outside the contract");
    }
    let pages = ats_of(&held["pages"]);
    let pictures = ats_of(&held["pictures"]);
    let files = ats_of(&held["files"]);
    let cited = held["cited"].as_array().map_or(0, Vec::len);
    for (kind, at) in references_in(html) {
        let there = match kind.as_str() {
            "page" => pages.contains(&at),
            "picture" => pictures.contains(&at),
            "file" => files.contains(&at),
            _ => usize::try_from(at).is_ok_and(|at| at < cited),
        };
        if !there {
            return damaged("a reference leads to what does not travel");
        }
    }
    for (at, width, height) in pictures_in(html) {
        let picture = usize::try_from(at)
            .ok()
            .and_then(|at| held["pictures"].get(at));
        let says = |name: &str, value: &str| {
            picture
                .and_then(|one| whole(&one[name]))
                .is_some_and(|number| number.to_string() == value)
        };
        if !says("width", &width) || !says("height", &height) {
            return damaged("a picture is drawn at another size than its own");
        }
    }
    Ok(())
}

fn pictures_in_place(held: &Value, of_them: &[Vec<u8>]) -> Result<(), Unopened> {
    let said = held["pictures"].as_array().map_or(&[][..], Vec::as_slice);
    if said.len() != of_them.len() {
        return damaged("it holds another number of pictures than it says");
    }
    let fit = said
        .iter()
        .zip(of_them)
        .enumerate()
        .all(|(at, (one, bytes))| {
            whole(&one["at"]) == Some(at as u64) && whole(&one["size"]) == Some(bytes.len() as u64)
        });
    if fit {
        Ok(())
    } else {
        damaged("a picture is not what it says")
    }
}

fn pages_in_place(held: &Value) -> Result<&[Value], Unopened> {
    let pages = held["pages"].as_array().map_or(&[][..], Vec::as_slice);
    let mut ids = BTreeSet::new();
    let mut last = 0;
    for page in pages {
        let at = whole(&page["at"]).unwrap_or_default();
        if page["page_of"] != held["id"] || at <= last || !ids.insert(page["id"].as_str()) {
            return damaged("a page is not in its place in this document");
        }
        last = at;
    }
    Ok(pages)
}

// All a reader looks at once it is open: only it can, and a writer's word counts for nothing.
pub fn owner_block(main: &Opened) -> Option<String> {
    let manifest: Value = serde_json::from_slice(&main.manifest).ok()?;
    manifest
        .get("owner")
        .filter(|one| !one.is_null())
        .map(Value::to_string)
}

pub fn document_in(main: &Opened) -> Result<Value, Unopened> {
    let (Some(first), Some(rest)) = (main.parts.first(), main.parts.get(1)) else {
        return damaged("it holds no words");
    };
    let mut first = words(first)?;
    let mut rest = words(rest)?;
    if first.contains_key("body")
        || first.contains_key("pages")
        || rest.keys().any(|one| first.contains_key(one))
    {
        return damaged("a part says what is not its own");
    }
    for name in ["body", "pages"] {
        if let Some(said) = rest.remove(name) {
            first.insert(name.to_owned(), said);
        }
    }
    let held = Value::Object(first);
    if !check("document", &held).is_empty() {
        return damaged("its document is outside the contract");
    }
    let manifest: Value = serde_json::from_slice(&main.manifest).unwrap_or_default();
    if held["id"] != manifest["id"] {
        return damaged("another document is inside");
    }
    if manifest["sealed"] != true && held.get("owner_name").is_some() {
        return damaged("what is public has nothing to answer for its owner's name");
    }
    pictures_in_place(&held, &main.parts[2..])?;
    let pages = pages_in_place(&held)?;
    let files = held["files"].as_array().map_or(0, Vec::len);
    if (files > 0) != held["files_hash"].is_string() {
        return damaged("files and the hash that answers for them go together");
    }
    references_hold(held["html"].as_str().unwrap_or_default(), &held)?;
    for page in pages {
        references_hold(page["html"].as_str().unwrap_or_default(), &held)?;
    }
    Ok(held)
}

pub fn reading(bytes: &[u8], opening: &Opening, served: &Served) -> Result<Reading, Unopened> {
    let main = opened(bytes, opening, served)?;
    let document = told_apart(bytes, document_in(&main))?;
    let Opened {
        parts,
        verdict,
        signer,
        owner,
        ..
    } = main;
    let pictures = parts.into_iter().skip(2).collect();
    Ok(Reading {
        document,
        pictures,
        verdict,
        signer,
        owner,
    })
}

pub fn files_for(
    document: &Value,
    bytes: &[u8],
    opening: &Opening,
    served: &Served,
) -> Result<Opened, Unopened> {
    told_apart(bytes, files_as_said(document, bytes, opening, served))
}

fn files_as_said(
    document: &Value,
    bytes: &[u8],
    opening: &Opening,
    served: &Served,
) -> Result<Opened, Unopened> {
    let files = opened(bytes, opening, served)?;
    let hash = Sha256::digest([files.manifest.as_slice(), &files.index].concat());
    if URL_SAFE_NO_PAD.encode(hash) != document["files_hash"].as_str().unwrap_or_default() {
        return damaged("it is not the envelope its document answers for");
    }
    let said = document["files"].as_array().map_or(&[][..], Vec::as_slice);
    if said.len() != files.parts.len() {
        return damaged("it holds another number of files than it says");
    }
    let fit = said
        .iter()
        .zip(&files.parts)
        .enumerate()
        .all(|(at, (one, bytes))| {
            whole(&one["at"]) == Some(at as u64)
                && whole(&one["size"]).is_none_or(|size| size == bytes.len() as u64)
        });
    if fit {
        Ok(files)
    } else {
        damaged("a file is not what it says")
    }
}

fn u16_at(bytes: &[u8], at: usize) -> Option<u64> {
    let said = <[u8; 2]>::try_from(bytes.get(at..at.checked_add(2)?)?).ok()?;
    Some(u64::from(u16::from_be_bytes(said)))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u64> {
    let said = <[u8; 4]>::try_from(bytes.get(at..at.checked_add(4)?)?).ok()?;
    Some(u64::from(u32::from_be_bytes(said)))
}

fn png_size(bytes: &[u8]) -> Option<(u64, u64)> {
    if !bytes.starts_with(PNG_MARK) {
        return None;
    }
    let mut said = None;
    let mut at = PNG_MARK.len();
    while at + 12 <= bytes.len() {
        let length = usize::try_from(u32_at(bytes, at)?).ok()?;
        let name = bytes.get(at + 4..at + 8)?;
        if at == PNG_MARK.len() && name == b"IHDR" && length >= 8 {
            said = u32_at(bytes, at + 8).zip(u32_at(bytes, at + 12));
        }
        at = at.checked_add(12)?.checked_add(length)?;
        if name == b"IEND" {
            return said.filter(|_| at == bytes.len());
        }
    }
    None
}

fn jpeg_size(bytes: &[u8]) -> Option<(u64, u64)> {
    if !bytes.starts_with(JPEG_MARK) {
        return None;
    }
    let mut said = None;
    let mut at = 2;
    while at + 1 < bytes.len() {
        let mark = bytes[at + 1];
        at += match (bytes[at], mark) {
            (0xFF, 0xD9) => return said.filter(|_| at + 2 == bytes.len()),
            (0xFF, 0xFF) => 1,
            (0xFF, 0x00 | 0x01 | 0xD0..=0xD8) => 2,
            (0xFF, _) => {
                if (0xC0..=0xCF).contains(&mark) && ![0xC4, 0xC8, 0xCC].contains(&mark) {
                    said = u16_at(bytes, at + 7).zip(u16_at(bytes, at + 5));
                }
                2 + usize::try_from(u16_at(bytes, at + 2)?).ok()?
            }
            _ => 1,
        };
    }
    None
}

pub fn picture_holds(bytes: &[u8], said: &Value) -> bool {
    let size = if said["kind"] == "image/png" {
        png_size(bytes)
    } else {
        jpeg_size(bytes)
    };
    whole(&said["size"]) == Some(bytes.len() as u64)
        && size.is_some_and(|(wide, tall)| {
            whole(&said["width"]) == Some(wide) && whole(&said["height"]) == Some(tall)
        })
}

pub fn laid(document: &Value) -> Laid {
    let sheet = &document["sheet"];
    Laid {
        paper: serde_json::from_value(sheet["paper"].clone()).unwrap_or(Paper::A4),
        on: sheet["on"] == true,
        orientation: serde_json::from_value(sheet["orientation"].clone())
            .unwrap_or(Orientation::Portrait),
    }
}

pub struct Reader {
    pub served: Served,
}

impl EnvelopeReader for Reader {
    fn open(&mut self, file: Vec<u8>, opening: Opening) -> Result<Vec<Vec<u8>>, String> {
        let open = opened(&file, &opening, &self.served).map_err(|wrong| wrong.to_string())?;
        if opening.kind == EnvelopeKind::Main {
            told_apart(&file, document_in(&open)).map_err(|wrong| wrong.to_string())?;
        }
        Ok(open.parts)
    }
}

#[cfg(test)]
#[path = "reading_test.rs"]
mod tests;
