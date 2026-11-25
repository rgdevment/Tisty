use super::*;
use crate::answers::agents_homes::SINCE;
use crate::answers::agents_homes::tests::{
    as_laptop, body_for, laptop, laptops_folder, laptops_in, ledger, made_for_me, me, my_folder,
    named, root, runs,
};
use crate::answers::agents_sync::tests::{
    HERE, Room, doc_here, doc_of_file, every_agent_file, file_brought_from, loose, proposed, room,
    sub_of,
};
use crate::answers::agents_sync::{bodies, round_here};

const MAIN: &str = "# Claude
Tests first.
";

fn doc_moved(room: &Room, file: &str, to: FolderId) -> Option<Moving> {
    let ledger = ledger(room);
    doc_moving(
        &room.session,
        Around {
            ledger: Some(&ledger),
            homes: Some(&room.homes),
        },
        doc_of_file(room, file),
        Some(to),
    )
}

fn at_root_and_applied(room: &mut Room, name: &str) -> String {
    room.bring(&[]);
    let file = doc_here(room, "claude", Kind::Rule, name, "For all.\n");
    round_here(&mut room.session, &room.homes, HERE).unwrap();
    file
}

fn heres(moving: &Moving) -> Vec<Here> {
    moving.docs.iter().map(|one| one.here).collect()
}

fn office() -> Scope {
    Scope::Other {
        name: "Office".to_string(),
    }
}

#[test]
fn moving_inside_the_same_computer_or_out_of_agents_changes_nothing() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    let outside = ulid::Ulid::generate();
    let made = tisty_core::Op::FolderAdd {
        id: outside,
        d: tisty_core::event::FolderAdd {
            name: "Outside".to_string(),
            order: "z".to_string(),
            parent: None,
            icon: None,
            color: None,
            computer: None,
        },
    };
    room.session.commit(made).unwrap();

    assert_eq!(doc_moved(&room, &file, sub_of(&room, "Codex")), None);
    assert_eq!(doc_moved(&room, &file, outside), None);
}

#[test]
fn an_applied_rule_moved_from_the_root_into_this_computers_folder_keeps_its_file() {
    let mut room = room();
    let file = at_root_and_applied(&mut room, "rules/all.md");

    let moving = doc_moved(&room, &file, sub_of(&room, "Claude")).unwrap();

    assert_eq!((&moving.from, &moving.to), (&Scope::Everyone, &Scope::Mine));
    assert_eq!(heres(&moving), [Here::Keeps]);
}

#[test]
fn an_applied_rule_of_this_computer_moved_to_another_computers_folder_stops_here() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    let other = laptop();
    named(&mut room, &other, "Office");
    let folder = laptops_folder(&mut room, &other);

    let moving = doc_moved(&room, &file, folder).unwrap();

    assert_eq!((&moving.from, &moving.to), (&Scope::Mine, &office()));
    assert_eq!(heres(&moving), [Here::Stops]);
}

#[test]
fn a_rule_with_unapproved_words_moved_to_the_root_is_unapproved() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    let edited = read(&room.session, &file)
        .unwrap()
        .replace("Short answers.", "Changed.");
    tisty_core::docs::write(&room.session.paths.docs(), &file, &edited).unwrap();
    room.session.reload().unwrap();

    let moving = doc_moved(&room, &file, root(&room)).unwrap();

    assert_eq!(heres(&moving), [Here::Unapproved]);
}

#[test]
fn a_rule_moved_to_the_root_beside_another_for_the_same_file_asks() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let moved = made_for_me(&mut room, "CLAUDE.md", "Moved.\n");

    let moving = doc_moved(&room, &moved, root(&room)).unwrap();

    assert_eq!((&moving.from, &moving.to), (&Scope::Mine, &Scope::Everyone));
    assert_eq!(heres(&moving), [Here::Asks]);
}

#[test]
fn without_a_consented_ledger_every_document_is_untouched_and_the_move_is_still_named() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    let other = laptop();
    named(&mut room, &other, "Office");
    let folder = laptops_folder(&mut room, &other);
    let id = doc_of_file(&room, &file);

    let away = doc_moving(
        &room.session,
        Around {
            ledger: None,
            homes: Some(&room.homes),
        },
        id,
        Some(folder),
    )
    .unwrap();
    let to_all = doc_moving(
        &room.session,
        Around {
            ledger: None,
            homes: Some(&room.homes),
        },
        id,
        Some(root(&room)),
    )
    .unwrap();

    assert_eq!(heres(&away), [Here::Untouched]);
    assert_eq!(heres(&to_all), [Here::Untouched]);
    assert_eq!((away.from, away.to), (Scope::Mine, office()));
    assert_eq!((to_all.from, to_all.to), (Scope::Mine, Scope::Everyone));
}

#[test]
fn older_names_a_computer_below_the_release_until_it_reaches_it_and_never_this_one() {
    let mut room = room();
    let other = laptop();
    runs(&mut room, &other, "1.50.1");
    named(&mut room, &other, "Office");

    let before = older(&room.session);
    runs(&mut room, &other, SINCE);
    let after = older(&room.session);

    assert_eq!(before, ["Office"]);
    assert!(after.is_empty(), "{after:?}");
    assert!(!before.contains(&room.session.state.called(&me(&room))));
}

#[test]
fn older_skips_a_computer_on_this_very_build_and_names_a_lower_or_unknown_one() {
    let mut room = room();
    let (same, lower, unknown) = (laptop(), laptop(), laptop());
    let joined = tisty_core::Op::DeviceJoin {
        d: unknown.clone(),
        k: None,
        p: None,
    };
    as_laptop(&mut room, &unknown, vec![joined]);
    for (who, name) in [(&same, "Same"), (&lower, "Lower"), (&unknown, "Unknown")] {
        named(&mut room, who, name);
    }
    runs(
        &mut room,
        &same,
        &tisty_core::called::cleaned(tisty_core::VERSION),
    );
    runs(&mut room, &lower, "1.49.0");

    assert_eq!(older(&room.session), ["Lower", "Unknown"]);
}

#[test]
fn this_computers_claude_folder_moved_to_the_root_lists_its_documents() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md", ".claude/rules/tone.md"]);
    let claude = sub_of(&room, "Claude");
    let ledger = ledger(&room);

    let moving = folder_moving(
        &room.session,
        Around {
            ledger: Some(&ledger),
            homes: Some(&room.homes),
        },
        claude,
        Some(root(&room)),
    )
    .unwrap();

    assert_eq!((&moving.from, &moving.to), (&Scope::Mine, &Scope::Everyone));
    let mut files: Vec<&str> = moving.docs.iter().map(|one| one.file.as_str()).collect();
    files.sort();
    assert_eq!(files, ["CLAUDE.md", "rules/tone.md"]);
}

#[test]
fn a_folder_with_a_pending_proposal_is_refused_across_computers_and_allowed_inside_one() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    let body = read(&room.session, &file).unwrap();
    proposed(&mut room, &file, &body.replace("Short", "Brief"));
    let claude = sub_of(&room, "Claude");
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let state = &room.session.state;

    let refused = folder_resolved(state, claude, Some(folder)).unwrap_err();

    assert_eq!(refused.code, "agentProposalInFolder");
    assert!(folder_resolved(state, claude, Some(my_folder(&room))).is_ok());
}

fn three_rules(room: &mut Room, other: &DeviceId) {
    room.bring(&[]);
    doc_here(room, "claude", Kind::Rule, "rules/shared.md", "Shared.\n");
    made_for_me(room, "rules/mine.md", "Mine.\n");
    let folder = laptops_folder(room, other);
    let body = body_for("claude", "rules/theirs.md", "Theirs.\n", Some(other));
    laptops_in(room, other, folder, &body);
}

#[test]
fn a_round_writes_this_computers_and_the_roots_rules_and_never_another_computers() {
    let mut room = room();
    let other = laptop();
    three_rules(&mut room, &other);

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(room.read(".claude/rules/shared.md"), "Shared.\n");
    assert_eq!(room.read(".claude/rules/mine.md"), "Mine.\n");
    assert!(!std::path::Path::new(&room.at(".claude/rules/theirs.md")).exists());
}

#[test]
fn a_computer_without_claude_writes_no_claude_file_even_with_every_kind_of_rule() {
    let mut room = room();
    let other = laptop();
    std::fs::remove_dir_all(room.home.join(".claude")).unwrap();
    three_rules(&mut room, &other);
    let before = every_agent_file(&room);

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(every_agent_file(&room), before);
    assert!(!room.home.join(".claude").exists());
}

fn moved_for_real(room: &mut Room, file: &str, to: FolderId) {
    let id = doc_of_file(room, file);
    let ops = crate::answers::papers::beside_docs(&room.session.state, id, Some(to), None);
    room.session.commit_all(ops).unwrap();
    let homes = room.homes.clone();
    crate::answers::agents_homes::moved(&mut room.session, Some(&homes), HERE, &[file.to_string()])
        .unwrap();
}

fn seen_here(room: &Room, file: &str) -> Option<Seen> {
    gated(&room.session, &ledger(room)).seen(file)
}

#[test]
fn a_preview_of_writes_is_a_move_whose_round_writes_the_file() {
    let mut room = room();
    room.bring(&[]);
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let said = body_for("claude", "rules/theirs.md", "Theirs.\n", Some(&other));
    let file = laptops_in(&mut room, &other, folder, &said);
    let mine = my_folder(&room);

    let moving = doc_moved(&room, &file, mine).unwrap();
    assert_eq!(heres(&moving), [Here::Writes]);
    assert!(!room.home.join(".claude/rules/theirs.md").exists());
    moved_for_real(&mut room, &file, mine);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(room.read(".claude/rules/theirs.md"), "Theirs.\n");
}

#[test]
fn a_preview_of_keeps_is_a_move_whose_round_leaves_the_file_with_the_documents_text() {
    let mut room = room();
    let file = at_root_and_applied(&mut room, "rules/all.md");
    assert_eq!(room.read(".claude/rules/all.md"), "For all.\n");
    let claude = sub_of(&room, "Claude");

    let moving = doc_moved(&room, &file, claude).unwrap();
    assert_eq!(heres(&moving), [Here::Keeps]);
    moved_for_real(&mut room, &file, claude);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(room.read(".claude/rules/all.md"), "For all.\n");
    assert!(ledger(&room).took(&file));
    assert_eq!(seen_here(&room, &file), Some(Seen::Admitted));
}

#[test]
fn a_preview_of_asks_is_a_move_whose_round_writes_neither_document_and_the_choice_waits() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let own = file_brought_from(&room, "CLAUDE.md");
    let moved = made_for_me(&mut room, "CLAUDE.md", "Moved.\n");
    let root = root(&room);

    let moving = doc_moved(&room, &moved, root).unwrap();
    assert_eq!(heres(&moving), [Here::Asks]);
    moved_for_real(&mut room, &moved, root);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(room.read(".claude/CLAUDE.md"), MAIN);
    assert!(ledger(&room).waits(&moved));
    assert!(!ledger(&room).took(&moved));
    assert_eq!(seen_here(&room, &moved), Some(Seen::WaitingHere));
    assert_eq!(seen_here(&room, &own), Some(Seen::Admitted));
}

#[test]
fn a_preview_of_stops_is_a_move_after_which_this_computer_asks_about_the_file_and_leaves_it() {
    let mut room = room();
    room.bring(&[".claude/rules/tone.md"]);
    let file = file_brought_from(&room, "rules/tone.md");
    let other = laptop();
    named(&mut room, &other, "Office");
    let folder = laptops_folder(&mut room, &other);

    let moving = doc_moved(&room, &file, folder).unwrap();
    assert_eq!(heres(&moving), [Here::Stops]);
    moved_for_real(&mut room, &file, folder);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert!(ledger(&room).left(&file), "asked about, not written");
    assert_eq!(room.read(".claude/rules/tone.md"), "Short answers.\n");
    assert!(loose(&room).contains(&room.at(".claude/rules/tone.md")));
    assert_ne!(seen_here(&room, &file), Some(Seen::Admitted));
}

fn claude_rule_for_the_root(room: &mut Room) -> String {
    doc_here(room, "claude", Kind::Rule, "rules/idle.md", "Idle.\n")
}

fn unread_move_writes_nothing(room: &mut Room, file: &str) {
    let mine = my_folder(room);
    let before = every_agent_file(room);

    let moving = doc_moved(room, file, mine).unwrap();
    assert_eq!(heres(&moving), [Here::Unread]);
    moved_for_real(room, file, mine);
    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(every_agent_file(room), before);
    assert!(!room.home.join(".claude/rules/idle.md").exists());
}

#[test]
fn a_claude_rule_moved_here_without_claude_on_this_computer_is_unread_and_nothing_is_written() {
    let mut room = room();
    std::fs::remove_dir_all(room.home.join(".claude")).unwrap();
    room.bring(&[]);
    let file = claude_rule_for_the_root(&mut room);

    unread_move_writes_nothing(&mut room, &file);

    assert!(!room.home.join(".claude").exists());
}

#[test]
fn a_claude_rule_moved_here_while_claude_is_paused_is_unread_and_nothing_is_written() {
    let mut room = room();
    room.bring(&[]);
    let file = claude_rule_for_the_root(&mut room);
    let mut paused = ledger(&room);
    paused.turn("claude", false);
    paused.keep(room.session.paths.data()).unwrap();

    unread_move_writes_nothing(&mut room, &file);
}

#[test]
fn a_move_from_another_computer_leaves_this_computers_file_written_and_its_own() {
    let mut room = room();
    let file = at_root_and_applied(&mut room, "rules/all.md");
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let body = read(&room.session, &file).unwrap();
    let marked = agents::meant_for(&body, Some(&other)).unwrap();
    crate::answers::agents_sync::rewritten(&mut room.session, &file, &marked).unwrap();
    let id = doc_of_file(&room, &file);
    let filed = tisty_core::Op::DocMove {
        id,
        d: tisty_core::event::Filed {
            folder: Some(Some(folder)),
            page_of: None,
            order: None,
        },
    };
    as_laptop(&mut room, &other, vec![filed]);
    assert_eq!(seen_here(&room, &file), Some(Seen::OtherComputer));
    let before = every_agent_file(&room);

    round_here(&mut room.session, &room.homes, HERE).unwrap();

    assert_eq!(every_agent_file(&room), before, "not a byte changes");
    assert_eq!(room.read(".claude/rules/all.md"), "For all.\n");
    assert!(!ledger(&room).left(&file), "this computer is not asked");
    assert!(loose(&room).is_empty(), "the file is not left loose");
    let own = bodies(&room.session)
        .into_iter()
        .map(|(one, _)| one)
        .find(|one| computer_of_file(&room, one) == Some(me(&room)))
        .expect("the file became a document of this computer");
    assert_eq!(seen_here(&room, &own), Some(Seen::Admitted));
    assert_eq!(seen_here(&room, &file), Some(Seen::OtherComputer));
}

fn computer_of_file(room: &Room, file: &str) -> Option<DeviceId> {
    let kept = kept_of(&room.session, file).unwrap();
    agents::computer_of(&room.session.state, kept.folder)
}
