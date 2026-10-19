use std::time::Duration;

use crate::said::{cause_of, retry_after, token_refused, told};
use crate::{Cause, Hitch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Reachable,
    /// Something between here and the provider opens secure connections and signs them itself.
    Inspected,
    Blocked,
    /// Nothing refused the way, but the provider took longer than it should to answer on it.
    Slow,
    Troubled,
}

impl Health {
    pub fn code(self) -> &'static str {
        match self {
            Health::Reachable => "reachable",
            Health::Inspected => "inspected",
            Health::Blocked => "blocked",
            Health::Slow => "slow",
            Health::Troubled => "troubled",
        }
    }

    pub fn cause(self) -> Cause {
        match self {
            Health::Inspected => Cause::Filtered,
            Health::Slow => Cause::Slow,
            Health::Reachable | Health::Blocked | Health::Troubled => Cause::Blocked,
        }
    }
}

/// Goes through whatever proxy the system names, like every other request: never around it.
pub fn probe_at(url: &str, reached_within: Duration, within: Duration) -> Health {
    let Ok(http) = reqwest::blocking::Client::builder()
        .timeout(within)
        .connect_timeout(reached_within)
        .build()
    else {
        return Health::Blocked;
    };
    match http.get(url).send() {
        Ok(answer) => answered(answer.status().as_u16()),
        Err(e) => kind_of(e).0,
    }
}

/// One reading of a failed request, shared by the check before connecting and every round after it.
pub fn kind_of(e: reqwest::Error) -> (Health, String) {
    let slow = e.is_timeout();
    let said = told(&e.without_url());
    // A filter that stalls the handshake until it times out is still a filter.
    match (failed(&said), slow) {
        (Health::Inspected, _) => (Health::Inspected, said),
        (_, true) => (Health::Slow, said),
        (health, false) => (health, said),
    }
}

pub fn verb_of(verb: &reqwest::Method) -> &'static str {
    match *verb {
        reqwest::Method::GET => "GET",
        reqwest::Method::POST => "POST",
        reqwest::Method::PUT => "PUT",
        reqwest::Method::PATCH => "PATCH",
        reqwest::Method::DELETE => "DELETE",
        _ => "OTHER",
    }
}

// Only a way that never opened says what stood in it; a line cut mid-answer is tried again with no cause.
pub fn unreachable(e: reqwest::Error) -> Hitch {
    let opening = e.is_connect() || e.is_timeout();
    let (health, said) = kind_of(e);
    Hitch::Unreachable(opening.then(|| health.cause()), said)
}

pub fn answered(status: u16) -> Health {
    match status {
        500.. => Health::Troubled,
        _ => Health::Reachable,
    }
}

pub fn failed(said: &str) -> Health {
    match cause_of(false, said) {
        Cause::Filtered => Health::Inspected,
        Cause::Slow => Health::Slow,
        _ => Health::Blocked,
    }
}

// Blocking on purpose: every carrier call already runs off the window's async runtime.
pub fn client(within: Duration) -> Result<reqwest::blocking::Client, Hitch> {
    reqwest::blocking::Client::builder()
        .timeout(within)
        .build()
        .map_err(|e| Hitch::Broke(told(&e.without_url())))
}

pub fn waited(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    retry_after(
        headers
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|one| one.to_str().ok()),
    )
}

/// What a sign-in or a renewal asks of a provider's token address, read the same way for every provider.
pub fn token_asked<T: serde::de::DeserializeOwned>(
    http: &reqwest::blocking::Client,
    who: &str,
    url: &str,
    lost: &[&str],
    fields: &[(&str, &str)],
) -> Result<T, Hitch> {
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
        .map_err(unreachable)?;
    let status = answer.status().as_u16();
    let wait = waited(answer.headers());
    let text = answer.text().map_err(unreachable)?;
    match status {
        200 => serde_json::from_str(&text).map_err(|_| {
            Hitch::Broke(format!(
                "{who} answered the sign-in in a shape Tisty does not read"
            ))
        }),
        status => Err(token_refused(who, status, wait, &text, lost)),
    }
}
