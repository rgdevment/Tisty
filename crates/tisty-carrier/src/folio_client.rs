use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const ID_CHARACTERS: usize = 8;
const PROOF_BITS_AT_MOST: u32 = 32;
const PROOFS_AT_MOST: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asked<'a> {
    pub verb: Verb,
    pub path: String,
    pub headers: Vec<(&'static str, String)>,
    pub body: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answered {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Answered {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(one, _)| one.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

pub trait Door {
    fn ask(&self, asked: &Asked<'_>) -> Option<Answered>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Why {
    Newer,
    Older,
    Damaged,
    Unsigned,
    Key,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "refused", rename_all = "snake_case")]
pub enum Refused {
    BadUpload,
    Paused,
    Blocked,
    NotYours,
    Revoked,
    Missing,
    Behind,
    SharesFull,
    RoomFull {
        #[serde(default)]
        held: Option<u64>,
        #[serde(default)]
        arriving: Option<u64>,
        #[serde(default, rename = "atMost")]
        at_most: Option<u64>,
    },
    LanguagesFull,
    MachinesFull,
    TooBig,
    Envelope {
        why: Why,
    },
    Outdated,
    Slow {
        until: u64,
    },
    Quota {
        until: u64,
    },
    SharesToday {
        until: u64,
    },
    Proof {
        salt: String,
        bits: u32,
    },
    Failed,
    UploadsOff,
    Terms {
        #[serde(default, deserialize_with = "said_as_text")]
        version: Option<String>,
    },
    KeyBlocked,
    NotBlocked,
    Appealed,
    #[serde(other)]
    Unknown,
}

fn said_as_text<'de, D: serde::Deserializer<'de>>(said: D) -> Result<Option<String>, D::Error> {
    Ok(match serde_json::Value::deserialize(said)? {
        serde_json::Value::String(text) => Some(text),
        serde_json::Value::Number(number) => {
            number.as_u64().map(|whole| whole.to_string()).or_else(|| {
                number
                    .as_f64()
                    .filter(|one| one.fract() == 0.0 && one.is_sign_positive())
                    .map(|whole| format!("{whole:.0}"))
            })
        }
        _ => None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stopped {
    Unreached,
    Garbled,
    Refused(Refused),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Start {
    pub id: String,
    pub mac: String,
    pub salt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct First<'a> {
    pub mac: &'a str,
    pub admin: &'a str,
    pub ttl_days: u64,
    pub anchor: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version<'a> {
    pub id: &'a str,
    pub envelope: &'a [u8],
    pub files_hash: Option<&'a str>,
    pub first: Option<First<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Stored {
    pub version: u64,
    #[serde(default, rename = "filesWanted")]
    pub files_wanted: bool,
}

// An id goes into every path asked for afterwards, so a server's word for it is checked first.
fn holds_as_id(id: &str) -> bool {
    id.len() == ID_CHARACTERS && id.chars().all(|one| one.is_ascii_alphanumeric())
}

fn read<T: DeserializeOwned>(answered: &Answered) -> Result<T, Stopped> {
    if (200..300).contains(&answered.status) {
        return serde_json::from_slice(&answered.body).map_err(|_| Stopped::Garbled);
    }
    let refused = serde_json::from_slice(&answered.body).map_err(|_| Stopped::Garbled)?;
    Err(Stopped::Refused(refused))
}

fn zero_bits_at_start(digest: &[u8], bits: u32) -> bool {
    let whole = (bits / 8) as usize;
    let rest = bits % 8;
    digest.len() >= whole + usize::from(rest > 0)
        && digest[..whole].iter().all(|one| *one == 0)
        && (rest == 0 || digest[whole] >> (8 - rest) == 0)
}

pub(crate) fn solved(salt: &str, bits: u32) -> Option<u64> {
    if bits > PROOF_BITS_AT_MOST {
        return None;
    }
    let mut salted = Sha256::new();
    salted.update(salt.as_bytes());
    salted.update(b"|");
    (0..=u64::MAX).find(|number| {
        let mut hash = salted.clone();
        hash.update(number.to_string().as_bytes());
        zero_bits_at_start(&hash.finalize(), bits)
    })
}

// Sent twice, Folio stores it once, so asking again with the proof is safe.
fn answered_with_proof(door: &dyn Door, mut asked: Asked<'_>) -> Result<Answered, Stopped> {
    for _ in 0..PROOFS_AT_MOST {
        let answered = door.ask(&asked).ok_or(Stopped::Unreached)?;
        if (200..300).contains(&answered.status) {
            return Ok(answered);
        }
        match read::<serde_json::Value>(&answered) {
            Err(Stopped::Refused(Refused::Proof { salt, bits })) => {
                let number = solved(&salt, bits).ok_or(Stopped::Garbled)?;
                asked.headers.retain(|(name, _)| *name != "Folio-Proof");
                asked.headers.push(("Folio-Proof", number.to_string()));
            }
            Err(other) => return Err(other),
            Ok(_) => return Err(Stopped::Garbled),
        }
    }
    let answered = door.ask(&asked).ok_or(Stopped::Unreached)?;
    if (200..300).contains(&answered.status) {
        return Ok(answered);
    }
    read::<serde_json::Value>(&answered).and(Err(Stopped::Garbled))
}

fn asked_with_proof<T: DeserializeOwned>(door: &dyn Door, asked: Asked<'_>) -> Result<T, Stopped> {
    read(&answered_with_proof(door, asked)?)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    pub bytes: Vec<u8>,
    pub version: u64,
    pub arrived_ms: i64,
    pub revocations: Option<String>,
    pub reviewed: Option<String>,
    pub reviewed_arrived_ms: Option<i64>,
}

pub fn fetch(door: &dyn Door, id: &str) -> Result<Fetched, Stopped> {
    let answered = answered_with_proof(
        door,
        Asked {
            verb: Verb::Get,
            path: format!("/docs/{id}"),
            headers: Vec::new(),
            body: &[],
        },
    )?;
    let number = |name| answered.header(name)?.parse().ok();
    let version = number("Folio-Version").ok_or(Stopped::Garbled)?;
    let arrived_ms = answered
        .header("Folio-Arrived")
        .and_then(|said| said.parse().ok())
        .ok_or(Stopped::Garbled)?;
    let revocations = answered.header("Folio-Revocations").map(str::to_owned);
    let reviewed = answered.header("Folio-Reviewed").map(str::to_owned);
    let reviewed_arrived_ms = answered
        .header("Folio-Reviewed-Arrived")
        .and_then(|said| said.parse().ok());
    Ok(Fetched {
        bytes: answered.body,
        version,
        arrived_ms,
        revocations,
        reviewed,
        reviewed_arrived_ms,
    })
}

pub fn fetch_files(door: &dyn Door, id: &str, hash: &str) -> Result<Vec<u8>, Stopped> {
    if hash.len() != 43
        || !hash
            .chars()
            .all(|one| one.is_ascii_alphanumeric() || one == '-' || one == '_')
    {
        return Err(Stopped::Garbled);
    }
    let answered = answered_with_proof(
        door,
        Asked {
            verb: Verb::Get,
            path: format!("/docs/{id}/files/{hash}"),
            headers: Vec::new(),
            body: &[],
        },
    )?;
    Ok(answered.body)
}

pub fn start(door: &dyn Door) -> Result<Start, Stopped> {
    let asked = Asked {
        verb: Verb::Post,
        path: "/start".to_owned(),
        headers: Vec::new(),
        body: &[],
    };
    let start: Start = read(&door.ask(&asked).ok_or(Stopped::Unreached)?)?;
    if holds_as_id(&start.id) {
        Ok(start)
    } else {
        Err(Stopped::Garbled)
    }
}

pub fn send_version(door: &dyn Door, version: &Version<'_>) -> Result<Stored, Stopped> {
    let mut headers = Vec::new();
    if let Some(hash) = version.files_hash {
        headers.push(("Folio-Files", hash.to_owned()));
    }
    if let Some(first) = &version.first {
        headers.push(("Folio-Start", first.mac.to_owned()));
        headers.push(("Folio-Admin", first.admin.to_owned()));
        headers.push(("Folio-Ttl-Days", first.ttl_days.to_string()));
        if let Some(anchor) = first.anchor {
            headers.push(("Folio-Anchor", anchor.to_owned()));
        }
    }
    asked_with_proof(
        door,
        Asked {
            verb: Verb::Put,
            path: format!("/docs/{}", version.id),
            headers,
            body: version.envelope,
        },
    )
}

pub fn send_files(door: &dyn Door, id: &str, envelope: &[u8]) -> Result<(), Stopped> {
    #[derive(Deserialize)]
    struct FilesStored {}

    let asked = Asked {
        verb: Verb::Put,
        path: format!("/docs/{id}/files"),
        headers: Vec::new(),
        body: envelope,
    };
    let _: FilesStored = read(&door.ask(&asked).ok_or(Stopped::Unreached)?)?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Latest {
    pub version: u64,
    #[serde(default, rename = "nextAt")]
    pub next_at: Option<u64>,
    #[serde(default, rename = "reviewedSeq")]
    pub reviewed_seq: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub latest: u64,
    pub dies_on: u64,
    pub paused_at: Option<u64>,
    pub blocked_at: Option<u64>,
    pub read_total: u64,
    #[serde(default)]
    pub appealed_at: Option<u64>,
    #[serde(default)]
    pub may_correct: bool,
    #[serde(default)]
    pub fix_by: Option<u64>,
}

#[derive(Deserialize)]
struct Told {
    status: Status,
}

#[derive(Deserialize)]
struct Done {}

fn asked_as_admin(
    door: &dyn Door,
    verb: Verb,
    path: String,
    admin: &str,
) -> Result<Answered, Stopped> {
    let asked = Asked {
        verb,
        path,
        headers: vec![("Folio-Admin", admin.to_owned())],
        body: &[],
    };
    door.ask(&asked).ok_or(Stopped::Unreached)
}

pub fn latest(door: &dyn Door, id: &str) -> Result<Latest, Stopped> {
    let asked = Asked {
        verb: Verb::Get,
        path: format!("/docs/{id}/latest"),
        headers: Vec::new(),
        body: &[],
    };
    read(&door.ask(&asked).ok_or(Stopped::Unreached)?)
}

pub fn status(door: &dyn Door, id: &str, admin: &str) -> Result<Status, Stopped> {
    let answered = asked_as_admin(door, Verb::Get, format!("/docs/{id}/admin"), admin)?;
    read::<Told>(&answered).map(|told| told.status)
}

pub fn pause(door: &dyn Door, id: &str, admin: &str) -> Result<(), Stopped> {
    let answered = asked_as_admin(door, Verb::Post, format!("/docs/{id}/pause"), admin)?;
    read::<Done>(&answered).map(|_| ())
}

pub fn resume(door: &dyn Door, id: &str, admin: &str) -> Result<(), Stopped> {
    let answered = asked_as_admin(door, Verb::Post, format!("/docs/{id}/resume"), admin)?;
    read::<Done>(&answered).map(|_| ())
}

#[derive(Serialize)]
struct Permitted<'a> {
    permit: Option<&'a str>,
}

/// Who may edit changes only here, with the link to manage it: an upload never changes it (148).
pub fn permit(door: &dyn Door, id: &str, admin: &str, permit: Option<&str>) -> Result<(), Stopped> {
    let body = serde_json::to_vec(&Permitted { permit }).map_err(|_| Stopped::Garbled)?;
    let asked = Asked {
        verb: Verb::Put,
        path: format!("/docs/{id}/permit"),
        headers: vec![("Folio-Admin", admin.to_owned())],
        body: &body,
    };
    read::<Done>(&door.ask(&asked).ok_or(Stopped::Unreached)?).map(|_| ())
}

pub fn stop(door: &dyn Door, id: &str, admin: &str) -> Result<(), Stopped> {
    let answered = asked_as_admin(door, Verb::Delete, format!("/docs/{id}"), admin)?;
    match read::<Done>(&answered) {
        Err(Stopped::Refused(Refused::Missing)) | Ok(_) => Ok(()),
        Err(other) => Err(other),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Taken {
    pub root: String,
    pub machine: String,
    pub at: String,
    pub signed: String,
}

pub fn taken_back(door: &dyn Door, taken: &[Taken]) -> Result<(), Stopped> {
    let body = serde_json::to_vec(taken).map_err(|_| Stopped::Garbled)?;
    let asked = Asked {
        verb: Verb::Post,
        path: "/revocations".to_owned(),
        headers: Vec::new(),
        body: &body,
    };
    read::<Done>(&door.ask(&asked).ok_or(Stopped::Unreached)?).map(|_| ())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Claim {
    Accepted,
    Review,
}

#[derive(Deserialize)]
struct Appealed {
    claim: Claim,
}

pub fn appeal(door: &dyn Door, id: &str, admin: &str) -> Result<Claim, Stopped> {
    let answered = asked_as_admin(door, Verb::Post, format!("/docs/{id}/appeal"), admin)?;
    read::<Appealed>(&answered).map(|one| one.claim)
}

pub fn api_of(host: &str) -> String {
    format!("https://{host}/api/v1")
}

pub fn read_link(host: &str, id: &str, key: &str) -> String {
    format!("https://{host}/{id}#{key}")
}

pub fn edit_link(host: &str, id: &str, key: &str, token: &str) -> String {
    format!("https://{host}/{id}#{key}.{token}")
}

#[cfg(test)]
#[path = "folio_client_test.rs"]
mod tests;
