pub const HUES: &[&str] = &[
    "red", "orange", "amber", "green", "teal", "blue", "indigo", "purple", "pink", "brown", "gray",
];

pub fn kept(said: &str) -> Option<&'static str> {
    HUES.iter().copied().find(|one| *one == said)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_colour_outside_the_palette_is_refused() {
        assert!(kept("mauve").is_none());
        assert_eq!(kept("teal"), Some("teal"));
    }

    #[test]
    fn every_name_is_lowercase_ascii_so_it_travels_between_machines() {
        assert!(
            HUES.iter()
                .all(|one| one.chars().all(|c| c.is_ascii_lowercase()))
        );
    }
}
