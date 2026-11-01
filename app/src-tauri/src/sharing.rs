use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tisty_core::event::Shared;
use tisty_share_proto::envelope::{self, Draft, Kind, LinkKey, Refused as Unsealed, Signer};
use tisty_share_proto::identity::{REVOCATIONS_AT_ONCE, Revocation};
use tisty_share_proto::limits::Limits;
use tisty_share_proto::work::{self, Challenge};

use crate::handed::Voucher;
use crate::refusing::Refusal;

/// Tisty Share as Tisty knows it; each share keeps the server it went to, so a later one can live elsewhere.
pub(crate) const SERVER: &str = "https://tisty.cl";
const API: &str = "/api/v1/";
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(30);

/// What came back from one request.
pub(crate) struct Said {
    pub(crate) status: u16,
    pub(crate) body: Vec<u8>,
    /// The `Tisty-Revoked` the server sends with a document, for the reader to weigh the signature by.
    pub(crate) revoked: Option<String>,
}

/// One request as the API takes it, so the same client speaks to a server or, in its tests, to the API itself.
pub(crate) trait Wire {
    fn send(&self, asked: Asking<'_>) -> Result<Said, Refusal>;
}

pub(crate) struct Asking<'a> {
    pub(crate) method: &'a str,
    /// Below `/api/v1/`.
    pub(crate) path: String,
    /// The edit or management token.
    pub(crate) token: Option<&'a str>,
    /// The proof of work, as JSON.
    pub(crate) work: Option<String>,
    pub(crate) body: Vec<u8>,
}

pub(crate) struct Net {
    client: reqwest::blocking::Client,
    server: String,
}

impl Net {
    pub(crate) fn to(server: &str) -> Result<Self, Refusal> {
        let client = reqwest::blocking::Client::builder()
            .timeout(PATIENCE)
            .user_agent(concat!("tisty/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| Refusal::of("shareOffline"))?;
        Ok(Self {
            client,
            server: server.trim_end_matches('/').to_owned(),
        })
    }
}

impl Wire for Net {
    fn send(&self, asked: Asking<'_>) -> Result<Said, Refusal> {
        let url = format!("{}{API}{}", self.server, asked.path);
        let method = reqwest::Method::from_bytes(asked.method.as_bytes())
            .map_err(|_| Refusal::of("internal"))?;
        let mut request = self.client.request(method, url).body(asked.body);
        if let Some(token) = asked.token {
            request = request.bearer_auth(token);
        }
        if let Some(work) = asked.work {
            request = request.header("Tisty-Work", work);
        }
        let answer = request.send().map_err(|_| Refusal::of("shareOffline"))?;
        let status = answer.status().as_u16();
        let revoked = answer
            .headers()
            .get("Tisty-Revoked")
            .and_then(|one| one.to_str().ok())
            .map(str::to_owned);
        let body = answer.bytes().map_err(|_| Refusal::of("shareOffline"))?;
        Ok(Said {
            status,
            body: body.to_vec(),
            revoked,
        })
    }
}

/// What the person chose in the «Share» dialog.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Chosen {
    /// A password the reader must give, or none for a link alone.
    pub(crate) password: Option<String>,
    /// Whether the edit link is handed out (decision 67).
    pub(crate) editable: bool,
    /// Days without activity before it goes, when shorter than the server's (decision 50).
    pub(crate) idle_days: Option<u32>,
    /// The print of the words the dialog looked through for keys, so what goes out is what was looked at.
    #[serde(default)]
    pub(crate) seen: Option<String>,
}

/// A print of a text, to tell whether it changed between two reads of it.
pub(crate) fn print_of(text: &str) -> String {
    use sha2::Digest;
    URL_SAFE_NO_PAD.encode(sha2::Sha256::digest(text.as_bytes()))
}

/// What the owner sees of a share on its card, beside the links the store keeps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Info {
    pub(crate) latest: Option<u64>,
    pub(crate) reads: u64,
    pub(crate) ends: i64,
    pub(crate) paused: bool,
    pub(crate) blocked: bool,
    pub(crate) gated: bool,
}

#[derive(Deserialize)]
struct Reserved {
    id: String,
    edit: String,
    admin: String,
}

/// A server's no, said as a refusal the window words: its own word rides along as the name.
fn refused(said: &Said) -> Refusal {
    let word = serde_json::from_slice::<serde_json::Value>(&said.body)
        .ok()
        .and_then(|value| value["refused"].as_str().map(str::to_owned));
    match word.as_deref() {
        // The one no the person can act on: this machine's key was taken back or never held.
        Some("unsigned") => Refusal::of("shareUnsigned"),
        Some("too_large") => Refusal::of("shareTooBig"),
        Some("slow") => Refusal::of("shareSlow"),
        // The server's own trouble, or no word at all: nothing the person can do but try again.
        Some("store" | "crowded" | "work_refused" | "no_randomness") | None => {
            Refusal::of("shareFailed")
        }
        Some(_) if said.status >= 500 => Refusal::of("shareFailed"),
        Some(word) => Refusal::about("shareRefused", word.to_owned()),
    }
}

fn answered<T: for<'de> Deserialize<'de>>(said: Said) -> Result<T, Refusal> {
    if !(200..300).contains(&said.status) {
        return Err(refused(&said));
    }
    serde_json::from_slice(&said.body).map_err(|_| Refusal::of("shareFailed"))
}

fn done(said: Said) -> Result<(), Refusal> {
    match (200..300).contains(&said.status) {
        true => Ok(()),
        false => Err(refused(&said)),
    }
}

pub(crate) fn get(wire: &impl Wire, path: String, token: Option<&str>) -> Result<Said, Refusal> {
    wire.send(Asking {
        method: "GET",
        path,
        token,
        work: None,
        body: Vec::new(),
    })
}

// Each piece of work answers one ask only, so it is asked for right before it is spent.
fn worked(wire: &impl Wire, ask: serde_json::Value) -> Result<String, Refusal> {
    let said = wire.send(Asking {
        method: "POST",
        path: "challenge".into(),
        token: None,
        work: None,
        body: ask.to_string().into_bytes(),
    })?;
    let challenge: Challenge = answered(said)?;
    let proof = work::solve(&challenge).ok_or(Refusal::of("shareFailed"))?;
    serde_json::to_string(&proof).map_err(|_| Refusal::of("internal"))
}

fn managed(wire: &impl Wire, shared: &Shared, what: serde_json::Value) -> Result<(), Refusal> {
    done(wire.send(Asking {
        method: "POST",
        path: format!("docs/{}/manage", shared.link),
        token: Some(&shared.admin),
        work: None,
        body: what.to_string().into_bytes(),
    })?)
}

/// Where a picture the words name sits in an envelope, which the reader turns back into the picture (decision 79).
const PICTURE: &str = "tisty:picture/";

/// The words with each picture they name from the attachments put in its place in the envelope, and those pictures
/// written again, so nothing but the pixels travels. What is no picture, or sits in code, stays as it was.
pub(crate) fn pictured(
    text: &str,
    read: impl Fn(&str) -> Option<Vec<u8>>,
) -> (String, Vec<(Kind, Vec<u8>)>) {
    let code = tisty_core::beside::written_as_code(text);
    let in_code = |at: usize| code.get(at).copied().unwrap_or(false);
    let mut said = String::with_capacity(text.len());
    let mut pictures: Vec<(Kind, Vec<u8>)> = Vec::new();
    let mut placed: Vec<String> = Vec::new();
    let mut rest = 0;
    let mut from = 0;
    while let Some(found) = text[from..].find("![") {
        let at = from + found;
        from = at + 2;
        let Some((opens, closes)) = destination(text, at) else {
            continue;
        };
        if in_code(at) || in_code(opens) {
            continue;
        }
        let inner = &text[opens..closes];
        let starts = opens + (inner.len() - inner.trim_start_matches([' ', '\t']).len());
        let named = &text[starts..closes];
        // Within `<…>` a destination may hold spaces; bare, the first space ends it.
        let written = match named.strip_prefix('<').and_then(|inside| inside.find('>')) {
            Some(ends) => &named[..ends + 2],
            None => named.split([' ', '\t']).next().unwrap_or_default(),
        };
        let target = written.trim_start_matches('<').trim_end_matches('>');
        if !tisty_core::attach::names_an_attachment(target) {
            continue;
        }
        let place = match placed.iter().position(|one| one == target) {
            Some(place) => place,
            None => {
                let made = read(target).and_then(|bytes| {
                    tisty_share_proto::pictures::recompressed(&bytes, &Limits::default()).ok()
                });
                let Some(made) = made else {
                    continue;
                };
                pictures.push(made);
                placed.push(target.to_owned());
                placed.len() - 1
            }
        };
        said.push_str(&text[rest..starts]);
        said.push_str(&format!("{PICTURE}{place}"));
        rest = starts + written.len();
        from = closes;
    }
    said.push_str(&text[rest..]);
    (said, pictures)
}

/// The bytes of an attachment the words name, read where the store keeps them; called off the window's thread.
pub(crate) fn attachment(target: &str, data: &std::path::Path) -> Option<Vec<u8>> {
    std::fs::read(tisty_core::attach::resolve(target, data).ok()?).ok()
}

// The `(` and `)` of an image's destination: right after the `]` that closes its alt text, all on one line.
fn destination(text: &str, image: usize) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut at = image + 2;
    loop {
        match bytes.get(at)? {
            b'\n' => return None,
            b'\\' => at += 1,
            b'[' => depth += 1,
            b']' if depth == 0 => break,
            b']' => depth -= 1,
            _ => {}
        }
        at += 1;
    }
    if bytes.get(at + 1) != Some(&b'(') {
        return None;
    }
    let opens = at + 2;
    // Within `<…>` a `)` is part of the name, so the destination closes only after the `>`.
    let named = opens + (text[opens..].len() - text[opens..].trim_start_matches([' ', '\t']).len());
    let after = match bytes.get(named) {
        Some(b'<') => named + text[named..].find(['>', '\n'])?,
        _ => opens,
    };
    let closes = after + text[after..].find([')', '\n'])?;
    (bytes[closes] == b')').then_some((opens, closes))
}

/// Shares a document's words for the first time: reserves its links, seals and signs it and puts its first version up.
pub(crate) fn share(
    wire: &impl Wire,
    server: &str,
    text: &str,
    pictures: &[(Kind, Vec<u8>)],
    chosen: &Chosen,
    (key, certificate): &Voucher,
    name: Option<&str>,
) -> Result<Shared, Refusal> {
    let limits: Limits = answered(get(wire, "limits".into(), None)?)?;
    let work = worked(wire, json!({ "for": "reserve" }))?;
    let reserved: Reserved = answered(wire.send(Asking {
        method: "POST",
        path: "docs".into(),
        token: None,
        work: Some(work),
        body: Vec::new(),
    })?)?;
    let link_key = LinkKey::minted().ok_or(Refusal::of("internal"))?;
    // Everything shared is signed: the key is the only identity there is, since any alias can be taken.
    let signer = Signer {
        key,
        certificate,
        at: jiff::Timestamp::now(),
        name,
    };
    let pictures: Vec<(Kind, &[u8])> = pictures
        .iter()
        .map(|(kind, bytes)| (*kind, bytes.as_slice()))
        .collect();
    let draft = Draft {
        id: &reserved.id,
        version: 1,
        from: None,
        text,
        pictures: &pictures,
    };
    let sealed =
        envelope::seal(&link_key, &draft, Some(&signer), &limits).map_err(|why| match why {
            // Sealed against the server's own limits, so a document over them is told here, never by the server.
            Unsealed::TextTooBig
            | Unsealed::PictureTooBig
            | Unsealed::PicturesTooBig
            | Unsealed::TooManyPictures => Refusal::of("shareTooBig"),
            _ => Refusal::of("shareFailed"),
        })?;
    let work = worked(
        wire,
        json!({
            "for": "upload",
            "id": reserved.id,
            "digest": URL_SAFE_NO_PAD.encode(sealed.digest()),
            "bytes": sealed.as_bytes().len(),
        }),
    )?;
    let shared = Shared {
        server: server.to_owned(),
        link: reserved.id,
        key: link_key.to_text().to_string(),
        edit: reserved.edit,
        admin: reserved.admin,
        editable: chosen.editable,
        gated: chosen.password.is_some(),
        version: 1,
    };
    let uploaded = wire
        .send(Asking {
            method: "PUT",
            path: format!("docs/{}", shared.link),
            token: Some(&shared.edit),
            work: Some(work),
            body: sealed.into_bytes(),
        })
        .and_then(done);
    if let Err(refused) = uploaded {
        // An answer lost on the way back may still have left it up, and a link nobody kept is a link nobody can stop.
        let _ = stop(wire, &shared);
        return Err(refused);
    }
    if let Err(refused) = guarded(wire, &shared, chosen) {
        // Up without the password or the shorter life it was asked for is not what the person shared, so it goes.
        let _ = stop(wire, &shared);
        return Err(refused);
    }
    Ok(shared)
}

/// The password and the shorter life the person chose, set once the first version is up.
fn guarded(wire: &impl Wire, shared: &Shared, chosen: &Chosen) -> Result<(), Refusal> {
    if let Some(password) = &chosen.password {
        managed(wire, shared, json!({ "do": "password", "with": password }))?;
    }
    if let Some(days) = chosen.idle_days {
        let seconds = i64::from(days) * 86_400;
        managed(wire, shared, json!({ "do": "idle", "with": seconds }))?;
    }
    Ok(())
}

/// How it stands on the server: its reads, when it ends, and whether it is paused or blocked.
pub(crate) fn info(wire: &impl Wire, shared: &Shared) -> Result<Info, Refusal> {
    answered(get(
        wire,
        format!("docs/{}/info", shared.link),
        Some(&shared.admin),
    )?)
}

/// Its links stop leading anywhere; the document itself stays as it is (decision 48).
pub(crate) fn stop(wire: &impl Wire, shared: &Shared) -> Result<(), Refusal> {
    managed(wire, shared, json!({ "do": "delete" }))
}

/// The server no longer holds it: it ran out, or was stopped from another machine.
pub(crate) fn already_gone(refused: &Refusal) -> bool {
    refused.code == "shareRefused" && matches!(refused.name.as_deref(), Some("missing" | "gone"))
}

/// Hands the server a root's word taking keys back, so readers stop seeing what they signed as holding (decision 73).
pub(crate) fn revoke(wire: &impl Wire, revoked: &[Revocation]) -> Result<(), Refusal> {
    // A few at once, so even a person who took back many machines asks the server only a few times.
    for some in revoked.chunks(REVOCATIONS_AT_ONCE) {
        let body = serde_json::to_vec(some).map_err(|_| Refusal::of("internal"))?;
        done(wire.send(Asking {
            method: "POST",
            path: "revocations".into(),
            token: None,
            work: None,
            body,
        })?)?;
    }
    Ok(())
}

const TOLD: &str = "revocations-told.json";

/// Every key this person took back, the root's own signed word on each.
pub(crate) fn taken_back_by(
    persons: &tisty_core::person::Persons,
    me: &tisty_share_proto::identity::Person,
) -> Vec<Revocation> {
    persons
        .revocations
        .iter()
        .filter(|one| one.unchecked().is_some_and(|(person, _)| person == *me))
        .cloned()
        .collect()
}

// A revocation is named by what it signs, so the same word said twice is told once.
fn told_name(revoked: &Revocation) -> String {
    serde_json::to_string(revoked).unwrap_or_default()
}

/// What of these the server has not heard yet, by the list this machine keeps of what it told.
pub(crate) fn untold(cache: &std::path::Path, revoked: Vec<Revocation>) -> Vec<Revocation> {
    let told: std::collections::HashSet<String> = std::fs::read(cache.join(TOLD))
        .ok()
        .and_then(|said| serde_json::from_slice(&said).ok())
        .unwrap_or_default();
    revoked
        .into_iter()
        .filter(|one| !told.contains(&told_name(one)))
        .collect()
}

fn heard(cache: &std::path::Path, revoked: &[Revocation]) {
    let at = cache.join(TOLD);
    let mut told: std::collections::BTreeSet<String> = std::fs::read(&at)
        .ok()
        .and_then(|said| serde_json::from_slice(&said).ok())
        .unwrap_or_default();
    told.extend(revoked.iter().map(told_name));
    if let Ok(said) = serde_json::to_vec(&told) {
        let _ = std::fs::write(at, said);
    }
}

/// Tells the server in the background what it has not heard; what does not reach it is tried again the next round.
pub(crate) fn revoke_untold(cache: std::path::PathBuf, revoked: Vec<Revocation>) {
    // The told list is read off the caller's thread, which may be holding the session.
    tauri::async_runtime::spawn_blocking(move || {
        let revoked = untold(&cache, revoked);
        if revoked.is_empty() {
            return;
        }
        match Net::to(SERVER).and_then(|wire| revoke(&wire, &revoked)) {
            Ok(()) => heard(&cache, &revoked),
            Err(refused) => tisty_core::witness::note(
                tisty_core::witness::channel::WINDOW,
                "Tisty Share did not hear that a key was taken back; the next round tells it",
                &[(
                    "why",
                    tisty_core::witness::Fact::Why(refused.code.to_string()),
                )],
            ),
        }
    });
}

/// The link to read it, and the one to edit it when it was handed out (decision 66).
pub(crate) fn links(shared: &Shared) -> (String, Option<String>) {
    let read = format!(
        "{}/{}#{}",
        shared.server.trim_end_matches('/'),
        shared.link,
        shared.key
    );
    let edit = shared.editable.then(|| format!("{read}.{}", shared.edit));
    (read, edit)
}

#[cfg(test)]
#[path = "sharing_here.rs"]
pub(crate) mod here;

#[cfg(test)]
#[path = "sharing_test.rs"]
mod tests;
