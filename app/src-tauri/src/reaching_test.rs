use std::io::Write;
use std::net::TcpListener;

use super::*;

fn ana() -> CloudAt {
    CloudAt {
        provider: GOOGLE.into(),
        account: "0123".into(),
        email: "ana@example.com".into(),
    }
}

#[test]
fn a_build_without_google_s_ids_offers_no_cloud_and_says_so() {
    let home = tempfile::tempdir().unwrap();
    let paths = Paths::new(home.path().join("data"), home.path().join("config"));
    let reaching = Reaching::of(&paths);
    let chosen = tisty_core::config::Sync::cloud(ana());

    let keeping = tisty_carrier::chosen_with(Some(&chosen), Some(&reaching));

    assert_eq!(keeping.chosen, tisty_carrier::Chosen::Cloud);
    assert_eq!(keeping.carrier.is_some(), offers(GOOGLE));
    assert_eq!(offers(GOOGLE), tisty_google::client().is_some());
}

#[test]
fn a_provider_this_build_does_not_know_is_never_built() {
    let home = tempfile::tempdir().unwrap();
    let at = CloudAt {
        provider: "dropbox".into(),
        ..ana()
    };

    assert!(cloud_of(home.path(), &at).is_err());
}

#[test]
fn google_s_token_stays_where_1_30_kept_it_and_another_provider_is_named_first() {
    let other = CloudAt {
        provider: "onedrive".into(),
        ..ana()
    };

    assert_eq!(scope_of(&ana()), "0123");
    assert_eq!(scope_of(&other), "onedrive-0123");
    assert_eq!(offers(GOOGLE), tisty_google::client().is_some());
    assert_eq!(offers(ONEDRIVE), tisty_onedrive::client().is_some());
    assert!(!offers("dropbox"));
    assert_eq!(offered(), !clouds().is_empty());
}

#[test]
fn each_account_keeps_its_own_home() {
    let home = tempfile::tempdir().unwrap();
    let paths = Paths::new(home.path().join("data"), home.path().join("config"));
    let reaching = Reaching::of(&paths);
    let other = CloudAt {
        account: "9999".into(),
        ..ana()
    };

    assert_ne!(reaching.home_of(&ana()), reaching.home_of(&other));
}

fn served(answer: &'static [u8]) -> String {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let at = format!(
        "http://127.0.0.1:{}/cloud.json",
        listener.local_addr().unwrap().port()
    );
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut asked = [0u8; 2048];
            let _ = std::io::Read::read(&mut stream, &mut asked);
            let _ = stream.write_all(answer);
        }
    });
    at
}

#[test]
fn a_word_is_never_followed_elsewhere_nor_read_past_what_it_can_weigh() {
    let moved = served(
        b"HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:9/other\r\nContent-Length: 0\r\n\r\n",
    );
    assert_eq!(brought(&moved), None);

    let big: &'static [u8] = Box::leak(
        [
            b"HTTP/1.1 200 OK\r\nContent-Length: 40000\r\n\r\n".to_vec(),
            vec![b'x'; 40_000],
        ]
        .concat()
        .into_boxed_slice(),
    );
    let at = served(big);
    let held = brought(&at).unwrap();
    assert_eq!(held.len(), policy::READ_AT_MOST + 1);
}

#[test]
fn a_build_without_the_maintainer_s_key_never_asks() {
    let home = tempfile::tempdir().unwrap();
    if policy_key().is_some() {
        return;
    }

    heed(home.path(), GOOGLE);

    assert!(!home.path().join(POLICY_ASKED).exists());
}

#[test]
fn what_kept_a_cloud_from_being_built_is_said_and_nothing_was_kept() {
    let home = tempfile::tempdir().unwrap();
    let paths = Paths::new(home.path().join("data"), home.path().join("config"));
    let reaching = Reaching::of(&paths);
    let elsewhere = CloudAt {
        provider: "onedrive".into(),
        ..ana()
    };

    assert!(reaching.cloud(&elsewhere).is_none());
    assert!(
        reaching
            .why_not()
            .is_some_and(|why| why.contains("onedrive"))
    );
    assert!(reaching.taken().is_none());
    assert!(reaching.cloud(&elsewhere).is_none());
}

#[test]
fn a_cloud_built_apart_leaves_the_one_kept_for_the_chosen_account_alone() {
    let room = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(room.path().join("data"), room.path().join("config"));
    let reaching = Reaching::of(&paths);
    let other = CloudAt {
        provider: "elsewhere".into(),
        account: "9999".into(),
        email: "otra@example.com".into(),
    };

    assert!(reaching.apart(&other).is_none());
    assert!(reaching.built.lock().unwrap().is_none());
}
