//! A hint only: the window's own round still finds every change, rung or not.

use crate::Paths;

/// Kept in the cache, never in the data a cloud carries: another machine's agent rings its own bell.
pub fn stir(paths: &Paths) {
    let _ = std::fs::create_dir_all(paths.cache());
    let _ = std::fs::write(
        paths.stirred_file(),
        jiff::Timestamp::now().as_nanosecond().to_string(),
    );
}

/// What the bell says now; a change from the last look means something was written since.
pub fn rung(paths: &Paths) -> Option<String> {
    std::fs::read_to_string(paths.stirred_file()).ok()
}

#[cfg(test)]
#[path = "stirring_test.rs"]
mod tests;
