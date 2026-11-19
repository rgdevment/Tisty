use super::*;

#[test]
fn a_meaning_keeps_one_byte_a_value_and_comes_back_as_written() {
    let meant = Meant::of(&[0.5, -1.0, 0.25]);
    assert_eq!(meant.values(), [64, -127, 32]);
    assert!((meant.scale() - 1.0 / 127.0).abs() < 1e-9);
    let said = meant.said();
    assert_eq!(said.len(), 4 + 3);
    assert_eq!(Meant::read(&said), Some(meant));
}

#[test]
fn a_meaning_of_nothing_says_nothing() {
    let nothing = Meant::of(&[0.0, 0.0]);
    assert!(!nothing.says_something());
    assert_eq!(nothing.values(), [0, 0]);
    assert!(!Meant::of(&[]).says_something());
    assert!(Meant::of(&[0.1]).says_something());
}

#[test]
fn bytes_too_short_for_a_scale_are_no_meaning() {
    assert_eq!(Meant::read(&[1, 2, 3]), None);
    assert_eq!(
        Meant::read(&1f32.to_le_bytes()).map(|one| one.values().len()),
        Some(0)
    );
}
