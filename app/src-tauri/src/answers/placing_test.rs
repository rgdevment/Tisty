use super::*;
use tisty_core::Paths;
use tisty_core::event::{DocAdd, FolderAdd};
use tisty_core::{Op, agents};
use ulid::Ulid;

struct Shelf {
    _tmp: tempfile::TempDir,
    session: Session,
    plain: DocId,
    ideas: DocId,
    ideas_page: DocId,
    diary: DocId,
    diary_page: DocId,
    agents: FolderId,
    agents_doc: DocId,
    work: FolderId,
}

fn folder(session: &mut Session, name: &str, icon: Option<&str>) -> FolderId {
    let id = Ulid::generate();
    let d = FolderAdd {
        name: name.into(),
        order: "a0".into(),
        parent: None,
        icon: icon.map(str::to_string),
        color: None,
        computer: None,
    };
    session.commit(Op::FolderAdd { id, d }).unwrap();
    id
}

const DIARY: &str = "---
diary: on
---

# 2026
";

fn doc(session: &mut Session, folder: Option<FolderId>, page_of: Option<DocId>) -> DocId {
    written(
        session, folder, page_of, "# Note
",
    )
}

fn written(
    session: &mut Session,
    folder: Option<FolderId>,
    page_of: Option<DocId>,
    body: &str,
) -> DocId {
    let device = session.store.device().clone();
    let made = tisty_core::docs::create(&session.paths.docs(), &device, body).unwrap();
    let id = Ulid::generate();
    let d = DocAdd {
        file: made.id,
        folder,
        page_of,
        order: "a0".into(),
        ..Default::default()
    };
    session.commit(Op::DocAdd { id, d }).unwrap();
    id
}

fn shelf() -> Shelf {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths).unwrap();
    let work = folder(&mut session, "Work", None);
    let agents = folder(&mut session, agents::FOLDER, Some(agents::ICON));
    let plain = doc(&mut session, Some(work), None);
    let ideas = doc(&mut session, None, None);
    let ideas_page = doc(&mut session, None, Some(ideas));
    let diary = written(&mut session, None, None, DIARY);
    let diary_page = doc(&mut session, None, Some(diary));
    let agents_doc = doc(&mut session, Some(agents), None);
    session.state.docs.get_mut(&ideas).unwrap().brainstorm = true;
    session.state.docs.get_mut(&diary).unwrap().diary = true;
    Shelf {
        _tmp: tmp,
        session,
        plain,
        ideas,
        ideas_page,
        diary,
        diary_page,
        agents,
        agents_doc,
        work,
    }
}

fn code(answer: Answer<()>) -> Option<&'static str> {
    answer.err().map(|one| one.code)
}

#[test]
fn ideas_and_the_diary_cannot_be_hung_as_pages_nor_let_go() {
    let shelf = shelf();
    let hang = |page, book| code(may_hang(&shelf.session, page, book));

    assert_eq!(hang(shelf.ideas, Some(shelf.plain)), Some("ideasStayPut"));
    assert_eq!(hang(shelf.diary, Some(shelf.plain)), Some("diaryStaysPut"));
    assert_eq!(hang(shelf.ideas_page, None), Some("ideasStayPut"));
    assert_eq!(hang(shelf.diary_page, None), Some("diaryStaysPut"));
}

#[test]
fn nothing_is_hung_under_ideas_or_a_diary_book() {
    let shelf = shelf();
    let hang = |page, book| code(may_hang(&shelf.session, page, book));

    assert_eq!(
        hang(shelf.plain, Some(shelf.ideas)),
        Some("ideasHoldNoPages")
    );
    assert_eq!(
        hang(shelf.plain, Some(shelf.diary)),
        Some("diaryHoldsOnlyDays")
    );
}

#[test]
fn plain_and_agents_documents_hang_and_let_go_freely() {
    let shelf = shelf();

    assert_eq!(
        code(may_hang(
            &shelf.session,
            shelf.plain,
            Some(shelf.agents_doc)
        )),
        None
    );
    assert_eq!(
        code(may_hang(
            &shelf.session,
            shelf.agents_doc,
            Some(shelf.plain)
        )),
        None
    );
    assert_eq!(code(may_hang(&shelf.session, shelf.plain, None)), None);
}

#[test]
fn hanging_a_document_that_is_not_there_says_so() {
    let shelf = shelf();

    assert_eq!(
        code(may_hang(&shelf.session, Ulid::generate(), None)),
        Some("noSuchDoc")
    );
    assert_eq!(
        code(may_file(&shelf.session, Ulid::generate())),
        Some("noSuchDoc")
    );
}

#[test]
fn ideas_and_the_diary_do_not_move_into_a_folder_and_neither_do_their_pages() {
    let shelf = shelf();
    let file = |doc| code(may_file(&shelf.session, doc));

    assert_eq!(file(shelf.ideas), Some("ideasStayPut"));
    assert_eq!(file(shelf.ideas_page), Some("ideasStayPut"));
    assert_eq!(file(shelf.diary), Some("diaryStaysPut"));
    assert_eq!(file(shelf.diary_page), Some("diaryStaysPut"));
    assert_eq!(file(shelf.plain), None);
    assert_eq!(file(shelf.agents_doc), None);
}

#[test]
fn inside_agents_only_a_doc_is_made() {
    let shelf = shelf();
    let make = |what| code(may_make(&shelf.session, Some(shelf.agents), None, what));

    assert_eq!(make(Making::Doc), None);
    for what in [Making::Folder, Making::Diagram, Making::Import] {
        assert_eq!(make(what), Some("agentsHoldOnlyRules"), "{what:?}");
    }
    assert_eq!(
        code(may_make(
            &shelf.session,
            Some(shelf.work),
            None,
            Making::Folder
        )),
        None
    );
}

#[test]
fn a_page_is_made_only_in_a_book_that_takes_pages() {
    let shelf = shelf();
    let page_of = |book| code(may_make(&shelf.session, None, Some(book), Making::Doc));

    assert_eq!(page_of(shelf.ideas), Some("ideasHoldNoPages"));
    assert_eq!(page_of(shelf.diary), Some("diaryHoldsOnlyDays"));
    assert_eq!(page_of(shelf.plain), None);
}
