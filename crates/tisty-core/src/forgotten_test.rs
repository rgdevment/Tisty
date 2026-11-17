use super::*;
use crate::event::{DocAdd, Event, Flag, Op, Said, Tongue};
use crate::{DeviceId, State};

const NOW: u64 = 1_800_000_000;

struct Shelf {
    room: tempfile::TempDir,
    state: State,
    opened: Used,
    files: std::collections::BTreeMap<String, String>,
    clock: i64,
}

impl Shelf {
    fn new() -> Self {
        Self {
            room: tempfile::tempdir().unwrap(),
            state: State::default(),
            opened: Used::new(),
            files: std::collections::BTreeMap::new(),
            clock: 1_000,
        }
    }

    fn root(&self) -> &Path {
        self.room.path()
    }

    fn apply(&mut self, op: Op) {
        self.clock += 1;
        let at = jiff::Timestamp::from_millisecond(self.clock).unwrap();
        self.state
            .apply(&Event::new(DeviceId("dev_here".into()), at, op));
    }

    fn put(&mut self, title: &str, days_ago: u64, shape: impl FnOnce(&mut DocAdd)) -> DocId {
        self.put_with(title, days_ago, "", shape)
    }

    fn put_with(
        &mut self,
        title: &str,
        days_ago: u64,
        front: &str,
        shape: impl FnOnce(&mut DocAdd),
    ) -> DocId {
        let file = format!("hevp3wf9-{:04}", self.files.len() + 1);
        self.files.insert(title.into(), file.clone());
        std::fs::write(
            self.root().join(format!("{file}.md")),
            format!("{front}# {title}\n"),
        )
        .unwrap();
        let id = ulid::Ulid::generate();
        let wrote = i64::try_from(NOW - days_ago * DAY).unwrap();
        let mut add = DocAdd {
            file: file.clone(),
            order: file,
            wrote: Some(jiff::Timestamp::from_second(wrote).unwrap()),
            said: Some(Said {
                title: title.into(),
                ..Said::default()
            }),
            ..DocAdd::default()
        };
        shape(&mut add);
        self.apply(Op::DocAdd { id, d: add });
        id
    }

    fn file(&self, title: &str) -> &str {
        &self.files[title]
    }

    fn opened_days_ago(&mut self, title: &str, days: u64) {
        let file = self.file(title).to_string();
        self.opened.insert(file, NOW - days * DAY);
    }

    fn forgotten(&self) -> Vec<Forgotten> {
        forgotten(self.root(), &self.state, &self.opened, NOW)
    }

    fn docs_forgotten(&self) -> Vec<DocId> {
        self.forgotten().into_iter().map(|one| one.doc).collect()
    }
}

#[test]
fn a_document_nobody_wrote_or_opened_for_sixty_days_is_forgotten_and_one_of_fifty_nine_is_not() {
    let mut shelf = Shelf::new();
    let old = shelf.put("viejo", 60, |_| {});
    shelf.put("reciente", 59, |_| {});

    assert_eq!(shelf.docs_forgotten(), vec![old]);
}

#[test]
fn opening_it_here_brings_it_back_to_mind_however_long_ago_it_was_written() {
    let mut shelf = Shelf::new();
    shelf.put("recetas", 400, |_| {});
    shelf.opened_days_ago("recetas", 3);

    assert!(shelf.forgotten().is_empty());
}

#[test]
fn it_says_whether_its_last_day_was_an_opening_here_or_a_write() {
    let mut shelf = Shelf::new();
    let opened = shelf.put("abierto", 400, |_| {});
    shelf.opened_days_ago("abierto", 90);
    let written = shelf.put("escrito", 120, |_| {});

    let found = shelf.forgotten();

    assert_eq!(
        found,
        vec![
            Forgotten {
                doc: written,
                last: NOW - 120 * DAY,
                last_opened_here: false,
            },
            Forgotten {
                doc: opened,
                last: NOW - 90 * DAY,
                last_opened_here: true,
            },
        ],
        "the longest out of mind comes first"
    );
}

#[test]
fn what_is_put_away_marked_missing_sown_or_a_sheet_of_ideas_or_a_diary_never_counts() {
    let mut shelf = Shelf::new();
    let kept = shelf.put("guardado", 100, |_| {});
    let archived = shelf.put("archivado", 100, |_| {});
    shelf.apply(Op::DocArchive { id: archived });
    let marked = shelf.put("marcado", 100, |_| {});
    shelf.apply(Op::DocFlag {
        id: marked,
        d: Flag {
            body: "Una copia de otro".into(),
            at: None,
            by: None,
            via: None,
            reason: None,
            instead: None,
        },
    });
    shelf.put("sembrado", 100, |add| {
        add.sown = true;
        add.guest = true;
        add.by = Some(crate::untouched::SOWN_BY.into());
        add.made = None;
        add.wrote = None;
    });
    shelf.put_with("ideas", 100, "---\nbrainstorm: on\n---\n", |add| {
        if let Some(said) = add.said.as_mut() {
            said.brainstorm = true;
        }
    });
    shelf.put_with("diario", 100, "---\ndiary: on\n---\n", |add| {
        if let Some(said) = add.said.as_mut() {
            said.diary = true;
        }
    });
    let missing = shelf.put("perdido", 100, |_| {});
    let gone = format!("{}.md", shelf.file("perdido"));
    std::fs::remove_file(shelf.root().join(gone)).unwrap();

    let found = shelf.docs_forgotten();

    let titles: Vec<_> = found
        .iter()
        .map(|id| shelf.state.docs[id].title.clone())
        .collect();
    assert_eq!(
        found,
        vec![kept],
        "only the one kept in the open counts: {titles:?}"
    );
    assert!(!found.contains(&missing));
}

#[test]
fn a_page_opened_lately_keeps_its_document_in_mind_and_is_never_listed_itself() {
    let mut shelf = Shelf::new();
    let book = shelf.put("libro", 200, |_| {});
    shelf.put("capitulo", 200, |add| add.page_of = Some(book));
    let other = shelf.put("otro", 200, |_| {});
    shelf.put("hoja", 200, |add| add.page_of = Some(other));
    shelf.opened_days_ago("capitulo", 2);

    assert_eq!(shelf.docs_forgotten(), vec![other]);
}

#[test]
fn every_language_of_a_document_is_one_and_opening_any_of_them_counts_for_all() {
    let mut shelf = Shelf::new();
    let anchor = shelf.put("receta", 200, |_| {});
    let english = shelf.put("recipe", 200, |_| {});
    shelf.apply(Op::DocTongue {
        id: english,
        d: Tongue { of: Some(anchor) },
    });

    assert_eq!(
        shelf.docs_forgotten(),
        vec![anchor],
        "listed once, by its anchor"
    );

    shelf.opened_days_ago("recipe", 1);

    assert!(shelf.forgotten().is_empty());
}

#[test]
fn opening_a_language_put_away_still_counts_for_the_one_in_the_open() {
    let mut shelf = Shelf::new();
    let anchor = shelf.put("receta", 200, |_| {});
    let english = shelf.put("recipe", 200, |_| {});
    shelf.apply(Op::DocTongue {
        id: english,
        d: Tongue { of: Some(anchor) },
    });
    shelf.apply(Op::DocArchive { id: english });

    assert_eq!(shelf.docs_forgotten(), vec![anchor]);

    shelf.opened_days_ago("recipe", 1);

    assert!(shelf.forgotten().is_empty());
}

#[test]
fn the_dice_leaves_out_what_was_opened_this_week_unless_that_is_everything() {
    let mut shelf = Shelf::new();
    let rested = shelf.put("descansado", 3, |_| {});
    shelf.put("fresco", 3, |_| {});
    shelf.opened_days_ago("fresco", 1);

    for roll in 0..8 {
        assert_eq!(
            at_random(shelf.root(), &shelf.state, &shelf.opened, NOW, roll),
            Some(rested)
        );
    }

    shelf.opened_days_ago("descansado", 2);
    let picked: std::collections::BTreeSet<DocId> = (0..8)
        .filter_map(|roll| at_random(shelf.root(), &shelf.state, &shelf.opened, NOW, roll))
        .collect();
    assert_eq!(picked.len(), 2, "with all opened lately, any of them");
}

#[test]
fn the_dice_has_nothing_to_roll_on_an_empty_library() {
    let shelf = Shelf::new();

    assert_eq!(
        at_random(shelf.root(), &shelf.state, &shelf.opened, NOW, 7),
        None
    );
}

#[test]
fn an_opening_is_written_down_here_under_the_document_s_own_name() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("cache").join("opened.json");
    let looks_like_an_attachment = "nota-deadbeef";

    opened(&at, "hevp3wf9-0371");
    opened(&at, looks_like_an_attachment);

    let held = opened_here(&at);
    assert!(held.contains_key("hevp3wf9-0371"));
    assert!(
        held.contains_key(looks_like_an_attachment),
        "a name is never read as an attachment's: {held:?}"
    );
}
