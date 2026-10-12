use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

use super::*;

const CLIENT: Client = Client {
    id: "id.apps.googleusercontent.com",
    secret: "not-a-secret",
};

/// One canned answer on a port of its own; hands back what it was asked.
fn provider(status: &str, body: &str) -> (String, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let url = format!(
        "http://127.0.0.1:{}/token",
        listener.local_addr().unwrap().port()
    );
    let reply = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let served = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut asked = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let read = stream.read(&mut chunk).unwrap();
            asked.extend_from_slice(&chunk[..read]);
            let text = String::from_utf8_lossy(&asked);
            if let Some(head) = text.find("\r\n\r\n") {
                let wants = text[..head]
                    .lines()
                    .find_map(|one| {
                        one.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|n| n.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if asked.len() >= head + 4 + wants {
                    break;
                }
            }
            if read == 0 {
                break;
            }
        }
        stream.write_all(reply.as_bytes()).unwrap();
        String::from_utf8_lossy(&asked).into_owned()
    });
    (url, served)
}

fn said_in(consent: &Consent, key: &str) -> String {
    let query = consent.url.split_once('?').unwrap().1;
    url::form_urlencoded::parse(query.as_bytes())
        .find(|(one, _)| one == key)
        .unwrap()
        .1
        .into_owned()
}

/// What the browser does after the person agrees: it lands on the loopback with the answer.
fn lands(redirect: &str, query: &str) -> String {
    let port: u16 = redirect
        .trim_end_matches('/')
        .rsplit(':')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(
        stream,
        "GET /?{query} HTTP/1.1
Host: 127.0.0.1

"
    )
    .unwrap();
    let mut page = String::new();
    stream.read_to_string(&mut page).unwrap();
    page
}

#[test]
fn the_challenge_is_the_one_rfc_7636_gives_for_its_own_verifier() {
    assert_eq!(
        challenge_of("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
}

#[test]
fn the_page_asks_only_for_the_files_tisty_creates_with_a_challenge_and_a_loopback() {
    let consent = Consent::begin(CLIENT).unwrap();
    let query = consent.url.split_once('?').unwrap().1;
    let said: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

    assert_eq!(said["scope"], SCOPE);
    assert_eq!(said["code_challenge_method"], "S256");
    assert_eq!(said["code_challenge"].len(), 43);
    assert_eq!(said["client_id"], CLIENT.id);
    assert!(said["redirect_uri"].starts_with("http://127.0.0.1:"));
    assert!(
        !consent.url.contains(CLIENT.secret),
        "the secret travelled in the browser"
    );
}

#[test]
fn a_code_that_comes_back_is_traded_for_lasting_access_with_its_verifier() {
    let (token_url, asked) = provider(
        "200 OK",
        &format!(
            r#"{{"access_token":"ya29.a","expires_in":3599,"refresh_token":"1//r","scope":"{SCOPE}"}}"#
        ),
    );
    let consent = Consent::begin(CLIENT).unwrap();
    let (redirect, state) = (
        said_in(&consent, "redirect_uri"),
        said_in(&consent, "state"),
    );
    let waiting =
        std::thread::spawn(move || consent.finish(CLIENT, Duration::from_secs(5), &token_url));

    let page = lands(&redirect, &format!("state={state}&code=4%2Fcode"));

    let granted = waiting.join().unwrap().unwrap();
    assert!(page.contains("connected"), "{page}");
    assert_eq!(
        (granted.refresh.as_str(), granted.access.as_str()),
        ("1//r", "ya29.a")
    );
    let asked = asked.join().unwrap();
    assert!(asked.contains("code=4%2Fcode"), "{asked}");
    assert!(asked.contains("code_verifier="), "{asked}");
    assert!(asked.contains("grant_type=authorization_code"), "{asked}");
}

#[test]
fn an_answer_for_another_request_is_refused_before_anything_is_traded() {
    let consent = Consent::begin(CLIENT).unwrap();
    let redirect = said_in(&consent, "redirect_uri");
    let waiting = std::thread::spawn(move || {
        consent.finish(CLIENT, Duration::from_secs(5), "http://127.0.0.1:9/")
    });

    let page = lands(&redirect, "state=someone-else&code=stolen");

    assert!(page.contains("could not connect"), "{page}");
    assert!(matches!(waiting.join().unwrap(), Err(Hitch::Broke(_))));
}

#[test]
fn turning_the_sign_in_down_leaves_the_account_unconnected() {
    let consent = Consent::begin(CLIENT).unwrap();
    let (redirect, state) = (
        said_in(&consent, "redirect_uri"),
        said_in(&consent, "state"),
    );
    let waiting = std::thread::spawn(move || {
        consent.finish(CLIENT, Duration::from_secs(5), "http://127.0.0.1:9/")
    });

    lands(&redirect, &format!("state={state}&error=access_denied"));

    assert!(matches!(waiting.join().unwrap(), Err(Hitch::Lost)));
}

#[test]
fn a_sign_in_nobody_finishes_gives_up_instead_of_waiting_forever() {
    let consent = Consent::begin(CLIENT).unwrap();

    let given_up = consent.finish(CLIENT, Duration::from_millis(200), "http://127.0.0.1:9/");

    assert!(matches!(given_up, Err(Hitch::Unreachable(_))));
}

#[test]
fn a_build_without_a_client_id_offers_no_cloud() {
    assert_eq!(client().is_some(), crate::cloud_offered());
    if option_env!("TISTY_GOOGLE_CLIENT_ID").is_none() {
        assert!(!crate::cloud_offered());
    }
}

#[test]
fn granted_scope_beyond_drive_file_is_turned_down() {
    assert!(only_drive_file(Some(SCOPE)));
    assert!(!only_drive_file(Some(
        "https://www.googleapis.com/auth/drive"
    )));
    assert!(!only_drive_file(Some(&format!(
        "{SCOPE} https://www.googleapis.com/auth/drive"
    ))));
    assert!(!only_drive_file(None));
}

#[test]
fn a_refresh_google_no_longer_honours_means_connecting_again() {
    let (token_url, asked) = provider("400 Bad Request", r#"{"error":"invalid_grant"}"#);

    let renewed = GoogleRenew::new(CLIENT, token_url).renew("1//old");

    assert!(matches!(renewed, Err(Hitch::Lost)));
    let asked = asked.join().unwrap();
    assert!(asked.contains("grant_type=refresh_token"));
    assert!(asked.contains("refresh_token=1%2F%2Fold"));
}

#[test]
fn a_renewal_brings_a_fresh_access_and_a_rotated_refresh_when_google_gives_one() {
    let (token_url, _) = provider(
        "200 OK",
        r#"{"access_token":"ya29.b","expires_in":3599,"refresh_token":"1//new"}"#,
    );

    let renewed = GoogleRenew::new(CLIENT, token_url).renew("1//old").unwrap();

    assert_eq!(renewed.access, "ya29.b");
    assert_eq!(renewed.lasts, Duration::from_secs(3599));
    assert_eq!(renewed.refresh.as_deref(), Some("1//new"));
}

#[test]
fn being_told_to_slow_down_waits_as_long_as_google_says() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let url = format!(
        "http://127.0.0.1:{}/token",
        listener.local_addr().unwrap().port()
    );
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut chunk = [0u8; 4096];
        let _ = stream.read(&mut chunk);
        let _ = stream.write_all(b"HTTP/1.1 429 Too Many Requests\r\nRetry-After: 7\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    });

    let renewed = GoogleRenew::new(CLIENT, url).renew("1//old");

    assert!(matches!(renewed, Err(Hitch::Limited { wait }) if wait == Duration::from_secs(7)));
}
