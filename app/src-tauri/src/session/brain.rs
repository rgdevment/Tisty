use std::time::{Duration, Instant};

use tisty_core::Paths;
use tisty_core::brain::{self, Round};
use tisty_core::witness::{self, channel};

use crate::herald::Nudge;

const EVERY: Duration = Duration::from_secs(30);
const LISTENS: Duration = Duration::from_secs(1);

pub(super) fn keep(paths: Paths, nudge: Nudge) {
    std::thread::spawn(move || {
        let mut heard = tisty_core::stirring::rung(&paths);
        let mut round = Instant::now();
        kept(&paths, Round::Starting);
        loop {
            std::thread::sleep(LISTENS);
            let bell = tisty_core::stirring::rung(&paths);
            let written = nudge.taken() || bell != heard;
            if !written && round.elapsed() < EVERY {
                continue;
            }
            heard = bell;
            round = Instant::now();
            kept(
                &paths,
                if written {
                    Round::Nudged
                } else {
                    Round::Looking
                },
            );
        }
    });
}

fn kept(paths: &Paths, round: Round) {
    let done = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        brain::kept_up(paths, round)
    }));
    if done.is_err() {
        witness::error(channel::BRAIN, "the brain could not be kept up", &[]);
    }
}
