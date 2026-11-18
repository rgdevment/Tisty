use ulid::Ulid;

use super::super::tests::{World, doc, task};
use super::*;
use crate::Op;

fn citing(id: &str, kind: &str, title: &str, card: bool, under: &[(&str, &str)]) -> Citing {
    Citing {
        id: id.into(),
        kind: kind.into(),
        title: title.into(),
        state: Some("open".into()),
        card,
        under: under
            .iter()
            .map(|(id, title)| Under {
                id: (*id).into(),
                title: (*title).into(),
            })
            .collect(),
    }
}

fn written(world: &World, file: &str, body: &str) {
    std::fs::write(
        crate::docs::resolve(&world.paths.docs(), file).unwrap(),
        body,
    )
    .unwrap();
}

#[test]
fn whoever_cites_or_mentions_a_document_is_told_once_with_the_headings_over_each_link() {
    let world = World::new();
    let mut store = world.store();
    let (vet_id, vet) = doc(&world, &mut store, "# Vet\n", None, None);
    let (_, trips) = doc(&world, &mut store, "# Trips\n", None, None);
    written(
        &world,
        &trips,
        &format!(
            "# Trips\n\n## Hotels\n\nAsk the [vet](tisty:doc/{vet}#rabies).\n\n### By the sea\n\n\
             ![Vet](tisty:doc/{vet})\n\n## Food\n\n`[vet](tisty:doc/{vet})`\n"
        ),
    );
    let (_, notes) = doc(&world, &mut store, "# Notes\n", None, None);
    written(
        &world,
        &notes,
        &format!("# Notes\n\nSee the [card](tisty:doc/{vet_id}).\n"),
    );
    let errand = task(&mut store, "book the vet", &[]);
    store
        .append(Op::TaskLog {
            id: errand,
            d: crate::event::LogAdd::new(Ulid::generate(), format!("[vet](tisty:doc/{vet})")),
        })
        .unwrap();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());

    let said = brain
        .cited_by(&vet, &world.paths.docs(), &mut Headed::default())
        .unwrap()
        .unwrap();

    assert_eq!(
        said,
        vec![
            citing(&notes, "doc", "Notes", false, &[]),
            citing(
                &trips,
                "doc",
                "Trips",
                true,
                &[("hotels", "Hotels"), ("by-the-sea", "By the sea")]
            ),
            citing(&errand.to_string(), "task", "book the vet", false, &[]),
        ]
    );
    assert_eq!(
        brain
            .cited_by(
                &vet_id.to_string(),
                &world.paths.docs(),
                &mut Headed::default()
            )
            .unwrap(),
        Some(said),
        "the document's ULID names it as its file does"
    );
}

#[test]
fn a_task_is_told_who_cites_it_by_card_by_its_page_file_and_from_another_task() {
    let world = World::new();
    let mut store = world.store();
    let tyres = task(&mut store, "Cambiar los neumáticos", &[]);
    let (_, car) = doc(&world, &mut store, "# Auto\n", None, None);
    written(
        &world,
        &car,
        &format!("# Auto\n\n## Taller\n\n![Neumáticos](tisty:task/{tyres})\n"),
    );
    let page = format!("t_{}-0001", tyres.to_string().to_lowercase());
    let (_, notes) = doc(&world, &mut store, "# Notas\n", None, None);
    written(
        &world,
        &notes,
        &format!("# Notas\n\n## Marzo\n\nComo dice [su página](tisty:doc/{page}).\n"),
    );
    let trip = task(&mut store, "Viaje al sur", &[]);
    store
        .append(Op::TaskLog {
            id: trip,
            d: crate::event::LogAdd::new(
                Ulid::generate(),
                format!("antes: [neumáticos](tisty:task/{tyres})"),
            ),
        })
        .unwrap();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());

    let said = brain
        .cited_by(
            &tyres.to_string(),
            &world.paths.docs(),
            &mut Headed::default(),
        )
        .unwrap()
        .unwrap();

    assert_eq!(
        said,
        vec![
            citing(&car, "doc", "Auto", true, &[("taller", "Taller")]),
            citing(&notes, "doc", "Notas", false, &[("marzo", "Marzo")]),
            citing(&trip.to_string(), "task", "Viaje al sur", false, &[]),
        ]
    );
}

#[test]
fn a_page_its_tags_and_its_files_are_no_citation() {
    let world = World::new();
    let mut store = world.store();
    let (book_id, book) = doc(&world, &mut store, "# Trips\n\n#travel\n", None, None);
    let (_, page) = doc(&world, &mut store, "# Lisbon\n", Some(book_id), None);
    written(
        &world,
        &book,
        &format!("# Trips\n\n![Lisbon](tisty:doc/{page})\n"),
    );
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());

    assert_eq!(
        brain
            .cited_by(&book, &world.paths.docs(), &mut Headed::default())
            .unwrap(),
        Some(vec![])
    );
    assert_eq!(
        brain
            .cited_by(&page, &world.paths.docs(), &mut Headed::default())
            .unwrap(),
        Some(vec![]),
        "the book's page line places the page; it does not cite it"
    );
}

#[test]
fn a_base_no_round_has_filled_says_nothing_rather_than_nobody() {
    let world = World::new();
    let brain = world.brain();

    assert_eq!(
        brain
            .cited_by("a-0001", &world.paths.docs(), &mut Headed::default())
            .unwrap(),
        None
    );
}

#[test]
fn a_reader_neither_makes_a_base_nor_reads_a_newer_one() {
    let world = World::new();

    assert!(Brain::looked_at(world.paths.cache()).is_none());
    assert!(!world.paths.cache().join(FILE).exists());

    let brain = world.brain();
    brain
        .db
        .execute("UPDATE meta SET value = '999' WHERE key = 'schema'", [])
        .unwrap();
    drop(brain);

    assert!(Brain::looked_at(world.paths.cache()).is_none());
}

#[test]
fn headings_are_found_past_front_matter_and_carriage_returns() {
    let aimed = ["a-0001".to_string(), "a-0001".to_string()];
    let body =
        "---\r\ntags: [x]\r\n---\r\n# Doc\r\n\r\n## Más tarde\r\n\r\n[a](tisty:doc/a-0001)\r\n";

    assert_eq!(
        under(&headed(body), &aimed),
        vec![Under {
            id: "más-tarde".into(),
            title: "Más tarde".into()
        }]
    );
    assert_eq!(
        under(&headed("# Doc\n\n[a](tisty:doc/a-0001)\n"), &aimed),
        vec![],
        "the title is the document itself, not a section of it"
    );
}

#[test]
fn a_citing_body_is_read_again_only_once_a_round_has_seen_it_move() {
    let world = World::new();
    let mut store = world.store();
    let (_, vet) = doc(&world, &mut store, "# Vet\n", None, None);
    let (_, trips) = doc(&world, &mut store, "# Trips\n", None, None);
    written(
        &world,
        &trips,
        &format!("# Trips\n\n## Hotels\n\n[vet](tisty:doc/{vet})\n"),
    );
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());
    let mut headed = Headed::default();
    let sections = |brain: &Brain, headed: &mut Headed| -> Vec<String> {
        brain
            .cited_by(&vet, &world.paths.docs(), headed)
            .unwrap()
            .unwrap()[0]
            .under
            .iter()
            .map(|one| one.id.clone())
            .collect()
    };
    assert_eq!(sections(&brain, &mut headed), ["hotels"]);

    written(
        &world,
        &trips,
        &format!("# Trips\n\n## Food\n\n[vet](tisty:doc/{vet})\n"),
    );
    assert_eq!(
        sections(&brain, &mut headed),
        ["hotels"],
        "no round saw the body move, so what it said is kept"
    );

    world.refreshed(&mut brain, &world.state());
    assert_eq!(sections(&brain, &mut headed), ["food"]);
}

#[test]
fn a_body_that_could_not_be_read_is_tried_again_on_the_next_ask() {
    let world = World::new();
    let mut store = world.store();
    let (_, vet) = doc(&world, &mut store, "# Vet\n", None, None);
    let (_, trips) = doc(&world, &mut store, "# Trips\n", None, None);
    written(
        &world,
        &trips,
        &format!("# Trips\n\n## Hotels\n\n[vet](tisty:doc/{vet})\n"),
    );
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());
    let at = crate::docs::resolve(&world.paths.docs(), &trips).unwrap();
    let body = std::fs::read(&at).unwrap();
    std::fs::remove_file(&at).unwrap();
    let mut headed = Headed::default();

    let gone = brain
        .cited_by(&vet, &world.paths.docs(), &mut headed)
        .unwrap()
        .unwrap();
    std::fs::write(&at, body).unwrap();
    let back = brain
        .cited_by(&vet, &world.paths.docs(), &mut headed)
        .unwrap()
        .unwrap();

    assert!(gone[0].under.is_empty());
    assert_eq!(back[0].under[0].id, "hotels");
}
