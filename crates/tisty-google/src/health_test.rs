use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

use super::*;

const SOON: Duration = Duration::from_millis(500);

fn answering(status: &'static str) -> String {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let at = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut asked = [0u8; 1024];
            let _ = stream.read(&mut asked);
            let _ = write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
        }
    });
    format!("http://{at}/drive/v3/about")
}

#[test]
fn a_refusal_to_a_stranger_is_google_answering() {
    assert_eq!(
        probe_at(&answering("401 Unauthorized"), SOON, SOON),
        Health::Reachable
    );
    assert_eq!(
        probe_at(&answering("403 Forbidden"), SOON, SOON),
        Health::Reachable
    );
}

#[test]
fn google_failing_on_its_side_is_told_apart_from_a_blocked_way() {
    assert_eq!(
        probe_at(&answering("503 Service Unavailable"), SOON, SOON),
        Health::Troubled
    );
    assert_eq!(answered(500), Health::Troubled);
}

#[test]
fn a_way_that_opens_but_never_answers_is_slow_not_blocked_and_given_up_on_soon() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let at = listener.local_addr().unwrap();
    let started = std::time::Instant::now();

    let health = probe_at(&format!("http://{at}/drive/v3/about"), SOON, SOON);

    assert_eq!(health, Health::Slow);
    assert!(started.elapsed() < Duration::from_secs(5));
    drop(listener);
}

#[test]
fn a_door_shut_at_once_is_blocked() {
    let at = TcpListener::bind(("127.0.0.1", 0))
        .unwrap()
        .local_addr()
        .unwrap();

    // Windows answers a closed local port only after its own retries, about two seconds in.
    let patient = Duration::from_secs(8);
    assert_eq!(
        probe_at(&format!("http://{at}/drive/v3/about"), patient, patient),
        Health::Blocked
    );
}

#[test]
fn a_name_that_resolves_nowhere_is_blocked() {
    assert_eq!(
        failed("error sending request: dns error: failed to lookup address information"),
        Health::Blocked
    );
    assert_eq!(
        failed("tcp connect error: connection reset by peer (os error 104)"),
        Health::Blocked
    );
}

#[test]
fn a_way_that_took_too_long_is_slow_not_blocked() {
    assert_eq!(failed("operation timed out"), Health::Slow);
    assert_eq!(
        failed("error sending request: client error (Connect): connection timeout"),
        Health::Slow
    );
}

#[test]
fn a_certificate_nobody_here_trusts_is_a_filter_reading_secure_connections() {
    for said in [
        "The certificate chain was issued by an authority that is not trusted. (os error -2146762487)",
        "error:0A000086:SSL routines:tls_post_process_server_certificate:certificate verify failed",
        "The certificate for this server is invalid",
        "invalid peer certificate: UnknownIssuer",
        "self signed certificate in certificate chain",
    ] {
        assert_eq!(failed(said), Health::Inspected, "{said}");
    }
}

#[test]
fn every_answer_has_a_word_the_window_reads() {
    assert_eq!(Health::Reachable.code(), "reachable");
    assert_eq!(Health::Inspected.code(), "inspected");
    assert_eq!(Health::Blocked.code(), "blocked");
    assert_eq!(Health::Slow.code(), "slow");
    assert_eq!(Health::Troubled.code(), "troubled");
}

#[test]
fn only_googles_api_host_is_ever_asked() {
    assert!(PROBED.starts_with("https://www.googleapis.com/"));
    assert!(PROBED_WITHIN <= Duration::from_secs(10));
    assert!(REACHED_WITHIN < PROBED_WITHIN);
}
