use super::*;

fn told(body: &str, log: &[&str]) -> Vec<crate::refs::Ref> {
    let mut task = Task::new(ulid::Ulid::generate(), "x", "a0");
    task.description = Some(body.to_string());
    task.log = log
        .iter()
        .map(|one| LogEntry {
            id: ulid::Ulid::generate(),
            at: Timestamp::from_second(0).unwrap(),
            tz: None,
            body: (*one).to_string(),
            by: None,
            via: None,
        })
        .collect();
    task.references()
}

#[test]
fn one_target_written_twice_with_two_labels_is_one_trace() {
    let all = told(
        "[the report](https://x.example/1)",
        &["[final report](https://x.example/1)"],
    );

    assert_eq!(all.len(), 1, "the same link came back twice: {all:?}");
}

#[test]
fn a_step_anchor_is_not_something_the_task_left_behind() {
    let all = told("see [[#3]] and [[CUSLEG-1]]", &[]);

    assert_eq!(all.len(), 1);
    assert_eq!(all[0].target, "CUSLEG-1");
}
