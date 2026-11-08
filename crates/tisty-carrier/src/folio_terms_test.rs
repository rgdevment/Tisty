use std::cell::Cell;

use super::{
    Standing, TERMS_EN_URL, TERMS_ES_URL, Tongue, UNMET, UNMET_FOR, Unkept, Unsure, accept,
    accepted_version, print_of, read_again, standing,
};
use crate::folio_limits::{self, FolioLimits, FolioTerms, KEPT, LIMITS_URL, SIGNATURE_URL};
use crate::policy;

const TEXT: &str = "Términos de prueba\n";
const PRINT_MADE_BY_NODE: &str = "bb66e9e3c26bcb49f0678ff9e115b9d66bf04a26c0482f34c3bc06aa934de6de";
const REAL: &[u8] = include_bytes!("folio_signed/limits.json");
const REAL_SIG: &str = include_str!("folio_signed/limits.json.minisig");

fn terms(version: &str) -> FolioTerms {
    FolioTerms {
        version: version.to_owned(),
        es: print_of(TEXT.as_bytes()),
        en: print_of(b"Test terms\n"),
    }
}

fn seeded(terms: Option<FolioTerms>) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    let limits = FolioLimits {
        terms,
        ..FolioLimits::default()
    };
    std::fs::write(home.path().join(KEPT), serde_json::to_vec(&limits).unwrap()).unwrap();
    home
}

fn only_the_terms(url: &str) -> Option<Vec<u8>> {
    match url {
        TERMS_ES_URL => Some(TEXT.as_bytes().to_vec()),
        TERMS_EN_URL => Some(b"Test terms\n".to_vec()),
        _ => None,
    }
}

#[test]
fn a_refusal_reads_the_limits_once_and_never_stands_on_what_was_refused() {
    let home = seeded(Some(terms("1")));
    accept(home.path(), Tongue::Es, "1").unwrap();
    let asked = Cell::new(0);

    let told = read_again(
        home.path(),
        Tongue::Es,
        |url| {
            if url == LIMITS_URL {
                asked.set(asked.get() + 1);
            }
            None
        },
        Some("1"),
    );

    assert_eq!(told, Err(Unsure::Settling));
    assert_eq!(asked.get(), 1);
    assert_eq!(accepted_version(home.path()).as_deref(), Some("1"));
}

#[test]
fn a_refusal_reads_the_limits_even_within_the_hour() {
    let home = seeded(Some(terms("1")));
    accept(home.path(), Tongue::Es, "1").unwrap();
    std::fs::write(home.path().join("folio-limits-asked"), b"").unwrap();
    let asked = Cell::new(0);

    let _ = read_again(
        home.path(),
        Tongue::Es,
        |url| {
            if url == LIMITS_URL {
                asked.set(asked.get() + 1);
            }
            None
        },
        Some("1"),
    );

    assert_eq!(asked.get(), 1);
}

#[test]
fn nothing_accepted_names_no_version() {
    let home = seeded(Some(terms("1")));

    assert_eq!(accepted_version(home.path()), None);
}

#[test]
fn a_print_is_the_same_one_folio_makes_of_those_bytes() {
    assert_eq!(print_of(TEXT.as_bytes()), PRINT_MADE_BY_NODE);
}

#[test]
fn the_first_time_it_hands_over_the_signed_text_in_the_persons_language() {
    let home = seeded(Some(terms("1")));

    assert_eq!(
        standing(home.path(), Tongue::Es, only_the_terms),
        Ok(Standing::ToAccept {
            version: "1".to_owned(),
            text: TEXT.to_owned(),
            accepted_before: None,
        })
    );
    assert_eq!(
        standing(home.path(), Tongue::En, only_the_terms),
        Ok(Standing::ToAccept {
            version: "1".to_owned(),
            text: "Test terms\n".to_owned(),
            accepted_before: None,
        })
    );
}

#[test]
fn accepted_it_asks_nothing_more_and_does_not_fetch_the_text() {
    let home = seeded(Some(terms("1")));
    accept(home.path(), Tongue::Es, "1").unwrap();
    let fetched = Cell::new(0);

    let told = standing(home.path(), Tongue::Es, |url: &str| {
        if url == TERMS_ES_URL {
            fetched.set(fetched.get() + 1);
        }
        only_the_terms(url)
    });

    assert_eq!(told, Ok(Standing::Accepted));
    assert_eq!(fetched.get(), 0);
}

#[test]
fn a_new_version_asks_again_and_says_which_was_accepted() {
    let home = seeded(Some(terms("1")));
    accept(home.path(), Tongue::Es, "1").unwrap();
    let newer = seeded(Some(terms("2")));
    std::fs::copy(newer.path().join(KEPT), home.path().join(KEPT)).unwrap();

    assert_eq!(
        standing(home.path(), Tongue::Es, only_the_terms),
        Ok(Standing::ToAccept {
            version: "2".to_owned(),
            text: TEXT.to_owned(),
            accepted_before: Some("1".to_owned()),
        })
    );
}

#[test]
fn a_text_that_is_not_the_signed_one_is_never_shown() {
    let home = seeded(Some(terms("1")));

    let told = standing(home.path(), Tongue::Es, |url: &str| {
        (url == TERMS_ES_URL).then(|| b"otro texto".to_vec())
    });

    assert_eq!(told, Err(Unsure::Settling));
}

fn unmet_since(home: &std::path::Path, ago: u64, print: &str) {
    let since = policy::now() - ago;
    std::fs::write(home.join(UNMET), format!("{since} {print}")).unwrap();
}

fn another_text(url: &str) -> Option<Vec<u8>> {
    (url == TERMS_ES_URL).then(|| b"otro texto".to_vec())
}

fn accepted_then_moved_on() -> tempfile::TempDir {
    let home = seeded(Some(terms("1")));
    accept(home.path(), Tongue::Es, "1").unwrap();
    let newer = seeded(Some(terms("2")));
    std::fs::copy(newer.path().join(KEPT), home.path().join(KEPT)).unwrap();
    home
}

#[test]
fn with_nothing_accepted_an_unmet_print_waits_an_hour_and_then_says_so() {
    let home = seeded(Some(terms("1")));
    let wanted = print_of(TEXT.as_bytes());

    unmet_since(home.path(), UNMET_FOR - 30, &wanted);
    assert_eq!(
        standing(home.path(), Tongue::Es, another_text),
        Err(Unsure::Settling)
    );

    unmet_since(home.path(), UNMET_FOR, &wanted);
    assert_eq!(
        standing(home.path(), Tongue::Es, another_text),
        Err(Unsure::Unmatched)
    );
}

#[test]
fn with_terms_accepted_an_unmet_print_holds_nobody_back() {
    let home = accepted_then_moved_on();
    let wanted = print_of(TEXT.as_bytes());

    assert_eq!(
        standing(home.path(), Tongue::Es, another_text),
        Ok(Standing::Accepted)
    );

    unmet_since(home.path(), UNMET_FOR, &wanted);
    assert_eq!(
        standing(home.path(), Tongue::Es, another_text),
        Ok(Standing::StillAccepted)
    );
    assert_eq!(accepted_version(home.path()).as_deref(), Some("1"));
}

#[test]
fn a_newer_print_starts_its_own_hour() {
    let home = seeded(Some(terms("1")));
    unmet_since(home.path(), 2 * UNMET_FOR, "an earlier print");

    assert_eq!(
        standing(home.path(), Tongue::Es, another_text),
        Err(Unsure::Settling)
    );
}

#[test]
fn terms_that_match_again_end_the_wait() {
    let home = seeded(Some(terms("1")));
    unmet_since(home.path(), UNMET_FOR, &print_of(TEXT.as_bytes()));

    assert!(standing(home.path(), Tongue::Es, only_the_terms).is_ok());
    assert!(!home.path().join(UNMET).exists());
    assert_eq!(
        standing(home.path(), Tongue::Es, another_text),
        Err(Unsure::Settling)
    );
}

#[test]
fn folio_refusing_what_was_accepted_waits_for_terms_that_check_out() {
    let home = seeded(Some(terms("1")));
    accept(home.path(), Tongue::Es, "1").unwrap();

    let told = read_again(home.path(), Tongue::Es, |_: &str| None, Some("3"));
    assert_eq!(told, Err(Unsure::Settling));
    assert_eq!(accepted_version(home.path()).as_deref(), Some("1"));

    unmet_since(home.path(), UNMET_FOR, "asked:3");
    let told = read_again(home.path(), Tongue::Es, |_: &str| None, Some("3"));
    assert_eq!(told, Err(Unsure::Unmatched));
}

#[test]
fn folio_refusing_without_naming_a_version_is_timed_too() {
    let home = seeded(Some(terms("1")));
    accept(home.path(), Tongue::Es, "1").unwrap();
    unmet_since(home.path(), UNMET_FOR, "asked:");

    let told = read_again(home.path(), Tongue::Es, |_: &str| None, None);

    assert_eq!(told, Err(Unsure::Unmatched));
}

#[test]
fn a_refusal_shows_the_newer_terms_once_they_check_out() {
    let home = accepted_then_moved_on();

    let told = read_again(home.path(), Tongue::Es, only_the_terms, Some("2"));

    assert_eq!(
        told,
        Ok(Standing::ToAccept {
            version: "2".to_owned(),
            text: TEXT.to_owned(),
            accepted_before: Some("1".to_owned()),
        })
    );
}

#[test]
fn terms_that_cannot_be_brought_are_no_word_rather_than_a_wait() {
    let home = seeded(Some(terms("1")));

    let told = standing(home.path(), Tongue::Es, |_: &str| None);

    assert_eq!(told, Err(Unsure::NoWord));
    assert!(!home.path().join(UNMET).exists());
}

#[test]
fn with_terms_accepted_a_text_that_cannot_be_brought_holds_nobody_back() {
    let home = accepted_then_moved_on();

    let told = standing(home.path(), Tongue::Es, |_: &str| None);

    assert_eq!(told, Ok(Standing::Accepted));
}

#[test]
fn each_reading_asks_for_the_limits_afresh_within_the_hour() {
    let home = seeded(Some(terms("1")));
    std::fs::write(home.path().join("folio-limits-asked"), b"").unwrap();
    let limits_asked = Cell::new(0);

    let told = standing(home.path(), Tongue::Es, |url: &str| match url {
        LIMITS_URL => {
            limits_asked.set(limits_asked.get() + 1);
            None
        }
        TERMS_ES_URL => Some(b"recien publicado".to_vec()),
        _ => None,
    });

    assert_eq!(told, Err(Unsure::Settling));
    assert_eq!(limits_asked.get(), 1);
}

#[test]
fn a_press_within_a_minute_of_the_last_reads_what_was_kept() {
    let home = seeded(Some(terms("1")));
    let limits_asked = Cell::new(0);
    let counting = |url: &str| match url {
        LIMITS_URL => {
            limits_asked.set(limits_asked.get() + 1);
            Some(REAL.to_vec())
        }
        SIGNATURE_URL => Some(REAL_SIG.as_bytes().to_vec()),
        _ => only_the_terms(url),
    };

    let _ = standing(home.path(), Tongue::Es, counting);
    let _ = standing(home.path(), Tongue::Es, counting);

    assert_eq!(limits_asked.get(), 1);
}

#[test]
fn a_press_that_reached_nothing_lets_the_next_one_read_again() {
    let home = seeded(Some(terms("1")));
    let limits_asked = Cell::new(0);
    let unreached = |url: &str| {
        if url == LIMITS_URL {
            limits_asked.set(limits_asked.get() + 1);
        }
        None
    };

    let _ = standing(home.path(), Tongue::Es, unreached);
    let _ = standing(home.path(), Tongue::Es, unreached);

    assert_eq!(limits_asked.get(), 2);
}

#[test]
fn a_press_whose_limits_were_refused_lets_the_next_one_read_again() {
    let home = seeded(Some(terms("1")));
    let limits_asked = Cell::new(0);
    let unsigned = |url: &str| match url {
        LIMITS_URL => {
            limits_asked.set(limits_asked.get() + 1);
            Some(REAL.to_vec())
        }
        SIGNATURE_URL => Some(b"not a signature".to_vec()),
        _ => None,
    };

    let _ = standing(home.path(), Tongue::Es, unsigned);
    let first = limits_asked.get();
    let _ = standing(home.path(), Tongue::Es, unsigned);

    assert!(
        limits_asked.get() > first,
        "{first} then {}",
        limits_asked.get()
    );
}

fn accepted_then(current: &str) -> tempfile::TempDir {
    let home = seeded(Some(terms("1")));
    accept(home.path(), Tongue::Es, "1").unwrap();
    let newer = seeded(Some(terms(current)));
    std::fs::copy(newer.path().join(KEPT), home.path().join(KEPT)).unwrap();
    home
}

#[test]
fn terms_accepted_one_version_back_still_stand() {
    let home = accepted_then("2");

    assert_eq!(
        standing(home.path(), Tongue::Es, another_text),
        Ok(Standing::Accepted)
    );
}

#[test]
fn a_version_that_is_not_a_number_is_taken_as_still_standing() {
    let home = accepted_then("2026-11");

    assert_eq!(
        standing(home.path(), Tongue::Es, another_text),
        Ok(Standing::Accepted)
    );
}

#[test]
fn terms_accepted_two_versions_back_hold_nobody_up_as_accepted() {
    let home = seeded(Some(terms("1")));
    accept(home.path(), Tongue::Es, "1").unwrap();
    let newer = seeded(Some(terms("3")));
    std::fs::copy(newer.path().join(KEPT), home.path().join(KEPT)).unwrap();

    assert_eq!(
        standing(home.path(), Tongue::Es, another_text),
        Err(Unsure::Settling)
    );
    assert_eq!(
        standing(home.path(), Tongue::Es, |_: &str| None),
        Err(Unsure::NoWord)
    );
}

#[test]
fn signed_limits_without_terms_let_nobody_through() {
    let home = tempfile::tempdir().unwrap();

    let told = standing(home.path(), Tongue::Es, |url: &str| match url {
        LIMITS_URL => Some(REAL.to_vec()),
        SIGNATURE_URL => Some(REAL_SIG.as_bytes().to_vec()),
        _ => None,
    });

    assert_eq!(told, Err(Unsure::NotPublished));
    assert!(folio_limits::kept(home.path()).is_some());
}

#[test]
fn with_nothing_heard_and_nothing_kept_it_does_not_guess() {
    let home = tempfile::tempdir().unwrap();

    assert_eq!(
        standing(home.path(), Tongue::Es, |_: &str| None),
        Err(Unsure::NoWord)
    );
}

#[test]
fn only_the_version_folio_says_now_can_be_accepted() {
    let home = seeded(Some(terms("2")));

    assert_eq!(accept(home.path(), Tongue::Es, "1"), Err(Unkept::Moved));
    assert_eq!(
        standing(home.path(), Tongue::Es, only_the_terms),
        Ok(Standing::ToAccept {
            version: "2".to_owned(),
            text: TEXT.to_owned(),
            accepted_before: None,
        })
    );
}
