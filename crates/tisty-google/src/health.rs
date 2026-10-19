use std::time::Duration;

pub use tisty_carrier::net::{Health, answered, failed, kind_of, probe_at, unreachable};

/// Asked with no account and no token: Google answers 401 at once when the way to it is open.
pub const PROBED: &str = "https://www.googleapis.com/drive/v3/about";
pub const PROBED_WITHIN: Duration = Duration::from_secs(10);
pub const REACHED_WITHIN: Duration = Duration::from_secs(5);

pub fn probe() -> Health {
    probe_at(PROBED, REACHED_WITHIN, PROBED_WITHIN)
}

#[cfg(test)]
#[path = "health_test.rs"]
mod tests;
