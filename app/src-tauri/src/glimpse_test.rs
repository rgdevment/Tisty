/// A page cut at the cap lands mid-character sooner or later, and slicing a string there is a
/// panic. What is read off the wire is bytes, so the cut is made where bytes may be cut.
#[test]
fn a_page_cut_in_the_middle_of_a_letter_is_still_read() {
    let mut raw = "a".repeat(super::READS - 1).into_bytes();
    raw.extend_from_slice("ñ".as_bytes());
    let said = String::from_utf8_lossy(&raw[..super::READS]).into_owned();

    assert_eq!(
        said.len(),
        super::READS + 2,
        "el trozo partido se reemplaza"
    );
    assert!(said.ends_with('\u{fffd}'), "y nada estalla al leerlo");
}

#[test]
fn a_cache_of_glimpses_stops_growing_by_weight_as_well_as_by_count() {
    let room = tempfile::tempdir().unwrap();
    let heavy = "x".repeat(600 * 1024);
    for n in 0..60 {
        keep(
            room.path(),
            &format!("https://ejemplo.com/{n}"),
            &super::Glimpse {
                title: Some(format!("uno {n}")),
                said: None,
                shot: Some(heavy.clone()),
            },
        );
    }

    let weighs: u64 = std::fs::read_dir(room.path())
        .unwrap()
        .filter_map(|one| one.ok()?.metadata().ok())
        .map(|one| one.len())
        .sum();

    assert!(
        weighs <= super::WEIGHS_AT_MOST,
        "sesenta caratulas grandes caben en menos de lo que pesan: {weighs}"
    );
}

#[test]
fn a_cache_of_glimpses_stops_growing_and_lets_the_oldest_go() {
    let room = tempfile::tempdir().unwrap();
    for n in 0..(KEEPS + 20) {
        keep(
            room.path(),
            &format!("https://ejemplo.org/{n}"),
            &Glimpse {
                title: Some(format!("uno {n}")),
                said: None,
                shot: None,
            },
        );
    }

    let held = std::fs::read_dir(room.path().join("glimpses"))
        .unwrap()
        .count();

    assert!(held <= KEEPS + 1, "quedaron {held}");
    assert!(kept(room.path(), &format!("https://ejemplo.org/{}", KEEPS + 19)).is_some());
}

use super::*;

#[test]
fn a_page_says_its_name_through_open_graph_before_its_title_tag() {
    let head = r#"<title>Lo de siempre</title><meta property="og:title" content="Lo que quiere que veas">"#;

    assert_eq!(
        named(head, "og:title").as_deref(),
        Some("Lo que quiere que veas")
    );
    assert_eq!(titled(head).as_deref(), Some("Lo de siempre"));
}

#[test]
fn a_meta_that_names_something_else_is_not_taken_for_this_one() {
    let head =
        r#"<meta name="author" content="Alguien"><meta name="description" content="De esto va">"#;

    assert_eq!(named(head, "description").as_deref(), Some("De esto va"));
    assert_eq!(named(head, "og:title"), None);
}

#[test]
fn what_a_page_writes_in_its_head_comes_back_as_plain_words() {
    let head = r#"<meta property="og:title" content="Uno &amp; otro
   con   aire">"#;

    assert_eq!(
        named(head, "og:title").as_deref(),
        Some("Uno & otro con aire")
    );
}

#[test]
fn a_picture_named_by_its_path_is_asked_for_at_the_same_host() {
    assert_eq!(
        whole("https://ejemplo.org/uno/dos", "/cover.png"),
        "https://ejemplo.org/cover.png"
    );
    assert_eq!(
        whole("https://ejemplo.org/uno", "https://otro.example/x.png"),
        "https://otro.example/x.png"
    );
}

#[test]
fn nothing_of_the_machine_is_ever_asked_for() {
    assert!(!worldly("file:///etc/passwd"));
    assert!(!worldly("/etc/passwd"));
    assert!(!worldly("tisty:doc/mac0-0001"));
    assert!(worldly("https://ejemplo.org"));
}

#[test]
fn bytes_travel_as_the_letters_that_stand_for_them() {
    assert_eq!(encoded(b"Ma"), "TWE=");
    assert_eq!(encoded(b"Man"), "TWFu");
    assert_eq!(encoded(b"M"), "TQ==");
    assert_eq!(encoded(b""), "");
}

#[test]
fn a_glimpse_written_down_is_read_back_the_same() {
    let room = tempfile::tempdir().unwrap();
    let one = Glimpse {
        title: Some("Tisty".into()),
        said: Some("Notas y tareas".into()),
        shot: None,
    };

    keep(room.path(), "https://ejemplo.org", &one);

    assert_eq!(kept(room.path(), "https://ejemplo.org"), Some(one));
    assert_eq!(kept(room.path(), "https://otra.example"), None);
}
