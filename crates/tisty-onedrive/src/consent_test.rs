use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use super::*;

const CLIENT: Client = Client { id: "client-1" };

fn said_in(consent: &Consent, key: &str) -> String {
    let query = consent.url.split_once('?').unwrap().1;
    url::form_urlencoded::parse(query.as_bytes())
        .find(|(one, _)| one == key)
        .unwrap()
        .1
        .into_owned()
}

fn token_answering(answer: &'static str) -> (Endpoints, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let token = format!(
        "http://127.0.0.1:{}/token",
        listener.local_addr().unwrap().port()
    );
    let heard = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut got = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let read = stream.read(&mut chunk).unwrap_or(0);
            got.extend_from_slice(&chunk[..read]);
            let text = String::from_utf8_lossy(&got).to_string();
            if let Some((head, sent)) = text.split_once("\r\n\r\n") {
                let wants = head
                    .lines()
                    .find_map(|one| {
                        one.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|n| n.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if sent.len() >= wants {
                    write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{answer}",
                        answer.len()
                    )
                    .unwrap();
                    return text;
                }
            }
            if read == 0 {
                return text;
            }
        }
    });
    (Endpoints { token }, heard)
}

fn lands_while(consent: &Consent, query: &str) -> std::thread::JoinHandle<String> {
    let (port, query) = (port_of(consent), query.to_string());
    std::thread::spawn(move || landed(port, &query))
}

fn port_of(consent: &Consent) -> u16 {
    said_in(consent, "redirect_uri")
        .rsplit(':')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

fn landed(port: u16, query: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(stream, "GET /?{query} HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();
    let mut page = String::new();
    stream.read_to_string(&mut page).unwrap();
    page
}

#[test]
fn the_page_asks_microsoft_for_the_app_folder_with_a_challenge_and_localhost() {
    let consent = Consent::begin(CLIENT).unwrap();

    assert!(consent.url.starts_with(CONSENT_URL));
    assert_eq!(said_in(&consent, "client_id"), "client-1");
    assert_eq!(said_in(&consent, "scope"), SCOPES);
    assert_eq!(said_in(&consent, "code_challenge_method"), "S256");
    assert_eq!(
        said_in(&consent, "code_challenge"),
        challenge_of(&consent.verifier)
    );
    assert!(said_in(&consent, "redirect_uri").starts_with("http://localhost:"));
    assert!(!consent.url.contains("client_secret"));
}

#[test]
fn a_code_that_comes_back_is_traded_without_a_secret() {
    let (at, heard) = token_answering(
        r#"{"access_token":"a1","expires_in":3600,"refresh_token":"r1","scope":"https://graph.microsoft.com/Files.ReadWrite.AppFolder https://graph.microsoft.com/User.Read"}"#,
    );
    let consent = Consent::begin(CLIENT).unwrap();
    let state = said_in(&consent, "state");
    let redirect = said_in(&consent, "redirect_uri");
    let query = format!("code=c1&state={state}");
    let page = lands_while(&consent, &query);
    let finishing = std::thread::spawn(move || consent.finish(Duration::from_secs(10), &at));
    let granted = finishing.join().unwrap().unwrap();
    let page = page.join().unwrap();
    assert_eq!(granted.refresh, "r1");
    assert!(page.contains(tisty_carrier::loopback::CONNECTED), "{page}");
    let asked = heard.join().unwrap();
    let body = asked.split_once("\r\n\r\n").unwrap().1;
    let said: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(body.as_bytes())
            .into_owned()
            .collect();
    assert_eq!(said["grant_type"], "authorization_code");
    assert_eq!(said["code"], "c1");
    assert_eq!(said["redirect_uri"], redirect);
    assert!(!said.contains_key("client_secret"));
}

#[test]
fn saying_no_on_microsofts_page_leaves_the_account_unconnected() {
    let consent = Consent::begin(CLIENT).unwrap();
    let state = said_in(&consent, "state");
    let query = format!("error=access_denied&state={state}");
    let page = std::thread::scope(|scope| {
        let port = port_of(&consent);
        let landing = scope.spawn(move || landed(port, &query));
        let ended = consent
            .loopback
            .answered(Duration::from_secs(10))
            .err()
            .unwrap();
        assert!(matches!(ended, Ended::Failed(Hitch::Lost)));
        landing.join().unwrap()
    });
    assert!(
        page.contains(tisty_carrier::loopback::NOT_CONNECTED),
        "{page}"
    );
}

#[test]
fn only_a_grant_of_the_app_folder_counts() {
    assert!(only_app_folder(Some(
        "https://graph.microsoft.com/Files.ReadWrite.AppFolder https://graph.microsoft.com/User.Read"
    )));
    assert!(only_app_folder(Some(
        "Files.ReadWrite.AppFolder User.Read profile openid email"
    )));
    assert!(!only_app_folder(Some("User.Read")));
    assert!(!only_app_folder(Some(
        "Files.ReadWrite.AppFolder Files.ReadWrite.All"
    )));
    assert!(!only_app_folder(None));
}
