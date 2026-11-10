use std::collections::BTreeMap;
use std::io::Read;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tisty_share_proto::identity::{
    self, Certificate, Person, Revocation, ShareKey, Signed, Verdict,
};
use tisty_share_proto::name::named_well;

use crate::event::{DeviceId, Edited};
use crate::model::Kept;
use crate::person::Persons;
use crate::state::State;
use crate::{Error, Result};

pub(crate) const ENTRY: &str = "tisty-docs.signed.json";
pub(crate) const ENTRY_AT_MOST: u64 = 64 * 1024;
const ABOUT: &str = "tisty.parcel.v1";
const EDITORS_AT_MOST: usize = 64;

pub struct Signer<'a> {
    pub key: &'a ShareKey,
    pub certificate: &'a Certificate,
    pub name: Option<&'a str>,
}

impl Signer<'_> {
    pub(crate) fn signs(&self) -> bool {
        self.certificate
            .vouches()
            .is_ok_and(|vouched| vouched.share == self.key.public())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Signature {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    signed: Signed,
}

#[derive(Debug, Default)]
pub(crate) struct Prints(BTreeMap<String, [u8; 32]>);

impl Prints {
    pub(crate) fn add(&mut self, named: &str, digest: [u8; 32]) -> bool {
        self.0.insert(named.to_owned(), digest).is_none()
    }

    fn bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for (named, digest) in &self.0 {
            out.extend_from_slice(&(named.len() as u64).to_be_bytes());
            out.extend_from_slice(named.as_bytes());
            out.extend_from_slice(digest);
        }
        out
    }
}

pub(crate) struct Hashing<R> {
    inner: R,
    hasher: Sha256,
}

impl<R> Hashing<R> {
    pub(crate) fn new(inner: R) -> Self {
        Self {
            inner,
            hasher: Sha256::new(),
        }
    }

    pub(crate) fn digest(self) -> [u8; 32] {
        self.hasher.finalize().into()
    }
}

impl<R: Read> Read for Hashing<R> {
    fn read(&mut self, into: &mut [u8]) -> std::io::Result<usize> {
        let got = self.inner.read(into)?;
        self.hasher.update(&into[..got]);
        Ok(got)
    }
}

pub(crate) fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

// Each part says its length, so no byte can move from one into the next and still be signed.
fn said(manifest: &[u8], prints: &Prints, name: Option<&str>) -> Vec<u8> {
    let prints = prints.bytes();
    let name = name.unwrap_or_default().as_bytes();
    let mut out = Vec::with_capacity(manifest.len() + prints.len() + name.len() + 24);
    for part in [manifest, prints.as_slice(), name] {
        out.extend_from_slice(&(part.len() as u64).to_be_bytes());
        out.extend_from_slice(part);
    }
    out
}

pub(crate) fn signed(signer: &Signer, manifest: &[u8], prints: &Prints) -> Option<Vec<u8>> {
    let name = signer.name.filter(|one| named_well(one)).map(str::to_owned);
    let signed = signer.key.sign(
        signer.certificate,
        ABOUT,
        &said(manifest, prints, name.as_deref()),
    )?;
    serde_json::to_vec_pretty(&Signature { name, signed }).ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claimed {
    pub root: String,
    pub print: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Mine,
    Theirs,
}

pub(crate) struct Asked {
    pub(crate) unlocked: bool,
    pub(crate) answer: Option<Answer>,
    pub(crate) same_store: bool,
}

pub(crate) fn mine(
    closed: &Closed,
    persons: &Persons,
    device: &DeviceId,
    asked: &Asked,
) -> Result<bool> {
    match closed {
        Closed::Broken => Ok(false),
        Closed::By { person, name } => match (persons.owner(device), asked.answer) {
            (Some(_), _) => Ok(persons.names(person, device)),
            (None, Some(_)) => Ok(false),
            (None, None) => Err(Error::ParcelSignedBy {
                root: person.shown(),
                print: person.fingerprint(),
                name: name.clone(),
            }),
        },
        Closed::Unsigned if asked.same_store => Ok(true),
        Closed::Unsigned => match (asked.unlocked, asked.answer) {
            (false, _) => Ok(false),
            (true, Some(answer)) => Ok(answer == Answer::Mine),
            (true, None) => Err(Error::ParcelAsksIfMine),
        },
    }
}

pub(crate) struct Vouched {
    root: String,
    name: Option<String>,
}

impl Closed {
    pub(crate) fn vouched(&self) -> Option<Vouched> {
        match self {
            Closed::By { person, name } => Some(Vouched {
                root: person.shown(),
                name: name.clone(),
            }),
            _ => None,
        }
    }
}

/// Who edited a document under Folio and in which Folio document, as this store last heard it.
pub(crate) fn edited_of(state: &State, kept: &Kept) -> Option<(String, serde_json::Value)> {
    let listed = |said: &str| {
        serde_json::from_str::<serde_json::Value>(said)
            .ok()
            .filter(|all| all.as_array().is_some_and(|all| !all.is_empty()))
    };
    if let Some(was) = state.sharing.get(&kept.file)
        && let Some(carried) = was.carried.as_deref()
        && let Ok(carried) = serde_json::from_str::<serde_json::Value>(carried)
        && carried["editors"]
            .as_array()
            .is_some_and(|all| !all.is_empty())
    {
        return Some((was.id.clone(), carried["editors"].clone()));
    }
    if let Some(stopped) = state
        .sharing_editors
        .get(&kept.file)
        .and_then(|one| listed(one))
        && let Some(folio) = stopped[0]["in"].as_str()
    {
        return Some((folio.to_owned(), stopped));
    }
    let edited = kept.edited.as_ref()?;
    Some((edited.folio.clone(), listed(&edited.editors)?))
}

// Each entry answers only for its own Folio document, so one carried for another is left behind.
pub(crate) fn edited(
    vouched: Option<&Vouched>,
    folio: Option<&str>,
    editors: Option<&serde_json::Value>,
) -> Option<Edited> {
    let (vouched, folio) = (vouched?, folio?);
    let kept: Vec<&serde_json::Value> = editors?
        .as_array()?
        .iter()
        .filter(|one| one["in"].as_str() == Some(folio))
        .take(EDITORS_AT_MOST)
        .collect();
    (!kept.is_empty()).then(|| Edited {
        folio: folio.to_owned(),
        root: vouched.root.clone(),
        by: vouched.name.clone(),
        editors: serde_json::Value::from(kept.into_iter().cloned().collect::<Vec<_>>()).to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Closed {
    Unsigned,
    By {
        person: Box<Person>,
        name: Option<String>,
    },
    Broken,
}

pub(crate) fn closed(
    entry: Option<&[u8]>,
    manifest: &[u8],
    prints: Option<&Prints>,
    revoked: &[Revocation],
) -> Closed {
    let Some(entry) = entry else {
        return Closed::Unsigned;
    };
    let (Some(prints), Ok(signature)) = (prints, serde_json::from_slice::<Signature>(entry)) else {
        return Closed::Broken;
    };
    let message = said(manifest, prints, signature.name.as_deref());
    match identity::check(&signature.signed, ABOUT, &message, revoked) {
        // Signed or not, a name that hides a letter or passes for a fingerprint is never shown.
        Verdict::Holds { person, .. } => Closed::By {
            person: Box::new(person),
            name: signature.name.filter(|one| named_well(one)),
        },
        _ => Closed::Broken,
    }
}

#[cfg(test)]
#[path = "signed_parcel_test.rs"]
mod tests;
