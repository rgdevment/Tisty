use super::*;
use tisty_core::event::{DeviceId, Event, ListAdd, StepAdd, TaskAdd};
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
fn a_cited_task_brings_its_list_its_steps_drawn_and_its_parts() {
    let mut state = State::default();
    let list = ulid::Ulid::generate();
    let d = ListAdd {
        name: "Casa".into(),
        order: "a0".into(),
        color: None,
    };
    applied(&mut state, 1, Op::ListAdd { id: list, d });
    let move_house = written(&mut state, "Mudanza", Some(list), None);
    let quotes = stepped(&mut state, move_house, "Pedir **tres** presupuestos", "a0");
    stepped(&mut state, move_house, "Llamar a Jaime", "a1");
    applied(
        &mut state,
        3,
        Op::StepDone {
            id: move_house,
            d: StepRef { step: quotes },
        },
    );
    let pack = written(&mut state, "Embalar", None, Some(move_house));
    written(&mut state, "Contratar el camión", None, Some(move_house));
    applied(
        &mut state,
        4,
        Op::TaskDone {
            id: pack,
            filled: false,
        },
    );

    let said = cited(&state, &[move_house.to_string()]);

    assert_eq!(said.len(), 1);
    let card = &said[0];
    assert_eq!(card.title, "Mudanza");
    assert_eq!(card.status, Status::Open);
    assert_eq!(card.list.as_deref(), Some("Casa"));
    assert_eq!(card.parts, Some(Counted { closed: 1, all: 2 }));
    assert_eq!(
        card.steps,
        vec![
            CitedStep {
                id: quotes.to_string(),
                html: "Pedir <strong>tres</strong> presupuestos".into(),
                done: true,
            },
            CitedStep {
                id: card.steps[1].id.clone(),
                html: "Llamar a Jaime".into(),
                done: false,
            },
        ]
    );
}

#[test]
fn a_closed_task_comes_with_its_closing_and_a_missing_one_is_left_out() {
    let mut state = State::default();
    let tyres = written(&mut state, "Cambiar los neumáticos", None, None);
    applied(
        &mut state,
        5,
        Op::TaskDone {
            id: tyres,
            filled: false,
        },
    );
    let erased = written(&mut state, "Borrada", None, None);
    applied(&mut state, 6, Op::TaskDelete { id: erased });
    let elsewhere = ulid::Ulid::generate().to_string();

    let said = cited(
        &state,
        &[
            elsewhere,
            erased.to_string(),
            "not an id".into(),
            tyres.to_string(),
        ],
    );

    assert_eq!(said.len(), 1);
    assert_eq!(said[0].status, Status::Done);
    assert!(said[0].completed_at.is_some());
    assert!(said[0].steps.is_empty());
    assert_eq!(said[0].parts, None);
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
