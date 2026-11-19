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
            &gathered(&self.session, HERE),
            &self.homes,
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
