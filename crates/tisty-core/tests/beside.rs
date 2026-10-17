use tisty_core::Paths;
use tisty_core::beside::brought;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x06\0\0\0";

#[test]
fn a_file_named_beside_a_note_comes_in_and_one_that_is_not_there_is_named() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let outside = tmp.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("foto.png"), PNG).unwrap();
    let note = outside.join("nota.md");
    let body = "Mira ![foto](<foto.png>) y ![otra](falta.png) y [web](https://x.y)\n";
    std::fs::write(&note, body).unwrap();

    let (out, done) = brought(&paths, &note, body, &mut |at| {
        at.canonicalize()
            .map_err(|_| "no file is there".to_string())
    });

    assert_eq!(done.kept, 1, "{:?}", done.missed);
    assert_eq!(done.missed.len(), 1, "{:?}", done.missed);
    assert!(!done.told.is_empty());
    assert!(!out.contains("(<foto.png>)"), "{out}");
    assert!(out.contains("![foto](<foto-"), "{out}");
    assert!(
        out.contains(" otra "),
        "the words that named it stay: {out}"
    );
    assert!(out.contains("[web](<https://x.y>)"), "{out}");
}

#[test]
fn a_file_outside_the_note_folder_never_comes_in() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let outside = tmp.path().join("outside");
    std::fs::create_dir_all(outside.join("dentro")).unwrap();
    std::fs::write(tmp.path().join("foto.png"), PNG).unwrap();
    let note = outside.join("dentro").join("nota.md");
    let body = "![foto](../../foto.png)\n";
    std::fs::write(&note, body).unwrap();

    let (out, done) = brought(&paths, &note, body, &mut |at| {
        at.canonicalize()
            .map_err(|_| "no file is there".to_string())
    });

    assert_eq!(done.kept, 0);
    assert!(
        done.missed[0].contains("sits outside the folder"),
        "{:?}",
        done.missed
    );
    assert_eq!(out, "foto\n");
}
