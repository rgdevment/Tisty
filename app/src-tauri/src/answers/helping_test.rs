use super::*;

#[test]
fn every_page_names_its_twin_in_the_other_language() {
    let english = help_pages(false);
    let spanish = help_pages(true);
    assert_eq!(english.len(), spanish.len());
    for one in &english {
        assert!(
            spanish.iter().any(|other| other.slug == one.twin),
            "{} has no twin",
            one.slug
        );
    }
    assert_eq!(english[0].slug, "");
    assert!(help_page(true, "sincronizar-y-respaldar".into()).is_some());
    assert!(help_page(true, "nowhere".into()).is_none());
}

#[test]
fn pictures_are_missing_until_kept_and_each_one_is_counted_once() {
    let cache = tempfile::tempdir().unwrap();
    let at = folder(cache.path());
    let before = standing(&at, true);
    assert!(before.weighs > 0);
    assert_eq!(before.missing, before.weighs);
    assert!(before.kept.is_empty());

    let first = tisty_help::pictures(true).remove(0);
    std::fs::create_dir_all(&at).unwrap();
    std::fs::write(kept_at(&at, &first), b"png").unwrap();
    let after = standing(&at, true);
    assert_eq!(after.kept.len(), 1);
    assert_eq!(
        after.missing,
        before.weighs - tisty_help::weight(&first).unwrap()
    );
}

fn serving_one(answer: Vec<u8>) -> String {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let at = format!(
        "http://127.0.0.1:{}/pic.png",
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
    crate::bringing::client(std::time::Duration::from_secs(10)).unwrap()
}

fn png_answer(said: usize, body: &[u8], kind: &str) -> Vec<u8> {
    [
        format!("HTTP/1.1 200 OK\r\nContent-Type: {kind}\r\nContent-Length: {said}\r\nConnection: close\r\n\r\n")
            .into_bytes(),
        body.to_vec(),
    ]
    .concat()
}

#[test]
fn a_real_picture_is_written_to_its_place() {
    let room = tempfile::tempdir().unwrap();
    let to = room.path().join("pic.png");
    let body = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    let at = serving_one(png_answer(body.len(), &body, "image/png"));

    brought(&asking(), &at, &to).unwrap();

    assert_eq!(std::fs::read(&to).unwrap(), body);
}

#[test]
fn a_page_served_as_a_picture_is_refused_and_nothing_is_written() {
    let room = tempfile::tempdir().unwrap();
    let to = room.path().join("pic.png");
    let body = b"<!doctype html><html>sign in</html>";
    let at = serving_one(png_answer(body.len(), body, "image/png"));

    let why = brought(&asking(), &at, &to).unwrap_err();

    assert_eq!(why, "not a picture");
    assert!(!to.exists());
}

#[test]
fn a_picture_cut_off_mid_way_leaves_no_file() {
    let room = tempfile::tempdir().unwrap();
    let to = room.path().join("pic.png");
    let at = serving_one(png_answer(
        1000,
        b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR",
        "image/png",
    ));

    assert!(brought(&asking(), &at, &to).is_err());
    assert!(!to.exists());
}
