use super::*;
use tisty_core::event::{DeviceId, DocAdd, Event, ListAdd, TaskAdd};
use tisty_core::model::{Priority, Status};

fn ev(ms: i64, op: Op) -> Event {
    Event::new(
        DeviceId("dev_a".into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    )
}

fn kept(state: &mut State, file: &str) {
    let d = DocAdd {
        wrote: None,
        guest: false,
        sown: false,
        made: None,
        by: None,
        said: None,
        file: file.into(),
        order: "a0".into(),
        folder: None,
        page_of: None,
        task_of: None,
        edited: None,
    };
    state.apply(&ev(
        1,
        Op::DocAdd {
            id: ulid::Ulid::generate(),
            d,
        },
    ));
}

fn listed(state: &mut State, name: &str) -> ulid::Ulid {
    let id = ulid::Ulid::generate();
    let d = ListAdd {
        name: name.into(),
        order: "a0".into(),
        color: None,
    };
    state.apply(&ev(1, Op::ListAdd { id, d }));
    id
}

fn whole_in(state: &mut State, list: Option<ulid::Ulid>) -> TaskId {
    let id = ulid::Ulid::generate();
    let mut d = TaskAdd::new("Mudanza", "a0");
    d.list = list;
    state.apply(&ev(2, Op::TaskAdd { id, d }));
    id
}

fn drafted(text: &str) -> Draft {
    tisty_nl::parse(text, &jiff::Zoned::now(), "es").into()
}

fn in_doc(doc: &str, before: &str) -> Born {
    Born {
        doc: Some(doc.into()),
        before: Some(before.into()),
        whole: None,
    }
}

fn on_page_of(whole: TaskId) -> Born {
    Born {
        whole: Some(whole.to_string()),
        ..Born::default()
    }
}

fn filed(state: &mut State, draft: Draft, whole: Option<TaskId>) -> tisty_core::Task {
    let plan = tisty_core::capture::plan(state, draft).unwrap();
    for op in hung(plan.ops, plan.task, whole) {
        state.apply(&ev(3, op));
    }
    state.tasks[&plan.task].clone()
}

#[test]
fn a_task_written_in_a_document_is_born_under_the_heading_above_it_as_its_link_names_it() {
    let mut state = State::default();
    kept(&mut state, "a-0001");
    let before = "# Viaje\n\n## Año nuevo\n\nUvas.\n\n## Año nuevo\n\nCena con Marisol ";
    let whole = format!("{before}\n\n## Después\n\nNada.\n");
    let mut draft = drafted("Comprar las uvas el lunes");

    let hung_from = in_doc("a-0001", before).rooted(&state, &mut draft).unwrap();

    let heads = tisty_markdown::outline(&tisty_markdown::read(&whole));
    assert_eq!(hung_from, None);
    assert_eq!(draft.source, Some(format!("a-0001#{}", heads[2].id)));
    assert_ne!(
        heads[1].id, heads[2].id,
        "the second heading has its own anchor"
    );
}

#[test]
fn a_task_written_under_no_heading_still_names_its_document() {
    let mut state = State::default();
    kept(&mut state, "a-0001");
    let mut draft = drafted("Llamar a Jaime");

    in_doc("a-0001", "Notas sueltas ")
        .rooted(&state, &mut draft)
        .unwrap();

    assert_eq!(draft.source.as_deref(), Some("a-0001#top"));
}

#[test]
fn a_document_that_is_not_here_names_no_origin() {
    let state = State::default();
    let mut draft = drafted("Llamar a Jaime");

    in_doc("a-0001", "# Notas\n")
        .rooted(&state, &mut draft)
        .unwrap();

    assert_eq!(draft.source, None);
}

#[test]
fn a_task_written_on_a_task_s_page_is_its_part_read_as_the_quick_capture_reads_it() {
    let mut state = State::default();
    let home = listed(&mut state, "Casa");
    let whole = whole_in(&mut state, Some(home));
    let mut draft = drafted("Llamar al vet el lunes #Rina !hacer");

    let hung_from = on_page_of(whole).rooted(&state, &mut draft).unwrap();
    let part = filed(&mut state, draft, hung_from);

    assert_eq!(part.part_of, Some(whole));
    assert_eq!(part.list, Some(home), "a part waits in its whole's list");
    assert_eq!(part.title, "Llamar al vet");
    assert!(part.date.is_some());
    assert_eq!(part.priority, Priority::Do);
    assert_eq!(part.tags.len(), 1);
    assert_eq!(
        part.source, None,
        "its whole already says where it came from"
    );
}

#[test]
fn a_part_that_names_its_own_list_keeps_it() {
    let mut state = State::default();
    let home = listed(&mut state, "Casa");
    let car = listed(&mut state, "Auto");
    let whole = whole_in(&mut state, Some(home));
    let mut draft = drafted("Cambiar los neumáticos @Auto");

    let hung_from = on_page_of(whole).rooted(&state, &mut draft).unwrap();
    let part = filed(&mut state, draft, hung_from);

    assert_eq!(part.list, Some(car));
    assert_eq!(part.part_of, Some(whole));
}

#[test]
fn a_part_is_refused_where_the_whole_could_not_hold_it() {
    let mut state = State::default();
    let whole = whole_in(&mut state, None);
    let mut repeating = drafted("Regar las plantas cada lunes");
    assert!(repeating.repeat.is_some());

    let refused = on_page_of(whole)
        .rooted(&state, &mut repeating)
        .unwrap_err();
    assert_eq!(refused.code, "partRepeats");

    state.apply(&ev(
        4,
        Op::TaskDone {
            id: whole,
            filled: false,
        },
    ));
    assert_eq!(state.tasks[&whole].status, Status::Done);
    let refused = on_page_of(whole)
        .rooted(&state, &mut drafted("Embalar los libros"))
        .unwrap_err();
    assert_eq!(refused.code, "wholeClosed");
}
