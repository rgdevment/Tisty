use super::*;
use tisty_core::Paths;

const HERE: &str = "this-computer";

fn bring_chosen(
    session: &mut Session,
    homes: &Homes,
    here: &str,
    chosen: &[String],
) -> Answer<Brought> {
    let (ledger, held) = consented_here(session, here)?;
    let chosen: BTreeSet<String> = chosen.iter().cloned().collect();
    let (read, refused) = read_chosen(&ledger, &found_here(homes), &chosen, &held);
    brought_home(session, homes, ledger, &read, refused)
}

struct Room {
    _tmp: tempfile::TempDir,
    home: std::path::PathBuf,
    homes: Homes,
    session: Session,
}

fn put(at: &Path, said: &str) {
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, said).unwrap();
}

fn room() -> Room {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    put(&home.join(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    put(&home.join(".claude/rules/tone.md"), "Short answers.\n");
    put(&home.join(".claude/settings.json"), "{\"token\": 1}\n");
    put(&home.join(".codex/AGENTS.md"), "Codex rules.\n");
    put(
        &home.join(".copilot/copilot-instructions.md"),
        "Be brief.\n",
    );
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let session = Session::at(paths).unwrap();
    let homes = Homes::with(&home, &[]);
    Room {
        _tmp: tmp,
        home,
        homes,
        session,
    }
}

impl Room {
    fn at(&self, name: &str) -> String {
        name.split('/')
            .fold(self.home.clone(), |at, step| at.join(step))
            .to_string_lossy()
            .into_owned()
    }

    fn look(&self) -> Look {
        looked(
            &gathered(&self.session),
            &self.homes,
            HERE,
            &found_here(&self.homes),
        )
    }

    fn bring(&mut self, chosen: &[&str]) -> Brought {
        let chosen: Vec<String> = chosen.iter().map(|one| self.at(one)).collect();
        bring_chosen(&mut self.session, &self.homes, HERE, &chosen).unwrap()
    }

    fn folders(&self) -> Vec<String> {
        let mut all: Vec<String> = self
            .session
            .state
            .folders
            .values()
            .map(|one| one.name.clone())
            .collect();
        all.sort();
        all
    }

    fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.at(name)).unwrap()
    }

    fn kept(&self, name: &str) -> usize {
        agents::kept(self.session.paths.data(), Path::new(&self.at(name))).len()
    }
}

#[test]
fn looking_lists_names_only_and_says_yes_was_never_given() {
    let room = room();
    let look = room.look();
    assert!(look.asks);
    assert!(!look.consented);
    assert_eq!(look.folder, None);
    let ids: Vec<&str> = look.assistants.iter().map(|one| one.id.as_str()).collect();
    assert_eq!(ids, ["claude", "codex", "copilot"]);
    let claude = &look.assistants[0];
    let settings = claude.files.iter().find(|one| one.name == "settings.json");
    assert!(
        settings.is_none(),
        "settings.json is not where Claude keeps rules"
    );
    assert!(claude.files.iter().any(|one| one.name == "rules/tone.md"));
    assert!(
        !agents::folder(room.session.paths.data()).exists(),
        "«Ahora no» leaves nothing behind"
    );
}

#[test]
fn bringing_files_only_the_chosen_ones_and_the_next_round_writes_nothing() {
    let mut room = room();
    let before = room.read(".claude/CLAUDE.md");
    let brought = room.bring(&[".claude/CLAUDE.md", ".codex/AGENTS.md"]);
    assert_eq!(brought.made, 2);
    assert!(brought.refused.is_empty(), "{:?}", brought.refused);
    assert_eq!(room.folders(), ["AGENTS", "Claude", "Codex", "Global"]);
    let root = root_of(&room.session.state).unwrap();
    assert_eq!(
        room.session.state.folders[&root].color.as_deref(),
        Some(places::HUE)
    );
    assert!(brought.round.both.is_empty());
    assert!(
        brought
            .round
            .told
            .iter()
            .all(|one| one.standing == "upToDate"),
        "{:?}",
        brought.round.told
    );
    assert_eq!(room.read(".claude/CLAUDE.md"), before);
    assert_eq!(room.kept(".claude/CLAUDE.md"), 0);

    let look = room.look();
    assert!(!look.asks);
    assert_eq!(look.docs, 2);
    let held: Vec<&str> = look
        .assistants
        .iter()
        .flat_map(|one| &one.files)
        .filter(|one| one.held)
        .map(|one| one.name.as_str())
        .collect();
    assert_eq!(held, ["CLAUDE.md", "AGENTS.md"]);

    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    let again = round_now(&room.session, &room.homes, &mut ledger);
    assert!(again.told.iter().all(|one| one.standing == "upToDate"));
}

#[test]
fn saying_yes_with_nothing_chosen_makes_no_folder() {
    let mut room = room();
    let brought = room.bring(&[]);
    assert_eq!(brought.made, 0);
    assert!(room.folders().is_empty());
    assert!(room.look().consented);
}

#[test]
fn a_file_that_reads_like_a_credential_is_refused_and_the_rest_still_comes() {
    let mut room = room();
    put(
        Path::new(&room.at(".claude/rules/keys.md")),
        "token: ghp_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n",
    );
    let brought = room.bring(&[".claude/rules/keys.md", ".claude/rules/tone.md"]);
    assert_eq!(brought.made, 1);
    assert_eq!(brought.refused.len(), 1, "{:?}", brought.refused);
    assert!(brought.refused[0].at.ends_with("keys.md"));
    assert_eq!(brought.refused[0].reason, "secret");
}

fn second_computer(room: &mut Room) {
    room.bring(&[".claude/CLAUDE.md"]);
    let data = room.session.paths.data().to_path_buf();
    std::fs::remove_dir_all(agents::folder(&data)).unwrap();
    put(
        Path::new(&room.at(".claude/CLAUDE.md")),
        "# Claude\nTests last.\n",
    );
}

#[test]
fn on_a_computer_that_has_not_said_yes_a_file_that_differs_is_put_to_the_person() {
    let mut room = room();
    second_computer(&mut room);
    let look = room.look();
    assert!(
        look.asks,
        "documents are here but this computer never said yes"
    );
    let brought = room.bring(&[]);
    let [both] = brought.round.both.as_slice() else {
        panic!("{:?}", brought.round);
    };
    assert!(both.at.ends_with("CLAUDE.md"));
    assert!(!both.composed);
    let sides: Vec<(&str, &str)> = both
        .lines
        .iter()
        .map(|one| (one.side, one.text.as_str()))
        .collect();
    assert_eq!(
        sides,
        [
            ("same", "# Claude"),
            ("tisty", "Tests first."),
            ("file", "Tests last."),
        ]
    );
}

#[test]
fn keeping_tisty_writes_the_file_and_backs_up_what_it_held() {
    let mut room = room();
    second_computer(&mut room);
    room.bring(&[]);
    let at = room.at(".claude/CLAUDE.md");
    let after = kept_side(&mut room.session, &room.homes, HERE, &at, true).unwrap();
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert_eq!(room.kept(".claude/CLAUDE.md"), 1);
    assert!(after.both.is_empty());
}

#[test]
fn keeping_the_file_rewrites_the_document_and_leaves_the_file_alone() {
    let mut room = room();
    second_computer(&mut room);
    room.bring(&[]);
    let at = room.at(".claude/CLAUDE.md");
    let after = kept_side(&mut room.session, &room.homes, HERE, &at, false).unwrap();
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests last.\n");
    assert!(after.both.is_empty(), "{after:?}");
    let docs = bodies(&room.session);
    assert!(docs[0].1.contains("Tests last."), "{docs:?}");
    assert_eq!(room.kept(".claude/CLAUDE.md"), 0);
}

#[test]
fn an_assistant_turned_off_is_left_as_it_is_and_nothing_in_tisty_goes() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    turned(&room.session, &room.homes, HERE, Some("claude"), false).unwrap();
    let file = bodies(&room.session)[0].0.clone();
    let body = bodies(&room.session)[0]
        .1
        .replace("Tests first.", "Tests always.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &body).unwrap();
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    let told = round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(told.told[0].standing, "off");
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert_eq!(bodies(&room.session).len(), 1);

    turned(&room.session, &room.homes, HERE, None, false).unwrap();
    turned(&room.session, &room.homes, HERE, Some("claude"), true).unwrap();
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "# Claude\nTests first.\n",
        "the whole folder is still off"
    );
    turned(&room.session, &room.homes, HERE, None, true).unwrap();
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests always.\n");
}

#[test]
fn emptying_agents_takes_it_out_of_tisty_and_touches_no_agent_file() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md", ".codex/AGENTS.md"]);
    assert!(
        !emptied(&mut room.session).unwrap(),
        "it still holds documents"
    );
    let ids: Vec<String> = room
        .session
        .state
        .docs
        .values()
        .map(|one| one.id.to_string())
        .collect();
    for id in ids {
        room.session.drop_doc(&id).unwrap();
    }
    assert!(emptied(&mut room.session).unwrap());
    assert!(room.folders().is_empty(), "{:?}", room.folders());
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert_eq!(room.read(".codex/AGENTS.md"), "Codex rules.\n");

    let look = room.look();
    assert!(look.asks);
    let brought = room.bring(&[".claude/CLAUDE.md"]);
    assert_eq!(brought.made, 1, "entering again brings back what it finds");
}

#[test]
fn the_comparison_keeps_the_order_of_both_sides() {
    let lines = compared(b"a\nb\nc\n", b"a\nx\nc\nd\n");
    let sides: Vec<(&str, &str)> = lines
        .iter()
        .map(|one| (one.side, one.text.as_str()))
        .collect();
    assert_eq!(
        sides,
        [
            ("same", "a"),
            ("tisty", "b"),
            ("file", "x"),
            ("same", "c"),
            ("file", "d"),
        ]
    );
}

fn global_doc(room: &mut Room, said: &str) {
    let body = agents::doc_of(
        agents::GLOBAL,
        agents::Kind::Rule,
        "style.md",
        said.as_bytes(),
    )
    .unwrap();
    let root = root_of(&room.session.state).unwrap();
    let device = room.session.store.device().clone();
    let made = tisty_core::docs::create(&room.session.paths.docs(), &device, &body).unwrap();
    room.session
        .commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                file: made.id,
                folder: Some(root),
                order: "a".to_string(),
                said: Some(tisty_core::event::Said::of(&body)),
                ..Default::default()
            },
        })
        .unwrap();
}

#[test]
fn a_file_made_only_of_a_global_rule_never_rewrites_that_rule_from_one_assistant() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    global_doc(&mut room, "Codex rules.\nSpanish.\n");
    std::fs::remove_dir_all(agents::folder(room.session.paths.data())).unwrap();
    let brought = room.bring(&[]);
    let both = brought
        .round
        .both
        .iter()
        .find(|one| one.at.ends_with("AGENTS.md"))
        .unwrap_or_else(|| panic!("{:?}", brought.round));
    assert!(both.composed, "a Global rule belongs to every assistant");
    let at = room.at(".codex/AGENTS.md");
    let refusal = kept_side(&mut room.session, &room.homes, HERE, &at, false).unwrap_err();
    assert_eq!(
        refusal.code, "agentUnattributed",
        "said in the person's language, not as raw text"
    );
    let global = bodies(&room.session)
        .into_iter()
        .find(|(_, body)| body.contains("global/rule"))
        .unwrap();
    assert!(global.1.contains("Spanish."), "{}", global.1);
}

#[test]
fn tistys_own_line_in_geminis_file_is_not_taken_for_a_change_outside() {
    let mut room = room();
    put(Path::new(&room.at(".gemini/GEMINI.md")), "- remembered\n");
    room.bring(&[".gemini/GEMINI.md"]);
    global_doc(&mut room, "Spanish.\n");
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(
        room.read(".gemini/GEMINI.md"),
        "- remembered\n@tisty-rules.md\n"
    );
    let again = round_now(&room.session, &room.homes, &mut ledger);
    assert!(
        again
            .told
            .iter()
            .filter(|one| one.assistant == "gemini")
            .all(|one| one.standing == "upToDate"),
        "{:?}",
        again.told
    );
}

#[test]
fn a_choice_left_for_later_is_asked_again() {
    let mut room = room();
    second_computer(&mut room);
    room.bring(&[]);
    let look = room.look();
    assert!(look.asks, "the file and its document still differ");
    let [both] = look.both.as_slice() else {
        panic!("{:?}", look.both);
    };
    assert!(both.at.ends_with("CLAUDE.md"));

    let at = room.at(".claude/CLAUDE.md");
    kept_side(&mut room.session, &room.homes, HERE, &at, true).unwrap();
    let look = room.look();
    assert!(look.both.is_empty());
    assert!(!look.asks);
}
