use std::time::Duration;

use tisty_carrier::{Cause, Hitch};

/// Asked with no account and no token: Google answers 401 at once when the way to it is open.
pub const PROBED: &str = "https://www.googleapis.com/drive/v3/about";
pub const PROBED_WITHIN: Duration = Duration::from_secs(10);
pub const REACHED_WITHIN: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Reachable,
    /// Something between here and Google opens secure connections and signs them itself.
    Inspected,
    Blocked,
    /// Nothing refused the way, but Google took longer than it should to answer on it.
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
pub fn probe() -> Health {
    probe_at(PROBED, REACHED_WITHIN, PROBED_WITHIN)
}

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
    let said = crate::told(&e.without_url());
    match slow {
        true => (Health::Slow, said),
        false => (failed(&said), said),
    }
}

// Only a way that never opened says what stood in it; a line cut mid-answer is tried again with no cause.
pub(crate) fn unreachable(e: reqwest::Error) -> Hitch {
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
    match tisty_carrier::said::cause_of(false, said) {
        Cause::Filtered => Health::Inspected,
        Cause::Slow => Health::Slow,
        _ => Health::Blocked,
    }
}

#[cfg(test)]
#[path = "health_test.rs"]
mod tests;
