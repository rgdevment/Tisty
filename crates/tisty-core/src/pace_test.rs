use super::*;
use crate::model::{DateSpec, Status, TaskId};
use ulid::Ulid;

// One case a line: ☀ start, ⚑ deadline, today, open and closed parts, then pace, time % and done %.
const CASES: &str = "
2026-10-06  2026-12-04  2026-10-27  10  0   slow      36   0
2026-10-06  2026-12-04  2026-10-27  4   6   on_track  36   60
2026-10-06  2026-12-04  2026-10-06  10  0   on_track  0    0
2026-10-01  2026-10-11  2026-10-05  8   2   slow      40   20
2026-10-01  2026-10-11  2026-10-05  7   3   on_track  40   30
2026-10-01  2026-10-11  2026-10-06  10  0   slow      50   0
2026-10-01  2026-10-11  2026-10-07  8   2   behind    60   20
2026-10-01  2026-10-11  2026-10-07  7   3   slow      60   30
2026-10-01  2026-10-11  2026-10-20  6   4   behind    100  40
2026-10-01  2026-10-11  2026-10-20  5   5   slow      100  50
2026-10-01  2026-10-11  2026-10-20  1   9   on_track  100  90
2026-10-01  2026-10-01  2026-10-01  1   0   on_track  0    0
";

fn day(raw: &str) -> Option<DateSpec> {
    Some(DateSpec::all_day(raw.parse().unwrap(), "UTC"))
}

fn task(start: &str, due: &str) -> Task {
    let mut task = Task::new(Ulid::generate(), "A3 · Migración de Sodimac Uruguay", "a0");
    task.date = day(start);
    task.deadline = day(due);
    task
}

#[test]
fn time_gone_against_parts_closed_reads_as_decided() {
    for case in CASES.lines().filter(|line| !line.trim().is_empty()) {
        let [start, due, today, open, closed, how, time, done] =
            case.split_whitespace().collect::<Vec<_>>()[..]
        else {
            panic!("a case has eight columns: {case}");
        };

        let got = pace(
            &task(start, due),
            open.parse().unwrap(),
            closed.parse().unwrap(),
            today.parse().unwrap(),
        )
        .unwrap();

        assert_eq!(serde_json::to_value(got.pace).unwrap(), how, "{case}");
        assert_eq!(
            (got.time.to_string(), got.done.to_string()),
            (time.into(), done.into()),
            "{case}"
        );
    }
}

#[test]
fn there_is_no_pace_without_both_dates_parts_and_a_start_that_came() {
    let today = "2026-10-27".parse().unwrap();
    let mut alone = task("2026-10-06", "2026-12-04");
    alone.deadline = None;
    let mut closed = task("2026-10-06", "2026-12-04");
    closed.status = Status::Done;

    assert_eq!(pace(&alone, 10, 0, today), None);
    assert_eq!(pace(&closed, 10, 0, today), None);
    assert_eq!(pace(&task("2026-10-06", "2026-12-04"), 0, 0, today), None);
    assert_eq!(pace(&task("2026-11-02", "2026-12-04"), 10, 0, today), None);
}

#[test]
fn the_parts_put_out_of_sight_are_not_counted() {
    let a3 = task("2026-10-06", "2026-12-04");
    let mut state = State::default();
    let part = |closed: bool, away: bool| {
        let mut one = Task::new(TaskId::generate(), "parte", "a0");
        one.part_of = Some(a3.id);
        one.hidden = away;
        if closed {
            one.status = Status::Done;
        }
        one
    };
    for one in [part(true, false), part(false, false), part(false, true)] {
        state.tasks.insert(one.id, one);
    }

    let got = paced(&state, &a3, "2026-10-27".parse().unwrap()).unwrap();

    assert_eq!(got.done, 50);
}
