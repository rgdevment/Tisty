use super::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

fn serving(answer: Vec<u8>) -> String {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let at = format!(
        "http://127.0.0.1:{}/file",
        listener.local_addr().unwrap().port()
    );
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut asked = [0u8; 2048];
            let _ = stream.read(&mut asked);
            let _ = stream.write_all(&answer);
        }
    });
    at
}

fn asking() -> reqwest::blocking::Client {
    client(Duration::from_secs(10)).unwrap()
}

fn with_length(status: &str, said: usize, body: &[u8]) -> Vec<u8> {
    [
        format!("HTTP/1.1 {status}\r\nContent-Length: {said}\r\nConnection: close\r\n\r\n")
            .into_bytes(),
        body.to_vec(),
    ]
    .concat()
}

#[test]
fn a_body_under_the_cap_comes_back_whole() {
    let at = serving(with_length("200 OK", 5, b"hello"));

    assert_eq!(brought(&asking(), &at, 1024).unwrap(), b"hello");
}

#[test]
fn a_body_of_exactly_the_cap_comes_back_whole() {
    let body = vec![b'x'; 1024];
    let at = serving(with_length("200 OK", 1024, &body));

    assert_eq!(brought(&asking(), &at, 1024).unwrap(), body);
}

#[test]
fn a_missing_file_is_refused() {
    let at = serving(with_length("404 Not Found", 0, b""));

    assert!(brought(&asking(), &at, 1024).is_err());
}

#[test]
fn a_file_that_says_it_is_larger_than_the_cap_is_refused_before_it_is_read() {
    let at = serving(with_length("200 OK", 5_000_000_000, &[b'x'; 100]));

    let why = brought(&asking(), &at, 1024).unwrap_err();

    assert!(why.contains("larger than 1024"), "{why}");
}

#[test]
fn a_body_sent_in_chunks_past_the_cap_is_refused() {
    let mut answer =
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n".to_vec();
    for _ in 0..64 {
        answer.extend_from_slice(b"400\r\n");
        answer.extend_from_slice(&[b'x'; 1024]);
        answer.extend_from_slice(b"\r\n");
    }
    answer.extend_from_slice(b"0\r\n\r\n");
    let at = serving(answer);

    let why = brought(&asking(), &at, 1024).unwrap_err();

    assert!(why.contains("larger than 1024"), "{why}");
}

#[test]
fn a_body_with_no_length_past_the_cap_is_refused() {
    let answer = [
        b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n".to_vec(),
        vec![b'x'; 64 * 1024],
    ]
    .concat();
    let at = serving(answer);

    let why = brought(&asking(), &at, 1024).unwrap_err();

    assert!(why.contains("larger than 1024"), "{why}");
}

#[test]
fn a_body_cut_off_mid_way_is_an_error_not_a_short_file() {
    let at = serving(with_length("200 OK", 1000, &[b'x'; 100]));

    assert!(brought(&asking(), &at, 1024).is_err());
}

#[test]
fn the_first_bytes_of_each_picture_kind_are_recognised() {
    assert!(is_picture(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"));
    assert!(is_picture(b"\xff\xd8\xff\xe0\0\x10JFIF"));
    assert!(is_picture(b"GIF87a\x01\0\x01\0"));
    assert!(is_picture(b"GIF89a\x01\0\x01\0"));
    assert!(is_picture(b"RIFF\x24\0\0\0WEBPVP8 "));
}

#[test]
fn text_and_other_formats_are_not_pictures() {
    assert!(!is_picture(b"<!doctype html><html></html>"));
    assert!(!is_picture(
        b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>"
    ));
    assert!(!is_picture(b""));
    assert!(!is_picture(b"RIFF\x24\0\0\0WAVEfmt "));
    assert!(!is_picture(b"RIFF\x24\0\0\0WEB"));
    assert!(!is_picture(b"RIFF"));
}

#[test]
fn a_redirect_is_not_followed_to_another_server() {
    let reached = Arc::new(AtomicBool::new(false));
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let elsewhere = format!(
        "http://127.0.0.1:{}/file",
        listener.local_addr().unwrap().port()
    );
    listener.set_nonblocking(true).unwrap();
    let watching = Arc::clone(&reached);
    let watcher = std::thread::spawn(move || {
        for _ in 0..100 {
            if let Ok((mut stream, _)) = listener.accept() {
                watching.store(true, Ordering::SeqCst);
                let _ = stream.write_all(&with_length("200 OK", 5, b"hello"));
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    });
    let at = serving(
        format!("HTTP/1.1 302 Found\r\nLocation: {elsewhere}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .into_bytes(),
    );

    assert!(brought(&asking(), &at, 1024).is_err());

    watcher.join().unwrap();
    assert!(!reached.load(Ordering::SeqCst));
}
