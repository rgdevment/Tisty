use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use tisty_carrier::folio_client::{Answered, Asked, Door, Refused, Stopped, Taken, Why};
use tisty_carrier::folio_limits::SharingNow;
use tisty_carrier::folio_terms::{Standing, Unsure};
use tisty_carrier::{folio_today, policy};
use tisty_core::event::Signature;
use tisty_envelope::family::Going;
use tisty_envelope::sealing::Sealed as Envelopes;
use tisty_share_proto::identity::Revocation;
use tisty_sharing::Remembered;

use crate::answers::folio_sync::{Ahead, before_upload};

use super::{
    Agreed, Card, after_terms_refused, card_of, counted_as_shared, delivered,
    first_time_without_room, folio_terms_drawn, going_with, headings_over, held_back_here,
    kept_from, may_start, new_terms_asked, newest_first, press_given_back, said, said_kept,
    said_to_whom, sent_taken_back,
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
                editors: None,
                files,
            },
        }],
        told: Vec::new(),
        remembering: vec![("file".to_owned(), remembered(Some("key"), None))],
        anchors: BTreeMap::new(),
        changed: BTreeSet::from(["Ab3xK9pQ".to_owned()]),
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

    let home = tempfile::tempdir().unwrap();
    delivered(
        &folio,
        home.path(),
        None,
        &kept_in(&kept),
        &sealed_for(None),
        &starts(),
        3,
    )
    .unwrap();

    assert_eq!(folio_today::shared_today(home.path(), policy::now()), 1);
    let asked = folio.asked.borrow();
    assert_eq!(asked[0].0, "/docs/Ab3xK9pQ");
    assert!(asked[0].1.contains(&("Folio-Start", "mac".to_owned())));
    assert!(asked[0].1.contains(&("Folio-Ttl-Days", "3".to_owned())));
    assert!(asked[0].1.contains(&("Folio-Admin", "admin".to_owned())));
    assert_eq!(kept.borrow()[0].0, "file");
}

#[test]
fn a_language_that_goes_with_its_anchor_is_not_a_new_share_of_the_day() {
    let home = tempfile::tempdir().unwrap();
    let folio = FakeFolio::default();
    folio
        .answers
        .borrow_mut()
        .push_back((200, r#"{"stored":true,"version":1}"#));
    let mut sealed = sealed_for(None);
    sealed
        .anchors
        .insert("Ab3xK9pQ".to_owned(), "Anchor12".to_owned());

    delivered(
        &folio,
        home.path(),
        None,
        &kept_in(&Kept::default()),
        &sealed,
        &starts(),
        1,
    )
    .unwrap();

    assert_eq!(folio_today::shared_today(home.path(), policy::now()), 0);
}

#[test]
fn a_new_version_is_not_a_new_share_of_the_day() {
    let home = tempfile::tempdir().unwrap();
    let folio = FakeFolio::default();
    folio
        .answers
        .borrow_mut()
        .push_back((200, r#"{"stored":true,"version":3}"#));

    delivered(
        &folio,
        home.path(),
        None,
        &kept_in(&Kept::default()),
        &sealed_for(None),
        &BTreeMap::new(),
        1,
    )
    .unwrap();

    assert_eq!(folio_today::shared_today(home.path(), policy::now()), 0);
}

fn a_root_and_what_it_cites() -> (tisty_sharing::Sealed, BTreeMap<String, String>) {
    let top = remembered(Some("key"), None);
    let mut cited = remembered(Some("key"), None);
    cited.id = "Cited123".to_owned();
    cited.held_by = Some(top.id.clone());
    let going = |id: &str| Going {
        id: id.to_owned(),
        sealed: Envelopes {
            main: vec![1],
            files: None,
            files_hash: None,
            editors: None,
        },
    };
    let sealed = tisty_sharing::Sealed {
        going: vec![going(&top.id), going(&cited.id)],
        told: Vec::new(),
        remembering: vec![("top".to_owned(), top), ("cited".to_owned(), cited)],
        anchors: BTreeMap::new(),
        changed: BTreeSet::from(["Ab3xK9pQ".to_owned(), "Cited123".to_owned()]),
    };
    let starts = BTreeMap::from([
        ("Ab3xK9pQ".to_owned(), "mac".to_owned()),
        ("Cited123".to_owned(), "mac".to_owned()),
    ]);
    (sealed, starts)
}

#[test]
fn what_is_cited_goes_before_what_cites_it() {
    let home = tempfile::tempdir().unwrap();
    let folio = FakeFolio::default();
    folio.answers.borrow_mut().extend([
        (200, r#"{"stored":true,"version":1}"#),
        (200, r#"{"stored":true,"version":1}"#),
    ]);
    let (sealed, starts) = a_root_and_what_it_cites();

    delivered(
        &folio,
        home.path(),
        None,
        &kept_in(&Kept::default()),
        &sealed,
        &starts,
        1,
    )
    .unwrap();

    let asked = folio.asked.borrow();
    assert_eq!(asked[0].0, "/docs/Cited123");
    assert_eq!(asked[1].0, "/docs/Ab3xK9pQ");
    assert_eq!(folio_today::shared_today(home.path(), policy::now()), 2);
}

#[test]
fn past_the_day_s_new_documents_the_root_is_never_left_citing_nothing() {
    let home = tempfile::tempdir().unwrap();
    let kept = Kept::default();
    let folio = FakeFolio::default();
    folio
        .answers
        .borrow_mut()
        .push_back((429, r#"{"refused":"shares_today","until":1800000000000}"#));
    let (sealed, starts) = a_root_and_what_it_cites();

    let told = delivered(
        &folio,
        home.path(),
        None,
        &kept_in(&kept),
        &sealed,
        &starts,
        1,
    )
    .unwrap_err();

    assert_eq!(told.code, "folioSharesToday");
    assert_eq!(folio.asked.borrow().len(), 1, "the root never went");
    assert!(kept.borrow().is_empty());
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
        tempfile::tempdir().unwrap().path(),
        None,
        &kept_in(&kept),
        &sealed_for(Some(vec![2])),
        &starts(),
        1,
    )
    .unwrap();

    assert_eq!(folio.asked.borrow()[1].0, "/docs/Ab3xK9pQ/files");
}

fn taken(many: usize) -> Vec<Taken> {
    (0..many)
        .map(|one| Taken {
            root: "root".to_owned(),
            machine: format!("machine{one}"),
            at: "1".to_owned(),
            signed: "signed".to_owned(),
        })
        .collect()
}

#[test]
fn a_key_folio_has_no_room_for_is_told_even_when_an_earlier_chunk_went() {
    let folio = FakeFolio::default();
    folio.answers.borrow_mut().extend([
        (200, r#"{"done":true}"#),
        (409, r#"{"refused":"machines_full"}"#),
    ]);

    assert!(sent_taken_back(
        &folio,
        tempfile::tempdir().unwrap().path(),
        &taken(9)
    ));
    assert_eq!(folio.asked.borrow().len(), 2);
}

#[test]
fn past_the_hour_it_asks_no_more_and_says_nothing_of_room() {
    let folio = FakeFolio::default();
    folio.answers.borrow_mut().extend([
        (200, r#"{"done":true}"#),
        (429, r#"{"refused":"quota","until":1}"#),
        (409, r#"{"refused":"machines_full"}"#),
    ]);

    assert!(!sent_taken_back(
        &folio,
        tempfile::tempdir().unwrap().path(),
        &taken(24)
    ));
    assert_eq!(folio.asked.borrow().len(), 2);
}

#[test]
fn the_newest_key_taken_back_goes_first() {
    let at = |day: u8| format!("2026-11-{day:02}T00:00:00Z").parse().unwrap();
    let said = |machine: &str, day: u8| Revocation {
        root: "root".to_owned(),
        share: machine.to_owned(),
        at: at(day),
        sig: "signed".to_owned(),
    };
    let all = [said("old", 1), said("newest", 3), said("middle", 2)];

    let order: Vec<String> = newest_first(all.iter())
        .into_iter()
        .map(|one| one.machine)
        .collect();

    assert_eq!(order, ["newest", "middle", "old"]);
}

#[test]
fn no_room_is_said_once_for_each_key_that_finds_none() {
    let home = tempfile::tempdir().unwrap();
    assert!(first_time_without_room(home.path(), 65));
    assert!(!first_time_without_room(home.path(), 65));
    assert!(!first_time_without_room(home.path(), 64));
    assert!(first_time_without_room(home.path(), 66));
}

#[test]
fn a_key_that_went_or_can_go_again_next_sync_is_not_told() {
    let folio = FakeFolio::default();
    folio
        .answers
        .borrow_mut()
        .push_back((200, r#"{"done":true}"#));
    assert!(!sent_taken_back(
        &folio,
        tempfile::tempdir().unwrap().path(),
        &taken(1)
    ));

    let unreachable = FakeFolio::default();
    assert!(!sent_taken_back(
        &unreachable,
        tempfile::tempdir().unwrap().path(),
        &taken(1)
    ));

    let busy = FakeFolio::default();
    busy.answers
        .borrow_mut()
        .push_back((429, r#"{"refused":"quota","until":1}"#));
    assert!(!sent_taken_back(
        &busy,
        tempfile::tempdir().unwrap().path(),
        &taken(1)
    ));
}

#[test]
fn what_folio_refused_is_not_remembered_as_sent() {
    let kept = Kept::default();
    let folio = FakeFolio::default();
    folio
        .answers
        .borrow_mut()
        .push_back((409, r#"{"refused":"shares_full"}"#));

    let told = delivered(
        &folio,
        tempfile::tempdir().unwrap().path(),
        None,
        &kept_in(&kept),
        &sealed_for(None),
        &starts(),
        1,
    )
    .unwrap_err();

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
        tempfile::tempdir().unwrap().path(),
        None,
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

    let told = delivered(
        &folio,
        tempfile::tempdir().unwrap().path(),
        None,
        &kept_in(&kept),
        &sealed_for(None),
        &starts(),
        1,
    )
    .unwrap_err();

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
        sealed_under: None,
        version_by: None,
        version_root: None,
    }
}

#[test]
fn each_refusal_from_folio_is_said_in_words_the_window_knows() {
    for (stopped, code) in [
        (Stopped::Unreached, "folioUnreached"),
        (Stopped::Garbled, "folioGarbled"),
        (Stopped::Refused(Refused::SharesFull), "folioFull"),
        (
            Stopped::Refused(Refused::RoomFull {
                held: None,
                arriving: None,
                at_most: None,
            }),
            "folioRoomFull",
        ),
        (
            Stopped::Refused(Refused::LanguagesFull),
            "folioLanguagesFull",
        ),
        (Stopped::Refused(Refused::MachinesFull), "folioRefused"),
        (Stopped::Refused(Refused::KeyBlocked), "folioKeyBlocked"),
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
fn a_refusal_over_the_terms_carries_the_version_folio_asked_for() {
    let told = said(&Stopped::Refused(Refused::Terms {
        version: Some("12".to_owned()),
    }));
    let unnamed = said(&Stopped::Refused(Refused::Terms { version: None }));

    assert_eq!(told.code, "folioTermsToAccept");
    assert_eq!(told.name.as_deref(), Some("12"));
    assert_eq!(unnamed.code, "folioTermsToAccept");
    assert_eq!(unnamed.name, None);
}

#[test]
fn a_blocked_key_names_the_owner_when_an_editor_uploads() {
    let home = tempfile::tempdir().unwrap();
    let blocked = || crate::Refusal::of("folioKeyBlocked");

    assert_eq!(
        said_to_whom(blocked(), true, home.path()).code,
        "folioKeyBlocked"
    );
    assert_eq!(
        said_to_whom(blocked(), false, home.path()).code,
        "folioOwnerKeyBlocked"
    );
    assert_eq!(
        said_to_whom(crate::Refusal::of("folioTooBig"), false, home.path()).code,
        "folioTooBig"
    );
}

#[test]
fn a_full_folio_names_its_signed_figure_or_says_it_without_one() {
    let home = tempfile::tempdir().unwrap();
    let said = |code| said_to_whom(crate::Refusal::of(code), true, home.path());

    assert_eq!(said("folioFull").code, "folioFull");
    assert_eq!(said("folioLanguagesFull").code, "folioLanguagesFull");
    assert_eq!(said("folioRoomFull").code, "folioRoomFullAll");

    std::fs::write(
        home.path().join("folio-limits.json"),
        r#"{"serial":30,"shares_at_most":2,"languages_at_most":20,"key_bytes_at_most":25000000}"#,
    )
    .unwrap();
    let full = said("folioFull");
    assert_eq!(
        (full.code, full.name.as_deref()),
        ("folioFullOf", Some("2"))
    );
    let tongues = said("folioLanguagesFull");
    assert_eq!(
        (tongues.code, tongues.name.as_deref()),
        ("folioLanguagesFullOf", Some("20"))
    );
}

#[test]
fn a_key_out_of_room_says_whose_and_how_much_folio_signs_for() {
    let home = tempfile::tempdir().unwrap();
    let full = || crate::Refusal::of("folioRoomFull");

    assert_eq!(
        said_to_whom(full(), false, home.path()).code,
        "folioOwnerRoomFullAll"
    );

    std::fs::write(
        home.path().join("folio-limits.json"),
        r#"{"serial":30,"key_bytes_at_most":25000000}"#,
    )
    .unwrap();
    let editor = said_to_whom(full(), false, home.path());
    assert_eq!(editor.code, "folioOwnerRoomFull");
    assert_eq!(editor.name.as_deref(), Some("25000000"));
    let owner = said_to_whom(full(), true, home.path());
    assert_eq!(owner.code, "folioRoomFull");
    assert_eq!(owner.name.as_deref(), Some("25000000"));
}

#[test]
fn a_key_out_of_room_hands_on_folio_s_figures_and_whose_they_are() {
    let home = tempfile::tempdir().unwrap();
    let told = said(&Stopped::Refused(Refused::RoomFull {
        held: Some(23_400_000),
        arriving: Some(4_000_000),
        at_most: Some(25_000_000),
    }));

    assert_eq!(told.code, "folioRoomFullOf");
    assert_eq!(told.name.as_deref(), Some("23400000 4000000 25000000"));
    let editor = said_to_whom(told.clone(), false, home.path());
    assert_eq!(editor.code, "folioOwnerRoomFullOf");
    assert_eq!(editor.name, told.name);
    assert_eq!(
        said_to_whom(told, true, home.path()).code,
        "folioRoomFullOf"
    );
}

#[test]
fn a_wait_tells_the_window_until_when() {
    let told = said(&Stopped::Refused(Refused::Slow { until: 99 }));

    assert_eq!(told.code, "folioWait");
    assert_eq!(told.name.as_deref(), Some("99"));

    let today = said(&Stopped::Refused(Refused::SharesToday { until: 120 }));
    assert_eq!(today.code, "folioSharesToday");
    let network = said(&Stopped::Refused(Refused::Quota { until: 120 }));
    assert_eq!(network.code, "folioQuota");
    assert_eq!(network.name.as_deref(), Some("120"));
}

#[test]
fn the_edit_link_exists_only_where_a_token_does() {
    assert_eq!(
        card_of(&remembered(Some("key"), None), &Signature::default(), None),
        Some(Card {
            read: "https://tisty.cl/Ab3xK9pQ#key".to_owned(),
            edit: None,
            version: 2,
            taken: false,
            held: false,
            told: Vec::new(),
            owner: None,
            signer: None,
            editors: Vec::new(),
            signs_as: None,
        })
    );
    assert_eq!(
        card_of(
            &remembered(Some("key"), Some("token")),
            &Signature::default(),
            None
        )
        .and_then(|one| one.edit),
        Some("https://tisty.cl/Ab3xK9pQ#key.token".to_owned())
    );
}

#[test]
fn what_has_no_link_key_has_no_card() {
    assert_eq!(
        card_of(&remembered(None, None), &Signature::default(), None),
        None
    );
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
fn a_refusal_this_version_does_not_know_asks_to_update_it() {
    assert_eq!(
        said(&Stopped::Refused(Refused::Unknown)).code,
        "folioOutdated"
    );
    assert_eq!(
        said(&Stopped::Refused(Refused::Envelope { why: Why::Unknown })).code,
        "folioOutdated"
    );
    assert_eq!(
        said(&Stopped::Refused(Refused::Envelope { why: Why::Damaged })).code,
        "folioRefused"
    );
}

#[test]
fn a_press_comes_back_only_when_nothing_reached_folio() {
    let home = tempfile::tempdir().unwrap();
    tisty_carrier::folio_pace::press(home.path(), "d", 100, None).unwrap();

    let kept = press_given_back(crate::Refusal::of("folioFull"), home.path(), "d", 100);
    assert_eq!(kept.code, "folioFull");
    assert_eq!(pressed_in(home.path()), 1);

    for code in [
        "folioTermsToAccept",
        "folioTermsUpdating",
        "folioTermsUnmatched",
        "folioUnreached",
        "folioUploadsOff",
    ] {
        let given = press_given_back(crate::Refusal::of(code), home.path(), "d", 100);
        assert_eq!(given.code, code);
        assert_eq!(pressed_in(home.path()), 0, "{code}");
        tisty_carrier::folio_pace::press(home.path(), "d", 100, None).unwrap();
    }
}

fn five_minutes_apart() -> tisty_carrier::folio_limits::FolioLimits {
    tisty_carrier::folio_limits::FolioLimits {
        upload_waits_minutes: Some(vec![5]),
        uploads_of_a_root_an_hour: Some(10),
        ..Default::default()
    }
}

fn latest_is(said: &'static str) -> FakeFolio {
    let folio = FakeFolio::default();
    folio.answers.borrow_mut().push_back((200, said));
    folio
}

#[test]
fn a_sync_with_nothing_changed_still_asks_for_the_latest_and_leaves_the_waits_alone() {
    let home = tempfile::tempdir().unwrap();
    let folio = latest_is(r#"{"version":2}"#);
    let was = remembered(Some("key"), None);
    let limits = five_minutes_apart();

    let ahead = before_upload(&folio, home.path(), &was, true, Some(&limits), 100);

    assert_eq!(ahead.unwrap(), Ahead::Unchanged);
    assert_eq!(folio.asked.borrow()[0].0, "/docs/Ab3xK9pQ/latest");
    assert!(!home.path().join("folio-sync-presses.json").exists());
}

#[test]
fn a_sync_with_nothing_changed_brings_what_another_sent_even_during_the_wait() {
    let home = tempfile::tempdir().unwrap();
    let was = remembered(Some("key"), None);
    let limits = five_minutes_apart();
    let first = before_upload(
        &latest_is(r#"{"version":2}"#),
        home.path(),
        &was,
        false,
        Some(&limits),
        100,
    );
    assert_eq!(first.unwrap(), Ahead::Upload);

    let ahead = before_upload(
        &latest_is(r#"{"version":3}"#),
        home.path(),
        &was,
        true,
        Some(&limits),
        150,
    );

    assert_eq!(ahead.unwrap(), Ahead::Newer);
    assert_eq!(pressed_in(home.path()), 1, "only the upload was counted");
}

#[test]
fn a_sync_with_nothing_changed_that_reaches_nothing_says_so_and_counts_nothing() {
    let home = tempfile::tempdir().unwrap();
    let was = remembered(Some("key"), None);

    let ahead = before_upload(&FakeFolio::default(), home.path(), &was, true, None, 100);

    assert_eq!(ahead.unwrap_err().code, "folioUnreached");
    assert!(!home.path().join("folio-sync-presses.json").exists());
}

#[test]
fn a_sync_with_nothing_to_send_stops_only_for_new_terms() {
    let agreed = Ok(Agreed {
        version: "1".to_owned(),
        newer_unchecked: true,
    });

    assert!(new_terms_asked(&Err(crate::Refusal::of(
        "folioTermsToAccept"
    ))));
    assert!(!new_terms_asked(&Err(crate::Refusal::of(
        "folioUploadsOff"
    ))));
    assert!(!new_terms_asked(&Err(crate::Refusal::of(
        "folioTermsUpdating"
    ))));
    assert!(!new_terms_asked(&agreed));
}

#[test]
fn a_press_during_the_wait_is_refused_here_without_asking_folio() {
    let home = tempfile::tempdir().unwrap();
    let folio = latest_is(r#"{"version":2}"#);
    let was = remembered(Some("key"), None);
    let limits = five_minutes_apart();

    assert_eq!(
        before_upload(&folio, home.path(), &was, false, Some(&limits), 100).unwrap(),
        Ahead::Upload
    );
    let refused = before_upload(&folio, home.path(), &was, false, Some(&limits), 399).unwrap_err();

    assert_eq!(refused.code, "folioWait");
    assert_eq!(refused.name.as_deref(), Some("400000"));
    assert_eq!(folio.asked.borrow().len(), 1, "only the first asked Folio");
}

#[test]
fn a_sync_cancelled_once_folio_was_asked_still_starts_the_wait() {
    let home = tempfile::tempdir().unwrap();
    let was = remembered(Some("key"), None);
    let limits = five_minutes_apart();

    let ahead = before_upload(
        &latest_is(r#"{"version":2}"#),
        home.path(),
        &was,
        false,
        Some(&limits),
        100,
    );

    assert_eq!(ahead.unwrap(), Ahead::Upload);
    assert_eq!(
        tisty_carrier::folio_pace::waiting_until(home.path(), &was.id, 101, Some(&limits)),
        Some(400)
    );
    let again = before_upload(
        &latest_is(r#"{"version":2}"#),
        home.path(),
        &was,
        false,
        Some(&limits),
        200,
    );
    assert_eq!(again.unwrap_err().code, "folioWait");
}

#[test]
fn a_press_folio_never_heard_gives_the_wait_back() {
    let home = tempfile::tempdir().unwrap();
    let was = remembered(Some("key"), None);
    let limits = five_minutes_apart();

    let unreached = before_upload(
        &FakeFolio::default(),
        home.path(),
        &was,
        false,
        Some(&limits),
        100,
    );

    assert_eq!(unreached.unwrap_err().code, "folioUnreached");
    assert_eq!(pressed_in(home.path()), 0);
}

#[test]
fn a_newer_version_from_another_is_brought_before_anything_goes_up() {
    let home = tempfile::tempdir().unwrap();
    let was = remembered(Some("key"), None);

    let ahead = before_upload(
        &latest_is(r#"{"version":3}"#),
        home.path(),
        &was,
        false,
        None,
        100,
    );

    assert_eq!(ahead.unwrap(), Ahead::Newer);
}

#[test]
fn a_family_folio_holds_back_waits_here_with_no_start_and_no_upload() {
    let home = tempfile::tempdir().unwrap();
    let folio = latest_is(r#"{"version":2,"hash":"h","nextAt":400000}"#);
    let was = remembered(Some("key"), None);

    let refused = before_upload(&folio, home.path(), &was, false, None, 100).unwrap_err();

    assert_eq!(refused.code, "folioWait");
    assert_eq!(refused.name.as_deref(), Some("400000"));
    assert_eq!(folio.asked.borrow().len(), 1, "only the latest was asked");
    assert_eq!(pressed_in(home.path()), 0);
    assert_eq!(
        tisty_carrier::folio_pace::waiting_until(home.path(), &was.id, 101, None),
        Some(400)
    );
}

#[test]
fn a_sync_with_nothing_changed_keeps_folio_s_next_time_for_the_button() {
    let home = tempfile::tempdir().unwrap();
    let folio = latest_is(r#"{"version":2,"nextAt":400000}"#);
    let was = remembered(Some("key"), None);

    let ahead = before_upload(&folio, home.path(), &was, true, None, 100);

    assert_eq!(ahead.unwrap(), Ahead::Unchanged);
    assert_eq!(
        tisty_carrier::folio_pace::waiting_until(home.path(), &was.id, 101, None),
        Some(400)
    );
}

#[test]
fn a_next_time_already_past_lets_the_upload_go() {
    let home = tempfile::tempdir().unwrap();
    let folio = latest_is(r#"{"version":2,"nextAt":50000}"#);
    let was = remembered(Some("key"), None);

    let ahead = before_upload(&folio, home.path(), &was, false, None, 100);

    assert_eq!(ahead.unwrap(), Ahead::Upload);
    assert_eq!(pressed_in(home.path()), 1);
}

#[test]
fn what_folio_refuses_until_a_time_is_kept_and_asked_for_here_first() {
    let home = tempfile::tempdir().unwrap();
    let later = (policy::now() + 3_600) * 1000;

    kept_from(
        &Stopped::Refused(Refused::Quota { until: later }),
        home.path(),
        None,
    );
    let network = held_back_here(home.path(), 0).unwrap_err();
    assert_eq!(network.code, "folioQuota");
    assert_eq!(network.name, Some(later.to_string()));
}

#[test]
fn a_day_folio_said_is_full_stops_only_what_would_be_new() {
    let home = tempfile::tempdir().unwrap();
    let tomorrow = (policy::now() + 3_600) * 1000;

    kept_from(
        &Stopped::Refused(Refused::SharesToday { until: tomorrow }),
        home.path(),
        None,
    );

    assert_eq!(
        held_back_here(home.path(), 1).unwrap_err().code,
        "folioSharesToday"
    );
    assert!(held_back_here(home.path(), 0).is_ok());
    assert!(folio_today::full_now(home.path(), policy::now()));
}

#[test]
fn a_wait_with_no_document_to_keep_it_under_is_left_to_folio() {
    let home = tempfile::tempdir().unwrap();
    let later = (policy::now() + 600) * 1000;

    let refused = said_kept(
        &Stopped::Refused(Refused::Slow { until: later }),
        home.path(),
        None,
    );

    assert_eq!(refused.code, "folioWait");
    assert!(held_back_here(home.path(), 1).is_ok());
}

#[test]
fn a_cited_document_folio_holds_back_holds_back_the_share_it_goes_with() {
    let home = tempfile::tempdir().unwrap();
    let later: u64 = 4_000_000_000_000;
    let folio = FakeFolio::default();
    folio
        .answers
        .borrow_mut()
        .push_back((429, r#"{"refused":"slow","until":4000000000000}"#));
    let (sealed, starts) = a_root_and_what_it_cites();

    let told = delivered(
        &folio,
        home.path(),
        Some("Ab3xK9pQ"),
        &kept_in(&Kept::default()),
        &sealed,
        &starts,
        1,
    )
    .unwrap_err();

    assert_eq!(told.code, "folioWait");
    assert_eq!(folio.asked.borrow()[0].0, "/docs/Cited123");
    assert_eq!(
        tisty_carrier::folio_pace::waiting_until(home.path(), "Ab3xK9pQ", policy::now(), None),
        Some(later / 1000)
    );
}

#[test]
fn revocations_folio_has_no_hour_for_keep_the_network_waiting() {
    let home = tempfile::tempdir().unwrap();
    let folio = FakeFolio::default();
    folio
        .answers
        .borrow_mut()
        .push_back((429, r#"{"refused":"quota","until":4000000000000}"#));

    sent_taken_back(&folio, home.path(), &taken(9));

    assert_eq!(
        held_back_here(home.path(), 0).unwrap_err().code,
        "folioQuota"
    );
}

#[test]
fn a_network_folio_holds_back_is_refused_here_with_its_own_words() {
    let home = tempfile::tempdir().unwrap();
    let now = policy::now();
    tisty_carrier::folio_pace::hold(
        home.path(),
        tisty_carrier::folio_pace::NETWORK,
        now + 600,
        now,
    );
    let folio = FakeFolio::default();
    let was = remembered(Some("key"), None);

    let refused = before_upload(&folio, home.path(), &was, false, None, now).unwrap_err();

    assert_eq!(refused.code, "folioQuota");
    assert!(folio.asked.borrow().is_empty());
}

fn pressed_in(home: &std::path::Path) -> usize {
    let said = std::fs::read_to_string(home.join("folio-sync-presses.json")).unwrap();
    let all: BTreeMap<String, Vec<u64>> = serde_json::from_str(&said).unwrap();
    all.values().map(Vec::len).sum()
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

#[test]
fn terms_cut_at_a_column_read_as_paragraphs_under_their_headings() {
    let terms = "## En corto\n\nFolio convierte un documento de Tisty en un link para compartirlo.\nNo tiene cuentas: el link es el permiso. Lo que compartes va cifrado, y el\nservicio lo guarda sin la llave para leerlo.\n\n- **Uno.** Algo.\n\nDenuncias: <abuso@tisty.app>.\n\n[English](TERMS.md)\n";

    let drawn = folio_terms_drawn(terms.to_owned());

    assert!(drawn.contains("<h2"), "{drawn}");
    assert!(drawn.contains("<ul"), "{drawn}");
    assert!(!drawn.contains("<br"), "{drawn}");
    assert!(!drawn.contains("English"), "{drawn}");
    assert!(drawn.contains("mailto:abuso@tisty.app"), "{drawn}");
}

#[test]
fn a_refusal_over_the_terms_shows_them_when_they_changed_and_says_so_when_they_did_not() {
    let to_accept = Ok(Standing::ToAccept {
        version: "2".to_owned(),
        text: "t".to_owned(),
        accepted_before: Some("1".to_owned()),
    });
    let said_of = |standing| after_terms_refused(standing).code;

    assert_eq!(said_of(to_accept), "folioTermsToAccept");
    assert_eq!(said_of(Ok(Standing::Accepted)), "folioTermsUnmatched");
    assert_eq!(said_of(Ok(Standing::StillAccepted)), "folioTermsUnmatched");
    assert_eq!(said_of(Err(Unsure::Unmatched)), "folioTermsUnmatched");
    assert_eq!(said_of(Err(Unsure::Settling)), "folioTermsUpdating");
    assert_eq!(said_of(Err(Unsure::NoWord)), "folioUnreached");
    assert_eq!(said_of(Err(Unsure::NotPublished)), "folioTermsNotPublished");
}
