use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tisty_core::event::Shared;
use tisty_share_proto::envelope::{self, Draft, LinkKey, Signer};
use tisty_share_proto::identity::Revocation;
use tisty_share_proto::limits::Limits;
use tisty_share_proto::work::{self, Challenge};

use crate::handed::Voucher;
use crate::refusing::Refusal;

/// Tisty Share as Tisty knows it out of the box; a person running their own server names it in the settings.
pub(crate) const SERVER: &str = "https://tisty.cl";
const API: &str = "/api/v1/";
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(30);

/// What came back from one request.
pub(crate) struct Said {
    pub(crate) status: u16,
    pub(crate) body: Vec<u8>,
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
        let body = answer.bytes().map_err(|_| Refusal::of("shareOffline"))?;
        Ok(Said {
            status,
            body: body.to_vec(),
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
        .and_then(|value| value["refused"].as_str().map(str::to_owned))
        .unwrap_or_else(|| said.status.to_string());
    // The one no the person can act on: this machine's key was taken back or never held.
    if word == "unsigned" {
        return Refusal::of("shareUnsigned");
    }
    Refusal::about("shareRefused", word)
}

fn answered<T: for<'de> Deserialize<'de>>(said: Said) -> Result<T, Refusal> {
    if !(200..300).contains(&said.status) {
        return Err(refused(&said));
    }
    serde_json::from_slice(&said.body).map_err(|_| Refusal::of("shareRefused"))
}

fn done(said: Said) -> Result<(), Refusal> {
    match (200..300).contains(&said.status) {
        true => Ok(()),
        false => Err(refused(&said)),
    }
}

fn get(wire: &impl Wire, path: String, token: Option<&str>) -> Result<Said, Refusal> {
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
    let proof = work::solve(&challenge).ok_or(Refusal::of("shareRefused"))?;
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

/// Shares a document's words for the first time: reserves its links, seals and signs it and puts its first version up.
pub(crate) fn share(
    wire: &impl Wire,
    server: &str,
    text: &str,
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
    let draft = Draft {
        id: &reserved.id,
        version: 1,
        from: None,
        text,
        pictures: &[],
    };
    let sealed = envelope::seal(&link_key, &draft, Some(&signer), &limits)
        .map_err(|why| Refusal::about("shareRefused", format!("{why:?}")))?;
    let work = worked(
        wire,
        json!({
            "for": "upload",
            "id": reserved.id,
            "digest": URL_SAFE_NO_PAD.encode(sealed.digest()),
            "bytes": sealed.as_bytes().len(),
        }),
    )?;
    done(wire.send(Asking {
        method: "PUT",
        path: format!("docs/{}", reserved.id),
        token: Some(&reserved.edit),
        work: Some(work),
        body: sealed.into_bytes(),
    })?)?;
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
    if let Some(password) = &chosen.password {
        managed(wire, &shared, json!({ "do": "password", "with": password }))?;
    }
    if let Some(days) = chosen.idle_days {
        let seconds = i64::from(days) * 86_400;
        managed(wire, &shared, json!({ "do": "idle", "with": seconds }))?;
    }
    Ok(shared)
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
    for one in revoked {
        let body = serde_json::to_vec(one).map_err(|_| Refusal::of("internal"))?;
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

/// Tells the server in the background; one that cannot be reached hears it with the next share.
pub(crate) fn revoke_later(revoked: Vec<Revocation>) {
    if revoked.is_empty() {
        return;
    }
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(refused) = Net::to(SERVER).and_then(|wire| revoke(&wire, &revoked)) {
            tisty_core::witness::note(
                tisty_core::witness::channel::WINDOW,
                "Tisty Share did not hear that a key was taken back; the next share tells it",
                &[(
                    "why",
                    tisty_core::witness::Fact::Why(refused.code.to_string()),
                )],
            );
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
#[path = "sharing_test.rs"]
mod tests;
