use std::time::Duration;

use reqwest::blocking::{Client as Http, RequestBuilder, Response};
use serde::Deserialize;
use tisty_carrier::Hitch;
use tisty_core::witness::{self, Fact, channel};

use super::OneDrive;
use crate::{WAITED_UNTOLD, unreachable, waited};
use tisty_carrier::said::told;

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
            let asked = asked.build().map_err(unreachable)?;
            let answer = self.traced(asked, None).map_err(unreachable)?;
            match (answer.status().as_u16(), renewed) {
                (401, false) => {
                    witness::note(
                        channel::CLOUD,
                        "OneDrive no longer took the token, so it was renewed",
                        &[],
                    );
                    self.tokens.spoiled(&access);
                    renewed = true;
                    continue;
                }
                (401, true) => witness::warn(
                    channel::CLOUD,
                    "OneDrive refused the token even once renewed",
                    &[],
                ),
                _ => {}
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

    pub(super) fn traced(
        &self,
        asked: reqwest::blocking::Request,
        kind: Option<&'static str>,
    ) -> reqwest::Result<Response> {
        let verb = tisty_carrier::net::verb_of(asked.method());
        let kind = kind.unwrap_or_else(|| kind_of(&asked));
        let sent = asked
            .body()
            .and_then(|body| body.as_bytes())
            .map_or(0, |body| body.len() as u64);
        let began = std::time::Instant::now();
        let answer = self.http.execute(asked);
        let held = answer.as_ref().ok();
        tisty_carrier::said::traced(
            "OneDrive was asked",
            tisty_carrier::said::Asked {
                verb,
                kind,
                status: held.map(|one| one.status().as_u16()),
                began,
                sent,
                received: held.and_then(Response::content_length).unwrap_or(0),
            },
        );
        answer
    }

    // Download and upload addresses carry their own permission, so they are taken only from Microsoft's hosts.
    pub(super) fn carries(&self, link: &str) -> Result<(), Hitch> {
        let api = self.at.api.parse::<url::Url>().ok();
        let ok = link.parse::<url::Url>().is_ok_and(|at| {
            let host = at.host_str().unwrap_or_default();
            let same = api
                .as_ref()
                .is_some_and(|api| api.scheme() == at.scheme() && api.host_str() == Some(host));
            let theirs = at.scheme() == "https"
                && CARRIERS
                    .iter()
                    .any(|one| host == *one || host.ends_with(&format!(".{one}")));
            same || theirs
        });
        match ok {
            true => Ok(()),
            false => Err(Hitch::Broke(
                "OneDrive handed an address outside Microsoft's own".into(),
            )),
        }
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

// The folder itself refused is the grant to it gone, not one file inside it.
pub(super) fn judged_whole(answer: Response) -> Result<Response, Hitch> {
    if answer.status().as_u16() != 403 {
        return judged(answer);
    }
    let (status, wait, refused) = read_refusal(answer);
    Err(match refusal(status, wait, refused.as_ref()) {
        Hitch::Broke(_) => {
            let codes = refused.as_ref().map(codes_of).unwrap_or_default();
            witness::warn(
                channel::CLOUD,
                "OneDrive refused the app folder itself, so the sign-in has to be given again",
                &[
                    ("status", Fact::Count(status.into())),
                    ("codes", Fact::Id(codes.join("/"))),
                ],
            );
            Hitch::Lost
        }
        other => other,
    })
}

// An address that carries its own permission and refuses has lapsed; the sign-in still stands.
pub(super) fn judged_carried(answer: Response) -> Result<Response, Hitch> {
    judged(answer).map_err(|refused| match refused {
        Hitch::Lost => Hitch::Troubled("OneDrive refused an address it had handed out".into()),
        other => other,
    })
}

pub(super) fn judged(answer: Response) -> Result<Response, Hitch> {
    if answer.status().is_success() {
        return Ok(answer);
    }
    let (status, wait, refused) = read_refusal(answer);
    Err(refusal(status, wait, refused.as_ref()))
}

fn read_refusal(answer: Response) -> (u16, Option<Duration>, Option<Refused>) {
    let status = answer.status().as_u16();
    let wait = waited(answer.headers());
    let refused = serde_json::from_reader::<_, Refusal>(answer)
        .ok()
        .map(|one| one.error);
    (status, wait, refused)
}

fn refusal(status: u16, wait: Option<Duration>, refused: Option<&Refused>) -> Hitch {
    let codes = refused.map(codes_of).unwrap_or_default();
    let message = refused.map(|one| one.message.clone()).unwrap_or_default();
    let names = |code: &str| codes.contains(&code);
    match status {
        401 => Hitch::Lost,
        _ if names("quotaLimitReached") => Hitch::Full,
        507 => Hitch::Full,
        // A token Microsoft no longer takes answers 401 even once renewed; a refusal here is about one thing, not the sign-in.
        403 if names("accessDenied") => Hitch::Broke(format!(
            "OneDrive holds something in Tisty's folder that Tisty may not touch: {message}"
        )),
        404 => Hitch::Missing(message),
        409 | 412 => Hitch::Changed(message),
        // A file held by someone else, or a pace Microsoft asks for, passes on its own.
        423 | 429 | 509 => Hitch::Limited {
            wait: wait.unwrap_or(WAITED_UNTOLD),
        },
        _ if names("activityLimitReached") => Hitch::Limited {
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

const CARRIERS: [&str; 8] = [
    "1drv.com",
    "microsoftpersonalcontent.com",
    "svc.ms",
    "onedrive.com",
    "live.com",
    "livefilestore.com",
    "sharepoint.com",
    "microsoft.com",
];

fn kind_of(asked: &reqwest::blocking::Request) -> &'static str {
    let path = asked.url().path();
    match *asked.method() {
        reqwest::Method::DELETE => "delete",
        reqwest::Method::PATCH => "edit",
        _ if path.ends_with("createUploadSession") => "upload-open",
        _ if path.ends_with("/content") => "upload",
        _ if path.contains("delta") => "changes",
        _ if path.ends_with("/drive") => "about",
        _ if path.ends_with("/me") => "account",
        _ => "read",
    }
}
