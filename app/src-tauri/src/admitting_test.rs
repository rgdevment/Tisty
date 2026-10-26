use super::*;
use crate::handed::tests::{Two, key, knows, sealed_with, two};
use tisty_share_proto::handing::handed;

fn knocking_of(two: &Two) -> Knocking {
    held(&two.asker).state.devices.clear();
    sealed_with(two, &key(7));
    let knocking = Knocking::default();
    ask(&two.asker, &knocking, &two.asking, None).unwrap();
    knocking
}

fn step(two: &Two, knocking: &Knocking) -> Admitted {
    shown(&two.asker, knocking, &two.asking, None).unwrap()
}

#[test]
fn a_new_computer_is_let_in_once_the_same_code_is_said_yes_to_on_both() {
    let two = two();
    let knocking = knocking_of(&two);
    let newcomers = looked(&two.giver, &two.giving, Some(key(7))).unwrap();
    assert_eq!(newcomers.len(), 1);
    assert!(!newcomers[0].taken);
    assert_eq!(step(&two, &knocking).step, "waiting");

    let code = offer(&two.giver, &two.giving, Some(key(7)), &two.asking.device.0).unwrap();
    let compared = step(&two, &knocking);
    assert_eq!(
        (compared.step, compared.code.as_deref()),
        ("compare", Some(code.as_str()))
    );
    granted(
        &two.giver,
        &two.giving,
        Some(key(7)),
        &two.asking.device.0,
        &code,
    )
    .unwrap();
    assert_eq!(
        step(&two, &knocking).step,
        "compare",
        "taken before the person said yes here"
    );

    trusted(&knocking, &two.asking, &code).unwrap();

    assert_eq!(step(&two, &knocking).step, "done");
    assert_eq!(
        crate::handed::kept(&two.cloud)
            .unwrap()
            .map(|one| one.named_for(&two.cloud)),
        Some(key(7).named_for(&two.cloud))
    );
}

#[test]
fn a_giver_swapped_on_the_way_shows_another_code_and_is_never_trusted() {
    let two = two();
    let knocking = knocking_of(&two);
    let code = offer(&two.giver, &two.giving, Some(key(7)), &two.asking.device.0).unwrap();
    let impostor = Giver::of(
        "dev_zz",
        "Impostor",
        &tisty_share_proto::handing::SigningKey::from_bytes(&[5; 32]).verifying_key(),
    );
    let ask = asked_now(&knocking, &two.asking).unwrap().unwrap();
    knock::knocked(&*two.plain, &two.asking.home, &ask).unwrap();
    knock::offered(
        &*two.plain,
        &two.asking.home,
        &two.asking.device.0,
        &impostor,
    )
    .unwrap();

    assert_ne!(step(&two, &knocking).code, Some(code.clone()));
    assert!(trusted(&knocking, &two.asking, &code).is_err());
}

#[test]
fn an_answer_from_another_than_the_trusted_giver_is_taken_away() {
    let two = two();
    let knocking = knocking_of(&two);
    let code = offer(&two.giver, &two.giving, Some(key(7)), &two.asking.device.0).unwrap();
    trusted(&knocking, &two.asking, &code).unwrap();
    let ask = asked_now(&knocking, &two.asking).unwrap().unwrap();
    let forged = handed(
        &ask,
        &key(4),
        "dev_zz",
        &tisty_share_proto::handing::SigningKey::from_bytes(&[5; 32]),
    )
    .unwrap();
    knock::answer(&*two.plain, &two.asking.home, &two.asking.device.0, &forged).unwrap();

    assert_eq!(step(&two, &knocking).step, "trusted");
    assert!(crate::handed::kept(&two.cloud).unwrap().is_none());
    assert_eq!(
        knock::answered(&*two.plain, &two.asking.home, &two.asking.device.0).unwrap(),
        None
    );
}

#[test]
fn only_the_first_computer_to_take_an_ask_lets_it_in() {
    let two = two();
    let _knocking = knocking_of(&two);
    let other = Giver::of(
        "dev_zz",
        "PC",
        &tisty_share_proto::handing::SigningKey::from_bytes(&[5; 32]).verifying_key(),
    );
    knock::offered(&*two.plain, &two.asking.home, &two.asking.device.0, &other).unwrap();

    assert!(offer(&two.giver, &two.giving, Some(key(7)), &two.asking.device.0).is_err());
    assert!(
        granted(
            &two.giver,
            &two.giving,
            Some(key(7)),
            &two.asking.device.0,
            ""
        )
        .is_err()
    );
    assert!(looked(&two.giver, &two.giving, Some(key(7))).unwrap()[0].taken);
}

#[test]
fn a_computer_already_on_the_list_is_no_newcomer_and_one_without_the_key_lets_nobody_in() {
    let two = two();
    sealed_with(&two, &key(7));
    knows(&two.giver, &two.asking);
    let knocking = Knocking::default();
    assert!(ask(&two.asker, &knocking, &two.asking, None).is_err());
    let mut asked = None;
    crate::handed::ask_now(&two.asker, &mut asked, &two.asking).unwrap();

    assert!(
        looked(&two.giver, &two.giving, Some(key(7)))
            .unwrap()
            .is_empty()
    );
    assert!(looked(&two.giver, &two.giving, None).is_err());
    assert!(ask(&two.asker, &knocking, &two.asking, Some(key(7))).is_err());
}

#[test]
fn cancelling_takes_the_ask_and_its_giver_away() {
    let two = two();
    let knocking = knocking_of(&two);
    offer(&two.giver, &two.giving, Some(key(7)), &two.asking.device.0).unwrap();

    cancelled(&knocking, &two.asking).unwrap();

    assert_eq!(step(&two, &knocking).step, "idle");
    assert!(
        knock::knocks(&*two.plain, &two.asking.home)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn an_ask_swapped_after_its_code_was_shown_gets_nothing_on_the_yes() {
    let two = two();
    let knocking = knocking_of(&two);
    let code = offer(&two.giver, &two.giving, Some(key(7)), &two.asking.device.0).unwrap();
    let mut swapped = None;
    crate::handed::ask_now(&two.asker, &mut swapped, &two.asking).unwrap();
    offer(&two.giver, &two.giving, Some(key(7)), &two.asking.device.0).unwrap();

    assert!(
        granted(
            &two.giver,
            &two.giving,
            Some(key(7)),
            &two.asking.device.0,
            &code
        )
        .is_err()
    );
    assert_eq!(
        knock::answered(&*two.plain, &two.asking.home, &two.asking.device.0).unwrap(),
        None
    );
    drop(knocking);
}

#[test]
fn an_answer_already_standing_from_another_is_put_aside_for_this_one() {
    let two = two();
    let knocking = knocking_of(&two);
    let code = offer(&two.giver, &two.giving, Some(key(7)), &two.asking.device.0).unwrap();
    let ask = asked_now(&knocking, &two.asking).unwrap().unwrap();
    let forged = handed(
        &ask,
        &key(4),
        "dev_zz",
        &tisty_share_proto::handing::SigningKey::from_bytes(&[5; 32]),
    )
    .unwrap();
    knock::answer(&*two.plain, &two.asking.home, &two.asking.device.0, &forged).unwrap();

    granted(
        &two.giver,
        &two.giving,
        Some(key(7)),
        &two.asking.device.0,
        &code,
    )
    .unwrap();
    trusted(&knocking, &two.asking, &code).unwrap();

    assert_eq!(step(&two, &knocking).step, "done");
}

#[test]
fn the_computer_that_took_an_ask_still_sees_it_as_its_own() {
    let two = two();
    let _knocking = knocking_of(&two);
    offer(&two.giver, &two.giving, Some(key(7)), &two.asking.device.0).unwrap();

    assert!(!looked(&two.giver, &two.giving, Some(key(7))).unwrap()[0].taken);
}
