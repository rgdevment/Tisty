use super::*;
use crate::handed::tests::{key, sealed_with, two};
use tisty_core::Op;

fn seed(n: u8) -> Seed {
    Seed::from_bytes(&[n; 32])
}

#[test]
fn the_phrase_of_this_cloud_keeps_its_key_and_signs_for_this_computer_without_keeping_the_seed() {
    let two = two();
    sealed_with(&two, &key(7));

    entered(&two.asker, &Knocking::default(), &two.asking, &seed(7)).unwrap();

    assert_eq!(held(&two.asker).config.key_handed, Some(two.cloud.clone()));
    assert_eq!(
        crate::handed::kept(&two.cloud)
            .unwrap()
            .map(|one| one.named_for(&two.cloud)),
        Some(key(7).named_for(&two.cloud))
    );
    assert!(crate::identity::phrase_key(&two.asker).unwrap().is_none());
    assert!(
        held(&two.asker)
            .state
            .persons
            .names(&seed(7).root().person())
    );
}

#[test]
fn a_phrase_that_does_not_open_this_cloud_keeps_nothing() {
    let two = two();
    sealed_with(&two, &key(7));

    let refused = entered(&two.asker, &Knocking::default(), &two.asking, &seed(4)).unwrap_err();

    assert_eq!(refused.code, "phraseNotThisCloud");
    assert_eq!(held(&two.asker).config.key_handed, None);
    assert!(crate::handed::kept(&two.cloud).unwrap().is_none());
}

#[test]
fn a_phrase_of_another_person_than_the_one_the_log_names_is_refused() {
    let two = two();
    sealed_with(&two, &key(7));
    crate::identity::begun(&mut held(&two.asker), &seed(3)).unwrap();

    let refused = entered(&two.asker, &Knocking::default(), &two.asking, &seed(7)).unwrap_err();

    assert_eq!(refused.code, "phraseOtherPerson");
}

#[test]
fn the_other_computers_heard_from_come_latest_first_and_never_this_one() {
    let two = two();
    let mut session = held(&two.asker);
    for (device, at) in [("dev_old", 100), ("dev_new", 900), ("dev_mute", 0)] {
        let device = tisty_core::DeviceId(device.into());
        session
            .commit(Op::DeviceJoin {
                d: device.clone(),
                k: Some(tisty_core::DeviceKind::Machine),
                p: None,
            })
            .unwrap();
        if at > 0 {
            session.state.heard.insert(device, at);
        }
    }

    let said: Vec<String> = heard(&session).into_iter().map(|one| one.name).collect();

    assert_eq!(
        said,
        vec![
            tisty_core::config::nicknamed("dev_new"),
            tisty_core::config::nicknamed("dev_old"),
        ]
    );
}

#[test]
fn opening_with_the_phrase_takes_this_computer_s_ask_away() {
    let two = two();
    sealed_with(&two, &key(7));
    let knocking = Knocking::default();
    crate::handed::ask_now(&two.asker, &mut knocking.asked.lock().unwrap(), &two.asking).unwrap();

    entered(&two.asker, &knocking, &two.asking, &seed(7)).unwrap();

    assert!(knocking.asked.lock().unwrap().is_none());
    assert!(
        tisty_carrier::knock::knocks(&*two.plain, &two.asking.home)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_phrase_refused_for_another_person_keeps_no_key() {
    let two = two();
    sealed_with(&two, &key(7));
    crate::identity::begun(&mut held(&two.asker), &seed(3)).unwrap();

    assert!(entered(&two.asker, &Knocking::default(), &two.asking, &seed(7)).is_err());

    assert!(crate::handed::kept(&two.cloud).unwrap().is_none());
    assert_eq!(held(&two.asker).config.key_handed, None);
}
