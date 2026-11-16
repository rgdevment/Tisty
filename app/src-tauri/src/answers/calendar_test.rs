use std::time::{Duration, SystemTime};

use tisty_core::config::Calendar;

use super::{READ_WITHIN, chosen, forget_read, opens_here};

fn aged(at: &std::path::Path, by: Duration) {
    std::fs::write(at, "BEGIN:VCALENDAR").unwrap();
    std::fs::File::options()
        .write(true)
        .open(at)
        .unwrap()
        .set_modified(SystemTime::now() - by)
        .unwrap();
}

#[test]
fn a_file_calendar_may_still_be_reading_stays_and_only_long_read_ones_go() {
    let room = tempfile::tempdir().unwrap();
    let (old, fresh, kept) = (
        room.path().join("old.ics"),
        room.path().join("fresh.ics"),
        room.path().join("kept.ics"),
    );
    aged(&old, READ_WITHIN * 2);
    aged(&fresh, Duration::from_secs(5));
    aged(&kept, READ_WITHIN * 2);

    forget_read(room.path(), &kept).unwrap();

    assert!(!old.exists());
    assert!(fresh.exists() && kept.exists());
}

#[test]
fn apple_s_calendar_opens_only_on_a_mac_and_the_web_ones_everywhere() {
    assert!(opens_here(Calendar::Google) && opens_here(Calendar::Outlook));
    assert!(opens_here(Calendar::OutlookWork));
    assert_eq!(opens_here(Calendar::Apple), cfg!(target_os = "macos"));
}

#[test]
fn a_calendar_asked_for_once_must_be_one_tisty_offers_here() {
    assert_eq!(chosen("google").ok(), Some(Calendar::Google));
    assert_eq!(chosen("outlook-work").ok(), Some(Calendar::OutlookWork));
    assert!(chosen("yahoo").is_err());
    assert_eq!(chosen("apple").is_ok(), cfg!(target_os = "macos"));
}
