use std::time::{Duration, SystemTime};

use super::*;
use crate::LetGo;

const PHOTO: &[u8] = b"los bytes de una foto";

fn later() -> SystemTime {
    SystemTime::now() + Duration::from_secs(3600)
}

fn sent(desk: &Desk) -> tisty_core::attach::Kept {
    let kept = desk.kept("foto", PHOTO);
    desk.store()
        .append(kept.told(tisty_core::attach::Via::Window))
        .unwrap();
    desk.wrote("con foto");
    desk.round(Way::Both);
    kept
}

fn freed(desk: &Desk, unused_since: SystemTime) -> Result<LetGo, Trouble> {
    desk.carrier
        .set_free(&desk.here, unused_since, &mut |_| true)
}

#[test]
fn space_is_freed_only_by_a_cloud_and_only_of_what_lies_up_there_unused() {
    for setting in settings() {
        let who = &setting.name;
        let (one, two) = pair(&setting);
        let kept = sent(&one);

        let could = one.carrier.could_free(&one.here, later());
        let done = freed(&one, later()).unwrap();

        if let Place::Folder(_) = setting.place {
            assert_eq!(could, LetGo::default(), "{who}");
            assert_eq!(done, LetGo::default(), "{who}");
            assert!(one.held(&kept.at).is_some(), "{who}");
            continue;
        }
        assert_eq!(could.let_go, std::slice::from_ref(&kept.at), "{who}");
        assert_eq!(done.let_go, std::slice::from_ref(&kept.at), "{who}");
        assert_eq!(done.freed, PHOTO.len() as u64, "{who}");
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
            &one.here.data.join(tisty_core::lately::USED),
            &kept.at,
            seconds,
        );

        let could = one.carrier.could_free(&one.here, later());
        let done = freed(&one, later()).unwrap();

        assert!(could.let_go.is_empty(), "{who}");
        assert!(done.let_go.is_empty(), "{who}");
        assert_eq!(one.held(&kept.at).as_deref(), Some(PHOTO), "{who}");
    }
}

#[test]
fn an_attachment_attached_lately_stays_however_old_its_file_is() {
    for setting in clouds() {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        let kept = sent(&one);
        let at = tisty_core::attach::resolve(&kept.at, &one.here.data).unwrap();
        let long_ago = SystemTime::now() - Duration::from_secs(90 * 24 * 3600);
        std::fs::File::options()
            .write(true)
            .open(&at)
            .unwrap()
            .set_modified(long_ago)
            .unwrap();

        let done = freed(&one, SystemTime::now() - Duration::from_secs(3600)).unwrap();

        assert!(done.let_go.is_empty(), "{who}");
        assert_eq!(one.held(&kept.at).as_deref(), Some(PHOTO), "{who}");
    }
}

#[test]
fn a_date_no_clock_could_hold_never_stops_the_freeing() {
    for setting in clouds() {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        let kept = sent(&one);
        let used = one.here.data.join(tisty_core::lately::USED);
        std::fs::create_dir_all(&one.here.data).unwrap();
        std::fs::write(&used, format!("{{\"{}\": {}}}", kept.at, u64::MAX)).unwrap();

        let done = freed(&one, later()).unwrap();

        assert!(done.let_go.is_empty(), "{who}");
        assert_eq!(one.held(&kept.at).as_deref(), Some(PHOTO), "{who}");
    }
}

#[test]
fn a_copy_up_there_that_changed_since_the_last_round_keeps_the_one_here() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        let kept = sent(&one);
        let swapped = setting.room.path().join("swapped");
        std::fs::write(&swapped, vec![b'x'; PHOTO.len()]).unwrap();
        remote.put(&kept.at, &swapped, Expect::Any).unwrap();

        let done = freed(&one, later()).unwrap();

        assert!(done.let_go.is_empty(), "{who}");
        assert_eq!(done.kept, std::slice::from_ref(&kept.at), "{who}");
        assert_eq!(one.held(&kept.at).as_deref(), Some(PHOTO), "{who}");
    }
}

#[test]
fn nothing_is_freed_while_the_cloud_cannot_be_asked_first() {
    for setting in clouds() {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        let kept = sent(&one);
        let _round = crate::cloud::lock::Round::take(&home_of(&setting, "dev_a"))
            .unwrap()
            .unwrap();

        let done = freed(&one, later());

        assert!(done.is_err(), "{who}: {done:?}");
        assert_eq!(one.held(&kept.at).as_deref(), Some(PHOTO), "{who}");
    }
}

#[test]
fn the_same_bytes_attached_again_after_freeing_come_back_under_their_name() {
    for setting in clouds() {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        let kept = sent(&one);
        freed(&one, later()).unwrap();

        let again = one.kept("otra", PHOTO);

        assert_eq!(again.at, kept.at, "{who}");
        assert_eq!(one.held(&kept.at).as_deref(), Some(PHOTO), "{who}");
    }
}

#[test]
fn a_provider_that_gives_no_hash_never_has_a_copy_here_let_go() {
    for (setting, _) in each_kind(|fake| fake.without_hashes()) {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        let kept = sent(&one);

        let could = one.carrier.could_free(&one.here, later());
        let done = freed(&one, later()).unwrap();

        assert!(could.let_go.is_empty(), "{who}");
        assert!(done.let_go.is_empty(), "{who}");
        assert_eq!(one.held(&kept.at).as_deref(), Some(PHOTO), "{who}");
    }
}

#[test]
fn an_attachment_shown_under_a_moment_of_its_own_still_counts_as_used() {
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
            &one.here.data.join(tisty_core::lately::USED),
            &format!("{}#t=5", kept.at),
            seconds,
        );

        let done = freed(&one, later()).unwrap();

        assert!(done.let_go.is_empty(), "{who}");
    }
}

#[test]
fn only_a_cloud_says_it_frees_space_so_the_window_offers_it_nowhere_else() {
    for setting in settings() {
        let (one, _) = pair(&setting);

        let frees = one.carrier.frees_space();

        assert_eq!(
            frees,
            matches!(setting.place, Place::Cloud(_)),
            "{}",
            setting.name
        );
    }
}

#[test]
fn the_estimate_reads_no_bytes_and_only_letting_go_checks_them() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        let kept = sent(&one);
        let swapped = setting.room.path().join("swapped");
        std::fs::write(&swapped, vec![b'x'; PHOTO.len()]).unwrap();
        remote.put(&kept.at, &swapped, Expect::Any).unwrap();
        one.round(Way::Pull);

        let could = one.carrier.could_free(&one.here, later());
        let done = freed(&one, later()).unwrap();

        assert_eq!(could.let_go, std::slice::from_ref(&kept.at), "{who}");
        assert!(
            done.let_go.is_empty(),
            "{who}: the estimate was taken as a check"
        );
        assert_eq!(one.held(&kept.at).as_deref(), Some(PHOTO), "{who}");
    }
}

#[test]
fn an_attachment_opened_while_space_is_freed_stays() {
    for setting in clouds() {
        let who = &setting.name;
        let (one, _) = pair(&setting);
        let first = one.kept("foto", PHOTO);
        let second = one.kept("otra", b"los bytes de otra foto");
        one.store()
            .append(first.told(tisty_core::attach::Via::Window))
            .unwrap();
        one.store()
            .append(second.told(tisty_core::attach::Via::Window))
            .unwrap();
        one.wrote("con fotos");
        one.round(Way::Both);
        let used = one.here.data.join(tisty_core::lately::USED);
        let seconds = later()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 60;

        let done = one
            .carrier
            .set_free(&one.here, later(), &mut |far| {
                for reference in [&first.at, &second.at] {
                    if !far.let_go.contains(reference) {
                        tisty_core::lately::used_on(&used, reference, seconds);
                    }
                }
                true
            })
            .unwrap();

        assert_eq!(done.let_go.len(), 1, "{who}: {done:?}");
        let stayed = [&first, &second]
            .into_iter()
            .find(|one| !done.let_go.contains(&one.at))
            .unwrap();
        assert!(
            one.held(&stayed.at).is_some(),
            "{who}: opened, and still let go of"
        );
    }
}

#[test]
fn an_attachment_freed_here_whose_copy_left_the_cloud_is_told_gone_not_later() {
    for setting in clouds() {
        let who = &setting.name;
        let remote = counting(&setting);
        let (one, _) = pair(&setting);
        let kept = sent(&one);
        freed(&one, later()).unwrap();
        remote.delete(&kept.at, None).unwrap();
        one.round(Way::Pull);

        let asked = one.carrier.brought(&one.here, &kept.at).unwrap();

        assert_eq!(asked, crate::Brought::Gone, "{who}");
        let back = setting.room.path().join("back");
        std::fs::write(&back, PHOTO).unwrap();
        remote.put(&kept.at, &back, Expect::Any).unwrap();
        assert_eq!(
            one.carrier.brought(&one.here, &kept.at).unwrap(),
            crate::Brought::At(tisty_core::attach::resolve(&kept.at, &one.here.data).unwrap()),
            "{who}: put back up there and still told gone"
        );
    }
}
