use tisty_core::Paths;
use tisty_core::beside::{Carried, Way, brought, retargeted, unescaped};

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x06\0\0\0";

fn carried(paths: &Paths, note: &std::path::Path, body: &str, notes: bool) -> (String, Carried) {
    let mut reach = |at: &std::path::Path| {
        at.canonicalize()
            .map_err(|_| "no file is there".to_string())
    };
    brought(
        paths,
        note,
        body,
        &mut Way {
            reach: &mut reach,
            outside: "",
            notes,
            came: tisty_core::attach::Via::Imported,
        },
    )
}

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

    let (out, done) = carried(&paths, &note, body, true);

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

    let (out, done) = carried(&paths, &note, body, true);

    assert_eq!(done.kept, 0);
    assert!(
        done.missed[0].contains("sits outside the folder"),
        "{:?}",
        done.missed
    );
    assert_eq!(out, "foto\n");
}

#[test]
fn a_link_with_a_scheme_of_its_own_is_left_as_it_was_and_a_colon_in_a_name_is_still_a_file() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let note = tmp.path().join("nota.md");
    let body = "[llamar](tel:+5691234) y [bóveda](obsidian://open?vault=x) y [ftp](ftp://x.y/a) y ![m](reunion:notas.png)\n";
    std::fs::write(&note, body).unwrap();

    let (out, done) = carried(&paths, &note, body, true);

    assert!(out.contains("(<tel:+5691234>)"), "{out}");
    assert!(out.contains("(<obsidian://open?vault=x>)"), "{out}");
    assert!(out.contains("(<ftp://x.y/a>)"), "{out}");
    assert_eq!(
        done.missed.len(),
        1,
        "a name with a colon is looked for: {:?}",
        done.missed
    );
}

#[test]
fn a_note_named_beside_is_counted_rather_than_kept_as_a_file_when_asked() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::write(tmp.path().join("capitulo.md"), "# Uno\n").unwrap();
    let note = tmp.path().join("indice.md");
    let body = "Ver [el capítulo](capitulo.md)\n";
    std::fs::write(&note, body).unwrap();

    let (out, done) = carried(&paths, &note, body, false);

    assert_eq!(out, "Ver el capítulo\n");
    assert_eq!(done.papers, ["capitulo.md"]);
    assert_eq!(done.kept, 0);
    assert!(done.missed.is_empty());
}

fn kept(body: &str) -> String {
    retargeted(body, &mut |_, target, title| {
        Some((target.to_string(), title.to_string()))
    })
}

#[test]
fn a_link_is_rewritten_and_what_surrounds_it_is_left_alone() {
    let out = retargeted("ver [doc](tisty:doc/1) ahora", &mut |_, target, title| {
        Some((format!("nuevo:{target}"), title.to_string()))
    });
    assert_eq!(out, "ver [doc](<nuevo:tisty:doc/1>) ahora");
}

#[test]
fn a_body_with_nothing_to_point_at_comes_back_as_it_went_in() {
    let body = "nada que reescribir (ni esto) [ni esto";
    assert_eq!(kept(body), body);
    assert_eq!(kept("texto ](y) mas"), "texto ](y) mas");
}

#[test]
fn a_link_whose_parenthesis_never_closes_is_left_as_written() {
    let body = "esto [x](sin cerrar";
    assert_eq!(kept(body), body);
}

#[test]
fn a_target_carrying_its_own_parentheses_is_read_whole() {
    assert_eq!(kept("[x](a(b)c)"), "[x](<a(b)c>)");
}

#[test]
fn a_picture_keeps_its_mark_and_a_refused_link_keeps_only_its_words() {
    assert_eq!(kept("mira ![foto](a.png)"), "mira ![foto](<a.png>)");
    assert_eq!(retargeted("mira [doc](x)", &mut |_, _, _| None), "mira doc");
    assert_eq!(
        retargeted("mira ![foto](a.png)", &mut |_, _, _| None),
        "mira foto",
        "el signo de la imagen se quedo sin nada que marcar"
    );
    assert_eq!(kept("a[!x](y)"), "a[!x](<y>)");
}

#[test]
fn a_link_written_inside_a_fence_is_code_and_is_not_pointed_anywhere_else() {
    let body = "```\n[x](y)\n```\n";
    assert_eq!(kept(body), body);
}

#[test]
fn an_escape_is_read_only_where_two_digits_follow_it() {
    assert_eq!(unescaped("%41BC"), "ABC");
    assert_eq!(unescaped("a%41"), "aA");
    assert_eq!(unescaped("a%4"), "a%4");
    assert_eq!(unescaped("a%zz"), "a%zz");
}
