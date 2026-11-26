use super::*;
use crate::answers::agents_sync::tests::{Room, room};
use crate::answers::agents_sync::{gated, here as machine};
use tisty_core::Op;

#[test]
fn a_hooks_state_reads_from_the_gate_and_its_yes() {
    assert_eq!(state_of(Seen::HookRegistered, None), "registered");
    assert_eq!(state_of(Seen::HookWaiting, None), "waiting");
    assert_eq!(state_of(Seen::Unread, None), "away");
}

#[test]
fn every_event_has_a_sample_an_agent_would_send() {
    for event in Event::ALL {
        let said: serde_json::Value = serde_json::from_slice(&sample(event, Tools::Shell)).unwrap();
        assert_eq!(said["session_id"], TRY_AS);
    }
}

fn opened_room() -> Room {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    room
}

fn script_name() -> &'static str {
    match cfg!(windows) {
        true => "guard.ps1",
        false => "guard.sh",
    }
}

fn trigger(agents: &[&str]) -> Trigger {
    Trigger {
        event: Event::BeforeTool,
        tools: Tools::Shell,
        timeout: 20,
        agents: agents.iter().map(|one| (*one).to_string()).collect(),
    }
}

// Agents no catalogue knows, so registering one never reaches a file of the person's.
fn hook_in_agents(room: &mut Room, name: &str, script: &str) -> String {
    let body = agents::hook_doc(name, None, &trigger(&["nobody"]), script).unwrap();
    let root = super::super::agents_sync::root_of(&room.session.state).unwrap();
    let device = room.session.store.device().clone();
    let made = tisty_core::docs::create(&room.session.paths.docs(), &device, &body).unwrap();
    room.session
        .commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                file: made.id.clone(),
                folder: Some(root),
                order: "a".to_string(),
                said: Some(tisty_core::event::Said::of(&body).agreed()),
                ..Default::default()
            },
        })
        .unwrap();
    made.id
}

fn rewritten(room: &mut Room, file: &str, body: &str) {
    let id = room
        .session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
        .id;
    std::fs::write(room.session.paths.docs().join(format!("{file}.md")), body).unwrap();
    room.session
        .commit(Op::DocSaid {
            id,
            d: tisty_core::event::Said::of(body).agreed(),
        })
        .unwrap();
}

fn body_of(room: &Room, file: &str) -> String {
    tisty_core::docs::read(&room.session.paths.docs(), file).unwrap()
}

fn state(room: &Room, file: &str) -> &'static str {
    looked(&room.session, file).unwrap().state
}

fn code<T>(answer: Answer<T>) -> &'static str {
    answer.err().expect("refused").code
}

fn ledger(room: &Room) -> Ledger {
    Ledger::open(room.session.paths.data(), &machine())
}

fn registering(room: &Room, file: &str) -> Answer<HookLook> {
    let print = tisty_core::docs::print_of_text(&body_of(room, file));
    registered(&room.session, file, &print)
}

fn new_hook(name: &str, everyone: bool) -> NewHook {
    NewHook {
        name: name.to_string(),
        event: "before-tool".to_string(),
        tools: "shell".to_string(),
        timeout: 5,
        agents: vec!["nobody".to_string()],
        everyone,
    }
}

fn folder_of(room: &Room, file: &str) -> tisty_core::model::Folder {
    let at = room
        .session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .and_then(|one| one.folder)
        .unwrap();
    room.session.state.folders[&at].clone()
}

#[test]
fn a_hook_for_everyone_is_made_in_global_without_a_computer_mark() {
    let mut room = opened_room();
    let made = made(&mut room.session, &new_hook("guard.sh", true)).unwrap();
    let folder = folder_of(&room, &made.id);
    assert_eq!(folder.name, crate::wiring::agents::GLOBAL_FOLDER);
    assert_eq!(folder.computer, None);
    let card = Card::of(&body_of(&room, &made.id)).unwrap();
    assert_eq!(card.kind, agents::Kind::Hook);
    assert_eq!(card.computer, None);
    assert_eq!(card.name, "guard.sh");
    assert_eq!(state(&room, &made.id), "waiting");
}

#[test]
fn a_hook_for_this_computer_is_made_in_its_own_folder_with_the_on_mark() {
    let mut room = opened_room();
    let me = room.session.store.device().clone();
    let made = made(&mut room.session, &new_hook("mine.sh", false)).unwrap();
    let folder = folder_of(&room, &made.id);
    assert_eq!(folder.computer, Some(me.clone()));
    let body = body_of(&room, &made.id);
    assert!(body.contains(&format!("on-{}", me.0)), "{body}");
    assert_eq!(Card::of(&body).unwrap().computer, Some(me));
}

#[test]
fn a_hook_is_never_made_with_a_bad_name_a_taken_one_or_a_trigger_it_cannot_read() {
    let mut room = opened_room();
    made(&mut room.session, &new_hook("guard.sh", true)).unwrap();
    for (asked, said) in [
        (new_hook("Guard.exe", true), "agentHookNameBad"),
        (new_hook("guard.sh", false), "agentHookNameTaken"),
        (new_hook("guard.sh", true), "agentHookNameTaken"),
        (
            NewHook {
                event: "later".into(),
                ..new_hook("x.sh", true)
            },
            "agentHookUnread",
        ),
        (
            NewHook {
                tools: "web".into(),
                ..new_hook("x.sh", true)
            },
            "agentHookUnread",
        ),
    ] {
        let refused = made(&mut room.session, &asked).err().map(|one| one.code);
        assert_eq!(refused, Some(said), "{asked:?}");
    }
}

#[test]
fn registering_needs_the_print_just_shown_and_a_hook_that_waits() {
    let mut room = opened_room();
    let file = hook_in_agents(&mut room, "guard.sh", "exit 2\n");
    assert_eq!(state(&room, &file), "waiting");
    assert_eq!(
        code(registered(&room.session, &file, "not-the-print")),
        "agentHookChangedSince"
    );
    assert!(ledger(&room).hook_yes(&file).is_none());

    let look = registering(&room, &file).unwrap();
    assert_eq!(look.state, "registered");
    assert!(look.kept);
    let yes = ledger(&room).hook_yes(&file).cloned().unwrap();
    assert_eq!(yes.name, "guard.sh");
    assert_eq!(yes.agents, ["nobody"]);
    assert!(agents::script_kept(room.session.paths.data(), &yes).is_some());
}

#[test]
fn editing_the_body_of_a_registered_hook_makes_it_changed_and_its_old_print_is_refused() {
    let mut room = opened_room();
    let file = hook_in_agents(&mut room, "guard.sh", "exit 2\n");
    let old_print = tisty_core::docs::print_of_text(&body_of(&room, &file));
    registering(&room, &file).unwrap();
    let yes = ledger(&room).hook_yes(&file).cloned().unwrap();

    let edited = body_of(&room, &file).replace("exit 2", "exit 0");
    rewritten(&mut room, &file, &edited);
    assert_eq!(state(&room, &file), "changed");
    assert_eq!(
        code(registered(&room.session, &file, &old_print)),
        "agentHookChangedSince"
    );
    assert!(ledger(&room).hook_yes(&file).is_some());

    let mut kept = ledger(&room);
    let gate = gated(&room.session, &kept);
    assert!(!retired(&room.session, &gate, &mut kept));
    assert!(kept.hook_yes(&file).is_some());
    assert!(agents::script_kept(room.session.paths.data(), &yes).is_none());
}

#[test]
fn registering_again_after_an_edit_replaces_the_yes_and_the_script() {
    let mut room = opened_room();
    let file = hook_in_agents(&mut room, "guard.sh", "exit 2\n");
    registering(&room, &file).unwrap();
    let edited = body_of(&room, &file).replace("exit 2", "exit 0");
    rewritten(&mut room, &file, &edited);
    assert_eq!(registering(&room, &file).unwrap().state, "registered");
    let yes = ledger(&room).hook_yes(&file).cloned().unwrap();
    let kept = agents::script_kept(room.session.paths.data(), &yes).unwrap();
    assert_eq!(kept, b"exit 0\n");
}

#[test]
fn registering_after_the_name_changed_drops_the_old_names_script_and_yes() {
    let mut room = opened_room();
    let file = hook_in_agents(&mut room, "guard.sh", "exit 2\n");
    registering(&room, &file).unwrap();
    let old = ledger(&room).hook_yes(&file).cloned().unwrap();
    let renamed = body_of(&room, &file).replace("guard.sh", "watch.sh");
    rewritten(&mut room, &file, &renamed);
    let look = registering(&room, &file).unwrap();
    assert_eq!(look.name, "watch.sh");
    let now = ledger(&room);
    assert_eq!(now.hooks().count(), 1);
    assert_eq!(now.hook_yes(&file).unwrap().name, "watch.sh");
    assert!(!agents::script_at(room.session.paths.data(), "guard.sh").exists());
    assert!(agents::script_at(room.session.paths.data(), "watch.sh").exists());
    assert!(agents::script_kept(room.session.paths.data(), &old).is_none());
}

#[test]
fn a_name_another_registered_file_holds_is_refused() {
    let mut room = opened_room();
    let first = hook_in_agents(&mut room, "guard.sh", "exit 2\n");
    let second = hook_in_agents(&mut room, "other.sh", "exit 0\n");
    registering(&room, &first).unwrap();
    let renamed = body_of(&room, &second).replace("other.sh", "guard.sh");
    rewritten(&mut room, &second, &renamed);
    assert_eq!(code(registering(&room, &second)), "agentHookNameTaken");
    assert!(ledger(&room).hook_yes(&second).is_none());
    assert_eq!(ledger(&room).hooks().count(), 1);
}

#[test]
fn a_hook_whose_document_is_deleted_is_unregistered_and_its_script_dropped() {
    let mut room = opened_room();
    let file = hook_in_agents(&mut room, "guard.sh", "exit 2\n");
    registering(&room, &file).unwrap();
    let yes = ledger(&room).hook_yes(&file).cloned().unwrap();

    let mut kept = ledger(&room);
    let gate = gated(&room.session, &kept);
    assert!(!retired(&room.session, &gate, &mut kept));
    assert!(kept.hook_yes(&file).is_some());

    let id = room
        .session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
        .id;
    room.session.commit(Op::DocArchive { id }).unwrap();
    let gate = gated(&room.session, &kept);
    assert!(retired(&room.session, &gate, &mut kept));
    assert!(kept.hook_yes(&file).is_none());
    assert!(agents::script_kept(room.session.paths.data(), &yes).is_none());
}

#[test]
fn a_hook_moved_out_of_the_agents_folder_is_unregistered_too() {
    let mut room = opened_room();
    let file = hook_in_agents(&mut room, "guard.sh", "exit 2\n");
    registering(&room, &file).unwrap();
    let id = room
        .session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
        .id;
    room.session
        .commit(Op::DocMove {
            id,
            d: tisty_core::event::Filed {
                folder: Some(None),
                page_of: None,
                order: None,
            },
        })
        .unwrap();
    let mut kept = ledger(&room);
    let gate = gated(&room.session, &kept);
    assert!(retired(&room.session, &gate, &mut kept));
    assert!(kept.hook_yes(&file).is_none());
}

#[test]
fn unregistering_drops_the_yes_and_the_script_and_the_hook_waits_again() {
    let mut room = opened_room();
    let file = hook_in_agents(&mut room, "guard.sh", "exit 2\n");
    registering(&room, &file).unwrap();
    let look = unregistered(&room.session, &file).unwrap();
    assert_eq!(look.state, "waiting");
    assert!(!look.kept);
    assert!(ledger(&room).hook_yes(&file).is_none());
    assert!(!agents::script_at(room.session.paths.data(), "guard.sh").exists());
    assert_eq!(unregistered(&room.session, &file).unwrap().state, "waiting");
}

#[test]
fn a_dry_run_refuses_a_print_that_was_not_shown_and_runs_the_one_that_was() {
    let mut room = opened_room();
    let script = match cfg!(windows) {
        true => "[Console]::Error.WriteLine('seen'); exit 2\n",
        false => "echo seen >&2\nexit 2\n",
    };
    let file = hook_in_agents(&mut room, script_name(), script);
    assert_eq!(
        code(trial(&room.session, &file, "not-the-print").map(|_| ())),
        "agentHookChangedSince"
    );
    let print = tisty_core::docs::print_of_text(&body_of(&room, &file));
    let ran = tried(trial(&room.session, &file, &print).unwrap()).unwrap();
    assert_eq!(ran.code, Some(2));
    assert_eq!(ran.said, "seen");
    assert!(!ran.timed_out);
}

#[test]
fn a_dry_run_of_a_registered_hook_uses_the_kept_script_not_the_edited_words() {
    let mut room = opened_room();
    let file = hook_in_agents(&mut room, script_name(), "exit 2\n");
    registering(&room, &file).unwrap();
    let edited = body_of(&room, &file).replace("exit 2", "exit 0");
    rewritten(&mut room, &file, &edited);
    let print = tisty_core::docs::print_of_text(&edited);
    let ran = tried(trial(&room.session, &file, &print).unwrap()).unwrap();
    assert_eq!(ran.code, Some(0));
}

#[test]
fn looking_at_a_document_that_is_not_a_hook_or_not_there_is_refused() {
    let mut room = opened_room();
    assert_eq!(code(looked(&room.session, "nothing")), "noSuchDoc");
    let rule = crate::answers::agents_sync::tests::style_rule(&mut room, "Short.\n");
    assert_eq!(code(looked(&room.session, &rule)), "agentHookUnread");
}
