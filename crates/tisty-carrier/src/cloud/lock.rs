use std::path::Path;

const FILE: &str = "round.lock";

pub struct Round(std::fs::File);

impl Round {
    pub fn take(home: &Path) -> Option<Self> {
        std::fs::create_dir_all(home).ok()?;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(home.join(FILE))
            .ok()?;
        file.try_lock().ok()?;
        Some(Self(file))
    }
}

impl Drop for Round {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}
