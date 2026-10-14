use std::time::Duration;

use tisty_carrier::{Cause, Hitch, Renew, Renewed};

pub(crate) use tisty_carrier::said::{WAITED_UNTOLD, told};

mod graph;
mod names;
pub mod quickxor;

pub use graph::{Account, GraphAt, OneDrive};
pub use names::fitting;

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

// Blocking on purpose: every carrier call already runs off the window's async runtime.
pub(crate) fn http() -> Result<reqwest::blocking::Client, Hitch> {
    reqwest::blocking::Client::builder()
        .timeout(ASKED_WITHIN)
        .build()
        .map_err(|e| Hitch::Broke(told(&e.without_url())))
}

pub(crate) fn waited(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    tisty_carrier::said::retry_after(
        headers
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|one| one.to_str().ok()),
    )
}

// Only a way that never opened says what stood in it; a line cut mid-answer is tried again with no cause.
pub(crate) fn unreachable(e: reqwest::Error) -> Hitch {
    let opening = e.is_connect() || e.is_timeout();
    let slow = e.is_timeout();
    let said = told(&e.without_url());
    Hitch::Unreachable(opening.then(|| cause_of(slow, &said)), said)
}

pub(crate) fn cause_of(slow: bool, said: &str) -> Cause {
    let said = said.to_lowercase();
    let inspected = [
        "certificate",
        "x509",
        "tls",
        "ssl",
        "handshake",
        "trust",
        "unknown issuer",
        "self signed",
        "self-signed",
    ];
    if inspected.iter().any(|one| said.contains(one)) {
        Cause::Filtered
    } else if slow || said.contains("timed out") || said.contains("timeout") {
        Cause::Slow
    } else {
        Cause::Blocked
    }
}

#[derive(serde::Deserialize)]
struct TokenAnswer {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
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
        let body = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", self.client.id)
            .append_pair("grant_type", "refresh_token")
            .append_pair("refresh_token", refresh)
            .append_pair("scope", SCOPES)
            .finish();
        let answer = self
            .http
            .post(&self.at.token)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(body)
            .send()
            .map_err(unreachable)?;
        let status = answer.status().as_u16();
        let wait = waited(answer.headers());
        let text = answer.text().map_err(unreachable)?;
        match status {
            200 => {
                let granted: TokenAnswer = serde_json::from_str(&text).map_err(|_| {
                    Hitch::Broke(
                        "Microsoft answered the sign-in in a shape Tisty does not read".into(),
                    )
                })?;
                Ok(Renewed {
                    access: granted.access_token,
                    lasts: Duration::from_secs(granted.expires_in),
                    refresh: granted.refresh_token,
                })
            }
            status => Err(tisty_carrier::said::token_refused(
                "Microsoft",
                status,
                wait,
                &text,
                &["invalid_grant", "interaction_required", "consent_required"],
            )),
        }
    }
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;
