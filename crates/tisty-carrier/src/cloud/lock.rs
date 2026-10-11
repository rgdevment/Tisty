use std::fs::TryLockError;
use std::path::Path;
use std::time::{Duration, Instant};

const ROUND: &str = "round.lock";
pub const RENEWING: &str = "renewing.lock";
const EVERY: Duration = Duration::from_millis(50);

pub struct Held(std::fs::File);

impl Held {
    pub fn take(home: &Path, named: &str) -> std::io::Result<Option<Self>> {
        std::fs::create_dir_all(home)?;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(home.join(named))?;
        match file.try_lock() {
            Ok(()) => Ok(Some(Self(file))),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(TryLockError::Error(why)) => Err(why),
        }
    }

    pub fn wait_for(home: &Path, named: &str, within: Duration) -> std::io::Result<Option<Self>> {
        let started = Instant::now();
        loop {
            if let Some(held) = Self::take(home, named)? {
                return Ok(Some(held));
            }
            if started.elapsed() >= within {
                return Ok(None);
            }
            std::thread::sleep(EVERY);
        }
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

pub struct Round;

impl Round {
    pub fn take(home: &Path) -> std::io::Result<Option<Held>> {
        Held::take(home, ROUND)
    }

    pub fn wait_for(home: &Path, within: Duration) -> std::io::Result<Option<Held>> {
        Held::wait_for(home, ROUND, within)
    }
}
