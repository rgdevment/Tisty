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
use tisty_contracts::{check, check_html};

const VECTORS: &str = include_str!("../vectors.json");

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

fn seconds(said: &str) -> i64 {
    // Whatever the moment, it is whole seconds, and only this date is ever signed in the samples.
    assert_eq!(said, "2026-11-03T12:00:00Z");
    1_793_707_200
}

const MAIN: (&str, &str) = ("k7Qm2xPa", "main");
const FILES: (&str, &str) = ("k7Qm2xPa", "files");
const PUBLIC: (&str, &str) = ("wordsOnly", "main");

pub fn opened(bytes: &[u8], asked: (&str, &str), link_key: Option<&[u8]>) -> Result<Opened, String> {
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
    if size > 65536 {
        return Err("its manifest is past its size".into());
    }
    let manifest_bytes = piece(size)?;
    let manifest: Value = serde_json::from_slice(manifest_bytes).map_err(|_| "its manifest")?;
    if !check("envelope", &manifest).is_empty() {
        return Err("its manifest".into());
    }
    if manifest["id"] != asked.0 || manifest["kind"] != asked.1 {
        return Err("not what it asked for".into());
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
    if size > 4096 {
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
        let travelled = piece(part["sealed_size"].as_u64().unwrap() as usize)?;
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
        let open_size = part["open_size"].as_u64().unwrap();
        let held = if part["compressed"] == true {
            let mut inflated = Vec::new();
            DeflateDecoder::new(held.as_slice())
                .take(open_size + 1)
                .read_to_end(&mut inflated)
                .map_err(|_| "it does not inflate")?;
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
    Ok(Opened { manifest: manifest_bytes.to_vec(), index: index.to_vec(), parts: out })
}

// The first two parts of a main envelope are its words; what follows them is pictures.
pub fn document_of(opened: &Opened) -> Value {
    let mut out = Map::new();
    for part in &opened.parts[..2] {
        let Value::Object(fields) = serde_json::from_slice(part).unwrap() else {
            panic!("the words of a main envelope are an object");
        };
        out.extend(fields);
    }
    Value::Object(out)
}

fn vectors() -> Value {
    serde_json::from_str(VECTORS).unwrap()
}

#[test]
fn a_main_envelope_gives_its_document_its_pages_and_its_pictures() {
    let all = vectors();
    let main = opened(&hex(&all["main"]["envelope"]), MAIN, Some(&b64(&all["link_key"]))).unwrap();
    let held = document_of(&main);
    assert_eq!(held, all["main"]["document"]);
    assert_eq!(check("document", &held), Vec::<String>::new());
    assert_eq!(check_html(held["html"].as_str().unwrap()), Vec::<String>::new());
    assert_eq!(held["id"], MAIN.0, "the document inside is the one the manifest says");
    let pictures: Vec<Vec<u8>> = all["main"]["pictures"].as_array().unwrap().iter().map(hex).collect();
    assert_eq!(main.parts[2..], pictures);
}

#[test]
fn a_files_envelope_gives_its_files_and_the_main_envelope_answers_for_it() {
    let all = vectors();
    let held = opened(&hex(&all["files"]["envelope"]), FILES, Some(&b64(&all["link_key"]))).unwrap();
    let files: Vec<Vec<u8>> = all["files"]["files"].as_array().unwrap().iter().map(hex).collect();
    assert_eq!(held.parts, files);
    let hash = Sha256::digest([held.manifest, held.index].concat());
    assert_eq!(hash.as_slice(), b64(&all["main"]["document"]["files_hash"]));
}

#[test]
fn a_public_envelope_opens_without_a_key() {
    let all = vectors();
    let held = document_of(&opened(&hex(&all["public"]["envelope"]), PUBLIC, None).unwrap());
    assert_eq!(held, all["public"]["document"]);
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
