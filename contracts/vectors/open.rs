//! Opens the sample envelopes the way Tisty must read what it sealed.

use std::io::Read;

use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit as _, Nonce};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, VerifyingKey};
use flate2::read::DeflateDecoder;
use hkdf::Hkdf;
use hmac::{Hmac, KeyInit as _, Mac};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use tisty_contracts::{VERSION, check, check_html, pictures_in, references_in};

const VECTORS: &str = include_str!("../vectors.json");

// This reader's own, and no number of the contract's: each reader knows what it can hold.
const OPENS_AT_MOST: u64 = 64 * 1024 * 1024;
const FIRST_SHARED: (&str, u64) = ("1", 2);

pub struct Opened {
    pub manifest: Vec<u8>,
    pub index: Vec<u8>,
    pub parts: Vec<Vec<u8>>,
}

fn b64(said: &Value) -> Vec<u8> {
    URL_SAFE_NO_PAD.decode(said.as_str().unwrap()).unwrap()
}

fn hex(said: &Value) -> Vec<u8> {
    let said = said.as_str().unwrap();
    (0..said.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&said[at..at + 2], 16).unwrap())
        .collect()
}

fn verified(key: &[u8], signature: &[u8], said: &[u8]) -> Result<(), String> {
    let key = VerifyingKey::from_bytes(key.try_into().map_err(|_| "a key")?).map_err(|_| "a key")?;
    let signature = Signature::from_slice(signature).map_err(|_| "a signature")?;
    key.verify_strict(said, &signature).map_err(|_| "the signature does not hold".to_owned())
}

// What JavaScript can tell of a number: an integer, however it was written.
fn whole(said: &Value) -> Option<u64> {
    said.as_u64().or_else(|| said.as_f64().filter(|one| one.fract() == 0.0).map(|one| one as u64))
}

fn seconds(said: &str) -> i64 {
    let number = |at: std::ops::Range<usize>| said[at].parse::<i64>().unwrap();
    let (year, month, day) = (number(0..4), number(5..7), number(8..10));
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let of_era = year.rem_euclid(400);
    let of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let days = era * 146_097 + of_era * 365 + of_era / 4 - of_era / 100 + of_year - 719_468;
    days * 86_400 + number(11..13) * 3_600 + number(14..16) * 60 + number(17..19)
}

const MAIN: (&str, &str) = ("k7Qm2xPa", "main");
const FILES: (&str, &str) = ("k7Qm2xPa", "files");
const PUBLIC: (&str, &str) = ("wordsOnly", "main");

fn opens_here(said: &str) -> bool {
    if said.contains('-') || VERSION.contains('-') {
        return said == VERSION;
    }
    let major = |version: &'static str| version.split('.').next().unwrap_or_default();
    let said_major = said.split('.').next().unwrap_or_default();
    said_major == major(VERSION) && minor_of(said).is_some_and(|minor| said_major != FIRST_SHARED.0 || minor >= FIRST_SHARED.1)
}

fn minor_of(said: &str) -> Option<u64> {
    let written = said.split('.').nth(1)?;
    written.parse().ok().filter(|one: &u64| one.to_string() == written)
}

fn under_a_later_minor(bytes: &[u8]) -> bool {
    let Some(size) = bytes.get(4..8).map(|one| u32::from_be_bytes(one.try_into().unwrap()) as usize) else {
        return false;
    };
    let Some(manifest) = bytes.get(8..8 + size).and_then(|one| serde_json::from_slice::<Value>(one).ok()) else {
        return false;
    };
    let said = manifest["contract"].as_str().unwrap_or_default();
    let major = |version: &str| version.split('.').next().unwrap_or_default().to_owned();
    !said.contains('-') && major(said) == major(VERSION) && minor_of(said) > minor_of(VERSION)
}

fn told_apart<T>(bytes: &[u8], result: Result<T, String>) -> Result<T, String> {
    result.map_err(|wrong| if under_a_later_minor(bytes) { "newer: update to read it".to_owned() } else { wrong })
}

// The one rule of who may sign a version: its owner, or whoever holds the token its owner allowed.
fn owner_answers(manifest: &Value, signature: &Value, manifest_bytes: &[u8], index: &[u8]) -> Result<(), String> {
    let owner = &manifest["owner"];
    let root = b64(&owner["root"]);
    let machine = b64(&owner["machine"]);
    let since = seconds(owner["since"].as_str().unwrap());
    let certified = [b"tisty-share.certificate.v1\0".as_slice(), &root, &machine, &since.to_be_bytes()].concat();
    verified(&root, &b64(&owner["certificate"]), &certified)?;
    let id = manifest["id"].as_str().unwrap().as_bytes();
    let permit = if owner["permit"].is_string() { b64(&owner["permit"]) } else { vec![0; 32] };
    let about = b"owner";
    let said = [
        b"tisty-share.signed.v1\0".as_slice(),
        &root,
        &machine,
        &(about.len() as u64).to_be_bytes(),
        about,
        &(id.len() as u64).to_be_bytes(),
        id,
        &permit,
        &b64(&owner["name_tag"]),
    ]
    .concat();
    verified(&machine, &b64(&owner["signed"]), &said)?;
    if owner["root"] == signature["root"] {
        return Ok(());
    }
    if !owner["permit"].is_string() || !signature["permit_signed"].is_string() {
        return Err("another root needs its owner's permit".into());
    }
    let hash = Sha256::digest(manifest_bytes);
    let said = [
        b"tisty-share.permit.v1\0".as_slice(),
        &b64(&signature["root"]),
        &b64(&signature["machine"]),
        &hash,
        index,
    ]
    .concat();
    verified(&permit, &b64(&signature["permit_signed"]), &said)
}

fn owner_name_holds(manifest: &Value, first: &[u8], link_key: &[u8]) -> Result<(), String> {
    let id = manifest["id"].as_str().unwrap().as_bytes();
    let first: Value = serde_json::from_slice(first).map_err(|_| "its words")?;
    let mut tag = Hmac::<Sha256>::new_from_slice(link_key).unwrap();
    tag.update(b"tisty-owner-name\0");
    tag.update(&(id.len() as u64).to_be_bytes());
    tag.update(id);
    tag.update(first["owner_name"].as_str().unwrap_or_default().as_bytes());
    if tag.finalize().into_bytes().as_slice() == b64(&manifest["owner"]["name_tag"]) {
        Ok(())
    } else {
        Err("its owner's name is not the one its owner signed".into())
    }
}

pub fn opened(bytes: &[u8], asked: (&str, &str), link_key: Option<&[u8]>) -> Result<Opened, String> {
    told_apart(bytes, opened_as_said(bytes, asked, link_key))
}

fn opened_as_said(bytes: &[u8], asked: (&str, &str), link_key: Option<&[u8]>) -> Result<Opened, String> {
    if bytes.len() < 4 || &bytes[..4] != b"TENV" {
        return Err("its mark".into());
    }
    let mut at = 4;
    let mut piece = |size: usize| -> Result<&[u8], String> {
        let out = bytes.get(at..at + size).ok_or("it was cut")?;
        at += size;
        Ok(out)
    };
    let size = u32::from_be_bytes(piece(4)?.try_into().unwrap()) as usize;
    if size > 16384 {
        return Err("its manifest is past its size".into());
    }
    let manifest_bytes = piece(size)?;
    let manifest: Value = serde_json::from_slice(manifest_bytes).map_err(|_| "its manifest")?;
    // Before anything else: a newer version may say what this one would call damaged.
    if !manifest["contract"].as_str().is_some_and(opens_here) {
        return Err("written under a version this reader does not open".into());
    }
    if !check("envelope", &manifest).is_empty() {
        return Err("its manifest".into());
    }
    if manifest["id"] != asked.0 || manifest["kind"] != asked.1 {
        return Err("not what it asked for".into());
    }
    if whole(&manifest["from"]).is_some_and(|from| Some(from) >= whole(&manifest["version"])) {
        return Err("it starts from a version that is not earlier".into());
    }
    let squeezes = |part: &Value| part["compressed"] != true || part["name"] == "first" || part["name"] == "rest";
    if !manifest["parts"].as_array().unwrap().iter().all(squeezes) {
        return Err("a picture or a file is never compressed".into());
    }
    let names: Vec<&str> = manifest["parts"].as_array().unwrap().iter().map(|part| part["name"].as_str().unwrap()).collect();
    let in_order = match asked.1 {
        "main" => names.len() >= 2 && names[0] == "first" && names[1] == "rest" && names[2..].iter().all(|name| *name == "picture"),
        _ => names.iter().all(|name| *name == "file"),
    };
    if !in_order {
        return Err("its parts are not in their order".into());
    }
    if manifest["sealed"] != true && link_key.is_some() {
        return Err("a key answers only for what is sealed".into());
    }
    let size = u32::from_be_bytes(piece(4)?.try_into().unwrap()) as usize;
    if size > 2048 {
        return Err("its signature is past its size".into());
    }
    let signature: Value = serde_json::from_slice(piece(size)?).map_err(|_| "its signature")?;
    if !check("signature", &signature).is_empty() {
        return Err("its signature".into());
    }
    let parts = manifest["parts"].as_array().unwrap();
    let index = piece(32 * parts.len())?;

    let root = b64(&signature["root"]);
    let machine = b64(&signature["machine"]);
    let since = seconds(signature["since"].as_str().unwrap());
    let certified = [b"tisty-share.certificate.v1\0".as_slice(), &root, &machine, &since.to_be_bytes()].concat();
    verified(&root, &b64(&signature["certificate"]), &certified)?;
    let about = b"envelope";
    let manifest_hash = Sha256::digest(manifest_bytes);
    let said = [
        b"tisty-share.signed.v1\0".as_slice(),
        &root,
        &machine,
        &(about.len() as u64).to_be_bytes(),
        about,
        &manifest_hash,
        index,
    ]
    .concat();
    verified(&machine, &b64(&signature["signed"]), &said)?;

    let sealed = manifest["sealed"] == true;
    if sealed {
        owner_answers(&manifest, &signature, manifest_bytes, index)?;
    }
    let cipher = if sealed {
        let link_key = link_key.ok_or("no key")?;
        let head = b64(&manifest["head"]);
        let mut commitment = Hmac::<Sha256>::new_from_slice(link_key).unwrap();
        commitment.update(b"tisty-envelope-commit");
        commitment.update(&head);
        if commitment.finalize().into_bytes().as_slice() != b64(&manifest["commitment"]) {
            return Err("another key".into());
        }
        let mut key = [0u8; 32];
        Hkdf::<Sha256>::new(Some(&head), link_key)
            .expand(b"tisty-envelope-key", &mut key)
            .unwrap();
        Some(Aes256Gcm::new_from_slice(&key).unwrap())
    } else {
        None
    };

    let mut out = Vec::new();
    for (number, part) in parts.iter().enumerate() {
        let travelled = piece(whole(&part["sealed_size"]).unwrap() as usize)?;
        if Sha256::digest(travelled).as_slice() != &index[32 * number..32 * number + 32] {
            return Err("its hash".into());
        }
        let held = match &cipher {
            Some(cipher) => {
                let mut nonce = [0u8; 12];
                nonce[8..].copy_from_slice(&(number as u32).to_be_bytes());
                let payload = Payload { msg: travelled, aad: manifest_bytes };
                cipher.decrypt(Nonce::from_slice(&nonce), payload).map_err(|_| "it does not open")?
            }
            None => travelled.to_vec(),
        };
        let open_size = whole(&part["open_size"]).unwrap();
        let held = if part["compressed"] == true {
            if open_size > OPENS_AT_MOST {
                return Err("it says more than this reader holds".into());
            }
            let mut inflated = Vec::new();
            let mut decoder = DeflateDecoder::new(held.as_slice());
            (&mut decoder).take(open_size + 1).read_to_end(&mut inflated).map_err(|_| "it does not inflate")?;
            if decoder.total_in() != held.len() as u64 {
                return Err("something follows what it squeezed".into());
            }
            inflated
        } else {
            held
        };
        if held.len() as u64 != open_size {
            return Err("it gives another size".into());
        }
        out.push(held);
    }
    if piece(1).is_ok() {
        return Err("something follows its parts".into());
    }
    if sealed && asked.1 == "main" {
        owner_name_holds(&manifest, &out[0], link_key.ok_or("no key")?)?;
    }
    Ok(Opened { manifest: manifest_bytes.to_vec(), index: index.to_vec(), parts: out })
}

fn ats_of(list: &Value) -> Vec<u64> {
    list.as_array().into_iter().flatten().filter_map(|one| whole(&one["at"])).collect()
}

fn references_hold(html: &str, held: &Value) -> Result<(), String> {
    if !check_html(html).is_empty() {
        return Err("its HTML is outside the contract".into());
    }
    for (kind, at) in references_in(html) {
        let there = match kind.as_str() {
            "page" => ats_of(&held["pages"]).contains(&at),
            "picture" => ats_of(&held["pictures"]).contains(&at),
            "file" => ats_of(&held["files"]).contains(&at),
            _ => (at as usize) < held["cited"].as_array().map_or(0, Vec::len),
        };
        if !there {
            return Err(format!("tisty:{kind}/{at} is not there"));
        }
    }
    for (at, width, height) in pictures_in(html) {
        let picture = &held["pictures"][at as usize];
        let says = |name: &str, value: &str| whole(&picture[name]).is_some_and(|number| format!("{number}") == value);
        if !says("width", &width) || !says("height", &height) {
            return Err("a picture drawn at another size than its own".into());
        }
    }
    Ok(())
}

// All a reader looks at once it is open: only it can, and a writer's word counts for nothing.
pub fn document_in(main: &Opened) -> Result<Value, String> {
    let words = |part: &[u8]| -> Result<Map<String, Value>, String> {
        match serde_json::from_slice(part) {
            Ok(Value::Object(fields)) => Ok(fields),
            _ => Err("its words are not an object".to_owned()),
        }
    };
    let first = words(&main.parts[0])?;
    let rest = words(&main.parts[1])?;
    if first.contains_key("body") || first.contains_key("pages") || rest.keys().any(|one| first.contains_key(one)) {
        return Err("a part says what is not its own".into());
    }
    let held = document_of(main);
    let manifest: Value = serde_json::from_slice(&main.manifest).map_err(|_| "its manifest")?;
    if !check("document", &held).is_empty() {
        return Err("its document is outside the contract".into());
    }
    if held["id"] != manifest["id"] {
        return Err("another document inside".into());
    }
    if manifest["sealed"] != true && held.get("owner_name").is_some() {
        return Err("what is public has nothing to answer for its owner's name".into());
    }
    let empty = Vec::new();
    let pictures = held["pictures"].as_array().unwrap_or(&empty);
    if pictures.len() != main.parts.len() - 2 {
        return Err("another number of pictures than it says".into());
    }
    for (at, one) in pictures.iter().enumerate() {
        if whole(&one["at"]) != Some(at as u64) || whole(&one["size"]) != Some(main.parts[at + 2].len() as u64) {
            return Err("a picture is not what it says".into());
        }
    }
    let pages = held["pages"].as_array().unwrap_or(&empty);
    let mut ids = std::collections::BTreeSet::new();
    let mut last = 0;
    for page in pages {
        let at = whole(&page["at"]).unwrap_or_default();
        if page["page_of"] != held["id"] || at <= last || !ids.insert(page["id"].as_str()) {
            return Err("a page is not in its place in this document".into());
        }
        last = at;
    }
    let files = held["files"].as_array().map_or(0, Vec::len);
    if (files > 0) != held["files_hash"].is_string() {
        return Err("files and the hash that answers for them go together".into());
    }
    references_hold(held["html"].as_str().unwrap_or_default(), &held)?;
    for page in pages {
        references_hold(page["html"].as_str().unwrap_or_default(), &held)?;
    }
    Ok(held)
}

pub fn reading(bytes: &[u8], asked: (&str, &str), link_key: Option<&[u8]>) -> Result<(Opened, Value), String> {
    let main = opened(bytes, asked, link_key)?;
    let held = told_apart(bytes, document_in(&main))?;
    Ok((main, held))
}

pub fn files_for(held: &Value, bytes: &[u8], asked: (&str, &str), link_key: Option<&[u8]>) -> Result<Opened, String> {
    told_apart(bytes, files_as_said(held, bytes, asked, link_key))
}

fn files_as_said(held: &Value, bytes: &[u8], asked: (&str, &str), link_key: Option<&[u8]>) -> Result<Opened, String> {
    let files = opened(bytes, asked, link_key)?;
    let hash = Sha256::digest([files.manifest.as_slice(), &files.index].concat());
    if URL_SAFE_NO_PAD.encode(hash) != held["files_hash"].as_str().unwrap_or_default() {
        return Err("not the envelope its document answers for".into());
    }
    let said = held["files"].as_array().map_or(&[][..], Vec::as_slice);
    if said.len() != files.parts.len() {
        return Err("another number of files than it says".into());
    }
    for (at, one) in said.iter().enumerate() {
        if whole(&one["at"]) != Some(at as u64) || whole(&one["size"]).is_some_and(|size| size != files.parts[at].len() as u64) {
            return Err("a file is not what it says".into());
        }
    }
    Ok(files)
}

fn u16_at(bytes: &[u8], at: usize) -> Option<u64> {
    Some(u64::from(u16::from_be_bytes(bytes.get(at..at + 2)?.try_into().ok()?)))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u64> {
    Some(u64::from(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?)))
}

// Whether the bytes of a PNG hold what it says, up to its last one.
fn png_size(bytes: &[u8]) -> Option<(u64, u64)> {
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return None;
    }
    let mut said = None;
    let mut at = 8;
    while at + 12 <= bytes.len() {
        let length = u32_at(bytes, at)? as usize;
        let name = &bytes[at + 4..at + 8];
        if at == 8 && name == b"IHDR" && length >= 8 {
            said = Some((u32_at(bytes, at + 8)?, u32_at(bytes, at + 12)?));
        }
        at += 12 + length;
        if name == b"IEND" {
            return said.filter(|_| at == bytes.len());
        }
    }
    None
}

fn jpeg_size(bytes: &[u8]) -> Option<(u64, u64)> {
    if !bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
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
                2 + u16_at(bytes, at + 2)? as usize
            }
            _ => 1,
        };
    }
    None
}

// What a reader looks at before it draws one. A picture that fails shows its alt, and the words still read.
pub fn picture_holds(bytes: &[u8], said: &Value) -> bool {
    let size = if said["kind"] == "image/png" { png_size(bytes) } else { jpeg_size(bytes) };
    said["size"] == bytes.len() as u64 && size.is_some_and(|(wide, tall)| said["width"] == wide && said["height"] == tall)
}

// The first two parts of a main envelope are its words; what follows them is pictures.
pub fn document_of(opened: &Opened) -> Value {
    let fields = |part: &[u8]| -> Map<String, Value> {
        let Value::Object(fields) = serde_json::from_slice(part).unwrap() else {
            panic!("the words of a main envelope are an object");
        };
        fields
    };
    let mut out = fields(&opened.parts[0]);
    let mut rest = fields(&opened.parts[1]);
    for name in ["body", "pages"] {
        if let Some(said) = rest.remove(name) {
            out.insert(name.to_owned(), said);
        }
    }
    Value::Object(out)
}

// What a reader says of a signature that holds, from what the server serves beside it. Whole seconds in one form sort as they read.
fn verdict_of(signed_by: &Value, served: &Value) -> &'static str {
    let root = signed_by["root"].as_str().unwrap();
    let machine = signed_by["machine"].as_str().unwrap();
    let earliest = served["revocations"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|one| one["root"] == root && one["machine"] == machine)
        .filter(|one| {
            let at = seconds(one["at"].as_str().unwrap());
            let said = [b"tisty-share.revocation.v1\0".as_slice(), &b64(&one["root"]), &b64(&one["machine"]), &at.to_be_bytes()].concat();
            verified(&b64(&one["root"]), &b64(&one["signed"]), &said).is_ok()
        })
        .filter_map(|one| one["at"].as_str())
        .min();
    match (earliest, served["received"].as_str()) {
        (None, _) => "holds",
        (Some(at), Some(received)) if received < at => "retired",
        _ => "doubtful",
    }
}

fn vectors() -> Value {
    serde_json::from_str(VECTORS).unwrap()
}

#[test]
fn a_main_envelope_gives_its_document_its_pages_and_its_pictures() {
    let all = vectors();
    let (main, held) = reading(&hex(&all["main"]["envelope"]), MAIN, Some(&b64(&all["link_key"]))).unwrap();
    assert_eq!(held, all["main"]["document"]);
    assert!(picture_holds(&main.parts[2], &held["pictures"][0]));
    let pictures: Vec<Vec<u8>> = all["main"]["pictures"].as_array().unwrap().iter().map(hex).collect();
    assert_eq!(main.parts[2..], pictures);
}

#[test]
fn a_files_envelope_gives_its_files_and_the_main_envelope_answers_for_it() {
    let all = vectors();
    let key = b64(&all["link_key"]);
    let held = files_for(&all["main"]["document"], &hex(&all["files"]["envelope"]), FILES, Some(&key)).unwrap();
    let files: Vec<Vec<u8>> = all["files"]["files"].as_array().unwrap().iter().map(hex).collect();
    assert_eq!(held.parts, files);
}

#[test]
fn a_public_envelope_opens_without_a_key() {
    let all = vectors();
    let (_, held) = reading(&hex(&all["public"]["envelope"]), PUBLIC, None).unwrap();
    assert_eq!(held, all["public"]["document"]);
}

#[test]
fn words_are_counted_in_bytes_as_a_machine_holds_them() {
    let said = |body: String| serde_json::json!({ "id": "x", "html": "<p>x</p>", "body": body });
    assert_eq!(check("document", &said("a".repeat(512000))), Vec::<String>::new());
    assert!(!check("document", &said("€".repeat(200000))).is_empty());
}

#[test]
fn a_revocation_is_signed_by_the_root() {
    let all = vectors();
    let said = &all["revocation"];
    assert_eq!(check("revocation", said), Vec::<String>::new());
    let root = b64(&said["root"]);
    let at = seconds(said["at"].as_str().unwrap());
    let signed = [b"tisty-share.revocation.v1\0".as_slice(), &root, &b64(&said["machine"]), &at.to_be_bytes()].concat();
    verified(&root, &b64(&said["signed"]), &signed).unwrap();
}

#[test]
fn only_its_owner_or_who_holds_its_token_signs_a_version() {
    let all = vectors();
    let editing = &all["editing"];
    let id = editing["id"].as_str().unwrap();
    let key = b64(&all["link_key"]);
    for one in editing["opens"].as_array().unwrap() {
        let bytes = hex(&one["envelope"]);
        assert!(reading(&bytes, (id, "main"), Some(&key)).is_ok(), "{}", one["says"]);
    }
    for one in editing["refused"].as_array().unwrap() {
        let bytes = hex(&one["envelope"]);
        assert!(reading(&bytes, (id, "main"), Some(&key)).is_err(), "{}", one["says"]);
    }
}

#[test]
fn what_a_rest_says_beyond_its_body_and_pages_is_left_aside() {
    let all = vectors();
    let id = all["editing"]["id"].as_str().unwrap();
    let bytes = hex(&all["editing"]["rest_left_aside"]);
    let (_, held) = reading(&bytes, (id, "main"), Some(&b64(&all["link_key"]))).unwrap();
    assert!(held.get("owner_name").is_none());
    assert!(held.get("title").is_none());
    assert_eq!(held["body"], "a");
}

#[test]
fn under_a_later_minor_files_that_do_not_fit_read_as_newer() {
    let all = vectors();
    let said = &all["newer_files"];
    let wrong = files_for(&said["document"], &hex(&said["files"]), ("hostile", "files"), None).err().unwrap();
    assert!(wrong.starts_with("newer"), "{wrong}");
}

#[test]
fn under_a_later_minor_what_it_would_refuse_reads_as_newer() {
    let all = vectors();
    for one in all["newer_than_this"].as_array().unwrap() {
        let wrong = reading(&hex(&one["envelope"]), ("hostile", "main"), None).err().unwrap();
        assert!(wrong.starts_with("newer"), "{}: {wrong}", one["says"]);
    }
}

#[test]
fn what_comes_before_the_first_version_shared_is_too_old_never_damaged() {
    let all = vectors();
    let said = &all["before_the_first_shared"];
    let bytes = hex(&said["envelope"]);
    let wrong = reading(&bytes, (said["id"].as_str().unwrap(), "main"), Some(&b64(&all["link_key"]))).err().unwrap();
    assert!(wrong.contains("version"), "{wrong}");
}

#[test]
fn no_key_of_small_order_passes() {
    let all = vectors();
    for one in all["small_order"].as_array().unwrap() {
        let said = serde_json::json!({ "root": one, "machine": all["signed_by"]["machine"], "at": "2026-11-03T12:00:00Z", "signed": all["revocation"]["signed"] });
        assert!(!check("revocation", &said).is_empty(), "{one}");
    }
}

#[test]
fn what_each_published_version_wrote_still_opens() {
    let all = vectors();
    for (version, said) in all["frozen"].as_object().unwrap() {
        let key = b64(&said["link_key"]);
        let (_, held) = reading(&hex(&said["main"]["envelope"]), MAIN, Some(&key)).expect(version);
        assert_eq!(held, said["main"]["document"], "{version}");
        files_for(&held, &hex(&said["files"]["envelope"]), FILES, Some(&key)).expect(version);
        let (_, open) = reading(&hex(&said["public"]["envelope"]), PUBLIC, None).expect(version);
        assert_eq!(open, said["public"]["document"], "{version}");
    }
}

#[test]
fn a_revocation_of_its_owner_s_machine_weighs_on_what_an_editor_signed() {
    let all = vectors();
    let said = &all["editing"]["owner_revoked"];
    let id = all["editing"]["id"].as_str().unwrap();
    let opened = opened(&hex(&said["envelope"]), (id, "main"), Some(&b64(&all["link_key"]))).unwrap();
    let manifest: Value = serde_json::from_slice(&opened.manifest).unwrap();
    assert_eq!(verdict_of(&manifest["owner"], &said["served"]), said["verdict"]);
}

#[test]
fn a_reader_refuses_what_a_writer_should_never_have_sealed() {
    let all = vectors();
    for one in all["refused"].as_array().unwrap() {
        let bytes = hex(&one["envelope"]);
        assert!(reading(&bytes, ("hostile", "main"), None).is_err(), "{}", one["says"]);
    }
    assert!(reading(&hex(&all["past_its_end"]), ("hostile", "main"), None).is_err());
}

#[test]
fn a_reader_refuses_files_its_document_does_not_answer_for() {
    let all = vectors();
    let key = b64(&all["link_key"]);
    let (_, held) = reading(&hex(&all["main"]["envelope"]), MAIN, Some(&key)).unwrap();
    for one in all["files_refused"].as_array().unwrap() {
        assert!(files_for(&held, &hex(&one["files"]), FILES, Some(&key)).is_err(), "{}", one["says"]);
    }
    for one in all["files_lying"].as_array().unwrap() {
        assert!(files_for(&one["document"], &hex(&one["files"]), ("hostile", "files"), None).is_err(), "{}", one["says"]);
    }
}

#[test]
fn a_reader_weighs_a_signature_by_what_the_server_serves() {
    let all = vectors();
    for one in all["verdicts"].as_array().unwrap() {
        assert_eq!(check("served", &one["served"]), Vec::<String>::new());
        assert_eq!(verdict_of(&all["signed_by"], &one["served"]), one["verdict"], "{}", one["says"]);
    }
}

#[test]
fn a_reader_opens_what_a_later_minor_only_adds() {
    let all = vectors();
    for one in all["opening"].as_array().unwrap() {
        let said = reading(&hex(&one["envelope"]), ("hostile", "main"), None);
        assert!(said.is_ok(), "{}: {:?}", one["says"], said.err());
    }
}

#[test]
fn a_newer_major_version_is_told_from_a_damaged_one_whatever_it_holds() {
    let all = vectors();
    let said = reading(&hex(&all["a_newer_major"]), ("hostile", "main"), None);
    assert!(said.err().is_some_and(|wrong| wrong.contains("version")));
}

#[test]
fn a_reader_looks_at_a_picture_before_it_draws_it() {
    let all = vectors();
    for one in all["pictures"]["hold"].as_array().unwrap() {
        assert!(picture_holds(&hex(&one["bytes"]), one), "{}", one["kind"]);
    }
    for one in all["pictures"]["fail"].as_array().unwrap() {
        assert!(!picture_holds(&hex(&one["bytes"]), one), "{}", one["says"]);
    }
}

#[test]
fn a_reference_is_read_as_a_browser_reads_it_and_only_where_it_leads() {
    let all = vectors();
    let said: Vec<(String, u64)> = all["references"]["said"]
        .as_array()
        .unwrap()
        .iter()
        .map(|one| (one["kind"].as_str().unwrap().to_owned(), one["at"].as_u64().unwrap()))
        .collect();
    assert_eq!(references_in(all["references"]["html"].as_str().unwrap()), said);
}

#[test]
fn the_fingerprint_is_what_the_words_of_the_contract_make_of_a_root() {
    let all = vectors();
    let digest = Sha256::digest([b"tisty-share.person.v1\0".as_slice(), &b64(&all["identity"]["root"])].concat());
    let mut wide = [0u8; 16];
    wide[7..].copy_from_slice(&digest[..9]);
    let digits = format!("{:020}", u128::from_be_bytes(wide) >> 6);
    let groups: Vec<&str> = digits.as_bytes().chunks(5).map(|one| std::str::from_utf8(one).unwrap()).collect();
    assert_eq!(groups.join(" "), all["identity"]["fingerprint"]);
}

#[test]
fn another_key_does_not_open_it() {
    let all = vectors();
    assert!(opened(&hex(&all["main"]["envelope"]), MAIN, Some(&[0u8; 32])).is_err());
}

#[test]
fn a_link_with_a_key_refuses_an_envelope_that_is_not_sealed() {
    let all = vectors();
    let key = b64(&all["link_key"]);
    assert!(opened(&hex(&all["public"]["envelope"]), PUBLIC, Some(&key)).is_err());
}

#[test]
fn neither_another_document_nor_another_kind_passes_for_the_one_asked_for() {
    let all = vectors();
    let key = b64(&all["link_key"]);
    assert!(opened(&hex(&all["public"]["envelope"]), MAIN, None).is_err());
    assert!(opened(&hex(&all["files"]["envelope"]), MAIN, Some(&key)).is_err());
}

#[test]
fn changing_or_cutting_it_does_not_open_it() {
    let all = vectors();
    let key = b64(&all["link_key"]);
    let whole = hex(&all["main"]["envelope"]);
    let mut last = whole.clone();
    *last.last_mut().unwrap() ^= 1;
    assert!(opened(&last, MAIN, Some(&key)).is_err());
    let mut manifest = whole.clone();
    manifest[40] ^= 1;
    assert!(opened(&manifest, MAIN, Some(&key)).is_err());
    assert!(opened(&whole[..whole.len() - 1], MAIN, Some(&key)).is_err());
}
