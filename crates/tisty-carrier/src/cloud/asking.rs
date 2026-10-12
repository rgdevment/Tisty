use std::time::{Duration, Instant};

use super::{Cloud, now_ms};
use crate::{Hitch, Watch};

// Drive pushes only to a public address and Tisty keeps none, so hearing it is asking at its pace.
pub(super) struct Asking {
    cloud: Cloud,
    mark: Option<u64>,
    asked: Option<Instant>,
}

impl Asking {
    pub(super) fn of(cloud: &Cloud) -> Box<dyn Watch> {
        let last = cloud.stirred_last();
        Box::new(Self {
            cloud: Cloud::over(cloud.remote.clone(), cloud.home.clone()),
            mark: (last.asked_ms > 0).then_some(last.mark),
            asked: None,
        })
    }

    // Kept here too, so a pace that could not be written down never turns into asking without pause.
    fn due_in(&self) -> Duration {
        let last = self.cloud.stirred_last();
        let due = last.asked_ms.saturating_add(last.every_ms);
        let written = Duration::from_millis(due.saturating_sub(now_ms()));
        let ours = self.asked.map_or(Duration::ZERO, |asked| {
            self.cloud
                .remote
                .limits()
                .poll_every
                .saturating_sub(asked.elapsed())
        });
        written.max(ours)
    }
}

impl Watch for Asking {
    fn told(&mut self, within: Duration) -> Result<bool, Hitch> {
        let began = Instant::now();
        loop {
            let left = within.saturating_sub(began.elapsed());
            let due = self.due_in();
            if due > left {
                std::thread::sleep(left);
                return Ok(false);
            }
            std::thread::sleep(due);
            let mark = self.cloud.asked_again();
            self.asked = Some(Instant::now());
            if self.mark.replace(mark).is_some_and(|was| was != mark) {
                return Ok(true);
            }
            if began.elapsed() >= within {
                return Ok(false);
            }
        }
    }
}
