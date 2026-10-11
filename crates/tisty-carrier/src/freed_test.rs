use std::time::{Duration, SystemTime};

use super::*;

const PHOTO: &[u8] = b"los bytes de una foto";

fn later() -> SystemTime {
    SystemTime::now() + Duration::from_secs(3600)
}

fn sent(desk: &Desk) -> tisty_core::attach::Kept {
    let kept = desk.kept("foto", PHOTO);
    desk.store().append(kept.told()).unwrap();
    desk.wrote("con foto");
    desk.round(Way::Both);
    kept
}

#[test]
fn space_is_freed_only_by_a_cloud_and_only_of_what_lies_up_there_unused() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = sent(&one);

        let freed = one.carrier.set_free(&one.here, later());

        if let Place::Folder(_) = setting.place {
            assert_eq!(freed, crate::Freed::default(), "{who}");
            assert!(one.held(&kept.at).is_some(), "{who}");
            continue;
        }
        assert_eq!(freed.let_go, std::slice::from_ref(&kept.at), "{who}");
        assert_eq!(freed.bytes, PHOTO.len() as u64, "{who}");
        assert_eq!(one.held(&kept.at), None, "{who}");

        one.round(Way::Both);
        assert_eq!(one.held(&kept.at), None, "{who}: a round brought it back");
        one.carrier.brought(&one.here, &kept.at).unwrap();
        assert_eq!(one.held(&kept.at).as_deref(), Some(PHOTO), "{who}");
        two.round(Way::Pull);
        assert_eq!(
            two.carrier.brought(&two.here, &kept.at).unwrap(),
            crate::Brought::At(tisty_core::attach::resolve(&kept.at, &two.here.data).unwrap()),
            "{who}: freeing it here took it from everyone"
        );
    }
}

#[test]
fn an_attachment_opened_lately_stays() {
    for setting in clouds() {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        let kept = sent(&one);
        let seconds = later()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 60;
        tisty_core::lately::used_on(
            &one.here.aside.join(tisty_core::lately::USED),
            &kept.at,
            seconds,
        );

        let could = one.carrier.could_free(&one.here, later());
        let freed = one.carrier.set_free(&one.here, later());

        assert_eq!(could, crate::Freed::default(), "{who}");
        assert_eq!(freed, crate::Freed::default(), "{who}");
        assert_eq!(one.held(&kept.at).as_deref(), Some(PHOTO), "{who}");
    }
}

#[test]
fn a_copy_up_there_that_is_not_the_same_keeps_the_one_here() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        let kept = sent(&one);
        let swapped = setting.room.path().join("swapped");
        std::fs::write(&swapped, vec![b'x'; PHOTO.len()]).unwrap();
        remote.put(&kept.at, &swapped, Expect::Any).unwrap();
        one.round(Way::Pull);

        let freed = one.carrier.set_free(&one.here, later());

        assert!(freed.let_go.is_empty(), "{who}");
        assert_eq!(one.held(&kept.at).as_deref(), Some(PHOTO), "{who}");
    }
}
