use super::*;
use tisty_core::model::{DateSpec, Status};
use ulid::Ulid;

// Written as text, a case a line, so adding one is a line and never a copy of a test.
const TASKS: &str = "
2026-10-06  2026-12-04  open    underway
2026-10-06  -           open    late
-           2026-10-26  open    overdue
-           2026-10-29  open    due_soon
2026-10-27  -           open    today
2026-11-02  -           open    ahead
-           -           open    -
2026-10-06  2026-10-26  done    -
";

fn today() -> jiff::civil::Date {
    "2026-10-27".parse().unwrap()
}

fn day(raw: &str) -> Option<DateSpec> {
    (raw != "-").then(|| DateSpec::all_day(raw.parse().unwrap(), "UTC"))
}

fn filed() -> Vec<(Task, &'static str)> {
    TASKS
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let [on, due, open, reads] = line.split_whitespace().collect::<Vec<_>>()[..] else {
                panic!("a task has four columns: {line}");
            };
            let mut task = Task::new(Ulid::generate(), line, "a0");
            task.date = day(on);
            task.deadline = day(due);
            if open == "done" {
                task.status = Status::Done;
            }
            (task, reads)
        })
        .collect()
}

#[test]
fn an_assistant_reads_each_task_as_the_window_groups_it() {
    for (task, reads) in filed() {
        let said = standing_of(&task, today()).map(|how| json!(how));

        assert_eq!(said, (reads != "-").then(|| json!(reads)), "{}", task.title);
    }
}

#[test]
fn what_find_read_and_catch_up_hand_over_carries_the_standing() {
    let state = State::default();
    for (task, reads) in filed() {
        let told = brief(&task, &state, today());

        assert_eq!(
            told.get("standing").cloned(),
            (reads != "-").then(|| json!(reads)),
            "{}",
            task.title
        );
    }
}
