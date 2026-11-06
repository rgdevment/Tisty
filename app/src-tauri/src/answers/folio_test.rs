use tisty_carrier::folio_client::{Refused, Stopped, Why};
use tisty_sharing::Remembered;

use super::{Card, card_of, headings_over, said};

fn remembered(link_key: Option<&str>, edit_token: Option<&str>) -> Remembered {
    Remembered {
        id: "Ab3xK9pQ".to_owned(),
        version: 2,
        link_key: link_key.map(str::to_owned),
        files_print: None,
        files_hash: None,
        files_key: None,
        admin_token: Some("admin".to_owned()),
        edit_token: edit_token.map(str::to_owned),
        words_print: None,
        signed_as: None,
    }
}

#[test]
fn each_refusal_from_folio_is_said_in_words_the_window_knows() {
    for (stopped, code) in [
        (Stopped::Unreached, "folioUnreached"),
        (Stopped::Garbled, "folioGarbled"),
        (Stopped::Refused(Refused::SharesFull), "folioFull"),
        (Stopped::Refused(Refused::TooBig), "folioTooBig"),
        (Stopped::Refused(Refused::UploadsOff), "folioUploadsOff"),
        (Stopped::Refused(Refused::Paused), "folioPaused"),
        (Stopped::Refused(Refused::Blocked), "folioBlocked"),
        (Stopped::Refused(Refused::Behind), "folioBehind"),
        (Stopped::Refused(Refused::Outdated), "folioOutdated"),
        (Stopped::Refused(Refused::Revoked), "folioRevoked"),
        (Stopped::Refused(Refused::Missing), "folioGone"),
        (Stopped::Refused(Refused::NotYours), "notOneOwnDoc"),
        (Stopped::Refused(Refused::Failed), "folioRefused"),
        (
            Stopped::Refused(Refused::Envelope { why: Why::Damaged }),
            "folioRefused",
        ),
    ] {
        assert_eq!(said(&stopped).code, code);
    }
}

#[test]
fn a_wait_tells_the_window_until_when() {
    let told = said(&Stopped::Refused(Refused::Slow { until: 99 }));

    assert_eq!(told.code, "folioWait");
    assert_eq!(told.name.as_deref(), Some("99"));
}

#[test]
fn the_edit_link_exists_only_where_a_token_does() {
    assert_eq!(
        card_of(&remembered(Some("key"), None)),
        Some(Card {
            read: "https://tisty.cl/Ab3xK9pQ#key".to_owned(),
            edit: None,
            version: 2,
        })
    );
    assert_eq!(
        card_of(&remembered(Some("key"), Some("token"))).and_then(|one| one.edit),
        Some("https://tisty.cl/Ab3xK9pQ#key.token".to_owned())
    );
}

#[test]
fn what_has_no_link_key_has_no_card() {
    assert_eq!(card_of(&remembered(None, None)), None);
}

#[test]
fn a_secret_is_placed_under_its_heading_past_properties_and_code() {
    let over = headings_over("---\ntitle: x\n---\n# Reservas\nclave\n```\n# no\n```\nmás");

    assert_eq!(over[0], None);
    assert_eq!(over[3], None);
    assert_eq!(over[4].as_deref(), Some("Reservas"));
    assert_eq!(over[8].as_deref(), Some("Reservas"));
}
