use super::*;
use crate::model::{DateSpec, Task};
use ulid::Ulid;

// One case a line: today, ☀ date, ⚑ deadline (or -), and where the task stands.
const CASES: &str = "
2026-10-27  -           -           undated
2026-10-27  2026-11-02  -           ahead
2026-10-27  2026-10-28  -           ahead
2026-10-27  2026-10-27  -           today
2026-10-27  2026-10-06  -           late
2026-10-27  2026-11-02  2026-12-04  ahead
2026-10-27  2026-10-27  2026-12-04  underway
2026-10-27  2026-10-06  2026-12-04  underway
2026-10-27  2026-10-06  2026-10-31  underway
2026-10-27  2026-10-06  2026-10-30  due_soon
2026-10-27  2026-10-29  2026-10-29  due_soon
2026-10-27  2026-10-06  2026-10-27  due_soon
2026-10-27  2026-10-06  2026-10-26  overdue
2026-10-27  2026-11-09  2026-10-26  overdue
2026-10-27  -           2026-12-04  ahead
2026-10-27  -           2026-10-31  ahead
2026-10-27  -           2026-10-30  due_soon
2026-10-27  -           2026-10-27  due_soon
2026-10-27  -           2026-10-20  overdue
2026-12-30  -           2027-01-02  due_soon
2026-12-30  2026-12-01  2027-01-03  underway
";

fn day(raw: &str) -> Option<DateSpec> {
    (raw != "-").then(|| DateSpec::all_day(raw.parse().unwrap(), "UTC"))
}

#[test]
fn each_pair_of_dates_stands_where_it_was_decided() {
    for case in CASES.lines().filter(|line| !line.trim().is_empty()) {
        let [today, on, due, stands] = case.split_whitespace().collect::<Vec<_>>()[..] else {
            panic!("a case has four columns: {case}");
        };
        let mut task = Task::new(Ulid::generate(), case, "a0");
        task.date = day(on);
        task.deadline = day(due);

        let got = standing(&task, today.parse().unwrap());

        assert_eq!(serde_json::to_value(got).unwrap(), stands, "{case}");
    }
}
