use super::*;

#[test]
fn the_download_fills_the_bar_on_its_own() {
    assert_eq!(step(0.0), ("getting", 0));
    assert_eq!(step(0.4), ("getting", 50));
    assert_eq!(step(0.79), ("getting", 98));
}

#[test]
fn what_the_store_counts_as_the_install_is_shown_as_installing() {
    assert_eq!(step(0.8), ("installing", 100));
    assert_eq!(step(0.9), ("installing", 100));
    assert_eq!(step(1.0), ("installing", 100));
}

#[test]
fn a_figure_that_makes_no_sense_still_leaves_a_bar_that_does() {
    assert_eq!(step(-1.0), ("getting", 0));
    assert_eq!(step(2.0), ("installing", 100));
}

const HERE: &str = "1.40.1";

fn offered_before(version: Option<&str>) -> Memory {
    Memory {
        version: version.map(str::to_string),
        offered: Some(true),
    }
}

fn coming_before(version: &str) -> Memory {
    Memory {
        version: Some(version.to_string()),
        offered: Some(false),
    }
}

fn known(version: &str) -> Ahead {
    Ahead::Is(version.to_string())
}

#[test]
fn the_store_s_offer_can_be_taken_whatever_is_known_of_its_number() {
    let named = verdict(Shelf::Waiting, known("1.41.1"), HERE, Memory::default());
    let ready = named.ready.expect("an offer");
    assert!(ready.installs);
    assert_eq!(ready.version, "1.41.1");
    assert_eq!(named.memory, offered_before(Some("1.41.1")));

    for ahead in [Ahead::Nothing, Ahead::Unheard] {
        let ready = verdict(Shelf::Waiting, ahead, HERE, Memory::default()).ready;
        let ready = ready.expect("an offer with no number is still an offer");
        assert!(ready.installs);
        assert_eq!(ready.version, "");
    }
}

#[test]
fn with_nobody_able_to_say_the_version_known_stays_while_it_is_still_ahead() {
    let ahead_of_us = verdict(
        Shelf::Waiting,
        Ahead::Unheard,
        HERE,
        coming_before("1.41.1"),
    );
    assert_eq!(ahead_of_us.ready.expect("an offer").version, "1.41.1");

    let behind_us = verdict(
        Shelf::Waiting,
        Ahead::Unheard,
        HERE,
        coming_before("1.40.1"),
    );
    assert_eq!(behind_us.ready.expect("an offer").version, "");
}

#[test]
fn what_already_landed_offers_nothing_to_press() {
    let landed = verdict(
        Shelf::Landed("1.41.1".into()),
        Ahead::Nothing,
        HERE,
        Memory::default(),
    );
    let ready = landed.ready.expect("it is here");
    assert!(!ready.installs);
    assert_eq!(ready.version, "1.41.1");
}

#[test]
fn a_queue_held_up_is_told_apart_from_one_that_finishes_by_itself() {
    let moving = verdict(Shelf::Queued, known("1.41.1"), HERE, Memory::default());
    let moving = moving.ready.expect("on its way");
    assert_eq!(moving.coming, Some(Coming::Queued));

    let paused = verdict(Shelf::Paused, known("1.41.1"), HERE, Memory::default());
    let paused = paused.ready.expect("paused there");
    assert_eq!(paused.coming, Some(Coming::Paused));
    assert!(!paused.installs, "only the Store can start it again");
}

#[test]
fn a_copy_the_store_did_not_sell_is_never_told_the_store_brings_anything() {
    let said = verdict(
        Shelf::NotSold,
        known("1.41.1"),
        HERE,
        offered_before(Some("1.41.1")),
    );
    assert_eq!(said.ready, None);
    assert_eq!(said.memory, Memory::default());
    assert!(!said.unanswered, "the Store was never asked");
}

#[test]
fn a_version_the_catalog_lists_is_on_its_way_even_when_the_store_says_nothing() {
    for shelf in [Shelf::Current, Shelf::Silent] {
        let said = verdict(shelf, known("1.41.1"), HERE, Memory::default());
        let ready = said.ready.expect("on its way");
        assert_eq!(ready.coming, Some(Coming::NotYet));
        assert!(!ready.installs);
        assert_eq!(said.memory, coming_before("1.41.1"));
        assert!(!said.unanswered);
    }
}

#[test]
fn a_store_that_answered_and_a_catalog_with_nothing_newer_end_the_matter() {
    let said = verdict(
        Shelf::Current,
        Ahead::Nothing,
        HERE,
        offered_before(Some("1.41.1")),
    );
    assert_eq!(said.ready, None);
    assert_eq!(said.memory, Memory::default());
    assert!(!said.unanswered);
}

#[test]
fn a_store_that_did_not_answer_is_reported_even_when_the_catalog_lists_nothing_newer() {
    let offer = verdict(
        Shelf::Silent,
        Ahead::Nothing,
        HERE,
        offered_before(Some("1.41.1")),
    );
    assert!(offer.unanswered);
    assert!(offer.ready.expect("its own offer stands").installs);
    assert_eq!(offer.memory, offered_before(Some("1.41.1")));

    let coming = verdict(Shelf::Silent, Ahead::Nothing, HERE, coming_before("1.41.1"));
    assert!(coming.unanswered);
    assert_eq!(
        coming.ready, None,
        "the catalog says no such version is out"
    );
    assert_eq!(coming.memory, Memory::default());
}

#[test]
fn the_store_saying_it_has_no_offer_takes_back_the_offer_it_made() {
    let said = verdict(
        Shelf::Current,
        Ahead::Unheard,
        HERE,
        offered_before(Some("1.41.1")),
    );
    assert_eq!(said.ready, None);
    assert_eq!(said.memory, Memory::default());

    let coming = verdict(
        Shelf::Current,
        Ahead::Unheard,
        HERE,
        coming_before("1.41.1"),
    );
    assert!(coming.ready.expect("still on its way").coming.is_some());
    assert_eq!(coming.memory, coming_before("1.41.1"));
}

#[test]
fn a_store_that_did_not_answer_leaves_what_it_last_said_standing() {
    let said = verdict(
        Shelf::Silent,
        Ahead::Unheard,
        HERE,
        offered_before(Some("1.41.1")),
    );
    assert!(said.ready.expect("still standing").installs);
    assert_eq!(said.memory, offered_before(Some("1.41.1")));
    assert!(said.unanswered);
}

#[test]
fn what_was_last_said_ends_when_this_copy_catches_up_or_had_no_number() {
    assert_eq!(last_said(HERE, &offered_before(Some("1.40.1"))), None);
    assert_eq!(last_said(HERE, &coming_before("1.40.1")), None);
    assert_eq!(last_said(HERE, &offered_before(None)), None);
    assert_eq!(last_said(HERE, &Memory::default()), None);
    let offer = last_said(HERE, &offered_before(Some("1.41.1")));
    assert!(offer.expect("an offer").installs);
    let coming = last_said(HERE, &coming_before("1.41.1"));
    assert!(coming.expect("on its way").coming.is_some());
}

#[test]
fn only_a_press_is_waited_for_and_only_when_there_may_be_something() {
    assert!(waits(true, &known("1.41.1")));
    assert!(waits(true, &Ahead::Unheard));
    assert!(!waits(true, &Ahead::Nothing));
    assert!(!waits(false, &known("1.41.1")));
    assert!(!waits(false, &Ahead::Unheard));
}

#[test]
fn only_a_press_is_told_the_store_did_not_answer() {
    let said = verdict(Shelf::Silent, Ahead::Unheard, HERE, Memory::default());
    assert!(said.refused(true));
    assert!(!said.refused(false));

    let answered = verdict(Shelf::Current, Ahead::Nothing, HERE, Memory::default());
    assert!(!answered.refused(true));
}

#[cfg(windows)]
#[test]
fn only_what_quitting_tisty_will_not_clear_counts_as_paused() {
    use windows::Services::Store::{
        StoreQueueItemExtendedState as Reason, StoreQueueItemState as State,
    };

    for reason in [
        Reason::Paused,
        Reason::PausedLowBattery,
        Reason::PausedWiFiRequired,
    ] {
        assert_eq!(there::of_the_queue(State::Paused, reason), Shelf::Paused);
    }
    for reason in [Reason::PausedPackagesInUse, Reason::PausedReadyToInstall] {
        assert_eq!(there::of_the_queue(State::Paused, reason), Shelf::Queued);
    }
    assert_eq!(
        there::of_the_queue(State::Active, Reason::ActiveDownloading),
        Shelf::Queued
    );
}
