use tisty_core::arriving::{from_html, from_word, tidied};
use tisty_core::leaving_word::to_word;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\x01\x90\0\0\0\xc8\x08\x02\0\0\0";

fn back(bytes: &[u8]) -> (String, usize) {
    let word = from_word(bytes).expect("a Word file Word can open");
    (tidied(&from_html(&word.html)).body, word.pictures.len())
}

#[test]
fn a_document_goes_out_as_word_and_comes_back_as_it_was() {
    let body = "# Plan\n\n**Dato** y ==marcado== y [web](https://x.y)\n\n- uno\n- dos\n\n1. primero\n\n```\nlet x = 1;\n  y\n```\n\n| a | b |\n| --- | --- |\n| 1 | 2 |";
    let bytes = to_word(&[body.to_string()], &mut |_| None).unwrap();
    let (said, _) = back(&bytes);

    for piece in [
        "# Plan",
        "**Dato**",
        "==marcado==",
        "[web](https://x.y)",
        "- uno",
        "1. primero",
        "```\nlet x = 1;\n  y\n```",
        "| a | b |",
    ] {
        assert!(said.contains(piece), "{piece:?} missing from {said}");
    }
}

#[test]
fn a_picture_travels_inside_and_one_word_cannot_show_stays_its_words() {
    let body = "![logo](logo-0006bf09.png)\n\n![otra](otra-0006bf10.webp)";
    let mut asked = Vec::new();
    let bytes = to_word(&[body.to_string()], &mut |src| {
        asked.push(src.to_string());
        Some(if src.ends_with(".png") {
            PNG.to_vec()
        } else {
            b"RIFF....WEBP".to_vec()
        })
    })
    .unwrap();
    let (said, pictures) = back(&bytes);

    assert_eq!(asked, ["logo-0006bf09.png", "otra-0006bf10.webp"]);
    assert_eq!(pictures, 1);
    assert!(said.contains("otra"), "{said}");
}

#[test]
fn pages_follow_one_another_and_a_link_to_another_document_is_its_words() {
    let pages = [
        "# Uno\n\n[ver](tisty:doc/abc.md)".to_string(),
        "# Dos".to_string(),
    ];
    let (said, _) = back(&to_word(&pages, &mut |_| None).unwrap());

    assert!(said.contains("# Uno"), "{said}");
    assert!(said.contains("# Dos"), "{said}");
    assert!(
        said.contains("ver") && !said.contains("tisty:doc"),
        "{said}"
    );
}
