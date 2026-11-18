use super::*;
use tisty_core::event::{DeviceId, Event, StepAdd, TaskAdd};
use tisty_core::model::StepId;

fn ev(ms: i64, op: Op) -> Event {
    Event::new(
        DeviceId("dev_a".into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    )
}

fn applied(state: &mut State, ms: i64, op: Op) {
    state.apply(&ev(ms, op));
}

fn written(
    state: &mut State,
    title: &str,
    list: Option<ulid::Ulid>,
    part_of: Option<TaskId>,
) -> TaskId {
    let id = ulid::Ulid::generate();
    let mut d = TaskAdd::new(title, "a0");
    d.list = list;
    d.part_of = part_of;
    applied(state, 1, Op::TaskAdd { id, d });
    id
}

fn stepped(state: &mut State, id: TaskId, text: &str, order: &str) -> StepId {
    let step = ulid::Ulid::generate();
    let d = StepAdd {
        step,
        text: text.into(),
        order: order.into(),
    };
    applied(state, 2, Op::StepAdd { id, d });
    step
}

#[test]
fn a_step_is_ticked_and_unticked_from_a_card_only_while_its_task_is_open() {
    let mut state = State::default();
    let vet = written(&mut state, "Llevar a Rina al veterinario", None, None);
    let call = stepped(&mut state, vet, "Pedir hora", "a0");
    let said = call.to_string();

    let ticked = marking(&state, vet, &said, true).unwrap();
    assert!(matches!(ticked, Some(Op::StepDone { .. })));
    applied(&mut state, 3, ticked.unwrap());
    assert_eq!(marking(&state, vet, &said, true).unwrap(), None);
    assert!(matches!(
        marking(&state, vet, &said, false).unwrap(),
        Some(Op::StepUndone { .. })
    ));
    assert_eq!(marking(&state, vet, "gone", true).unwrap(), None);

    applied(
        &mut state,
        4,
        Op::TaskDone {
            id: vet,
            filled: false,
        },
    );
    let refused = marking(&state, vet, &said, false).unwrap_err();
    assert_eq!(refused.code, "taskClosed");
    let nowhere = marking(&state, ulid::Ulid::generate(), &said, true).unwrap_err();
    assert_eq!(nowhere.code, "notATaskId");
}

#[test]
fn a_page_left_open_with_its_old_box_does_not_untick_what_a_card_ticked() {
    let mut state = State::default();
    let vet = written(&mut state, "Llevar a Rina al veterinario", None, None);
    let call = stepped(&mut state, vet, "Pedir hora", "a0");
    let page = "## Pasos\n\n- [ ] Pedir hora\n";
    let ticked = marking(&state, vet, &call.to_string(), true)
        .unwrap()
        .unwrap();
    applied(&mut state, 3, ticked);
    let task = &state.tasks[&vet];

    let settled = tisty_core::stages::reconciled_with(task, Some(page), page, &[]);

    assert!(
        !settled.iter().any(|op| matches!(op, Op::StepUndone { .. })),
        "{settled:?}"
    );
}
