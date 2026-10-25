use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMPS: AtomicU64 = AtomicU64::new(0);

pub(crate) struct Temp(pub(crate) PathBuf);

impl Temp {
    pub(crate) fn new() -> Self {
        let n = TEMPS.fetch_add(1, Ordering::Relaxed);
        let at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        Self(std::env::temp_dir().join(format!("tisty-temp-{}-{n}-{at:x}", std::process::id())))
    }

    pub(crate) fn create(&self) -> std::io::Result<std::fs::File> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        options.open(&self.0)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
