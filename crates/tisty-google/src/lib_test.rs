use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use super::*;

const CLIENT: Client = Client {
    id: "id.apps.googleusercontent.com",
    secret: "not-a-secret",
};

struct Reply {
    status: &'static str,
    headers: &'static str,
    body: String,
}

fn reply(status: &'static str, body: &str) -> Reply {
    Reply {
        status,
        headers: "",
        body: body.to_string(),
    }
}

fn read_request(stream: &mut TcpStream) -> String {
    let mut asked = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let read = stream.read(&mut chunk).unwrap_or(0);
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
    String::from_utf8_lossy(&asked).into_owned()
}

fn provider(replies: Vec<Reply>) -> (Endpoints, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let served = std::thread::spawn(move || {
        replies
            .into_iter()
            .map(|one| {
                let (mut stream, _) = listener.accept().unwrap();
                let asked = read_request(&mut stream);
                write!(
                    stream,
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                    one.status,
                    one.headers,
                    one.body.len(),
                    one.body
                )
                .unwrap();
                asked
            })
            .collect()
    });
    let at = Endpoints {
        token: format!("{base}/token"),
        revoke: format!("{base}/revoke"),
    };
    (at, served)
}

fn nowhere() -> Endpoints {
    Endpoints {
        token: "http://127.0.0.1:9/token".into(),
        revoke: "http://127.0.0.1:9/revoke".into(),
    }
}

fn said_in(consent: &Consent, key: &str) -> String {
    let query = consent.url.split_once('?').unwrap().1;
    url::form_urlencoded::parse(query.as_bytes())
        .find(|(one, _)| one == key)
        .unwrap()
        .1
        .into_owned()
}

fn port_of(redirect: &str) -> u16 {
    redirect
        .trim_end_matches('/')
        .rsplit(':')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

fn lands(redirect: &str, query: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port_of(redirect))).unwrap();
    write!(stream, "GET /?{query} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").unwrap();
    let mut page = String::new();
    stream.read_to_string(&mut page).unwrap();
    page
}

fn waiting(consent: Consent, at: Endpoints) -> JoinHandle<Result<Granted, Hitch>> {
    std::thread::spawn(move || consent.finish(Duration::from_secs(10), &at))
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

    assert_eq!(said_in(&consent, "scope"), SCOPE);
    assert_eq!(said_in(&consent, "client_id"), CLIENT.id);
    assert_eq!(said_in(&consent, "code_challenge_method"), "S256");
    assert_eq!(said_in(&consent, "code_challenge").len(), 43);
    assert!(said_in(&consent, "redirect_uri").starts_with("http://127.0.0.1:"));
    assert!(
        !consent.url.contains(CLIENT.secret),
        "the secret travelled in the browser"
    );
}

#[test]
fn a_code_that_comes_back_is_traded_and_only_then_the_browser_hears_it_worked() {
    let (at, asked) = provider(vec![reply(
        "200 OK",
        &format!(
            r#"{{"access_token":"ya29.a","expires_in":3599,"refresh_token":"1//r","scope":"{SCOPE}"}}"#
        ),
    )]);
    let consent = Consent::begin(CLIENT).unwrap();
    let (redirect, state) = (
        said_in(&consent, "redirect_uri"),
        said_in(&consent, "state"),
    );
    let waiting = waiting(consent, at);

    let page = lands(&redirect, &format!("state={state}&code=4%2Fcode"));

    let granted = waiting.join().unwrap().unwrap();
    assert!(page.contains(CONNECTED), "{page}");
    assert_eq!(
        (granted.refresh.as_str(), granted.access.as_str()),
        ("1//r", "ya29.a")
    );
    let asked = &asked.join().unwrap()[0];
    assert!(asked.contains("code=4%2Fcode"), "{asked}");
    assert!(asked.contains("code_verifier="), "{asked}");
    assert!(
        asked.contains(&format!("client_id={}", CLIENT.id)),
        "{asked}"
    );
}

#[test]
fn a_trade_google_turns_down_is_never_told_to_the_browser_as_connected() {
    let (at, _) = provider(vec![reply(
        "400 Bad Request",
        r#"{"error":"invalid_grant"}"#,
    )]);
    let consent = Consent::begin(CLIENT).unwrap();
    let (redirect, state) = (
        said_in(&consent, "redirect_uri"),
        said_in(&consent, "state"),
    );
    let waiting = waiting(consent, at);

    let page = lands(&redirect, &format!("state={state}&code=4%2Fcode"));

    assert!(page.contains(NOT_CONNECTED), "{page}");
    assert!(matches!(waiting.join().unwrap(), Err(Hitch::Lost)));
}

#[test]
fn saying_no_on_googles_page_leaves_the_account_unconnected() {
    let consent = Consent::begin(CLIENT).unwrap();
    let (redirect, state) = (
        said_in(&consent, "redirect_uri"),
        said_in(&consent, "state"),
    );
    let waiting = waiting(consent, nowhere());

    let page = lands(&redirect, &format!("state={state}&error=access_denied"));

    assert!(page.contains(NOT_CONNECTED), "{page}");
    assert!(matches!(waiting.join().unwrap(), Err(Hitch::Lost)));
}

#[test]
fn a_browser_that_sends_many_headers_still_reads_the_whole_page() {
    let consent = Consent::begin(CLIENT).unwrap();
    let (redirect, state) = (
        said_in(&consent, "redirect_uri"),
        said_in(&consent, "state"),
    );
    let waiting = waiting(consent, nowhere());
    let headers: String = (0..80)
        .map(|at| format!("X-Header-{at}: {}\r\n", "v".repeat(80)))
        .collect();

    let mut stream = TcpStream::connect(("127.0.0.1", port_of(&redirect))).unwrap();
    write!(
        stream,
        "GET /?state={state}&error=access_denied HTTP/1.1\r\nHost: 127.0.0.1\r\n{headers}\r\n"
    )
    .unwrap();
    let mut page = String::new();
    let read = stream.read_to_string(&mut page);

    assert!(read.is_ok(), "the browser was reset: {read:?}");
    assert!(page.contains(NOT_CONNECTED), "{page}");
    assert!(matches!(waiting.join().unwrap(), Err(Hitch::Lost)));
}

#[test]
fn a_grant_wider_than_drive_file_is_refused_and_handed_back() {
    let (at, asked) = provider(vec![
        reply(
            "200 OK",
            r#"{"access_token":"ya29.a","expires_in":3599,"refresh_token":"1//wide","scope":"https://www.googleapis.com/auth/drive"}"#,
        ),
        reply("200 OK", ""),
    ]);
    let consent = Consent::begin(CLIENT).unwrap();
    let (redirect, state) = (
        said_in(&consent, "redirect_uri"),
        said_in(&consent, "state"),
    );
    let waiting = waiting(consent, at);

    let page = lands(&redirect, &format!("state={state}&code=4%2Fcode"));

    assert!(page.contains(NOT_CONNECTED), "{page}");
    assert!(matches!(waiting.join().unwrap(), Err(Hitch::Broke(_))));
    let asked = asked.join().unwrap();
    assert!(asked[1].starts_with("POST /revoke"), "{}", asked[1]);
    assert!(asked[1].contains("token=1%2F%2Fwide"), "{}", asked[1]);
}

#[test]
fn what_reaches_the_port_without_our_state_is_waited_past_and_never_traded() {
    let consent = Consent::begin(CLIENT).unwrap();
    let (redirect, state) = (
        said_in(&consent, "redirect_uri"),
        said_in(&consent, "state"),
    );
    let waiting = waiting(consent, nowhere());

    let other = lands(&redirect, "state=someone-else&code=stolen");
    let bare = lands(&redirect, "code=stolen");
    let refusal = lands(&redirect, "error=access_denied");
    lands(
        &redirect,
        &format!("state={state}&error=temporarily_unavailable"),
    );

    for page in [other, bare, refusal] {
        assert!(page.starts_with("HTTP/1.1 404"), "{page}");
    }
    assert!(
        matches!(waiting.join().unwrap(), Err(Hitch::Broke(_))),
        "an answer without our state ended the sign-in"
    );
}

#[test]
fn a_connection_that_says_nothing_does_not_stand_in_the_way_of_the_answer() {
    let consent = Consent::begin(CLIENT).unwrap();
    let (redirect, state) = (
        said_in(&consent, "redirect_uri"),
        said_in(&consent, "state"),
    );
    let waiting = waiting(consent, nowhere());

    let _idle = TcpStream::connect(("127.0.0.1", port_of(&redirect))).unwrap();
    lands(
        &redirect,
        &format!("state={state}&error=temporarily_unavailable"),
    );

    assert!(matches!(waiting.join().unwrap(), Err(Hitch::Broke(_))));
}

#[test]
fn a_request_that_never_ends_is_cut_short_and_the_answer_still_lands() {
    let consent = Consent::begin(CLIENT).unwrap();
    let (redirect, state) = (
        said_in(&consent, "redirect_uri"),
        said_in(&consent, "state"),
    );
    let waiting = waiting(consent, nowhere());

    let mut endless = TcpStream::connect(("127.0.0.1", port_of(&redirect))).unwrap();
    let _ = endless.write_all(&vec![b'a'; 64 * 1024]);
    lands(
        &redirect,
        &format!("state={state}&error=temporarily_unavailable"),
    );

    assert!(matches!(waiting.join().unwrap(), Err(Hitch::Broke(_))));
}

#[test]
fn a_sign_in_nobody_finishes_leaves_the_account_unconnected() {
    let consent = Consent::begin(CLIENT).unwrap();

    let given_up = consent.finish(Duration::from_millis(200), &nowhere());

    assert!(matches!(given_up, Err(Hitch::Lost)));
}

#[test]
fn waiting_for_as_long_as_it_takes_does_not_overflow() {
    let consent = Consent::begin(CLIENT).unwrap();
    let (redirect, state) = (
        said_in(&consent, "redirect_uri"),
        said_in(&consent, "state"),
    );
    let waiting = std::thread::spawn(move || consent.finish(Duration::MAX, &nowhere()));

    lands(
        &redirect,
        &format!("state={state}&error=temporarily_unavailable"),
    );

    assert!(matches!(waiting.join().unwrap(), Err(Hitch::Broke(_))));
}

#[test]
fn only_a_grant_of_drive_file_alone_counts() {
    assert!(only_drive_file(Some(SCOPE)));
    assert!(!only_drive_file(Some(
        "https://www.googleapis.com/auth/drive"
    )));
    assert!(!only_drive_file(Some(&format!(
        "{SCOPE} https://www.googleapis.com/auth/drive"
    ))));
    assert!(!only_drive_file(None));
    assert!(
        !only_drive_file(Some("")),
        "a grant that left drive.file unticked was taken as connected"
    );
}

#[test]
fn a_refresh_google_no_longer_honours_means_connecting_again() {
    let (at, asked) = provider(vec![reply(
        "400 Bad Request",
        r#"{"error":"invalid_grant"}"#,
    )]);

    let renewed = GoogleRenew::new(CLIENT, at).unwrap().renew("1//old");

    assert!(matches!(renewed, Err(Hitch::Lost)));
    let asked = &asked.join().unwrap()[0];
    assert!(asked.contains("grant_type=refresh_token"));
    assert!(asked.contains("refresh_token=1%2F%2Fold"));
}

#[test]
fn a_client_google_does_not_allow_is_a_build_to_fix_not_an_account_to_reconnect() {
    let (at, _) = provider(vec![reply(
        "401 Unauthorized",
        r#"{"error":"unauthorized_client"}"#,
    )]);

    let renewed = GoogleRenew::new(CLIENT, at).unwrap().renew("1//old");

    assert!(matches!(renewed, Err(Hitch::Broke(_))));
}

#[test]
fn a_renewal_brings_a_fresh_access_and_a_rotated_refresh_when_google_gives_one() {
    let (at, _) = provider(vec![reply(
        "200 OK",
        r#"{"access_token":"ya29.b","expires_in":3599,"refresh_token":"1//new"}"#,
    )]);

    let renewed = GoogleRenew::new(CLIENT, at)
        .unwrap()
        .renew("1//old")
        .unwrap();

    assert_eq!(renewed.access, "ya29.b");
    assert_eq!(renewed.lasts, Duration::from_secs(3599));
    assert_eq!(renewed.refresh.as_deref(), Some("1//new"));
}

#[test]
fn being_told_to_wait_waits_as_long_as_google_says_and_never_less_than_a_second() {
    let (at, _) = provider(vec![
        Reply {
            status: "429 Too Many Requests",
            headers: "Retry-After: 7\r\n",
            body: String::new(),
        },
        Reply {
            status: "503 Service Unavailable",
            headers: "Retry-After: 120\r\n",
            body: String::new(),
        },
        Reply {
            status: "429 Too Many Requests",
            headers: "Retry-After: 0\r\n",
            body: String::new(),
        },
    ]);
    let renew = GoogleRenew::new(CLIENT, at).unwrap();

    let waits: Vec<_> = (0..3)
        .map(|_| match renew.renew("1//old") {
            Err(Hitch::Limited { wait }) => wait.as_secs(),
            other => panic!("{:?}", other.err()),
        })
        .collect();

    assert_eq!(waits, [7, 120, 1]);
}

#[test]
fn a_build_without_a_client_id_and_its_secret_offers_no_cloud() {
    assert_eq!(client_of(None, None), None);
    assert_eq!(client_of(Some("id"), None), None);
    assert_eq!(client_of(None, Some("secret")), None);
    assert_eq!(client_of(Some(""), Some("secret")), None);
    assert_eq!(client_of(Some("id"), Some("")), None);
    assert_eq!(
        client_of(Some("id"), Some("secret")),
        Some(Client {
            id: "id",
            secret: "secret"
        })
    );
    if option_env!("TISTY_GOOGLE_CLIENT_ID").is_none() {
        assert_eq!(client(), None);
    }
}

#[test]
fn every_sign_in_asks_with_a_state_and_a_verifier_of_its_own() {
    let (one, two) = (
        Consent::begin(CLIENT).unwrap(),
        Consent::begin(CLIENT).unwrap(),
    );

    assert_ne!(said_in(&one, "state"), said_in(&two, "state"));
    assert_ne!(one.verifier, two.verifier);
    assert!(said_in(&one, "state").len() >= 32);
    assert!(
        one.verifier.len() >= 43,
        "RFC 7636 asks at least 43 characters"
    );
    assert_eq!(said_in(&one, "code_challenge"), challenge_of(&one.verifier));
}

#[test]
fn what_went_wrong_keeps_every_cause_under_it() {
    #[derive(Debug)]
    struct Under;
    impl std::fmt::Display for Under {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("the certificate is not trusted")
        }
    }
    impl std::error::Error for Under {}
    #[derive(Debug)]
    struct Top(Under);
    impl std::fmt::Display for Top {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("error sending request")
        }
    }
    impl std::error::Error for Top {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.0)
        }
    }

    assert_eq!(
        told(&Top(Under)),
        "error sending request: the certificate is not trusted"
    );
}

fn read_with(request: &str, mut clock: impl FnMut() -> Instant) -> (Option<String>, usize) {
    let mut asked = 0;
    let mut patience = |_: Duration| {
        asked += 1;
        true
    };
    let query = query_of(request.as_bytes(), &mut clock, &mut patience);
    (query, asked)
}

#[test]
fn the_request_is_read_through_its_headers_and_stops_at_the_blank_line() {
    let still = Instant::now();
    let request = "GET /?state=s&code=c HTTP/1.1\r\nHost: 127.0.0.1\r\nAccept: */*\r\nUser-Agent: x\r\n\r\nleft=over";

    let (query, asked) = read_with(request, || still);

    assert_eq!(query.as_deref(), Some("state=s&code=c"));
    assert_eq!(
        asked, 5,
        "the first line, three headers and the blank line, nothing past it"
    );
}

#[test]
fn the_headers_stop_being_read_once_their_time_is_up() {
    let start = Instant::now();
    let mut ticks = 0;
    let clock = move || {
        ticks += 1;
        if ticks <= 2 {
            start
        } else {
            start + HEARD_WITHIN + Duration::from_secs(1)
        }
    };

    let (query, asked) = read_with(
        "GET /?state=s HTTP/1.1\r\nHost: a\r\nAccept: b\r\nUser-Agent: c\r\n\r\n",
        clock,
    );

    assert_eq!(query.as_deref(), Some("state=s"));
    assert_eq!(asked, 2, "headers went on being read after the time was up");
}

#[test]
fn a_long_answer_within_the_cap_is_read_whole_and_one_past_it_is_cut() {
    let still = Instant::now();
    let long = "a".repeat(4000);

    let (query, _) = read_with(&format!("GET /?state={long} HTTP/1.1\r\n\r\n"), || still);
    let (past, _) = read_with(
        &format!("GET /?state={} HTTP/1.1\r\n\r\n", "a".repeat(20_000)),
        || still,
    );

    assert_eq!(query, Some(format!("state={long}")));
    assert_ne!(
        past.map(|one| one.len()),
        Some(20_006),
        "a request past 16 KiB was read whole"
    );
}

#[test]
fn a_socket_dropped_before_it_was_taken_never_ends_the_sign_in_but_a_real_fault_does() {
    use std::io::ErrorKind;
    for kind in [
        ErrorKind::ConnectionAborted,
        ErrorKind::ConnectionReset,
        ErrorKind::Interrupted,
    ] {
        assert!(passing(kind), "{kind:?}");
    }
    for kind in [
        ErrorKind::WouldBlock,
        ErrorKind::PermissionDenied,
        ErrorKind::AddrInUse,
    ] {
        assert!(!passing(kind), "{kind:?}");
    }
}
