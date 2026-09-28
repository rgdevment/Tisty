use std::sync::atomic::{AtomicU64, Ordering};

static OPENED: AtomicU64 = AtomicU64::new(0);

pub fn opened() {
    OPENED.fetch_add(1, Ordering::Relaxed);
}

pub fn opens() -> u64 {
    OPENED.load(Ordering::Relaxed)
}

pub fn from_now() -> u64 {
    OPENED.swap(0, Ordering::Relaxed)
}
