use std::time::Duration;

use reqwest::blocking::{Client as Http, RequestBuilder, Response};
use serde::Deserialize;
use tisty_carrier::Hitch;

use super::Drive;
use crate::{WAITED_UNTOLD, told, waited};

const CONNECTED_WITHIN: Duration = Duration::from_secs(30);
const SLOWEST: u64 = 64 * 1024;
const MARGIN: Duration = Duration::from_secs(2 * 60);
const DAILY_WAIT: Duration = Duration::from_secs(60 * 60);

pub(super) fn transfer_http() -> Result<Http, Hitch> {
    Http::builder()
        .connect_timeout(CONNECTED_WITHIN)
        .build()
        .map_err(|e| Hitch::Broke(told(&e.without_url())))
}

// A transfer gets what it would take at a crawl, so a slow line finishes and only a dead one is cut.
pub(super) fn given(bytes: u64) -> Duration {
    MARGIN + Duration::from_secs(bytes / SLOWEST)
}

pub(super) fn quoted(text: &str) -> String {
    text.replace('\\', "\\\\").replace('\'', "\\'")
}

pub(super) fn json<T: serde::de::DeserializeOwned>(answer: Response) -> Result<T, Hitch> {
    serde_json::from_reader(answer)
        .map_err(|_| Hitch::Broke("Google Drive answered in a shape Tisty does not read".into()))
}

impl Drive {
    pub(super) fn call(
        &self,
        patience: Option<Duration>,
        build: impl Fn(&Http) -> RequestBuilder,
    ) -> Result<Response, Hitch> {
        let mut renewed = false;
        loop {
            let access = self.tokens.access()?;
            let mut asked = build(&self.http).bearer_auth(&access);
            if let Some(patience) = patience {
                asked = asked.timeout(patience);
            }
            let answer = asked
                .send()
                .map_err(|e| Hitch::Unreachable(told(&e.without_url())))?;
            if answer.status().as_u16() == 401 && !renewed {
                self.tokens.spoiled(&access);
                renewed = true;
                continue;
            }
            return judged(answer);
        }
    }
}

#[derive(Deserialize)]
struct Refusal {
    error: RefusalBody,
}

#[derive(Deserialize)]
struct RefusalBody {
    #[serde(default)]
    message: String,
    #[serde(default)]
    errors: Vec<Reasoned>,
}

#[derive(Deserialize)]
struct Reasoned {
    #[serde(default)]
    reason: String,
}

pub(super) fn judged(answer: Response) -> Result<Response, Hitch> {
    let status = answer.status().as_u16();
    if answer.status().is_success() {
        return Ok(answer);
    }
    let wait = waited(answer.headers());
    let refusal = serde_json::from_reader::<_, Refusal>(answer)
        .ok()
        .map(|one| one.error);
    let reason = refusal
        .as_ref()
        .and_then(|one| one.errors.first())
        .map(|one| one.reason.as_str())
        .unwrap_or_default();
    let message = refusal
        .as_ref()
        .map(|one| one.message.clone())
        .unwrap_or_default();
    Err(match (status, reason) {
        (401, _) => Hitch::Lost,
        (403, "storageQuotaExceeded" | "quotaExceeded") => Hitch::Full,
        (403, "dailyLimitExceeded") => Hitch::Limited {
            wait: wait.unwrap_or(DAILY_WAIT),
        },
        (403, "userRateLimitExceeded" | "rateLimitExceeded" | "sharingRateLimitExceeded")
        | (429, _) => Hitch::Limited {
            wait: wait.unwrap_or(WAITED_UNTOLD),
        },
        (403, "insufficientPermissions") => Hitch::Lost,
        // Signing in again does not hand Tisty a file another app or person holds, so it stops and says so.
        (403, "appNotAuthorizedToFile" | "insufficientFilePermissions") => Hitch::Broke(format!(
            "Google Drive holds a file in the Tisty folder that Tisty may not change: {message}"
        )),
        (403, "domainPolicy") => Hitch::Broke(
            "the administrators of this Google account turned off apps like Tisty in Drive".into(),
        ),
        (
            403,
            "activeItemCreationLimitExceeded"
            | "numChildrenInNonRootLimitExceeded"
            | "myDriveHierarchyDepthLimitExceeded",
        ) => Hitch::Full,
        (404, _) => Hitch::Missing(message),
        (412, _) => Hitch::Changed(message),
        (500.., _) => match wait {
            Some(wait) => Hitch::Limited { wait },
            None => Hitch::Unreachable(format!("Google Drive answered {status}")),
        },
        _ => Hitch::Broke(format!("Google Drive refused: {status} {reason} {message}")),
    })
}
