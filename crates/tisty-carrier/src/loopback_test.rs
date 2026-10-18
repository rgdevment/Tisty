use std::time::{Duration, Instant};

use super::*;

#[test]
fn the_challenge_is_the_one_rfc_7636_gives_for_its_own_verifier() {
    assert_eq!(
        challenge_of("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
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

#[test]
fn a_browser_that_reaches_localhost_over_ipv6_still_lands() {
    use std::io::Write;
    let open = Loopback::open("Microsoft", true).unwrap();
    let Ok(mut stream) = std::net::TcpStream::connect(("::1", open.port)) else {
        return;
    };
    write!(
        stream,
        "GET /?code=c1&state={} HTTP/1.1\r\nHost: localhost\r\n\r\n",
        open.state
    )
    .unwrap();

    let (code, _) = open.answered(Duration::from_secs(5)).unwrap();

    assert_eq!(code, "c1");
}
