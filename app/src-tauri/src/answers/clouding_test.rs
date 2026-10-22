use super::*;

fn ana() -> Sync {
    Sync::cloud(CloudAt {
        provider: GOOGLE.into(),
        account: "0123".into(),
        email: "ana@example.com".into(),
    })
}

fn config_with(sync: Option<Sync>) -> tisty_core::Config {
    let home = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(home.path().join("data"), home.path().join("config"));
    let mut config = tisty_core::Config::load_or_init(&paths).unwrap();
    config.sync = sync;
    config
}

#[test]
fn only_a_copy_that_shares_with_nobody_starts_on_the_cloud_here() {
    for alone in [None, Some(Sync::alone())] {
        let mut config = config_with(alone);
        assert!(choosing_cloud(ana(), true)(&mut config).is_none());
        assert_eq!(config.sync, Some(ana()));
    }

    let mut again = config_with(Some(ana()));
    assert!(choosing_cloud(ana(), true)(&mut again).is_none());

    let mut folder = config_with(Some(Sync::folder("G:/compartida")));
    let refused = choosing_cloud(ana(), true)(&mut folder);
    assert_eq!(refused.map(|one| one.code), Some("cloudNeedsMove"));
    assert_eq!(folder.sync, Some(Sync::folder("G:/compartida")));

    let other = Sync::cloud(CloudAt {
        provider: GOOGLE.into(),
        account: "9999".into(),
        email: "otra@example.com".into(),
    });
    let mut moved = config_with(Some(other.clone()));
    let refused = choosing_cloud(ana(), true)(&mut moved);
    assert_eq!(refused.map(|one| one.code), Some("cloudNeedsMove"));
    assert_eq!(moved.sync, Some(other.clone()));

    let mut never_joined = config_with(Some(other));
    assert!(choosing_cloud(ana(), false)(&mut never_joined).is_none());
    assert_eq!(never_joined.sync, Some(ana()));
}

#[test]
fn what_google_answers_reads_as_what_the_window_can_say() {
    assert_eq!(refused(Hitch::Lost, "Google Drive").code, "cloudLost");
    assert_eq!(refused(Hitch::Full, "Google Drive").code, "cloudFull");
    let elsewhere = refused(
        Hitch::Elsewhere {
            found: "otra@example.com".into(),
        },
        "Google Drive",
    );
    assert_eq!(elsewhere.code, "cloudElsewhere");
    assert_eq!(elsewhere.name.as_deref(), Some("otra@example.com"));
    assert_eq!(
        refused(
            Hitch::Unreachable(Some(tisty_carrier::Cause::Blocked), "sin red".into()),
            "Google Drive"
        )
        .code,
        "cloudNotNow"
    );
    let broke = refused(
        Hitch::Broke("Google Drive refused: 400".into()),
        "Google Drive",
    );
    assert_eq!(broke.code, "syncFailed");
    assert_eq!(
        broke.name, None,
        "a provider's English reached a window that may speak Spanish"
    );
}

#[test]
fn a_grant_waiting_for_its_account_is_held_only_in_memory() {
    let unfiled = Unfiled(Mutex::new(Some("1//refresh".into())));

    assert_eq!(
        unfiled.read(KEPT_AS).unwrap().as_deref(),
        Some("1//refresh")
    );
    unfiled.write(KEPT_AS, "1//rotated").unwrap();
    assert_eq!(
        unfiled.read(KEPT_AS).unwrap().as_deref(),
        Some("1//rotated")
    );
    unfiled.forget(KEPT_AS).unwrap();
    assert_eq!(unfiled.read(KEPT_AS).unwrap(), None);
}

#[test]
fn the_same_account_under_a_new_address_is_kept_and_the_address_written_down() {
    let renamed = Sync::cloud(CloudAt {
        provider: GOOGLE.into(),
        account: "0123".into(),
        email: "ana.nueva@example.com".into(),
    });
    let mut config = config_with(Some(ana()));

    assert!(choosing_cloud(renamed.clone(), true)(&mut config).is_none());
    assert_eq!(config.sync, Some(renamed));
}

#[test]
fn the_status_of_a_cloud_says_the_account_the_day_s_spend_the_switch_and_the_last_round() {
    let home = tempfile::tempdir().unwrap();
    let at = CloudAt {
        provider: GOOGLE.into(),
        account: "0123".into(),
        email: "ana@example.com".into(),
    };
    tisty_carrier::policy::pause_here(home.path(), true).unwrap();
    tisty_carrier::round_noted(
        home.path(),
        &tisty_carrier::LastRound {
            at: 1,
            code: "cloudFull".into(),
            name: Some("Google Drive".into()),
            left: 0,
            names: Vec::new(),
            ..Default::default()
        },
    );

    let status = status_of(&at, home.path(), false);

    assert_eq!(status.account, "0123");
    assert_eq!(status.email, "ana@example.com");
    assert!(!status.joined);
    assert!(status.paused_here);
    assert_eq!(status.units_spent, 0);
    assert_eq!(
        status.last.map(|one| one.code),
        Some("cloudFull".to_string())
    );
}

#[test]
fn a_cloud_is_lost_for_good_only_when_this_build_can_never_reach_it_again() {
    assert!(
        lost_for_good(false, false, None),
        "a build made without Google"
    );
    assert!(lost_for_good(true, true, None), "an access let go of");
    assert!(
        lost_for_good(true, false, Some("cloudLost")),
        "an access withdrawn"
    );
    assert!(!lost_for_good(true, false, Some("cloudFull")));
    assert!(!lost_for_good(true, false, Some("deferred-offline")));
    assert!(!lost_for_good(true, false, None));
}

#[test]
fn a_keychain_that_holds_no_token_is_an_access_gone() {
    let home = tempfile::tempdir().unwrap();
    let at = drive_of("sin-token");

    assert!(access_gone(&at));
    assert!(lost_for_good_at(home.path(), &at));
}

#[test]
fn a_token_kept_for_one_provider_is_never_another_s_by_the_same_account() {
    let google = drive_of("same-id");
    let other = tisty_core::config::CloudAt {
        provider: "elsewhere".into(),
        ..google.clone()
    };
    crate::reaching::secrets_for(&google)
        .unwrap()
        .write(crate::reaching::KEPT_AS, "token")
        .unwrap();

    assert!(has_access(&google).is_ok());
    assert!(access_gone(&other));
    assert_eq!(
        has_access(&other).unwrap_err().code,
        "cloudLost",
        "a provider this build does not know is never reached with Google's token"
    );
    forgotten(&google).unwrap();
}

#[test]
fn a_provider_this_build_does_not_offer_is_lost_for_good() {
    let home = tempfile::tempdir().unwrap();
    let other = tisty_core::config::CloudAt {
        provider: "elsewhere".into(),
        ..drive_of("kept-elsewhere")
    };
    crate::reaching::secrets_for(&other)
        .unwrap()
        .write(crate::reaching::KEPT_AS, "token")
        .unwrap();

    assert!(lost_for_good_at(home.path(), &other));
    forgotten(&other).unwrap();
}

fn drive_of(account: &str) -> tisty_core::config::CloudAt {
    tisty_core::config::CloudAt {
        provider: crate::reaching::GOOGLE.into(),
        account: account.into(),
        email: "ana@example.com".into(),
    }
}

#[test]
fn a_sign_in_that_never_came_back_is_not_told_as_access_denied() {
    assert_eq!(
        ended(tisty_google::Ended::NoAnswer, "Google Drive").code,
        "cloudNoAnswer"
    );
    assert_eq!(
        ended(tisty_google::Ended::Dropped, "Google Drive").code,
        "cloudDropped"
    );
    assert_eq!(
        ended(tisty_google::Ended::Failed(Hitch::Lost), "Google Drive").code,
        "cloudDenied"
    );
}

#[test]
fn a_newer_sign_in_lets_go_of_the_one_still_waiting_and_only_its_own_end_clears_it() {
    let consenting = Consenting::default();
    let (older, newer) = (
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
    );

    consenting.waits_on(older.clone());
    consenting.waits_on(newer.clone());
    consenting.done_with(&older);

    assert!(
        older.load(Ordering::Relaxed),
        "two sign-ins raced for one account"
    );
    assert!(!newer.load(Ordering::Relaxed));
    consenting.let_go();
    assert!(
        newer.load(Ordering::Relaxed),
        "the older one's end cleared the newer one, and cancelling reached nothing"
    );
}

#[test]
fn a_sign_in_that_did_not_finish_is_told_by_how_it_ended() {
    assert_eq!(how_it_ended("cloudNoAnswer"), "no-answer");
    assert_eq!(how_it_ended("cloudDropped"), "dropped");
    assert_eq!(how_it_ended("cloudDenied"), "denied");
    assert_eq!(how_it_ended("cloudNotNow"), "unreachable");
    assert_eq!(how_it_ended("cloudElsewhere"), "refused");
}

// Answers each request in turn with what the script says, reading it whole first so nothing is cut while it is sent.
fn dav_server(
    script: Vec<(&'static str, String)>,
) -> (tisty_webdav::DavAt, std::thread::JoinHandle<()>) {
    use std::io::{BufRead, Read, Write};
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let base = format!(
        "http://127.0.0.1:{}/dav/mario",
        listener.local_addr().unwrap().port()
    );
    let served = std::thread::spawn(move || {
        for (status, body) in script {
            let (stream, _) = listener.accept().unwrap();
            let mut reading = std::io::BufReader::new(stream);
            let mut length = 0;
            loop {
                let mut line = String::new();
                reading.read_line(&mut line).unwrap();
                if line.trim().is_empty() {
                    break;
                }
                if let Some((name, value)) = line.split_once(':')
                    && name.eq_ignore_ascii_case("content-length")
                {
                    length = value.trim().parse().unwrap();
                }
            }
            let mut asked = vec![0u8; length];
            reading.read_exact(&mut asked).unwrap();
            write!(
                reading.get_mut(),
                "HTTP/1.1 {status}\r\nContent-Type: application/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        }
    });
    (tisty_webdav::DavAt::under(&base, "mario").unwrap(), served)
}

fn kept_note(etag: &str) -> String {
    format!(
        r#"<?xml version="1.0"?><d:multistatus xmlns:d="DAV:"><d:response><d:href>/dav/mario/Tisty/a.md</d:href><d:propstat><d:prop><d:resourcetype/><d:getcontentlength>4</d:getcontentlength><d:getetag>{etag}</d:getetag></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response></d:multistatus>"#
    )
}

#[test]
fn a_webdav_server_that_guards_writes_is_connected_and_its_password_kept() {
    let aside = tempfile::tempdir().unwrap();
    let (dav, served) = dav_server(vec![("207 Multi-Status", kept_note("\"e1\""))]);

    let connected = dav_connected(dav.clone(), "r0ck4me!".into(), aside.path()).unwrap();

    served.join().unwrap();
    assert_eq!(connected.provider, WEBDAV);
    assert_eq!(connected.account, dav.account());
    assert_eq!(connected.email, dav.shown());
    let at = tisty_core::config::CloudAt {
        provider: WEBDAV.into(),
        account: connected.account,
        email: connected.email,
    };
    assert!(has_access(&at).is_ok());
    forgotten(&at).unwrap();
}

#[test]
fn a_webdav_server_with_no_strong_etag_is_never_connected() {
    let aside = tempfile::tempdir().unwrap();
    let (dav, served) = dav_server(vec![("207 Multi-Status", kept_note("W/\"e1\""))]);

    let refused = dav_connected(dav.clone(), "r0ck4me!".into(), aside.path()).unwrap_err();

    served.join().unwrap();
    assert_eq!(refused.code, "davUnguarded");
    assert!(access_gone(&tisty_core::config::CloudAt {
        provider: WEBDAV.into(),
        account: dav.account(),
        email: dav.shown(),
    }));
}

#[test]
fn a_webdav_password_refused_is_said_as_such() {
    let aside = tempfile::tempdir().unwrap();
    let (dav, served) = dav_server(vec![
        ("401 Unauthorized", String::new()),
        ("401 Unauthorized", String::new()),
    ]);

    let refused = dav_connected(dav, "wrong".into(), aside.path()).unwrap_err();

    served.join().unwrap();
    assert_eq!(refused.code, "davDenied");
    assert_eq!(how_it_ended(refused.code), "denied");
}

#[test]
fn what_a_webdav_server_refuses_reads_as_what_the_window_can_say() {
    assert_eq!(dav_refused(Hitch::Lost).code, "davDenied");
    assert_eq!(dav_refused(Hitch::Broke("409".into())).code, "davRefused");
    assert_eq!(
        dav_refused(Hitch::Missing("Tisty".into())).code,
        "davRefused"
    );
    assert_eq!(
        dav_refused(Hitch::Changed("tisty-probe".into())).code,
        "davRefused"
    );
    assert_eq!(dav_refused(Hitch::Full).code, "cloudFull");
}
