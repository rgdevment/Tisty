use std::time::Duration;

use reqwest::Method;
use reqwest::blocking::{Client as Http, RequestBuilder, Response};
use tisty_carrier::Hitch;
use tisty_carrier::net::{unreachable, waited};
use tisty_carrier::said::{WAITED_UNTOLD, told};
use tisty_core::witness::{self, channel};

use super::WebDav;

const CONNECTED_WITHIN: Duration = Duration::from_secs(30);
const SLOWEST: u64 = 64 * 1024;
const MARGIN: Duration = Duration::from_secs(2 * 60);

pub(super) fn transfer_http() -> Result<Http, Hitch> {
    Http::builder()
        .connect_timeout(CONNECTED_WITHIN)
        // A redirect would carry the password to wherever it points.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| Hitch::Broke(told(&e.without_url())))
}

// A transfer gets what it would take at a crawl, so a slow line finishes and only a dead one is cut.
pub(super) fn given(bytes: u64) -> Duration {
    MARGIN + Duration::from_secs(bytes / SLOWEST)
}

pub(super) fn verb(name: &'static str) -> Method {
    Method::from_bytes(name.as_bytes()).unwrap_or(Method::GET)
}

impl WebDav {
    /// Asked with the person's password, read again once when the server no longer takes it; the answer comes back whatever its status.
    pub(super) fn sent(
        &self,
        kind: &'static str,
        patience: Option<Duration>,
        build: impl Fn(&Http) -> Result<RequestBuilder, Hitch>,
    ) -> Result<Response, Hitch> {
        let mut renewed = false;
        loop {
            let password = self.tokens.access()?;
            let mut asked = build(&self.http)?.basic_auth(self.at.user(), Some(&password));
            if let Some(patience) = patience {
                asked = asked.timeout(patience);
            }
            let asked = asked.build().map_err(unreachable)?;
            let answer = self.traced(asked, kind).map_err(unreachable)?;
            if answer.status().as_u16() == 401 && !renewed {
                // Another copy may have stored a new password since this one read it.
                self.tokens.spoiled(&password);
                renewed = true;
                continue;
            }
            return Ok(answer);
        }
    }

    fn traced(
        &self,
        asked: reqwest::blocking::Request,
        kind: &'static str,
    ) -> reqwest::Result<Response> {
        let verb = tisty_carrier::net::verb_of(asked.method());
        let began = std::time::Instant::now();
        let answer = self.http.execute(asked);
        let held = answer.as_ref().ok();
        tisty_carrier::said::traced(
            "the WebDAV server was asked",
            tisty_carrier::said::Asked {
                verb,
                kind,
                status: held.map(|one| one.status().as_u16()),
                began,
                sent: 0,
                received: held.and_then(Response::content_length).unwrap_or(0),
            },
        );
        answer
    }

    /// A file or folder inside Tisty's own, each part of its name escaped on its own.
    pub(super) fn url_of(&self, name: &str, collection: bool) -> Result<url::Url, Hitch> {
        let mut at = self.at.folder().clone();
        {
            let mut parts = at
                .path_segments_mut()
                .map_err(|()| Hitch::Broke("not a WebDAV address".into()))?;
            parts.pop_if_empty();
            parts.extend(name.split('/').filter(|part| !part.is_empty()));
            if collection {
                parts.push("");
            }
        }
        Ok(at)
    }

    /// The name inside Tisty's folder an address a server answered with stands for, or None when it lies outside it.
    pub(super) fn name_of(&self, href: &str) -> Option<String> {
        let decoded = |path: &str| {
            percent_encoding::percent_decode_str(path)
                .decode_utf8()
                .ok()
                .map(|one| one.trim_end_matches('/').to_string())
        };
        let path = decoded(self.at.folder().join(href).ok()?.path())?;
        let folder = decoded(self.at.folder().path())?;
        match path == folder {
            true => Some(String::new()),
            false => path
                .strip_prefix(&format!("{folder}/"))
                .map(|inside| inside.trim_matches('/').to_string()),
        }
    }
}

pub(super) fn judged(answer: Response) -> Result<Response, Hitch> {
    if answer.status().is_success() {
        return Ok(answer);
    }
    Err(refusal(answer.status().as_u16(), waited(answer.headers())))
}

pub(super) fn refusal(status: u16, wait: Option<Duration>) -> Hitch {
    match status {
        401 => {
            witness::warn(
                channel::CLOUD,
                "the WebDAV server no longer takes the password",
                &[],
            );
            Hitch::Lost
        }
        403 => Hitch::Broke("the WebDAV server refused what was asked".into()),
        // A folder that is not there answers 409 to what is written inside it.
        404 | 409 => Hitch::Missing(String::new()),
        412 => Hitch::Changed(String::new()),
        507 => Hitch::Full,
        // A file locked by someone else, or a pace the server asks for, passes on its own.
        423 | 429 => Hitch::Limited {
            wait: wait.unwrap_or(WAITED_UNTOLD),
        },
        500.. => match wait {
            Some(wait) => Hitch::Limited { wait },
            None => Hitch::Troubled(format!("the WebDAV server answered {status}")),
        },
        _ => Hitch::Broke(format!("the WebDAV server refused: {status}")),
    }
}

/// Only a strong ETag can be sent back in `If-Match`; a weak one, or none, is checked by asking first.
pub(super) fn strong(revision: &str) -> bool {
    revision.starts_with('"')
}

#[cfg(test)]
#[path = "wire_test.rs"]
mod tests;
