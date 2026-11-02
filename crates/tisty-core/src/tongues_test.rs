use crate::event::{DeviceId, DocAdd, Event, Said, Tongue};
use crate::model::DocId;
use crate::{Op, State};
use ulid::Ulid;

fn at(ms: i64, op: Op) -> Event {
    let mut event = Event::new(
        DeviceId("dev_a".into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    );
    event.optional = event.op.is_optional();
    event
}

fn written(ms: i64, id: DocId, body: &str) -> Event {
    at(
        ms,
        Op::DocAdd {
            id,
            d: DocAdd {
                file: format!("dev_a-{ms:04}"),
                order: "a0".into(),
                said: Some(Said::of(body)),
                ..Default::default()
            },
        },
    )
}

fn paged(ms: i64, id: DocId, up: DocId, order: &str) -> Event {
    at(
        ms,
        Op::DocAdd {
            id,
            d: DocAdd {
                file: format!("dev_a-{ms:04}"),
                order: order.into(),
                page_of: Some(up),
                ..Default::default()
            },
        },
    )
}

fn joined(ms: i64, id: DocId, of: Option<DocId>) -> Event {
    at(
        ms,
        Op::DocTongue {
            id,
            d: Tongue { of },
        },
    )
}

fn ids(of: &[&crate::model::Kept]) -> Vec<DocId> {
    of.iter().map(|one| one.id).collect()
}

const ES: &str = "---\nlang: es\n---\n# Paseos\n";
const EN: &str = "---\nlang: en\n---\n# Walks\n";

#[test]
fn a_translation_and_its_anchor_are_one_document_in_two_languages() {
    let (es, en) = (Ulid::generate(), Ulid::generate());
    let state = State::replay(&[
        written(1, es, ES),
        written(2, en, EN),
        joined(3, en, Some(es)),
    ]);

    assert_eq!(ids(&state.tongues_of(es)), vec![es, en]);
    assert_eq!(ids(&state.tongues_of(en)), vec![es, en]);
    assert_eq!(state.docs[&en].lang.as_deref(), Some("en"));
    assert_eq!(state.families().len(), 1);
}

#[test]
fn a_document_alone_has_no_languages_to_choose_from() {
    let es = Ulid::generate();
    let state = State::replay(&[written(1, es, ES)]);

    assert!(state.tongues_of(es).is_empty());
    assert!(state.families().is_empty());
    assert!(!state.apart_in_tongues(es));
}

#[test]
fn joining_a_member_joins_its_anchor_so_a_group_stays_one_level_deep() {
    let (es, en, pt) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
    let state = State::replay(&[
        written(1, es, ES),
        written(2, en, EN),
        written(3, pt, "---\nlang: pt-BR\n---\n# Passeios\n"),
        joined(4, en, Some(es)),
        joined(5, pt, Some(en)),
    ]);

    assert_eq!(state.docs[&pt].translation_of, Some(es));
    assert_eq!(ids(&state.tongues_of(pt)), vec![es, en, pt]);
}

#[test]
fn an_anchor_that_joins_another_brings_its_languages_along() {
    let (es, en, pt) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
    let state = State::replay(&[
        written(1, es, ES),
        written(2, en, EN),
        written(3, pt, "---\nlang: pt\n---\n# Passeios\n"),
        joined(4, en, Some(es)),
        joined(5, es, Some(pt)),
    ]);

    assert_eq!(state.docs[&en].translation_of, Some(pt));
    assert_eq!(state.docs[&es].translation_of, Some(pt));
    assert_eq!(state.tongues_of(en).len(), 3);
}

#[test]
fn a_language_said_twice_stays_with_the_one_that_joined_first() {
    let (es, en, again) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
    // A copy brought back from a backup says it was made long ago; it joined last all the same.
    let mut old = written(0, again, EN);
    if let Op::DocAdd { d, .. } = &mut old.op {
        d.made = Some(jiff::Timestamp::from_millisecond(-1000).unwrap());
    }
    let state = State::replay(&[
        written(1, es, ES),
        written(2, en, EN),
        joined(3, en, Some(es)),
        old,
        joined(4, again, Some(es)),
    ]);

    assert_eq!(ids(&state.tongues_of(es)), vec![es, en]);
    assert!(state.apart_in_tongues(again));
    assert!(!state.apart_in_tongues(en));
}

#[test]
fn a_member_that_says_no_language_is_shown_on_its_own() {
    let (es, en, bare) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
    let state = State::replay(&[
        written(1, es, ES),
        written(2, en, EN),
        written(3, bare, "# Sin idioma\n"),
        joined(4, en, Some(es)),
        joined(5, bare, Some(es)),
    ]);

    assert!(state.apart_in_tongues(bare));
    assert_eq!(state.tongues_of(es).len(), 2);
}

#[test]
fn taking_a_language_out_leaves_the_others_together() {
    let (es, en, pt) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
    let state = State::replay(&[
        written(1, es, ES),
        written(2, en, EN),
        written(3, pt, "---\nlang: pt\n---\n# Passeios\n"),
        joined(4, en, Some(es)),
        joined(5, pt, Some(es)),
        joined(6, en, None),
    ]);

    assert_eq!(ids(&state.tongues_of(es)), vec![es, pt]);
    assert!(state.tongues_of(en).is_empty());
}

#[test]
fn a_page_never_joins_and_is_never_joined() {
    let (es, en, page) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
    let state = State::replay(&[
        written(1, es, ES),
        written(2, en, EN),
        paged(3, page, es, "a0"),
        joined(4, page, Some(en)),
        joined(5, en, Some(page)),
    ]);

    assert_eq!(state.docs[&page].translation_of, None);
    assert_eq!(state.docs[&en].translation_of, None);
}

#[test]
fn deleting_the_anchor_keeps_the_others_together() {
    let (es, en, pt) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
    let state = State::replay(&[
        written(1, es, ES),
        written(2, en, EN),
        written(3, pt, "---\nlang: pt\n---\n# Passeios\n"),
        joined(4, en, Some(es)),
        joined(5, pt, Some(es)),
        at(6, Op::DocDelete { id: es }),
    ]);

    assert_eq!(ids(&state.tongues_of(en)), vec![en, pt]);
}

#[test]
fn a_page_finds_its_twin_where_it_stands_in_the_other_language() {
    let (es, en) = (Ulid::generate(), Ulid::generate());
    let (es1, es2, en1, en2) = (
        Ulid::generate(),
        Ulid::generate(),
        Ulid::generate(),
        Ulid::generate(),
    );
    let state = State::replay(&[
        written(1, es, ES),
        written(2, en, EN),
        paged(3, es1, es, "a0"),
        paged(4, es2, es, "a1"),
        paged(5, en2, en, "a1"),
        paged(6, en1, en, "a0"),
        joined(7, en, Some(es)),
    ]);

    assert_eq!(state.twin_page(es2, en).map(|one| one.id), Some(en2));
    assert_eq!(state.twin_page(en1, es).map(|one| one.id), Some(es1));
}

#[test]
fn a_note_from_a_build_that_never_read_languages_keeps_the_one_it_had() {
    let es = Ulid::generate();
    let mut said = Said::of("# Paseos, ahora sin encabezado\n");
    said.lang = None;
    let state = State::replay(&[written(1, es, ES), at(2, Op::DocSaid { id: es, d: said })]);
    assert_eq!(state.docs[&es].lang.as_deref(), Some("es"));

    let gone = State::replay(&[
        written(1, es, ES),
        at(
            2,
            Op::DocSaid {
                id: es,
                d: Said::of("# Paseos\n"),
            },
        ),
    ]);
    assert_eq!(
        gone.docs[&es].lang, None,
        "a body that drops its language says so"
    );
}

#[test]
fn undoing_a_join_takes_the_document_back_out() {
    let (es, en) = (Ulid::generate(), Ulid::generate());
    let before = State::replay(&[written(1, es, ES), written(2, en, EN)]);
    let undo = crate::undo::inverse(
        &at(
            3,
            Op::DocTongue {
                id: en,
                d: Tongue { of: Some(es) },
            },
        ),
        &before,
    );
    assert_eq!(
        undo,
        Some(vec![Op::DocTongue {
            id: en,
            d: Tongue { of: None },
        }])
    );
}

#[test]
fn the_languages_in_its_folder_go_with_it_and_one_elsewhere_stays_put() {
    let (es, en, pt) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
    let elsewhere = Ulid::generate();
    let mut moved = written(3, pt, "---\nlang: pt\n---\n# Passeios\n");
    if let Op::DocAdd { d, .. } = &mut moved.op {
        d.folder = Some(elsewhere);
    }
    let state = State::replay(&[
        written(1, es, ES),
        written(2, en, EN),
        moved,
        joined(4, en, Some(es)),
        joined(5, pt, Some(es)),
    ]);

    assert_eq!(state.kin(es), vec![en]);
    assert_eq!(state.kin(pt), Vec::<DocId>::new());
    assert_eq!(
        state.tongues_of(pt).len(),
        3,
        "the selector still reaches it"
    );
}
