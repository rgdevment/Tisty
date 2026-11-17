use super::{
    Card, PIECE_ABOUT, body_of, cards_of, drawn_from, far_kind, far_picture, near_picture,
    neighbour, reaches_outside,
};

fn kept(
    state: &mut tisty_core::State,
    file: &str,
    title: &str,
    page_of: Option<ulid::Ulid>,
    archived: bool,
) -> ulid::Ulid {
    let id = ulid::Ulid::generate();
    let one: tisty_core::model::Kept = serde_json::from_value(serde_json::json!({
        "id": id.to_string(), "file": file, "order": file, "title": title,
        "page_of": page_of.map(|of| of.to_string()), "archived": archived,
    }))
    .unwrap();
    state.docs.insert(id, one);
    id
}

#[test]
fn a_documents_cards_say_what_each_names_as_the_main_window_does() {
    let mut state = tisty_core::State::default();
    let here = kept(&mut state, "here", "Aquí", None, false);
    kept(&mut state, "p1", "Primera", Some(here), false);
    kept(&mut state, "p2", "Segunda", Some(here), false);
    kept(&mut state, "other", "Otra", None, false);
    kept(&mut state, "old", "Vieja", None, true);
    let files = ["p2", "p1", "other", "old", "here", "missing"].map(String::from);
    let cards = cards_of(
        &state,
        "here",
        "![B](tisty:doc/p2)\n\n![A](tisty:doc/p1)\n",
        files.to_vec(),
    );
    assert_eq!(
        cards["p2"],
        Card::Page {
            title: "Segunda".into(),
            at: 1,
            of: 2
        }
    );
    assert_eq!(
        cards["p1"],
        Card::Page {
            title: "Primera".into(),
            at: 2,
            of: 2
        }
    );
    assert_eq!(
        cards["other"],
        Card::Doc {
            title: "Otra".into()
        }
    );
    assert_eq!(
        cards["old"],
        Card::Away {
            title: "Vieja".into()
        }
    );
    assert_eq!(
        cards["here"],
        Card::Itself {
            title: "Aquí".into()
        }
    );
    assert_eq!(cards["missing"], Card::Gone);
}

fn file_of_size(room: &tempfile::TempDir, bytes: usize) -> std::path::PathBuf {
    let at = room.path().join("long.md");
    std::fs::write(&at, "x".repeat(bytes)).unwrap();
    at
}

#[test]
fn reading_takes_a_file_a_document_could_not_hold() {
    let room = tempfile::tempdir().unwrap();
    let at = file_of_size(&room, tisty_core::docs::BODY_AT_MOST as usize + 1);
    assert!(tisty_core::docs::read_outside(&at).is_err());
    assert!(body_of(&at).is_ok());
}

#[test]
fn reading_refuses_past_its_own_ceiling_and_names_it() {
    let room = tempfile::tempdir().unwrap();
    let at = file_of_size(&room, tisty_core::docs::READ_AT_MOST as usize + 1);
    let refused = body_of(&at).unwrap_err();
    assert_eq!(refused.code, "documentTooBig");
}

#[test]
fn reading_does_not_change_a_byte_of_the_file() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("kept.md");
    let written = "# Title\r\n\r\ncut at\r\neighty columns #ff0000 [[x]]\r\n";
    std::fs::write(&at, written).unwrap();
    let before = std::fs::metadata(&at).unwrap().modified().unwrap();
    let body = body_of(&at).unwrap();
    drawn_from(&body, false);
    assert_eq!(std::fs::read_to_string(&at).unwrap(), written);
    assert_eq!(std::fs::metadata(&at).unwrap().modified().unwrap(), before);
}

#[test]
fn the_pieces_add_up_to_what_one_drawing_says() {
    let body: String = (0..6000)
        .map(|at| format!("## Heading {at}\n\nA paragraph that takes its room, number {at}.\n\n"))
        .collect();
    let (pieces, heads) = drawn_from(&body, false);
    assert!(pieces.len() > 1);
    assert_eq!(heads.len(), 6000);
    assert_eq!(
        pieces.concat(),
        tisty_markdown::html(&tisty_markdown::read(&body))
    );
    let last = pieces.len() - 1;
    assert!(pieces[..last].iter().all(|one| one.len() >= PIECE_ABOUT));
}

#[test]
fn the_index_names_what_the_page_anchors() {
    let (pieces, heads) = drawn_from("# One\n\n## Two\n", false);
    assert_eq!(heads.len(), 2);
    assert_eq!((heads[1].level, heads[1].said.as_str()), (2, "Two"));
    assert!(pieces[0].contains(&format!("id=\"{}\"", heads[1].id)));
}

#[test]
fn a_document_of_ones_own_is_read_in_the_tisty_language_and_a_strangers_file_is_not() {
    let body = "# Notas\n\nUna línea que un editor cortó al llegar a las ochenta columnas de ancho\ny siguió aquí.\n";
    let (own, _) = drawn_from(body, true);
    let (outside, _) = drawn_from(body, false);
    assert_eq!(
        own.concat(),
        tisty_markdown::html(&tisty_markdown::read(body))
    );
    assert_eq!(
        outside.concat(),
        tisty_markdown::html(&tisty_markdown::read_file(body))
    );
    assert_ne!(own.concat(), outside.concat());
}

#[test]
fn an_empty_file_is_no_pieces_at_all() {
    let (pieces, heads) = drawn_from("", false);
    assert!(pieces.is_empty());
    assert!(heads.is_empty());
}

fn folder_with_a_file() -> (tempfile::TempDir, std::path::PathBuf) {
    let room = tempfile::tempdir().unwrap();
    let inside = room.path().join("project");
    std::fs::create_dir_all(inside.join("img")).unwrap();
    let file = inside.join("readme.md");
    std::fs::write(&file, "# hi").unwrap();
    std::fs::write(inside.join("img").join("a b.png"), [1, 2, 3]).unwrap();
    std::fs::write(room.path().join("secret.png"), [9]).unwrap();
    std::fs::write(inside.join("notes.txt"), "text").unwrap();
    (room, file)
}

#[test]
fn a_picture_beside_the_file_arrives_as_data() {
    let (_room, file) = folder_with_a_file();
    let said = near_picture(&file, "./img/a%20b.png").unwrap();
    assert_eq!(said, "data:image/png;base64,AQID");
}

#[test]
fn a_picture_is_never_read_from_above_the_files_folder() {
    let (room, file) = folder_with_a_file();
    assert!(near_picture(&file, "../secret.png").is_err());
    let whole = room.path().join("secret.png");
    assert!(near_picture(&file, &whole.display().to_string()).is_err());
    assert!(near_picture(&file, "img/../../secret.png").is_err());
}

#[test]
fn only_a_picture_is_read_as_one() {
    let (_room, file) = folder_with_a_file();
    assert!(near_picture(&file, "notes.txt").is_err());
    assert!(near_picture(&file, "readme.md").is_err());
    assert!(near_picture(&file, "missing.png").is_err());
}

#[test]
fn what_the_web_answers_counts_only_if_it_says_it_is_a_picture() {
    assert_eq!(
        far_kind(Some("IMAGE/png; x=1")).as_deref(),
        Some("image/png")
    );
    assert_eq!(far_kind(Some("text/html")), None);
    assert_eq!(far_kind(None), None);
}

#[test]
fn a_neighbour_is_only_what_the_window_could_open() {
    let (room, file) = folder_with_a_file();
    let above = room.path().join("other.md");
    std::fs::write(&above, "# other").unwrap();
    assert_eq!(neighbour(&file, "../other.md"), Some(above));
    assert!(neighbour(&file, "./notes.txt").is_some());
    assert_eq!(neighbour(&file, "img/a%20b.png"), None);
    assert_eq!(neighbour(&file, "missing.md"), None);
}

#[test]
fn a_file_cut_inside_a_letter_at_the_ceiling_is_still_too_big() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("largo.md");
    let mut said = "x".repeat(tisty_core::docs::READ_AT_MOST as usize);
    said.push('é');
    std::fs::write(&at, said).unwrap();
    assert_eq!(body_of(&at).unwrap_err().code, "documentTooBig");
}

#[test]
fn only_an_address_on_the_web_is_fetched() {
    assert!(far_picture("file:///C:/secret.png").is_err());
    assert!(far_picture("./a.png").is_err());
}

#[test]
fn a_picture_never_reaches_this_machine_or_its_network() {
    for inside in [
        "http://localhost/a.png",
        "http://127.0.0.1:8080/a.png",
        "http://0.0.0.0/a.png",
        "http://10.0.0.8/a.png",
        "http://172.16.3.1/a.png",
        "http://192.168.1.1/reboot",
        "http://169.254.169.254/latest",
        "http://100.64.0.1/a.png",
        "http://[::1]/a.png",
        "http://[fe80::1]/a.png",
        "http://[fd00::1]/a.png",
        "http://[::ffff:127.0.0.1]/a.png",
        "http://[::ffff:192.168.1.1]/a.png",
    ] {
        let address = reqwest::Url::parse(inside).unwrap();
        assert!(!reaches_outside(&address), "{inside}");
        assert!(far_picture(inside).is_err(), "{inside}");
    }
}

#[test]
fn a_public_address_is_outside() {
    for outside in [
        "https://1.1.1.1/a.png",
        "http://8.8.8.8/",
        "https://[2606:4700::1111]/",
    ] {
        assert!(
            reaches_outside(&reqwest::Url::parse(outside).unwrap()),
            "{outside}"
        );
    }
}
