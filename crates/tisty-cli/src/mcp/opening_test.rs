use super::*;

fn lines(count: usize, long: usize) -> Vec<String> {
    let mut all = vec!["head".to_string(), "Your last open tasks here:".to_string()];
    all.extend((0..count).map(|n| format!("- {n} {}", "x".repeat(long))));
    all.push("tail".to_string());
    all
}

#[test]
fn what_fits_goes_out_whole() {
    let said = capped(lines(3, 10));
    assert!(said.starts_with("head\n"));
    assert!(said.ends_with("\ntail"));
    assert_eq!(said.lines().count(), 6);
}

#[test]
fn past_the_cap_whole_list_lines_go_from_the_last_up_and_the_frame_stays() {
    let said = capped(lines(30, 200));
    assert!(said.chars().count() <= SAID_AT_MOST);
    assert!(said.starts_with("head\n"));
    assert!(said.ends_with("\ntail"));
    assert!(said.contains("- 0 "));
    assert!(!said.contains("- 29 "));
    assert!(
        said.lines()
            .all(|one| !one.starts_with("- ") || one.len() > 200)
    );
}

#[test]
fn a_frame_too_long_by_itself_is_cut_at_the_cap() {
    let said = capped(vec!["y".repeat(SAID_AT_MOST * 2)]);
    assert_eq!(said.chars().count(), SAID_AT_MOST);
    assert!(said.ends_with('…'));
}
