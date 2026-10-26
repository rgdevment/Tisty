use super::*;
use crate::handed::tests::{Two, key, knows, sealed_with, two};
use tisty_share_proto::handing::{OneUse, SigningKey, handed};

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

fn newcomer(two: &Two) -> &str {
    &two.asking.device.0
}

fn code_shown(two: &Two, knocking: &Knocking) -> String {
    let compared = step(two, knocking);
    assert_eq!(compared.step, "compare");
    compared.code.unwrap()
}

fn impostor() -> SigningKey {
    SigningKey::from_bytes(&[5; 32])
}

#[test]
fn a_new_computer_is_let_in_once_its_code_is_typed_there_and_said_yes_to_here() {
    let two = two();
    let knocking = knocking_of(&two);
    let newcomers = looked(&two.giver, &two.giving, Some(key(7))).unwrap();
    assert_eq!(newcomers.len(), 1);
    assert!(!newcomers[0].taken);
    assert_eq!(step(&two, &knocking).step, "waiting");

    offer(&two.giver, &two.giving, Some(key(7)), newcomer(&two)).unwrap();
    let code = code_shown(&two, &knocking);
    granted(&two.giver, &two.giving, Some(key(7)), newcomer(&two), &code).unwrap();
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
fn a_code_typed_wrong_or_before_the_new_computer_shows_one_hands_nothing_over() {
    let two = two();
    let knocking = knocking_of(&two);
    offer(&two.giver, &two.giving, Some(key(7)), newcomer(&two)).unwrap();

    let early = granted(&two.giver, &two.giving, Some(key(7)), newcomer(&two), "1");
    assert_eq!(early.unwrap_err().code, "admitNotYet");
    let code = code_shown(&two, &knocking);
    let wrong: String = code
        .chars()
        .map(|one| match one {
            '0'..='8' => char::from(one as u8 + 1),
            '9' => '0',
            other => other,
        })
        .collect();
    let refused = granted(
        &two.giver,
        &two.giving,
        Some(key(7)),
        newcomer(&two),
        &wrong,
    );

    assert_eq!(refused.unwrap_err().code, "admitWrongCode");
    assert_eq!(
        knock::answered(&*two.plain, &two.asking.home, newcomer(&two)).unwrap(),
        None
    );
    let spaced = code.replace(' ', "");
    granted(
        &two.giver,
        &two.giving,
        Some(key(7)),
        newcomer(&two),
        &spaced,
    )
    .unwrap();
}

#[test]
fn a_giver_swapped_on_the_way_makes_the_typed_code_fail() {
    let two = two();
    let knocking = knocking_of(&two);
    offer(&two.giver, &two.giving, Some(key(7)), newcomer(&two)).unwrap();
    let (ask, _) = asked_now(&knocking, &two.asking).unwrap();
    knock::knocked(&*two.plain, &two.asking.home, &ask).unwrap();
    let fake = Giver::of("dev_zz", "Impostor", &impostor().verifying_key());
    knock::offered(&*two.plain, &two.asking.home, newcomer(&two), &fake).unwrap();

    let shown_there = code_shown(&two, &knocking);

    assert!(
        granted(
            &two.giver,
            &two.giving,
            Some(key(7)),
            newcomer(&two),
            &shown_there
        )
        .is_err()
    );
}

#[test]
fn an_answer_from_another_than_the_trusted_giver_is_taken_away() {
    let two = two();
    let knocking = knocking_of(&two);
    offer(&two.giver, &two.giving, Some(key(7)), newcomer(&two)).unwrap();
    let code = code_shown(&two, &knocking);
    trusted(&knocking, &two.asking, &code).unwrap();
    let (ask, _) = asked_now(&knocking, &two.asking).unwrap();
    let forged = handed(&ask, &key(4), "dev_zz", &impostor()).unwrap();
    knock::answer(&*two.plain, &two.asking.home, newcomer(&two), &forged).unwrap();

    assert_eq!(step(&two, &knocking).step, "trusted");
    assert!(crate::handed::kept(&two.cloud).unwrap().is_none());
    assert_eq!(
        knock::answered(&*two.plain, &two.asking.home, newcomer(&two)).unwrap(),
        None
    );
}

#[test]
fn only_the_first_computer_to_take_an_ask_lets_it_in() {
    let two = two();
    let _knocking = knocking_of(&two);
    let other = Giver::of("dev_zz", "PC", &impostor().verifying_key());
    knock::offered(&*two.plain, &two.asking.home, newcomer(&two), &other).unwrap();

    let taken = offer(&two.giver, &two.giving, Some(key(7)), newcomer(&two));
    assert_eq!(taken.unwrap_err().code, "admitTaken");
    let refused = granted(&two.giver, &two.giving, Some(key(7)), newcomer(&two), "");
    assert_eq!(refused.unwrap_err().code, "admitTaken");
    assert!(looked(&two.giver, &two.giving, Some(key(7))).unwrap()[0].taken);
}

#[test]
fn a_computer_already_on_the_list_is_no_newcomer_and_one_without_the_key_lets_nobody_in() {
    let two = two();
    sealed_with(&two, &key(7));
    knows(&two.giver, &two.asking);
    let knocking = Knocking::default();
    let on_list = ask(&two.asker, &knocking, &two.asking, None);
    assert_eq!(on_list.unwrap_err().code, "admitOnList");
    let mut asked = None;
    crate::handed::ask_now(&two.asker, &mut asked, &two.asking).unwrap();

    assert!(
        looked(&two.giver, &two.giving, Some(key(7)))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        looked(&two.giver, &two.giving, None).unwrap_err().code,
        "admitNoKey"
    );
    let already = ask(&two.asker, &knocking, &two.asking, Some(key(7)));
    assert_eq!(already.unwrap_err().code, "admitAlreadyIn");
}

#[test]
fn cancelling_takes_the_ask_its_giver_and_its_reveal_away() {
    let two = two();
    let knocking = knocking_of(&two);
    offer(&two.giver, &two.giving, Some(key(7)), newcomer(&two)).unwrap();
    code_shown(&two, &knocking);

    cancelled(&knocking, &two.asking).unwrap();

    assert_eq!(step(&two, &knocking).step, "idle");
    assert!(
        knock::knocks(&*two.plain, &two.asking.home)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        knock::reveal_of(&*two.plain, &two.asking.home, newcomer(&two)).unwrap(),
        None
    );
}

#[test]
fn letting_it_in_twice_keeps_the_answer_already_put_up() {
    let two = two();
    let knocking = knocking_of(&two);
    offer(&two.giver, &two.giving, Some(key(7)), newcomer(&two)).unwrap();
    let code = code_shown(&two, &knocking);
    granted(&two.giver, &two.giving, Some(key(7)), newcomer(&two), &code).unwrap();
    let first = knock::answered(&*two.plain, &two.asking.home, newcomer(&two)).unwrap();

    granted(&two.giver, &two.giving, Some(key(7)), newcomer(&two), &code).unwrap();

    assert_eq!(
        knock::answered(&*two.plain, &two.asking.home, newcomer(&two)).unwrap(),
        first
    );
}

#[test]
fn an_answer_already_trusted_is_taken_even_past_the_ask_s_hour() {
    let two = two();
    let knocking = knocking_of(&two);
    let one_use = OneUse::minted().unwrap();
    let mine = tisty_core::signing::mine(&two.asking.paths, &two.asking.device).unwrap();
    let old = Ask::signed(
        newcomer(&two),
        "PC",
        &mine,
        &one_use,
        &two.cloud,
        now() - NEW_FOR - 60,
    );
    knock::knocked(&*two.plain, &two.asking.home, &old).unwrap();
    let reveal = one_use.revealed();
    *knocking.asked.lock().unwrap() = Some((old.clone(), one_use));
    let giving = tisty_core::signing::mine(&two.giving.paths, &two.giving.device).unwrap();
    let giver = Giver::of(&two.giving.device.0, "Mac", &giving.verifying_key());
    knock::offered(&*two.plain, &two.asking.home, newcomer(&two), &giver).unwrap();
    let code = old.code(&giving.verifying_key(), &reveal).unwrap();
    trusted(&knocking, &two.asking, &code).unwrap();
    let answer = handed(&old, &key(7), &two.giving.device.0, &giving).unwrap();
    knock::answer(&*two.plain, &two.asking.home, newcomer(&two), &answer).unwrap();

    assert_eq!(step(&two, &knocking).step, "done");
}

#[test]
fn an_ask_past_its_hour_is_made_again_rather_than_left_waiting() {
    let two = two();
    let knocking = knocking_of(&two);
    let one_use = OneUse::minted().unwrap();
    let signer = tisty_core::signing::mine(&two.asking.paths, &two.asking.device).unwrap();
    let old = Ask::signed(
        newcomer(&two),
        "PC",
        &signer,
        &one_use,
        &two.cloud,
        now() - NEW_FOR - 60,
    );
    *knocking.asked.lock().unwrap() = Some((old.clone(), one_use));

    assert_eq!(step(&two, &knocking).step, "waiting");

    let up_there = knock::ask_of(&*two.plain, &two.asking.home, newcomer(&two))
        .unwrap()
        .unwrap();
    assert_ne!(up_there, old);
    assert!(now() - up_there.at < 60);
}

#[test]
fn an_answer_already_standing_from_another_is_put_aside_for_this_one() {
    let two = two();
    let knocking = knocking_of(&two);
    offer(&two.giver, &two.giving, Some(key(7)), newcomer(&two)).unwrap();
    let code = code_shown(&two, &knocking);
    let (ask, _) = asked_now(&knocking, &two.asking).unwrap();
    let forged = handed(&ask, &key(4), "dev_zz", &impostor()).unwrap();
    knock::answer(&*two.plain, &two.asking.home, newcomer(&two), &forged).unwrap();

    granted(&two.giver, &two.giving, Some(key(7)), newcomer(&two), &code).unwrap();
    trusted(&knocking, &two.asking, &code).unwrap();

    assert_eq!(step(&two, &knocking).step, "done");
}

#[test]
fn the_computer_that_took_an_ask_still_sees_it_as_its_own() {
    let two = two();
    let _knocking = knocking_of(&two);
    offer(&two.giver, &two.giving, Some(key(7)), newcomer(&two)).unwrap();

    assert!(!looked(&two.giver, &two.giving, Some(key(7))).unwrap()[0].taken);
}

#[test]
fn an_ask_past_its_hour_that_a_computer_took_is_not_made_again() {
    let two = two();
    let knocking = knocking_of(&two);
    let one_use = OneUse::minted().unwrap();
    let mine = tisty_core::signing::mine(&two.asking.paths, &two.asking.device).unwrap();
    let old = Ask::signed(
        newcomer(&two),
        "PC",
        &mine,
        &one_use,
        &two.cloud,
        now() - NEW_FOR - 60,
    );
    knock::knocked(&*two.plain, &two.asking.home, &old).unwrap();
    *knocking.asked.lock().unwrap() = Some((old.clone(), one_use));
    let giver = Giver::of("dev_zz", "Mac", &impostor().verifying_key());
    knock::offered(&*two.plain, &two.asking.home, newcomer(&two), &giver).unwrap();

    assert_eq!(step(&two, &knocking).step, "compare");

    assert_eq!(
        knock::ask_of(&*two.plain, &two.asking.home, newcomer(&two)).unwrap(),
        Some(old)
    );
}
