use super::*;
use tisty_core::Paths;
use tisty_core::event::{DocAdd, FolderAdd};
use tisty_core::model::DEEPEST;
use ulid::Ulid;

struct Shelf {
    _tmp: tempfile::TempDir,
    session: Session,
}

fn shelf() -> Shelf {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let session = Session::at(paths).unwrap();
    Shelf { _tmp: tmp, session }
}

impl Shelf {
    fn folder(&mut self, name: &str, parent: Option<FolderId>) -> FolderId {
        let id = Ulid::generate();
        let d = FolderAdd {
            name: name.into(),
            order: "a0".into(),
            parent,
            icon: None,
            color: None,
            computer: None,
        };
        self.session.commit(Op::FolderAdd { id, d }).unwrap();
        id
    }

    fn doc(&mut self, in_folder: Option<FolderId>) -> DocId {
        let device = self.session.store.device().clone();
        let made =
            tisty_core::docs::create(&self.session.paths.docs(), &device, "# Note\n").unwrap();
        let id = Ulid::generate();
        let d = DocAdd {
            file: made.id,
            folder: in_folder,
            order: "a0".into(),
            ..Default::default()
        };
        self.session.commit(Op::DocAdd { id, d }).unwrap();
        id
    }

    fn said(&mut self, in_folder: Option<FolderId>, body: &str) -> DocId {
        let device = self.session.store.device().clone();
        let made = tisty_core::docs::create(&self.session.paths.docs(), &device, body).unwrap();
        let id = Ulid::generate();
        let d = DocAdd {
            file: made.id,
            folder: in_folder,
            order: "a0".into(),
            said: Some(tisty_core::event::Said::of(body)),
            ..Default::default()
        };
        self.session.commit(Op::DocAdd { id, d }).unwrap();
        id
    }

    fn lock(&mut self, id: DocId) {
        self.session.commit(Op::DocLock { id }).unwrap();
    }

    fn parent_of(&self, id: FolderId) -> Option<FolderId> {
        self.session
            .state
            .folders
            .get(&id)
            .and_then(|one| one.parent)
    }

    fn folder_of(&self, id: DocId) -> Option<FolderId> {
        self.session.state.docs.get(&id).and_then(|one| one.folder)
    }

    fn has_folder(&self, id: FolderId) -> bool {
        self.session.state.folders.contains_key(&id)
    }

    fn has_doc(&self, id: DocId) -> bool {
        self.session.state.docs.contains_key(&id)
    }

    fn emptied(&mut self, id: FolderId, contents: Contents) -> Answer<()> {
        emptied(&mut self.session, id, contents)
    }
}

fn nested() -> (Shelf, FolderId, FolderId, FolderId, Vec<DocId>) {
    let mut shelf = shelf();
    let a = shelf.folder("A", None);
    let b = shelf.folder("B", Some(a));
    let c = shelf.folder("C", Some(b));
    let docs = vec![shelf.doc(Some(b)), shelf.doc(Some(b))];
    (shelf, a, b, c, docs)
}

#[test]
fn up_hands_subfolders_and_documents_to_the_parent_not_the_root() {
    let (mut shelf, a, b, c, docs) = nested();

    shelf.emptied(b, Contents::Up).unwrap();

    assert_eq!(shelf.parent_of(c), Some(a));
    for doc in docs {
        assert_eq!(shelf.folder_of(doc), Some(a));
    }
}

#[test]
fn up_from_a_top_folder_hands_everything_to_the_root() {
    let (mut shelf, a, b, _, _) = nested();
    let inside = shelf.doc(Some(a));

    shelf.emptied(a, Contents::Up).unwrap();

    assert_eq!(shelf.parent_of(b), None);
    assert_eq!(shelf.folder_of(inside), None);
}

#[test]
fn into_hands_them_to_the_folder_asked() {
    let (mut shelf, _, b, c, docs) = nested();
    let x = shelf.folder("X", None);

    shelf.emptied(b, Contents::Into(Some(x))).unwrap();

    assert_eq!(shelf.parent_of(c), Some(x));
    for doc in docs {
        assert_eq!(shelf.folder_of(doc), Some(x));
    }
}

#[test]
fn into_the_folders_own_child_is_refused_and_moves_nothing() {
    let (mut shelf, _, b, c, docs) = nested();

    let refused = shelf.emptied(b, Contents::Into(Some(c))).unwrap_err();

    assert_eq!(refused.code, "intoItself");
    assert_eq!(shelf.parent_of(c), Some(b));
    assert_eq!(shelf.folder_of(docs[0]), Some(b));
}

#[test]
fn into_itself_is_refused() {
    let (mut shelf, _, b, _, _) = nested();

    let refused = shelf.emptied(b, Contents::Into(Some(b))).unwrap_err();

    assert_eq!(refused.code, "intoItself");
}

#[test]
fn into_a_folder_that_would_go_past_the_depth_limit_is_refused() {
    let (mut shelf, _, b, c, docs) = nested();
    let mut deepest = shelf.folder("X1", None);
    for step in 2..=DEEPEST {
        deepest = shelf.folder(&format!("X{step}"), Some(deepest));
    }

    let refused = shelf.emptied(b, Contents::Into(Some(deepest))).unwrap_err();

    assert_eq!(refused.code, "tooDeep");
    assert_eq!(shelf.parent_of(c), Some(b));
    assert_eq!(shelf.folder_of(docs[0]), Some(b));
}

#[test]
fn into_a_folder_that_does_not_exist_is_refused() {
    let (mut shelf, _, b, _, _) = nested();

    let refused = shelf
        .emptied(b, Contents::Into(Some(Ulid::generate())))
        .unwrap_err();

    assert_eq!(refused.code, "noSuchFolder");
}

#[test]
fn drop_deletes_every_document_and_subfolder_of_the_tree() {
    let (mut shelf, a, b, c, docs) = nested();
    let deep = shelf.doc(Some(c));
    let outside = shelf.doc(Some(a));

    shelf.emptied(b, Contents::Dropped).unwrap();

    assert!(!shelf.has_folder(c));
    for doc in docs.into_iter().chain([deep]) {
        assert!(!shelf.has_doc(doc));
    }
    assert!(shelf.has_doc(outside));
    assert!(
        shelf.has_folder(b),
        "the folder itself goes with the command"
    );
}

#[test]
fn one_locked_document_deep_in_the_tree_deletes_nothing_at_all() {
    let (mut shelf, _, b, c, docs) = nested();
    let deep = shelf.doc(Some(c));
    shelf.lock(deep);

    let refused = shelf.emptied(b, Contents::Dropped).unwrap_err();

    assert_eq!(refused.code, "documentLocked");
    assert!(shelf.has_folder(c));
    for doc in docs.into_iter().chain([deep]) {
        assert!(shelf.has_doc(doc));
    }
}

#[test]
fn the_word_asked_picks_the_way_and_defaults_to_up() {
    let at = Some(Ulid::generate());

    assert!(Contents::asked(None, at).unwrap() == Contents::Up);
    assert!(Contents::asked(Some("into"), at).unwrap() == Contents::Into(at));
    assert!(Contents::asked(Some("drop"), at).unwrap() == Contents::Dropped);
    assert!(Contents::asked(Some("else"), at).is_err());
}

#[test]
fn a_folder_of_agents_that_holds_files_is_not_deleted_from_here() {
    let mut shelf = shelf();
    let root = Ulid::generate();
    let d = FolderAdd {
        name: tisty_core::agents::FOLDER.into(),
        order: "a0".into(),
        parent: None,
        icon: Some(tisty_core::agents::ICON.into()),
        color: None,
        computer: None,
    };
    shelf.session.commit(Op::FolderAdd { id: root, d }).unwrap();
    let claude = shelf.folder("Claude", Some(root));
    let rule = shelf.doc(Some(claude));

    for contents in [Contents::Up, Contents::Into(None), Contents::Dropped] {
        let refused = shelf.emptied(claude, contents).unwrap_err();
        assert_eq!(refused.code, "agentsFolderHolds");
    }
    assert_eq!(shelf.folder_of(rule), Some(claude));
    assert!(shelf.has_doc(rule));
}

#[test]
fn dropping_a_folder_deletes_a_document_and_its_other_language_once() {
    let mut shelf = shelf();
    let folder = shelf.folder("Guías", None);
    let spanish = shelf.said(
        Some(folder),
        "---
lang: es
---
# Guía
",
    );
    let english = shelf.said(
        Some(folder),
        "---
lang: en
---
# Guide
",
    );
    let d = tisty_core::event::Tongue { of: Some(spanish) };
    shelf
        .session
        .commit(Op::DocTongue { id: english, d })
        .unwrap();
    assert!(!shelf.session.state.kin(spanish).is_empty());

    shelf.emptied(folder, Contents::Dropped).unwrap();

    assert!(!shelf.has_doc(spanish));
    assert!(!shelf.has_doc(english));
}
