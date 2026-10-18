use std::time::Duration;

use tisty_carrier::Hitch;
use tisty_carrier::loopback::{Ended, Loopback, challenge_of, random_text, tell_browser};

use crate::{Client, Endpoints, SCOPES, http, posted};

const CONSENT_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const GRAPH: &str = "https://graph.microsoft.com/";
const ALLOWED: [&str; 6] = [
    "Files.ReadWrite.AppFolder",
    "User.Read",
    "offline_access",
    "openid",
    "profile",
    "email",
];

pub struct Granted {
    pub refresh: String,
    pub access: String,
    pub lasts: Duration,
}

pub struct Consent {
    pub url: String,
    client: Client,
    verifier: String,
    redirect: String,
    loopback: Loopback,
}

impl Consent {
    // Registered as http://localhost, which Microsoft matches on any port.
    pub fn begin(client: Client) -> Result<Self, Hitch> {
        let loopback = Loopback::open("Microsoft")?;
        let redirect = format!("http://localhost:{}", loopback.port);
        let verifier = random_text(48)?;
        let query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", client.id)
            .append_pair("redirect_uri", &redirect)
            .append_pair("response_type", "code")
            .append_pair("response_mode", "query")
            .append_pair("scope", SCOPES)
            .append_pair("code_challenge", &challenge_of(&verifier))
            .append_pair("code_challenge_method", "S256")
            .append_pair("state", &loopback.state)
            .append_pair("prompt", "select_account")
            .finish();
        Ok(Self {
            url: format!("{CONSENT_URL}?{query}"),
            client,
            verifier,
            redirect,
            loopback,
        })
    }

    /// Set from anywhere, it ends the wait for the browser at once.
    pub fn dropping(&self) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
        self.loopback.dropping()
    }

    pub fn finish(self, within: Duration, at: &Endpoints) -> Result<Granted, Ended> {
        let (code, mut browser) = self.loopback.answered(within)?;
        let granted = self.traded(&code, at);
        tell_browser(&mut browser, granted.is_ok());
        granted.map_err(Ended::from)
    }

    fn traded(&self, code: &str, at: &Endpoints) -> Result<Granted, Hitch> {
        let answer = posted(
            &http()?,
            &at.token,
            &[
                ("client_id", self.client.id),
                ("code", code),
                ("code_verifier", &self.verifier),
                ("grant_type", "authorization_code"),
                ("redirect_uri", &self.redirect),
                ("scope", SCOPES),
            ],
        )?;
        if !only_app_folder(answer.scope.as_deref()) {
            return Err(Hitch::Broke(
                "Microsoft did not grant access to just Tisty's own folder in OneDrive, so the sign-in was refused"
                    .into(),
            ));
        }
        let Some(refresh) = answer.refresh_token else {
            return Err(Hitch::Broke(
                "Microsoft gave no lasting access to keep".into(),
            ));
        };
        Ok(Granted {
            refresh,
            access: answer.access_token,
            lasts: Duration::from_secs(answer.expires_in),
        })
    }
}

pub(crate) fn only_app_folder(scope: Option<&str>) -> bool {
    scope.is_some_and(|granted| {
        let named = || {
            granted
                .split_whitespace()
                .map(|one| one.trim_start_matches(GRAPH))
        };
        let known = |one: &str, of: &[&str]| of.iter().any(|it| it.eq_ignore_ascii_case(one));
        named().any(|one| known(one, &ALLOWED[..1])) && named().all(|one| known(one, &ALLOWED))
    })
}

#[cfg(test)]
#[path = "consent_test.rs"]
mod tests;
