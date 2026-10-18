use super::{Kind, Odd, Unread, read, taken};
use crate::model::{Cadence, Priority, Repeat, Unit};
use jiff::Zoned;
use jiff::civil::date;

fn now() -> Zoned {
    date(2026, 10, 18)
        .at(12, 0, 0, 0)
        .in_tz("America/Santiago")
        .unwrap()
}

fn one(body: &str) -> super::Taken {
    let text = format!("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n{body}END:VCALENDAR\r\n");
    let calendar = read(&text).unwrap();
    assert_eq!(calendar.items.len(), 1, "{calendar:?}");
    taken(&calendar.items[0], &now())
}

#[test]
fn a_folded_and_escaped_event_reads_as_one_title_place_and_note() {
    let said = one(
        "BEGIN:VEVENT\r\nUID:a1\r\nSUMMARY:Vet for Rina\\, the\r\n  booster\r\nLOCATION:Av. 1\\; piso 2\r\nDESCRIPTION:Bring the card\\nand the leash\r\nDTSTART;TZID=America/Santiago:20261022T100000\r\nEND:VEVENT\r\n",
    );

    assert_eq!(said.title, "Vet for Rina, the booster");
    assert_eq!(said.place, "Av. 1; piso 2");
    assert_eq!(
        said.described.as_deref(),
        Some("Av. 1; piso 2\n\nBring the card\nand the leash")
    );
    assert_eq!(said.source.as_deref(), Some("ics:a1"));
    let at = said.date.unwrap();
    assert_eq!(at.at, date(2026, 10, 22).at(10, 0, 0, 0));
    assert!(at.has_time && !at.floating);
    assert_eq!(
        said.reminder.unwrap().at,
        date(2026, 10, 22).at(9, 30, 0, 0),
        "half an hour before, with no alarm of its own"
    );
}

#[test]
fn a_time_from_another_zone_lands_on_this_clock() {
    let said = one(
        "BEGIN:VEVENT\r\nUID:b\r\nSUMMARY:Call Madrid\r\nDTSTART;TZID=Europe/Madrid:20261022T160000\r\nEND:VEVENT\r\n",
    );
    assert_eq!(said.date.unwrap().at, date(2026, 10, 22).at(11, 0, 0, 0));
    let utc =
        one("BEGIN:VEVENT\r\nUID:c\r\nSUMMARY:Sync\r\nDTSTART:20261022T150000Z\r\nEND:VEVENT\r\n");
    assert_eq!(utc.date.unwrap().at, date(2026, 10, 22).at(12, 0, 0, 0));
    let outlook = one(
        "BEGIN:VEVENT\r\nUID:d\r\nSUMMARY:Board\r\nDTSTART;TZID=Pacific Standard Time:20261022T090000\r\nEND:VEVENT\r\n",
    );
    assert_eq!(outlook.date.unwrap().at, date(2026, 10, 22).at(13, 0, 0, 0));
    let odd = one(
        "BEGIN:VEVENT\r\nUID:d2\r\nSUMMARY:Board\r\nDTSTART;TZID=Atlantis Time:20261022T090000\r\nEND:VEVENT\r\n",
    );
    assert!(odd.odd.contains(&Odd::Zone));
    assert!(odd.date.unwrap().floating);
}

#[test]
fn a_character_split_across_a_fold_comes_back_whole() {
    let mut bytes = b"BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:o\r\nSUMMARY:Reuni".to_vec();
    let accent = "ó".as_bytes();
    bytes.push(accent[0]);
    bytes.extend_from_slice(b"\r\n ");
    bytes.push(accent[1]);
    bytes.extend_from_slice(b"n\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n");

    let calendar = super::read_bytes(&bytes).unwrap();

    assert_eq!(calendar.items[0].summary, "Reunión");
}

#[test]
fn a_cancellation_is_shown_and_never_taken() {
    let text = "BEGIN:VCALENDAR\nMETHOD:CANCEL\nBEGIN:VEVENT\nUID:p\nSUMMARY:Standup\nDTSTART:20261022T150000Z\nEND:VEVENT\nEND:VCALENDAR\n";
    let item = &read(text).unwrap().items[0];
    let said = taken(item, &now());
    assert!(said.odd.contains(&Odd::Cancelled) && !said.takeable());
    let status = one(
        "BEGIN:VEVENT\r\nUID:q\r\nSUMMARY:Off\r\nSTATUS:CANCELLED\r\nDTSTART:20261022T150000Z\r\nEND:VEVENT\r\n",
    );
    assert!(!status.takeable());
}

#[test]
fn a_series_begun_long_ago_starts_at_its_next_turn_and_an_ended_one_is_past() {
    let running = one(
        "BEGIN:VEVENT\r\nUID:r\r\nSUMMARY:Standup\r\nDTSTART;TZID=America/Santiago:20240101T090000\r\nRRULE:FREQ=WEEKLY\r\nEND:VEVENT\r\n",
    );
    assert_eq!(running.date.unwrap().at, date(2026, 10, 19).at(9, 0, 0, 0));
    assert!(!running.odd.contains(&Odd::Past));
    let ended = one(
        "BEGIN:VEVENT\r\nUID:s\r\nSUMMARY:Course\r\nDTSTART;TZID=America/Santiago:20240101T090000\r\nRRULE:FREQ=WEEKLY;COUNT=4\r\nEND:VEVENT\r\n",
    );
    assert_eq!(ended.repeat, None);
    assert!(ended.odd.contains(&Odd::Past) && ended.takeable());
}

#[test]
fn today_is_not_past_and_a_huge_count_or_alarm_breaks_nothing() {
    let today = one(
        "BEGIN:VEVENT\r\nUID:t\r\nSUMMARY:Today\r\nDTSTART;VALUE=DATE:20261018\r\nEND:VEVENT\r\n",
    );
    assert!(!today.odd.contains(&Odd::Past));
    let huge = one(
        "BEGIN:VEVENT\r\nUID:u\r\nSUMMARY:Forever\r\nDTSTART;VALUE=DATE:20261020\r\nRRULE:FREQ=YEARLY;COUNT=4000000000\r\nBEGIN:VALARM\r\nTRIGGER:-P99999999999W\r\nEND:VALARM\r\nEND:VEVENT\r\n",
    );
    assert_eq!(huge.title, "Forever");
}

#[test]
fn a_to_do_s_alarm_counts_from_its_start_unless_it_says_its_end() {
    let start = one(
        "BEGIN:VTODO\r\nUID:v\r\nSUMMARY:Report\r\nDTSTART;TZID=America/Santiago:20261019T090000\r\nDUE;TZID=America/Santiago:20261023T170000\r\nBEGIN:VALARM\r\nTRIGGER:-PT15M\r\nEND:VALARM\r\nEND:VTODO\r\n",
    );
    assert_eq!(
        start.reminder.unwrap().at,
        date(2026, 10, 19).at(8, 45, 0, 0)
    );
    let end = one(
        "BEGIN:VTODO\r\nUID:w\r\nSUMMARY:Report\r\nDTSTART;TZID=America/Santiago:20261019T090000\r\nDUE;TZID=America/Santiago:20261023T170000\r\nBEGIN:VALARM\r\nTRIGGER;RELATED=END:-PT1H\r\nEND:VALARM\r\nEND:VTODO\r\n",
    );
    assert_eq!(end.reminder.unwrap().at, date(2026, 10, 23).at(16, 0, 0, 0));
}

#[test]
fn an_entry_with_no_identity_is_known_by_what_it_says() {
    let body = "BEGIN:VEVENT\r\nSUMMARY:Lunch\r\nDTSTART:20261022T150000Z\r\nEND:VEVENT\r\n";
    let (once, again) = (one(body), one(body));
    assert!(once.source.as_deref().unwrap().starts_with("ics:~"));
    assert_eq!(once.source, again.source);
}

#[test]
fn a_whole_day_is_reminded_the_evening_before_and_an_alarm_of_its_own_wins() {
    let day = one(
        "BEGIN:VEVENT\r\nUID:e\r\nSUMMARY:Holiday\r\nDTSTART;VALUE=DATE:20261025\r\nEND:VEVENT\r\n",
    );
    let at = day.date.unwrap();
    assert!(!at.has_time);
    assert_eq!(day.reminder.unwrap().at, date(2026, 10, 24).at(18, 0, 0, 0));
    let alarmed = one(
        "BEGIN:VEVENT\r\nUID:f\r\nSUMMARY:Dentist\r\nDTSTART;TZID=America/Santiago:20261022T100000\r\nBEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-P1DT2H\r\nEND:VALARM\r\nEND:VEVENT\r\n",
    );
    assert_eq!(
        alarmed.reminder.unwrap().at,
        date(2026, 10, 21).at(8, 0, 0, 0)
    );
}

#[test]
fn simple_repeats_are_kept_and_the_rest_are_said() {
    let weekly = one(
        "BEGIN:VEVENT\r\nUID:g\r\nSUMMARY:Gym\r\nDTSTART;TZID=America/Santiago:20261019T070000\r\nRRULE:FREQ=WEEKLY;INTERVAL=2;BYDAY=MO;COUNT=3\r\nEND:VEVENT\r\n",
    );
    assert_eq!(
        weekly.repeat,
        Some(Repeat {
            until: Some(date(2026, 11, 16)),
            ..Repeat::due(Cadence {
                every: 2,
                unit: Unit::Week
            })
        })
    );
    let two_days = one(
        "BEGIN:VEVENT\r\nUID:h\r\nSUMMARY:Run\r\nDTSTART;TZID=America/Santiago:20261019T070000\r\nRRULE:FREQ=WEEKLY;BYDAY=MO,TH\r\nEND:VEVENT\r\n",
    );
    assert_eq!(two_days.repeat, None);
    assert!(two_days.odd.contains(&Odd::Repeat));
    let skipping = one(
        "BEGIN:VEVENT\r\nUID:i\r\nSUMMARY:Class\r\nDTSTART;TZID=America/Santiago:20261019T070000\r\nRRULE:FREQ=DAILY\r\nEXDATE;TZID=America/Santiago:20261020T070000\r\nEND:VEVENT\r\n",
    );
    assert!(skipping.odd.contains(&Odd::Repeat));
}

#[test]
fn a_to_do_brings_its_deadline_priority_and_whether_it_is_done() {
    let todo = one(
        "BEGIN:VTODO\r\nUID:j\r\nSUMMARY:Pay rent\r\nDUE;VALUE=DATE:20261101\r\nPRIORITY:1\r\nEND:VTODO\r\n",
    );
    assert_eq!(todo.date, None);
    assert!(todo.deadline.is_some());
    assert_eq!(todo.priority, Some(Priority::Do));
    assert!(todo.takeable());
    let done = one("BEGIN:VTODO\r\nUID:k\r\nSUMMARY:Old\r\nSTATUS:COMPLETED\r\nEND:VTODO\r\n");
    assert!(done.done && !done.takeable());
}

#[test]
fn what_has_passed_is_said_and_takes_no_reminder() {
    let past = one(
        "BEGIN:VEVENT\r\nUID:l\r\nSUMMARY:Yesterday\r\nDTSTART;TZID=America/Santiago:20261017T100000\r\nEND:VEVENT\r\n",
    );
    assert!(past.odd.contains(&Odd::Past));
    assert_eq!(past.reminder, None);
}

#[test]
fn an_untitled_entry_or_a_moved_occurrence_is_shown_but_not_taken() {
    let text = "BEGIN:VCALENDAR\nMETHOD:REQUEST\nBEGIN:VEVENT\nUID:m\nDTSTART:20261022T150000Z\nEND:VEVENT\nBEGIN:VEVENT\nUID:m\nSUMMARY:Moved\nRECURRENCE-ID:20261029T150000Z\nDTSTART:20261030T150000Z\nEND:VEVENT\nEND:VCALENDAR\n";
    let calendar = read(text).unwrap();
    assert_eq!(calendar.method.as_deref(), Some("REQUEST"));
    assert_eq!(calendar.items[0].kind, Kind::Event);
    let untitled = taken(&calendar.items[0], &now());
    let moved = taken(&calendar.items[1], &now());
    assert!(!untitled.takeable() && untitled.odd.contains(&Odd::Untitled));
    assert!(!moved.takeable() && moved.odd.contains(&Odd::Moved));
}

#[test]
fn a_file_that_is_not_a_calendar_says_so_and_brings_nothing() {
    assert_eq!(read("# Notes\n"), Err(Unread::NotACalendar));
    assert_eq!(read(""), Err(Unread::NotACalendar));
    let huge = "x".repeat(super::BYTES_AT_MOST + 1);
    assert_eq!(read(&huge), Err(Unread::TooBig));
}

#[test]
fn a_rule_part_only_restates_the_start_at_the_rhythm_that_keeps_it() {
    let daily_mondays = one(
        "BEGIN:VEVENT\r\nUID:x1\r\nSUMMARY:Mondays\r\nDTSTART;TZID=America/Santiago:20261019T070000\r\nRRULE:FREQ=DAILY;BYDAY=MO\r\nEND:VEVENT\r\n",
    );
    assert!(daily_mondays.odd.contains(&Odd::Repeat));
    let october_only = one(
        "BEGIN:VEVENT\r\nUID:x2\r\nSUMMARY:October\r\nDTSTART;TZID=America/Santiago:20261019T070000\r\nRRULE:FREQ=WEEKLY;BYMONTH=10\r\nEND:VEVENT\r\n",
    );
    assert!(october_only.odd.contains(&Odd::Repeat));
}

#[test]
fn a_weekday_is_read_on_the_event_s_own_calendar() {
    let late = one(
        "BEGIN:VEVENT\r\nUID:x3\r\nSUMMARY:Late call\r\nDTSTART;TZID=Asia/Tokyo:20261020T060000\r\nRRULE:FREQ=WEEKLY;BYDAY=TU\r\nEND:VEVENT\r\n",
    );
    assert!(late.repeat.is_some(), "{late:?}");
    assert_eq!(late.date.unwrap().at.date(), date(2026, 10, 19));
}

#[test]
fn the_31st_stays_the_31st_and_a_series_ending_today_does_not_come_back() {
    let rent = one(
        "BEGIN:VEVENT\r\nUID:x4\r\nSUMMARY:Rent\r\nDTSTART;VALUE=DATE:20240131\r\nRRULE:FREQ=MONTHLY\r\nEND:VEVENT\r\n",
    );
    assert_eq!(rent.date.unwrap().at.date(), date(2026, 10, 31));
    let ending = one(
        "BEGIN:VEVENT\r\nUID:x5\r\nSUMMARY:Course\r\nDTSTART;TZID=America/Santiago:20261004T090000\r\nRRULE:FREQ=WEEKLY;UNTIL=20261018T235959Z\r\nEND:VEVENT\r\n",
    );
    assert_eq!(ending.repeat, None);
    assert!(ending.odd.contains(&Odd::Past));
}

#[test]
fn an_event_s_alarm_from_its_end_counts_from_when_it_ends() {
    let exam = one(
        "BEGIN:VEVENT\r\nUID:x6\r\nSUMMARY:Exam\r\nDTSTART;TZID=America/Santiago:20261022T090000\r\nDTEND;TZID=America/Santiago:20261022T120000\r\nBEGIN:VALARM\r\nTRIGGER;RELATED=END:-PT15M\r\nEND:VALARM\r\nEND:VEVENT\r\n",
    );
    assert_eq!(
        exam.reminder.unwrap().at,
        date(2026, 10, 22).at(11, 45, 0, 0)
    );
}

#[test]
fn an_outlook_zone_with_a_colon_still_gives_the_date_and_an_unreadable_one_is_said() {
    let outlook = one(
        "BEGIN:VEVENT\r\nUID:x7\r\nSUMMARY:Board\r\nDTSTART;TZID=(UTC-05:00) Eastern Time (US & Canada):20261020T090000\r\nEND:VEVENT\r\n",
    );
    assert_eq!(outlook.date.unwrap().at, date(2026, 10, 20).at(9, 0, 0, 0));
    assert!(outlook.odd.contains(&Odd::Zone));
    let broken =
        one("BEGIN:VEVENT\r\nUID:x8\r\nSUMMARY:Broken\r\nDTSTART:2026-10-20\r\nEND:VEVENT\r\n");
    assert!(broken.odd.contains(&Odd::Undated));
}

#[test]
fn an_event_without_an_alarm_is_reminded_before_it_starts_not_before_it_ends() {
    let meeting = one(
        "BEGIN:VEVENT\r\nUID:y1\r\nSUMMARY:Meeting\r\nDTSTART;TZID=America/Santiago:20261022T100000\r\nDTEND;TZID=America/Santiago:20261022T110000\r\nEND:VEVENT\r\n",
    );
    assert_eq!(
        meeting.reminder.unwrap().at,
        date(2026, 10, 22).at(9, 30, 0, 0)
    );
}

#[test]
fn an_end_given_as_a_duration_and_a_series_end_move_with_their_turn() {
    let exam = one(
        "BEGIN:VEVENT\r\nUID:y2\r\nSUMMARY:Exam\r\nDTSTART;TZID=America/Santiago:20261022T090000\r\nDURATION:PT3H\r\nBEGIN:VALARM\r\nTRIGGER;RELATED=END:-PT15M\r\nEND:VALARM\r\nEND:VEVENT\r\n",
    );
    assert_eq!(
        exam.reminder.unwrap().at,
        date(2026, 10, 22).at(11, 45, 0, 0)
    );
    let weekly = one(
        "BEGIN:VEVENT\r\nUID:y3\r\nSUMMARY:Class\r\nDTSTART;TZID=America/Santiago:20250106T090000\r\nDTEND;TZID=America/Santiago:20250106T100000\r\nRRULE:FREQ=WEEKLY\r\nBEGIN:VALARM\r\nTRIGGER;RELATED=END:-PT15M\r\nEND:VALARM\r\nEND:VEVENT\r\n",
    );
    assert_eq!(
        weekly.reminder.unwrap().at,
        date(2026, 10, 19).at(9, 45, 0, 0)
    );
}

#[test]
fn a_series_end_written_in_utc_is_read_on_this_clock() {
    let daily = one(
        "BEGIN:VEVENT\r\nUID:y4\r\nSUMMARY:Pills\r\nDTSTART;TZID=America/Santiago:20261019T200000\r\nRRULE:FREQ=DAILY;UNTIL=20261029T025959Z\r\nEND:VEVENT\r\n",
    );
    assert_eq!(daily.repeat.unwrap().until, Some(date(2026, 10, 28)));
}
