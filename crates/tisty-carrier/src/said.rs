use std::time::Duration;

use crate::{Cause, Hitch};

const WAITED_AT_LEAST: Duration = Duration::from_secs(1);
pub const WAITED_UNTOLD: Duration = Duration::from_secs(60);

/// What went wrong with every cause beneath it, the way a provider's client nests them.
pub fn told(error: &dyn std::error::Error) -> String {
    let mut said = error.to_string();
    let mut under = error.source();
    while let Some(one) = under {
        said.push_str(": ");
        said.push_str(&one.to_string());
        under = one.source();
    }
    said
}

/// A `Retry-After` in seconds, never under one: a provider that says zero still means wait.
pub fn retry_after(said: Option<&str>) -> Option<Duration> {
    said.and_then(|one| one.trim().parse::<u64>().ok())
        .map(|secs| Duration::from_secs(secs).max(WAITED_AT_LEAST))
}

#[derive(serde::Deserialize)]
struct TokenRefusal {
    error: Option<String>,
}

/// A token endpoint's refusal read alike for every provider; `lost` names the errors only signing in again mends.
pub fn token_refused(
    who: &str,
    status: u16,
    wait: Option<Duration>,
    body: &str,
    lost: &[&str],
) -> Hitch {
    match status {
        429 => Hitch::Limited {
            wait: wait.unwrap_or(WAITED_UNTOLD),
        },
        500.. => match wait {
            Some(wait) => Hitch::Limited { wait },
            // Never the provider's store itself, so a round's spending and its stall guard leave it out.
            None => Hitch::Unreachable(None, format!("{who} answered {status}")),
        },
        _ => {
            let error = serde_json::from_str::<TokenRefusal>(body)
                .ok()
                .and_then(|one| one.error)
                .unwrap_or_default();
            match lost.contains(&error.as_str()) {
                true => Hitch::Lost,
                false => Hitch::Broke(format!("{who} refused the token: {error}")),
            }
        }
    }
}

pub fn cause_of(slow: bool, said: &str) -> Cause {
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
        Cause::Filtered
    } else if slow || said.contains("timed out") || said.contains("timeout") {
        Cause::Slow
    } else {
        Cause::Blocked
    }
}

pub struct Asked {
    pub verb: &'static str,
    pub kind: &'static str,
    pub status: Option<u16>,
    pub began: std::time::Instant,
    pub sent: u64,
    pub received: u64,
}

/// One line for each ask at the finest trail: never where it went, what it named or who asked.
pub fn traced(said: &'static str, asked: Asked) {
    use tisty_core::witness::{self, Fact, channel};
    if !witness::keeping_all() {
        return;
    }
    witness::trace(
        channel::CLOUD,
        said,
        &[
            ("verb", Fact::Word(asked.verb)),
            ("kind", Fact::Word(asked.kind)),
            (
                "status",
                asked
                    .status
                    .map_or(Fact::Word("none"), |one| Fact::Count(one.into())),
            ),
            (
                "ms",
                Fact::Count(
                    usize::try_from(asked.began.elapsed().as_millis()).unwrap_or(usize::MAX),
                ),
            ),
            ("sent", Fact::Bytes(asked.sent)),
            ("received", Fact::Bytes(asked.received)),
        ],
    );
}

#[cfg(test)]
#[path = "said_test.rs"]
mod tests;
