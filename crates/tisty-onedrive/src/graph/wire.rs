use std::time::Duration;

use reqwest::blocking::{Client as Http, RequestBuilder, Response};
use serde::Deserialize;
use tisty_carrier::Hitch;

use super::OneDrive;
use crate::{WAITED_UNTOLD, told, unreachable, waited};

const CONNECTED_WITHIN: Duration = Duration::from_secs(30);
const SLOWEST: u64 = 64 * 1024;
const MARGIN: Duration = Duration::from_secs(2 * 60);

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

// A line cut while the body was still coming is no connection, never a shape Tisty misreads.
pub(super) fn json<T: serde::de::DeserializeOwned>(answer: Response) -> Result<T, Hitch> {
    serde_json::from_reader(answer).map_err(|e| match e.is_io() {
        true => Hitch::Unreachable(None, "the line was cut while OneDrive was answering".into()),
        false => Hitch::Broke("OneDrive answered in a shape Tisty does not read".into()),
    })
}

impl OneDrive {
    /// Asked with the access token, renewed once when OneDrive no longer takes it; the answer comes back whatever its status.
    pub(super) fn sent(
        &self,
        patience: Option<std::time::Duration>,
        build: impl Fn(&Http) -> RequestBuilder,
    ) -> Result<Response, Hitch> {
        let mut renewed = false;
        loop {
            let access = self.tokens.access()?;
            let mut asked = build(&self.http).bearer_auth(&access);
            if let Some(patience) = patience {
                asked = asked.timeout(patience);
            }
            let answer = asked.send().map_err(unreachable)?;
            if answer.status().as_u16() == 401 && !renewed {
                self.tokens.spoiled(&access);
                renewed = true;
                continue;
            }
            return Ok(answer);
        }
    }

    pub(super) fn call(
        &self,
        patience: Option<std::time::Duration>,
        build: impl Fn(&Http) -> RequestBuilder,
    ) -> Result<Response, Hitch> {
        judged(self.sent(patience, build)?)
    }

    /// The app's folder by a path inside it, which Graph takes between colons.
    pub(super) fn by_path(&self, name: &str, then: Option<&str>) -> Result<String, Hitch> {
        let mut at = url::Url::parse(&format!("{}/me/drive/special/", self.at.api))
            .map_err(|_| Hitch::Broke("not a OneDrive address".into()))?;
        {
            let mut parts = at
                .path_segments_mut()
                .map_err(|_| Hitch::Broke("not a OneDrive address".into()))?;
            parts.pop_if_empty().push("approot:");
            let mut names = name.split('/').peekable();
            while let Some(part) = names.next() {
                match names.peek() {
                    Some(_) => parts.push(part),
                    None => parts.push(&format!("{part}:")),
                };
            }
            if let Some(then) = then {
                parts.push(then);
            }
        }
        Ok(at.into())
    }

    // Graph hands back whole addresses to carry on from; the token only ever goes where Graph itself lives.
    pub(super) fn ours(&self, link: &str) -> Result<(), Hitch> {
        match link.starts_with(&format!("{}/", self.at.api)) {
            true => Ok(()),
            false => Err(Hitch::Broke(
                "OneDrive pointed somewhere other than itself".into(),
            )),
        }
    }
}

#[derive(Deserialize)]
struct Refusal {
    error: Refused,
}

#[derive(Deserialize)]
struct Refused {
    #[serde(default)]
    code: String,
    #[serde(default)]
    message: String,
    #[serde(default, alias = "innererror", rename = "innerError")]
    inner: Option<Box<Refused>>,
}

// Graph nests ever finer codes, and the coarse one is all a client is promised; every one is looked at.
fn codes_of(refused: &Refused) -> Vec<&str> {
    let mut codes = vec![refused.code.as_str()];
    let mut under = refused.inner.as_deref();
    while let Some(one) = under {
        codes.push(one.code.as_str());
        under = one.inner.as_deref();
    }
    codes
}

pub(super) fn judged(answer: Response) -> Result<Response, Hitch> {
    if answer.status().is_success() {
        return Ok(answer);
    }
    let status = answer.status().as_u16();
    let wait = waited(answer.headers());
    let refused = serde_json::from_reader::<_, Refusal>(answer)
        .ok()
        .map(|one| one.error);
    Err(refusal(status, wait, refused.as_ref()))
}

fn refusal(status: u16, wait: Option<Duration>, refused: Option<&Refused>) -> Hitch {
    let codes = refused.map(codes_of).unwrap_or_default();
    let message = refused.map(|one| one.message.clone()).unwrap_or_default();
    let names = |code: &str| codes.contains(&code);
    match status {
        401 => Hitch::Lost,
        _ if names("quotaLimitReached") => Hitch::Full,
        507 => Hitch::Full,
        // Signing in again is what hands Tisty its folder back.
        403 if names("accessDenied") || names("unauthenticated") => Hitch::Lost,
        404 => Hitch::Missing(message),
        409 | 412 => Hitch::Changed(message),
        429 | 509 => Hitch::Limited {
            wait: wait.unwrap_or(WAITED_UNTOLD),
        },
        500.. => match wait {
            Some(wait) => Hitch::Limited { wait },
            None => Hitch::Troubled(format!("OneDrive answered {status}")),
        },
        _ => Hitch::Broke(format!(
            "OneDrive refused: {status} {} {message}",
            codes.join("/")
        )),
    }
}
