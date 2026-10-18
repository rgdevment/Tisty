use std::time::Duration;

use tisty_carrier::Hitch;
use tisty_carrier::loopback::{Asking, Ended, Granted};

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
    // Registered as http://localhost, which Microsoft matches on any port.
    pub fn begin(client: Client) -> Result<Self, Hitch> {
        let asking = Asking::begin(
            "Microsoft",
            CONSENT_URL,
            |port| format!("http://localhost:{port}"),
            client.id,
            SCOPES,
            &[("response_mode", "query"), ("prompt", "select_account")],
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
    let answer = posted(
        &http()?,
        &at.token,
        &[
            ("client_id", client.id),
            ("code", code),
            ("code_verifier", &asking.verifier),
            ("grant_type", "authorization_code"),
            ("redirect_uri", &asking.redirect),
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
