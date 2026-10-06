use super::*;
use crate::event::{DeviceId, Event, Op};
use sha2::{Digest, Sha256};

fn print_of(body: &[u8]) -> String {
    Sha256::digest(body)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn laid(root: &Path, slug: &str, body: &[u8]) -> String {
    let sha = print_of(body);
    let at = format!("attachments/{}/{slug}-{}.mp4", &sha[..2], &sha[2..10]);
    let file = root.join(&at);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, body).unwrap();
    at
}

fn kept(state: &mut State, at: &str, body: &[u8]) {
    state.apply(&Event::new(
        DeviceId("dev_a".into()),
        jiff::Timestamp::from_millisecond(1).unwrap(),
        Op::AttachKept {
            d: Held {
                at: at.into(),
                sha256: print_of(body),
                bytes: body.len() as u64,
            },
        },
    ));
}

#[test]
fn what_is_pointed_at_and_answered_for_by_no_history_is_listed_with_its_size() {
    let here = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let answered = laid(folder.path(), "answered", b"already kept");
    let only_there = laid(
        folder.path(),
        "video",
        b"a large video kept only in the folder",
    );
    let unnamed = laid(folder.path(), "nobody-points-here", b"loose");
    kept(&mut state, &answered, b"already kept");

    let named = vec![
        answered.clone(),
        only_there.clone(),
        only_there.clone(),
        "attachments/ab/gone-12345678.mp4".to_string(),
        "https://example.com/a.mp4".to_string(),
    ];
    let told = unvouched(&state, &named, &[here.path(), folder.path()]);

    assert_eq!(
        told,
        vec![Unvouched {
            at: only_there,
            bytes: 37,
        }],
        "the answered one, the missing one, the web link and {unnamed} are left out"
    );
}

#[test]
fn a_file_that_is_what_its_name_says_is_answered_for_with_its_whole_print() {
    let folder = tempfile::tempdir().unwrap();
    let body = b"the meeting recording";
    let at = laid(folder.path(), "quotations-walkthrough", body);

    let Vouch::Held { held, here } = vouch(&at, &[folder.path()]).unwrap() else {
        panic!("a file whose bytes match its name was not answered for");
    };
    assert!(
        here,
        "found in the first place given, the machine's own store"
    );

    assert_eq!(held.at, at);
    assert_eq!(held.sha256, print_of(body));
    assert_eq!(held.bytes, body.len() as u64);
}

#[test]
fn a_name_whose_print_was_written_again_and_again_is_still_read_by_its_last_stamp() {
    let folder = tempfile::tempdir().unwrap();
    let body = b"an attachment kept four times over";
    let sha = print_of(body);
    let stamp = &sha[2..10];
    let at = format!(
        "attachments/{}/final-zip-{stamp}-{stamp}-{stamp}-{stamp}.001",
        &sha[..2]
    );
    let file = folder.path().join(&at);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, body).unwrap();

    assert!(matches!(
        vouch(&at, &[folder.path()]).unwrap(),
        Vouch::Held { .. }
    ));
}

#[test]
fn a_file_that_is_not_what_its_name_says_is_never_written_down() {
    let folder = tempfile::tempdir().unwrap();
    let at = laid(
        folder.path(),
        "swapped",
        b"the bytes the name was made from",
    );
    std::fs::write(folder.path().join(&at), b"other bytes under the same name").unwrap();

    assert_eq!(vouch(&at, &[folder.path()]).unwrap(), Vouch::Unlike);
}

#[test]
fn a_file_no_longer_anywhere_or_outside_the_store_is_not_answered_for() {
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(folder.path().join("secret.txt"), b"x").unwrap();

    assert_eq!(
        vouch("attachments/ab/gone-12345678.mp4", &[folder.path()]).unwrap(),
        Vouch::Gone
    );
    assert!(!matches!(
        vouch("attachments/../secret.txt", &[folder.path()]).unwrap(),
        Vouch::Held { .. }
    ));
    assert!(
        unvouched(
            &State::default(),
            &["attachments/../secret.txt".to_string()],
            &[folder.path()]
        )
        .is_empty(),
        "a reference that climbs out of the store was looked at"
    );
}

#[test]
fn a_file_found_only_in_the_shared_folder_is_not_said_to_be_here() {
    let here = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let at = laid(folder.path(), "only-there", b"kept only in the folder");

    let Vouch::Held {
        here: found_here, ..
    } = vouch(&at, &[here.path(), folder.path()]).unwrap()
    else {
        panic!("the folder's copy was not answered for");
    };
    assert!(
        !found_here,
        "a machine that holds only the folder's copy would claim it, and another could let go of its own"
    );
}

#[test]
fn a_print_written_down_and_let_go_stays_known_with_nobody_claiming_the_copy() {
    let mut state = State::default();
    let body = b"a video that lives only in the folder";
    let at = format!(
        "attachments/{}/video-{}.mp4",
        &print_of(body)[..2],
        &print_of(body)[2..10]
    );
    kept(&mut state, &at, body);
    state.apply(&Event::new(
        DeviceId("dev_a".into()),
        jiff::Timestamp::from_millisecond(2).unwrap(),
        Op::AttachLetGo { d: at.clone() },
    ));

    assert_eq!(
        state.kept.get(&at).map(|one| one.0.clone()),
        Some(print_of(body)),
        "the print must stay known once the copy is let go"
    );
    assert!(
        !state.holders.contains_key(&at),
        "a machine that only saw the folder's copy still claims to hold it"
    );
}
