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
    assert!(moving.coming && !moving.paused);

    let paused = verdict(Shelf::Paused, known("1.41.1"), HERE, Memory::default());
    let paused = paused.ready.expect("paused there");
    assert!(paused.coming && paused.paused);
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
    assert!(said.unanswered);
}

#[test]
fn a_version_the_catalog_lists_is_on_its_way_even_when_the_store_says_nothing() {
    for shelf in [Shelf::Current, Shelf::Silent] {
        let said = verdict(shelf, known("1.41.1"), HERE, Memory::default());
        let ready = said.ready.expect("on its way");
        assert!(ready.coming && !ready.installs);
        assert_eq!(said.memory, coming_before("1.41.1"));
        assert!(!said.unanswered);
    }
}

#[test]
fn a_catalog_with_nothing_newer_ends_the_matter_whatever_was_remembered() {
    for shelf in [Shelf::Current, Shelf::Silent] {
        let said = verdict(shelf, Ahead::Nothing, HERE, offered_before(Some("1.41.1")));
        assert_eq!(said.ready, None);
        assert_eq!(said.memory, Memory::default());
        assert!(!said.unanswered);
    }
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
    assert!(coming.ready.expect("still on its way").coming);
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
    assert!(coming.expect("on its way").coming);
}
