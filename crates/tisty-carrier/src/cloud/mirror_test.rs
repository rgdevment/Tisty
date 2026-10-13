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
