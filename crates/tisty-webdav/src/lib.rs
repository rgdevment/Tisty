use std::time::Duration;

use tisty_carrier::{Hitch, Renew, Renewed};

mod dav;
mod names;

pub use dav::WebDav;
pub use names::fitting;

/// The folder Tisty keeps to itself inside the one the person names, so nothing of theirs is listed or touched.
pub const FOLDER: &str = "Tisty";
pub(crate) const ASKED_WITHIN: Duration = Duration::from_secs(30);
// The password never lapses on its own, so it is held in memory for a working day before the keychain is read again.
const HELD_FOR: Duration = Duration::from_secs(12 * 60 * 60);

/// Where Tisty's folder lives on a WebDAV server, and whose it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DavAt {
    folder: url::Url,
    user: String,
}

impl DavAt {
    /// The folder the person named, with Tisty's own made inside it.
    pub fn under(given: &str, user: &str) -> Result<Self, Hitch> {
        let mut folder = parsed(given.trim())?;
        if !folder.username().is_empty() || folder.password().is_some() {
            return Err(Hitch::Broke(
                "a WebDAV address carries no user or password of its own".into(),
            ));
        }
        folder
            .path_segments_mut()
            .map_err(|_| not_an_address())?
            .pop_if_empty()
            .push(FOLDER)
            .push("");
        Self::of(folder, user)
    }

    /// Read back from the account it was filed under, which names the folder and its user.
    pub fn of_account(account: &str) -> Result<Self, Hitch> {
        let mut folder = parsed(account)?;
        let user = percent_encoding::percent_decode_str(folder.username())
            .decode_utf8()
            .map_err(|_| not_an_address())?
            .to_string();
        folder.set_username("").map_err(|()| not_an_address())?;
        Self::of(folder, &user)
    }

    fn of(folder: url::Url, user: &str) -> Result<Self, Hitch> {
        if user.trim().is_empty() {
            return Err(Hitch::Broke("a WebDAV account needs a user".into()));
        }
        if !folder.path().ends_with('/') {
            return Err(not_an_address());
        }
        Ok(Self {
            folder,
            user: user.trim().to_string(),
        })
    }

    /// The folder and its user in one address, so a server and a person never mix with another pair.
    pub fn account(&self) -> String {
        let mut named = self.folder.clone();
        let _ = named.set_username(&self.user);
        named.into()
    }

    /// What the person reads: who they are and on which server.
    pub fn shown(&self) -> String {
        format!(
            "{}@{}",
            self.user,
            self.folder.host_str().unwrap_or_default()
        )
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub(crate) fn folder(&self) -> &url::Url {
        &self.folder
    }
}

// The password travels with every request, so it only ever crosses an encrypted line; this machine alone is spared.
fn parsed(given: &str) -> Result<url::Url, Hitch> {
    let at = url::Url::parse(given).map_err(|_| not_an_address())?;
    let local = matches!(
        at.host(),
        Some(url::Host::Ipv4(ip)) if ip.is_loopback()
    ) || matches!(at.host(), Some(url::Host::Ipv6(ip)) if ip.is_loopback())
        || at.host_str() == Some("localhost");
    match at.scheme() {
        "https" => Ok(at),
        "http" if local => Ok(at),
        "http" => Err(Hitch::Broke(
            "a WebDAV server is reached only over https".into(),
        )),
        _ => Err(not_an_address()),
    }
    .and_then(|at| match at.query().is_some() || at.fragment().is_some() {
        true => Err(not_an_address()),
        false => Ok(at),
    })
}

fn not_an_address() -> Hitch {
    Hitch::Broke("not a WebDAV address".into())
}

/// The app password kept in the keychain is the credential itself: renewing it only reads it back.
pub struct Password;

impl Renew for Password {
    fn renew(&self, kept: &str) -> Result<Renewed, Hitch> {
        Ok(Renewed {
            access: kept.to_string(),
            lasts: HELD_FOR,
            refresh: None,
        })
    }
}

/// Asked with no user and no password: any server that answers at all, even refusing, is reachable.
pub fn probe(at: &DavAt) -> tisty_carrier::net::Health {
    tisty_carrier::net::probe(at.folder.as_str())
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;
