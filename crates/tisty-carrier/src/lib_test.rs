use super::*;
use tisty_core::Op;
use tisty_core::event::{DeviceId, TaskAdd};

struct Desk {
    _room: tempfile::TempDir,
    here: Here,
    shared: PathBuf,
}

fn desk() -> Desk {
    let room = tempfile::tempdir().unwrap();
    let here = Here {
        data: room.path().join("data"),
        aside: room.path().join("cache"),
        device: "dev_a".to_string(),
    };
    let mut store =
        tisty_core::Store::open(here.data.join(STORE), DeviceId("dev_a".into())).unwrap();
    store
        .append(Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: TaskAdd::new("lo de dev_a", "a0"),
        })
        .unwrap();
    let shared = room.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    Desk {
        _room: room,
        here,
        shared,
    }
}

fn round<'a>(way: Way, saying: &'a mut dyn FnMut(Reached)) -> Round<'a> {
    Round {
        way,
        alive: &[],
        holds: Holds::Everywhere,
        saying,
    }
}

#[test]
fn nothing_chosen_and_staying_here_carry_nothing() {
    for sync in [None, Some(Sync::Local)] {
        let keeping = chosen(sync.as_ref());

        assert!(keeping.carrier.is_none());
        assert_eq!(keeping.chosen, Chosen::Alone);
    }
}

#[test]
fn a_folder_is_carried_by_a_folder_at_that_place() {
    let at = PathBuf::from("compartida");

    let keeping = chosen(Some(&Sync::Folder(at.clone())));

    assert_eq!(keeping.chosen, Chosen::Folder);
    assert_eq!(keeping.carrier.unwrap().place(), Some(at.as_path()));
}

#[test]
fn a_way_a_later_build_knows_carries_nothing_and_says_so() {
    let later = Sync::Unknown(toml::Value::String("nube".into()));

    let keeping = chosen(Some(&later));

    assert!(keeping.carrier.is_none());
    assert_eq!(keeping.chosen, Chosen::Later);
}

#[test]
fn a_quiet_round_through_the_carrier_opens_what_the_round_opens_today() {
    let desk = desk();
    let folder = Folder::at(desk.shared.clone());
    folder
        .carry(&desk.here, round(Way::Push, &mut |_| {}))
        .unwrap();
    folder
        .carry(&desk.here, round(Way::Both, &mut |_| {}))
        .unwrap();

    let _ = tisty_core::counting::from_now();
    let today = tisty_sync::carry_telling(
        &desk.here.data,
        Some(&desk.here.aside),
        &desk.here.device,
        &desk.shared,
        Way::Both,
        &[],
        Holds::Everywhere,
        &mut |_| {},
    )
    .unwrap();
    let opened_today = tisty_core::counting::from_now();
    let through = folder
        .carry(&desk.here, round(Way::Both, &mut |_| {}))
        .unwrap();
    let opened_through = tisty_core::counting::from_now();

    assert_eq!(through, today);
    assert_eq!(
        opened_through, opened_today,
        "the carrier read more than the round it hands the work to"
    );
}

#[test]
fn a_folder_answers_from_its_own_place() {
    let desk = desk();
    let folder = Folder::at(desk.shared.clone());
    assert!(folder.reachable());
    assert!(!Folder::at(desk.shared.join("ninguna")).reachable());

    folder
        .carry(&desk.here, round(Way::Push, &mut |_| {}))
        .unwrap();

    assert!(folder.been_here(&desk.here));
    assert_eq!(folder.theirs(), tisty_sync::theirs(&desk.shared));
    assert_eq!(folder.kin(&desk.here), Kin::SameLineage);
    assert_eq!(folder.unclaimed(), Holding::Whole);
    assert_eq!(folder.paper("dev_a-0001"), Paper::default());
}
