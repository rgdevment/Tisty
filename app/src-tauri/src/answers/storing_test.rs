use super::*;
use tisty_core::config::Sync;

fn folder(at: &str) -> Option<Sync> {
    Some(Sync::Folder(at.into()))
}

#[test]
fn a_round_that_stopped_keeps_saying_why_while_the_folder_is_the_same() {
    let why = Refusal::about("syncNewer", "dev_b");

    let stuck = stuck_after(Some(&why), &folder("G:/shared"), &folder("G:/shared"))
        .expect("the stop went unsaid");

    assert_eq!(
        (stuck.code, stuck.name.as_deref()),
        ("syncNewer", Some("dev_b"))
    );
}

#[test]
fn a_round_that_got_through_clears_the_stop() {
    assert!(stuck_after(None, &folder("G:/shared"), &folder("G:/shared")).is_none());
}

#[test]
fn a_stop_from_a_folder_left_while_the_round_ran_is_not_raised() {
    let why = Refusal::of("syncNewer");

    assert!(stuck_after(Some(&why), &folder("G:/shared"), &folder("D:/other")).is_none());
    assert!(stuck_after(Some(&why), &folder("G:/shared"), &Some(Sync::Local)).is_none());
}

#[test]
fn having_no_folder_is_no_stop_to_report() {
    assert!(stuck_after(Some(&Refusal::of("noRemote")), &None, &None).is_none());
}

#[test]
fn a_store_restored_apart_from_its_folder_says_so() {
    let why = Refusal::about("restoredApart", "G:/shared");

    assert!(stuck_after(Some(&why), &folder("G:/shared"), &folder("G:/shared")).is_some());
}
