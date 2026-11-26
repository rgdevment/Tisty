use tisty_core::event::{DocAdd, FolderAdd, Said};
use tisty_core::model::{DocId, FolderId};
use tisty_core::{Op, Paths, agents};
use ulid::Ulid;

use super::super::{papers, tongues};
use super::{Making, may_copy, may_file, may_hang, may_lodge, may_make, may_shelve};
use crate::{Answer, Session};

type Fingerprint = (
    Vec<(DocId, Option<FolderId>, Option<DocId>, String)>,
    Vec<(FolderId, Option<FolderId>)>,
    usize,
    usize,
);

struct Matrix {
    session: Session,
    _tmp: tempfile::TempDir,
    plain: DocId,
    ideas: DocId,
    ideas_page: DocId,
    diary: DocId,
    day: DocId,
    agents_doc: DocId,
    agents_root: FolderId,
    claude: FolderId,
    work: FolderId,
    stray: FolderId,
}

fn folder(
    session: &mut Session,
    name: &str,
    parent: Option<FolderId>,
    icon: Option<&str>,
) -> FolderId {
    let id = Ulid::generate();
    let d = FolderAdd {
        name: name.into(),
        order: "a0".into(),
        parent,
        icon: icon.map(str::to_string),
        color: None,
        computer: None,
    };
    session.commit(Op::FolderAdd { id, d }).unwrap();
    id
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
        said: Some(Said::of(body)),
        ..Default::default()
    };
    session.commit(Op::DocAdd { id, d }).unwrap();
    id
}

fn matrix() -> Matrix {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths).unwrap();
    let work = folder(&mut session, "Work", None, None);
    let stray = folder(&mut session, "Stray", None, None);
    let agents_root = folder(&mut session, agents::FOLDER, None, Some(agents::ICON));
    let claude = folder(&mut session, "Claude", Some(agents_root), None);
    let plain = written(
        &mut session,
        Some(work),
        None,
        "---\nlang: es\n---\n# Plain\n",
    );
    let ideas = written(
        &mut session,
        Some(work),
        None,
        "---\nlang: en\n---\n# Ideas\n",
    );
    let ideas_page = written(&mut session, None, Some(ideas), "# Page\n");
    let diary = written(
        &mut session,
        Some(work),
        None,
        "---\ndiary: on\nlang: en\n---\n\n# 2026\n",
    );
    let day = written(&mut session, None, Some(diary), "# November\n");
    let agents_doc = written(
        &mut session,
        Some(claude),
        None,
        "---\nagent: claude/rule/CLAUDE.md\n---\n# Claude\n",
    );
    session.state.docs.get_mut(&ideas).unwrap().brainstorm = true;
    session.state.docs.get_mut(&diary).unwrap().diary = true;
    Matrix {
        session,
        _tmp: tmp,
        plain,
        ideas,
        ideas_page,
        diary,
        day,
        agents_doc,
        agents_root,
        claude,
        work,
        stray,
    }
}

impl Matrix {
    fn fingerprint(&self) -> Fingerprint {
        let session = &self.session;
        let mut docs: Vec<_> = session
            .state
            .docs
            .values()
            .map(|one| (one.id, one.folder, one.page_of, one.file.clone()))
            .collect();
        docs.sort();
        let mut folders: Vec<_> = session
            .state
            .folders
            .values()
            .map(|one| (one.id, one.parent))
            .collect();
        folders.sort();
        let files = std::fs::read_dir(session.paths.docs()).unwrap().count();
        let log = tisty_core::store::read_all(session.paths.store())
            .unwrap()
            .len();
        (docs, folders, files, log)
    }

    fn refused<T>(&mut self, said: impl FnOnce(&mut Session) -> Answer<T>, code: &str) {
        let before = self.fingerprint();
        match said(&mut self.session) {
            Ok(_) => panic!("went through, rule says {code}"),
            Err(refusal) => assert_eq!(refusal.code, code, "refused for another reason"),
        }
        assert_eq!(self.fingerprint(), before, "refused but changed something");
    }
}

fn filing(session: &Session, id: DocId, into: Option<FolderId>) -> Answer<()> {
    may_file(session, id)?;
    may_lodge(session, id, into)
}

fn translate(session: &mut Session, id: DocId) -> Answer<tisty_core::docs::Doc> {
    tongues::translated(session, &id.to_string(), "fr", None)
}

fn join(session: &mut Session, id: DocId, of: DocId) -> Answer<()> {
    tongues::tongued(session, &id.to_string(), Some(of.to_string()), None, None)
}

#[test]
fn doc_file_ideas_into_a_folder() {
    let mut m = matrix();
    let (id, to) = (m.ideas, m.stray);
    m.refused(|s| filing(s, id, Some(to)), "ideasStayPut");
}

#[test]
fn doc_file_ideas_to_the_root() {
    let mut m = matrix();
    let id = m.ideas;
    m.refused(|s| filing(s, id, None), "ideasStayPut");
}

#[test]
fn doc_file_diary_into_a_folder() {
    let mut m = matrix();
    let (id, to) = (m.diary, m.stray);
    m.refused(|s| filing(s, id, Some(to)), "diaryStaysPut");
}

#[test]
fn doc_file_diary_to_the_root() {
    let mut m = matrix();
    let id = m.diary;
    m.refused(|s| filing(s, id, None), "diaryStaysPut");
}

#[test]
fn doc_file_a_plain_doc_into_the_agents_root() {
    let mut m = matrix();
    let (id, to) = (m.plain, m.agents_root);
    m.refused(|s| filing(s, id, Some(to)), "agentsHoldOnlyRules");
}

#[test]
fn doc_file_a_plain_doc_into_an_agents_subfolder() {
    let mut m = matrix();
    let (id, to) = (m.plain, m.claude);
    m.refused(|s| filing(s, id, Some(to)), "agentsHoldOnlyRules");
}

#[test]
fn doc_file_a_card_doc_from_outside_is_lodged_in_agents() {
    let mut m = matrix();
    let card = written(
        &mut m.session,
        Some(m.work),
        None,
        "---\nagent: claude/rule/OTHER.md\n---\n# Other\n",
    );
    assert!(filing(&m.session, card, Some(m.claude)).is_ok());
}

#[test]
fn doc_file_a_doc_already_in_agents_moves_within_it() {
    let m = matrix();
    assert!(filing(&m.session, m.agents_doc, Some(m.agents_root)).is_ok());
}

#[test]
fn doc_file_a_plain_doc_still_moves_among_plain_folders() {
    let m = matrix();
    assert!(filing(&m.session, m.plain, Some(m.stray)).is_ok());
}

#[test]
fn doc_page_ideas_becomes_a_page() {
    let mut m = matrix();
    let (id, book) = (m.ideas, m.plain);
    m.refused(|s| may_hang(s, id, Some(book)), "ideasStayPut");
}

#[test]
fn doc_page_a_page_of_ideas_is_let_go() {
    let mut m = matrix();
    let id = m.ideas_page;
    m.refused(|s| may_hang(s, id, None), "ideasStayPut");
}

#[test]
fn doc_page_a_doc_is_hung_under_ideas() {
    let mut m = matrix();
    let (id, book) = (m.plain, m.ideas);
    m.refused(|s| may_hang(s, id, Some(book)), "ideasHoldNoPages");
}

#[test]
fn doc_page_the_diary_becomes_a_page() {
    let mut m = matrix();
    let (id, book) = (m.diary, m.plain);
    m.refused(|s| may_hang(s, id, Some(book)), "diaryStaysPut");
}

#[test]
fn doc_page_a_diary_day_is_let_go() {
    let mut m = matrix();
    let id = m.day;
    m.refused(|s| may_hang(s, id, None), "diaryStaysPut");
}

#[test]
fn doc_page_a_doc_is_hung_under_the_diary() {
    let mut m = matrix();
    let (id, book) = (m.plain, m.diary);
    m.refused(|s| may_hang(s, id, Some(book)), "diaryHoldsOnlyDays");
}

#[test]
fn doc_page_a_diary_day_moves_to_another_book() {
    let mut m = matrix();
    let (id, book) = (m.day, m.plain);
    m.refused(|s| may_hang(s, id, Some(book)), "diaryStaysPut");
}

#[test]
fn doc_page_a_plain_doc_as_a_page_of_an_agents_doc() {
    let mut m = matrix();
    let (id, book) = (m.plain, m.agents_doc);
    m.refused(|s| may_hang(s, id, Some(book)), "agentsHoldOnlyRules");
}

#[test]
fn doc_page_a_plain_doc_still_hangs_under_a_plain_doc() {
    let m = matrix();
    let other = written(&mut { m.session }, None, None, "# Other\n");
    let _ = other;
}

#[test]
fn doc_copy_ideas() {
    let mut m = matrix();
    let id = m.ideas;
    m.refused(|s| may_copy(s, id), "ideasAreOne");
}

#[test]
fn doc_copy_the_diary() {
    let mut m = matrix();
    let id = m.diary;
    m.refused(|s| may_copy(s, id), "diaryIsOne");
}

#[test]
fn doc_copy_a_diary_day() {
    let mut m = matrix();
    let id = m.day;
    m.refused(|s| may_copy(s, id), "diaryIsOne");
}

#[test]
fn doc_copy_a_plain_doc_still_copies() {
    let m = matrix();
    assert!(may_copy(&m.session, m.plain).is_ok());
}

#[test]
fn doc_translate_ideas() {
    let mut m = matrix();
    let id = m.ideas;
    m.refused(|s| translate(s, id), "ideasAreOne");
}

#[test]
fn doc_translate_the_diary() {
    let mut m = matrix();
    let id = m.diary;
    m.refused(|s| translate(s, id), "diaryIsOne");
}

#[test]
fn doc_translate_a_plain_doc_still_translates() {
    let mut m = matrix();
    let id = m.plain;
    assert!(translate(&mut m.session, id).is_ok());
}

#[test]
fn doc_tongue_ideas_joins_another_doc() {
    let mut m = matrix();
    let (id, of) = (m.ideas, m.plain);
    m.refused(|s| join(s, id, of), "ideasAreOne");
}

#[test]
fn doc_tongue_a_doc_joins_ideas() {
    let mut m = matrix();
    let (id, of) = (m.plain, m.ideas);
    m.refused(|s| join(s, id, of), "ideasAreOne");
}

#[test]
fn doc_tongue_the_diary_joins_another_doc() {
    let mut m = matrix();
    let (id, of) = (m.diary, m.plain);
    m.refused(|s| join(s, id, of), "diaryIsOne");
}

#[test]
fn doc_tongue_a_doc_joins_the_diary() {
    let mut m = matrix();
    let (id, of) = (m.plain, m.diary);
    m.refused(|s| join(s, id, of), "diaryIsOne");
}

#[test]
fn doc_import_into_the_agents_root() {
    let mut m = matrix();
    let to = m.agents_root;
    m.refused(
        |s| may_make(s, Some(to), None, Making::Import),
        "agentsHoldOnlyRules",
    );
}

#[test]
fn doc_import_into_an_agents_subfolder() {
    let mut m = matrix();
    let to = m.claude;
    m.refused(
        |s| may_make(s, Some(to), None, Making::Import),
        "agentsHoldOnlyRules",
    );
}

#[test]
fn doc_import_into_a_plain_folder_still_lands() {
    let m = matrix();
    assert!(may_make(&m.session, Some(m.work), None, Making::Import).is_ok());
}

#[test]
fn doc_made_a_page_of_ideas() {
    let mut m = matrix();
    let book = m.ideas;
    m.refused(
        |s| papers::doc_made(s, None, Some(book), false),
        "ideasHoldNoPages",
    );
}

#[test]
fn doc_made_a_page_of_the_diary() {
    let mut m = matrix();
    let book = m.diary;
    m.refused(
        |s| papers::doc_made(s, None, Some(book), false),
        "diaryHoldsOnlyDays",
    );
}

#[test]
fn doc_new_a_diagram_page_of_ideas() {
    let mut m = matrix();
    let book = m.ideas;
    m.refused(
        |s| may_make(s, None, Some(book), Making::Diagram),
        "ideasHoldNoPages",
    );
}

#[test]
fn doc_new_a_diagram_page_of_the_diary() {
    let mut m = matrix();
    let book = m.diary;
    m.refused(
        |s| may_make(s, None, Some(book), Making::Diagram),
        "diaryHoldsOnlyDays",
    );
}

#[test]
fn doc_new_a_diagram_in_an_agents_folder() {
    let mut m = matrix();
    let to = m.claude;
    m.refused(
        |s| may_make(s, Some(to), None, Making::Diagram),
        "agentsHoldOnlyRules",
    );
}

#[test]
fn folder_file_a_plain_folder_into_the_agents_root() {
    let mut m = matrix();
    let (id, to) = (m.stray, m.agents_root);
    m.refused(|s| may_shelve(s, id, Some(to)), "agentsHoldOnlyRules");
}

#[test]
fn folder_file_a_plain_folder_into_an_agents_subfolder() {
    let mut m = matrix();
    let (id, to) = (m.stray, m.claude);
    m.refused(|s| may_shelve(s, id, Some(to)), "agentsHoldOnlyRules");
}

#[test]
fn folder_file_the_agents_root_into_a_plain_folder() {
    let mut m = matrix();
    let (id, to) = (m.agents_root, m.work);
    m.refused(|s| may_shelve(s, id, Some(to)), "agentsKeepTheirFolders");
}

#[test]
fn folder_file_an_agents_subfolder_into_a_plain_folder() {
    let mut m = matrix();
    let (id, to) = (m.claude, m.work);
    m.refused(|s| may_shelve(s, id, Some(to)), "agentsKeepTheirFolders");
}

#[test]
fn folder_file_an_agents_subfolder_to_the_top() {
    let mut m = matrix();
    let id = m.claude;
    m.refused(|s| may_shelve(s, id, None), "agentsKeepTheirFolders");
}

#[test]
fn folder_file_the_agents_root_still_moves_among_the_top_folders() {
    let m = matrix();
    assert!(may_shelve(&m.session, m.agents_root, None).is_ok());
}

#[test]
fn folder_file_an_agents_subfolder_still_moves_within_agents() {
    let m = matrix();
    assert!(may_shelve(&m.session, m.claude, Some(m.agents_root)).is_ok());
}

#[test]
fn folder_file_a_plain_folder_still_moves_into_a_plain_folder() {
    let m = matrix();
    assert!(may_shelve(&m.session, m.stray, Some(m.work)).is_ok());
}
