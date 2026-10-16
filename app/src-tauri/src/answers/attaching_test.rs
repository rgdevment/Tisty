use super::{pasted_name, safe_to_open};

#[test]
fn a_name_that_only_windows_reads_as_a_program_is_never_opened() {
    for name in [
        ".exe",
        ".bat",
        ".cmd",
        "pay.exe.",
        "pay.exe ",
        "pay.exe...",
        "x.exe",
        "x.msi",
        "x.settingcontent-ms",
        "x.appref-ms",
        "x.jnlp",
        "x.py",
        "x.inf",
        "x.scpt",
        "x.mobileconfig",
        "x.inetloc",
        "x.command",
        "x.desktop",
        "x.EXE",
        "x.Bat",
    ] {
        assert!(
            !safe_to_open(std::path::Path::new(name)),
            "{name} would be opened"
        );
    }
}

#[test]
fn the_files_a_person_actually_attaches_still_open() {
    for name in [
        "informe.pdf",
        "foto.png",
        "hoja.xlsx",
        "notas.md",
        "musica.mp3",
        "video.mp4",
        "datos.csv",
        "archivo.zip",
        "diagrama.svg",
        "carta.docx",
        "FOTO.JPEG",
    ] {
        assert!(
            safe_to_open(std::path::Path::new(name)),
            "{name} was refused"
        );
    }
}

#[test]
fn a_pasted_file_keeps_its_own_name_and_never_a_folder() {
    assert_eq!(pasted_name("captura.png", "image/png"), "captura.png");
    assert_eq!(pasted_name("C:\\Users\\x\\foto.jpg", ""), "foto.jpg");
    assert_eq!(pasted_name("../../etc/passwd", ""), "passwd");
    assert_eq!(pasted_name(".env", "text/plain"), ".env");
}

#[test]
fn a_pasted_file_with_no_name_is_called_by_what_it_holds() {
    assert_eq!(pasted_name("", "image/png"), "pasted.png");
    assert_eq!(pasted_name("  ", "image/jpeg"), "pasted.jpg");
    assert_eq!(pasted_name("..", "application/pdf"), "pasted.pdf");
    assert_eq!(pasted_name("", "application/x-thing"), "pasted.bin");
}
