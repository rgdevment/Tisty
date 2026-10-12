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
    assert_eq!(keeping.carrier.is_some(), offered());
    assert_eq!(offered(), tisty_google::client().is_some());
}

#[test]
fn a_provider_this_build_does_not_know_is_never_built() {
    let home = tempfile::tempdir().unwrap();
    let at = CloudAt {
        provider: "onedrive".into(),
        ..ana()
    };

    assert!(cloud_of(home.path(), &at).is_err());
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

    heed(home.path());

    assert!(!home.path().join(POLICY_ASKED).exists());
}

#[test]
fn what_kept_a_cloud_from_being_built_is_said_and_letting_go_drops_nothing_built() {
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
    reaching.let_go();
    assert!(reaching.cloud(&elsewhere).is_none());
}
