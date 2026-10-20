use super::*;
use tisty_core::event::{Body, TaskAdd};

const STAYS: &str = "revision-diseno-a42d4603.mp4";
const GOES: &str = "ttkjw8qofb-a42d4603-a42d4603.mp4";

fn with_twins() -> (tempfile::TempDir, Session, String, ulid::Ulid) {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::paths::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths.clone()).unwrap();
    let shed = paths.data().join("attachments");
    std::fs::create_dir_all(&shed).unwrap();
    std::fs::write(shed.join(STAYS), b"the same video").unwrap();
    std::fs::write(shed.join(GOES), b"the same video").unwrap();
    let device = tisty_core::DeviceId("dev_a".into());
    let doc = tisty_core::docs::create(&paths.docs(), &device, "# Demo").unwrap();
    tisty_core::docs::write(
        &paths.docs(),
        &doc.id,
        &format!("# Demo\n\n![video]({GOES})\n"),
    )
    .unwrap();
    let task = ulid::Ulid::generate();
    session
        .commit(Op::TaskAdd {
            id: task,
            d: TaskAdd::new("ver la demo".to_string(), "a0"),
        })
        .unwrap();
    session
        .commit(Op::TaskDescribe {
            id: task,
            d: Body {
                body: Some(format!("![video](attachments/b7/{GOES})")),
            },
        })
        .unwrap();
    (kept, session, doc.id, task)
}

#[test]
fn the_copy_with_a_readable_name_and_no_old_shelf_is_proposed_to_stay() {
    let (_kept, session, doc, task) = with_twins();

    let pairs = seen(&session, tisty_core::attach::twins(session.paths.data()));

    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].stays, format!("attachments/{STAYS}"));
    let goes = pairs[0]
        .copies
        .iter()
        .find(|one| one.at.ends_with(GOES))
        .unwrap();
    let named: Vec<&str> = goes.links.iter().map(|one| one.id.as_str()).collect();
    assert!(named.contains(&doc.as_str()), "{named:?}");
    assert!(named.contains(&task.to_string().as_str()), "{named:?}");
}

#[test]
fn joining_points_every_link_at_the_copy_that_stays_and_bins_the_other() {
    let (_kept, mut session, doc, task) = with_twins();

    let done = joined(
        &mut session,
        format!("attachments/{STAYS}"),
        vec![format!("attachments/{GOES}")],
    )
    .unwrap();

    assert_eq!(
        done.relinked, 2,
        "one link in the document and one in the task"
    );
    assert_eq!(done.freed, b"the same video".len() as u64);
    let body = tisty_core::docs::read(&session.paths.docs(), &doc).unwrap();
    assert!(body.contains(STAYS) && !body.contains(GOES), "{body}");
    let said = session.state.tasks[&task].description.clone().unwrap();
    assert_eq!(said, format!("![video](attachments/b7/{STAYS})"));
    assert!(!session.paths.data().join("attachments").join(GOES).exists());
    assert!(
        session
            .paths
            .data()
            .join("attachments")
            .join(STAYS)
            .exists()
    );
}

#[test]
fn copies_that_are_not_the_same_bytes_are_never_joined() {
    let (_kept, mut session, _doc, _task) = with_twins();
    std::fs::write(
        session.paths.data().join("attachments").join(GOES),
        b"another video!",
    )
    .unwrap();

    let refused = joined(
        &mut session,
        format!("attachments/{STAYS}"),
        vec![format!("attachments/{GOES}")],
    )
    .unwrap_err();

    assert_eq!(refused.code, "twinsGone");
}
