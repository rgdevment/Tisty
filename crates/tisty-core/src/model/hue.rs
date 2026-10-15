pub use tisty_markdown::vocab::HUES;

pub fn kept(said: &str) -> Option<&'static str> {
    HUES.iter().copied().find(|one| *one == said)
}

#[cfg(test)]
#[path = "hue_test.rs"]
mod tests;
