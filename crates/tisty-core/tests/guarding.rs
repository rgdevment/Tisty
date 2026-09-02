use std::path::Path;

use tisty_core::attach::{COPIED_UP_TO, keep, resolve};
use tisty_core::docs::read_outside;

#[test]
fn resolve_refuses_every_shape_of_escape() {
    let root = Path::new("/data");
    for climbing in [
        "../../../etc/passwd",
        "attachments/../../../etc/passwd",
        "/etc/passwd",
        "//server/share/x.mp4",
        r"\\server\share\x.mp4",
        r"..\..\.ssh\id_rsa",
        "C:/Windows/win.ini",
        "C:foo",
        "file:///etc/passwd",
        "data:text/html,x",
        "javascript:alert(1)",
        "tisty:doc/a3f1-0001",
        "attachments/ab/cd.png:stream",
        "NUL",
        "",
        ".",
        "..",
    ] {
        assert!(resolve(climbing, root).is_err(), "«{climbing}» pasó");
    }
    assert_eq!(
        resolve("attachments/ab/cd.png", root).unwrap(),
        root.join("attachments/ab/cd.png")
    );
}

#[test]
fn resolve_reaches_anything_under_the_data_root_not_only_attachments() {
    let root = tempfile::tempdir().unwrap();
    let private = root.path().join("config").join("private");
    std::fs::create_dir_all(&private).unwrap();
    std::fs::write(private.join("tisty.log"), "lo que la app anota\n").unwrap();

    let at = resolve("config/private/tisty.log", root.path()).expect("aceptada");

    assert!(at.is_file());
    assert_eq!(std::fs::metadata(&at).unwrap().len(), 20);
}

#[test]
fn resolve_refuses_a_climb_written_with_percent_escapes() {
    let root = Path::new("/data");

    for hidden in [
        "%2e%2e/%2e%2e/etc/passwd",
        "%2E%2E/secret",
        "attachments%2F..%2Fsecret",
        "attachments%5Cab%5Ccd.png",
    ] {
        assert!(
            resolve(hidden, root).is_err(),
            "«{hidden}» salió del almacén"
        );
    }

    let at = resolve("attachments/ab/mi%20foto.png", root).expect("un nombre con espacios");
    assert_eq!(at, root.join("attachments").join("ab").join("mi foto.png"));
    assert!(at.starts_with(root), "sigue dentro del almacén");
}

#[cfg(unix)]
#[test]
fn a_symlink_inside_the_store_is_followed_out_of_it() {
    let outside = tempfile::tempdir().unwrap();
    let secret = outside.path().join("id_rsa");
    std::fs::write(&secret, b"-----BEGIN OPENSSH PRIVATE KEY-----\n").unwrap();

    let root = tempfile::tempdir().unwrap();
    let shelf = root.path().join("attachments").join("ab");
    std::fs::create_dir_all(&shelf).unwrap();
    std::os::unix::fs::symlink(&secret, shelf.join("leak.pdf")).unwrap();

    let at = resolve("attachments/ab/leak.pdf", root.path()).expect("resolve lo aceptó");

    assert!(at.starts_with(root.path()), "la ruta parece de dentro");
    assert!(at.is_file(), "served/opened lo tomarían por fichero");
    assert_eq!(
        std::fs::metadata(&at).unwrap().len(),
        std::fs::metadata(&secret).unwrap().len(),
        "weighs devuelve el tamaño del fichero de fuera"
    );
    assert_eq!(std::fs::read(&at).unwrap(), std::fs::read(&secret).unwrap());
}

#[test]
fn a_hand_edited_ledger_never_hands_back_a_path_that_climbs_out() {
    let outside = tempfile::tempdir().unwrap();
    let planted = outside.path().join("passwd");
    std::fs::write(&planted, b"root:x:0:0:root:/root:/bin/sh\n").unwrap();

    let root = tempfile::tempdir().unwrap();
    let source = outside.path().join("informe.pdf");
    std::fs::write(&source, b"el informe de verdad").unwrap();

    let honest = keep(&source, root.path(), COPIED_UP_TO).unwrap();
    std::fs::remove_file(root.path().join(&honest.at)).unwrap();

    let climbing = format!(
        "../{}/passwd",
        outside.path().file_name().unwrap().to_str().unwrap()
    );
    std::fs::write(
        root.path().join("attachments.jsonl"),
        format!(
            "{{\"at\":\"{climbing}\",\"sha256\":\"{}\",\"bytes\":7}}\n",
            honest.sha256
        ),
    )
    .unwrap();

    let again = keep(&source, root.path(), COPIED_UP_TO).unwrap();

    assert_ne!(again.at, climbing, "the ledger was believed");
    assert!(!again.written("informe.pdf").contains(".."));
    assert_eq!(
        std::fs::read(root.path().join(&again.at)).unwrap(),
        b"el informe de verdad",
        "and the file was written again where it belongs"
    );
}

#[test]
fn the_ledger_is_not_believed_without_checking_the_bytes() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let source = outside.path().join("mio.bin");
    std::fs::write(&source, b"mis bytes").unwrap();

    let honest = keep(&source, root.path(), COPIED_UP_TO).unwrap();
    let impostor = root.path().join("attachments/ab");
    std::fs::create_dir_all(&impostor).unwrap();
    std::fs::write(impostor.join("otro.bin"), b"bytes ajenos").unwrap();
    std::fs::write(
        root.path().join("attachments.jsonl"),
        format!(
            "{{\"at\":\"attachments/ab/otro.bin\",\"sha256\":\"{}\",\"bytes\":9}}\n",
            honest.sha256
        ),
    )
    .unwrap();

    let again = keep(&source, root.path(), COPIED_UP_TO).unwrap();

    assert_eq!(
        std::fs::read(root.path().join(&again.at)).unwrap(),
        b"mis bytes",
        "the hash said one thing and the file another"
    );
}

#[test]
fn read_outside_reads_any_absolute_path_with_no_scope_at_all() {
    let outside = tempfile::tempdir().unwrap();
    let secret = outside.path().join("secreto.txt");
    std::fs::write(&secret, "contraseñas y demás").unwrap();

    assert_eq!(read_outside(&secret).unwrap(), "contraseñas y demás");
}
