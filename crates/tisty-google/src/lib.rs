use std::time::Duration;

use tisty_carrier::loopback::Asking;
use tisty_carrier::{Hitch, Renew, Renewed};

pub use tisty_carrier::loopback::{Ended, Granted, challenge_of};
pub(crate) use tisty_carrier::said::{WAITED_UNTOLD, told};

mod drive;
pub mod health;

pub use drive::{Account, Drive, DriveAt};

const CONSENT_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const SCOPE: &str = "https://www.googleapis.com/auth/drive.appdata";
pub(crate) const ASKED_WITHIN: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Client {
    pub id: &'static str,
    pub secret: &'static str,
}

// Fixed when the release is built, so a fork or a local build without them offers no cloud.
pub fn client() -> Option<Client> {
    client_of(
        option_env!("TISTY_GOOGLE_CLIENT_ID"),
        option_env!("TISTY_GOOGLE_CLIENT_SECRET"),
    )
}

fn client_of(id: Option<&'static str>, secret: Option<&'static str>) -> Option<Client> {
    match (id, secret) {
        (Some(id), Some(secret)) if !id.is_empty() && !secret.is_empty() => {
            Some(Client { id, secret })
        }
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoints {
    pub token: String,
    pub revoke: String,
}

impl Endpoints {
    pub fn google() -> Self {
        Self {
            token: "https://oauth2.googleapis.com/token".into(),
            revoke: "https://oauth2.googleapis.com/revoke".into(),
        }
    }
}

pub struct Consent {
    asking: Asking,
    client: Client,
}

impl std::ops::Deref for Consent {
    type Target = Asking;
    fn deref(&self) -> &Asking {
        &self.asking
    }
}

impl Consent {
    pub fn begin(client: Client) -> Result<Self, Hitch> {
        let asking = Asking::begin(
            "Google",
            CONSENT_URL,
            false,
            client.id,
            SCOPE,
            &[("access_type", "offline"), ("prompt", "consent")],
        )?;
        Ok(Self { asking, client })
    }

    pub fn finish(self, within: Duration, at: &Endpoints) -> Result<Granted, Ended> {
        let client = self.client;
        self.asking
            .finish(within, |code, asking| traded(client, code, asking, at))
    }
}

fn traded(client: Client, code: &str, asking: &Asking, at: &Endpoints) -> Result<Granted, Hitch> {
    let http = http()?;
    let answer = post_form(
        &http,
        &at.token,
        &[
            ("client_id", client.id),
            ("client_secret", client.secret),
            ("code", code),
            ("code_verifier", &asking.verifier),
            ("grant_type", "authorization_code"),
            ("redirect_uri", &asking.redirect),
        ],
    )?;
    if !only_app_data(answer.scope.as_deref()) {
        revoke(&http, at, &answer);
        return Err(Hitch::Broke(
            "Google did not grant access to just Tisty's own space in Drive, so the sign-in was refused"
                .into(),
        ));
    }
    let Some(refresh) = answer.refresh_token.clone() else {
        revoke(&http, at, &answer);
        return Err(Hitch::Broke("Google gave no lasting access to keep".into()));
    };
    Ok(Granted {
        refresh,
        access: answer.access_token,
        lasts: Duration::from_secs(answer.expires_in),
    })
}

fn only_app_data(scope: Option<&str>) -> bool {
    scope.is_some_and(|granted| {
        granted.split_whitespace().next().is_some()
            && granted.split_whitespace().all(|one| one == SCOPE)
    })
}

#[derive(serde::Deserialize)]
struct TokenAnswer {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
    scope: Option<String>,
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

fn post_form(
    http: &reqwest::blocking::Client,
    url: &str,
    fields: &[(&str, &str)],
) -> Result<TokenAnswer, Hitch> {
    let body = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(fields)
        .finish();
    let answer = http
        .post(url)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send()
        .map_err(crate::health::unreachable)?;
    let status = answer.status();
    let wait = waited(answer.headers());
    let text = answer.text().map_err(crate::health::unreachable)?;
    match status.as_u16() {
        200 => serde_json::from_str(&text).map_err(|_| {
            Hitch::Broke("Google answered the sign-in in a shape Tisty does not read".into())
        }),
        status => Err(tisty_carrier::said::token_refused(
            "Google",
            status,
            wait,
            &text,
            &["invalid_grant"],
        )),
    }
}

// A grant Tisty turns down is handed back, so the account never lists access Tisty said it refused.
fn revoke(http: &reqwest::blocking::Client, at: &Endpoints, granted: &TokenAnswer) {
    let token = granted
        .refresh_token
        .as_deref()
        .unwrap_or(&granted.access_token);
    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("token", token)
        .finish();
    let _ = http
        .post(&at.revoke)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send();
}

/// Takes Tisty's access away for this person on every machine at once, which is how Google revokes.
pub fn revoke_everywhere(token: &str, at: &Endpoints) -> Result<(), Hitch> {
    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("token", token)
        .finish();
    let answer = http()?
        .post(&at.revoke)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send()
        .map_err(crate::health::unreachable)?;
    match answer.status().as_u16() {
        200 => Ok(()),
        // A token Google no longer knows was already revoked, which is what was asked.
        400 => Ok(()),
        429 => Err(Hitch::Limited {
            wait: waited(answer.headers()).unwrap_or(WAITED_UNTOLD),
        }),
        status => Err(Hitch::Troubled(format!("Google answered {status}"))),
    }
}

pub struct GoogleRenew {
    client: Client,
    at: Endpoints,
    http: reqwest::blocking::Client,
}

impl GoogleRenew {
    pub fn new(client: Client, at: Endpoints) -> Result<Self, Hitch> {
        Ok(Self {
            client,
            at,
            http: http()?,
        })
    }
}

impl Renew for GoogleRenew {
    fn renew(&self, refresh: &str) -> Result<Renewed, Hitch> {
        let answer = post_form(
            &self.http,
            &self.at.token,
            &[
                ("client_id", self.client.id),
                ("client_secret", self.client.secret),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh),
            ],
        )?;
        Ok(Renewed {
            access: answer.access_token,
            lasts: Duration::from_secs(answer.expires_in),
            refresh: answer.refresh_token,
        })
    }
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;
