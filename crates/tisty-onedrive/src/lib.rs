use std::time::Duration;

use tisty_carrier::{Hitch, Renew, Renewed};

pub(crate) use tisty_carrier::said::WAITED_UNTOLD;

mod consent;
mod graph;
mod names;
pub mod quickxor;

pub use consent::Consent;
pub use graph::{Account, GraphAt, OneDrive};
pub use names::fitting;
pub use tisty_carrier::loopback::Ended;
pub use tisty_carrier::loopback::Granted;

pub const SCOPES: &str = "Files.ReadWrite.AppFolder offline_access User.Read";
pub(crate) const ASKED_WITHIN: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Client {
    pub id: &'static str,
}

// A public client: Microsoft gives it no secret, and a build made without the id offers no OneDrive.
pub fn client() -> Option<Client> {
    client_of(option_env!("TISTY_MICROSOFT_CLIENT_ID"))
}

fn client_of(id: Option<&'static str>) -> Option<Client> {
    id.filter(|id| !id.trim().is_empty())
        .map(|id| Client { id })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoints {
    pub token: String,
}

impl Endpoints {
    pub fn microsoft() -> Self {
        Self {
            token: "https://login.microsoftonline.com/consumers/oauth2/v2.0/token".into(),
        }
    }
}

pub(crate) fn http() -> Result<reqwest::blocking::Client, Hitch> {
    tisty_carrier::net::client(ASKED_WITHIN)
}

pub(crate) use tisty_carrier::net::{unreachable, waited};

/// Asked with no account and no token: Microsoft answers 401 at once when the way to it is open.
pub const PROBED: &str = "https://graph.microsoft.com/v1.0/me";

pub fn probe() -> tisty_carrier::net::Health {
    tisty_carrier::net::probe(PROBED)
}

#[derive(serde::Deserialize)]
pub(crate) struct TokenAnswer {
    pub(crate) access_token: String,
    pub(crate) expires_in: u64,
    pub(crate) refresh_token: Option<String>,
    #[serde(default)]
    pub(crate) scope: Option<String>,
}

pub(crate) fn posted(
    http: &reqwest::blocking::Client,
    url: &str,
    lost: &[&str],
    fields: &[(&str, &str)],
) -> Result<TokenAnswer, Hitch> {
    tisty_carrier::net::token_asked(http, "Microsoft", url, lost, fields)
}

pub struct MicrosoftRenew {
    client: Client,
    at: Endpoints,
    http: reqwest::blocking::Client,
}

impl MicrosoftRenew {
    pub fn new(client: Client, at: Endpoints) -> Result<Self, Hitch> {
        Ok(Self {
            client,
            at,
            http: http()?,
        })
    }
}

// Microsoft hands back a new refresh token on every use and may stop taking the old one, so it always travels back to be kept.
impl Renew for MicrosoftRenew {
    fn renew(&self, refresh: &str) -> Result<Renewed, Hitch> {
        let granted = posted(
            &self.http,
            &self.at.token,
            &["invalid_grant", "interaction_required", "consent_required"],
            &[
                ("client_id", self.client.id),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh),
                ("scope", SCOPES),
            ],
        )?;
        Ok(Renewed {
            access: granted.access_token,
            lasts: Duration::from_secs(granted.expires_in),
            refresh: granted.refresh_token,
        })
    }
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;
