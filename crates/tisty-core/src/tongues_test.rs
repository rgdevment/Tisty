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

#[test]
fn a_language_leaves_with_the_key_of_its_group_and_one_alone_with_none() {
    let (es, en, alone) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
    let state = State::replay(&[
        written(1, es, ES),
        written(2, en, EN),
        written(3, alone, "# Solo\n"),
        joined(4, en, Some(es)),
    ]);

    assert_eq!(
        state.keyed("dev_a-0002", EN),
        format!("---\nlang: en\ntranslationKey: {es}\n---\n# Walks\n")
    );
    assert_eq!(state.keyed("dev_a-0003", "# Solo\n"), "# Solo\n");
}

#[test]
fn a_language_brought_back_joins_the_group_its_key_names_unless_that_language_is_kept() {
    let (es, en, back) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
    let state = State::replay(&[
        written(1, es, ES),
        written(2, en, EN),
        joined(3, en, Some(es)),
    ]);
    let pt = format!("---\nlang: pt\ntranslationKey: {es}\n---\n# Passeios\n");
    let again = format!("---\nlang: en\ntranslationKey: {es}\n---\n# Walks\n");

    assert_eq!(state.keyed_to(&pt, back), Some(es));
    assert_eq!(state.keyed_to(&again, back), None);
    assert_eq!(state.keyed_to("# Nada\n", back), None);
}

#[test]
fn languages_brought_in_elsewhere_find_each_other_by_their_key() {
    let first = Ulid::generate();
    let es = "---\nlang: es\ntranslationKey: from-another-tisty\n---\n# Paseos\n";
    let state = State::replay(&[written(1, first, es)]);
    let en = "---\nlang: en\ntranslationKey: from-another-tisty\n---\n# Walks\n";

    assert_eq!(state.keyed_to(en, Ulid::generate()), Some(first));
}

#[test]
fn every_language_shown_on_its_own_is_found_in_one_pass_and_one_alone_is_not() {
    let (es, en, again, bare) = (
        Ulid::generate(),
        Ulid::generate(),
        Ulid::generate(),
        Ulid::generate(),
    );
    let state = State::replay(&[
        written(1, es, ES),
        written(2, en, EN),
        written(3, again, EN),
        written(4, bare, "# Sin idioma\n"),
        joined(5, en, Some(es)),
        joined(6, again, Some(es)),
        joined(7, bare, Some(es)),
    ]);

    assert_eq!(state.apart(), [again, bare].into_iter().collect());
    assert!(state.apart_in_tongues(again) && !state.apart_in_tongues(es));

    let (one, other) = (Ulid::generate(), Ulid::generate());
    let pair = State::replay(&[
        written(1, one, ES),
        written(2, other, "# Sin idioma\n"),
        joined(3, other, Some(one)),
    ]);
    assert!(
        !pair.apart_in_tongues(one),
        "the only language said is nobody's double"
    );
    assert!(pair.apart_in_tongues(other));
}

#[test]
fn joining_moves_a_document_and_the_languages_beside_it_to_the_others_folder() {
    let (es, en, fr, de) = (
        Ulid::generate(),
        Ulid::generate(),
        Ulid::generate(),
        Ulid::generate(),
    );
    let there = Ulid::generate();
    let mut far = written(1, es, ES);
    if let Op::DocAdd { d, .. } = &mut far.op {
        d.folder = Some(there);
    }
    let mut state = State::replay(&[
        far,
        written(2, en, EN),
        written(3, fr, "---\nlang: fr\n---\n# Promenades\n"),
        written(4, de, "---\nlang: de\n---\n# Spazierg\u{e4}nge\n"),
        joined(5, fr, Some(en)),
    ]);
    assert!(state.tongue_taken(es, "es") && !state.tongue_taken(es, "en"));
    assert!(state.joining(fr, en).is_empty(), "already one of them");

    for op in state.joining(en, es) {
        state.apply(&at(10, op));
    }

    assert_eq!(state.docs[&en].folder, Some(there));
    assert_eq!(
        state.docs[&fr].folder,
        Some(there),
        "its languages came along"
    );
    assert_eq!(state.tongues_of(de).len(), 0);
    assert!(state.parting(de).is_empty(), "nothing to part from");
    let mut all = ids(&state.tongues_of(es));
    all.sort();
    let mut want = vec![es, en, fr];
    want.sort();
    assert_eq!(all, want);
}
