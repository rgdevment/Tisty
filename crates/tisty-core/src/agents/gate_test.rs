use super::*;
use crate::agents::{Kind, Standing, doc_of, overwrite, round};
use crate::event::{DocAdd, Event, Filed, FolderAdd, Op, Said};
use ulid::Ulid;

const HERE: &str = "dev_here";
const THERE: &str = "dev_there";
const AGENT: &str = "dev_agent";

struct World {
    docs: tempfile::TempDir,
    data: tempfile::TempDir,
    home: tempfile::TempDir,
    state: State,
    ledger: Ledger,
    root: FolderId,
    claude: FolderId,
    ms: i64,
}

fn rule(said: &str) -> String {
    doc_of("claude", Kind::Rule, "CLAUDE.md", said.as_bytes()).unwrap()
}

fn memory(said: &str) -> String {
    doc_of(
        "claude",
        Kind::Memory,
        "projects/p/memory/MEMORY.md",
        said.as_bytes(),
    )
    .unwrap()
}

fn world() -> World {
    let mut ledger = Ledger::fresh("machine-here");
    ledger.consent(true);
    let mut world = World {
        docs: tempfile::tempdir().unwrap(),
        data: tempfile::tempdir().unwrap(),
        home: tempfile::tempdir().unwrap(),
        state: State::default(),
        ledger,
        root: Ulid::generate(),
        claude: Ulid::generate(),
        ms: 0,
    };
    world.apply(
        AGENT,
        Op::DeviceJoin {
            d: DeviceId(AGENT.into()),
            k: Some(crate::event::DeviceKind::Agent),
            p: None,
        },
    );
    let (root, claude) = (world.root, world.claude);
    world.folder(HERE, root, "AGENTS", None);
    world.folder(HERE, claude, "Claude", Some(root));
    world
}

impl World {
    fn apply(&mut self, who: &str, op: Op) {
        self.ms += 1;
        let at = jiff::Timestamp::from_millisecond(self.ms).unwrap();
        self.state.apply(&Event::new(DeviceId(who.into()), at, op));
    }

    fn folder(&mut self, who: &str, id: FolderId, name: &str, parent: Option<FolderId>) {
        self.apply(
            who,
            Op::FolderAdd {
                id,
                d: FolderAdd {
                    name: name.into(),
                    order: "a0".into(),
                    parent,
                    icon: None,
                    color: None,
                },
            },
        );
    }

    fn said(body: &str, agreed: bool) -> Said {
        match agreed {
            true => Said::of(body).agreed(),
            false => Said::of(body),
        }
    }

    fn made(&mut self, who: &str, body: &str, agreed: bool) -> (Ulid, String) {
        self.made_in(who, body, agreed, Some(self.claude))
    }

    fn made_in(
        &mut self,
        who: &str,
        body: &str,
        agreed: bool,
        folder: Option<FolderId>,
    ) -> (Ulid, String) {
        let file = crate::docs::create(self.docs.path(), &DeviceId(who.into()), body)
            .unwrap()
            .id;
        let id = Ulid::generate();
        self.apply(
            who,
            Op::DocAdd {
                id,
                d: DocAdd {
                    file: file.clone(),
                    order: "a0".into(),
                    said: Some(Self::said(body, agreed)),
                    folder,
                    ..Default::default()
                },
            },
        );
        (id, file)
    }

    fn written(&mut self, who: &str, id: Ulid, file: &str, body: &str, agreed: bool) {
        std::fs::write(self.docs.path().join(format!("{file}.md")), body).unwrap();
        self.apply(
            who,
            Op::DocSaid {
                id,
                d: Self::said(body, agreed),
            },
        );
    }

    fn gate_for(&self, me: &str) -> Gate {
        gate(&Asked {
            state: &self.state,
            docs: self.docs.path(),
            root: Some(self.root),
            ledger: &self.ledger,
            me: &DeviceId(me.into()),
        })
    }

    fn gate(&self) -> Gate {
        self.gate_for(HERE)
    }

    fn target(&self, gate: &Gate) -> Target {
        let at = self.home.path().join(".claude").join("CLAUDE.md");
        let mut target = gate.target(&self.ledger, self.home.path().into(), at, "claude");
        for piece in gate.admitted() {
            target.put(piece);
        }
        target
    }

    fn round(&mut self) -> (Target, Standing) {
        let gate = self.gate();
        let target = self.target(&gate);
        let told = round(
            self.data.path(),
            &mut self.ledger,
            std::slice::from_ref(&target),
        );
        (target, told[0].clone())
    }

    fn on_disk(&self) -> Option<String> {
        std::fs::read_to_string(self.home.path().join(".claude").join("CLAUDE.md")).ok()
    }
}

#[test]
fn what_the_person_wrote_here_is_admitted_without_a_click() {
    let mut world = world();
    let (_, file) = world.made(HERE, &rule("Tests first.\n"), true);

    assert_eq!(world.gate().seen(&file), Some(Seen::Admitted));
    assert_eq!(world.round().1, Standing::Wrote);
    assert_eq!(world.on_disk().as_deref(), Some("Tests first.\n"));
}

#[test]
fn a_document_from_another_computer_waits_for_this_one_to_choose() {
    let mut world = world();
    let (_, file) = world.made(THERE, &rule("From the laptop.\n"), true);

    assert_eq!(world.gate().seen(&file), Some(Seen::WaitingHere));
    assert!(world.gate().admitted().is_empty());

    world.ledger.take(&file, false);
    assert_eq!(world.gate().seen(&file), Some(Seen::DeclinedHere));

    world.ledger.take(&file, true);
    assert_eq!(world.gate().seen(&file), Some(Seen::Admitted));
    assert_eq!(world.gate_for(THERE).seen(&file), Some(Seen::Admitted));
}

#[test]
fn what_this_computer_already_applies_updates_by_itself_when_another_changes_it() {
    let mut world = world();
    let (id, file) = world.made(THERE, &rule("One.\n"), true);
    world.ledger.take(&file, true);
    world.round();

    world.written(THERE, id, &file, &rule("Two.\n"), true);
    assert_eq!(world.round().1, Standing::Wrote);
    assert_eq!(world.on_disk().as_deref(), Some("Two.\n"));
}

#[test]
fn an_agent_saying_it_agreed_is_not_believed() {
    let mut world = world();
    let (id, file) = world.made(HERE, &rule("Mine.\n"), true);
    world.written(AGENT, id, &file, &rule("The agent's.\n"), true);

    assert_eq!(world.gate().seen(&file), Some(Seen::NotAgreed));
}

#[test]
fn a_body_changed_on_disk_with_no_word_in_the_log_is_not_agreed() {
    let mut world = world();
    let (_, file) = world.made(HERE, &rule("Mine.\n"), true);
    std::fs::write(
        world.docs.path().join(format!("{file}.md")),
        rule("Written around Tisty.\n"),
    )
    .unwrap();

    assert_eq!(world.gate().seen(&file), Some(Seen::NotAgreed));
}

#[test]
fn reading_a_body_back_from_disk_never_agrees_to_it() {
    let mut world = world();
    let (id, file) = world.made(HERE, &rule("Mine.\n"), true);
    world.written(AGENT, id, &file, &rule("The agent's.\n"), false);
    world.written(HERE, id, &file, &rule("The agent's.\n"), false);

    assert_eq!(world.gate().seen(&file), Some(Seen::NotAgreed));
    world.written(HERE, id, &file, &rule("The agent's.\n"), true);
    assert_eq!(world.gate().seen(&file), Some(Seen::Admitted));
}

#[test]
fn a_document_an_agent_moved_in_is_never_written() {
    let mut world = world();
    let (id, file) = world.made_in(HERE, &rule("Elsewhere.\n"), true, None);
    let claude = world.claude;
    world.apply(
        AGENT,
        Op::DocMove {
            id,
            d: Filed {
                folder: Some(Some(claude)),
                ..Default::default()
            },
        },
    );

    assert_eq!(world.gate().seen(&file), Some(Seen::PlacedByAgent));
}

#[test]
fn a_folder_an_agent_renamed_holds_back_everything_under_it() {
    let mut world = world();
    let (_, file) = world.made(HERE, &rule("Mine.\n"), true);
    let claude = world.claude;
    world.apply(
        AGENT,
        Op::FolderRename {
            id: claude,
            d: crate::event::Name {
                name: "Claude rules".into(),
            },
        },
    );

    assert_eq!(world.gate().seen(&file), Some(Seen::PlacedByAgent));
}

#[test]
fn memory_is_only_ever_the_computer_that_brought_it() {
    let mut world = world();
    let (_, there) = world.made(THERE, &memory("Laptop memory.\n"), true);
    let (_, here) = world.made(HERE, &memory("Desk memory.\n"), true);

    let gate = world.gate();
    assert_eq!(gate.seen(&there), Some(Seen::OtherComputersMemory));
    assert_eq!(gate.seen(&here), Some(Seen::Admitted));
    world.ledger.take(&there, true);
    assert_eq!(
        world.gate().seen(&there),
        Some(Seen::OtherComputersMemory),
        "choosing it here does not make it this computer's"
    );
}

#[test]
fn a_file_this_computer_wrote_is_held_when_one_of_its_documents_changes_without_the_person() {
    let mut world = world();
    let (id, file) = world.made(HERE, &rule("Mine.\n"), true);
    world.round();
    world.written(AGENT, id, &file, &rule("The agent's.\n"), false);

    let (target, told) = world.round();
    assert_eq!(told, Standing::Held);
    assert!(target.held());
    assert_eq!(world.on_disk().as_deref(), Some("Mine.\n"));
    assert_eq!(
        overwrite(world.data.path(), &mut world.ledger, &target),
        Err(crate::agents::Refused::Held)
    );
}

#[test]
fn a_document_never_written_here_that_fails_is_left_out_without_holding_the_rest() {
    let mut world = world();
    world.made(HERE, &rule("Mine.\n"), true);
    world.round();
    let (_, foreign) = world.made(AGENT, &rule("An agent's new rule.\n"), true);

    let gate = world.gate();
    assert_eq!(gate.seen(&foreign), Some(Seen::NotAgreed));
    assert!(!gate.held(&foreign));
    assert_eq!(world.round().1, Standing::UpToDate);
}

#[test]
fn a_document_an_agent_archived_holds_the_file_and_one_the_person_archived_leaves_it() {
    let mut world = world();
    let (keep, _) = world.made(HERE, &rule("Keep.\n"), true);
    let (drop, dropped) = world.made(HERE, &rule("Drop.\n"), true);
    let _ = keep;
    world.round();
    world.apply(AGENT, Op::DocArchive { id: drop });

    assert!(world.gate().held(&dropped));
    assert_eq!(world.round().1, Standing::Held);

    world.apply(HERE, Op::DocUnarchive { id: drop });
    world.apply(HERE, Op::DocArchive { id: drop });
    assert!(!world.gate().held(&dropped));
    assert_eq!(world.round().1, Standing::Wrote);
    assert_eq!(world.on_disk().as_deref(), Some("Keep.\n"));
}

#[test]
fn a_move_that_only_reorders_never_takes_the_placement_from_whoever_made_it() {
    let mut world = world();
    let (id, file) = world.made_in(AGENT, &rule("Agent placed.\n"), false, None);
    let claude = world.claude;
    world.apply(
        AGENT,
        Op::DocMove {
            id,
            d: Filed {
                folder: Some(Some(claude)),
                ..Default::default()
            },
        },
    );
    world.written(HERE, id, &file, &rule("Agent placed.\n"), true);
    world.apply(
        HERE,
        Op::DocMove {
            id,
            d: Filed {
                order: Some("b0".into()),
                ..Default::default()
            },
        },
    );

    assert_eq!(world.gate().seen(&file), Some(Seen::PlacedByAgent));
}

#[test]
fn a_file_written_before_the_ledger_knew_its_documents_waits_until_they_say_the_same() {
    let mut world = world();
    world.made(HERE, &rule("Global and mine.\n"), true);
    let at = world.home.path().join(".claude").join("CLAUDE.md");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "Mine.\n").unwrap();
    world.ledger.settle(&at, b"Mine.\n");

    let (target, told) = world.round();
    assert_eq!(told, Standing::Unproven);
    assert_eq!(world.on_disk().as_deref(), Some("Mine.\n"));

    overwrite(world.data.path(), &mut world.ledger, &target).unwrap();
    assert_eq!(world.on_disk().as_deref(), Some("Global and mine.\n"));
    assert!(!world.ledger.unproven(&at));
}

#[test]
fn a_file_whose_documents_change_while_its_bytes_do_not_learns_its_new_documents() {
    let mut world = world();
    let (old, _) = world.made(HERE, &rule("Same words.\n"), true);
    world.round();
    let (_, new) = world.made(HERE, &rule("Same words.\n"), true);
    world.apply(HERE, Op::DocArchive { id: old });

    assert_eq!(world.round().1, Standing::UpToDate);
    let at = world.home.path().join(".claude").join("CLAUDE.md");
    assert_eq!(
        world.ledger.applied_at(&at).collect::<Vec<_>>(),
        [new.as_str()]
    );
}

#[test]
fn a_file_written_before_the_ledger_knew_its_documents_is_taken_once_they_say_the_same() {
    let mut world = world();
    let (_, file) = world.made(HERE, &rule("Mine.\n"), true);
    let at = world.home.path().join(".claude").join("CLAUDE.md");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "Mine.\n").unwrap();
    world.ledger.settle(&at, b"Mine.\n");

    assert_eq!(world.round().1, Standing::UpToDate);
    assert!(!world.ledger.unproven(&at));
    assert_eq!(
        world.ledger.applied_at(&at).collect::<Vec<_>>(),
        [file.as_str()]
    );
}

#[test]
fn learning_a_files_documents_never_moves_what_the_file_was_last_seen_saying() {
    let mut world = world();
    world.made(HERE, &memory("What the document says.\n"), true);
    let at = world.home.path().join(".claude").join("CLAUDE.md");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "What the agent wrote.\n").unwrap();
    world.ledger.settle(&at, b"What the agent wrote.\n");

    assert_eq!(world.round().1, Standing::UpToDate);
    assert_eq!(world.round().1, Standing::UpToDate);
    assert_eq!(
        world.ledger.written(&at),
        Some(crate::attach::printed(b"What the agent wrote.\n").as_str())
    );
}

#[test]
fn only_an_agent_document_without_approval_is_counted_as_waiting_for_one() {
    let mut world = world();
    let (_, carded) = world.made(AGENT, &rule("An agent's rule.\n"), false);
    let (_, plain) = world.made(AGENT, "# Just notes\n", false);
    let gate = world.gate();
    assert_eq!(gate.unagreed(), 1);
    let seen: Vec<(&str, Seen)> = gate.all_seen().collect();
    assert!(seen.contains(&(carded.as_str(), Seen::NotAgreed)));
    assert!(seen.contains(&(plain.as_str(), Seen::NotAgreed)));
}
