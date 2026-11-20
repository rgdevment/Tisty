use super::*;
use crate::DeviceId;
use crate::agents::{FOLDER, ICON, holds, holds_folder, root_of, roots, tree};
use crate::event::{DocAdd, Event, FolderAdd, Said};
use crate::model::DocId;
use ulid::Ulid;

const DESK: &str = "dev_desk";
const LAPTOP: &str = "dev_laptop";
const AGENT: &str = "dev_agent";

struct Log {
    events: Vec<Event>,
    ms: i64,
}

impl Log {
    fn new() -> Self {
        let mut log = Self {
            events: Vec::new(),
            ms: 0,
        };
        log.push(
            AGENT,
            Op::DeviceJoin {
                d: DeviceId(AGENT.into()),
                k: Some(crate::event::DeviceKind::Agent),
                p: None,
            },
        );
        log
    }

    fn push(&mut self, who: &str, op: Op) {
        self.ms += 1;
        let at = jiff::Timestamp::from_millisecond(self.ms).unwrap();
        self.events.push(Event::new(DeviceId(who.into()), at, op));
    }

    fn pushed(&mut self, who: &str, ops: Vec<Op>) {
        for op in ops {
            self.push(who, op);
        }
    }

    fn folder(&mut self, who: &str, name: &str, parent: Option<FolderId>) -> FolderId {
        let id = Ulid::generate();
        let robot = parent.is_none() && name == FOLDER;
        self.push(
            who,
            Op::FolderAdd {
                id,
                d: FolderAdd {
                    name: name.into(),
                    order: "a0".into(),
                    parent,
                    icon: robot.then(|| ICON.to_string()),
                    color: None,
                },
            },
        );
        id
    }

    // Tisty names a folder of AGENTS the same way on every computer.
    fn sub(&mut self, who: &str, name: &str, root: FolderId) -> FolderId {
        let id = crate::agents::sub_id(root, name);
        self.push(
            who,
            Op::FolderAdd {
                id,
                d: FolderAdd {
                    name: name.into(),
                    order: "a0".into(),
                    parent: Some(root),
                    icon: None,
                    color: None,
                },
            },
        );
        id
    }

    fn doc(&mut self, who: &str, file: &str, folder: Option<FolderId>) -> DocId {
        let id = Ulid::generate();
        self.push(
            who,
            Op::DocAdd {
                id,
                d: DocAdd {
                    file: file.into(),
                    order: "a0".into(),
                    said: Some(Said::of(file).agreed()),
                    folder,
                    ..Default::default()
                },
            },
        );
        id
    }

    fn page(&mut self, who: &str, file: &str, of: DocId) -> DocId {
        let id = Ulid::generate();
        self.push(
            who,
            Op::DocAdd {
                id,
                d: DocAdd {
                    file: file.into(),
                    order: "a0".into(),
                    page_of: Some(of),
                    ..Default::default()
                },
            },
        );
        id
    }

    fn state(&self) -> State {
        let mut events = self.events.clone();
        events.sort_by_key(|one| one.timestamp);
        let mut state = State::default();
        for event in &events {
            state.apply(event);
        }
        state
    }
}

// Each computer made its own AGENTS before they synced: the desk's has the smaller id, so it stays.
struct Two {
    log: Log,
    desk: FolderId,
    desk_global: FolderId,
    laptop: FolderId,
    laptop_global: FolderId,
}

fn two() -> Two {
    let mut log = Log::new();
    let desk = log.folder(DESK, FOLDER, None);
    let desk_global = log.sub(DESK, "Global", desk);
    log.doc(DESK, "desk-global", Some(desk_global));
    std::thread::sleep(std::time::Duration::from_millis(2));
    let laptop = log.folder(LAPTOP, FOLDER, None);
    let laptop_global = log.sub(LAPTOP, "Global", laptop);
    log.doc(LAPTOP, "laptop-global", Some(laptop_global));
    assert!(desk < laptop);
    Two {
        log,
        desk,
        desk_global,
        laptop,
        laptop_global,
    }
}

fn folder_of(state: &State, file: &str) -> Option<FolderId> {
    state.docs.values().find(|one| one.file == file)?.folder
}

fn named(state: &State, parent: FolderId) -> Vec<String> {
    let mut all: Vec<String> = state
        .folders
        .values()
        .filter(|one| one.parent == Some(parent))
        .map(|one| one.name.clone())
        .collect();
    all.sort();
    all
}

#[test]
fn one_agents_stands_alone_and_nothing_is_joined() {
    let mut log = Log::new();
    let root = log.folder(DESK, FOLDER, None);
    log.folder(DESK, "Global", Some(root));
    assert!(joined(&log.state()).is_empty());
    assert!(joined(&State::default()).is_empty());
}

#[test]
fn two_agents_join_in_the_oldest_and_the_other_goes_once_empty() {
    let mut two = two();
    let claude = two.log.folder(LAPTOP, "Claude", Some(two.laptop));
    two.log.doc(LAPTOP, "laptop-claude", Some(claude));
    two.log.doc(LAPTOP, "laptop-loose", Some(two.laptop));
    let before = two.log.state();
    assert_eq!(roots(&before), [two.desk, two.laptop]);

    let ops = joined(&before);
    two.log.pushed(LAPTOP, ops);
    let after = two.log.state();

    assert_eq!(roots(&after), [two.desk]);
    assert_eq!(root_of(&after), Some(two.desk));
    assert!(!after.folders.contains_key(&two.laptop));
    assert!(!after.folders.contains_key(&two.laptop_global));
    assert_eq!(named(&after, two.desk), ["Claude", "Global"]);
    assert_eq!(folder_of(&after, "desk-global"), Some(two.desk_global));
    assert_eq!(folder_of(&after, "laptop-global"), Some(two.desk_global));
    assert_eq!(folder_of(&after, "laptop-claude"), Some(claude));
    assert_eq!(folder_of(&after, "laptop-loose"), Some(two.desk));
    assert_eq!(after.folders[&claude].parent, Some(two.desk));
    assert!(
        joined(&after).is_empty(),
        "a joined AGENTS asks for nothing more"
    );
}

#[test]
fn joining_keeps_every_approval_and_who_made_each_document() {
    let mut two = two();
    let before = two.log.state();
    let was = before
        .docs
        .values()
        .find(|one| one.file == "laptop-global")
        .unwrap()
        .clone();

    two.log.pushed(DESK, joined(&before));
    let after = two.log.state();
    let now = after.docs.get(&was.id).unwrap();

    assert_eq!(now.agreed, was.agreed);
    assert_eq!(now.agreed_by, was.agreed_by);
    assert_eq!(now.made_by, was.made_by);
    assert!(holds(&after, now));
}

#[test]
fn both_computers_joining_at_once_land_in_the_same_place_with_nothing_twice() {
    let mut two = two();
    let claude = two.log.folder(LAPTOP, "Claude", Some(two.laptop));
    two.log.doc(LAPTOP, "laptop-claude", Some(claude));
    let seen = two.log.state();
    let (desk, laptop) = (joined(&seen), joined(&seen));
    assert_eq!(
        desk, laptop,
        "both read the same log, so both plan the same"
    );

    let mut one_after_other = Log {
        events: two.log.events.clone(),
        ms: two.log.ms,
    };
    one_after_other.pushed(DESK, desk.clone());
    one_after_other.pushed(LAPTOP, laptop.clone());
    let mut woven = Log {
        events: two.log.events.clone(),
        ms: two.log.ms,
    };
    for (a, b) in desk.into_iter().zip(laptop) {
        woven.push(LAPTOP, b);
        woven.push(DESK, a);
    }

    for state in [one_after_other.state(), woven.state()] {
        assert_eq!(roots(&state), [two.desk]);
        assert_eq!(named(&state, two.desk), ["Claude", "Global"]);
        assert_eq!(folder_of(&state, "laptop-global"), Some(two.desk_global));
        assert_eq!(folder_of(&state, "laptop-claude"), Some(claude));
        assert_eq!(state.folders.len(), 3);
        assert!(joined(&state).is_empty());
    }
}

#[test]
fn folders_of_the_same_name_join_at_every_depth_and_pages_follow_their_document() {
    let mut two = two();
    let desk_claude = two.log.folder(DESK, "Claude", Some(two.desk));
    let desk_skills = two.log.folder(DESK, "Skills", Some(desk_claude));
    let laptop_claude = two.log.folder(LAPTOP, "Claude", Some(two.laptop));
    let laptop_skills = two.log.folder(LAPTOP, "Skills", Some(laptop_claude));
    let skill = two.log.doc(LAPTOP, "laptop-skill", Some(laptop_skills));
    two.log.page(LAPTOP, "laptop-skill-script", skill);

    two.log.pushed(LAPTOP, joined(&two.log.state()));
    let after = two.log.state();

    assert_eq!(named(&after, two.desk), ["Claude", "Global"]);
    assert_eq!(named(&after, desk_claude), ["Skills"]);
    assert_eq!(folder_of(&after, "laptop-skill"), Some(desk_skills));
    assert_eq!(folder_of(&after, "laptop-skill-script"), Some(desk_skills));
    assert!(!after.folders.contains_key(&laptop_claude));
    assert!(!after.folders.contains_key(&laptop_skills));
}

#[test]
fn two_folders_of_one_name_in_the_joining_agents_become_one() {
    let mut two = two();
    let first = two.log.folder(LAPTOP, "Codex", Some(two.laptop));
    let second = two.log.folder(LAPTOP, "Codex", Some(two.laptop));
    two.log.doc(LAPTOP, "codex-one", Some(first));
    two.log.doc(LAPTOP, "codex-two", Some(second));

    two.log.pushed(LAPTOP, joined(&two.log.state()));
    let after = two.log.state();

    assert_eq!(named(&after, two.desk), ["Codex", "Global"]);
    let codex = folder_of(&after, "codex-one");
    assert_eq!(folder_of(&after, "codex-two"), codex);
}

#[test]
fn an_archived_folder_moves_whole_and_what_it_holds_stays_away() {
    let mut two = two();
    let old = two.log.folder(LAPTOP, "Global", Some(two.laptop));
    two.log.doc(LAPTOP, "laptop-old", Some(old));
    two.log.push(LAPTOP, Op::FolderArchive { id: old });

    two.log.pushed(LAPTOP, joined(&two.log.state()));
    let after = two.log.state();

    assert_eq!(after.folders[&old].parent, Some(two.desk));
    assert!(after.folders[&old].archived);
    assert_eq!(folder_of(&after, "laptop-old"), Some(old));
    let kept = after
        .docs
        .values()
        .find(|one| one.file == "laptop-old")
        .unwrap();
    assert!(
        after.held_away(kept),
        "an archived rule never comes back alive"
    );
    assert_eq!(folder_of(&after, "laptop-global"), Some(two.desk_global));
}

#[test]
fn an_agents_holding_anything_an_agent_placed_is_left_as_it_is() {
    let mut two = two();
    two.log.doc(AGENT, "slipped-in", Some(two.laptop_global));
    assert!(joined(&two.log.state()).is_empty());

    let two = two_with_agent_folder();
    assert!(joined(&two.log.state()).is_empty());
}

fn two_with_agent_folder() -> Two {
    let mut two = two();
    two.log.folder(AGENT, "Gemini", Some(two.laptop));
    two
}

#[test]
fn a_document_written_into_a_folder_already_joined_is_gathered_into_its_twin() {
    let mut two = two();
    let ops = joined(&two.log.state());
    two.log.pushed(DESK, ops);
    two.log.doc(LAPTOP, "laptop-late", Some(two.laptop_global));
    let before = two.log.state();
    assert!(!holds(
        &before,
        before
            .docs
            .values()
            .find(|one| one.file == "laptop-late")
            .unwrap()
    ));

    let gathered = joined(&before);
    assert_eq!(
        gathered,
        joined(&before),
        "both computers gather it the same way"
    );
    two.log.pushed(LAPTOP, gathered);
    let after = two.log.state();
    assert_eq!(folder_of(&after, "laptop-late"), Some(two.desk_global));
    assert!(joined(&after).is_empty());
}

#[test]
fn a_document_the_join_lifted_out_of_its_folder_goes_back_into_the_twin() {
    let mut two = two();
    let ops = joined(&two.log.state());
    two.log.doc(LAPTOP, "laptop-late", Some(two.laptop_global));
    two.log.pushed(DESK, ops);
    let before = two.log.state();
    assert_eq!(folder_of(&before, "laptop-late"), None);

    two.log.pushed(DESK, joined(&before));
    let after = two.log.state();
    assert_eq!(folder_of(&after, "laptop-late"), Some(two.desk_global));
}

#[test]
fn a_document_the_person_lifted_out_by_deleting_its_folder_stays_out() {
    let mut log = Log::new();
    let root = log.folder(DESK, FOLDER, None);
    let claude = log.sub(DESK, "Claude", root);
    log.doc(DESK, "rule", Some(claude));
    log.push(DESK, Op::FolderDelete { id: claude });
    let state = log.state();
    assert_eq!(folder_of(&state, "rule"), None);
    assert!(
        joined(&state).is_empty(),
        "AGENTS is still there: the person took it out"
    );
}

#[test]
fn every_agents_counts_as_agents_for_what_it_holds() {
    let two = two();
    let state = two.log.state();
    assert!(holds_folder(&state, two.laptop_global));
    assert!(holds_folder(&state, two.desk_global));
    assert_eq!(
        tree(&state, root_of(&state).unwrap()),
        [two.desk, two.desk_global].into()
    );
}

#[test]
fn an_older_agents_an_agent_made_never_takes_in_the_persons() {
    let mut log = Log::new();
    let made_by_agent = log.folder(AGENT, FOLDER, None);
    std::thread::sleep(std::time::Duration::from_millis(2));
    let mine = log.folder(DESK, FOLDER, None);
    log.doc(DESK, "mine", Some(mine));
    assert!(made_by_agent < mine);
    assert!(joined(&log.state()).is_empty());
}
