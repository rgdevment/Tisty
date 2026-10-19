pub use tisty_carrier::net::{
    Health, PROBED_WITHIN, REACHED_WITHIN, answered, failed, kind_of, probe_at, unreachable,
};

/// Asked with no account and no token: Google answers 401 at once when the way to it is open.
pub const PROBED: &str = "https://www.googleapis.com/drive/v3/about";

pub fn probe() -> Health {
    tisty_carrier::net::probe(PROBED)
}

#[cfg(test)]
#[path = "health_test.rs"]
mod tests;
