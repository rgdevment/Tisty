use std::time::Duration;

/// Asked with no account and no token: Google answers 401 at once when the way to it is open.
pub const PROBED: &str = "https://www.googleapis.com/drive/v3/about";
pub const PROBED_WITHIN: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Reachable,
    /// Something between here and Google opens secure connections and signs them itself.
    Inspected,
    Blocked,
    Troubled,
}

impl Health {
    pub fn code(self) -> &'static str {
        match self {
            Health::Reachable => "reachable",
            Health::Inspected => "inspected",
            Health::Blocked => "blocked",
            Health::Troubled => "troubled",
        }
    }
}

/// Goes through whatever proxy the system names, like every other request: never around it.
pub fn probe() -> Health {
    probe_at(PROBED, PROBED_WITHIN)
}

pub fn probe_at(url: &str, within: Duration) -> Health {
    let Ok(http) = reqwest::blocking::Client::builder()
        .timeout(within)
        .connect_timeout(within)
        .build()
    else {
        return Health::Blocked;
    };
    match http.get(url).send() {
        Ok(answer) => answered(answer.status().as_u16()),
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
    match inspected.iter().any(|one| said.contains(one)) {
        true => Health::Inspected,
        false => Health::Blocked,
    }
}

#[cfg(test)]
#[path = "health_test.rs"]
mod tests;
