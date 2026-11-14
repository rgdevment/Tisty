use jiff::Zoned;
use jiff::civil::date;

use super::{Entry, When, entry, google, outlook, outlook_work};
use crate::ics::{read, taken, written};
use crate::model::{DateSpec, Task};

const TASK: &str = "01KA7B3VJ9RZ0Q8M5N2X4Y6W1T";
const LINK: &str = "tisty://task?id=01KA7B3VJ9RZ0Q8M5N2X4Y6W1T";

fn now() -> Zoned {
    date(2026, 11, 14)
        .at(12, 0, 0, 0)
        .in_tz("America/Santiago")
        .unwrap()
}

fn task(title: &str) -> Task {
    Task::new(TASK.parse().unwrap(), title, "a0")
}

fn at(day: i8, hour: i8, minute: i8) -> DateSpec {
    DateSpec::floating(
        date(2026, 11, day).at(hour, minute, 0, 0),
        "America/Santiago",
    )
}

fn saved(task: &Task) -> Entry {
    entry(task, LINK, &now()).unwrap()
}

#[test]
fn a_task_at_an_hour_lasts_half_an_hour_and_is_reminded_half_an_hour_before() {
    let mut walk = task("Walk Rina");
    walk.date = Some(at(20, 10, 0));

    let said = saved(&walk);

    assert_eq!(
        said.when,
        When::Hours {
            from: "2026-11-20T13:00:00Z".parse().unwrap(),
            to: "2026-11-20T13:30:00Z".parse().unwrap(),
        }
    );
    assert_eq!(said.minutes_before, vec![30]);
    assert_eq!(said.link, LINK);
    assert_eq!(said.id, TASK);
}

#[test]
fn a_whole_day_ends_the_next_day_and_is_reminded_the_evening_before() {
    let mut vet = task("Vet for Roger");
    vet.date = Some(DateSpec::all_day(date(2026, 11, 20), "America/Santiago"));

    let said = saved(&vet);

    assert_eq!(
        said.when,
        When::Days {
            from: date(2026, 11, 20),
            to: date(2026, 11, 21),
        }
    );
    assert_eq!(said.minutes_before, vec![6 * 60]);
}

#[test]
fn a_fixed_zone_keeps_its_own_hour() {
    let mut call = task("Call Jaime");
    call.date = Some(DateSpec::fixed(
        date(2026, 11, 20).at(9, 0, 0, 0),
        "Europe/Madrid",
    ));

    let said = saved(&call);

    assert!(matches!(
        said.when,
        When::Hours { from, .. } if from == "2026-11-20T08:00:00Z".parse().unwrap()
    ));
}

#[test]
fn the_task_s_own_reminders_win_and_those_already_past_are_left_out() {
    let mut paper = task("Hand in the form");
    paper.date = Some(at(20, 10, 0));
    paper.reminders = vec![at(20, 9, 0), at(13, 9, 0), at(19, 10, 0), at(20, 9, 0)];

    assert_eq!(saved(&paper).minutes_before, vec![60, 24 * 60]);
}

#[test]
fn without_a_day_the_deadline_is_the_day_and_without_either_there_is_nothing_to_save() {
    let mut due = task("Pay the rent");
    due.deadline = Some(DateSpec::all_day(date(2026, 11, 30), "America/Santiago"));
    assert!(matches!(
        saved(&due).when,
        When::Days { from, .. } if from == date(2026, 11, 30)
    ));

    let mut both = due.clone();
    both.date = Some(DateSpec::all_day(date(2026, 11, 25), "America/Santiago"));
    assert!(matches!(
        saved(&both).when,
        When::Days { from, .. } if from == date(2026, 11, 25)
    ));

    assert_eq!(entry(&task("Someday"), LINK, &now()), None);
}

fn odd_title() -> Task {
    let mut odd = task("  Vet & «booster» #2 + 50% off, Rina; ok? 🐶\\  ");
    odd.date = Some(at(20, 10, 0));
    odd
}

fn whole_day() -> Task {
    let mut day = task("Marisol's birthday");
    day.date = Some(DateSpec::all_day(date(2026, 11, 20), "America/Santiago"));
    day
}

#[test]
fn google_s_page_comes_filled_in() {
    insta::assert_snapshot!("google_at_an_hour", google(&saved(&odd_title())));
    insta::assert_snapshot!("google_whole_day", google(&saved(&whole_day())));
}

#[test]
fn outlook_s_page_comes_filled_in() {
    insta::assert_snapshot!("outlook_at_an_hour", outlook(&saved(&odd_title())));
    insta::assert_snapshot!("outlook_whole_day", outlook(&saved(&whole_day())));
    insta::assert_snapshot!(
        "outlook_work_at_an_hour",
        outlook_work(&saved(&odd_title()))
    );
}

#[test]
fn no_address_carries_a_space_a_plus_or_anything_a_browser_could_cut_it_at() {
    let odd = saved(&odd_title());
    for address in [google(&odd), outlook(&odd), outlook_work(&odd)] {
        let query = address.split_once('?').unwrap().1;
        for cut in [' ', '+', '#', '\n', '"', '<', '>'] {
            assert!(!query.contains(cut), "{cut:?} in {address}");
        }
    }
}

#[test]
fn the_calendar_file_comes_filled_in() {
    let stamp = "2026-11-14T15:00:00Z".parse().unwrap();
    let mut long = odd_title();
    long.title = format!(
        "{} {}",
        long.title,
        "Lucas brings the leash and the ball ".repeat(3)
    );
    insta::assert_snapshot!("ics_at_an_hour", written(&saved(&long), stamp));
    insta::assert_snapshot!("ics_whole_day", written(&saved(&whole_day()), stamp));
}

#[test]
fn every_line_of_the_file_fits_and_ends_the_way_calendars_expect() {
    let mut long = odd_title();
    long.title = "Ñandú ".repeat(40);
    let text = written(&saved(&long), "2026-11-14T15:00:00Z".parse().unwrap());

    assert!(text.ends_with("END:VCALENDAR\r\n"));
    for line in text.split_terminator("\r\n") {
        assert!(line.len() <= 75, "{line:?} is {} octets", line.len());
        assert!(!line.contains('\n'));
    }
}

#[test]
fn tisty_reads_back_what_it_wrote() {
    let mut walk = task("Walk Rina, then Roger; at last");
    walk.date = Some(at(20, 10, 0));
    walk.reminders = vec![at(20, 9, 15)];
    let text = written(&saved(&walk), "2026-11-14T15:00:00Z".parse().unwrap());

    let calendar = read(&text).unwrap();
    assert_eq!(calendar.items.len(), 1);
    let back = taken(&calendar.items[0], &now());

    assert_eq!(back.title, "Walk Rina, then Roger; at last");
    assert_eq!(
        back.source.as_deref(),
        Some(&*format!("ics:{TASK}@tisty.app"))
    );
    assert_eq!(back.date.unwrap().instant(now().time_zone()).unwrap(), {
        "2026-11-20T13:00:00Z".parse::<jiff::Timestamp>().unwrap()
    });
    assert_eq!(
        back.reminder.unwrap().at,
        date(2026, 11, 20).at(9, 15, 0, 0)
    );
    assert_eq!(back.described.as_deref(), Some(LINK));
}

#[test]
fn a_whole_day_reads_back_as_that_day_reminded_the_evening_before() {
    let text = written(
        &saved(&whole_day()),
        "2026-11-14T15:00:00Z".parse().unwrap(),
    );

    let calendar = read(&text).unwrap();
    let back = taken(&calendar.items[0], &now());

    let day = back.date.unwrap();
    assert!(!day.has_time);
    assert_eq!(day.date(), date(2026, 11, 20));
    assert_eq!(
        back.reminder.unwrap().at,
        date(2026, 11, 19).at(18, 0, 0, 0)
    );
}
