use serde::Serialize;
use tisty_share_proto::envelope::{Envelope, Kind, LinkKey, Refused, open};
use tisty_share_proto::identity::Verdict;
use tisty_share_proto::limits::Limits;
use zeroize::Zeroizing;

/// A version as its reader meets it: the page drawn as Tisty draws it, its pictures, and what is known of it.
pub struct Shown {
    pub html: String,
    pub about: About,
    pub pictures: Vec<(Kind, Zeroizing<Vec<u8>>)>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct About {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub version: u64,
    /// Who wrote these words and pictures, when the envelope carries their signature.
    pub wrote: Option<Signed>,
    /// Who sent these bytes as this version, when the envelope carries that signature.
    pub sent: Option<Signed>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "is", rename_all = "snake_case")]
pub enum Signed {
    Holds {
        fingerprint: String,
        since: i64,
    },
    /// The key was taken back, so nothing it signed can be told from a thief's.
    Revoked {
        fingerprint: String,
        at: i64,
    },
    Refused,
    Unreadable,
}

/// Why a link could not be read, as one word the page translates.
#[derive(Debug, PartialEq, Eq)]
pub enum Unread {
    /// The link carries no key, or one that is not a key.
    Keyless,
    /// The server's limits did not read.
    Limitless,
    Envelope(Refused),
}

impl Unread {
    pub fn word(&self) -> &'static str {
        match self {
            Self::Keyless => "keyless",
            Self::Limitless => "limitless",
            Self::Envelope(Refused::Locked) => "locked",
            Self::Envelope(Refused::Newer) => "newer",
            Self::Envelope(_) => "damaged",
        }
    }
}

/// Opens what the server sent with the key from the link, under the limits the server publishes.
pub fn shown(envelope: Vec<u8>, key: &str, limits: &str) -> Result<Shown, Unread> {
    let key = LinkKey::from_text(key).ok_or(Unread::Keyless)?;
    let limits: Limits = serde_json::from_str(limits).map_err(|_| Unread::Limitless)?;
    let envelope = Envelope::read(envelope, &limits).map_err(Unread::Envelope)?;
    // The web learns of no revocation; a revoked key shows as holding until Tisty is asked.
    let opened = open(&key, &envelope, &[]).map_err(Unread::Envelope)?;
    let html = tisty_markdown::html(&tisty_markdown::read(&opened.text));
    let about = About {
        title: tisty_markdown::front::title(&opened.text),
        tags: tisty_markdown::front::tags(&opened.text),
        version: opened.manifest.version,
        wrote: opened.wrote.map(signed),
        sent: opened.sent.map(signed),
    };
    Ok(Shown {
        html,
        about,
        pictures: opened.pictures,
    })
}

fn signed(verdict: Verdict) -> Signed {
    match verdict {
        Verdict::Holds { person, since } => Signed::Holds {
            fingerprint: person.fingerprint(),
            since: since.as_second(),
        },
        Verdict::Revoked { person, at } => Signed::Revoked {
            fingerprint: person.fingerprint(),
            at: at.as_second(),
        },
        Verdict::Refused => Signed::Refused,
        Verdict::Unreadable => Signed::Unreadable,
    }
}

#[cfg(test)]
#[path = "shown_test.rs"]
mod tests;
