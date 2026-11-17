use std::time::{Duration, Instant};

use tisty_core::Paths;
use tisty_core::brain::{self, Refreshed, Round};
use tisty_core::witness::{self, channel};

use crate::herald::Nudge;

const EVERY: Duration = Duration::from_secs(30);
const LISTENS: Duration = Duration::from_secs(1);
// Typing writes about once a second, and each round reads the whole state again.
const SETTLES: Duration = Duration::from_secs(5);

pub(super) fn keep(paths: Paths, nudge: Nudge, moved: Nudge) {
    std::thread::spawn(move || {
        let mut heard = tisty_core::stirring::rung(&paths);
        let mut written = false;
        let mut last = Instant::now();
        kept(&paths, Round::Starting, &moved);
        loop {
            std::thread::sleep(LISTENS);
            let bell = tisty_core::stirring::rung(&paths);
            written |= nudge.taken() || bell != heard;
            heard = bell;
            if let Some(round) = due(written, last.elapsed()) {
                kept(&paths, round, &moved);
                written = false;
                last = Instant::now();
            }
        }
    });
}

fn due(written: bool, since: Duration) -> Option<Round> {
    match (written, since) {
        (true, since) if since >= SETTLES => Some(Round::Nudged),
        (_, since) if since >= EVERY => Some(Round::Looking),
        _ => None,
    }
}

fn kept(paths: &Paths, round: Round, moved: &Nudge) {
    let done = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        brain::kept_up(paths, round)
    }));
    match done {
        Ok(Some(Refreshed::Moved { .. })) => moved.give(),
        Ok(_) => {}
        Err(_) => witness::error(channel::BRAIN, "the brain could not be kept up", &[]),
    }
}

#[cfg(test)]
#[path = "brain_test.rs"]
mod tests;
