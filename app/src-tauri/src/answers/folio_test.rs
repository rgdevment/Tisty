use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};

use tisty_carrier::folio_client::{Answered, Asked, Door, Refused, Stopped, Why};
use tisty_carrier::folio_limits::SharingNow;
use tisty_envelope::family::Going;
use tisty_envelope::sealing::Sealed as Envelopes;
use tisty_sharing::Remembered;

use super::{
    Card, card_of, counted_as_shared, delivered, going_with, headings_over, may_start, said,
};

type Heard = (String, Vec<(&'static str, String)>);

#[derive(Default)]
struct FakeFolio {
    answers: RefCell<VecDeque<(u16, &'static str)>>,
    asked: RefCell<Vec<Heard>>,
}

impl Door for FakeFolio {
    fn ask(&self, asked: &Asked<'_>) -> Option<Answered> {
        self.asked
            .borrow_mut()
            .push((asked.path.clone(), asked.headers.clone()));
        let (status, body) = self.answers.borrow_mut().pop_front()?;
        Some(Answered {
            status,
            headers: Vec::new(),
            body: body.as_bytes().to_vec(),
        })
    }
}

fn sealed_for(files: Option<Vec<u8>>) -> tisty_sharing::Sealed {
    tisty_sharing::Sealed {
        going: vec![Going {
            id: "Ab3xK9pQ".to_owned(),
            sealed: Envelopes {
                main: vec![1],
                files_hash: files.as_ref().map(|_| "hash".to_owned()),
                files,
            },
        }],
        told: Vec::new(),
        remembering: vec![("file".to_owned(), remembered(Some("key"), None))],
        anchors: BTreeMap::new(),
    }
}

fn starts() -> BTreeMap<String, String> {
    BTreeMap::from([("Ab3xK9pQ".to_owned(), "mac".to_owned())])
}

type Kept = RefCell<Vec<(String, Remembered)>>;

fn kept_in(into: &Kept) -> impl Fn(&[(String, Remembered)]) -> crate::Answer<()> + '_ {
    |went| {
        into.borrow_mut().extend_from_slice(went);
        Ok(())
    }
}

#[test]
fn a_first_version_goes_with_what_a_start_gave_and_is_remembered_once_stored() {
    let kept = Kept::default();
    let folio = FakeFolio::default();
    folio
        .answers
        .borrow_mut()
        .push_back((200, r#"{"stored":true,"version":1}"#));

    delivered(&folio, &kept_in(&kept), &sealed_for(None), &starts(), 3).unwrap();

    let asked = folio.asked.borrow();
    assert_eq!(asked[0].0, "/docs/Ab3xK9pQ");
    assert!(asked[0].1.contains(&("Folio-Start", "mac".to_owned())));
    assert!(asked[0].1.contains(&("Folio-Ttl-Days", "3".to_owned())));
    assert!(asked[0].1.contains(&("Folio-Admin", "admin".to_owned())));
    assert_eq!(kept.borrow()[0].0, "file");
}

#[test]
fn its_files_go_when_folio_wants_them() {
    let kept = Kept::default();
    let folio = FakeFolio::default();
    folio.answers.borrow_mut().extend([
        (200, r#"{"stored":true,"version":1,"filesWanted":true}"#),
        (200, r#"{"stored":true,"hash":"hash"}"#),
    ]);

    delivered(
        &folio,
        &kept_in(&kept),
        &sealed_for(Some(vec![2])),
        &starts(),
        1,
    )
    .unwrap();

    assert_eq!(folio.asked.borrow()[1].0, "/docs/Ab3xK9pQ/files");
}

#[test]
fn what_folio_refused_is_not_remembered_as_sent() {
    let kept = Kept::default();
    let folio = FakeFolio::default();
    folio
        .answers
        .borrow_mut()
        .push_back((409, r#"{"refused":"shares_full"}"#));

    let told = delivered(&folio, &kept_in(&kept), &sealed_for(None), &starts(), 1).unwrap_err();

    assert_eq!(told.code, "folioFull");
    assert!(kept.borrow().is_empty());
}

#[test]
fn a_version_stored_stays_remembered_when_its_files_fail_and_they_go_again_next_time() {
    let kept = Kept::default();
    let folio = FakeFolio::default();
    folio.answers.borrow_mut().extend([
        (200, r#"{"stored":true,"version":1,"filesWanted":true}"#),
        (500, r#"{"refused":"failed"}"#),
    ]);

    let told = delivered(
        &folio,
        &kept_in(&kept),
        &sealed_for(Some(vec![2])),
        &starts(),
        1,
    )
    .unwrap_err();

    assert_eq!(told.code, "folioRefused");
    let shelf = kept.borrow();
    let last = &shelf.last().unwrap().1;
    assert_eq!(last.admin_token.as_deref(), Some("admin"));
    assert_eq!(last.files_hash, None);
    assert_eq!(last.words_print, None);
}

#[test]
fn only_what_one_shared_oneself_takes_a_place_among_ones_shares() {
    let own = remembered(Some("key"), None);
    assert!(counted_as_shared(&own));

    let mut taken = remembered(Some("key"), Some("token"));
    taken.admin_token = None;
    assert!(!counted_as_shared(&taken));

    let mut held = remembered(Some("key"), None);
    held.held_by = Some("Top11111".to_owned());
    assert!(!counted_as_shared(&held));
}

#[test]
fn what_a_share_holds_goes_with_it_unless_another_still_cites_it() {
    let top = remembered(Some("key"), None);
    let held = |id: &str| {
        let mut one = remembered(Some("key"), None);
        one.id = id.to_owned();
        one.held_by = Some(top.id.clone());
        one
    };
    let mut other = remembered(Some("key"), None);
    other.id = "Other111".to_owned();
    other.cited = vec!["doc-kept".to_owned()];
    let shelf = BTreeMap::from([
        ("top".to_owned(), top.clone()),
        ("lang".to_owned(), held("Lang1111")),
        ("kept".to_owned(), held("Kept1111")),
        ("other".to_owned(), other),
    ]);

    let going = going_with(&shelf, "top", &top.id, |file| Some(format!("doc-{file}")));

    let files: Vec<&str> = going.iter().map(|(file, _)| file.as_str()).collect();
    assert_eq!(files, ["lang"]);
}

#[test]
fn files_folio_wants_and_this_side_does_not_hold_are_not_passed_over() {
    let kept = Kept::default();
    let folio = FakeFolio::default();
    folio
        .answers
        .borrow_mut()
        .push_back((200, r#"{"stored":true,"version":1,"filesWanted":true}"#));

    let told = delivered(&folio, &kept_in(&kept), &sealed_for(None), &starts(), 1).unwrap_err();

    assert_eq!(told.code, "folioGarbled");
}

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
        owner: None,
        owner_name: None,
        cited: Vec::new(),
        held_by: None,
        ttl_days: None,
        carried: None,
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
            taken: false,
            held: false,
            told: Vec::new(),
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

#[test]
fn only_an_open_folio_lets_a_new_share_start() {
    assert!(may_start(SharingNow::Open).is_ok());
    assert_eq!(
        may_start(SharingNow::Closed { note: None })
            .unwrap_err()
            .code,
        "folioClosed"
    );
    assert_eq!(
        may_start(SharingNow::Unreached).unwrap_err().code,
        "folioUnreached"
    );
}
