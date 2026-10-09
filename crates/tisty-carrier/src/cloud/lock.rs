use std::fs::TryLockError;
use std::path::Path;

const FILE: &str = "round.lock";

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
}

impl Drop for Round {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}
