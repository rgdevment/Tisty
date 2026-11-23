use super::{Answered, Dropped, Ledger, Path, Reach, dropped, reach, root_of, round_now};
use crate::answers::agents_sync::tests::{
    HERE, Room, doc_here, doc_of_file, file_brought_from, global_doc, moved_out, put, room,
};
use crate::answers::agents_sync::{bodies, emptied, loosened, turned};
use linked_test::linked_folder;
use tisty_core::agents::{GLOBAL, Kind};

#[path = "../../../../crates/tisty-core/src/agents/linked_test.rs"]
mod linked_test;

const CLAUDE_SKILL: &str = ".claude/skills/voice/SKILL.md";
const SHARED_SKILL: &str = ".agents/skills/voice/SKILL.md";
const SAID: &str = "Speak plainly.\n";
const WITH_GLOBAL: &str = "Spanish.\n\n# Claude\nTests first.\n";

fn skill_everywhere(room: &mut Room) -> String {
    room.bring(&[".claude/CLAUDE.md", ".claude/rules/tone.md"]);
    let file = doc_here(room, GLOBAL, Kind::Skill, "voice/SKILL.md", SAID);
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(CLAUDE_SKILL), SAID);
    assert_eq!(room.read(SHARED_SKILL), SAID);
    doc_of_file(room, &file).to_string()
}

fn reached(room: &Room, ids: &[String]) -> Reach {
    reach(&room.session, &room.homes, HERE, ids)
}

fn deleted(room: &mut Room, ids: &[String], away: bool) -> Dropped {
    let shown = reached(room, ids);
    deleted_as_shown(room, ids, away, &shown)
}

fn deleted_as_shown(room: &mut Room, ids: &[String], away: bool, shown: &Reach) -> Dropped {
    let answer = Answered {
        away,
        at: shown.at.clone(),
    };
    dropped(&mut room.session, &room.homes, HERE, ids, &answer).unwrap()
}

fn there(room: &Room, name: &str) -> bool {
    Path::new(&room.at(name)).exists()
}

fn names(all: &[&str]) -> Vec<String> {
    all.iter().map(ToString::to_string).collect()
}

#[test]
fn deleting_a_skill_three_agents_read_asks_once_and_takes_every_copy_with_a_backup() {
    let mut room = room();
    let id = skill_everywhere(&mut room);
    let asked = reached(&room, std::slice::from_ref(&id));
    assert_eq!(asked.agents, names(&["Claude", "Codex", "Copilot"]));
    assert!(asked.changed.is_empty() && asked.paused.is_empty());
    assert!(there(&room, CLAUDE_SKILL), "asking takes nothing away");

    let done = deleted(&mut room, &[id], true);
    assert_eq!(done.gone.len(), 1);
    assert!(
        done.refused.is_empty() && done.untaken.is_empty(),
        "{done:?}"
    );
    assert!(!there(&room, CLAUDE_SKILL) && !there(&room, SHARED_SKILL));
    assert_eq!(room.kept(CLAUDE_SKILL), 1);
    assert_eq!(room.kept(SHARED_SKILL), 1);
    assert!(room.look().loose.is_empty());
    assert_eq!(room.read(".claude/rules/tone.md"), "Short answers.\n");
}

#[test]
fn only_from_tisty_touches_no_agent_file_and_leaves_them_listed_without_asking() {
    let mut room = room();
    let id = skill_everywhere(&mut room);
    let done = deleted(&mut room, &[id], false);
    assert!(done.untaken.is_empty(), "{done:?}");
    assert_eq!(room.read(CLAUDE_SKILL), SAID);
    assert_eq!(room.read(SHARED_SKILL), SAID);
    assert_eq!(room.kept(CLAUDE_SKILL), 0);

    let look = room.look();
    assert_eq!(look.loose.len(), 2, "{look:?}");
    assert!(look.loose.iter().all(|one| one.stayed));
    assert!(
        !look.asks,
        "a file the person let stay never asks by itself"
    );
    assert_eq!(look.changes, 0);
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    let round = round_now(&room.session, &room.homes, &mut ledger);
    assert!(round.loose.iter().all(|one| one.stayed));
    assert_eq!(room.read(SHARED_SKILL), SAID, "no round writes over it");

    loosened(
        &room.session,
        &room.homes,
        HERE,
        &room.at(CLAUDE_SKILL),
        true,
    )
    .unwrap();
    assert!(!there(&room, CLAUDE_SKILL));
    assert_eq!(room.kept(CLAUDE_SKILL), 1);
    let left: Vec<String> = room.look().loose.into_iter().map(|one| one.at).collect();
    assert_eq!(
        left,
        [room.at(SHARED_SKILL)],
        "changing one's mind later still takes it away"
    );
}

#[test]
fn a_copy_changed_outside_is_said_apart_and_never_taken_unseen_even_if_it_changed_after_asking() {
    let mut room = room();
    let id = skill_everywhere(&mut room);
    let before = reached(&room, std::slice::from_ref(&id));
    assert!(before.changed.is_empty());
    put(Path::new(&room.at(SHARED_SKILL)), "Changed by hand.\n");
    let after = reached(&room, std::slice::from_ref(&id));
    assert_eq!(after.agents, names(&["Claude"]));
    assert_eq!(after.changed, names(&["Codex", "Copilot"]));

    deleted_as_shown(&mut room, &[id], true, &before);
    assert!(!there(&room, CLAUDE_SKILL));
    assert_eq!(room.read(SHARED_SKILL), "Changed by hand.\n");
    assert_eq!(room.kept(SHARED_SKILL), 0);
    let look = room.look();
    let [one] = look.loose.as_slice() else {
        panic!("{look:?}");
    };
    assert!(one.changed && !one.stayed);
    assert!(
        look.asks,
        "the person sees it in the question of loose files"
    );
}

#[test]
fn a_copy_the_question_never_named_stays_even_when_its_agent_syncs_again_before_the_answer() {
    let mut room = room();
    let id = skill_everywhere(&mut room);
    turned(&room.session, &room.homes, HERE, Some("codex"), false).unwrap();
    let shown = reached(&room, std::slice::from_ref(&id));
    assert_eq!(shown.agents, names(&["Claude"]));
    turned(&room.session, &room.homes, HERE, Some("codex"), true).unwrap();

    deleted_as_shown(&mut room, std::slice::from_ref(&id), true, &shown);
    assert!(!there(&room, CLAUDE_SKILL));
    assert_eq!(
        room.read(SHARED_SKILL),
        SAID,
        "the box said that copy stays"
    );
    let look = room.look();
    assert_eq!(look.loose.len(), 1, "{look:?}");
    assert!(
        !look.loose[0].stayed && look.asks,
        "it waits for its own question"
    );
}

#[test]
fn a_paused_agent_is_named_apart_keeps_its_copy_and_is_asked_once_it_syncs_again() {
    let mut room = room();
    let id = skill_everywhere(&mut room);
    turned(&room.session, &room.homes, HERE, Some("codex"), false).unwrap();
    let asked = reached(&room, std::slice::from_ref(&id));
    assert_eq!(asked.agents, names(&["Claude"]));
    assert_eq!(
        asked.paused,
        names(&["Codex"]),
        "only the agent on pause, not the others that read its folder"
    );

    let done = deleted(&mut room, &[id], true);
    assert!(done.untaken.is_empty(), "{done:?}");
    assert!(!there(&room, CLAUDE_SKILL));
    assert_eq!(room.read(SHARED_SKILL), SAID);
    assert!(room.look().loose.is_empty());

    turned(&room.session, &room.homes, HERE, Some("codex"), true).unwrap();
    let look = room.look();
    assert_eq!(look.loose.len(), 1, "{look:?}");
    assert!(!look.loose[0].stayed && look.asks);
}

fn global_beside_claude(room: &mut Room) -> String {
    room.bring(&[".claude/CLAUDE.md"]);
    global_doc(room, "Spanish.\n");
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".claude/CLAUDE.md"), WITH_GLOBAL);
    let global = bodies(&room.session)
        .into_iter()
        .find(|(_, body)| body.contains("Spanish."))
        .unwrap()
        .0;
    doc_of_file(room, &global).to_string()
}

#[test]
fn a_file_another_document_still_places_is_never_asked_about_and_only_rewritten() {
    let mut room = room();
    let id = global_beside_claude(&mut room);
    let shown = reached(&room, std::slice::from_ref(&id));
    assert!(shown.at.is_empty() && shown.agents.is_empty());
    assert_eq!(shown.rewritten, names(&["Claude"]));
    let done = deleted(&mut room, &[id], true);
    assert!(done.untaken.is_empty());
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
    assert!(room.look().loose.is_empty());
}

#[test]
fn a_file_a_document_holds_is_never_asked_about_nor_taken() {
    let mut room = room();
    let id = global_beside_claude(&mut room);
    let claude = file_brought_from(&room, "CLAUDE.md");
    let words = tisty_core::docs::read(&room.session.paths.docs(), &claude).unwrap();
    let unapproved = words.replace("Tests first.", "Tests later.");
    tisty_core::docs::write(&room.session.paths.docs(), &claude, &unapproved).unwrap();
    room.session.reload().unwrap();
    assert_eq!(reached(&room, std::slice::from_ref(&id)), Reach::default());
    let backups = room.kept(".claude/CLAUDE.md");
    deleted(&mut room, &[id], true);
    assert_eq!(room.read(".claude/CLAUDE.md"), WITH_GLOBAL);
    assert_eq!(room.kept(".claude/CLAUDE.md"), backups);
}

#[test]
fn a_file_left_loose_some_other_way_is_never_taken_by_deleting_something_else() {
    let mut room = room();
    let id = skill_everywhere(&mut room);
    let tone = file_brought_from(&room, "rules/tone.md");
    let tone_id = doc_of_file(&room, &tone);
    let ops = crate::answers::papers::beside_docs(&room.session.state, tone_id, None, None);
    room.session.commit_all(ops).unwrap();
    assert_eq!(room.look().loose.len(), 1);

    deleted(&mut room, &[id], true);
    assert_eq!(room.read(".claude/rules/tone.md"), "Short answers.\n");
    let left: Vec<String> = room.look().loose.into_iter().map(|one| one.at).collect();
    assert_eq!(
        left,
        [room.at(".claude/rules/tone.md")],
        "it waits for its own question"
    );
}

#[test]
fn several_deleted_at_once_ask_once_for_all_their_agents() {
    let mut room = room();
    let skill = skill_everywhere(&mut room);
    let tone = doc_of_file(&room, &file_brought_from(&room, "rules/tone.md")).to_string();
    let ids = [skill, tone];
    let asked = reached(&room, &ids);
    assert_eq!(asked.agents, names(&["Claude", "Codex", "Copilot"]));

    let done = deleted(&mut room, &ids, true);
    assert_eq!(done.gone.len(), 2);
    assert!(!there(&room, CLAUDE_SKILL) && !there(&room, SHARED_SKILL));
    assert!(!there(&room, ".claude/rules/tone.md"));
    assert_eq!(room.kept(".claude/rules/tone.md"), 1);
}

#[test]
fn a_document_that_cannot_be_deleted_takes_none_of_its_files_and_says_why() {
    let mut room = room();
    let id = skill_everywhere(&mut room);
    let doc = id.parse().unwrap();
    room.session
        .commit(tisty_core::Op::DocLock { id: doc })
        .unwrap();
    let done = deleted(&mut room, &[id, "not-an-id".to_string()], true);
    assert!(done.gone.is_empty());
    assert_eq!(done.refused.len(), 2);
    assert_eq!(done.refused[0].refusal.code, "documentLocked");
    assert_eq!(room.read(CLAUDE_SKILL), SAID);
    assert_eq!(room.read(SHARED_SKILL), SAID);
}

#[test]
fn a_computer_without_agents_yes_deletes_as_ever_and_never_touches_an_agent_file() {
    let mut room = room();
    let id = skill_everywhere(&mut room);
    let data = room.session.paths.data().to_path_buf();
    let mut ledger = Ledger::open(&data, HERE);
    ledger.consent(false);
    ledger.keep(&data).unwrap();
    assert_eq!(reached(&room, std::slice::from_ref(&id)), Reach::default());

    let done = deleted(&mut room, &[id], true);
    assert_eq!(done.gone.len(), 1);
    assert_eq!(room.read(CLAUDE_SKILL), SAID);
    assert_eq!(room.read(SHARED_SKILL), SAID);
    assert_eq!(room.kept(CLAUDE_SKILL), 0);
    assert!(
        Ledger::open(&data, HERE)
            .written(Path::new(&room.at(CLAUDE_SKILL)))
            .is_some()
    );
}

#[test]
fn a_skill_named_again_after_only_from_tisty_asks_before_writing_over_what_was_left() {
    let mut room = room();
    let id = skill_everywhere(&mut room);
    deleted(&mut room, &[id], false);
    let at = room.at(CLAUDE_SKILL);
    assert!(Ledger::open(room.session.paths.data(), HERE).stays(Path::new(&at)));

    doc_here(
        &mut room,
        GLOBAL,
        Kind::Skill,
        "voice/SKILL.md",
        "Speak gently.\n",
    );
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    let round = round_now(&room.session, &room.homes, &mut ledger);
    assert!(round.both.iter().any(|one| one.at == at), "{round:?}");
    assert_eq!(
        room.read(CLAUDE_SKILL),
        SAID,
        "nothing is written over it unasked"
    );
    let before = room.kept(CLAUDE_SKILL);

    crate::answers::agents_sync::kept_side(&mut room.session, &room.homes, HERE, &at, true)
        .unwrap();
    assert_eq!(room.read(CLAUDE_SKILL), "Speak gently.\n");
    assert_eq!(
        room.kept(CLAUDE_SKILL),
        before + 1,
        "what was left is kept first"
    );
    assert!(!Ledger::open(room.session.paths.data(), HERE).stays(Path::new(&at)));
}

#[test]
fn a_copy_a_paused_reader_keeps_is_said_apart_and_stays_whoever_owns_the_folder() {
    let mut room = room();
    let id = skill_everywhere(&mut room);
    turned(&room.session, &room.homes, HERE, Some("copilot"), false).unwrap();
    let shown = reached(&room, std::slice::from_ref(&id));
    assert_eq!(
        shown.agents,
        names(&["Claude"]),
        "a paused agent is never named as losing it"
    );
    assert_eq!(shown.paused, names(&["Copilot"]));
    assert_eq!(shown.at, [room.at(CLAUDE_SKILL)]);

    deleted_as_shown(&mut room, std::slice::from_ref(&id), true, &shown);
    assert!(!there(&room, CLAUDE_SKILL));
    assert_eq!(room.read(SHARED_SKILL), SAID, "Copilot still reads it");
    let shared = room.at(SHARED_SKILL);
    assert!(
        room.look().loose.iter().all(|one| one.at != shared),
        "nothing asks about it while an agent that reads it is paused"
    );
    turned(&room.session, &room.homes, HERE, Some("copilot"), true).unwrap();
    assert!(room.look().loose.iter().any(|one| one.at == shared));
}

#[test]
fn only_from_tisty_never_marks_a_copy_changed_outside_so_its_question_still_comes() {
    let mut room = room();
    let id = skill_everywhere(&mut room);
    put(Path::new(&room.at(SHARED_SKILL)), "Changed by hand.\n");
    let done = deleted(&mut room, &[id], false);
    assert!(done.unmarked.is_empty());
    let look = room.look();
    let shared = look
        .loose
        .iter()
        .find(|one| one.at == room.at(SHARED_SKILL))
        .unwrap();
    assert!(shared.changed && !shared.stayed && look.asks);
    let claude = look
        .loose
        .iter()
        .find(|one| one.at == room.at(CLAUDE_SKILL))
        .unwrap();
    assert!(claude.stayed);
}

#[test]
fn a_file_another_rule_still_makes_up_is_named_as_written_again() {
    let mut room = room();
    let id = global_beside_claude(&mut room);
    let shown = reached(&room, std::slice::from_ref(&id));
    assert!(shown.rewritten.contains(&"Claude".to_string()), "{shown:?}");
    assert!(
        shown
            .at
            .iter()
            .all(|at| *at != room.at(".claude/CLAUDE.md"))
    );
}

#[test]
fn a_path_the_question_never_showed_is_never_taken_even_if_it_is_loose_by_then() {
    let mut room = room();
    let id = skill_everywhere(&mut room);
    let shown = Reach {
        at: vec![room.at(CLAUDE_SKILL)],
        ..Reach::default()
    };
    deleted_as_shown(&mut room, std::slice::from_ref(&id), true, &shown);
    assert!(!there(&room, CLAUDE_SKILL));
    assert_eq!(room.read(SHARED_SKILL), SAID);
    let other = Reach {
        at: vec![room.at(".claude/nowhere/SKILL.md")],
        ..Reach::default()
    };
    let tone = doc_of_file(&room, &file_brought_from(&room, "rules/tone.md")).to_string();
    deleted_as_shown(&mut room, &[tone], true, &other);
    assert_eq!(
        room.read(".claude/rules/tone.md"),
        "Short answers.\n",
        "a path the ledger never named is nothing to act on"
    );
}

#[test]
fn geminis_rules_while_the_person_took_its_line_out_count_as_paused_and_stay() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    std::fs::create_dir_all(room.home.join(".gemini")).unwrap();
    put(Path::new(&room.at(".gemini/GEMINI.md")), "- likes tea\n");
    global_doc(&mut room, "Spanish.\n");
    crate::answers::agents_sync::round_here(&mut room.session, &room.homes, HERE).unwrap();
    assert_eq!(room.read(".gemini/tisty-rules.md"), "Spanish.\n");
    let gemini = room.at(".gemini/GEMINI.md");
    put(Path::new(&gemini), "- likes tea\n");
    let data = room.session.paths.data().to_path_buf();
    assert!(Ledger::open(&data, HERE).led(Path::new(&gemini)));

    let global = bodies(&room.session)
        .into_iter()
        .find(|(_, body)| body.contains("Spanish."))
        .unwrap()
        .0;
    let id = doc_of_file(&room, &global).to_string();
    let shown = reached(&room, std::slice::from_ref(&id));
    assert!(shown.paused.contains(&"Gemini".to_string()), "{shown:?}");
    assert!(!shown.at.contains(&room.at(".gemini/tisty-rules.md")));

    deleted_as_shown(&mut room, &[id], true, &shown);
    assert_eq!(room.read(".gemini/tisty-rules.md"), "Spanish.\n");
    assert!(
        Ledger::open(&data, HERE).led(Path::new(&gemini)),
        "the line the person took out stays their word"
    );
    let rules = room.at(".gemini/tisty-rules.md");
    assert!(
        room.look().loose.iter().all(|one| one.at != rules),
        "while Gemini waits for its line, its file is not asked about"
    );
    put(Path::new(&gemini), "- likes tea\n@tisty-rules.md\n");
    assert!(
        room.look().loose.iter().any(|one| one.at == rules),
        "with the line back, Gemini reads it again and it is asked about"
    );
}

#[cfg(any(windows, target_os = "macos"))]
#[test]
fn a_skill_named_in_other_capitals_keeps_the_one_file_both_place() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let upper = doc_here(&mut room, GLOBAL, Kind::Skill, "Voice/SKILL.md", SAID);
    let lower = doc_here(&mut room, GLOBAL, Kind::Skill, "voice/SKILL.md", SAID);
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert!(there(&room, CLAUDE_SKILL));
    let id = doc_of_file(&room, &lower).to_string();
    let shown = reached(&room, std::slice::from_ref(&id));
    assert!(shown.at.is_empty(), "{shown:?}");
    let both_spellings = Reach {
        at: vec![
            room.at(CLAUDE_SKILL),
            room.at(".claude/skills/Voice/SKILL.md"),
        ],
        ..Reach::default()
    };
    deleted_as_shown(&mut room, &[id], true, &both_spellings);
    assert_eq!(room.read(CLAUDE_SKILL), SAID, "«Voice» still places it");
    assert!(
        room.session
            .state
            .docs
            .values()
            .any(|one| one.file == upper)
    );
}

#[test]
fn a_copy_behind_a_link_is_named_as_not_taken_and_stays() {
    let mut room = room();
    let id = skill_everywhere(&mut room);
    let skills = room.home.join(".claude").join("skills");
    let elsewhere = room.home.join("dotfiles").join("voice");
    std::fs::create_dir_all(elsewhere.parent().unwrap()).unwrap();
    std::fs::rename(skills.join("voice"), &elsewhere).unwrap();
    linked_folder(&skills.join("voice"), &elsewhere);
    let shown = reached(&room, std::slice::from_ref(&id));

    let done = deleted_as_shown(&mut room, &[id], true, &shown);
    let [one] = done.untaken.as_slice() else {
        panic!("{done:?}");
    };
    assert_eq!(one.at, room.at(CLAUDE_SKILL));
    assert_eq!(one.refusal.code, "agentLinked");
    assert_eq!(
        std::fs::read_to_string(elsewhere.join("SKILL.md")).unwrap(),
        SAID
    );
}

#[test]
fn agents_emptied_after_only_from_tisty_goes_and_leaves_the_files_with_the_agents() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let tone = doc_of_file(&room, &file_brought_from(&room, "rules/tone.md")).to_string();
    deleted(&mut room, &[tone], false);
    assert!(emptied(&mut room.session, &room.homes, HERE).unwrap());
    assert_eq!(room.read(".claude/rules/tone.md"), "Short answers.\n");
}

#[test]
fn a_file_left_and_brought_back_loses_its_mark_and_asks_once_it_is_loose_again() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md", ".claude/rules/tone.md"]);
    let tone = doc_of_file(&room, &file_brought_from(&room, "rules/tone.md")).to_string();
    deleted(&mut room, &[tone], false);
    let at = room.at(".claude/rules/tone.md");
    assert!(Ledger::open(room.session.paths.data(), HERE).stays(Path::new(&at)));

    room.bring_as(&[".claude/rules/tone.md"], None);
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert!(
        !Ledger::open(room.session.paths.data(), HERE).stays(Path::new(&at)),
        "placed again, the file is Tisty's once more"
    );

    let file = file_brought_from(&room, "rules/tone.md");
    moved_out(&mut room, &file);
    let look = room.look();
    assert_eq!(look.loose.len(), 1, "{look:?}");
    assert!(!look.loose[0].stayed && look.asks);
}

#[test]
fn with_agents_gone_only_what_the_person_left_is_listed_and_taken_away_with_a_backup() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md", ".claude/rules/tone.md"]);
    let tone = doc_of_file(&room, &file_brought_from(&room, "rules/tone.md")).to_string();
    deleted(&mut room, &[tone], false);
    let claude = file_brought_from(&room, "CLAUDE.md");
    moved_out(&mut room, &claude);
    assert_eq!(room.look().loose.len(), 2);

    let root = root_of(&room.session.state).unwrap();
    let mut gone: Vec<_> = crate::answers::agents_sync::tree(&room.session.state, root)
        .into_iter()
        .collect();
    gone.sort_by_key(|at| std::cmp::Reverse(room.session.state.depth(Some(*at))));
    let ops = gone
        .into_iter()
        .map(|id| tisty_core::Op::FolderDelete { id })
        .collect();
    room.session.commit_all(ops).unwrap();

    let at = room.at(".claude/rules/tone.md");
    put(Path::new(&at), "Changed by hand.\n");
    let look = room.look();
    assert_eq!(look.folder, None);
    let [one] = look.loose.as_slice() else {
        panic!("only what the person left, nothing else: {look:?}");
    };
    assert_eq!(one.at, at);
    assert!(one.stayed && one.changed && !look.asks);

    loosened(&room.session, &room.homes, HERE, &at, true).unwrap();
    assert!(!there(&room, ".claude/rules/tone.md"));
    assert_eq!(room.kept(".claude/rules/tone.md"), 1);
    assert_eq!(room.read(".claude/CLAUDE.md"), "# Claude\nTests first.\n");
}
