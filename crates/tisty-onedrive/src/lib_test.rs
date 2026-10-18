use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread::JoinHandle;

use tisty_carrier::{Cause, Hitch, Renew};

use super::{Client, Endpoints, MicrosoftRenew, SCOPES, cause_of, client_of};

pub(crate) fn answering(
    status: &'static str,
    answer: &'static str,
) -> (Endpoints, JoinHandle<String>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let token = format!(
        "http://127.0.0.1:{}/consumers/oauth2/v2.0/token",
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
                        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{answer}",
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

const CLIENT: Client = Client { id: "client-1" };

#[test]
fn a_build_without_the_client_id_offers_no_onedrive() {
    assert_eq!(client_of(None), None);
    assert_eq!(client_of(Some("  ")), None);
    assert_eq!(client_of(Some("client-1")), Some(CLIENT));
}

#[test]
fn the_new_refresh_token_microsoft_hands_back_travels_back_to_be_kept() {
    let (at, heard) = answering(
        "200 OK",
        r#"{"access_token":"a2","expires_in":3599,"refresh_token":"r2","token_type":"Bearer"}"#,
    );
    let renew = MicrosoftRenew::new(CLIENT, at).unwrap();

    let renewed = renew.renew("r1").unwrap();

    assert_eq!(renewed.access, "a2");
    assert_eq!(renewed.refresh.as_deref(), Some("r2"));
    assert_eq!(renewed.lasts.as_secs(), 3599);
    let asked = heard.join().unwrap();
    assert!(asked.starts_with("POST /consumers/oauth2/v2.0/token"));
    let body = asked.split_once("\r\n\r\n").unwrap().1;
    let said: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(body.as_bytes())
            .into_owned()
            .collect();
    assert_eq!(said["grant_type"], "refresh_token");
    assert_eq!(said["refresh_token"], "r1");
    assert_eq!(said["client_id"], "client-1");
    assert_eq!(said["scope"], SCOPES);
    assert!(
        !said.contains_key("client_secret"),
        "a public client has no secret"
    );
}

#[test]
fn a_refresh_token_microsoft_no_longer_takes_means_signing_in_again() {
    let (at, _heard) = answering(
        "400 Bad Request",
        r#"{"error":"invalid_grant","error_description":"AADSTS70008: expired"}"#,
    );

    let refused = MicrosoftRenew::new(CLIENT, at).unwrap().renew("r1");

    assert_eq!(refused.err(), Some(Hitch::Lost));
}

#[test]
fn a_busy_sign_in_waits_as_microsoft_asks() {
    let (at, _heard) = answering("429 Too Many Requests", "{}");

    let refused = MicrosoftRenew::new(CLIENT, at).unwrap().renew("r1");

    assert!(matches!(refused, Err(Hitch::Limited { .. })));
}

#[test]
fn a_way_that_never_opened_says_what_stood_in_it() {
    assert_eq!(
        cause_of(false, "invalid peer certificate: UnknownIssuer"),
        Cause::Filtered
    );
    assert_eq!(cause_of(true, "operation timed out"), Cause::Slow);
    assert_eq!(cause_of(false, "connection refused"), Cause::Blocked);
}
