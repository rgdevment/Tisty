use std::fs::TryLockError;
use std::path::Path;
use std::time::{Duration, Instant};

const FILE: &str = "round.lock";
const EVERY: Duration = Duration::from_millis(50);

pub struct Round(std::fs::File);

impl Round {
    pub fn take(home: &Path) -> std::io::Result<Option<Self>> {
        std::fs::create_dir_all(home)?;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(home.join(FILE))?;
        match file.try_lock() {
            Ok(()) => Ok(Some(Self(file))),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(TryLockError::Error(why)) => Err(why),
        }
    }

    pub fn wait_for(home: &Path, within: Duration) -> std::io::Result<Option<Self>> {
        let started = Instant::now();
        loop {
            if let Some(round) = Self::take(home)? {
                return Ok(Some(round));
            }
            if started.elapsed() >= within {
                return Ok(None);
            }
            std::thread::sleep(EVERY);
        }
    }
}

impl Drop for Round {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}
