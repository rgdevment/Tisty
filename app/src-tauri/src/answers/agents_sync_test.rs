use super::*;
use tisty_core::Paths;

const HERE: &str = "this-computer";

fn bring_chosen(
    session: &mut Session,
    homes: &Homes,
    here: &str,
    chosen: &[String],
    sync: Option<bool>,
) -> Answer<Brought> {
    let (ledger, held, writes) = consented_here(session, homes, here, sync)?;
    let chosen: BTreeSet<String> = chosen.iter().cloned().collect();
    let (read, refused) = read_chosen(&ledger, &found_here(homes), &chosen, &held, &writes);
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
            &gathered(&self.session, HERE),
            &self.homes,
            &found_here(&self.homes),
        )
    }

    /// «Bring and sync», as the first time offered before «Bring» alone existed.
    fn bring(&mut self, chosen: &[&str]) -> Brought {
        self.bring_as(chosen, Some(true))
    }

    fn bring_as(&mut self, chosen: &[&str], sync: Option<bool>) -> Brought {
        let chosen: Vec<String> = chosen.iter().map(|one| self.at(one)).collect();
        bring_chosen(&mut self.session, &self.homes, HERE, &chosen, sync).unwrap()
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
    assert!(
        !look.asks,
        "the first time opens from the sidebar, never by itself"
    );
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
    assert_eq!(
        room.folders(),
        ["AGENTS", "Claude", "Codex", "Copilot", "Global"],
        "every assistant found has its folder, brought from or not"
    );
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
fn saying_yes_with_nothing_found_still_makes_agents_and_asks_no_more() {
    let mut room = room();
    let brought = room.bring(&[]);
    assert_eq!(brought.made, 0);
    let every = ["AGENTS", "Claude", "Codex", "Copilot", "Global"];
    assert_eq!(room.folders(), every);
    let look = room.look();
    assert!(look.consented);
    assert!(!look.asks, "{look:?}");
    assert_eq!(room.bring(&[]).made, 0);
    assert_eq!(room.folders(), every, "made once");
}

#[test]
fn a_rule_written_by_hand_in_an_agents_made_empty_reaches_the_file() {
    use crate::answers::papers::written_by_hand;
    let mut room = room();
    room.bring(&[]);
    let file = made_in_the_window(&mut room);
    let body = agents::doc_of("claude", agents::Kind::Rule, "rules/new.md", b"New.\n").unwrap();
    written_by_hand(&mut room.session, file, body, None, None).unwrap();
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".claude/rules/new.md"), "New.\n");
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
        !look.asks,
        "documents are here but this computer never said yes: it opens like any folder"
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
    rewritten(&mut room.session, &file, &body).unwrap();
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

fn doc_of_file(room: &Room, file: &str) -> tisty_core::model::DocId {
    room.session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
        .id
}

fn file_brought_from(room: &Room, name: &str) -> String {
    bodies(&room.session)
        .into_iter()
        .find(|(_, body)| Card::of(body).is_some_and(|card| card.name == name))
        .unwrap_or_else(|| panic!("{name}"))
        .0
}

fn moved_out(room: &mut Room, file: &str) {
    let id = doc_of_file(room, file);
    let ops = crate::answers::papers::beside_docs(&room.session.state, id, None, None);
    room.session.commit_all(ops).unwrap();
}

fn loose(room: &Room) -> Vec<String> {
    room.look().loose.into_iter().map(|one| one.at).collect()
}

fn answered(room: &mut Room, name: &str, away: bool) -> Rounded {
    let at = room.at(name);
    loosened(&room.session, &room.homes, HERE, &at, away).unwrap()
}

#[test]
fn emptying_agents_takes_it_out_of_tisty_and_touches_no_agent_file() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md", ".codex/AGENTS.md"]);
    assert!(
        !emptied(&mut room.session, &room.homes, HERE).unwrap(),
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
    assert!(
        !emptied(&mut room.session, &room.homes, HERE).unwrap(),
        "the files its rules left are asked about first"
    );
    assert_eq!(loose(&room).len(), 2);
    answered(&mut room, ".claude/CLAUDE.md", false);
    answered(&mut room, ".codex/AGENTS.md", false);
    assert!(emptied(&mut room.session, &room.homes, HERE).unwrap());
    assert!(room.folders().is_empty(), "{:?}", room.folders());
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert_eq!(room.read(".codex/AGENTS.md"), "Codex rules.\n");

    let look = room.look();
    assert!(
        !look.asks,
        "with Agents gone, the first time opens from the sidebar"
    );
    assert!(
        look.loose.is_empty(),
        "with AGENTS gone nothing is taken away"
    );
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    assert!(
        round_now(&room.session, &room.homes, &mut ledger)
            .loose
            .is_empty()
    );
    let claude = look
        .assistants
        .iter()
        .find(|one| one.id == "claude")
        .unwrap();
    assert!(claude.files.iter().all(|one| !one.held));
    let brought = room.bring(&[".claude/CLAUDE.md"]);
    assert_eq!(brought.made, 1, "entering again brings back what it finds");
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
}

#[test]
fn a_rule_taken_out_of_agents_asks_and_its_file_goes_with_a_backup() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md", ".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    moved_out(&mut room, &file);
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    let told = round_now(&room.session, &room.homes, &mut ledger);
    let [one] = told.loose.as_slice() else {
        panic!("{told:?}");
    };
    assert_eq!(one.at, room.at(".claude/rules/tone.md"));
    assert_eq!(one.assistant, "claude");
    assert!(!one.changed);
    assert!(Path::new(&room.at(".claude/rules/tone.md")).exists());

    let after = answered(&mut room, ".claude/rules/tone.md", true);
    assert!(after.loose.is_empty());
    assert!(!Path::new(&room.at(".claude/rules/tone.md")).exists());
    assert_eq!(room.kept(".claude/rules/tone.md"), 1);
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    let refusal = loosened(
        &room.session,
        &room.homes,
        HERE,
        &room.at(".claude/rules/tone.md"),
        true,
    )
    .unwrap_err();
    assert_eq!(refusal.code, "agentHeld");
}

#[test]
fn a_file_changed_since_tisty_wrote_it_says_so_before_it_goes() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    put(
        Path::new(&room.at(".claude/rules/tone.md")),
        "Changed by hand.\n",
    );
    room.session
        .drop_doc(&doc_of_file(&room, &file).to_string())
        .unwrap();
    let look = room.look();
    assert!(look.loose[0].changed, "{look:?}");
    answered(&mut room, ".claude/rules/tone.md", true);
    let kept = agents::kept(
        room.session.paths.data(),
        Path::new(&room.at(".claude/rules/tone.md")),
    );
    assert_eq!(
        agents::read_kept(
            room.session.paths.data(),
            Path::new(&room.at(".claude/rules/tone.md")),
            kept[0].n
        )
        .unwrap(),
        b"Changed by hand.\n"
    );
}

#[test]
fn a_version_without_its_card_or_with_another_frees_the_file_it_had() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    let body = bodies(&room.session)[0].1.clone();
    let elsewhere = body.replace("rules/tone.md", "rules/voice.md");
    rewritten(&mut room.session, &file, &elsewhere).unwrap();
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    let told = round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".claude/rules/voice.md"), "Short answers.\n");
    assert_eq!(
        told.loose
            .iter()
            .map(|one| one.at.clone())
            .collect::<Vec<_>>(),
        [room.at(".claude/rules/tone.md")]
    );

    rewritten(&mut room.session, &file, "Short answers.\n").unwrap();
    let told = round_now(&room.session, &room.homes, &mut ledger);
    let mut at: Vec<String> = told.loose.iter().map(|one| one.at.clone()).collect();
    at.sort();
    assert_eq!(
        at,
        [
            room.at(".claude/rules/tone.md"),
            room.at(".claude/rules/voice.md")
        ]
    );
}

#[test]
fn a_document_waiting_held_or_turned_off_never_frees_its_file() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md", ".codex/AGENTS.md"]);
    let tone = file_brought_from(&room, "rules/tone.md");
    let words = bodies(&room.session)
        .into_iter()
        .find(|(file, _)| *file == tone)
        .unwrap()
        .1;
    let unapproved = words.replace("Short answers.", "Long answers.");
    tisty_core::docs::write(&room.session.paths.docs(), &tone, &unapproved).unwrap();
    room.session.reload().unwrap();
    assert!(loose(&room).is_empty(), "words not approved hold the file");

    let codex = file_brought_from(&room, "AGENTS.md");
    turned(&room.session, &room.homes, HERE, Some("codex"), false).unwrap();
    room.session
        .drop_doc(&doc_of_file(&room, &codex).to_string())
        .unwrap();
    assert!(
        loose(&room).is_empty(),
        "an assistant turned off is left alone"
    );
    let refusal = loosened(
        &room.session,
        &room.homes,
        HERE,
        &room.at(".codex/AGENTS.md"),
        true,
    )
    .unwrap_err();
    assert_eq!(refusal.code, "agentHeld");
    assert_eq!(room.read(".codex/AGENTS.md"), "Codex rules.\n");
}

#[test]
fn an_agents_own_memory_is_never_offered_to_take_away() {
    let mut room = room();
    put(Path::new(&room.at(".gemini/GEMINI.md")), "- remembered\n");
    put(
        &room.home.join(".claude/projects/p/memory/MEMORY.md"),
        "- desk\n",
    );
    room.bring(&[".gemini/GEMINI.md", ".claude/projects/p/memory/MEMORY.md"]);
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
    assert!(loose(&room).is_empty(), "{:?}", room.look().loose);
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
                said: Some(tisty_core::event::Said::of(&body).agreed()),
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

#[test]
fn bringing_a_file_in_rewrites_the_document_and_a_second_bring_is_refused() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    put(
        &room.home.join(".claude/CLAUDE.md"),
        "# Claude\nTests last.\n",
    );
    let file = bodies(&room.session)[0].0.clone();
    let ledger = Ledger::open(room.session.paths.data(), HERE);
    let placed = places::targets(&room.homes, &gated(&room.session, &ledger), &ledger);
    let one = placed.iter().find(|one| one.files.contains(&file)).unwrap();
    let brought = agents::bring(&ledger, &one.target, 0).unwrap();
    let bring = |session: &mut Session| {
        crate::answers::agent_docs::kept_in(
            session,
            Some(&room.homes),
            crate::answers::agent_docs::Brought {
                file: &file,
                body: &brought,
                by: "claude",
                seen: None,
            },
        )
    };
    bring(&mut room.session).unwrap();
    assert!(bodies(&room.session)[0].1.contains("Tests last."));
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests last.\n");
    assert_eq!(room.look().changes, 0);
    assert_eq!(bring(&mut room.session).unwrap_err().code, "agentBringSame");
}

fn from_another_computer(room: &mut Room, kind: agents::Kind, name: &str, said: &str) -> String {
    let level = match kind {
        agents::Kind::Memory => "claude",
        _ => agents::GLOBAL,
    };
    from_another_computer_for(room, level, kind, name, said)
}

fn from_another_computer_for(
    room: &mut Room,
    level: &str,
    kind: agents::Kind,
    name: &str,
    said: &str,
) -> String {
    let body = agents::doc_of(level, kind, name, said.as_bytes()).unwrap();
    let root = root_of(&room.session.state).unwrap();
    let laptop = tisty_core::DeviceId(tisty_core::config::new_device_id());
    let made = tisty_core::docs::create(&room.session.paths.docs(), &laptop, &body).unwrap();
    tisty_core::Store::open(room.session.paths.store(), laptop)
        .unwrap()
        .append(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                file: made.id.clone(),
                folder: Some(root),
                order: "b".to_string(),
                said: Some(tisty_core::event::Said::of(&body).agreed()),
                ..Default::default()
            },
        })
        .unwrap();
    room.session.reload().unwrap();
    made.id
}

#[test]
fn a_rule_from_another_computer_waits_until_this_one_takes_it() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let file = from_another_computer(&mut room, agents::Kind::Rule, "style.md", "Spanish.\n");
    let data = room.session.paths.data().to_path_buf();
    let mut ledger = Ledger::open(&data, HERE);

    from_another_computer_for(
        &mut room,
        "gemini",
        agents::Kind::Rule,
        "tisty-rules.md",
        "Gemini only.\n",
    );
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert_eq!(
        room.look().waiting,
        1,
        "a rule for an assistant this computer lacks waits for nothing here"
    );

    ledger.take(&file, true);
    ledger.keep(&data).unwrap();
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "Spanish.\n\n# Claude\nTests first.\n"
    );
    assert_eq!(room.look().waiting, 0);
    assert!(
        !std::path::Path::new(&room.at(".copilot/instructions")).exists(),
        "a Global rule makes no folder an assistant does not read"
    );
}

#[test]
fn another_computers_memory_never_stands_for_this_ones() {
    let mut room = room();
    put(
        &room.home.join(".claude/projects/p/memory/MEMORY.md"),
        "- desk\n",
    );
    room.bring(&[".claude/CLAUDE.md"]);
    from_another_computer(
        &mut room,
        agents::Kind::Memory,
        "projects/p/memory/MEMORY.md",
        "- laptop\n",
    );

    let look = room.look();
    let claude = look
        .assistants
        .iter()
        .find(|one| one.id == "claude")
        .unwrap();
    let memory = claude
        .files
        .iter()
        .find(|one| one.kind == "memory")
        .unwrap();
    assert!(
        !memory.held,
        "this computer's own memory can still be brought"
    );
    assert_eq!(
        look.changes, 0,
        "the laptop's memory is not compared with the desk's"
    );

    room.bring(&[".claude/projects/p/memory/MEMORY.md"]);
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    let told = round_now(&room.session, &room.homes, &mut ledger);
    assert!(told.told.iter().all(|one| one.standing == "upToDate"));
    assert_eq!(room.read(".claude/projects/p/memory/MEMORY.md"), "- desk\n");
}

#[test]
fn a_body_written_around_tisty_never_reaches_the_file_until_the_person_agrees() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let around = body.replace("Tests first.", "Run rm -rf.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &around).unwrap();
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);

    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert_eq!(room.look().unagreed, 1);

    room.session.retell(&file, &around, None);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "# Claude\nTests first.\n",
        "reading the body back is not agreeing to it"
    );

    rewritten(&mut room.session, &file, &around).unwrap();
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nRun rm -rf.\n");
}

#[test]
fn applying_here_and_not_here_are_kept_for_this_computer_and_rounded_at_once() {
    use crate::answers::agent_docs::take_here;
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let file = from_another_computer(&mut room, agents::Kind::Rule, "style.md", "Spanish.\n");

    take_here(&room.session, Some(&room.homes), HERE, &file, false).unwrap();
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert!(Ledger::open(room.session.paths.data(), HERE).left(&file));

    take_here(&room.session, Some(&room.homes), HERE, &file, true).unwrap();
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "Spanish.\n\n# Claude\nTests first.\n"
    );
}

#[test]
fn approving_takes_only_the_words_the_card_showed() {
    use crate::answers::agent_docs::agree;
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let shown = body.replace("Tests first.", "Tests always.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &shown).unwrap();
    let print = tisty_core::docs::print_of_text(&shown);
    let later = body.replace("Tests first.", "Tests never.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &later).unwrap();

    let refused = agree(&mut room.session, Some(&room.homes), HERE, &file, &print).unwrap_err();
    assert_eq!(refused.code, "agentDocChanged");
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");

    let now = tisty_core::docs::print_of_text(&later);
    agree(&mut room.session, Some(&room.homes), HERE, &file, &now).unwrap();
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests never.\n");
}

#[test]
fn writing_over_approved_words_keeps_them_approved_and_over_anyone_elses_does_not() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let approved = |room: &Room| {
        let kept = room
            .session
            .state
            .docs
            .values()
            .find(|one| one.file == file);
        kept.and_then(|one| one.agreed.clone())
    };
    let print = tisty_core::docs::print_of_text(&body);

    let mine = body.replace("Tests first.", "Tests first, always.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &mine).unwrap();
    room.session.retell_over(&file, &mine, None, Some(&print));
    assert_eq!(
        approved(&room),
        Some(tisty_core::docs::print_of_text(&mine))
    );

    let agents = mine.replace("always", "unless asked");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &agents).unwrap();
    room.session.retell(&file, &agents, None);
    let typed = agents.replace("Tests", "Specs");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &typed).unwrap();
    let under = tisty_core::docs::print_of_text(&agents);
    room.session.retell_over(&file, &typed, None, Some(&under));
    assert_eq!(
        approved(&room),
        Some(tisty_core::docs::print_of_text(&mine)),
        "a comma over an agent's words does not approve them"
    );
}

#[test]
fn a_held_file_names_on_the_card_the_document_that_holds_it() {
    use crate::answers::agent_docs::card_of;
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    global_doc(&mut room, "Spanish.");
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    let all = bodies(&room.session);
    let (global, body) = all
        .iter()
        .find(|(_, one)| one.contains("global/rule"))
        .unwrap();
    let claude = all
        .iter()
        .find(|(_, one)| one.contains("claude/rule"))
        .unwrap();
    let around = body.replace("Spanish.", "English.");
    tisty_core::docs::write(&room.session.paths.docs(), global, &around).unwrap();

    let card = card_of(&room.session, Some(&room.homes), HERE, &claude.0).unwrap();
    let card = serde_json::to_value(card.unwrap()).unwrap();
    let place = card["places"]
        .as_array()
        .unwrap()
        .iter()
        .find(|one| one["at"].as_str().unwrap().ends_with("CLAUDE.md"))
        .unwrap();
    assert_eq!(place["said"]["state"], "held");
    assert_eq!(place["heldBy"][0], "global · style.md");
    assert_eq!(card["here"], "applied");
    let global_card = card_of(&room.session, Some(&room.homes), HERE, global).unwrap();
    assert_eq!(
        serde_json::to_value(global_card.unwrap()).unwrap()["here"],
        "notAgreed"
    );
}

fn made_in_the_window(room: &mut Room) -> String {
    let root = root_of(&room.session.state).unwrap();
    let device = room.session.store.device().clone();
    let made = tisty_core::docs::create(&room.session.paths.docs(), &device, "").unwrap();
    room.session
        .commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                file: made.id.clone(),
                folder: Some(root),
                order: "z".to_string(),
                said: Some(tisty_core::event::Said::of("").agreed()),
                ..Default::default()
            },
        })
        .unwrap();
    made.id
}

#[test]
fn a_rule_the_person_makes_in_the_window_is_written_with_no_extra_click() {
    use crate::answers::papers::written_by_hand;
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let file = made_in_the_window(&mut room);
    let body = agents::doc_of(
        agents::GLOBAL,
        agents::Kind::Rule,
        "style.md",
        b"Spanish.\n",
    )
    .unwrap();

    written_by_hand(&mut room.session, file, body, None, None).unwrap();
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "Spanish.\n\n# Claude\nTests first.\n"
    );
}

#[test]
fn writing_over_an_agents_words_without_seeing_them_approves_nothing() {
    use crate::answers::papers::written_by_hand;
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let agents = body.replace("Tests first.", "Run rm -rf.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &agents).unwrap();
    room.session.retell(&file, &agents, None);

    let typed = agents.replace("# Claude", "# Claude rules");
    written_by_hand(&mut room.session, file.clone(), typed, None, None).unwrap();
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
}

#[test]
fn writing_anyway_approves_only_what_the_page_was_opened_on() {
    use crate::answers::papers::written_by_hand;
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, approved) = bodies(&room.session)[0].clone();
    let agents = approved.replace("Tests first.", "Run rm -rf.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &agents).unwrap();
    room.session.retell(&file, &agents, None);
    let opened_on = tisty_core::docs::print_of_text(&agents);
    tisty_core::docs::write(&room.session.paths.docs(), &file, &approved).unwrap();
    room.session.retell(&file, &approved, None);

    let typed = agents.replace("# Claude", "# Claude rules");
    written_by_hand(
        &mut room.session,
        file.clone(),
        typed,
        Some(true),
        Some(opened_on),
    )
    .unwrap();
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");

    let now = bodies(&room.session)[0].1.clone();
    let agents_again = now.replace("Run rm -rf.", "Run it twice.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &agents_again).unwrap();
    room.session.retell(&file, &agents_again, None);
    let mine = approved.replace("Tests first.", "Tests first, always.");
    let approved_print = tisty_core::docs::print_of_text(&approved);
    written_by_hand(
        &mut room.session,
        file,
        mine,
        Some(true),
        Some(approved_print),
    )
    .unwrap();
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "# Claude\nTests first, always.\n"
    );
}

#[test]
fn bringing_the_file_in_never_takes_away_words_written_after_the_card_was_drawn() {
    use crate::answers::agent_docs::{Brought, kept_in};
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let drawn = tisty_core::docs::print_of_text(&body);
    put(
        &room.home.join(".claude/CLAUDE.md"),
        "# Claude\nTests last.\n",
    );
    let typed = body.replace("Tests first.", "Tests first, just typed.");
    rewritten(&mut room.session, &file, &typed).unwrap();

    let from_file = body.replace("Tests first.", "Tests last.");
    let brought = Brought {
        file: &file,
        body: &from_file,
        by: "claude",
        seen: Some(&drawn),
    };
    let refused = kept_in(&mut room.session, Some(&room.homes), brought).unwrap_err();
    assert_eq!(refused.code, "agentDocChanged");
    assert!(bodies(&room.session)[0].1.contains("just typed"));
}

#[test]
fn what_the_person_let_in_here_stays_applied_after_another_computer_approves_a_change() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    let (file, body) = bodies(&room.session)[0].clone();
    assert!(Ledger::open(room.session.paths.data(), HERE).took(&file));

    let changed = body.replace("Tests first.", "Tests from the laptop.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &changed).unwrap();
    let id = room
        .session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
        .id;
    let laptop = tisty_core::DeviceId(tisty_core::config::new_device_id());
    tisty_core::Store::open(room.session.paths.store(), laptop)
        .unwrap()
        .append(Op::DocSaid {
            id,
            d: tisty_core::event::Said::of(&changed).agreed(),
        })
        .unwrap();
    room.session.reload().unwrap();

    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "# Claude\nTests from the laptop.\n"
    );
}

#[test]
fn a_document_an_import_left_here_unapproved_waits_and_is_never_taken() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let file = made_in_the_window(&mut room);
    let body = agents::doc_of(
        agents::GLOBAL,
        agents::Kind::Rule,
        "style.md",
        b"Spanish.\n",
    )
    .unwrap();
    tisty_core::docs::write(&room.session.paths.docs(), &file, &body).unwrap();
    room.session.retell(&file, &body, None);

    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert!(!Ledger::open(room.session.paths.data(), HERE).took(&file));
}

fn proposed(room: &mut Room, file: &str, body: &str) -> String {
    let id = room
        .session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
        .id;
    room.session
        .commit(Op::DocProposal {
            id,
            d: tisty_core::event::Proposal {
                body: Some(body.to_string()),
                base: tisty_core::docs::read(&room.session.paths.docs(), file)
                    .ok()
                    .map(|now| tisty_core::docs::print_of_text(&now)),
                settles: None,
            },
        })
        .unwrap();
    tisty_core::docs::print_of_text(body)
}

fn proposal_shown(room: &Room, file: &str) -> serde_json::Value {
    let card =
        crate::answers::agent_docs::card_of(&room.session, Some(&room.homes), HERE, file).unwrap();
    serde_json::to_value(card.unwrap()).unwrap()["proposal"].clone()
}

#[test]
fn a_proposal_waits_beside_the_approved_text_and_only_approving_writes_it() {
    use crate::answers::agent_proposals::approve;
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let print = proposed(
        &mut room,
        &file,
        &body.replace("Tests first.", "Tests always."),
    );
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);

    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert_eq!(room.look().proposed, 1);
    let shown = proposal_shown(&room, &file);
    assert_eq!(shown["print"], print.as_str());
    assert_eq!(shown["stale"], false);
    let lines = shown["lines"].as_array().unwrap();
    assert!(lines.contains(&serde_json::json!({ "side": "file", "text": "Tests always." })));
    assert!(lines.contains(&serde_json::json!({ "side": "tisty", "text": "Tests first." })));

    let over = tisty_core::docs::print_of_text(&body);
    approve(&mut room.session, &file, &print, &over).unwrap();
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests always.\n");

    let sent = body.replace(
        "Tests first.",
        "Tests always.\n\tRun [[Notes]] and #check **now**.",
    );
    let print = proposed(&mut room, &file, &sent);
    let over = tisty_core::docs::print_of_text(&body.replace("Tests first.", "Tests always."));
    approve(&mut room.session, &file, &print, &over).unwrap();
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(
        tisty_core::docs::read(&room.session.paths.docs(), &file).unwrap(),
        sent
    );
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "# Claude\nTests always.\n\tRun [[Notes]] and #check **now**.\n"
    );
    assert!(room.session.state.proposals.is_empty());
    assert_eq!(room.look().proposed, 0);
    assert_eq!(room.look().unagreed, 0);
}

#[test]
fn approving_takes_only_the_proposal_and_the_text_the_card_showed() {
    use crate::answers::agent_proposals::approve;
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let first = proposed(
        &mut room,
        &file,
        &body.replace("Tests first.", "Tests always."),
    );
    let over = tisty_core::docs::print_of_text(&body);
    let second = proposed(
        &mut room,
        &file,
        &body.replace("Tests first.", "Tests never."),
    );

    let refused = approve(&mut room.session, &file, &first, &over).unwrap_err();
    assert_eq!(refused.code, "agentProposalChanged");
    let typed = body.replace("# Claude", "# Claude rules");
    rewritten(&mut room.session, &file, &typed).unwrap();
    let refused = approve(&mut room.session, &file, &second, &over).unwrap_err();
    assert_eq!(refused.code, "agentDocChanged");
    assert_eq!(
        tisty_core::docs::read(&room.session.paths.docs(), &file).unwrap(),
        typed
    );
    assert_eq!(proposal_shown(&room, &file)["stale"], true);

    let now = tisty_core::docs::print_of_text(&typed);
    approve(&mut room.session, &file, &second, &now).unwrap();
    let written = tisty_core::docs::read(&room.session.paths.docs(), &file).unwrap();
    assert!(written.contains("Tests never."), "{written}");
}

#[test]
fn an_old_proposal_never_takes_back_the_agent_line_the_person_changed_since() {
    use crate::answers::agent_proposals::approve;
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    global_doc(
        &mut room,
        "Spanish.
",
    );
    let (file, body) = bodies(&room.session)
        .into_iter()
        .find(|(_, body)| body.contains("global/rule/style.md"))
        .unwrap();
    let print = proposed(
        &mut room,
        &file,
        &body.replace("Spanish.", "Spanish always."),
    );

    // Without its agent: line the document draws no card, so only approving is left to refuse.
    for (typed, drawn) in [
        (
            body.replace("global/rule/style.md", "claude/rule/style.md"),
            true,
        ),
        (
            body.replace(
                "agent: global/rule/style.md
",
                "",
            ),
            false,
        ),
    ] {
        rewritten(&mut room.session, &file, &typed).unwrap();
        let now = tisty_core::docs::print_of_text(&typed);
        let refused = approve(&mut room.session, &file, &print, &now).unwrap_err();
        assert_eq!(refused.code, "agentProposalRetargeted", "{typed}");
        assert_eq!(
            tisty_core::docs::read(&room.session.paths.docs(), &file).unwrap(),
            typed
        );
        if drawn {
            assert_eq!(
                proposal_shown(&room, &file)["unfit"],
                "agentProposalRetargeted"
            );
        }
    }
    assert!(!room.session.state.proposals.is_empty());
}

#[test]
fn an_old_proposal_that_no_longer_fits_the_text_as_it_is_now_is_only_discarded() {
    use crate::answers::agent_proposals::approve;
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let print = proposed(
        &mut room,
        &file,
        &body.replace("Tests first.", "Tests always."),
    );
    let long = "Line.\n".repeat(tisty_core::agents::PROPOSAL_LINES_AT_MOST);
    let typed = format!("{body}{long}");
    rewritten(&mut room.session, &file, &typed).unwrap();

    assert_eq!(proposal_shown(&room, &file)["unfit"], "agentProposalUnfit");
    let now = tisty_core::docs::print_of_text(&typed);
    let refused = approve(&mut room.session, &file, &print, &now).unwrap_err();
    assert_eq!(refused.code, "agentProposalUnfit");
    assert_eq!(
        tisty_core::docs::read(&room.session.paths.docs(), &file).unwrap(),
        typed
    );
}

#[test]
fn discarding_settles_only_the_proposal_the_card_showed_and_writes_nothing() {
    use crate::answers::agent_proposals::discard;
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let first = proposed(&mut room, &file, &body.replace("Tests first.", "One."));
    let second = proposed(&mut room, &file, &body.replace("Tests first.", "Two."));

    let refused = discard(&mut room.session, &file, &first).unwrap_err();
    assert_eq!(refused.code, "agentProposalChanged");
    discard(&mut room.session, &file, &second).unwrap();
    assert!(room.session.state.proposals.is_empty());
    assert!(proposal_shown(&room, &file).is_null());
    assert_eq!(
        tisty_core::docs::read(&room.session.paths.docs(), &file).unwrap(),
        body
    );
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
}

#[test]
fn a_proposal_over_a_text_the_person_never_approved_waits_for_that_text_first() {
    use crate::answers::agent_proposals::approve;
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let around = body.replace("Tests first.", "Run rm -rf.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &around).unwrap();
    room.session.retell(&file, &around, None);
    let print = proposed(
        &mut room,
        &file,
        &around.replace("# Claude", "# Claude rules"),
    );

    let over = tisty_core::docs::print_of_text(&around);
    let refused = approve(&mut room.session, &file, &print, &over).unwrap_err();
    assert_eq!(refused.code, "agentProposalOverUnagreed");
    assert_eq!(
        tisty_core::docs::read(&room.session.paths.docs(), &file).unwrap(),
        around
    );
    assert!(!room.session.state.proposals.is_empty());
}

#[test]
fn tags_an_earlier_build_took_from_an_agent_s_words_go_and_its_approval_stays() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let id = room
        .session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
        .id;
    let mut earlier = tisty_core::event::Said::of(&body);
    earlier.tags = Some(vec![tisty_core::Tag::new("always").unwrap()]);
    room.session.commit(Op::DocSaid { id, d: earlier }).unwrap();
    assert_eq!(room.session.state.docs[&id].tags.len(), 1);

    untagged(&mut room.session);
    assert!(room.session.state.docs[&id].tags.is_empty());
    let card = crate::answers::agent_docs::card_of(&room.session, Some(&room.homes), HERE, &file)
        .unwrap()
        .unwrap();
    assert_eq!(serde_json::to_value(card).unwrap()["here"], "applied");
}

#[test]
fn a_proposal_another_process_wrote_survives_a_window_write_made_before_it_read_again() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md", ".codex/AGENTS.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let id = room
        .session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
        .id;
    let mut beside = tisty_core::Store::open(
        room.session.paths.store(),
        tisty_core::DeviceId("dev_agent".into()),
    )
    .unwrap();
    beside
        .append(Op::DocProposal {
            id,
            d: tisty_core::event::Proposal {
                body: Some(body.replace("Tests first.", "Tests always.")),
                base: None,
                settles: None,
            },
        })
        .unwrap();

    let (other, other_body) = bodies(&room.session)[1].clone();
    let typed = other_body.replace("Codex rules.", "Codex rules, typed.");
    tisty_core::docs::write(&room.session.paths.docs(), &other, &typed).unwrap();
    room.session.retell(&other, &typed, None);

    let projected =
        tisty_core::cache::project(&room.session.paths.store(), room.session.paths.cache())
            .unwrap();
    assert!(projected.proposals.contains_key(&id));
    room.session.reload().unwrap();
    assert!(room.session.state.proposals.contains_key(&id));
}

#[test]
fn a_main_file_never_brought_can_be_brought_later_and_takes_the_global_rule() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    global_doc(&mut room, "Spanish.\n");
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    let told = round_now(&room.session, &room.homes, &mut ledger);
    assert!(
        told.both
            .iter()
            .any(|one| one.at == room.at(".codex/AGENTS.md")),
        "{told:?}"
    );
    let codex = room
        .look()
        .assistants
        .into_iter()
        .find(|one| one.id == "codex")
        .unwrap();
    assert!(!codex.files[0].held, "it can still be brought");

    let brought = room.bring(&[".codex/AGENTS.md"]);
    assert_eq!(brought.made, 1);
    assert!(
        brought
            .round
            .both
            .iter()
            .all(|one| !one.at.ends_with("AGENTS.md"))
    );
    assert_eq!(room.read(".codex/AGENTS.md"), "Spanish.\n\nCodex rules.\n");
    assert_eq!(room.kept(".codex/AGENTS.md"), 1);
    assert!(room.look().loose.is_empty());
}

#[test]
fn a_file_tisty_writes_is_shown_as_in_agents_and_never_brought_back_in() {
    let mut room = room();
    std::fs::remove_file(room.at(".copilot/copilot-instructions.md")).unwrap();
    room.bring(&[".claude/CLAUDE.md"]);
    global_doc(&mut room, "Spanish.\n");
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".copilot/copilot-instructions.md"), "Spanish.\n");
    let copilot = room
        .look()
        .assistants
        .into_iter()
        .find(|one| one.id == "copilot")
        .unwrap();
    let file = copilot
        .files
        .iter()
        .find(|one| one.name == "copilot-instructions.md")
        .unwrap();
    assert!(file.held);
    assert_eq!(room.bring(&[".copilot/copilot-instructions.md"]).made, 0);
}

#[test]
fn a_codex_override_that_appears_later_can_be_brought_and_takes_the_global_rule() {
    let mut room = room();
    room.bring(&[".codex/AGENTS.md"]);
    global_doc(&mut room, "Spanish.\n");
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".codex/AGENTS.md"), "Spanish.\n\nCodex rules.\n");

    put(
        Path::new(&room.at(".codex/AGENTS.override.md")),
        "Override.\n",
    );
    let told = round_now(&room.session, &room.homes, &mut ledger);
    assert!(
        told.both
            .iter()
            .any(|one| one.at == room.at(".codex/AGENTS.override.md")),
        "{told:?}"
    );
    assert!(
        told.loose.is_empty(),
        "AGENTS.md still holds the person's rule"
    );

    room.bring(&[".codex/AGENTS.override.md"]);
    assert_eq!(
        room.read(".codex/AGENTS.override.md"),
        "Spanish.\n\nOverride.\n"
    );
    assert_eq!(room.read(".codex/AGENTS.md"), "Codex rules.\n");
    let look = room.look();
    assert!(
        look.both.iter().all(|one| one.assistant != "codex") && look.loose.is_empty(),
        "{look:?}"
    );
}

#[test]
fn a_global_rule_moving_to_codexs_override_leaves_no_file_to_take_away() {
    let mut room = room();
    std::fs::remove_file(room.at(".codex/AGENTS.md")).unwrap();
    room.bring(&[".claude/CLAUDE.md"]);
    global_doc(&mut room, "Spanish.\n");
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".codex/AGENTS.md"), "Spanish.\n");
    put(
        Path::new(&room.at(".codex/AGENTS.override.md")),
        "Override.\n",
    );
    let told = round_now(&room.session, &room.homes, &mut ledger);
    assert!(told.loose.is_empty(), "{told:?}");
}

fn gemini_room() -> (Room, Ledger) {
    let mut room = room();
    put(Path::new(&room.at(".gemini/GEMINI.md")), "- remembered\n");
    room.bring(&[".claude/CLAUDE.md"]);
    global_doc(&mut room, "Spanish.\n");
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(
        room.read(".gemini/GEMINI.md"),
        "- remembered\n@tisty-rules.md\n"
    );
    (room, ledger)
}

#[test]
fn geminis_line_taken_out_by_the_person_is_never_put_back_alone() {
    let (mut room, mut ledger) = gemini_room();
    put(Path::new(&room.at(".gemini/GEMINI.md")), "- remembered\n");
    let global = file_brought_from(&room, "style.md");
    let body = bodies(&room.session)
        .into_iter()
        .find(|(file, _)| *file == global)
        .unwrap()
        .1
        .replace("Spanish.", "Spanish always.");
    rewritten(&mut room.session, &global, &body).unwrap();

    let told = round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(told.unled, ["gemini"]);
    assert!(told.told.iter().any(|one| one.standing == "unled"));
    assert_eq!(room.read(".gemini/GEMINI.md"), "- remembered\n");
    assert_eq!(room.read(".gemini/tisty-rules.md"), "Spanish.\n", "paused");
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "Spanish always.\n\n# Claude\nTests first.\n"
    );
    let look = room.look();
    assert_eq!(look.unled, ["gemini"]);
    assert!(look.asks);
    assert!(look.loose.is_empty());
    let card = crate::answers::agent_docs::card_of(&room.session, Some(&room.homes), HERE, &global)
        .unwrap();
    let places = serde_json::to_value(card.unwrap()).unwrap()["places"].clone();
    let gemini = places
        .as_array()
        .unwrap()
        .iter()
        .find(|one| one["assistant"] == "gemini")
        .unwrap()
        .clone();
    assert_eq!(gemini["said"]["state"], "unled");

    let again = led_again(&room.session, &room.homes, HERE).unwrap();
    assert!(again.unled.is_empty());
    assert_eq!(
        room.read(".gemini/GEMINI.md"),
        "- remembered\n@tisty-rules.md\n"
    );
    assert_eq!(room.read(".gemini/tisty-rules.md"), "Spanish always.\n");
}

#[test]
fn a_ledger_from_an_earlier_build_marks_geminis_line_without_writing() {
    let (room, _) = gemini_room();
    let data = room.session.paths.data();
    let mut ledger = Ledger::open(data, HERE);
    let gemini = Path::new(&room.at(".gemini/GEMINI.md")).to_path_buf();
    ledger.forget(&gemini);
    let kept = room.kept(".gemini/GEMINI.md");
    round_now(&room.session, &room.homes, &mut ledger);
    assert!(ledger.led(&gemini));
    assert!(Ledger::open(data, HERE).led(&gemini));
    assert_eq!(room.kept(".gemini/GEMINI.md"), kept);
}

#[test]
fn taking_away_geminis_file_of_rules_takes_its_line_too() {
    let (mut room, mut ledger) = gemini_room();
    let global = file_brought_from(&room, "style.md");
    room.session
        .drop_doc(&doc_of_file(&room, &global).to_string())
        .unwrap();
    let told = round_now(&room.session, &room.homes, &mut ledger);
    assert!(
        told.loose
            .iter()
            .any(|one| one.at == room.at(".gemini/tisty-rules.md")),
        "{told:?}"
    );
    answered(&mut room, ".gemini/tisty-rules.md", true);
    assert!(!Path::new(&room.at(".gemini/tisty-rules.md")).exists());
    assert_eq!(room.read(".gemini/GEMINI.md"), "- remembered\n");
    assert!(room.look().unled.is_empty());
}

#[test]
fn a_loose_file_still_undecided_is_offered_to_bring_not_shown_as_in_agents() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    moved_out(&mut room, &file);
    let look = room.look();
    assert_eq!(look.loose.len(), 1);
    let claude = look
        .assistants
        .into_iter()
        .find(|one| one.id == "claude")
        .unwrap();
    let tone = claude
        .files
        .iter()
        .find(|one| one.name == "rules/tone.md")
        .unwrap();
    assert!(!tone.held, "no document in AGENTS holds it any more");
}

#[test]
fn a_line_an_earlier_build_put_in_and_the_person_took_out_stays_out() {
    let (room, _) = gemini_room();
    let data = room.session.paths.data();
    let gemini = Path::new(&room.at(".gemini/GEMINI.md")).to_path_buf();
    let mut ledger = Ledger::open(data, HERE);
    let mut earlier = serde_json::to_value(&ledger).unwrap();
    earlier.as_object_mut().unwrap().remove("led");
    ledger = serde_json::from_value(earlier).unwrap();
    assert!(!ledger.led(&gemini) && ledger.written(&gemini).is_some());
    put(&gemini, "- remembered\n");
    let told = round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".gemini/GEMINI.md"), "- remembered\n");
    assert_eq!(told.unled, ["gemini"]);
}

#[test]
fn the_last_rule_leaving_asks_about_its_file_before_agents_goes() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    room.session
        .drop_doc(&doc_of_file(&room, &file).to_string())
        .unwrap();
    assert!(!emptied(&mut room.session, &room.homes, HERE).unwrap());
    assert!(!room.folders().is_empty());
    answered(&mut room, ".claude/rules/tone.md", true);
    assert!(!Path::new(&room.at(".claude/rules/tone.md")).exists());
    assert_eq!(room.kept(".claude/rules/tone.md"), 1);
    assert!(emptied(&mut room.session, &room.homes, HERE).unwrap());
    assert!(room.folders().is_empty(), "{:?}", room.folders());
}

#[test]
fn the_state_names_only_the_assistants_whose_folder_is_here() {
    let room = room();
    std::fs::write(room.home.join(".cursor"), "not a folder").unwrap();
    let state = stance(&room.session, &room.homes, HERE);
    let names: Vec<&str> = state
        .assistants
        .iter()
        .map(|one| one.name.as_str())
        .collect();
    assert_eq!(names, ["Claude", "Codex", "Copilot"]);
    assert_eq!(state.folder, None);
    assert!(!state.consented);
    assert!(state.all_on);
}

#[test]
fn the_state_follows_the_first_time_and_the_switch_without_touching_a_file() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let before = room.read(".claude/CLAUDE.md");
    let on = stance(&room.session, &room.homes, HERE);
    assert!(on.folder.is_some() && on.consented && on.all_on);
    turned(&room.session, &room.homes, HERE, None, false).unwrap();
    let off = stance(&room.session, &room.homes, HERE);
    assert_eq!(
        (off.folder, off.consented, off.all_on),
        (on.folder, true, false)
    );
    assert_eq!(room.read(".claude/CLAUDE.md"), before);
}

#[test]
fn the_state_says_which_assistant_is_paused_here() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    turned(&room.session, &room.homes, HERE, Some("claude"), false).unwrap();
    let on = |id: &str| {
        stance(&room.session, &room.homes, HERE)
            .assistants
            .iter()
            .find(|one| one.id == id)
            .map(|one| one.on)
    };
    assert_eq!((on("claude"), on("codex")), (Some(false), Some(true)));
    turned(&room.session, &room.homes, HERE, Some("claude"), true).unwrap();
    assert_eq!(on("claude"), Some(true));
}

#[test]
fn another_computer_has_not_said_yes_for_this_one() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let there = stance(&room.session, &room.homes, "another-computer");
    assert!(there.folder.is_some());
    assert!(!there.consented);
    assert_eq!(there.held, 1, "brought there, so brought here too");
}

fn snapshot(at: &Path, into: &mut BTreeMap<PathBuf, Vec<u8>>) {
    for one in std::fs::read_dir(at).unwrap().flatten() {
        let path = one.path();
        if path.is_dir() {
            snapshot(&path, into);
        } else {
            into.insert(path.clone(), std::fs::read(&path).unwrap());
        }
    }
}

fn every_agent_file(room: &Room) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut all = BTreeMap::new();
    snapshot(&room.home, &mut all);
    all
}

const MAIN_FILES: [&str; 4] = [
    ".claude/CLAUDE.md",
    ".codex/AGENTS.md",
    ".copilot/copilot-instructions.md",
    ".gemini/GEMINI.md",
];

#[test]
fn bringing_alone_writes_no_agent_file_and_leaves_nothing_that_writes_later() {
    let mut room = room();
    put(Path::new(&room.at(".gemini/GEMINI.md")), "- remembered\n");
    let before = every_agent_file(&room);

    let brought = room.bring_as(&[".claude/CLAUDE.md", ".claude/rules/tone.md"], Some(false));
    assert_eq!(brought.made, 2);
    assert!(!brought.syncs);
    assert!(brought.round.both.is_empty() && brought.round.loose.is_empty());
    // A Global rule is what would reach every assistant at once if the folder synced.
    global_doc(&mut room, "Spanish.\n");

    let data = room.session.paths.data().to_path_buf();
    let mut ledger = Ledger::open(&data, HERE);
    assert!(ledger.consented() && !ledger.syncs(""));
    let round = round_now(&room.session, &room.homes, &mut ledger);
    assert!(
        round.told.iter().all(|one| one.standing == "off"),
        "{round:?}"
    );
    assert!(round.unled.is_empty() && round.loose.is_empty() && round.both.is_empty());
    led_again(&room.session, &room.homes, HERE).unwrap();
    let claude = room.at(".claude/CLAUDE.md");
    let _ = kept_side(&mut room.session, &room.homes, HERE, &claude, true);
    let codex = room.at(".codex/AGENTS.md");
    let _ = loosened(&room.session, &room.homes, HERE, &codex, true);
    let gate = gated(&room.session, &ledger);
    for one in places::targets(&room.homes, &gate, &ledger) {
        assert!(agents::overwrite(&data, &mut ledger, &one.target).is_err());
        if let Some(lead) = &one.lead_in {
            assert!(agents::lead_in(&data, &ledger, lead, &places::lead_in_line()).is_err());
        }
    }

    assert_eq!(every_agent_file(&room), before);
    for name in MAIN_FILES {
        assert_eq!(room.kept(name), 0, "{name}");
    }
    let look = room.look();
    assert!(
        !look.asks,
        "a folder brought but not synced opens like any other"
    );
    assert_eq!(look.docs, 3);
    let state = stance(&room.session, &room.homes, HERE);
    assert!(state.folder.is_some() && state.consented && !state.all_on);

    // «Sync» afterwards is the same switch as before, and only then is anything written.
    turned(&room.session, &room.homes, HERE, None, true).unwrap();
    assert_eq!(
        room.read(".gemini/GEMINI.md"),
        "- remembered\n@tisty-rules.md\n"
    );
    assert_eq!(room.read(".gemini/tisty-rules.md"), "Spanish.\n");
}

#[test]
fn bringing_another_file_keeps_the_switch_as_it_was() {
    let mut room = room();
    room.bring_as(&[".claude/CLAUDE.md"], Some(false));
    let more = room.bring_as(&[".codex/AGENTS.md"], None);
    assert_eq!(more.made, 1);
    assert!(!more.syncs, "bringing one more file never starts syncing");
    assert!(!stance(&room.session, &room.homes, HERE).all_on);

    let mut synced = self::room();
    synced.bring_as(&[".claude/CLAUDE.md"], Some(true));
    let more = synced.bring_as(&[".codex/AGENTS.md"], None);
    assert!(more.syncs, "nor stops it");
    assert!(stance(&synced.session, &synced.homes, HERE).all_on);
}

#[test]
fn a_first_yes_without_the_word_sync_never_syncs() {
    let mut room = room();
    let brought = room.bring_as(&[".claude/CLAUDE.md"], None);
    assert!(!brought.syncs);
    assert!(!Ledger::open(room.session.paths.data(), HERE).syncs(""));
}

#[test]
fn bring_and_sync_turns_back_on_a_folder_that_was_brought_alone() {
    let mut room = room();
    room.bring_as(&[".claude/CLAUDE.md"], Some(false));
    let brought = room.bring_as(&[], Some(true));
    assert!(brought.syncs);
    assert!(stance(&room.session, &room.homes, HERE).all_on);
}

// What #882 left on disk: the same ledger, with the folder switched off or on.
fn ledger_of_882(room: &Room, all_off: bool) {
    let at = agents::folder(room.session.paths.data()).join("ledger.json");
    let said = serde_json::json!({ "machine": HERE, "consented": true, "all_off": all_off });
    std::fs::write(at, serde_json::to_vec(&said).unwrap()).unwrap();
}

#[test]
fn an_agents_882_left_off_with_documents_stays_brought_and_never_syncs() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    ledger_of_882(&room, true);
    let before = every_agent_file(&room);

    let state = stance(&room.session, &room.homes, HERE);
    assert!(state.folder.is_some() && state.consented && !state.all_on);
    assert_eq!(state.held, 1);
    let look = room.look();
    assert_eq!(look.docs, 1);
    assert!(!look.asks);
    let more = room.bring_as(&[".codex/AGENTS.md"], None);
    assert!(!more.syncs);
    assert_eq!(every_agent_file(&room), before);
}

#[test]
fn an_agents_882_left_on_keeps_syncing() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    ledger_of_882(&room, false);

    let state = stance(&room.session, &room.homes, HERE);
    assert!(state.folder.is_some() && state.consented && state.all_on);
    let more = room.bring_as(&[".codex/AGENTS.md"], None);
    assert!(more.syncs);
}

#[test]
fn a_document_put_away_does_not_count_as_brought() {
    let mut room = room();
    room.bring_as(&[".claude/CLAUDE.md"], Some(false));
    assert_eq!(stance(&room.session, &room.homes, HERE).held, 1);
    let id = room.session.state.docs.values().next().unwrap().id;
    room.session.commit(Op::DocArchive { id }).unwrap();
    assert_eq!(stance(&room.session, &room.homes, HERE).held, 0);
}

// A Global rule from the laptop, seen once by a round here, so this computer is asked about it.
fn waiting_from_the_laptop(room: &mut Room) -> (String, String) {
    room.bring(&[".claude/CLAUDE.md"]);
    let file = from_another_computer(room, agents::Kind::Rule, "style.md", "Spanish.\n");
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.look().waiting, 1);
    let body = tisty_core::docs::read(&room.session.paths.docs(), &file).unwrap();
    (file, body)
}

fn still_waits_then_applies(room: &mut Room, file: &str, said: &str) {
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert_eq!(room.look().waiting, 1, "the card still offers «apply here»");
    let card = crate::answers::agent_docs::card_of(&room.session, Some(&room.homes), HERE, file)
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::to_value(card).unwrap()["here"],
        "approvedWaiting"
    );

    crate::answers::agent_docs::take_here(&room.session, Some(&room.homes), HERE, file, true)
        .unwrap();
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        format!("{said}\n# Claude\nTests first.\n")
    );
}

#[test]
fn approving_a_proposal_on_a_second_computer_changes_the_words_but_never_applies_them_there() {
    use crate::answers::agent_proposals::approve;
    let mut room = room();
    let (file, body) = waiting_from_the_laptop(&mut room);
    let print = proposed(
        &mut room,
        &file,
        &body.replace("Spanish.", "Always Spanish."),
    );

    let over = tisty_core::docs::print_of_text(&body);
    approve(&mut room.session, &file, &print, &over).unwrap();

    still_waits_then_applies(&mut room, &file, "Always Spanish.\n");
}

#[test]
fn approving_this_text_on_a_second_computer_never_applies_it_there() {
    use crate::answers::agent_docs::agree;
    let mut room = room();
    let (file, body) = waiting_from_the_laptop(&mut room);
    let unapproved = body.replace("Spanish.", "Spanish, briefly.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &unapproved).unwrap();
    room.session.retell(&file, &unapproved, None);
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.look().unagreed, 1);

    let print = tisty_core::docs::print_of_text(&unapproved);
    agree(&mut room.session, Some(&room.homes), HERE, &file, &print).unwrap();

    assert_eq!(room.look().unagreed, 0);
    still_waits_then_applies(&mut room, &file, "Spanish, briefly.\n");
}

#[test]
fn restoring_a_version_on_a_second_computer_never_applies_it_there() {
    let mut room = room();
    let (file, body) = waiting_from_the_laptop(&mut room);
    let earlier = body.replace("Spanish.", "Spanish, as it was.");

    crate::answers::earlier::put_back(&mut room.session, &file, &earlier).unwrap();

    still_waits_then_applies(&mut room, &file, "Spanish, as it was.\n");
}

#[test]
fn a_document_is_marked_to_wait_before_its_words_are_approved_even_with_no_round_yet() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let file = from_another_computer(&mut room, agents::Kind::Rule, "style.md", "Spanish.\n");
    let data = room.session.paths.data().to_path_buf();

    waits_marked(&room.session, "another-computer", &file);
    assert!(
        Ledger::open(&data, HERE).consented(),
        "another computer's ledger is never written over"
    );
    assert!(!Ledger::open(&data, HERE).waits(&file));

    waits_marked(&room.session, HERE, &file);
    assert!(Ledger::open(&data, HERE).waits(&file));

    let (mine, _) = bodies(&room.session)
        .into_iter()
        .find(|(one, _)| *one != file)
        .unwrap();
    waits_marked(&room.session, HERE, &mine);
    assert!(!Ledger::open(&data, HERE).waits(&mine));
}

// The laptop made its own AGENTS, with the smaller id, before the two computers ever synced.
fn laptops_own_agents(room: &mut Room) -> String {
    let laptop = tisty_core::DeviceId(tisty_core::config::new_device_id());
    let root = ulid::Ulid::from_parts(1, 1);
    let claude = ulid::Ulid::from_parts(1, 2);
    let body = agents::doc_of(
        "claude",
        agents::Kind::Rule,
        "CLAUDE.md",
        b"Laptop rules.\n",
    )
    .unwrap();
    let made = tisty_core::docs::create(&room.session.paths.docs(), &laptop, &body).unwrap();
    let folder = |id, name: &str, parent| Op::FolderAdd {
        id,
        d: tisty_core::event::FolderAdd {
            name: name.to_string(),
            order: "a0".to_string(),
            parent,
            icon: parent.is_none().then(|| places::ICON.to_string()),
            color: None,
        },
    };
    tisty_core::Store::open(room.session.paths.store(), laptop)
        .unwrap()
        .append_batch(vec![
            folder(root, places::FOLDER, None),
            folder(claude, "Claude", Some(root)),
            Op::DocAdd {
                id: ulid::Ulid::generate(),
                d: tisty_core::event::DocAdd {
                    file: made.id.clone(),
                    folder: Some(claude),
                    order: "a0".to_string(),
                    said: Some(tisty_core::event::Said::of(&body).agreed()),
                    ..Default::default()
                },
            },
        ])
        .unwrap();
    room.session.reload().unwrap();
    made.id
}

#[test]
fn two_agents_join_without_touching_a_file_offering_one_loose_or_losing_an_approval() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md", ".codex/AGENTS.md"]);
    let data = room.session.paths.data().to_path_buf();
    let mine: Vec<String> = bodies(&room.session)
        .into_iter()
        .map(|(file, _)| file)
        .collect();
    let before = Ledger::open(&data, HERE);
    let backups = room.kept(".claude/CLAUDE.md");

    let laptops = laptops_own_agents(&mut room);
    assert_eq!(agents::roots(&room.session.state).len(), 2);
    assert!(
        !room.look().loose.is_empty(),
        "before joining, this computer's own files read as left by every rule"
    );

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(agents::roots(&room.session.state).len(), 1);
    assert!(room.look().loose.is_empty());
    let held: Vec<String> = bodies(&room.session)
        .into_iter()
        .map(|(file, _)| file)
        .collect();
    assert!(mine.iter().all(|file| held.contains(file)));
    assert!(held.contains(&laptops));
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert_eq!(room.read(".codex/AGENTS.md"), "Codex rules.\n");
    assert_eq!(room.kept(".claude/CLAUDE.md"), backups);
    let after = Ledger::open(&data, HERE);
    assert!(
        mine.iter()
            .all(|file| after.took(file) == before.took(file))
    );
    assert_eq!(after.applied(), before.applied());
    assert_eq!(
        gated(&room.session, &after).seen(&laptops),
        Some(Seen::WaitingHere),
        "the laptop's twin of CLAUDE.md waits here like any rule from elsewhere"
    );
    let mut names: Vec<String> = room.folders();
    names.dedup();
    assert_eq!(names, room.folders(), "no folder is there twice");
}

#[test]
fn a_proposal_on_a_version_restored_without_its_card_stops_counting_and_is_let_go() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    proposed(
        &mut room,
        &file,
        &body.replace("Tests first.", "Tests always."),
    );
    assert_eq!(room.look().proposed, 1);

    crate::answers::earlier::put_back(&mut room.session, &file, "# Claude\nTests first.\n")
        .unwrap();
    assert_eq!(
        room.look().proposed,
        0,
        "a proposal with no card to show it on is not counted"
    );

    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert!(room.session.state.proposals.is_empty());
}

#[test]
fn a_proposal_whose_document_cannot_be_read_is_kept_for_later() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    proposed(
        &mut room,
        &file,
        &body.replace("Tests first.", "Tests always."),
    );
    let at = tisty_core::docs::resolve(&room.session.paths.docs(), &file).unwrap();
    std::fs::remove_file(at).unwrap();

    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert_eq!(room.session.state.proposals.len(), 1);
}

#[test]
fn a_body_on_disk_behind_the_log_never_lets_a_proposal_go() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    proposed(
        &mut room,
        &file,
        &body.replace("Tests first.", "Tests always."),
    );
    tisty_core::docs::write(
        &room.session.paths.docs(),
        &file,
        "# Claude\nAn older copy.\n",
    )
    .unwrap();

    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert_eq!(room.session.state.proposals.len(), 1);
}

#[test]
fn a_file_only_brought_is_never_recorded_as_written_nor_offered_to_take_away() {
    let mut room = room();
    room.bring_as(&[".claude/CLAUDE.md"], Some(false));
    let data = room.session.paths.data().to_path_buf();
    let claude = PathBuf::from(room.at(".claude/CLAUDE.md"));
    assert_eq!(Ledger::open(&data, HERE).written(&claude), None);

    let file = file_brought_from(&room, "CLAUDE.md");
    moved_out(&mut room, &file);
    turned(&room.session, &room.homes, HERE, None, true).unwrap();

    assert!(
        loose(&room).is_empty(),
        "a file Tisty never wrote is never offered"
    );
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
}

#[test]
fn geminis_memory_taken_out_never_reads_as_a_line_the_person_took_away() {
    let mut room = room();
    put(Path::new(&room.at(".gemini/GEMINI.md")), "- remembered\n");
    room.bring(&[".claude/CLAUDE.md", ".gemini/GEMINI.md"]);
    let memory = file_brought_from(&room, "GEMINI.md");
    moved_out(&mut room, &memory);
    global_doc(&mut room, "Spanish.\n");

    let round = round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert!(round.unled.is_empty(), "{round:?}");
    assert!(room.look().unled.is_empty());
    assert!(room.read(".gemini/GEMINI.md").contains("@tisty-rules.md"));
    assert!(room.read(".gemini/GEMINI.md").starts_with("- remembered\n"));
}

#[test]
fn a_file_brought_alone_is_taken_as_it_stands_once_the_folder_syncs() {
    let mut room = room();
    room.bring_as(&[".claude/CLAUDE.md"], Some(false));
    global_doc(&mut room, "Spanish.\n");

    let round = turned(&room.session, &room.homes, HERE, None, true).unwrap();

    assert!(
        round.both.iter().all(|one| one.assistant != "claude"),
        "the file brought stands for itself: {round:?}"
    );
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "Spanish.\n\n# Claude\nTests first.\n"
    );
    assert_eq!(room.kept(".claude/CLAUDE.md"), 1);
}

#[test]
fn approving_this_text_first_shows_what_changed_since_the_last_approval_and_whose_hand() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let later = body.replace("Tests first.", "Tests later.");
    tisty_core::history::Writer::unanswered(room.session.paths.data())
        .replaces(
            &file,
            body.as_bytes(),
            &later,
            tisty_core::history::Step::Stays,
        )
        .unwrap();
    tisty_core::docs::write(&room.session.paths.docs(), &file, &later).unwrap();
    room.session.retell(&file, &later, None);

    let card =
        crate::answers::agent_docs::card_of(&room.session, Some(&room.homes), HERE, &file).unwrap();
    let shown = serde_json::to_value(card.unwrap()).unwrap()["agreeing"].clone();
    assert_eq!(shown["hand"], "here");
    assert_eq!(shown["approved"], "kept");
    let lines = shown["lines"].as_array().unwrap();
    assert!(lines.contains(&serde_json::json!({ "side": "tisty", "text": "Tests first." })));
    assert!(lines.contains(&serde_json::json!({ "side": "file", "text": "Tests later." })));

    let print = tisty_core::docs::print_of_text(&later);
    crate::answers::agent_docs::agree(&mut room.session, Some(&room.homes), HERE, &file, &print)
        .unwrap();
    let agreed =
        crate::answers::agent_docs::card_of(&room.session, Some(&room.homes), HERE, &file).unwrap();
    assert!(serde_json::to_value(agreed.unwrap()).unwrap()["agreeing"].is_null());
}

#[test]
fn words_an_agent_on_another_computer_wrote_are_named_as_an_agents_before_approving() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let (file, body) = bodies(&room.session)[0].clone();
    let id = doc_of_file(&room, &file);
    let theirs = body.replace("Tests first.", "Run rm -rf.");
    let agent = tisty_core::DeviceId(tisty_core::config::new_device_id());
    tisty_core::Store::open(room.session.paths.store(), agent.clone())
        .unwrap()
        .append_batch(vec![
            Op::DeviceJoin {
                d: agent,
                k: Some(tisty_core::event::DeviceKind::Agent),
                p: None,
            },
            Op::DocSaid {
                id,
                d: tisty_core::event::Said::of(&theirs).agreed(),
            },
        ])
        .unwrap();
    tisty_core::docs::write(&room.session.paths.docs(), &file, &theirs).unwrap();
    room.session.reload().unwrap();

    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    let card =
        crate::answers::agent_docs::card_of(&room.session, Some(&room.homes), HERE, &file).unwrap();
    let shown = serde_json::to_value(card.unwrap()).unwrap();
    assert_eq!(shown["here"], "notAgreed");
    assert_eq!(shown["agreeing"]["hand"], "agent");
}

fn sub_of(room: &Room, name: &str) -> tisty_core::model::FolderId {
    let root = root_of(&room.session.state).unwrap();
    room.session
        .state
        .folders
        .values()
        .find(|one| one.parent == Some(root) && one.name == name)
        .unwrap_or_else(|| panic!("no folder {name}"))
        .id
}

#[test]
fn every_assistant_found_has_its_folder_and_a_rule_written_there_reaches_it() {
    let mut room = room();
    std::fs::create_dir_all(room.home.join(".gemini/history")).unwrap();
    room.bring(&[".claude/CLAUDE.md"]);
    let names = room.folders();
    for one in ["Global", "Claude", "Codex", "Copilot", "Gemini"] {
        assert!(names.contains(&one.to_string()), "{one}: {names:?}");
    }
    assert!(!Path::new(&room.at(".gemini/tisty-rules.md")).exists());
    assert!(!Path::new(&room.at(".gemini/GEMINI.md")).exists());

    let gemini = sub_of(&room, "Gemini");
    let seed = seeded(&room.session, Some(gemini));
    assert_eq!(
        Card::of(&seed).unwrap().address(),
        "gemini/rule/tisty-rules.md"
    );
    let file = made_in_the_window(&mut room);
    crate::answers::papers::written_by_hand(
        &mut room.session,
        file,
        format!("{seed}Spanish.\n"),
        None,
        None,
    )
    .unwrap();
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert_eq!(room.read(".gemini/tisty-rules.md"), "Spanish.\n");
}

#[test]
fn a_new_document_starts_as_a_rule_of_the_folder_it_is_made_in() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let address = |room: &Room, name: &str| {
        Card::of(&seeded(&room.session, Some(sub_of(room, name)))).map(|card| card.address())
    };
    assert_eq!(
        address(&room, "Global").as_deref(),
        Some("global/rule/rule.md")
    );
    assert_eq!(
        address(&room, "Claude").as_deref(),
        Some("claude/rule/rules/rule.md")
    );
    assert_eq!(
        address(&room, "Codex").as_deref(),
        Some("codex/rule/AGENTS.md")
    );
    assert_eq!(
        address(&room, "Copilot").as_deref(),
        Some("copilot/rule/copilot-instructions.md")
    );
    assert_eq!(seeded(&room.session, None), "");
    let root = root_of(&room.session.state).unwrap();
    assert_eq!(seeded(&room.session, Some(root)), "");
}

#[test]
fn a_folder_of_agents_the_person_deleted_is_never_made_again() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let codex = sub_of(&room, "Codex");
    room.session.commit(Op::FolderDelete { id: codex }).unwrap();
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert!(!room.folders().contains(&"Codex".to_string()));
}

#[test]
fn deleting_an_assistants_folder_asks_about_the_files_its_rules_wrote() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md", ".claude/rules/tone.md"]);
    let claude = sub_of(&room, "Claude");
    room.session
        .commit(Op::FolderDelete { id: claude })
        .unwrap();
    let told = round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert!(
        told.loose
            .iter()
            .any(|one| one.at == room.at(".claude/rules/tone.md")),
        "{told:?}"
    );
    assert!(!room.folders().contains(&"Claude".to_string()));
    answered(&mut room, ".claude/rules/tone.md", true);
    assert_eq!(room.kept(".claude/rules/tone.md"), 1);
}

#[test]
fn looking_again_finds_an_assistant_installed_since_and_gives_it_its_folder() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    assert!(!room.folders().contains(&"Gemini".to_string()));
    std::fs::create_dir_all(room.home.join(".gemini/tmp")).unwrap();

    let look = rescanned(&mut room.session, &room.homes, HERE).unwrap();

    assert!(room.folders().contains(&"Gemini".to_string()));
    let gemini = look
        .assistants
        .iter()
        .find(|one| one.id == "gemini")
        .unwrap();
    assert!(
        gemini.files.is_empty(),
        "nothing of Gemini's is read or brought"
    );
}

#[test]
fn looking_again_where_this_computer_never_said_yes_reads_nothing_and_makes_nothing() {
    let mut room = room();
    let look = rescanned(&mut room.session, &room.homes, HERE).unwrap();
    assert!(!look.consented);
    assert!(room.folders().is_empty());
}

#[test]
fn a_new_global_rule_still_empty_touches_no_agent_file() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let backups = room.kept(".claude/CLAUDE.md");
    let seed = seeded(&room.session, Some(sub_of(&room, "Global")));
    let file = made_in_the_window(&mut room);
    crate::answers::papers::written_by_hand(&mut room.session, file, seed, None, None).unwrap();

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert_eq!(room.kept(".claude/CLAUDE.md"), backups);
}

#[test]
fn a_global_rule_emptied_leaves_every_file_and_asks_about_one_it_alone_wrote() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    global_doc(&mut room, "Spanish.\n");
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "Spanish.\n\n# Claude\nTests first.\n"
    );
    let gemini = room.home.join(".gemini");
    std::fs::create_dir_all(&gemini).unwrap();
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert_eq!(room.read(".gemini/tisty-rules.md"), "Spanish.\n");

    let (file, body) = bodies(&room.session)
        .into_iter()
        .find(|(_, body)| body.contains("Spanish."))
        .unwrap();
    let emptied = body.replace("Spanish.\n", "");
    crate::answers::papers::written_by_hand(&mut room.session, file, emptied, None, None).unwrap();
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert!(
        loose(&room).contains(&room.at(".gemini/tisty-rules.md")),
        "the file the rule alone made is asked about, never left saying the old words in silence"
    );
}

#[test]
fn a_new_document_in_codex_or_gemini_opens_its_one_rules_file_once_there_is_one() {
    use crate::answers::papers::doc_made;
    let mut room = room();
    std::fs::create_dir_all(room.home.join(".gemini")).unwrap();
    room.bring(&[".codex/AGENTS.md"]);
    let codex_rule = file_brought_from(&room, "AGENTS.md");
    let count = room.session.state.docs.len();

    let codex = sub_of(&room, "Codex");
    let opened = doc_made(&mut room.session, Some(codex), None, true).unwrap();
    assert_eq!(opened.id, codex_rule);
    assert_eq!(room.session.state.docs.len(), count, "nothing new is made");

    let gemini = sub_of(&room, "Gemini");
    let first = doc_made(&mut room.session, Some(gemini), None, true).unwrap();
    let body = tisty_core::docs::read(&room.session.paths.docs(), &first.id).unwrap();
    assert_eq!(
        Card::of(&body).unwrap().address(),
        "gemini/rule/tisty-rules.md"
    );
    let again = doc_made(&mut room.session, Some(gemini), None, true).unwrap();
    assert_eq!(again.id, first.id);

    let claude = sub_of(&room, "Claude");
    let one = doc_made(&mut room.session, Some(claude), None, true).unwrap();
    let two = doc_made(&mut room.session, Some(claude), None, true).unwrap();
    assert_ne!(one.id, two.id, "Claude reads a whole folder of rules");

    let written_over = doc_made(&mut room.session, Some(gemini), None, false).unwrap();
    assert_ne!(
        written_over.id, first.id,
        "a caller that writes its own words gets a new one"
    );
}

#[test]
fn two_global_rules_named_alike_on_two_computers_both_reach_the_agents() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    global_doc(&mut room, "Spanish.\n");
    let theirs = from_another_computer_for(
        &mut room,
        agents::GLOBAL,
        agents::Kind::Rule,
        "style.md",
        "Short answers.\n",
    );

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    let addresses: BTreeSet<String> = bodies(&room.session)
        .iter()
        .filter_map(|(_, body)| Card::of(body).map(|card| card.address()))
        .filter(|address| address.starts_with("global/"))
        .collect();
    assert_eq!(
        addresses,
        ["global/rule/style-2.md", "global/rule/style.md"]
            .map(String::from)
            .into()
    );
    assert_eq!(
        room.look().waiting,
        1,
        "renaming the other computer's rule never applies it here"
    );
    crate::answers::agent_docs::take_here(&room.session, Some(&room.homes), HERE, &theirs, true)
        .unwrap();
    let claude = room.read(".claude/CLAUDE.md");
    assert!(
        claude.contains("Spanish.") && claude.contains("Short answers."),
        "{claude}"
    );
}
