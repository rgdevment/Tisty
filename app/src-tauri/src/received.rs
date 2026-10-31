use tisty_share_proto::envelope::{self, Envelope, LinkKey};
use tisty_share_proto::identity::{Revocation, Verdict};
use tisty_share_proto::limits::Limits;

use crate::refusing::Refusal;
use crate::sharing::{self, Wire};

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
    if shared.scheme() != "https"
        || shared.host_str() != server.host_str()
        || shared.port().is_some()
    {
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

/// Fetches it and opens it with its key; what its writer did not sign, or signed for another link, is not shown.
pub fn fetched(wire: &impl Wire, link: &Link) -> Result<Received, Refusal> {
    let said = sharing::get(wire, format!("docs/{}", link.id), None)?;
    match said.status {
        200..=299 => {}
        401 | 428 => return Err(Refusal::of("shareOpenGated")),
        404 | 410 => return Err(Refusal::of("shareOpenGone")),
        423 | 451 => return Err(Refusal::of("shareOpenClosed")),
        _ => return Err(Refusal::of("shareFailed")),
    }
    let key = LinkKey::from_text(&link.key).ok_or(Refusal::of("shareOpenUnsigned"))?;
    let envelope =
        Envelope::read(said.body, &Limits::default()).map_err(|_| Refusal::of("shareFailed"))?;
    let opened = envelope::open(&key, &envelope, &taken_back(said.revoked.as_deref()))
        .map_err(|_| Refusal::of("shareOpenUnsigned"))?;
    if !matches!(
        opened.wrote,
        Some(Verdict::Holds { .. } | Verdict::Revoked { .. })
    ) {
        return Err(Refusal::of("shareOpenUnsigned"));
    }
    let body = opened.text.to_string();
    Ok(Received {
        title: tisty_core::docs::titled(&body),
        signer: opened.written.and_then(|one| one.name),
        body,
    })
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
