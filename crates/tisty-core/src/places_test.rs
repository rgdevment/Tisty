use super::*;
use crate::event::{DeviceId, DocAdd, Event, FolderAdd};
use crate::{Op, State};
use ulid::Ulid;

fn ev(ms: i64, op: Op) -> Event {
    Event::new(
        DeviceId("dev_desk".into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    )
}

fn folder(name: &str, parent: Option<FolderId>, icon: Option<&str>) -> (FolderId, Op) {
    let id = Ulid::generate();
    let d = FolderAdd {
        name: name.into(),
        order: "a0".into(),
        parent,
        icon: icon.map(str::to_string),
        color: None,
        computer: None,
    };
    (id, Op::FolderAdd { id, d })
}

fn doc(file: &str, folder: Option<FolderId>, page_of: Option<DocId>) -> (DocId, Op) {
    let id = Ulid::generate();
    let d = DocAdd {
        file: file.into(),
        order: "a0".into(),
        folder,
        page_of,
        ..Default::default()
    };
    (id, Op::DocAdd { id, d })
}

struct World {
    state: State,
    plain: DocId,
    plain_folder: FolderId,
    ideas: DocId,
    ideas_page: DocId,
    diary: DocId,
    diary_page: DocId,
    agents: FolderId,
    agents_sub: FolderId,
    agents_doc: DocId,
}

fn world() -> World {
    let (plain_folder, add_plain_folder) = folder("Work", None, None);
    let (agents, add_agents) = folder(crate::agents::FOLDER, None, Some(crate::agents::ICON));
    let (agents_sub, add_sub) = folder("rules", Some(agents), None);
    let (plain, add_plain) = doc("dev_desk-0001", Some(plain_folder), None);
    let (ideas, add_ideas) = doc("dev_desk-0002", None, None);
    let (ideas_page, add_ideas_page) = doc("dev_desk-0003", None, Some(ideas));
    let (diary, add_diary) = doc("dev_desk-0004", None, None);
    let (diary_page, add_diary_page) = doc("dev_desk-0005", None, Some(diary));
    let (agents_doc, add_agents_doc) = doc("dev_desk-0006", Some(agents_sub), None);
    let ops = [
        add_plain_folder,
        add_agents,
        add_sub,
        add_plain,
        add_ideas,
        add_ideas_page,
        add_diary,
        add_diary_page,
        add_agents_doc,
    ];
    let events: Vec<Event> = ops
        .into_iter()
        .enumerate()
        .map(|(at, op)| ev(at as i64 + 1, op))
        .collect();
    let mut state = State::replay(&events);
    state.docs.get_mut(&ideas).unwrap().brainstorm = true;
    state.docs.get_mut(&diary).unwrap().diary = true;
    World {
        state,
        plain,
        plain_folder,
        ideas,
        ideas_page,
        diary,
        diary_page,
        agents,
        agents_sub,
        agents_doc,
    }
}

fn places_of(world: &World) -> Places {
    Places::read(Path::new("no-such-docs-folder"), &world.state)
}

fn code(ruled: Result<(), Ruled>) -> Option<&'static str> {
    ruled.err().map(|one| one.0)
}

#[test]
fn only_a_plain_folder_makes_everything() {
    for what in MAKINGS {
        assert!(Place::Plain.makes(what).is_ok(), "{what:?}");
    }
    assert_eq!(Place::Agents.rules().makes, vec![Making::Doc]);
}

#[test]
fn each_place_has_its_own_rules() {
    let rules = |place: Place| {
        let all = place.rules();
        (all.makes.len(), all.pages, all.moves, all.copies)
    };
    assert_eq!(rules(Place::Plain), (MAKINGS.len(), true, true, true));
    assert_eq!(rules(Place::Ideas), (0, false, false, false));
    assert_eq!(rules(Place::Diary), (0, false, false, false));
    assert_eq!(rules(Place::Agents), (1, true, true, true));
}

#[test]
fn each_place_refuses_with_its_own_code() {
    assert_eq!(
        code(Place::Ideas.makes(Making::Doc)),
        Some("ideasHoldOnlyIdeas")
    );
    assert_eq!(
        code(Place::Diary.makes(Making::Folder)),
        Some("diaryHoldsOnlyDays")
    );
    assert_eq!(
        code(Place::Agents.makes(Making::Folder)),
        Some("agentsHoldOnlyRules")
    );
    assert_eq!(code(Place::Ideas.takes_pages()), Some("ideasHoldNoPages"));
    assert_eq!(code(Place::Diary.takes_pages()), Some("diaryHoldsOnlyDays"));
    assert_eq!(code(Place::Ideas.moves()), Some("ideasStayPut"));
    assert_eq!(code(Place::Diary.moves()), Some("diaryStaysPut"));
    assert_eq!(code(Place::Ideas.copies()), Some("ideasAreOne"));
    assert_eq!(code(Place::Diary.copies()), Some("diaryIsOne"));
}

#[test]
fn a_page_takes_the_place_of_its_book() {
    let world = world();
    let places = places_of(&world);
    let of = |id: DocId| places.of_doc(&world.state, &world.state.docs[&id]);

    assert_eq!(of(world.ideas), Place::Ideas);
    assert_eq!(of(world.ideas_page), Place::Ideas);
    assert_eq!(of(world.diary), Place::Diary);
    assert_eq!(of(world.diary_page), Place::Diary);
    assert_eq!(of(world.agents_doc), Place::Agents);
    assert_eq!(of(world.plain), Place::Plain);
}

#[test]
fn a_folder_is_agents_only_inside_its_tree() {
    let world = world();
    let places = places_of(&world);

    assert_eq!(places.of_folder(Some(world.agents)), Place::Agents);
    assert_eq!(places.of_folder(Some(world.agents_sub)), Place::Agents);
    assert_eq!(places.of_folder(Some(world.plain_folder)), Place::Plain);
    assert_eq!(places.of_folder(None), Place::Plain);
}

#[test]
fn hanging_asks_the_page_to_move_and_the_book_to_take() {
    let world = world();
    let places = places_of(&world);
    let kept = |id: DocId| &world.state.docs[&id];
    let hang =
        |page: DocId, book: DocId| code(places.may_hang(&world.state, kept(page), kept(book)));

    assert_eq!(hang(world.plain, world.agents_doc), None);
    assert_eq!(hang(world.agents_doc, world.plain), None);
    assert_eq!(hang(world.ideas, world.plain), Some("ideasStayPut"));
    assert_eq!(hang(world.diary, world.plain), Some("diaryStaysPut"));
    assert_eq!(hang(world.plain, world.ideas), Some("ideasHoldNoPages"));
    assert_eq!(hang(world.plain, world.diary), Some("diaryHoldsOnlyDays"));
    assert_eq!(
        hang(world.plain, world.ideas_page),
        Some("ideasHoldNoPages")
    );
}

#[test]
fn filing_and_copying_follow_the_place_of_the_document() {
    let world = world();
    let places = places_of(&world);
    let kept = |id: DocId| &world.state.docs[&id];

    assert!(places.may_file(&world.state, kept(world.plain)).is_ok());
    assert!(
        places
            .may_file(&world.state, kept(world.agents_doc))
            .is_ok()
    );
    assert_eq!(
        code(places.may_file(&world.state, kept(world.ideas_page))),
        Some("ideasStayPut")
    );
    assert_eq!(
        code(places.may_file(&world.state, kept(world.diary_page))),
        Some("diaryStaysPut")
    );
    assert_eq!(
        code(places.may_copy(&world.state, kept(world.ideas))),
        Some("ideasAreOne")
    );
    assert_eq!(
        code(places.may_copy(&world.state, kept(world.diary))),
        Some("diaryIsOne")
    );
    assert!(places.may_copy(&world.state, kept(world.plain)).is_ok());
}

#[test]
fn inside_agents_only_a_doc_is_made() {
    let world = world();
    let places = places_of(&world);

    for at in [Some(world.agents), Some(world.agents_sub)] {
        assert_eq!(code(places.may_make(at, Making::Doc)), None);
        for what in [Making::Folder, Making::Diagram, Making::Import] {
            assert_eq!(
                code(places.may_make(at, what)),
                Some("agentsHoldOnlyRules"),
                "{what:?}"
            );
        }
    }
    for what in MAKINGS {
        assert_eq!(code(places.may_make(Some(world.plain_folder), what)), None);
    }
}
