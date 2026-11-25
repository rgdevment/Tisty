use super::*;
use crate::answers::agents_sync::tests::{
    HERE, Room, doc_here, doc_of_file, every_agent_file, file_brought_from,
    from_another_computer_for, loose, made_in, proposed, put, room, root_folder, sub_of,
};
use crate::answers::agents_sync::{bodies, round_here, round_now};
use tisty_core::agents::Kind;

const MAIN: &str = "# Claude\nTests first.\n";
const BOTH: &str = "Spanish.\n\n# Claude\nTests first.\n";

pub(crate) fn me(room: &Room) -> DeviceId {
    room.session.store.device().clone()
}

pub(crate) fn laptop() -> DeviceId {
    DeviceId(tisty_core::config::new_device_id())
}

pub(crate) fn as_laptop(room: &mut Room, laptop: &DeviceId, ops: Vec<Op>) {
    tisty_core::Store::open(room.session.paths.store(), laptop.clone())
        .unwrap()
        .append_batch(ops)
        .unwrap();
    room.session.reload().unwrap();
}

pub(crate) fn root(room: &Room) -> FolderId {
    root_of(&room.session.state).unwrap()
}

pub(crate) fn my_folder(room: &Room) -> FolderId {
    let (root, me) = (root(room), me(room));
    room.session
        .state
        .folders
        .values()
        .find(|one| one.parent == Some(root) && one.computer.as_ref() == Some(&me))
        .unwrap()
        .id
}

pub(crate) fn laptops_folder(room: &mut Room, laptop: &DeviceId) -> FolderId {
    let id = ulid::Ulid::generate();
    let made = Op::FolderAdd {
        id,
        d: tisty_core::event::FolderAdd {
            name: "laptop".to_string(),
            order: "z".to_string(),
            parent: Some(root(room)),
            icon: None,
            color: None,
            computer: Some(laptop.clone()),
        },
    };
    as_laptop(room, laptop, vec![made]);
    id
}

pub(crate) fn body_for(level: &str, name: &str, said: &str, computer: Option<&DeviceId>) -> String {
    let body = agents::doc_of(level, Kind::Rule, name, said.as_bytes()).unwrap();
    agents::meant_for(&body, computer).unwrap()
}

fn added(file: &str, folder: FolderId, body: &str) -> Op {
    Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: DocAdd {
            file: file.to_string(),
            folder: Some(folder),
            order: "a".to_string(),
            said: Some(Said::of(body).agreed()),
            ..Default::default()
        },
    }
}

// Approved by a Tisty that did not know computers, as 1.50 left it.
fn old_mine_in(room: &mut Room, folder: FolderId, body: &str) -> String {
    let made = tisty_core::docs::create(&room.session.paths.docs(), &me(room), body).unwrap();
    let mut said = Said::of(body).agreed();
    said.knows_computers = false;
    let op = Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: DocAdd {
            file: made.id.clone(),
            folder: Some(folder),
            order: "a".to_string(),
            said: Some(said),
            ..Default::default()
        },
    };
    room.session.commit(op).unwrap();
    made.id
}

pub(crate) fn mine_in(room: &mut Room, folder: FolderId, body: &str) -> String {
    let made = tisty_core::docs::create(&room.session.paths.docs(), &me(room), body).unwrap();
    room.session.commit(added(&made.id, folder, body)).unwrap();
    made.id
}

pub(crate) fn laptops_in(
    room: &mut Room,
    laptop: &DeviceId,
    folder: FolderId,
    body: &str,
) -> String {
    let made = tisty_core::docs::create(&room.session.paths.docs(), laptop, body).unwrap();
    as_laptop(room, laptop, vec![added(&made.id, folder, body)]);
    made.id
}

fn kept<'a>(room: &'a Room, file: &str) -> &'a tisty_core::model::Kept {
    room.session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
}

fn card_of(room: &Room, file: &str) -> Card {
    Card::of(&read(&room.session, file).unwrap()).unwrap()
}

fn shown(room: &Room, file: &str) -> serde_json::Value {
    let card = crate::answers::agent_docs::card_of(&room.session, Some(&room.homes), HERE, file);
    serde_json::to_value(card.unwrap().unwrap()).unwrap()
}

pub(crate) fn ledger(room: &Room) -> Ledger {
    Ledger::open(room.session.paths.data(), HERE)
}

fn stance_of(room: &Room, file: &str) -> places::Stance {
    let ledger = ledger(room);
    places::stance(&room.homes, &gated(&room.session, &ledger), &ledger, file)
}

fn seen(room: &Room, file: &str) -> Option<Seen> {
    gated(&room.session, &ledger(room)).seen(file)
}

fn computer_of(room: &Room, file: &str) -> Option<DeviceId> {
    agents::computer_of(&room.session.state, kept(room, file).folder)
}

fn moved_to(room: &mut Room, file: &str, folder: FolderId) {
    let id = doc_of_file(room, file);
    room.session
        .commit(Op::DocMove {
            id,
            d: Filed {
                folder: Some(Some(folder)),
                page_of: None,
                order: None,
            },
        })
        .unwrap();
    let homes = room.homes.clone();
    moved(&mut room.session, Some(&homes), HERE, &[file.to_string()]).unwrap();
}

fn take(room: &Room, file: &str, here: bool) {
    crate::answers::agent_docs::take_here(&room.session, Some(&room.homes), HERE, file, here)
        .unwrap();
}

#[test]
fn what_is_brought_goes_to_this_computers_folder_marked_for_it() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md", ".claude/rules/tone.md"]);
    let me = me(&room);
    for name in ["CLAUDE.md", "rules/tone.md"] {
        let file = file_brought_from(&room, name);
        assert_eq!(card_of(&room, &file).computer, Some(me.clone()), "{name}");
        assert_eq!(computer_of(&room, &file), Some(me.clone()), "{name}");
    }
    let state = &room.session.state;
    let mut at_root: Vec<&str> = state
        .folders
        .values()
        .filter(|one| one.parent == Some(root(&room)))
        .map(|one| match one.computer {
            Some(_) => "<computer>",
            None => one.name.as_str(),
        })
        .collect();
    at_root.sort();
    assert_eq!(at_root, ["<computer>", "Global"]);
}

#[test]
fn empty_agent_folders_go_to_this_computers_folder_and_only_global_stays_at_the_root() {
    let mut room = room();
    room.bring(&[]);
    let (state, mine) = (&room.session.state, my_folder(&room));
    let names = |parent| {
        let mut all: Vec<&str> = state
            .folders
            .values()
            .filter(|one| one.parent == Some(parent))
            .filter(|one| one.computer.is_none())
            .map(|one| one.name.as_str())
            .collect();
        all.sort();
        all
    };
    assert_eq!(names(mine), ["Claude", "Codex", "Copilot"]);
    assert_eq!(names(root(&room)), ["Global"]);
}

#[test]
fn a_computer_without_claude_or_codex_writes_nothing_for_them() {
    let mut room = room();
    std::fs::remove_dir_all(room.home.join(".claude")).unwrap();
    std::fs::remove_dir_all(room.home.join(".codex")).unwrap();
    room.bring(&[".copilot/copilot-instructions.md"]);
    let before = every_agent_file(&room);
    doc_here(
        &mut room,
        "claude",
        Kind::Rule,
        "CLAUDE.md",
        "Claude here.\n",
    );
    doc_here(&mut room, "codex", Kind::Rule, "AGENTS.md", "Codex here.\n");
    doc_here(&mut room, GLOBAL, Kind::Rule, "style.md", "Everywhere.\n");

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(every_agent_file(&room), before);
    assert!(!room.home.join(".claude").exists() && !room.home.join(".codex").exists());
}

#[test]
fn another_computers_own_claude_file_is_never_written_here_nor_counted() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let body = body_for("claude", "CLAUDE.md", "Laptop rules.\n", Some(&other));
    let file = laptops_in(&mut room, &other, folder, &body);
    let before = every_agent_file(&room);

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(every_agent_file(&room), before);
    assert_eq!(room.read(".claude/CLAUDE.md"), MAIN);
    assert_eq!(seen(&room, &file), Some(Seen::OtherComputer));
    assert_eq!(shown(&room, &file)["here"], "otherComputer");
    let look = room.look();
    assert_eq!((look.waiting, look.unagreed, look.proposed), (0, 0, 0));
    assert!(sides_of(&room, &file).rivals.is_empty());
}

fn sides_of(room: &Room, file: &str) -> places::Sides {
    places::sides(&room.homes, &gated(&room.session, &ledger(room)), file)
}

#[test]
fn another_computers_document_shows_no_proposal_and_counts_as_none() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let body = body_for("claude", "rules/tone.md", "Laptop tone.\n", Some(&other));
    let file = laptops_in(&mut room, &other, folder, &body);
    proposed(&mut room, &file, &body.replace("Laptop", "Short"));

    let card = shown(&room, &file);
    assert!(card["proposal"].is_null(), "{card}");
    assert_eq!(room.look().proposed, 0);
}

#[test]
fn what_another_computer_has_never_stands_for_a_file_to_bring_here() {
    let mut room = room();
    room.bring(&[]);
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let theirs = body_for("codex", "AGENTS.md", "Laptop codex.\n", Some(&other));
    laptops_in(&mut room, &other, folder, &theirs);
    from_another_computer_for(
        &mut room,
        "claude",
        Kind::Rule,
        "rules/tone.md",
        "Waiting tone.\n",
    );

    let brought = room.bring(&[".codex/AGENTS.md", ".claude/rules/tone.md"]);

    assert_eq!(
        brought.made, 2,
        "neither is already in Agents for this computer"
    );
}

#[test]
fn one_rules_file_only_looks_in_the_scope_of_the_folder_it_is_asked_in() {
    let mut room = room();
    room.bring(&[".codex/AGENTS.md"]);
    let mine = sub_of(&room, "Codex");
    let own = file_brought_from(&room, "AGENTS.md");
    let for_all = root_folder(&mut room, "Codex");
    let global = sub_of(&room, "Global");
    let found = |room: &Room, folder| {
        super::super::agents_sync::one_rules_file(&room.session, Some(folder)).map(|doc| doc.id)
    };
    assert_eq!(found(&room, mine), Some(own.clone()));
    assert_eq!(
        found(&room, for_all),
        None,
        "a rule of this computer is not for all"
    );
    assert_eq!(found(&room, global), None);

    let shared = mine_in(
        &mut room,
        for_all,
        &body_for("codex", "AGENTS.md", "For all.\n", None),
    );
    assert_eq!(found(&room, for_all), Some(shared));
    assert_eq!(found(&room, mine), Some(own));
}

fn rivals(room: &mut Room) -> (String, String) {
    room.bring(&[".claude/CLAUDE.md"]);
    let standing = file_brought_from(room, "CLAUDE.md");
    let rival = doc_here(room, "claude", Kind::Rule, "CLAUDE.md", "Spanish.\n");
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert_eq!(room.read(".claude/CLAUDE.md"), MAIN);
    (standing, rival)
}

fn claude_target_held(room: &Room) -> bool {
    let ledger = ledger(room);
    let gate = gated(&room.session, &ledger);
    let claude = room.at(".claude/CLAUDE.md");
    places::targets(&room.homes, &gate, &ledger)
        .into_iter()
        .find(|one| one.target.at == Path::new(&claude))
        .is_some_and(|one| one.target.held())
}

#[test]
fn with_none_of_two_rivals_written_the_place_is_held_and_nothing_is_written() {
    let mut room = room();
    room.bring(&[]);
    doc_here(&mut room, "claude", Kind::Rule, "CLAUDE.md", "One.\n");
    doc_here(&mut room, "claude", Kind::Rule, "CLAUDE.md", "Two.\n");
    let before = every_agent_file(&room);

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(every_agent_file(&room), before);
    assert!(claude_target_held(&room));
}

#[test]
fn the_side_this_computer_already_wrote_asks_too_when_a_rival_arrives_and_the_file_stays() {
    let mut room = room();
    let (standing, rival) = rivals(&mut room);
    assert!(claude_target_held(&room));
    assert_eq!(room.read(".claude/CLAUDE.md"), MAIN);
    for file in [&standing, &rival] {
        assert_eq!(shown(&room, file)["here"], "contested", "{file}");
        assert_eq!(shown(&room, file)["rivals"].as_array().unwrap().len(), 1);
    }
}

#[test]
fn an_own_document_made_after_an_everyone_one_was_applied_makes_both_ask_in_that_order() {
    let mut room = room();
    std::fs::remove_file(room.home.join(".claude/CLAUDE.md")).unwrap();
    room.bring(&[]);
    let everyone = doc_here(
        &mut room,
        "claude",
        Kind::Rule,
        "CLAUDE.md",
        "For all.
",
    );
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "For all.
"
    );
    assert!(!claude_target_held(&room));
    let folder = my_folder(&room);
    let own_body = body_for(
        "claude",
        "CLAUDE.md",
        "Mine.
",
        Some(&me(&room)),
    );
    let own = mine_in(&mut room, folder, &own_body);

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "For all.
"
    );
    assert!(claude_target_held(&room));
    assert_eq!(shown(&room, &everyone)["here"], "contested");
    assert_eq!(shown(&room, &own)["here"], "contested");

    take(&room, &own, true);

    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        "Mine.
"
    );
    assert_eq!(stance_of(&room, &own), places::Stance::Goes);
    assert_eq!(stance_of(&room, &everyone), places::Stance::Aside);
    assert_eq!(shown(&room, &everyone)["here"], "aside");
}

#[test]
fn an_everyone_document_from_another_computer_waits_and_names_the_own_one_it_clashes_with() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let own = file_brought_from(&room, "CLAUDE.md");
    let theirs = from_another_computer_for(
        &mut room,
        "claude",
        Kind::Rule,
        "CLAUDE.md",
        "Theirs.
",
    );

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(room.read(".claude/CLAUDE.md"), MAIN);
    let card = shown(&room, &theirs);
    assert_eq!(card["here"], "waiting");
    assert_eq!(card["called"]["of"], serde_json::json!({"is": "everyone"}));
    assert_eq!(card["rivals"][0]["of"], serde_json::json!({"is": "mine"}));
    assert!(card["rivals"][0]["title"].is_string(), "{card}");
    assert!(
        !claude_target_held(&room),
        "the own side keeps writing while the other waits"
    );
    assert_eq!(
        shown(&room, &own)["rivals"][0]["of"],
        serde_json::json!({"is": "everyone"})
    );
}

#[test]
fn a_rival_of_another_computer_is_named_with_that_computers_name() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let own = file_brought_from(&room, "CLAUDE.md");
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let theirs = body_for(
        "claude",
        "CLAUDE.md",
        "Laptop.
",
        Some(&other),
    );
    let file = laptops_in(&mut room, &other, folder, &theirs);

    let card = shown(&room, &file);

    assert_eq!(card["called"]["of"]["is"], "other", "{card}");
    assert!(card["called"]["of"]["name"].is_string());
    assert_eq!(
        shown(&room, &own)["called"]["of"],
        serde_json::json!({"is": "mine"})
    );
}

#[test]
fn taking_the_rival_here_writes_it_with_a_backup_and_makes_the_other_yield() {
    let mut room = room();
    let (standing, rival) = rivals(&mut room);

    take(&room, &rival, true);

    assert_eq!(room.read(".claude/CLAUDE.md"), "Spanish.\n");
    assert_eq!(room.kept(".claude/CLAUDE.md"), 1);
    assert_eq!(stance_of(&room, &standing), places::Stance::Aside);
    assert_eq!(seen(&room, &standing), Some(Seen::Admitted));
    assert_eq!(shown(&room, &standing)["here"], "aside");
    assert!(!ledger(&room).left(&standing), "put aside, never declined");
    assert_eq!(stance_of(&room, &rival), places::Stance::Goes);
}

#[test]
fn switching_back_to_the_first_side_writes_it_again_with_another_backup() {
    let mut room = room();
    let (standing, rival) = rivals(&mut room);
    take(&room, &rival, true);

    take(&room, &standing, true);

    assert_eq!(room.read(".claude/CLAUDE.md"), MAIN);
    assert_eq!(room.kept(".claude/CLAUDE.md"), 2);
    assert_eq!(stance_of(&room, &rival), places::Stance::Aside);
    assert_eq!(stance_of(&room, &standing), places::Stance::Goes);
}

#[test]
fn not_here_with_a_rival_chooses_the_rival_and_puts_this_one_aside() {
    let mut room = room();
    let (standing, rival) = rivals(&mut room);

    take(&room, &rival, false);

    assert_eq!(stance_of(&room, &rival), places::Stance::Aside);
    assert_eq!(stance_of(&room, &standing), places::Stance::Goes);
    assert_eq!(room.read(".claude/CLAUDE.md"), MAIN);
    assert_eq!(seen(&room, &rival), Some(Seen::Admitted));
    assert!(!ledger(&room).left(&rival));
}

#[test]
fn not_here_without_a_rival_is_a_plain_no() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let file = file_brought_from(&room, "CLAUDE.md");

    take(&room, &file, false);

    assert!(ledger(&room).left(&file));
    assert_eq!(seen(&room, &file), Some(Seen::DeclinedHere));
}

fn chosen_then(room: &mut Room) -> (String, String) {
    let (standing, rival) = rivals(room);
    take(room, &rival, true);
    (standing, rival)
}

// The chosen one went: the other never overwrites by itself, its place is held and asks again.
fn asks_again(room: &Room, standing: &str) {
    assert_eq!(stance_of(room, standing), places::Stance::Asks);
    assert_eq!(shown(room, standing)["here"], "contested");
    assert!(loose(room).is_empty(), "its file is not offered meanwhile");
    assert_eq!(room.read(".claude/CLAUDE.md"), "Spanish.\n");
}

#[test]
fn when_the_chosen_one_is_deleted_the_other_asks_and_never_overwrites_by_itself() {
    let mut room = room();
    let (standing, rival) = chosen_then(&mut room);

    room.session
        .drop_doc(&doc_of_file(&room, &rival).to_string())
        .unwrap();
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    asks_again(&room, &standing);
    take(&room, &standing, true);
    assert_eq!(room.read(".claude/CLAUDE.md"), MAIN);
}

#[test]
fn when_the_chosen_one_moves_to_another_computer_the_other_asks_and_never_overwrites() {
    let mut room = room();
    let (standing, rival) = chosen_then(&mut room);
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);

    moved_to(&mut room, &rival, folder);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    asks_again(&room, &standing);
}

#[test]
fn when_the_chosen_one_is_removed_by_sync_the_other_asks_and_never_overwrites() {
    let mut room = room();
    let (standing, rival) = chosen_then(&mut room);
    let other = laptop();
    let id = doc_of_file(&room, &rival);
    as_laptop(&mut room, &other, vec![Op::DocDelete { id }]);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    asks_again(&room, &standing);
}

#[test]
fn choosing_a_skill_takes_the_whole_folder_and_the_rival_skill_yields() {
    let mut room = room();
    room.bring(&[]);
    let skill = |room: &mut Room, level: &str, name: &str, said: &str| {
        let body = agents::doc_of(level, Kind::Skill, name, said.as_bytes()).unwrap();
        let root = root(room);
        mine_in(room, root, &body)
    };
    let head = skill(&mut room, GLOBAL, "tidy/SKILL.md", "Head.\n");
    let notes = skill(&mut room, GLOBAL, "tidy/notes.md", "Notes.\n");
    let rival = skill(&mut room, "claude", "tidy/SKILL.md", "Claude tidy.\n");
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert!(!Path::new(&room.at(".claude/skills/tidy/SKILL.md")).exists());

    take(&room, &head, true);

    assert_eq!(room.read(".claude/skills/tidy/SKILL.md"), "Head.\n");
    assert_eq!(room.read(".claude/skills/tidy/notes.md"), "Notes.\n");
    assert_eq!(seen(&room, &notes), Some(Seen::Admitted));
    assert_eq!(stance_of(&room, &rival), places::Stance::Aside);
    assert_eq!(seen(&room, &rival), Some(Seen::Admitted));
}

#[test]
fn a_global_skill_put_aside_in_claude_keeps_writing_in_the_shared_folder() {
    let mut room = room();
    room.bring(&[]);
    let skill = |room: &mut Room, level: &str, said: &str| {
        let body = agents::doc_of(level, Kind::Skill, "tidy/SKILL.md", said.as_bytes()).unwrap();
        let root = root(room);
        mine_in(room, root, &body)
    };
    let global = skill(&mut room, GLOBAL, "Head.\n");
    let own = skill(&mut room, "claude", "Claude tidy.\n");
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    take(&room, &own, true);

    assert_eq!(room.read(".claude/skills/tidy/SKILL.md"), "Claude tidy.\n");
    assert_eq!(room.read(".agents/skills/tidy/SKILL.md"), "Head.\n");
    assert_eq!(
        stance_of(&room, &global),
        places::Stance::Goes,
        "aside in one agent only, it still goes where nobody else writes"
    );
}

#[test]
fn a_codex_moved_to_the_root_waits_for_the_choice_and_the_choice_goes_both_ways() {
    let mut room = room();
    room.bring(&[".codex/AGENTS.md"]);
    let own = file_brought_from(&room, "AGENTS.md");
    let theirs = from_another_computer_for(
        &mut room,
        "codex",
        Kind::Rule,
        "AGENTS.md",
        "Other codex.\n",
    );
    let before = every_agent_file(&room);

    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert_eq!(
        every_agent_file(&room),
        before,
        "nothing changes until chosen"
    );

    take(&room, &theirs, true);
    assert_eq!(room.read(".codex/AGENTS.md"), "Other codex.\n");
    assert_eq!(room.kept(".codex/AGENTS.md"), 1);

    take(&room, &own, true);
    assert_eq!(room.read(".codex/AGENTS.md"), "Codex rules.\n");
    assert_eq!(room.kept(".codex/AGENTS.md"), 2);
}

struct Legacy {
    at: PathBuf,
    global: String,
    claude: String,
}

// What Tisty 1.50 left: root Claude and Global without a computer, a CLAUDE.md made of both.
fn legacy(room: &mut Room) -> Legacy {
    room.bring(&[]);
    let (claude_folder, global_folder) = (root_folder(room, "Claude"), sub_of(room, "Global"));
    let claude = old_mine_in(
        room,
        claude_folder,
        &body_for("claude", "CLAUDE.md", "# Claude\nTests first.\n", None),
    );
    let global = old_mine_in(
        room,
        global_folder,
        &body_for(GLOBAL, "style.md", "Spanish.\n", None),
    );
    let at = PathBuf::from(room.at(".claude/CLAUDE.md"));
    put(&at, BOTH);
    written(room, &at, &[&global, &claude]);
    Legacy { at, global, claude }
}

fn written(room: &Room, at: &Path, files: &[&str]) {
    let mut ledger = Ledger::fresh(HERE);
    ledger.consent(true);
    ledger.turn_all(true);
    ledger.settle(at, &std::fs::read(at).unwrap());
    ledger.applied_to(at, files.iter().map(ToString::to_string));
    ledger.keep(room.session.paths.data()).unwrap();
}

fn agreed_print(room: &Room, file: &str) -> Option<String> {
    kept(room, file).agreed.clone()
}

#[test]
fn a_mixed_file_of_1_50_is_one_document_of_this_computer_and_not_a_byte_changes() {
    let mut room = room();
    let old = legacy(&mut room);
    let before = every_agent_file(&room);

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(every_agent_file(&room), before);
    assert_eq!(room.read(".claude/CLAUDE.md"), BOTH);
    let me = me(&room);
    assert_eq!(computer_of(&room, &old.claude), Some(me.clone()));
    let body = read(&room.session, &old.claude).unwrap();
    assert_eq!(agents::rendered(&body).unwrap(), BOTH.as_bytes());
    let ledger = ledger(&room);
    assert_eq!(
        ledger.applied_at(&old.at).collect::<Vec<_>>(),
        [old.claude.as_str()]
    );
    assert!(ledger.homed());
    assert_eq!(room.kept(".claude/CLAUDE.md"), 0);
    assert!(
        !round_now(
            &room.session,
            &room.homes,
            &mut Ledger::open(room.session.paths.data(), HERE)
        )
        .told
        .iter()
        .any(|one| one.standing == "write")
    );
}

#[test]
fn a_file_taken_as_one_document_leaves_its_rivals_waiting_and_unwritten() {
    let mut room = room();
    let old = legacy(&mut room);
    let rival =
        from_another_computer_for(&mut room, "claude", Kind::Rule, "CLAUDE.md", "Elsewhere.\n");

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(room.read(".claude/CLAUDE.md"), BOTH);
    assert_eq!(seen(&room, &rival), Some(Seen::WaitingHere));
    assert_eq!(seen(&room, &old.claude), Some(Seen::Admitted));
    assert_eq!(stance_of(&room, &old.claude), places::Stance::Goes);
}

#[test]
fn homing_keeps_the_approval_leaves_global_at_the_root_and_runs_once() {
    let mut room = room();
    let old = legacy(&mut room);
    let approved = agreed_print(&room, &old.claude);
    assert!(approved.is_some());

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    let body = read(&room.session, &old.claude).unwrap();
    assert_eq!(card_of(&room, &old.claude).computer, Some(me(&room)));
    assert_eq!(
        agreed_print(&room, &old.claude),
        Some(tisty_core::docs::print_of_text(&body)),
        "still approved, for the words as they are now"
    );
    assert_eq!(
        kept(&room, &old.global).folder,
        Some(sub_of(&room, "Global"))
    );
    assert_eq!(computer_of(&room, &old.global), None);

    let for_all = root_folder(&mut room, "Claude");
    moved_to(&mut room, &old.claude, for_all);
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert_eq!(
        kept(&room, &old.claude).folder,
        Some(for_all),
        "never pulled back"
    );
    assert_eq!(card_of(&room, &old.claude).computer, None);
}

pub(crate) fn runs(room: &mut Room, other: &DeviceId, release: &str) {
    let ops = vec![
        Op::DeviceJoin {
            d: other.clone(),
            k: None,
            p: None,
        },
        Op::DeviceRelease {
            r: release.to_string(),
        },
    ];
    as_laptop(room, other, ops);
}

#[test]
fn homing_waits_while_another_computer_runs_an_earlier_version() {
    let mut room = room();
    let old = legacy(&mut room);
    let other = laptop();
    runs(&mut room, &other, "1.50.1");
    let before = every_agent_file(&room);

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(computer_of(&room, &old.claude), None, "not moved yet");
    assert!(!ledger(&room).homed());
    assert_eq!(
        ledger(&room).homing().map(|set| set.len()),
        Some(1),
        "what has to go is known from the first round"
    );
    assert_eq!(every_agent_file(&room), before);

    runs(&mut room, &other, SINCE);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(computer_of(&room, &old.claude), Some(me(&room)));
    assert!(ledger(&room).homed());
}

fn global_only_codex(room: &mut Room) -> Legacy {
    room.bring(&[]);
    let global_folder = sub_of(room, "Global");
    let global = old_mine_in(
        room,
        global_folder,
        &body_for(GLOBAL, "style.md", "Codex rules.\n", None),
    );
    let at = PathBuf::from(room.at(".codex/AGENTS.md"));
    written(room, &at, &[&global]);
    Legacy {
        at,
        global,
        claude: String::new(),
    }
}

fn codex_rule(room: &Room) -> Option<String> {
    bodies(&room.session)
        .into_iter()
        .find(|(_, body)| {
            Card::of(body).is_some_and(|card| card.address() == "codex/rule/AGENTS.md")
        })
        .map(|(file, _)| file)
}

#[test]
fn a_file_of_a_global_rule_alone_becomes_a_document_of_this_computer() {
    let mut room = room();
    let old = global_only_codex(&mut room);
    let before = every_agent_file(&room);

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    let file = codex_rule(&room).expect("a document for AGENTS.md");
    assert_eq!(computer_of(&room, &file), Some(me(&room)));
    assert_eq!(
        agents::rendered(&read(&room.session, &file).unwrap()).unwrap(),
        b"Codex rules.\n"
    );
    assert_eq!(every_agent_file(&room), before);
    assert_eq!(
        ledger(&room).applied_at(&old.at).collect::<Vec<_>>(),
        [file.as_str()]
    );
    assert!(kept(&room, &old.global).folder.is_some());
}

fn nothing_kept(room: &mut Room) {
    let before = every_agent_file(room);
    let docs = bodies(&room.session).len();
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert_eq!(codex_rule(room), None);
    assert_eq!(bodies(&room.session).len(), docs);
    assert_eq!(every_agent_file(room), before);
}

#[test]
fn a_file_changed_on_disk_since_tisty_wrote_it_is_not_taken() {
    let mut room = room();
    let old = global_only_codex(&mut room);
    put(&old.at, "Changed by hand.\n");
    nothing_kept(&mut room);
}

#[test]
fn a_file_of_an_agent_on_pause_is_not_taken() {
    let mut room = room();
    global_only_codex(&mut room);
    let mut ledger = ledger(&room);
    ledger.turn("codex", false);
    ledger.keep(room.session.paths.data()).unwrap();
    nothing_kept(&mut room);
}

#[test]
fn a_file_the_person_left_with_the_agent_is_not_taken() {
    let mut room = room();
    let old = global_only_codex(&mut room);
    let mut ledger = ledger(&room);
    ledger.stay(&old.at);
    ledger.keep(room.session.paths.data()).unwrap();
    nothing_kept(&mut room);
}

#[test]
fn a_document_sent_away_by_hand_here_does_not_make_its_file_ours() {
    let mut room = room();
    let old = global_only_codex(&mut room);
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    moved_to(&mut room, &old.global, folder);
    assert!(ledger(&room).left(&old.global));
    nothing_kept(&mut room);
}

#[test]
fn a_root_document_with_newer_words_than_the_file_keeps_them_and_the_file_is_taken_as_it_stands() {
    let mut room = room();
    let old = legacy(&mut room);
    let body = read(&room.session, &old.claude).unwrap();
    let newer = body.replace("Tests first.", "Tests later.");
    rewritten(&mut room.session, &old.claude, &newer).unwrap();
    let before = every_agent_file(&room);

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(every_agent_file(&room), before);
    assert!(
        read(&room.session, &old.claude)
            .unwrap()
            .contains("Tests later.")
    );
    assert_eq!(
        room.read(".claude/CLAUDE.md"),
        BOTH,
        "the file stays as 1.50 left it"
    );
    let mine = bodies(&room.session)
        .into_iter()
        .find(|(file, body)| {
            *file != old.claude && agents::rendered(body).is_ok_and(|said| said == BOTH.as_bytes())
        })
        .expect("a document of this computer equal to the file");
    assert_eq!(computer_of(&room, &mine.0), Some(me(&room)));
}

// The document that wrote a file went to another computer by that computer's hand, after the files were homed.
fn became_the_laptops(room: &mut Room) -> (Legacy, DeviceId) {
    let old = global_only_codex(room);
    let mut ledger = ledger(room);
    ledger.home_all(BTreeSet::new());
    ledger.keep(room.session.paths.data()).unwrap();
    let other = laptop();
    let folder = laptops_folder(room, &other);
    let body = body_for(GLOBAL, "style.md", "Codex rules.\n", Some(&other));
    rewritten(&mut room.session, &old.global, &body).unwrap();
    let id = doc_of_file(room, &old.global);
    let op = Op::DocMove {
        id,
        d: Filed {
            folder: Some(Some(folder)),
            page_of: None,
            order: None,
        },
    };
    as_laptop(room, &other, vec![op]);
    (old, other)
}

#[test]
fn a_document_that_became_another_computers_leaves_its_file_to_this_computer() {
    let mut room = room();
    let (old, _) = became_the_laptops(&mut room);
    assert_eq!(seen(&room, &old.global), Some(Seen::OtherComputer));
    let before = every_agent_file(&room);

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    let file = codex_rule(&room).expect("the file became a document of this computer");
    assert_eq!(computer_of(&room, &file), Some(me(&room)));
    assert_eq!(every_agent_file(&room), before);
}

fn tone_moved_by_hand(room: &mut Room, folder: FolderId) -> String {
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(room, "rules/tone.md");
    moved_to(room, &file, folder);
    file
}

#[test]
fn moving_a_document_to_another_computers_folder_changes_its_mark_and_asks_about_its_file() {
    let mut room = room();
    let other = laptop();
    room.bring(&[]);
    let folder = laptops_folder(&mut room, &other);
    let file = tone_moved_by_hand(&mut room, folder);

    assert_eq!(card_of(&room, &file).computer, Some(other));
    assert!(ledger(&room).left(&file), "left here, not written");
    assert!(
        agreed_print(&room, &file).is_some(),
        "it was approved, so it stays approved"
    );
    assert_eq!(shown(&room, &file)["here"], "otherComputer");
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert_eq!(room.read(".claude/rules/tone.md"), "Short answers.\n");
    assert!(loose(&room).contains(&room.at(".claude/rules/tone.md")));
}

#[test]
fn moving_an_unapproved_document_approves_nothing() {
    let mut room = room();
    let other = laptop();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    let body = read(&room.session, &file).unwrap();
    let edited = body.replace("Short answers.", "Short, changed.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &edited).unwrap();
    room.session.reload().unwrap();
    let folder = laptops_folder(&mut room, &other);

    moved_to(&mut room, &file, folder);

    assert_eq!(card_of(&room, &file).computer, Some(other));
    assert_ne!(
        agreed_print(&room, &file),
        Some(tisty_core::docs::print_of_text(
            &read(&room.session, &file).unwrap()
        ))
    );
}

// Made here and approved, but never written: moving it is the first word on it.
pub(crate) fn made_for_me(room: &mut Room, name: &str, said: &str) -> String {
    let (folder, me) = (my_folder(room), me(room));
    mine_in(room, folder, &body_for("claude", name, said, Some(&me)))
}

#[test]
fn moving_a_document_to_the_root_counts_as_taken_here() {
    let mut room = room();
    room.bring(&[]);
    let file = made_for_me(&mut room, "rules/tone.md", "Tone.\n");
    let for_all = root_folder(&mut room, "Claude");
    assert!(!ledger(&room).took(&file));

    moved_to(&mut room, &file, for_all);

    assert_eq!(card_of(&room, &file).computer, None);
    assert!(ledger(&room).took(&file));
    assert_eq!(seen(&room, &file), Some(Seen::Admitted));
}

#[test]
fn moving_a_document_to_the_root_beside_a_rival_waits_for_the_choice() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let file = made_for_me(&mut room, "CLAUDE.md", "Moved.\n");
    let for_all = root_folder(&mut room, "Claude");

    moved_to(&mut room, &file, for_all);

    assert!(ledger(&room).waits(&file));
    assert!(!ledger(&room).took(&file));
    assert_eq!(seen(&room, &file), Some(Seen::WaitingHere));
    assert_eq!(room.read(".claude/CLAUDE.md"), MAIN);
}

#[test]
fn moving_a_document_into_this_computers_folder_counts_as_taken() {
    let mut room = room();
    room.bring(&[]);
    let theirs =
        from_another_computer_for(&mut room, "claude", Kind::Rule, "rules/tone.md", "Tone.\n");
    let mine = my_folder(&room);

    moved_to(&mut room, &theirs, mine);

    assert_eq!(card_of(&room, &theirs).computer, Some(me(&room)));
    assert!(ledger(&room).took(&theirs));
}

#[test]
fn a_memory_never_changes_computer_when_it_moves() {
    let mut room = room();
    room.bring(&[]);
    let body = agents::doc_of(
        "claude",
        Kind::Memory,
        "projects/p/memory/MEMORY.md",
        b"- desk\n",
    )
    .unwrap();
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let file = laptops_in(&mut room, &other, folder, &body);
    let mine = my_folder(&room);
    let before = read(&room.session, &file).unwrap();

    moved_to(&mut room, &file, mine);

    assert_eq!(read(&room.session, &file).unwrap(), before);
    assert_eq!(card_of(&room, &file).computer, None);
}

#[test]
fn a_document_made_in_a_folder_of_a_computer_through_the_window_is_a_rule_of_it() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let claude = sub_of(&room, "Claude");
    let file = made_in(&mut room, claude);
    let seed = crate::answers::agents_sync::seeded(&room.session, Some(claude));
    crate::answers::papers::written_by_hand(&mut room.session, file.clone(), seed, None, None)
        .unwrap();

    assert_eq!(card_of(&room, &file).computer, Some(me(&room)));
    assert_eq!(computer_of(&room, &file), Some(me(&room)));
}

#[test]
fn in_another_computers_folder_a_new_document_seeds_nothing() {
    let mut room = room();
    room.bring(&[]);
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let claude = ulid::Ulid::generate();
    let made = Op::FolderAdd {
        id: claude,
        d: tisty_core::event::FolderAdd {
            name: "Claude".to_string(),
            order: "a".to_string(),
            parent: Some(folder),
            icon: None,
            color: None,
            computer: None,
        },
    };
    as_laptop(&mut room, &other, vec![made]);

    assert_eq!(
        crate::answers::agents_sync::seeded(&room.session, Some(claude)),
        ""
    );
    let mine = sub_of(&room, "Claude");
    assert!(!crate::answers::agents_sync::seeded(&room.session, Some(mine)).is_empty());
}

// The chosen one cannot be written for now: the other is held and asks, it never takes over by itself.
fn held_and_asking(room: &Room, standing: &str) {
    assert_eq!(seen(room, standing), Some(Seen::Admitted));
    assert_eq!(stance_of(room, standing), places::Stance::Asks);
    assert_eq!(shown(room, standing)["here"], "contested");
    assert_eq!(room.read(".claude/CLAUDE.md"), "Spanish.\n");
    assert!(loose(room).is_empty());
}

#[test]
fn while_the_chosen_one_waits_for_approval_the_other_never_overwrites_its_file() {
    let mut room = room();
    let (standing, rival) = chosen_then(&mut room);
    let body = read(&room.session, &rival).unwrap();
    tisty_core::docs::write(
        &room.session.paths.docs(),
        &rival,
        &body.replace("Spanish.", "Changed by hand."),
    )
    .unwrap();
    room.session.reload().unwrap();

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(seen(&room, &rival), Some(Seen::NotAgreed));
    held_and_asking(&room, &standing);
}

#[test]
fn while_the_chosen_one_cannot_be_read_the_other_never_overwrites_its_file() {
    let mut room = room();
    let (standing, rival) = chosen_then(&mut room);
    let at = tisty_core::docs::resolve(&room.session.paths.docs(), &rival).unwrap();
    std::fs::remove_file(at).unwrap();

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(seen(&room, &rival), Some(Seen::Unread));
    held_and_asking(&room, &standing);
}

#[test]
fn once_the_unapproved_chosen_one_is_deleted_the_other_still_only_asks() {
    let mut room = room();
    let (standing, rival) = chosen_then(&mut room);
    let body = read(&room.session, &rival).unwrap();
    tisty_core::docs::write(
        &room.session.paths.docs(),
        &rival,
        &body.replace("Spanish.", "Edited."),
    )
    .unwrap();
    room.session.reload().unwrap();
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    held_and_asking(&room, &standing);

    room.session
        .drop_doc(&doc_of_file(&room, &rival).to_string())
        .unwrap();
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    held_and_asking(&room, &standing);
    take(&room, &standing, true);
    assert_eq!(room.read(".claude/CLAUDE.md"), MAIN);
}

// What the other computer does by itself: the document goes into its folder, marked for it.
fn laptop_takes(room: &mut Room, other: &DeviceId, file: &str, folder: FolderId) {
    let body = read(&room.session, file).unwrap();
    let marked = agents::meant_for(&body, Some(other)).unwrap();
    rewritten(&mut room.session, file, &marked).unwrap();
    let id = doc_of_file(room, file);
    let op = Op::DocMove {
        id,
        d: Filed {
            folder: Some(Some(folder)),
            page_of: None,
            order: None,
        },
    };
    as_laptop(room, other, vec![op]);
}

fn an_own_and_a_laptops_both_admitted(room: &mut Room) -> (String, String) {
    room.bring(&[".claude/CLAUDE.md"]);
    let own = file_brought_from(room, "CLAUDE.md");
    let theirs = from_another_computer_for(room, "claude", Kind::Rule, "CLAUDE.md", "Laptop.\n");
    take(room, &theirs, true);
    assert_eq!(room.read(".claude/CLAUDE.md"), "Laptop.\n");
    (own, theirs)
}

fn equal_to(room: &Room, bytes: &str, but: &[&str]) -> Option<String> {
    bodies(&room.session)
        .into_iter()
        .filter(|(file, _)| !but.contains(&file.as_str()))
        .find(|(file, body)| {
            computer_of(room, file) == Some(me(room))
                && agents::rendered(body).is_ok_and(|said| said == bytes.as_bytes())
        })
        .map(|(file, _)| file)
}

#[test]
fn the_document_that_wrote_a_file_goes_to_the_laptop_and_the_file_becomes_one_of_this_computer() {
    let mut room = room();
    let (own, theirs) = an_own_and_a_laptops_both_admitted(&mut room);
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let before = every_agent_file(&room);

    laptop_takes(&mut room, &other, &theirs, folder);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(every_agent_file(&room), before, "not a byte changes");
    assert_eq!(room.read(".claude/CLAUDE.md"), "Laptop.\n");
    let made = equal_to(&room, "Laptop.\n", &[&theirs]).expect("a document equal to the file");
    assert_eq!(stance_of(&room, &made), places::Stance::Goes);
    assert_eq!(stance_of(&room, &own), places::Stance::Aside);
    assert!(loose(&room).is_empty());
}

#[test]
fn the_same_happens_when_this_computers_own_document_was_declined() {
    let mut room = room();
    let (own, theirs) = an_own_and_a_laptops_both_admitted(&mut room);
    let mut declined = ledger(&room);
    declined.take(&own, false);
    declined.keep(room.session.paths.data()).unwrap();
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let before = every_agent_file(&room);

    laptop_takes(&mut room, &other, &theirs, folder);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(every_agent_file(&room), before);
    let made = equal_to(&room, "Laptop.\n", &[&theirs]).expect("a document equal to the file");
    assert_eq!(seen(&room, &made), Some(Seen::Admitted));
    assert_eq!(seen(&room, &own), Some(Seen::DeclinedHere));
}

#[test]
fn an_own_document_that_already_says_the_same_is_the_one_taken() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let own = file_brought_from(&room, "CLAUDE.md");
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let theirs = from_another_computer_for(&mut room, "claude", Kind::Rule, "CLAUDE.md", MAIN);
    take(&room, &theirs, true);
    let before = bodies(&room.session).len();

    laptop_takes(&mut room, &other, &theirs, folder);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(room.read(".claude/CLAUDE.md"), MAIN);
    assert_eq!(bodies(&room.session).len(), before, "no new document");
    assert_eq!(stance_of(&room, &own), places::Stance::Goes);
    assert_eq!(
        ledger(&room)
            .applied_at(Path::new(&room.at(".claude/CLAUDE.md")))
            .collect::<Vec<_>>(),
        [own.as_str()]
    );
}

#[test]
fn a_file_the_other_computer_took_is_kept_at_once_unless_it_was_sent_away_here() {
    let mut room = room();
    let (_, theirs) = an_own_and_a_laptops_both_admitted(&mut room);
    let mut ledger = ledger(&room);
    ledger.take(&theirs, false);
    ledger.keep(room.session.paths.data()).unwrap();
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);

    laptop_takes(&mut room, &other, &theirs, folder);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(equal_to(&room, "Laptop.\n", &[&theirs]), None);
    assert_eq!(room.read(".claude/CLAUDE.md"), "Laptop.\n");
}

#[test]
fn a_document_of_another_computer_refuses_every_hand_on_it_from_here() {
    let mut room = room();
    room.bring(&[]);
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let body = body_for("claude", "rules/tone.md", "Laptop tone.\n", Some(&other));
    let file = laptops_in(&mut room, &other, folder, &body);

    let written = crate::answers::papers::written_by_hand(
        &mut room.session,
        file.clone(),
        body.replace("Laptop", "Mine"),
        None,
        None,
    )
    .err()
    .unwrap();
    assert_eq!(written.code, "agentOfAnother");
    let print = tisty_core::docs::print_of_text(&body);
    let agreed = crate::answers::agent_docs::agree(
        &mut room.session,
        Some(&room.homes),
        HERE,
        &file,
        &print,
    )
    .unwrap_err();
    assert_eq!(agreed.code, "agentOfAnother");
    let restored =
        crate::answers::earlier::put_back(&mut room.session, &file, &body.replace("Laptop", "Old"))
            .unwrap_err();
    assert_eq!(restored.code, "agentOfAnother");
    assert_eq!(read(&room.session, &file).unwrap(), body);
}

#[test]
fn moving_a_document_to_another_scope_with_a_proposal_pending_is_refused() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    let body = read(&room.session, &file).unwrap();
    proposed(&mut room, &file, &body.replace("Short", "Brief"));
    let id = doc_of_file(&room, &file);
    let for_all = root_folder(&mut room, "Claude");
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);

    for target in [Some(for_all), Some(folder), None] {
        let refused = resolved(&room.session.state, id, target).unwrap_err();
        assert_eq!(refused.code, "agentProposalFirst", "{target:?}");
    }
    let same_scope = sub_of(&room, "Claude");
    assert!(resolved(&room.session.state, id, Some(same_scope)).is_ok());
}

#[test]
fn moving_a_document_without_a_proposal_is_never_refused() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    let for_all = root_folder(&mut room, "Claude");

    assert!(
        resolved(
            &room.session.state,
            doc_of_file(&room, &file),
            Some(for_all)
        )
        .is_ok()
    );
}

#[test]
fn a_marked_body_whose_approval_did_not_know_computers_is_never_carried_as_approved() {
    let mut room = room();
    room.bring(&[]);
    let mine = my_folder(&room);
    let marked = body_for("claude", "rules/tone.md", "Tone.\n", Some(&me(&room)));
    let file = old_mine_in(&mut room, mine, &marked);
    assert!(agreed_print(&room, &file).is_some());
    let for_all = root_folder(&mut room, "Claude");

    moved_to(&mut room, &file, for_all);

    assert_eq!(card_of(&room, &file).computer, None);
    let body = read(&room.session, &file).unwrap();
    assert_ne!(
        agreed_print(&room, &file),
        Some(tisty_core::docs::print_of_text(&body)),
        "moving it approves nothing"
    );
}

#[test]
fn a_typo_edit_over_a_marked_body_approved_without_knowing_computers_approves_nothing() {
    let mut room = room();
    room.bring(&[]);
    let mine = my_folder(&room);
    let marked = body_for("claude", "rules/tone.md", "Tone.\n", Some(&me(&room)));
    let file = old_mine_in(&mut room, mine, &marked);
    let was = agreed_print(&room, &file).unwrap();
    let typo = marked.replace("Tone.", "Tone, briefly.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &typo).unwrap();

    room.session.retell_over(&file, &typo, None, Some(&was));

    assert_ne!(
        agreed_print(&room, &file),
        Some(tisty_core::docs::print_of_text(&typo))
    );
    assert_eq!(seen(&room, &file), Some(Seen::NotAgreed));
}

#[test]
fn a_document_this_build_made_and_approved_is_carried_as_approved_over_a_typo() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    let was = agreed_print(&room, &file).unwrap();
    let typo = read(&room.session, &file)
        .unwrap()
        .replace("Short answers.", "Short answers!");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &typo).unwrap();

    room.session.retell_over(&file, &typo, None, Some(&was));

    assert_eq!(
        agreed_print(&room, &file),
        Some(tisty_core::docs::print_of_text(&typo))
    );
}

#[test]
fn a_memory_moved_to_the_root_goes_by_its_folder_and_loses_the_mark() {
    let mut room = room();
    room.bring(&[]);
    let me = me(&room);
    let body = agents::doc_of(
        "claude",
        Kind::Memory,
        "projects/p/memory/MEMORY.md",
        b"- desk\n",
    )
    .unwrap();
    let marked = agents::meant_for(&body, Some(&me)).unwrap();
    let mine = my_folder(&room);
    let file = mine_in(&mut room, mine, &marked);
    let for_all = root_folder(&mut room, "Claude");

    moved_to(&mut room, &file, for_all);

    assert_eq!(card_of(&room, &file).computer, None);
}

// What the person had at the root when this build first ran, approved by a build that knew nothing of computers.
fn two_old_rules(room: &mut Room) -> (String, String) {
    room.bring(&[]);
    let folder = root_folder(room, "Claude");
    let first = old_mine_in(room, folder, &body_for("claude", "CLAUDE.md", MAIN, None));
    let second = old_mine_in(
        room,
        folder,
        &body_for("claude", "rules/tone.md", "Tone.\n", None),
    );
    (first, second)
}

fn wait_for_the_laptop(room: &mut Room) -> DeviceId {
    let other = laptop();
    runs(room, &other, "1.50.1");
    other
}

#[test]
fn a_document_made_at_the_root_by_this_build_is_never_homed_even_with_the_ledger_lost() {
    let mut room = room();
    room.bring(&[]);
    let folder = root_folder(&mut room, "Claude");
    let new = mine_in(
        &mut room,
        folder,
        &body_for("claude", "rules/tone.md", "Tone.\n", None),
    );
    std::fs::remove_dir_all(agents::folder(room.session.paths.data())).unwrap();
    room.bring(&[]);

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(kept(&room, &new).folder, Some(folder));
    assert!(ledger(&room).homed());
}

#[test]
fn what_has_to_go_home_is_taken_once_at_the_first_round() {
    let mut room = room();
    let (first, second) = two_old_rules(&mut room);
    let other = wait_for_the_laptop(&mut room);
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    let set = ledger(&room).homing().cloned().unwrap();
    assert_eq!(set, BTreeSet::from([first.clone(), second.clone()]));
    let folder = root_folder(&mut room, "Rules");
    let later = old_mine_in(
        &mut room,
        folder,
        &body_for("claude", "rules/later.md", "Later.\n", None),
    );

    runs(&mut room, &other, SINCE);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(computer_of(&room, &first), Some(me(&room)));
    assert_eq!(computer_of(&room, &second), Some(me(&room)));
    assert_eq!(
        computer_of(&room, &later),
        None,
        "not in the set, never swept"
    );
    assert!(ledger(&room).homed());
}

#[test]
fn one_left_halfway_is_finished_the_next_round() {
    let mut room = room();
    let (first, _) = two_old_rules(&mut room);
    let other = wait_for_the_laptop(&mut room);
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    let body = read(&room.session, &first).unwrap();
    let marked = agents::meant_for(&body, Some(&me(&room))).unwrap();
    tisty_core::docs::write(&room.session.paths.docs(), &first, &marked).unwrap();
    room.session.reload().unwrap();
    assert_eq!(computer_of(&room, &first), None);

    runs(&mut room, &other, SINCE);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(computer_of(&room, &first), Some(me(&room)));
    assert!(ledger(&room).homed());
}

#[test]
fn a_pending_proposal_stays_in_the_set_and_blocks_neither_the_others_nor_the_round() {
    let mut room = room();
    let (first, second) = two_old_rules(&mut room);
    let body = read(&room.session, &first).unwrap();
    proposed(&mut room, &first, &body.replace("Tests", "Specs"));

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(computer_of(&room, &second), Some(me(&room)));
    assert_eq!(computer_of(&room, &first), None);
    let ledger = ledger(&room);
    assert!(!ledger.homed());
    assert_eq!(
        ledger.homing().cloned(),
        Some(BTreeSet::from([first.clone()]))
    );
}

#[test]
fn a_document_that_cannot_be_read_stays_in_the_set_and_the_round_goes_on() {
    let mut room = room();
    let (first, second) = two_old_rules(&mut room);
    let other = wait_for_the_laptop(&mut room);
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    let at = tisty_core::docs::resolve(&room.session.paths.docs(), &first).unwrap();
    std::fs::remove_file(at).unwrap();

    runs(&mut room, &other, SINCE);
    let round = round_here(&mut room.session, &room.homes, HERE);

    assert!(round.is_ok());
    assert_eq!(computer_of(&room, &second), Some(me(&room)));
    assert!(!ledger(&room).homed());
}

#[test]
fn a_file_of_joined_documents_waits_for_homing_before_it_is_kept() {
    let mut room = room();
    let old = legacy(&mut room);
    let other = wait_for_the_laptop(&mut room);
    let before = every_agent_file(&room);

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(bodies(&room.session).len(), 2, "nothing is made yet");
    assert_eq!(every_agent_file(&room), before);
    runs(&mut room, &other, SINCE);
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert_eq!(computer_of(&room, &old.claude), Some(me(&room)));
    assert!(ledger(&room).homed());
}

#[test]
fn a_new_document_in_this_computers_codex_while_the_root_has_one_is_a_rule_and_asks_which() {
    let mut room = room();
    room.bring(&[]);
    let for_all = root_folder(&mut room, "Codex");
    let shared = mine_in(
        &mut room,
        for_all,
        &body_for("codex", "AGENTS.md", "For all.\n", None),
    );
    let own = sub_of(&room, "Codex");
    let seeded = crate::answers::agents_sync::seeded;

    let card = Card::of(&seeded(&room.session, Some(own))).unwrap();
    assert_eq!(card.address(), "codex/rule/AGENTS.md");
    assert_eq!(card.computer, Some(me(&room)));
    let seed = seeded(&room.session, Some(own));
    let mine = mine_in(&mut room, own, &seed);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(stance_of(&room, &mine), places::Stance::Asks);
    assert_eq!(stance_of(&room, &shared), places::Stance::Asks);
    assert_eq!(room.read(".codex/AGENTS.md"), "Codex rules.\n");
}

#[test]
fn for_agents_with_a_rules_folder_the_own_scope_counts_the_root_too() {
    let mut room = room();
    room.bring(&[]);
    let for_all = root_folder(&mut room, "Claude");
    mine_in(
        &mut room,
        for_all,
        &body_for("claude", "rules/rule.md", "Shared.\n", None),
    );
    let own = sub_of(&room, "Claude");

    let card = Card::of(&crate::answers::agents_sync::seeded(
        &room.session,
        Some(own),
    ))
    .unwrap();

    assert_eq!(card.address(), "claude/rule/CLAUDE.md");
    let me = me(&room);
    mine_in(
        &mut room,
        own,
        &body_for("claude", "CLAUDE.md", "Mine.\n", Some(&me)),
    );
    let again = Card::of(&crate::answers::agents_sync::seeded(
        &room.session,
        Some(own),
    ))
    .unwrap();
    assert_eq!(
        again.address(),
        "claude/rule/rules/rule-2.md",
        "the root's rule and this computer's own are both taken"
    );
}

fn tidy_skill(room: &mut Room, level: &str, name: &str, said: &str) -> String {
    let body = agents::doc_of(level, Kind::Skill, name, said.as_bytes()).unwrap();
    let root = root(room);
    mine_in(room, root, &body)
}

#[test]
fn taking_one_file_of_a_skill_with_no_rival_never_touches_its_siblings() {
    let mut room = room();
    room.bring(&[]);
    let head = tidy_skill(
        &mut room,
        GLOBAL,
        "tidy/SKILL.md",
        "Head.
",
    );
    let notes = tidy_skill(
        &mut room,
        GLOBAL,
        "tidy/notes.md",
        "Notes.
",
    );
    let mut ledger = ledger(&room);
    let gate = gated(&room.session, &ledger);

    chosen(&room.session, &mut ledger, &gate, &room.homes, &head, true);

    assert!(ledger.took(&head));
    assert!(!ledger.took(&notes), "a sibling is not taken along");
    assert!(!ledger.left(&notes));
}

#[test]
fn taking_a_skill_beside_a_rival_takes_its_siblings_except_the_declined_one() {
    let mut room = room();
    room.bring(&[]);
    let head = tidy_skill(&mut room, GLOBAL, "tidy/SKILL.md", "Head.\n");
    let notes = tidy_skill(&mut room, GLOBAL, "tidy/notes.md", "Notes.\n");
    let extra = tidy_skill(&mut room, GLOBAL, "tidy/extra.md", "Extra.\n");
    tidy_skill(&mut room, "claude", "tidy/SKILL.md", "Claude tidy.\n");
    let mut declined = ledger(&room);
    declined.take(&extra, false);
    declined.keep(room.session.paths.data()).unwrap();

    take(&room, &head, true);

    let ledger = ledger(&room);
    assert!(ledger.took(&head) && ledger.took(&notes));
    assert!(ledger.left(&extra) && !ledger.took(&extra));
}

#[test]
fn not_here_with_two_admitted_rivals_declines_only_this_file() {
    let mut room = room();
    room.bring(&[]);
    let one = doc_here(&mut room, "claude", Kind::Rule, "CLAUDE.md", "One.\n");
    let two = doc_here(&mut room, "claude", Kind::Rule, "CLAUDE.md", "Two.\n");
    let three = doc_here(&mut room, "claude", Kind::Rule, "CLAUDE.md", "Three.\n");

    take(&room, &one, false);

    let ledger = ledger(&room);
    assert!(ledger.left(&one));
    assert!(!ledger.left(&two) && !ledger.left(&three));
    assert_eq!(seen(&room, &one), Some(Seen::DeclinedHere));
}

#[test]
fn choosing_with_the_session_records_the_choice_per_place() {
    let mut room = room();
    let (standing, rival) = rivals(&mut room);
    let place = room.at(".claude/CLAUDE.md");

    take(&room, &rival, true);

    let ledger = ledger(&room);
    let recorded = ledger.chosen(&agents::path_key(Path::new(&place)));
    assert!(recorded.is_some(), "the choice is kept for the file");
    assert_eq!(stance_of(&room, &standing), places::Stance::Aside);
}

pub(crate) fn folder_named(room: &Room, name: &str, parent: FolderId) -> FolderId {
    room.session
        .state
        .folders
        .values()
        .find(|one| one.name == name && one.parent == Some(parent))
        .unwrap()
        .id
}

fn folder_under(room: &mut Room, who: &DeviceId, name: &str, parent: Option<FolderId>) -> FolderId {
    let id = ulid::Ulid::generate();
    let made = Op::FolderAdd {
        id,
        d: tisty_core::event::FolderAdd {
            name: name.to_string(),
            order: "z".to_string(),
            parent,
            icon: None,
            color: None,
            computer: None,
        },
    };
    as_laptop(room, who, vec![made]);
    id
}

pub(crate) fn named(room: &mut Room, who: &DeviceId, name: &str) {
    let said = Op::DeviceNamed {
        d: who.clone(),
        name: name.to_string(),
        os: None,
    };
    as_laptop(room, who, vec![said]);
}

fn looked_at(room: &Room, folder: FolderId) -> Option<Shown> {
    let state = &room.session.state;
    super::shown(state, &me(room), state.folders.get(&folder).unwrap())
}

#[test]
fn this_computers_folder_is_shown_as_mine_with_its_readable_name() {
    let mut room = room();
    room.bring(&[]);
    let who = me(&room);
    named(&mut room, &who, "Desk");

    let at = looked_at(&room, my_folder(&room)).unwrap();

    assert_eq!(at.computer.as_deref(), Some("Desk"));
    assert!(at.mine);
    assert!(!at.room);
}

#[test]
fn an_assistants_folder_in_this_computers_folder_is_mine_and_a_room_without_a_name() {
    let mut room = room();
    room.bring(&[]);
    let claude = folder_named(&room, "Claude", my_folder(&room));

    let at = looked_at(&room, claude).unwrap();

    assert_eq!((at.computer, at.mine, at.room), (None, true, true));
}

#[test]
fn a_folder_deeper_than_the_assistants_is_mine_but_not_a_room() {
    let mut room = room();
    room.bring(&[]);
    let claude = folder_named(&room, "Claude", my_folder(&room));
    let who = me(&room);
    let deeper = folder_under(&mut room, &who, "rules", Some(claude));

    let at = looked_at(&room, deeper).unwrap();

    assert_eq!((at.computer, at.mine, at.room), (None, true, false));
}

#[test]
fn another_computers_folder_and_its_assistants_are_not_mine() {
    let mut room = room();
    room.bring(&[]);
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    named(&mut room, &other, "Laptop");
    let claude = folder_under(&mut room, &other, "Claude", Some(folder));

    let at = looked_at(&room, folder).unwrap();
    assert_eq!(at.computer.as_deref(), Some("Laptop"));
    assert!(!at.mine && !at.room);
    let inside = looked_at(&room, claude).unwrap();
    assert_eq!(
        (inside.computer, inside.mine, inside.room),
        (None, false, true)
    );
}

#[test]
fn two_computers_called_the_same_are_told_apart_by_the_stored_name() {
    let mut room = room();
    room.bring(&[]);
    let (who, other) = (me(&room), laptop());
    let folder = laptops_folder(&mut room, &other);
    named(&mut room, &who, "Same");
    named(&mut room, &other, "Same");

    let theirs = looked_at(&room, folder).unwrap();
    let mine = looked_at(&room, my_folder(&room)).unwrap();

    assert_eq!(theirs.computer.as_deref(), Some("laptop"));
    let stored = room.session.state.folders[&my_folder(&room)].name.clone();
    assert_eq!(mine.computer, Some(stored));
}

#[test]
fn folders_outside_a_computers_folder_say_nothing() {
    let mut room = room();
    room.bring(&[]);
    let who = me(&room);
    let plain = folder_under(&mut room, &who, "Trabajo", None);
    let global = folder_named(&room, "Global", root(&room));

    assert_eq!(looked_at(&room, plain), None);
    assert_eq!(looked_at(&room, global), None);
    assert_eq!(looked_at(&room, root(&room)), None);
}

fn into_room_of(room: &Room, file: &str, folder: FolderId) -> Option<(FolderId, usize)> {
    let id = doc_of_file(room, file);
    into_room(&room.session, id, Some(folder)).map(|(made, ops)| (made, ops.len()))
}

#[test]
fn a_codex_document_dropped_at_the_root_goes_to_a_codex_folder_made_when_missing() {
    let mut room = room();
    room.bring(&[]);
    let file = doc_here(&mut room, "codex", Kind::Rule, "AGENTS.md", "Codex.\n");
    let root = root(&room);

    let (made, ops) = into_room_of(&room, &file, root).unwrap();

    assert_eq!(ops, 1, "the folder is made");
    let id = doc_of_file(&room, &file);
    let (_, ops) = into_room(&room.session, id, Some(root)).unwrap();
    room.session.commit_all(ops).unwrap();
    let state = &room.session.state;
    let folder = state.folders.get(&made).unwrap();
    assert_eq!(
        (
            folder.name.as_str(),
            folder.parent,
            folder.computer.as_ref()
        ),
        ("Codex", Some(root), None)
    );
}

#[test]
fn an_agents_folder_already_at_the_root_is_reused_not_made_again() {
    let mut room = room();
    room.bring(&[]);
    let existing = crate::answers::agents_sync::tests::root_folder(&mut room, "Claude");
    let file = doc_here(&mut room, "claude", Kind::Rule, "rules/all.md", "All.\n");

    let (made, ops) = into_room_of(&room, &file, root(&room)).unwrap();

    assert_eq!((made, ops), (existing, 0));
}

#[test]
fn a_global_skill_dropped_at_the_root_goes_to_the_global_folder() {
    let mut room = room();
    room.bring(&[]);
    let file = doc_here(&mut room, GLOBAL, Kind::Skill, "tidy/SKILL.md", "Tidy.\n");

    let (made, _) = into_room_of(&room, &file, root(&room)).unwrap();

    let state = &room.session.state;
    assert_eq!(state.folders.get(&made).unwrap().name, "Global");
    assert_eq!(made, sub_of(&room, "Global"));
}

#[test]
fn a_plain_document_or_a_move_elsewhere_than_the_root_stays_where_it_was_put() {
    let mut room = room();
    room.bring(&[]);
    let mine = my_folder(&room);
    let plain = made_in(&mut room, mine);
    let rule = doc_here(&mut room, "codex", Kind::Rule, "AGENTS.md", "Codex.\n");

    assert_eq!(into_room_of(&room, &plain, root(&room)), None);
    assert_eq!(into_room_of(&room, &rule, my_folder(&room)), None);
}

fn runs_of() -> String {
    tisty_core::called::cleaned(tisty_core::VERSION)
}

#[test]
fn another_device_on_this_very_build_is_not_an_earlier_tisty_whatever_its_number() {
    let mut room = room();
    let same = laptop();
    runs(&mut room, &same, &runs_of());
    let below = laptop();
    runs(&mut room, &below, "1.50.1");
    let unknown = laptop();
    as_laptop(
        &mut room,
        &unknown,
        vec![Op::DeviceJoin {
            d: unknown.clone(),
            k: None,
            p: None,
        }],
    );

    let earlier = earlier_elsewhere(&room.session);

    assert!(!earlier.contains(&same), "{earlier:?}");
    assert!(earlier.contains(&below) && earlier.contains(&unknown));
    assert!(!earlier.contains(&me(&room)));
}
