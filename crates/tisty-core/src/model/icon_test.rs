use super::*;

#[test]
fn a_name_outside_the_catalogue_is_refused() {
    assert!(kept("unicorn").is_none());
    assert_eq!(kept("home").as_deref(), Some("home"));
}

#[test]
fn every_key_is_lowercase_ascii_so_it_travels_between_machines() {
    assert!(
        ICONS
            .iter()
            .all(|key| key.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
    );
}

#[test]
fn no_key_is_written_twice() {
    let mut seen: Vec<&str> = ICONS.to_vec();
    seen.sort_unstable();
    let many = seen.len();
    seen.dedup();
    assert_eq!(seen.len(), many);
}

#[test]
fn the_families_cover_the_catalogue_exactly_once() {
    let counted: usize = FAMILIES.iter().map(|(_, many)| many).sum();
    assert_eq!(counted, ICONS.len());
}

#[test]
fn no_family_is_empty_and_none_is_named_twice() {
    let mut seen: Vec<&str> = FAMILIES.iter().map(|(name, _)| *name).collect();
    let many = seen.len();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), many);
    assert!(FAMILIES.iter().all(|(_, held)| *held > 0));
}
