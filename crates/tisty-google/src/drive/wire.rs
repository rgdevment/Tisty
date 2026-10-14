use std::time::{Duration, Instant};

use reqwest::Method;
use reqwest::blocking::{Client as Http, Request, RequestBuilder, Response};
use serde::Deserialize;
use tisty_carrier::Hitch;
use tisty_core::witness::{self, Fact, channel};

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

// A line cut while the body was still coming is no connection, never a shape Tisty misreads.
pub(super) fn json<T: serde::de::DeserializeOwned>(answer: Response) -> Result<T, Hitch> {
    serde_json::from_reader(answer).map_err(|e| match e.is_io() {
        true => Hitch::Unreachable(
            tisty_carrier::Cause::Blocked,
            "the line was cut while Google Drive was answering".into(),
        ),
        false => Hitch::Broke("Google Drive answered in a shape Tisty does not read".into()),
    })
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
            let asked = asked.build().map_err(crate::health::unreachable)?;
            let shown = witness::keeping_all().then(|| {
                let sent = asked
                    .body()
                    .and_then(|body| body.as_bytes())
                    .map_or(0, |body| body.len() as u64);
                (asked.method().clone(), kind_of(&asked), sent)
            });
            let began = Instant::now();
            let answer = self.http.execute(asked);
            if let Some((verb, kind, sent)) = &shown {
                traced(verb, kind, answer.as_ref().ok(), began, *sent);
            }
            let answer = answer.map_err(crate::health::unreachable)?;
            if answer.status().as_u16() == 401 && !renewed {
                self.tokens.spoiled(&access);
                renewed = true;
                continue;
            }
            return judged(answer);
        }
    }
}

/// One line for each ask at the finest trail: never where it went, what it named or who asked.
pub(super) fn traced(
    verb: &Method,
    kind: &'static str,
    answer: Option<&Response>,
    began: Instant,
    sent: u64,
) {
    if !witness::keeping_all() {
        return;
    }
    witness::trace(
        channel::CLOUD,
        "Google Drive was asked",
        &[
            ("verb", Fact::Word(verb_of(verb))),
            ("kind", Fact::Word(kind)),
            (
                "status",
                answer.map_or(Fact::Word("none"), |one| {
                    Fact::Count(one.status().as_u16().into())
                }),
            ),
            (
                "ms",
                Fact::Count(usize::try_from(began.elapsed().as_millis()).unwrap_or(usize::MAX)),
            ),
            ("sent", Fact::Bytes(sent)),
            (
                "received",
                Fact::Bytes(answer.and_then(Response::content_length).unwrap_or(0)),
            ),
        ],
    );
}

fn verb_of(verb: &Method) -> &'static str {
    match *verb {
        Method::GET => "GET",
        Method::POST => "POST",
        Method::PUT => "PUT",
        Method::PATCH => "PATCH",
        Method::DELETE => "DELETE",
        _ => "OTHER",
    }
}

pub(super) fn kind_of(asked: &Request) -> &'static str {
    let url = asked.url();
    let path = url.path();
    let wanted = |key: &str| {
        url.query_pairs()
            .find(|(one, _)| one == key)
            .map(|(_, value)| value.into_owned())
    };
    let upload = wanted("uploadType");
    match (asked.method(), path) {
        (_, _) if upload.as_deref() == Some("resumable") => "upload-open",
        (_, _) if upload.is_some() => "upload",
        (&Method::DELETE, _) => "delete",
        (_, path) if path.ends_with("/about") => "about",
        (_, path) if path.contains("/changes") => "changes",
        (_, path) if path.contains("/revisions") && wanted("alt").is_some() => "revision-download",
        (_, path) if path.contains("/revisions") => "revisions",
        (&Method::PATCH, _) => "edit",
        (&Method::POST, _) => "folder",
        (_, _) if wanted("alt").is_some() => "download",
        (_, path) if path.ends_with("/files") => match wanted("q") {
            Some(q) if q.contains("name =") => "search",
            _ => "list",
        },
        _ => "get",
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
        (403, "storageQuotaExceeded") => Hitch::Full,
        // Google names a project's quota this way too; only one that speaks of storage is a full Drive.
        (403, "quotaExceeded") if message.to_ascii_lowercase().contains("storage") => Hitch::Full,
        (403, "dailyLimitExceeded" | "quotaExceeded") => Hitch::Limited {
            wait: wait.unwrap_or(DAILY_WAIT),
        },
        (403, "userRateLimitExceeded" | "rateLimitExceeded" | "sharingRateLimitExceeded")
        | (429, _) => Hitch::Limited {
            wait: wait.unwrap_or(WAITED_UNTOLD),
        },
        // A grant from before Tisty kept to its own space cannot read it, and only signing in again mends that.
        (403, "insufficientPermissions" | "insufficientScopes") => Hitch::Lost,
        (403, _)
            if message
                .to_ascii_lowercase()
                .contains("application data folder") =>
        {
            Hitch::Lost
        }
        // Signing in again does not hand Tisty a file another app or person holds, so it stops and says so.
        (403, "appNotAuthorizedToFile" | "insufficientFilePermissions") => Hitch::Broke(format!(
            "Google Drive holds a file in Tisty's space that Tisty may not change: {message}"
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
            None => Hitch::Troubled(format!("Google Drive answered {status}")),
        },
        _ => Hitch::Broke(format!("Google Drive refused: {status} {reason} {message}")),
    })
}
