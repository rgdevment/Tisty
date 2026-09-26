use super::*;
use tisty_core::Tag;
use ulid::Ulid;

fn task(title: &str) -> Task {
    Task::new(Ulid::generate(), title, "a0")
}

fn day(s: &str) -> Date {
    s.parse().unwrap()
}

#[test]
fn relative_days_read_naturally() {
    let today = day("2026-08-05");
    assert_eq!(
        short(day("2026-08-05"), today, Lang::from_code("en")),
        "today"
    );
    assert_eq!(
        short(day("2026-08-06"), today, Lang::from_code("es")),
        "mañana"
    );
    assert_eq!(
        short(day("2026-08-04"), today, Lang::from_code("en")),
        "yesterday"
    );
}

#[test]
fn a_far_off_date_shows_the_day_and_month() {
    let out = short(day("2026-12-24"), day("2026-08-05"), Lang::from_code("en"));
    assert!(out.contains("24"), "{out}");
}

#[test]
fn an_empty_list_says_so_instead_of_printing_nothing() {
    let out = list(
        &[],
        &State::default(),
        "today",
        day("2026-08-05"),
        Lang::from_code("en"),
    );
    assert!(out.contains(Lang::from_code("en").get("nothing-here")));
}

#[test]
fn a_bare_task_renders_as_a_single_line() {
    let t = task("book a haircut");
    let out = list(
        &[&t],
        &State::default(),
        "today",
        day("2026-08-05"),
        Lang::from_code("en"),
    );
    let body: Vec<_> = out.lines().filter(|l| l.contains("haircut")).collect();

    assert_eq!(body.len(), 1);
    assert_eq!(out.lines().filter(|l| l.trim().starts_with('·')).count(), 0);
}

#[test]
fn a_documented_task_shows_its_sections() {
    let mut t = task("fix the failing checkout");
    t.description = Some("reproduces only with an empty cart".into());
    t.tags = vec![Tag::new("work").unwrap()];

    let out = detail(
        &t,
        &State::default(),
        day("2026-08-05"),
        Lang::from_code("en"),
    );

    assert!(out.contains("description"));
    assert!(out.contains("reproduces only with an empty cart"));
    assert!(out.contains("#work"));
    assert!(!out.contains("steps"), "no steps means no section");
    assert!(!out.contains("journal"));
}

#[test]
fn tasks_created_together_still_get_distinct_short_ids() {
    let mut a = task("first");
    let mut b = task("second");
    a.id = "01J8F2K3XQ0000000000000ABC".parse().unwrap();
    b.id = "01J8F2K3XQ0000000000000XYZ".parse().unwrap();
    assert_ne!(short_id(&a), short_id(&b));
}

#[test]
fn nothing_is_printed_for_what_nobody_placed() {
    assert_eq!(priority(Priority::Unset, Lang::from_code("en")), None);
    assert!(priority(Priority::Do, Lang::from_code("en")).is_some());
}
