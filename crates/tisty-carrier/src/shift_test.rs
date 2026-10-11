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
    std::fs::remove_file(at).unwrap();
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
