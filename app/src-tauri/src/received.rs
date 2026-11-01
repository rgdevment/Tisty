use std::path::Path;

use tisty_share_proto::envelope::{self, Envelope, Kind, LinkKey};
use tisty_share_proto::identity::{Revocation, Verdict};
use tisty_share_proto::limits::Limits;

use crate::refusing::Refusal;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use crate::sharing::{self, Asking, Said, Wire};

/// A Tisty Share link Tisty was asked to open: `tisty://share?link=…`, as the reader's «Open in Tisty» hands it over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub id: String,
    /// The key after the `#`; an edit token after it is left behind, since Tisty only reads a share for now.
    pub key: String,
}

impl Link {
    /// The read link as the person was given it, to open in a browser when Tisty cannot show it.
    pub fn read(&self) -> String {
        format!("{}/{}#{}", sharing::SERVER, self.id, self.key)
    }
}

/// What a shared document says, opened here with the key in its link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Received {
    pub title: String,
    pub body: String,
    /// The name its writer signed with, when they chose one.
    pub signer: Option<String>,
    /// The pictures the words name as `tisty:picture/N`, in their order in the envelope.
    pub pictures: Vec<(Kind, Vec<u8>)>,
}

// Whatever reaches Tisty from outside is a third party's word: only a link to Tisty Share's own server is fetched.
pub fn asked(said: &str) -> Option<Link> {
    let link = url::Url::parse(said.trim()).ok()?;
    if link.scheme() != "tisty"
        || link.host_str() != Some("share")
        || !matches!(link.path(), "" | "/")
    {
        return None;
    }
    let mut links = link.query_pairs().filter(|(key, _)| key == "link");
    let (_, shared) = links.next()?;
    if links.next().is_some() {
        return None;
    }
    let shared = url::Url::parse(&shared).ok()?;
    let server = url::Url::parse(sharing::SERVER).ok()?;
    let host = server.host_str()?;
    // The same server answers on www, and the web hands over the address it was read at.
    let ours = shared
        .host_str()
        .is_some_and(|one| one == host || one.strip_prefix("www.") == Some(host));
    if shared.scheme() != "https" || !ours || shared.port().is_some() {
        return None;
    }
    let id = shared.path().strip_prefix('/')?;
    let fair = (6..=32).contains(&id.len()) && id.chars().all(|one| one.is_ascii_alphanumeric());
    let key = shared.fragment()?.split('.').next()?;
    (fair && LinkKey::from_text(key).is_some()).then(|| Link {
        id: id.to_owned(),
        key: key.to_owned(),
    })
}

pub fn links(argv: &[String]) -> Vec<Link> {
    argv.iter().skip(1).filter_map(|one| asked(one)).collect()
}

fn read(
    wire: &impl Wire,
    link: &Link,
    password: Option<String>,
    work: Option<String>,
) -> Result<Said, Refusal> {
    wire.send(Asking {
        method: "GET",
        path: format!("docs/{}", link.id),
        token: None,
        work,
        password,
        body: Vec::new(),
    })
}

/// Fetches it and opens it with its key; what its writer did not sign, or signed for another link, is not shown.
pub fn fetched(wire: &impl Wire, link: &Link, password: Option<&str>) -> Result<Received, Refusal> {
    let password = password.map(|one| URL_SAFE_NO_PAD.encode(one));
    let mut said = read(wire, link, password.clone(), None)?;
    // A document guessed at asks for a little work beside the password, done once and only then (decision 76).
    if said.status == 428 && password.is_some() {
        let work = sharing::worked(
            wire,
            serde_json::json!({ "for": "password", "id": link.id }),
        )?;
        said = read(wire, link, password, Some(work))?;
    }
    let word = sharing::refused(&said).name;
    match said.status {
        200..=299 => {}
        401 if word.as_deref() == Some("password_wrong") => {
            return Err(Refusal::of("shareOpenWrong"));
        }
        401 | 428 => return Err(Refusal::of("shareOpenGated")),
        404 | 410 => return Err(Refusal::of("shareOpenGone")),
        429 => return Err(Refusal::of("shareSlow")),
        423 | 451 => return Err(Refusal::of("shareOpenClosed")),
        _ => return Err(Refusal::of("shareFailed")),
    }
    let key = LinkKey::from_text(&link.key).ok_or(Refusal::of("shareOpenUnsigned"))?;
    let envelope =
        Envelope::read(said.body, &Limits::default()).map_err(|_| Refusal::of("shareFailed"))?;
    let opened = envelope::open(&key, &envelope, &taken_back(said.revoked.as_deref()))
        .map_err(|_| Refusal::of("shareOpenUnsigned"))?;
    match opened.wrote {
        Some(Verdict::Holds { .. }) => {}
        // Its writer took that key back: the web says so beside it, and here it would read as theirs.
        Some(Verdict::Revoked { .. }) => return Err(Refusal::of("shareOpenRevoked")),
        _ => return Err(Refusal::of("shareOpenUnsigned")),
    }
    let body = opened.text.to_string();
    Ok(Received {
        title: tisty_core::docs::titled(&body),
        signer: opened.written.and_then(|one| one.name),
        pictures: opened
            .pictures
            .iter()
            .map(|(kind, bytes)| (*kind, bytes.to_vec()))
            .collect(),
        body,
    })
}

/// The words with each picture kept among the attachments and named by its file there, and the lines saying they came.
pub fn kept_pictures(got: &Received, data: &Path, scratch: &Path) -> (String, Vec<tisty_core::Op>) {
    let mut body = got.body.clone();
    let mut told = Vec::new();
    // Last first, so `tisty:picture/1` is never taken for the start of `tisty:picture/10`.
    for (at, (kind, bytes)) in got.pictures.iter().enumerate().rev() {
        let from = scratch.join(format!("picture-{at}.{}", extension(*kind)));
        if std::fs::write(&from, bytes).is_err() {
            continue;
        }
        let kept = tisty_core::attach::keep(&from, data, tisty_core::attach::COPIED_IN_DOC);
        let _ = std::fs::remove_file(&from);
        let Ok(kept) = kept else {
            continue;
        };
        told.push(kept.told(tisty_core::attach::Via::Imported));
        let leaf = tisty_core::attach::leaf_of(&kept.at).unwrap_or(&kept.at);
        body = body.replace(&format!("{}{at}", sharing::PICTURE), leaf);
    }
    (body, told)
}

fn extension(kind: Kind) -> &'static str {
    match kind {
        Kind::Jpeg => "jpg",
        Kind::Webp => "webp",
        Kind::Gif => "gif",
        _ => "png",
    }
}

// Only what a root signed is believed, as the web reader does: the server can leave a revocation out, never make one up.
fn taken_back(said: Option<&str>) -> Vec<Revocation> {
    let mut revoked: Vec<Revocation> = said
        .and_then(|said| serde_json::from_str(said).ok())
        .unwrap_or_default();
    revoked.retain(|one| one.holds().is_ok());
    revoked
}

#[cfg(test)]
#[path = "received_test.rs"]
mod tests;
