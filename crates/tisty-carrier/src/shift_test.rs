use super::*;
use crate::{Owed, Shift};

const PHOTO: &[u8] = b"los bytes de una foto";

fn step(desk: &Desk, from: Option<&dyn Carrier>, to: Option<&dyn Carrier>) -> Shift {
    let mut saying = |_| {};
    crate::shift(from, to, &desk.here, &[], &mut saying).unwrap()
}

impl Desk {
    fn attached(&self, body: &[u8]) -> tisty_core::attach::Kept {
        let kept = self.kept("foto", body);
        self.store().append(kept.told()).unwrap();
        self.wrote("con foto");
        kept
    }
}

fn let_go_here(desk: &Desk, reference: &str) {
    let at = tisty_core::attach::resolve(reference, &desk.here.data).unwrap();
    let _ = std::fs::remove_file(at);
}

fn origin(desk: &Desk, room: &std::path::Path) -> Folder {
    let origin = Folder::at(room.join("origin"));
    std::fs::create_dir_all(room.join("origin")).unwrap();
    let mut saying = |_| {};
    origin
        .carry(
            &desk.here,
            Round {
                way: Way::Both,
                alive: &[],
                holds: Holds::Everywhere,
                saying: &mut saying,
            },
        )
        .unwrap();
    origin
}

#[test]
fn leaving_a_place_brings_home_what_only_it_holds_before_it_is_ready() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = one.attached(PHOTO);
        one.round(Way::Both);
        two.round(Way::Pull);
        let_go_here(&two, &kept.at);
        assert_eq!(
            crate::homing(&*two.carrier, &two.here),
            Owed {
                files: 1,
                bytes: PHOTO.len() as u64
            },
            "{who}"
        );

        let done = step(&two, Some(&*two.carrier), None);

        assert_eq!(done, Shift::Ready, "{who}");
        assert_eq!(two.held(&kept.at).as_deref(), Some(PHOTO), "{who}");
    }
}

#[test]
fn a_new_place_is_ready_only_once_it_holds_all_the_old_one_did() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = one.attached(PHOTO);
        let old = origin(&one, setting.room.path());
        let_go_here(&one, &kept.at);

        let done = step(&one, Some(&old), Some(&*one.carrier));

        assert_eq!(done, Shift::Ready, "{who}");
        assert!(crate::filling(&*one.carrier, &one.here).nothing(), "{who}");
        two.round(Way::Pull);
        assert_eq!(two.titles(), ["con foto"], "{who}");
        let brought = two.carrier.brought(&two.here, &kept.at).unwrap();
        let crate::Brought::At(at) = brought else {
            panic!("{who}: the new place never got the photo: {brought:?}");
        };
        assert_eq!(std::fs::read(at).unwrap(), PHOTO, "{who}");
    }
}

#[test]
fn a_move_into_the_cloud_goes_by_stages_and_carries_on_where_it_stopped() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        let kept = one.attached(PHOTO);
        spent_today(&setting, "dev_a", fake.limits().units_a_day / 100 * 95);

        let stopped = step(&one, None, Some(&*one.carrier));

        let Shift::Filling { owed, deferred } = stopped else {
            panic!("{who}: {stopped:?}");
        };
        assert_eq!(owed.files, 1, "{who}");
        assert_eq!(
            deferred.map(|one| one.reason),
            Some(Reason::Budget),
            "{who}"
        );
        assert!(fake.about(&kept.at).unwrap().is_none(), "{who}");

        std::fs::remove_file(home_of(&setting, "dev_a").join("budget.json")).unwrap();
        let done = step(&one, None, Some(&*one.carrier));

        assert_eq!(done, Shift::Ready, "{who}");
        assert!(fake.about(&kept.at).unwrap().is_some(), "{who}");
    }
}

#[test]
fn an_attachment_no_place_ever_held_does_not_hold_the_move_back() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = one.attached(PHOTO);
        let_go_here(&one, &kept.at);
        one.round(Way::Both);
        two.round(Way::Pull);

        let done = step(&two, Some(&*two.carrier), None);

        assert_eq!(done, Shift::Ready, "{who}");
        assert!(crate::homing(&*two.carrier, &two.here).nothing(), "{who}");
    }
}

#[test]
fn a_new_place_whose_history_waits_for_tomorrow_is_not_ready() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        one.wrote("sin foto");
        spent_today(&setting, "dev_a", fake.limits().units_a_day);

        let stopped = step(&one, None, Some(&*one.carrier));

        let Shift::Filling { owed, deferred } = stopped else {
            panic!("{who}: {stopped:?}");
        };
        assert!(owed.nothing(), "{who}");
        assert_eq!(
            deferred.map(|one| one.reason),
            Some(Reason::Budget),
            "{who}"
        );
    }
}

#[test]
fn history_only_the_old_place_holds_comes_along_even_with_nothing_owed_home() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("solo en el lugar viejo");
        one.round(Way::Both);
        let new = Folder::at(setting.room.path().join("new"));
        std::fs::create_dir_all(setting.room.path().join("new")).unwrap();

        let done = step(&two, Some(&*two.carrier), Some(&new));

        assert_eq!(done, Shift::Ready, "{who}");
        assert_eq!(two.titles(), ["solo en el lugar viejo"], "{who}");
    }
}

#[test]
fn a_copy_of_the_wrong_size_in_a_folder_counts_as_not_there() {
    let room = tempfile::tempdir().unwrap();
    let setting = Setting {
        name: "folder".to_string(),
        place: Place::Folder(room.path().join("shared")),
        room,
    };
    std::fs::create_dir_all(setting.room.path().join("shared")).unwrap();
    let (one, two) = pair(&setting);
    let kept = one.attached(PHOTO);
    one.round(Way::Both);
    two.round(Way::Pull);
    let_go_here(&two, &kept.at);
    let there = tisty_core::attach::resolve(&kept.at, &setting.room.path().join("shared")).unwrap();
    std::fs::write(&there, b"corta").unwrap();

    assert!(crate::homing(&*two.carrier, &two.here).nothing());
    assert_eq!(
        crate::filling(&*one.carrier, &one.here),
        Owed {
            files: 1,
            bytes: PHOTO.len() as u64
        }
    );
}

#[test]
fn leaving_a_cloud_that_cannot_be_asked_now_is_not_ready_even_with_nothing_owed() {
    for setting in clouds() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("solo en la nube");
        one.round(Way::Both);
        let _round = crate::cloud::lock::Round::take(&home_of(&setting, "dev_b"))
            .unwrap()
            .unwrap();

        let stopped = step(&two, Some(&*two.carrier), None);

        let Shift::Homing { owed, deferred } = stopped else {
            panic!("{who}: {stopped:?}");
        };
        assert!(owed.nothing(), "{who}");
        assert_eq!(deferred.map(|one| one.reason), Some(Reason::Busy), "{who}");
    }
}

#[test]
fn leaving_a_cloud_sends_nothing_to_it_so_a_full_one_never_holds_the_person_there() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        one.round(Way::Both);
        one.wrote("aun sin subir");
        remote.fail_after(1, Hitch::Full);
        assert!(one.carry(Way::Both, Holds::Everywhere).is_err(), "{who}");
        remote.forget_counts();

        let done = step(&one, Some(&*one.carrier), None);

        assert_eq!(done, Shift::Ready, "{who}");
        assert_eq!(remote.counts().puts, 0, "{who}: leaving it sent to it");
    }
}

#[test]
fn an_attachment_swapped_up_there_is_never_brought_home_and_does_not_hold_the_move() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, two) = pair(&setting);
        let kept = one.attached(PHOTO);
        one.round(Way::Both);
        let swapped = setting.room.path().join("swapped");
        std::fs::write(&swapped, vec![b'x'; PHOTO.len()]).unwrap();
        remote.put(&kept.at, &swapped, Expect::Any).unwrap();
        two.round(Way::Pull);

        let done = step(&two, Some(&*two.carrier), None);

        assert_eq!(done, Shift::Ready, "{who}");
        assert!(
            two.held(&kept.at).is_none(),
            "{who}: the swapped bytes came home"
        );
        remote.forget_counts();
        step(&two, Some(&*two.carrier), None);
        assert_eq!(
            remote.counts().fetches,
            0,
            "{who}: fetched again what was refused"
        );
    }
}

fn spoiled_by(setting: &Setting, device: &str, reference: &str) -> bool {
    crate::cloud::index::Index::load(&home_of(setting, device)).spoiled(reference)
}

#[test]
fn an_attachment_retired_and_written_down_again_but_named_by_nothing_is_owed_nowhere() {
    for setting in settings() {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        let kept = one.attached(PHOTO);
        one.round(Way::Both);
        one.store()
            .append(Op::AttachRetire { d: kept.at.clone() })
            .unwrap();
        one.store().append(kept.told()).unwrap();
        let_go_here(&one, &kept.at);

        assert!(crate::homing(&*one.carrier, &one.here).nothing(), "{who}");
        assert!(crate::filling(&*one.carrier, &one.here).nothing(), "{who}");
    }
}

#[test]
fn an_attachment_that_could_not_be_put_in_place_is_tried_again_not_called_spoiled() {
    for setting in clouds() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = one.attached(PHOTO);
        one.round(Way::Both);
        two.round(Way::Pull);
        let target = tisty_core::attach::resolve(&kept.at, &two.here.data).unwrap();
        std::fs::create_dir_all(target.join("en el camino")).unwrap();

        let stopped = step(&two, Some(&*two.carrier), None);

        assert!(
            matches!(stopped, Shift::Homing { .. }),
            "{who}: {stopped:?}"
        );
        assert!(!spoiled_by(&setting, "dev_b", &kept.at), "{who}");
        std::fs::remove_dir_all(&target).unwrap();
        assert_eq!(step(&two, Some(&*two.carrier), None), Shift::Ready, "{who}");
        assert_eq!(two.held(&kept.at).as_deref(), Some(PHOTO), "{who}");
    }
}

#[test]
fn a_spoiled_copy_is_neither_fetched_when_opened_nor_left_up_there_by_a_move_into_that_cloud() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, two) = pair(&setting);
        let kept = one.attached(PHOTO);
        one.round(Way::Both);
        let swapped = setting.room.path().join("swapped");
        std::fs::write(&swapped, vec![b'x'; PHOTO.len()]).unwrap();
        remote.put(&kept.at, &swapped, Expect::Any).unwrap();
        two.round(Way::Pull);
        step(&two, Some(&*two.carrier), None);
        assert!(spoiled_by(&setting, "dev_b", &kept.at), "{who}");

        remote.forget_counts();
        assert!(two.carrier.brought(&two.here, &kept.at).is_err(), "{who}");
        assert_eq!(
            remote.counts().fetches,
            0,
            "{who}: fetched what was refused"
        );

        one.round(Way::Pull);
        let mut index = crate::cloud::index::Index::load(&home_of(&setting, "dev_a"));
        let revision = index.shelf[&kept.at].revision.clone();
        index.spoiled.insert(kept.at.clone(), revision);
        index.save(&home_of(&setting, "dev_a")).unwrap();
        let done = step(&one, None, Some(&*one.carrier));

        assert_eq!(done, Shift::Ready, "{who}");
        let mut there = Vec::new();
        remote.fetch(&kept.at, 0, &mut there).unwrap();
        assert_eq!(there, PHOTO, "{who}: the refused copy stayed up there");
    }
}

#[test]
fn history_the_pull_had_to_leave_for_later_leaves_the_move_homing() {
    for (setting, fake) in each_kind(|fake| fake) {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        one.wrote("lo primero de uno");
        one.round(Way::Both);
        two.round(Way::Both);
        one.wrote("lo ultimo de uno");
        one.round(Way::Both);
        fake.truncate_next(1);

        let stopped = step(&two, Some(&*two.carrier), None);

        let Shift::Homing { deferred, .. } = stopped else {
            panic!("{who}: {stopped:?}");
        };
        assert_eq!(
            deferred.map(|one| one.reason),
            Some(Reason::Changed),
            "{who}"
        );
        assert_eq!(step(&two, Some(&*two.carrier), None), Shift::Ready, "{who}");
        assert_eq!(
            two.titles(),
            ["lo primero de uno", "lo ultimo de uno"],
            "{who}"
        );
    }
}

#[test]
fn an_attachment_no_history_answers_for_is_brought_and_handed_back_to_be_written_down() {
    for setting in clouds() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = one.kept("foto", PHOTO);
        one.wrote("con foto");
        one.round(Way::Both);
        two.round(Way::Pull);

        let brought = two.carrier.brought(&two.here, &kept.at).unwrap();

        let crate::Brought::Unvouched { at, sha256, bytes } = brought else {
            panic!("{who}: {brought:?}");
        };
        assert_eq!(std::fs::read(at).unwrap(), PHOTO, "{who}");
        assert_eq!((sha256, bytes), (kept.sha256.clone(), kept.bytes), "{who}");
    }
}

#[test]
fn a_round_that_stopped_or_left_things_to_decide_never_counts_as_brought_over() {
    let stopped = crate::Moved {
        refused: Some(crate::Trouble::Broke("lleno".into())),
        ..Default::default()
    };
    assert!(crate::shift::settled(stopped).is_err());

    let astray = crate::Moved {
        astray: vec!["docs/notas.md".into()],
        ..Default::default()
    };
    assert_eq!(crate::shift::settled(astray).unwrap(), None);

    let unprojected = crate::Moved {
        unprojected: true,
        ..Default::default()
    };
    assert_eq!(crate::shift::settled(unprojected).unwrap(), None);

    assert!(
        crate::shift::settled(crate::Moved::default())
            .unwrap()
            .is_some()
    );
}

#[test]
fn what_comes_home_is_counted_up_to_a_size_when_asked() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = one.attached(PHOTO);
        one.round(Way::Both);
        two.round(Way::Pull);
        let_go_here(&two, &kept.at);

        let all = crate::homing(&*two.carrier, &two.here);
        let small = crate::homing_up_to(&*two.carrier, &two.here, PHOTO.len() as u64 - 1);

        assert_eq!(all.files, 1, "{who}");
        assert!(small.nothing(), "{who}");
    }
}
