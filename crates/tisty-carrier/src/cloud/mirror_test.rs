use super::*;
use crate::cloud::budget::{Class, Metered};
use crate::fakes::Fake;

fn refused_up_there(tree: &Path, fake: &Fake, docs: usize) {
    let local = tree.with_extension("theirs");
    std::fs::write(&local, b"lo de otra maquina").unwrap();
    std::fs::create_dir_all(tree.join(PAPERS)).unwrap();
    for at in 0..docs {
        let name = format!("{PAPERS}/dev_a-{at:04}.md");
        fake.put(&name, &local, Expect::Absent).unwrap();
        std::fs::write(tree.join(&name), b"lo de esta maquina").unwrap();
    }
    std::fs::remove_file(local).unwrap();
}

#[test]
fn a_push_whose_every_file_is_refused_and_written_down_is_never_taken_for_a_loop() {
    let room = tempfile::tempdir().unwrap();
    let tree = room.path().join("tree");
    let fake = Fake::drive();
    refused_up_there(&tree, &fake, 150);
    let meter = Metered::over(&fake, room.path(), Class::Attachments);
    let mut index = Index::default();

    let (pushed, hitch) = push(&meter, &tree, &mut index, &mut Pace::new(None, &mut |_| {}));

    assert_eq!(hitch, None, "a push of refused files was stopped as a loop");
    assert_eq!(pushed.left, 150);
    assert_eq!(
        index.owed.len(),
        150,
        "what refused the writes was not written down"
    );
}

#[test]
fn the_same_refusal_asked_again_and_again_still_stops_as_a_loop() {
    let room = tempfile::tempdir().unwrap();
    let tree = room.path().join("tree");
    let fake = Fake::drive();
    refused_up_there(&tree, &fake, 1);
    let meter = Metered::over(&fake, room.path(), Class::Attachments);
    let mut index = Index::default();

    let mut pushes = 0;
    let stopped = loop {
        let (_, hitch) = push(&meter, &tree, &mut index, &mut Pace::new(None, &mut |_| {}));
        if let Some(hitch) = hitch {
            break hitch;
        }
        pushes += 1;
        assert!(
            pushes < 1_000,
            "asking the same refused file forever was never stopped"
        );
    };

    assert!(matches!(stopped, Hitch::Troubled(_)), "{stopped:?}");
}

fn echoed_while_owed(fake: Fake) {
    let room = tempfile::tempdir().unwrap();
    let (tree, data) = (room.path().join("tree"), room.path().join("data"));
    let id = "dev_a-0001";
    let name = format!("{PAPERS}/{id}.md");
    let mut index = Index::default();
    pull(&fake, &tree, &mut index, Some(&data), false, &mut |_| {}).unwrap();
    std::fs::create_dir_all(tree.join(PAPERS)).unwrap();
    std::fs::write(tree.join(&name), b"lo primero\n").unwrap();
    let (pushed, _) = push(&fake, &tree, &mut index, &mut Pace::new(None, &mut |_| {}));
    assert_eq!(pushed.put, 1);

    let echo = room.path().join("echo");
    std::fs::write(&echo, b"lo primero\n").unwrap();
    fake.put(&name, &echo, Expect::Any).unwrap();
    let ours = "lo primero\ny lo que vino despues\n";
    std::fs::write(tree.join(&name), ours).unwrap();
    let print = tisty_core::attach::printed(ours.as_bytes());
    std::fs::create_dir_all(&data).unwrap();
    let mut said = tisty_core::docs::Carried::read(&data);
    said.keep(id, &print);
    said.save(&data).unwrap();
    tisty_core::docs::keep_carried(&data, id, ours).unwrap();

    pull(&fake, &tree, &mut index, Some(&data), false, &mut |_| {}).unwrap();

    assert_eq!(
        tisty_core::docs::Carried::read(&data).of(id),
        Some(print.as_str()),
        "our own write coming back took the document's base away"
    );
    assert_eq!(
        tisty_core::docs::read_carried(&data, id).as_deref(),
        Some(ours)
    );
    assert_eq!(std::fs::read_to_string(tree.join(&name)).unwrap(), ours);
    let (pushed, hitch) = push(&fake, &tree, &mut index, &mut Pace::new(None, &mut |_| {}));
    assert_eq!(
        (pushed.put, hitch),
        (1, None),
        "the change owed did not go up"
    );
    let mut up_there = Vec::new();
    fake.fetch(&name, 0, &mut up_there).unwrap();
    assert_eq!(up_there, ours.as_bytes());
}

#[test]
fn our_own_write_coming_back_leaves_a_change_still_owed_with_its_base() {
    echoed_while_owed(Fake::drive());
}

#[test]
fn our_own_write_coming_back_without_a_hash_leaves_a_change_still_owed_with_its_base() {
    echoed_while_owed(Fake::drive().without_hashes());
}
