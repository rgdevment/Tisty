use std::path::Path;

const LEFT_BEHIND_AFTER: i64 = 24 * 60 * 60;

pub fn ours() -> &'static str {
    static OURS: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    OURS.get_or_init(|| ulid::Ulid::generate().to_string())
}

pub fn named(turn: u64) -> String {
    format!(".{}.{turn}.part", ours())
}

pub fn beside(at: &Path, turn: u64) -> std::path::PathBuf {
    at.with_extension(format!("{}.{turn}.part", ours()))
}

pub fn spent(at: &Path, named: &str) -> bool {
    spent_by(at, named, jiff::Timestamp::now().as_second())
}

fn spent_by(at: &Path, named: &str, now: i64) -> bool {
    ours_shape(named) && !named.contains(ours()) && left_behind(at, now)
}

fn ours_shape(named: &str) -> bool {
    let Some(rest) = named.strip_suffix(".part") else {
        return false;
    };
    let mut back = rest.rsplit('.');
    let (Some(turn), Some(mark)) = (back.next(), back.next()) else {
        return false;
    };
    let marked = mark.len() == 26 && mark.bytes().all(|b| b.is_ascii_alphanumeric());
    !turn.is_empty() && turn.bytes().all(|b| b.is_ascii_digit()) && marked
}

fn left_behind(at: &Path, now: i64) -> bool {
    let Ok(when) = at.metadata().and_then(|one| one.modified()) else {
        return false;
    };
    let Ok(since) = when.duration_since(std::time::UNIX_EPOCH) else {
        return false;
    };
    i64::try_from(since.as_secs()).is_ok_and(|since| now - since >= LEFT_BEHIND_AFTER)
}

#[cfg(test)]
#[path = "parting_test.rs"]
mod tests;
