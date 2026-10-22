use tisty_carrier::{Hitch, Renew};

use super::{DavAt, Password};

#[test]
fn the_folder_named_holds_tisty_s_own_and_the_account_names_both_and_the_user() {
    let at = DavAt::under("https://cloud.example/remote.php/dav/files/mario", "mario").unwrap();

    assert_eq!(
        at.folder().as_str(),
        "https://cloud.example/remote.php/dav/files/mario/Tisty/"
    );
    assert_eq!(
        at.account(),
        "https://mario@cloud.example/remote.php/dav/files/mario/Tisty/"
    );
    assert_eq!(at.shown(), "mario@cloud.example");
}

#[test]
fn an_account_reads_back_as_the_same_folder_and_user() {
    let at = DavAt::under("https://cloud.example/dav/", "mario núñez@home").unwrap();

    let back = DavAt::of_account(&at.account()).unwrap();

    assert_eq!(back, at);
    assert_eq!(back.user(), "mario núñez@home");
}

#[test]
fn a_password_only_crosses_an_encrypted_line_unless_the_server_is_this_machine() {
    assert!(matches!(
        DavAt::under("http://cloud.example/dav/", "mario"),
        Err(Hitch::Broke(_))
    ));
    assert!(DavAt::under("http://127.0.0.1:8080/", "mario").is_ok());
    assert!(DavAt::under("http://localhost:8080/dav", "mario").is_ok());
    assert!(DavAt::under("http://[::1]:8080/", "mario").is_ok());
}

#[test]
fn an_address_that_carries_a_user_a_query_or_no_user_is_refused() {
    for (given, user) in [
        ("https://mario:r0ck4me!@cloud.example/dav/", "mario"),
        ("https://cloud.example/dav/?x=1", "mario"),
        ("https://cloud.example/dav/#top", "mario"),
        ("ftp://cloud.example/dav/", "mario"),
        ("not an address", "mario"),
        ("https://cloud.example/dav/", "  "),
    ] {
        assert!(DavAt::under(given, user).is_err(), "{given} {user}");
    }
}

#[test]
fn the_password_kept_is_the_credential_and_never_turns_into_another() {
    let renewed = Password.renew("r0ck4me!").unwrap();

    assert_eq!(renewed.access, "r0ck4me!");
    assert!(renewed.refresh.is_none());
}
