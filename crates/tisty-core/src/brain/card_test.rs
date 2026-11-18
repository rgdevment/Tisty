use jiff::civil::{Date, date};
use ulid::Ulid;

use super::super::Refreshed;
use super::super::tests::{World, doc, task};
use super::*;
use crate::event::TaskAdd;
use crate::model::{DateSpec, Folder, List, Status, Step};
use crate::{Op, docs};

fn today() -> Date {
    date(2026, 11, 17)
}

fn card_of(brain: &Brain, id: &str) -> Card {
    brain
        .card(id, today(), |_| true)
        .unwrap()
        .unwrap_or_else(|| panic!("{id} has a card"))
}

fn ids(all: &[Near]) -> Vec<&str> {
    all.iter().map(|one| one.id.as_str()).collect()
}

fn doc_facts(card: &Card) -> &DocFacts {
    match &card.facts {
        Facts::Doc(facts) => facts,
        Facts::Task(_) => panic!("{} is a document", card.id),
    }
}

fn task_facts(card: &Card) -> &TaskFacts {
    match &card.facts {
        Facts::Task(facts) => facts,
        Facts::Doc(_) => panic!("{} is a task", card.id),
    }
}

fn folder(id: Ulid, name: &str, parent: Option<Ulid>) -> Folder {
    Folder {
        id,
        name: name.into(),
        order: "a0".into(),
        parent,
        icon: None,
        color: None,
        archived: false,
    }
}

fn rewrite(world: &World, file: &str, body: &str) {
    std::fs::write(docs::resolve(&world.paths.docs(), file).unwrap(), body).unwrap();
}

#[test]
fn a_document_s_card_says_where_it_is_and_what_it_holds() {
    let world = World::new();
    let mut store = world.store();
    let body = "# Trips\n\nWhere to go.\n\n## Lisbon\n\nTrams.\n\n## Porto\n\nWine.\n\n#travel\n";
    let (trips, trips_file) = doc(&world, &mut store, body, None, None);
    let mut state = world.state();
    let (home, away) = (Ulid::generate(), Ulid::generate());
    state.folders.insert(home, folder(home, "Personal", None));
    state
        .folders
        .insert(away, folder(away, "Viajes", Some(home)));
    if let Some(one) = state.docs.get_mut(&trips) {
        one.folder = Some(away);
        one.lang = Some("en".into());
    }
    let mut brain = world.brain();

    world.refreshed(&mut brain, &state);

    let card = card_of(&brain, &trips_file);
    assert_eq!(
        (card.kind.as_str(), card.title.as_str(), card.state.as_str()),
        ("doc", "Trips", "open")
    );
    assert_eq!(card.folder.as_deref(), Some("Personal / Viajes"));
    assert_eq!(card.around.tags, ["travel"]);
    let facts = doc_facts(&card);
    assert_eq!(facts.lang.as_deref(), Some("en"));
    assert_eq!(facts.bytes, Some(body.len() as u64));
    assert_eq!(facts.lines, Some(body.lines().count()));
    let heads: Vec<(&str, usize)> = card
        .sections
        .iter()
        .map(|one| (one.title.as_str(), one.level))
        .collect();
    assert_eq!(heads, [("Trips", 1), ("Lisbon", 2), ("Porto", 2)]);
    let [from, to] = card.sections[1].bytes;
    assert_eq!(&body[from..to], "## Lisbon\n\nTrams.\n");
}

#[test]
fn a_book_reads_its_pages_in_order_and_a_page_names_its_book() {
    let world = World::new();
    let mut store = world.store();
    let (book, book_file) = doc(&world, &mut store, "# Trips\n", None, None);
    let (lisbon, lisbon_file) = doc(&world, &mut store, "# Lisbon\n", Some(book), None);
    let (porto, porto_file) = doc(&world, &mut store, "# Porto\n", Some(book), None);
    let mut state = world.state();
    let ordered = |state: &mut crate::State, id: Ulid, order: &str| {
        if let Some(one) = state.docs.get_mut(&id) {
            one.order = order.into();
        }
    };
    ordered(&mut state, lisbon, "b0");
    ordered(&mut state, porto, "a0");
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    assert_eq!(
        ids(&card_of(&brain, &book_file).around.pages),
        [porto_file.as_str(), lisbon_file.as_str()]
    );
    let page = card_of(&brain, &lisbon_file);
    assert_eq!(page.kind, "page");
    assert_eq!(page.around.book.map(|one| one.id), Some(book_file.clone()));

    ordered(&mut state, lisbon, "0");
    world.refreshed(&mut brain, &state);

    assert_eq!(
        ids(&card_of(&brain, &book_file).around.pages),
        [lisbon_file.as_str(), porto_file.as_str()],
        "a page moved without a word in the book still reorders it"
    );
}

#[test]
fn each_language_names_the_others_even_one_that_joins_later() {
    let world = World::new();
    let mut store = world.store();
    let (spanish, spanish_file) = doc(&world, &mut store, "# Viajes\n", None, None);
    let (english, english_file) = doc(&world, &mut store, "# Trips\n", None, None);
    let (portuguese, portuguese_file) = doc(&world, &mut store, "# Viagens\n", None, None);
    let mut state = world.state();
    let spoken = |state: &mut crate::State, id: Ulid, lang: &str, of: Option<Ulid>| {
        if let Some(one) = state.docs.get_mut(&id) {
            one.lang = Some(lang.into());
            one.translation_of = of;
        }
    };
    spoken(&mut state, spanish, "es", None);
    spoken(&mut state, english, "en", Some(spanish));
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    let langs = |brain: &Brain, file: &str| -> Vec<(String, String)> {
        card_of(brain, file)
            .around
            .languages
            .into_iter()
            .map(|one| (one.lang, one.doc))
            .collect()
    };
    assert_eq!(
        langs(&brain, &english_file),
        [("es".to_string(), spanish_file.clone())]
    );

    spoken(&mut state, portuguese, "pt", Some(spanish));
    world.refreshed(&mut brain, &state);

    let named: Vec<String> = langs(&brain, &spanish_file)
        .into_iter()
        .map(|(lang, _)| lang)
        .collect();
    assert_eq!(named, ["en", "pt"]);
    assert_eq!(langs(&brain, &portuguese_file).len(), 2);
}

#[test]
fn what_cites_and_mentions_a_card_is_read_from_the_other_side() {
    let world = World::new();
    let mut store = world.store();
    let errand = task(&mut store, "call the bank", &[]);
    let (trips, trips_file) = doc(&world, &mut store, "# Trips\n", None, None);
    let (_, notes) = doc(
        &world,
        &mut store,
        &format!(
            "# Notes\n\n![Trips](tisty:doc/{trips_file})\n\nCall [the bank](tisty:task/{errand}).\n"
        ),
        None,
        None,
    );
    let (_, plans) = doc(
        &world,
        &mut store,
        &format!("# Plans\n\n![Trips](tisty:doc/{trips})\n\n![bank](tisty:task/{errand})\n"),
        None,
        None,
    );
    let mut brain = world.brain();

    world.refreshed(&mut brain, &world.state());

    let from_notes = card_of(&brain, &notes);
    assert_eq!(ids(&from_notes.around.cites), [trips_file.as_str()]);
    let mut from_plans: Vec<String> = ids(&card_of(&brain, &plans).around.cites)
        .into_iter()
        .map(str::to_string)
        .collect();
    from_plans.sort();
    let mut wanted = vec![trips_file.clone(), errand.to_string()];
    wanted.sort();
    assert_eq!(
        from_plans, wanted,
        "a cite written by ULID names the document by its file"
    );
    assert_eq!(
        ids(&from_notes.around.mentions),
        [errand.to_string().as_str()]
    );
    let mut cited = ids(&card_of(&brain, &trips_file).around.cited_by)
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    cited.sort();
    let mut wanted = vec![notes.clone(), plans.clone()];
    wanted.sort();
    assert_eq!(cited, wanted, "a cite by ULID reaches the document too");
    let errand_card = card_of(&brain, &errand.to_string());
    assert_eq!(ids(&errand_card.around.mentioned_by), [notes.as_str()]);
    assert_eq!(
        ids(&errand_card.around.cited_by),
        [plans.as_str()],
        "the documents that cite a task"
    );
    let by_ulid = card_of(&brain, &trips.to_string().to_ascii_lowercase());
    assert_eq!(
        by_ulid.id, trips_file,
        "a ULID finds its document in any case"
    );
}

#[test]
fn a_task_s_card_says_how_it_stands_and_what_it_left() {
    let world = World::new();
    let mut store = world.store();
    let whole = task(&mut store, "move house", &[]);
    let part = Ulid::generate();
    let mut d = TaskAdd::new("pack the books", "a0");
    d.part_of = Some(whole);
    d.tags = vec![crate::model::Tag::new("errands").unwrap()];
    store.append(Op::TaskAdd { id: part, d }).unwrap();
    let mut state = world.state();
    state
        .kept
        .insert("attachments/form-1a2b3c4d.pdf".into(), ("ab12".into(), 3));
    let errands = Ulid::generate();
    state.lists.insert(
        errands,
        List {
            id: errands,
            name: "Errands".into(),
            order: "a0".into(),
            color: None,
            icon: None,
            archived: false,
        },
    );
    if let Some(one) = state.tasks.get_mut(&part) {
        one.description = Some(
            "Fill [the form](https://bank.example/form) and keep ![it](attachments/form-1a2b3c4d.pdf)."
                .into(),
        );
        one.list = Some(errands);
        one.priority = crate::model::Priority::Do;
        one.date = Some(DateSpec::all_day(date(2026, 11, 10), "UTC"));
        one.deadline = Some(DateSpec::all_day(date(2026, 11, 16), "UTC"));
        one.steps = ["empty the shelves", "tape the boxes"]
            .iter()
            .enumerate()
            .map(|(at, text)| Step {
                id: Ulid::generate(),
                text: (*text).into(),
                done: at == 0,
                order: format!("a{at}"),
                by_agent: false,
                said: false,
            })
            .collect();
        one.retally();
    }
    let mut brain = world.brain();

    world.refreshed(&mut brain, &state);

    let card = card_of(&brain, &part.to_string());
    assert_eq!(card.list.as_deref(), Some("Errands"));
    assert_eq!(card.state, "open");
    assert_eq!(card.standing, Some(Standing::Overdue));
    let facts = task_facts(&card);
    assert_eq!(facts.priority, Some(crate::model::Priority::Do));
    assert_eq!(facts.deadline, Some(date(2026, 11, 16)));
    assert_eq!(facts.steps, Some([1, 2]));
    assert_eq!(facts.links, ["https://bank.example/form"]);
    assert_eq!(
        card.around.files,
        [Used {
            file: "attachments/form-1a2b3c4d.pdf".into(),
            shows: None
        }]
    );
    assert_eq!(card.around.tags, ["errands"]);
    assert_eq!(
        card.around.part_of.map(|one| one.id),
        Some(whole.to_string())
    );
    assert_eq!(
        ids(&card_of(&brain, &whole.to_string()).around.parts),
        [part.to_string().as_str()]
    );

    if let Some(one) = state.tasks.get_mut(&part) {
        one.status = Status::Done;
    }
    world.refreshed(&mut brain, &state);

    assert_eq!(
        card_of(&brain, &part.to_string()).standing,
        None,
        "a closed task is late for nothing"
    );
}

#[test]
fn a_page_with_an_idea_hangs_from_its_book_and_holds_the_sheet_s_idea() {
    let world = World::new();
    let mut store = world.store();
    let (_, sheet) = doc(
        &world,
        &mut store,
        &crate::brainstorm::opening("Ideas"),
        None,
        None,
    );
    let idea = crate::brainstorm::jotted(
        "walk along the river",
        "2026-11-17T10:00".parse().unwrap(),
        crate::brainstorm::COLOURS[0],
    );
    let (book, book_file) = doc(&world, &mut store, "# Trips\n", None, None);
    let (_, page) = doc(
        &world,
        &mut store,
        &format!("# Lisbon\n\n{idea}\n"),
        Some(book),
        None,
    );
    rewrite(
        &world,
        &book_file,
        &format!("# Trips\n\n![Lisbon](tisty:doc/{page})\n"),
    );
    let born = Ulid::generate();
    let mut d = TaskAdd::new("try the new bakery", "a0");
    d.source = Some(format!("{sheet}#2026-11-17 10:00 · yellow"));
    store.append(Op::TaskAdd { id: born, d }).unwrap();
    let mut brain = world.brain();

    world.refreshed(&mut brain, &world.state());

    let page_card = card_of(&brain, &page);
    assert_eq!(
        page_card.around.book.map(|one| one.id),
        Some(book_file.clone())
    );
    assert_eq!(
        page_card.around.ideas_from.map(|one| one.id),
        Some(sheet.clone())
    );
    let book_card = card_of(&brain, &book_file);
    assert_eq!(ids(&book_card.around.pages), [page.as_str()]);
    assert!(book_card.around.ideas.is_empty());
    let born_card = card_of(&brain, &born.to_string());
    assert_eq!(
        born_card.around.origin.map(|one| one.id),
        Some(sheet.clone())
    );
    assert_eq!(born_card.around.ideas_from, None);
    let sheet_card = card_of(&brain, &sheet);
    assert_eq!(ids(&sheet_card.around.ideas), [page.as_str()]);
    assert_eq!(ids(&sheet_card.around.born), [born.to_string().as_str()]);
}

#[test]
fn a_task_whose_source_names_any_document_is_born_of_it() {
    let world = World::new();
    let mut store = world.store();
    let (_, budget) = doc(&world, &mut store, "# Budget\n\n## Rent\n", None, None);
    let born = Ulid::generate();
    let mut d = TaskAdd::new("pay the rent", "a0");
    d.source = Some(format!("{budget}#Rent"));
    store.append(Op::TaskAdd { id: born, d }).unwrap();
    let mut brain = world.brain();

    world.refreshed(&mut brain, &world.state());

    let card = card_of(&brain, &budget);
    assert_eq!(ids(&card.around.born), [born.to_string().as_str()]);
    assert!(
        card.around.ideas.is_empty(),
        "a plain document holds no ideas"
    );
}

#[test]
fn what_the_reader_cannot_see_is_left_out() {
    let world = World::new();
    let mut store = world.store();
    let secret = task(&mut store, "a surprise party", &[]);
    let (_, notes) = doc(
        &world,
        &mut store,
        &format!("# Notes\n\nSee [the party](tisty:task/{secret}).\n"),
        None,
        None,
    );
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());
    let hidden = secret.to_string();
    let seen = |id: &str| id != hidden;

    let card = brain.card(&notes, today(), seen).unwrap().unwrap();

    assert!(card.around.mentions.is_empty());
    assert_eq!(brain.card(&hidden, today(), seen).unwrap(), None);
    assert_eq!(
        brain.card("#nothing", today(), |_| true).unwrap(),
        None,
        "only documents, pages and tasks have a card"
    );
}

#[test]
fn a_source_naming_what_the_reader_cannot_see_is_left_unsaid() {
    let world = World::new();
    let mut store = world.store();
    let (_, diary) = doc(
        &world,
        &mut store,
        "# Diary\n\n## Party plans\n",
        None,
        None,
    );
    let born = Ulid::generate();
    let mut d = TaskAdd::new("buy balloons", "a0");
    d.source = Some(format!("{diary}#Party plans"));
    store.append(Op::TaskAdd { id: born, d }).unwrap();
    let mut elsewhere = TaskAdd::new("answer the school", "a0");
    elsewhere.source = Some("sereno#1234".into());
    let from_chat = Ulid::generate();
    store
        .append(Op::TaskAdd {
            id: from_chat,
            d: elsewhere,
        })
        .unwrap();
    let early = Ulid::generate();
    let mut ahead = TaskAdd::new("wrap the present", "a0");
    ahead.source = Some("zz-0009#The present".into());
    store
        .append(Op::TaskAdd {
            id: early,
            d: ahead,
        })
        .unwrap();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());
    let seen = |id: &str| id != diary && id != "zz-0009";
    let before_it_arrives = brain
        .card(&early.to_string(), today(), seen)
        .unwrap()
        .unwrap();
    assert_eq!(
        task_facts(&before_it_arrives).source,
        None,
        "a hidden document not indexed yet is not named either"
    );

    let card = brain
        .card(&born.to_string(), today(), seen)
        .unwrap()
        .unwrap();

    assert_eq!(task_facts(&card).source, None);
    assert_eq!(card.around.origin, None);
    let open = card_of(&brain, &born.to_string());
    assert_eq!(
        task_facts(&open).source.as_deref(),
        Some(format!("{diary}#Party plans").as_str())
    );
    let chat = brain
        .card(&from_chat.to_string(), today(), seen)
        .unwrap()
        .unwrap();
    assert_eq!(
        task_facts(&chat).source.as_deref(),
        Some("sereno#1234"),
        "a source that names no document stays"
    );
}

#[test]
fn a_task_born_of_another_task_s_page_is_no_part_of_it() {
    let world = World::new();
    let mut store = world.store();
    let first = task(&mut store, "plan the garden", &[]);
    let page = crate::task_docs::file_at(first, 1);
    docs::create_named(&world.paths.docs(), &page, "# Garden\n\n## Tomatoes\n").unwrap();
    store
        .append(Op::DocAdd {
            id: crate::task_docs::id_at(first, 1),
            d: crate::event::DocAdd {
                wrote: None,
                guest: false,
                sown: false,
                made: None,
                by: None,
                said: None,
                file: page.clone(),
                order: "a0".into(),
                folder: None,
                page_of: None,
                task_of: Some(first),
                edited: None,
            },
        })
        .unwrap();
    let born = Ulid::generate();
    let mut d = TaskAdd::new("plant tomatoes", "a0");
    d.source = Some(format!("{page}#Tomatoes"));
    store.append(Op::TaskAdd { id: born, d }).unwrap();
    let part = Ulid::generate();
    let mut d = TaskAdd::new("buy seeds", "a0");
    d.part_of = Some(first);
    store.append(Op::TaskAdd { id: part, d }).unwrap();
    let mut brain = world.brain();

    world.refreshed(&mut brain, &world.state());

    let card = card_of(&brain, &first.to_string());
    assert_eq!(ids(&card.around.parts), [part.to_string().as_str()]);
    assert_eq!(ids(&card.around.born), [born.to_string().as_str()]);
}

#[test]
fn a_folder_renamed_is_read_without_writing_its_documents_again() {
    let world = World::new();
    let mut store = world.store();
    let (trips, trips_file) = doc(&world, &mut store, "# Trips\n", None, None);
    let mut state = world.state();
    let place = Ulid::generate();
    state.folders.insert(place, folder(place, "Travel", None));
    if let Some(one) = state.docs.get_mut(&trips) {
        one.folder = Some(place);
    }
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    if let Some(one) = state.folders.get_mut(&place) {
        one.name = "Trips abroad".into();
    }

    assert_eq!(world.refreshed(&mut brain, &state), Refreshed::Unchanged);
    assert_eq!(
        card_of(&brain, &trips_file).folder.as_deref(),
        Some("Trips abroad")
    );
}

#[test]
fn a_card_reads_identity_first_and_sections_last() {
    let world = World::new();
    let mut store = world.store();
    let (_, trips) = doc(&world, &mut store, "# Trips\n\n#travel\n", None, None);
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());

    let said = serde_json::to_string(&card_of(&brain, &trips)).unwrap();

    assert!(said.starts_with(&format!(
        "{{\"id\":\"{trips}\",\"kind\":\"doc\",\"title\":\"Trips\",\"state\":\"open\""
    )));
    assert!(said.contains("\"tags\":[\"travel\"]"));
    assert!(
        said.ends_with(
            "\"sections\":[{\"title\":\"Trips\",\"level\":1,\"lines\":[1,3],\"bytes\":[0,17]}]}"
        ),
        "{said}"
    );
}

#[test]
fn a_bare_task_id_and_a_double_bracket_title_are_mentions_read_from_both_ends() {
    let world = World::new();
    let mut store = world.store();
    let errand = task(&mut store, "call the bank", &[]);
    let (_, trips) = doc(&world, &mut store, "# Trips\n", None, None);
    let (_, notes) = doc(
        &world,
        &mut store,
        &format!("# Notes\n\nCall {errand} today, as [[Trips]] says.\n"),
        None,
        None,
    );
    let mut brain = world.brain();

    world.refreshed(&mut brain, &world.state());

    let mut mentioned: Vec<String> = ids(&card_of(&brain, &notes).around.mentions)
        .into_iter()
        .map(str::to_string)
        .collect();
    mentioned.sort();
    let mut wanted = vec![errand.to_string(), trips.clone()];
    wanted.sort();
    assert_eq!(mentioned, wanted);
    assert_eq!(
        ids(&card_of(&brain, &trips).around.mentioned_by),
        [notes.as_str()]
    );
    assert_eq!(
        ids(&card_of(&brain, &errand.to_string()).around.mentioned_by),
        [notes.as_str()]
    );
}

#[test]
fn a_double_bracket_title_two_documents_carry_names_neither() {
    let world = World::new();
    let mut store = world.store();
    let (_, first) = doc(&world, &mut store, "# Trips\n", None, None);
    let (_, second) = doc(&world, &mut store, "# Trips\n\nAgain.\n", None, None);
    let (_, notes) = doc(
        &world,
        &mut store,
        "# Notes\n\nSee [[Trips]].\n",
        None,
        None,
    );
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());

    assert!(card_of(&brain, &notes).around.mentions.is_empty());
    assert!(card_of(&brain, &first).around.mentioned_by.is_empty());

    let hidden = second.clone();
    let seen = |id: &str| id != hidden;
    let card = brain.card(&notes, today(), seen).unwrap().unwrap();
    assert_eq!(
        ids(&card.around.mentions),
        [first.as_str()],
        "a twin the reader cannot see is not counted, so it neither shows nor gives itself away"
    );
    let first_card = brain.card(&first, today(), seen).unwrap().unwrap();
    assert_eq!(ids(&first_card.around.mentioned_by), [notes.as_str()]);
}

#[test]
fn a_document_renamed_to_a_double_bracket_title_is_found_without_its_mention_written_again() {
    let world = World::new();
    let mut store = world.store();
    let (_, plans) = doc(&world, &mut store, "# Plans\n", None, None);
    let (_, notes) = doc(
        &world,
        &mut store,
        "# Notes\n\nSee [[Trips]].\n",
        None,
        None,
    );
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());
    assert!(card_of(&brain, &notes).around.mentions.is_empty());

    rewrite(&world, &plans, "# Trips\n");
    let mut state = world.state();
    if let Some(one) = state.docs.values_mut().find(|one| one.file == plans) {
        one.title = Some("Trips".into());
    }
    world.refreshed(&mut brain, &state);

    assert_eq!(
        ids(&card_of(&brain, &notes).around.mentions),
        [plans.as_str()]
    );
}

#[test]
fn a_picture_s_card_line_says_what_it_shows_and_a_file_name_says_nothing() {
    let world = World::new();
    let mut store = world.store();
    let (_, album) = doc(
        &world,
        &mut store,
        "# Album\n\n![Rina on her chair](rina-3f9a1c2b.jpg)\n\n\
         ![IMG_2034.PNG](img-2034-9d02e4b7.png)\n\n![The form](form-1a2b3c4d.pdf)\n",
        None,
        None,
    );
    let mut state = world.state();
    for at in [
        "attachments/rina-3f9a1c2b.jpg",
        "attachments/img-2034-9d02e4b7.png",
        "attachments/form-1a2b3c4d.pdf",
    ] {
        state.kept.insert(at.into(), ("ab12".into(), 3));
    }
    let mut brain = world.brain();

    world.refreshed(&mut brain, &state);

    let mut files = card_of(&brain, &album).around.files;
    files.sort_by(|one, other| one.file.cmp(&other.file));
    assert_eq!(
        files,
        [
            Used {
                file: "attachments/form-1a2b3c4d.pdf".into(),
                shows: None
            },
            Used {
                file: "attachments/img-2034-9d02e4b7.png".into(),
                shows: None
            },
            Used {
                file: "attachments/rina-3f9a1c2b.jpg".into(),
                shows: Some("Rina on her chair".into())
            },
        ]
    );
}

#[test]
fn a_document_s_decisions_come_in_force_first_and_the_struck_ones_after() {
    let world = World::new();
    let mut store = world.store();
    let body = "# Trips\n\n## Decisions\n\n- ~~2026-11-12 · The car goes.~~ Changed.\n\
                - **2026-11-13 · We walk.** Always.\n";
    let (_, trips) = doc(&world, &mut store, body, None, None);
    let mut brain = world.brain();

    world.refreshed(&mut brain, &world.state());

    let card = card_of(&brain, &trips);
    let said: Vec<(usize, &str, bool)> = card
        .decisions
        .iter()
        .map(|one| (one.line, one.said.as_str(), one.replaced))
        .collect();
    assert_eq!(
        said,
        [
            (6, "2026-11-13 · We walk.", false),
            (5, "2026-11-12 · The car goes.", true)
        ]
    );
    assert_eq!(card.decisions_more, None);
    let json = serde_json::to_string(&card).unwrap();
    assert!(
        json.contains(
            "\"decisions\":[{\"line\":6,\"said\":\"2026-11-13 · We walk.\"},\
             {\"line\":5,\"said\":\"2026-11-12 · The car goes.\",\"replaced\":true}],\"sections\""
        ),
        "{json}"
    );
}
