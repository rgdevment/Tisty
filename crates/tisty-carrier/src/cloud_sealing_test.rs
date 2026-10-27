use super::*;
use crate::{Stepped, Turning};
use tisty_share_proto::cloud::CloudKey;
use tisty_share_proto::seed::Seed;
use tisty_sync::Turn;

const CLOUD: &str = "store_one";

fn key() -> CloudKey {
    CloudKey::of(&Seed::from_bytes(&[7; 32]))
}

fn drive() -> (Setting, Arc<Fake>) {
    each_kind(|fake| fake).remove(0)
}

fn written(setting: &Setting) -> Desk {
    let one = desk(setting, "dev_a");
    one.wrote("antes de cifrar");
    one.filed(DOC, "# Plan\n\nantes de cifrar\n");
    one.round(Way::Both);
    one
}

fn turned(fake: &Arc<Fake>, setting: &Setting) -> Result<Turning, Hitch> {
    let plain: Arc<dyn Remote> = fake.clone();
    Turning::begin(
        plain,
        &home_of(setting, "dev_a"),
        key(),
        CLOUD,
        "dev_a",
        Turn::On,
    )
}

fn to_the_end(turning: &mut Turning) {
    for _ in 0..200 {
        if matches!(turning.step().unwrap(), Stepped::Done | Stepped::GaveUp) {
            return;
        }
    }
    panic!("the turning never ended");
}

fn word(fake: &Fake) -> String {
    let mut said = Vec::new();
    fake.fetch(tisty_sync::NAMED, 0, &mut said).unwrap();
    String::from_utf8(said).unwrap()
}

fn reading_sealed(setting: &Setting, fake: &Arc<Fake>, device: &str, from: &Desk) -> Desk {
    let base = desk(setting, device);
    let sealed: Arc<dyn Remote> = Arc::new(crate::Sealed::new(fake.clone(), key(), CLOUD));
    let reader = Desk {
        here: base.here,
        carrier: Arc::new(Cloud::over(sealed, home_of(setting, device))),
    };
    reader.vouches_for(from);
    from.vouches_for(&reader);
    reader
}

#[test]
fn a_cloud_turned_on_holds_everything_sealed_and_says_so() {
    let (setting, fake) = drive();
    let one = written(&setting);

    let mut turning = turned(&fake, &setting).unwrap();
    to_the_end(&mut turning);

    assert_eq!(
        tisty_sync::sealing_in(&word(&fake)),
        tisty_sync::Sealing::Sealed(CLOUD.into())
    );
    assert!(
        !fake.list("store").unwrap().is_empty(),
        "the plain copy is left until it is wiped"
    );
    let two = reading_sealed(&setting, &fake, "dev_b", &one);
    two.round(Way::Both);
    assert_eq!(two.titles(), vec!["antes de cifrar".to_string()]);
    assert_eq!(two.doc(DOC).as_deref(), Some("# Plan\n\nantes de cifrar\n"));
    let plain = desk(&setting, "dev_c");
    let stopped = plain.carry(Way::Both, Holds::Everywhere).err();
    assert!(matches!(stopped, Some(Trouble::Sealed(_))), "{stopped:?}");
}

#[test]
fn a_turning_given_up_before_the_switch_leaves_the_cloud_as_it_was() {
    let (setting, fake) = drive();
    let one = written(&setting);
    let before = word(&fake);
    let mut turning = turned(&fake, &setting).unwrap();
    turning.step().unwrap();

    turning.abort(|| Ok(())).unwrap();

    assert_eq!(
        tisty_sync::sealing_in(&word(&fake)),
        tisty_sync::sealing_in(&before)
    );
    assert!(!word(&fake).contains("sealing"));
    assert!(fake.list("sealed").unwrap().is_empty());
    one.wrote("después");
    assert!(one.carry(Way::Both, Holds::Everywhere).is_ok());
}

#[test]
fn a_second_machine_cannot_claim_a_cloud_another_is_turning() {
    let (setting, fake) = drive();
    written(&setting);
    let _first = turned(&fake, &setting).unwrap();
    let plain: Arc<dyn Remote> = fake.clone();

    let second = Turning::begin(
        plain,
        &home_of(&setting, "dev_b"),
        key(),
        CLOUD,
        "dev_b",
        Turn::On,
    );

    assert!(matches!(second, Err(Hitch::Barred(Trouble::Sealing(_)))));
}

#[test]
fn a_place_that_was_left_is_never_turned() {
    let (setting, fake) = drive();
    written(&setting);
    let dir = tempfile::tempdir().unwrap();
    let moved = dir.path().join("moved");
    std::fs::write(&moved, "{}").unwrap();
    fake.put(tisty_core::moved::NAMED, &moved, Expect::Any)
        .unwrap();

    assert!(matches!(
        turned(&fake, &setting),
        Err(Hitch::Barred(Trouble::Refused(_)))
    ));
}

#[test]
fn a_cut_at_any_call_is_taken_up_again_and_loses_nothing() {
    let calls = {
        let (setting, fake) = drive();
        written(&setting);
        let before = fake.counts().requests;
        to_the_end(&mut turned(&fake, &setting).unwrap());
        fake.counts().requests - before
    };
    assert!(calls > 0);
    for cut in 0..calls as usize {
        let (setting, fake) = drive();
        let one = written(&setting);
        fake.fail_after(cut, Hitch::Unreachable(None, "cut".into()));
        let plain: Arc<dyn Remote> = fake.clone();
        let home = home_of(&setting, "dev_a");
        let mut turning = turned(&fake, &setting).ok();
        let mut done = false;
        for _ in 0..200 {
            let resumed = || Turning::resume(plain.clone(), &home, key());
            let Some(mut now) = turning
                .take()
                .or_else(resumed)
                .or_else(|| turned(&fake, &setting).ok())
            else {
                continue;
            };
            match now.step() {
                Ok(Stepped::Done | Stepped::GaveUp) => {
                    done = true;
                    break;
                }
                Ok(_) => turning = Some(now),
                Err(_) => {}
            }
        }
        assert!(done, "cut {cut}: never ended");
        let two = reading_sealed(&setting, &fake, "dev_b", &one);
        two.round(Way::Both);
        assert_eq!(
            two.titles(),
            vec!["antes de cifrar".to_string()],
            "cut {cut}"
        );
    }
}

fn turning_off(fake: &Arc<Fake>, setting: &Setting) -> Result<Turning, Hitch> {
    let plain: Arc<dyn Remote> = fake.clone();
    Turning::begin(
        plain,
        &home_of(setting, "dev_a"),
        key(),
        CLOUD,
        "dev_a",
        Turn::Off,
    )
}

fn sealed_on(setting: &Setting, fake: &Arc<Fake>) -> Desk {
    let one = written(setting);
    to_the_end(&mut turned(fake, setting).unwrap());
    one
}

#[test]
fn a_cloud_turned_off_is_plain_again_with_what_was_written_while_it_was_sealed() {
    let (setting, fake) = drive();
    let one = sealed_on(&setting, &fake);
    let two = reading_sealed(&setting, &fake, "dev_b", &one);
    two.round(Way::Both);
    two.wrote("mientras estaba cifrada");
    two.round(Way::Both);

    to_the_end(&mut turning_off(&fake, &setting).unwrap());

    assert_eq!(
        tisty_sync::sealing_in(&word(&fake)),
        tisty_sync::Sealing::Plain
    );
    assert_eq!(
        crate::Sealed::new(fake.clone(), key(), CLOUD)
            .marked()
            .unwrap(),
        None
    );
    let plain = desk(&setting, "dev_c");
    plain.vouches_for(&one);
    plain.vouches_for(&two);
    plain.round(Way::Both);
    assert_eq!(
        plain.titles(),
        vec![
            "antes de cifrar".to_string(),
            "mientras estaba cifrada".to_string()
        ]
    );
}

#[test]
fn turning_off_given_up_leaves_the_cloud_sealed() {
    let (setting, fake) = drive();
    sealed_on(&setting, &fake);
    let turning = turning_off(&fake, &setting).unwrap();

    turning.abort(|| Ok(())).unwrap();

    assert_eq!(
        tisty_sync::sealing_in(&word(&fake)),
        tisty_sync::Sealing::Sealed(CLOUD.into())
    );
}

#[test]
fn what_a_turn_leaves_behind_goes_only_where_its_twin_stands() {
    let (setting, fake) = drive();
    sealed_on(&setting, &fake);
    let plain: Arc<dyn Remote> = fake.clone();
    let home = home_of(&setting, "dev_a");

    let gone = crate::wipe_behind(plain.clone(), &home, key(), CLOUD)
        .unwrap()
        .gone;

    assert!(gone > 0);
    let left: Vec<String> = fake
        .list("")
        .unwrap()
        .into_iter()
        .map(|one| one.name)
        .filter(|name| !name.starts_with("sealed/"))
        .collect();
    assert!(
        left.iter()
            .all(|name| crate::sealed::stays_plain(name, false)),
        "{left:?}"
    );
    to_the_end(&mut turning_off(&fake, &setting).unwrap());
    crate::wipe_behind(plain, &home, key(), CLOUD).unwrap();
    assert!(fake.list("sealed").unwrap().is_empty());
}

#[test]
fn a_wipe_says_how_much_it_left_without_a_twin_and_leaves_it_alone() {
    let (setting, fake) = drive();
    sealed_on(&setting, &fake);
    let plain: Arc<dyn Remote> = fake.clone();
    let home = home_of(&setting, "dev_a");
    let dir = tempfile::tempdir().unwrap();
    let stray = dir.path().join("stray");
    std::fs::write(&stray, b"written after the copy").unwrap();
    fake.put("docs/zzzzzzzz-0999.md", &stray, Expect::Any)
        .unwrap();

    let wiped = crate::wipe_behind(plain, &home, key(), CLOUD).unwrap();

    assert!(wiped.gone > 0);
    assert_eq!(wiped.left, 1);
    assert!(fake.about("docs/zzzzzzzz-0999.md").unwrap().is_some());
}

#[test]
fn giving_up_cut_half_way_is_undone_the_rest_of_the_way_from_the_journal() {
    let (setting, fake) = drive();
    let before = {
        written(&setting);
        word(&fake)
    };
    let turning = turned(&fake, &setting).unwrap();
    fake.fail_after(2, Hitch::Unreachable(None, "cut".into()));
    assert!(turning.abort(|| Ok(())).is_err());

    let plain: Arc<dyn Remote> = fake.clone();
    let mut left = Turning::resume(plain, &home_of(&setting, "dev_a"), key()).unwrap();

    assert_eq!(left.step().unwrap(), Stepped::GaveUp);
    assert_eq!(
        tisty_sync::sealing_in(&word(&fake)),
        tisty_sync::sealing_in(&before)
    );
    assert!(!word(&fake).contains("sealing"));
    assert!(fake.list("sealed").unwrap().is_empty());
}

#[test]
fn a_copy_of_big_files_tells_how_far_it_went_between_them() {
    let (setting, fake) = drive();
    let one = written(&setting);
    for name in ["uno.mov", "dos.mov"] {
        let big = one.kept(name, &vec![7u8; 70 * 1024]);
        one.store()
            .append(big.told(tisty_core::attach::Via::Window))
            .unwrap();
    }
    one.round(Way::Both);

    let mut turning = turned(&fake, &setting).unwrap();
    let mut told = 0;
    for _ in 0..200 {
        match turning.step().unwrap() {
            Stepped::Copying { left, .. } if left > 0 => told += 1,
            Stepped::Done | Stepped::GaveUp => break,
            _ => {}
        }
    }

    assert!(told >= 2, "both files went in one step, told {told} times");
}

#[test]
fn a_file_too_big_to_seal_is_said_never_copied_and_never_wiped() {
    let most = 64 * 1024;
    let (setting, fake) = each_kind(|fake| fake.with_most_per_file(most)).remove(0);
    let one = written(&setting);
    let big = one.kept("grande.bin", &vec![7u8; (most - 10) as usize]);
    one.store()
        .append(big.told(tisty_core::attach::Via::Window))
        .unwrap();
    one.round(Way::Both);
    let held = fake
        .list("")
        .unwrap()
        .into_iter()
        .map(|seen| seen.name)
        .find(|name| name.starts_with(&format!("{}/", tisty_sync::HELD)))
        .expect("the attachment went up plain");

    let mut turning = turned(&fake, &setting).unwrap();
    while turning.refused().is_empty() {
        turning.step().unwrap();
    }
    assert!(
        turning.refused().contains_key(&held),
        "{:?}",
        turning.refused()
    );
    to_the_end(&mut turning);
    let plain: Arc<dyn Remote> = fake.clone();
    crate::wipe_behind(plain, &home_of(&setting, "dev_a"), key(), CLOUD).unwrap();

    assert!(
        fake.about(&held).unwrap().is_some(),
        "{} was wiped with no twin",
        big.sha256
    );
}

#[test]
fn giving_up_taken_up_again_never_undoes_a_cloud_another_machine_sealed_since() {
    let (setting, fake) = drive();
    let one = written(&setting);
    let mine = turned(&fake, &setting).unwrap();
    fake.fail_after(3, Hitch::Unreachable(None, "cut".into()));
    assert!(mine.abort(|| Ok(())).is_err());
    let plain: Arc<dyn Remote> = fake.clone();
    let mut theirs = Turning::begin(
        plain.clone(),
        &home_of(&setting, "dev_b"),
        key(),
        CLOUD,
        "dev_b",
        Turn::On,
    )
    .unwrap();
    to_the_end(&mut theirs);

    let mut left = Turning::resume(plain, &home_of(&setting, "dev_a"), key()).unwrap();
    assert_eq!(left.step().unwrap(), Stepped::GaveUp);

    let two = reading_sealed(&setting, &fake, "dev_c", &one);
    two.round(Way::Both);
    assert_eq!(two.titles(), vec!["antes de cifrar".to_string()]);
}

#[test]
fn turning_off_never_takes_a_file_that_was_too_big_to_seal_for_gone() {
    let most = 64 * 1024;
    let (setting, fake) = each_kind(|fake| fake.with_most_per_file(most)).remove(0);
    let one = written(&setting);
    let big = one.kept("grande.bin", &vec![7u8; (most - 10) as usize]);
    one.store()
        .append(big.told(tisty_core::attach::Via::Window))
        .unwrap();
    one.round(Way::Both);
    to_the_end(&mut turned(&fake, &setting).unwrap());
    let held_name = fake
        .list("")
        .unwrap()
        .into_iter()
        .map(|seen| seen.name)
        .find(|name| name.starts_with(&format!("{}/", tisty_sync::HELD)))
        .unwrap();

    to_the_end(&mut turning_off(&fake, &setting).unwrap());

    assert!(fake.about(&held_name).unwrap().is_some());
}

#[test]
fn giving_up_a_turn_off_takes_away_the_plain_copies_it_wrote() {
    let (setting, fake) = drive();
    sealed_on(&setting, &fake);
    let plain: Arc<dyn Remote> = fake.clone();
    crate::wipe_behind(plain, &home_of(&setting, "dev_a"), key(), CLOUD).unwrap();
    let mut turning = turning_off(&fake, &setting).unwrap();
    turning.step().unwrap();

    turning.abort(|| Ok(())).unwrap();

    let left: Vec<String> = fake
        .list("")
        .unwrap()
        .into_iter()
        .map(|one| one.name)
        .filter(|name| !name.starts_with("sealed/"))
        .filter(|name| !crate::sealed::stays_plain(name, false))
        .collect();
    assert!(left.is_empty(), "plain copies stayed: {left:?}");
}

#[test]
fn turning_off_with_another_phrase_is_refused_before_anything_is_claimed() {
    let (setting, fake) = drive();
    sealed_on(&setting, &fake);
    let plain: Arc<dyn Remote> = fake.clone();
    let before = word(&fake);
    let other = CloudKey::of(&Seed::from_bytes(&[8; 32]));

    let refused = Turning::begin(
        plain,
        &home_of(&setting, "dev_a"),
        other,
        CLOUD,
        "dev_a",
        Turn::Off,
    );

    assert!(matches!(
        refused,
        Err(Hitch::Barred(Trouble::OtherPhrase(_)))
    ));
    assert_eq!(
        tisty_sync::sealing_in(&word(&fake)),
        tisty_sync::sealing_in(&before)
    );
    assert!(!word(&fake).contains("sealing"));
}

#[test]
fn a_turn_is_never_begun_over_one_left_unfinished() {
    let (setting, fake) = drive();
    written(&setting);
    let _left = turned(&fake, &setting).unwrap();

    let again = turned(&fake, &setting);

    assert!(matches!(again, Err(Hitch::Barred(Trouble::Sealing(_)))));
}

#[test]
fn a_stale_copy_of_a_file_now_too_big_for_the_other_side_is_taken_away() {
    let most = 64 * 1024;
    let (setting, fake) = each_kind(|fake| fake.with_most_per_file(most)).remove(0);
    let one = written(&setting);
    let small = one.kept("crece.bin", &vec![7u8; 1000]);
    one.store()
        .append(small.told(tisty_core::attach::Via::Window))
        .unwrap();
    one.round(Way::Both);
    let held_name = fake
        .list("")
        .unwrap()
        .into_iter()
        .map(|seen| seen.name)
        .find(|name| name.starts_with(&format!("{}/", tisty_sync::HELD)))
        .unwrap();
    let sealed = crate::Sealed::new(fake.clone(), key(), CLOUD);
    let dir = tempfile::tempdir().unwrap();
    let stale = dir.path().join("stale");
    std::fs::write(&stale, b"old").unwrap();
    sealed.put(&held_name, &stale, Expect::Any).unwrap();
    let big = dir.path().join("big");
    std::fs::write(&big, vec![9u8; (most - 10) as usize]).unwrap();
    fake.put(&held_name, &big, Expect::Any).unwrap();

    to_the_end(&mut turned(&fake, &setting).unwrap());

    assert!(
        sealed.about(&held_name).unwrap().is_none(),
        "the stale sealed copy stayed"
    );
}

#[test]
fn another_machine_s_claim_is_given_up_only_once_it_is_old_enough() {
    let (setting, fake) = drive();
    written(&setting);
    let mut theirs = turned(&fake, &setting).unwrap();
    theirs.step().unwrap();
    let plain: Arc<dyn Remote> = fake.clone();
    let home = home_of(&setting, "dev_b");
    let week = 7 * 24 * 60 * 60;

    let given = |idle: u64| Turning::orphaned(plain.clone(), &home, key(), CLOUD, idle, |_| Ok(()));
    assert_eq!(given(week).unwrap(), None);
    assert!(tisty_sync::sealing_in(&word(&fake)) != tisty_sync::Sealing::Plain);

    assert_eq!(given(0).unwrap(), Some(Turn::On));
    assert_eq!(Turning::left(&home), None);
    assert_eq!(
        tisty_sync::sealing_in(&word(&fake)),
        tisty_sync::Sealing::Plain
    );
    assert!(fake.list("sealed").unwrap().is_empty());
}

#[test]
fn a_turn_left_half_way_says_which_way_without_the_key_and_the_size_it_turns() {
    let (setting, fake) = drive();
    written(&setting);
    let home = home_of(&setting, "dev_a");
    assert_eq!(Turning::left(&home), None);
    assert!(crate::turn_size(&*fake).unwrap() > 0);

    let _turning = turned(&fake, &setting).unwrap();

    assert!(Turning::left(&home).is_some_and(|left| left.on));
}

#[test]
fn another_machine_s_turn_off_given_up_takes_away_the_plain_copies_it_wrote() {
    let (setting, fake) = drive();
    sealed_on(&setting, &fake);
    let plain: Arc<dyn Remote> = fake.clone();
    crate::wipe_behind(plain.clone(), &home_of(&setting, "dev_a"), key(), CLOUD).unwrap();
    let mut theirs = turning_off(&fake, &setting).unwrap();
    theirs.step().unwrap();
    let mut told = None;

    let given = Turning::orphaned(
        plain,
        &home_of(&setting, "dev_b"),
        key(),
        CLOUD,
        0,
        |turn| {
            told = Some(turn);
            Ok(())
        },
    );

    assert_eq!(given.unwrap(), Some(Turn::Off));
    assert_eq!(told, Some(Turn::Off));
    assert_eq!(
        tisty_sync::sealing_in(&word(&fake)),
        tisty_sync::Sealing::Sealed(CLOUD.into())
    );
    let left: Vec<String> = fake
        .list("")
        .unwrap()
        .into_iter()
        .map(|one| one.name)
        .filter(|name| !name.starts_with("sealed/"))
        .filter(|name| !crate::sealed::stays_plain(name, false))
        .collect();
    assert!(left.is_empty(), "plain copies stayed: {left:?}");
}

#[test]
fn giving_up_refused_before_it_starts_leaves_the_turn_going() {
    let (setting, fake) = drive();
    written(&setting);
    let mut turning = turned(&fake, &setting).unwrap();
    turning.step().unwrap();
    let plain: Arc<dyn Remote> = fake.clone();
    let home = home_of(&setting, "dev_a");

    let refused = Turning::resume(plain.clone(), &home, key())
        .unwrap()
        .abort(|| Err(Hitch::Broke("the log could not be written".into())));

    assert!(refused.is_err());
    to_the_end(&mut Turning::resume(plain, &home, key()).unwrap());
    assert_eq!(
        tisty_sync::sealing_in(&word(&fake)),
        tisty_sync::Sealing::Sealed(CLOUD.into())
    );
}

#[test]
fn giving_up_a_turn_that_already_switched_says_so_and_undoes_nothing() {
    let (setting, fake) = drive();
    written(&setting);
    let plain: Arc<dyn Remote> = fake.clone();
    let home = home_of(&setting, "dev_a");
    let mut turning = turned(&fake, &setting).unwrap();
    let late = Turning::resume(plain, &home, key()).unwrap();
    to_the_end(&mut turning);

    assert!(!late.abort(|| panic!("nothing is undone")).unwrap());
    assert_eq!(
        tisty_sync::sealing_in(&word(&fake)),
        tisty_sync::Sealing::Sealed(CLOUD.into())
    );
}
