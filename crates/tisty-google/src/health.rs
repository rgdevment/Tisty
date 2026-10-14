use std::time::Duration;

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
        Err(e) if e.is_timeout() => Health::Slow,
        Err(e) => failed(&crate::told(&e.without_url())),
    }
}

pub fn answered(status: u16) -> Health {
    match status {
        500.. => Health::Troubled,
        _ => Health::Reachable,
    }
}

pub fn failed(said: &str) -> Health {
    let said = said.to_lowercase();
    let inspected = [
        "certificate",
        "x509",
        "tls",
        "ssl",
        "handshake",
        "trust",
        "unknown issuer",
        "self signed",
        "self-signed",
    ];
    if inspected.iter().any(|one| said.contains(one)) {
        Health::Inspected
    } else if said.contains("timed out") || said.contains("timeout") {
        Health::Slow
    } else {
        Health::Blocked
    }
}

#[cfg(test)]
#[path = "health_test.rs"]
mod tests;
