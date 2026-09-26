pub const HUES: &[&str] = &[
    "red", "orange", "amber", "green", "teal", "blue", "indigo", "purple", "pink", "brown", "gray",
];

pub fn kept(said: &str) -> Option<&'static str> {
    HUES.iter().copied().find(|one| *one == said)
}

#[cfg(test)]
#[path = "hue_test.rs"]
mod tests;
