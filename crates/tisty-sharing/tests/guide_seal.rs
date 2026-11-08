#[allow(dead_code)]
mod common;

use std::path::Path;

use common::Home;
use tisty_share_proto::seed::Seed;
use tisty_sharing::{GuideSeal, MAKER_ROOT, SEAL, author_of_guide, guide_files, sealed};

const AUTHOR: &str = "rgdevment";

fn signing_home() -> (tempfile::TempDir, Home) {
    let tmp = tempfile::tempdir().unwrap();
    let mut home = Home::at(tmp.path());
    home.with_a_name();
    (tmp, home)
}

fn root_of_this_home() -> String {
    Seed::from_bytes(&[0x33; 32]).root().person().shown()
}

fn a_guide(at: &Path) {
    for (tongue, cover) in [("es", "guia.md"), ("en", "guide.md")] {
        std::fs::create_dir_all(at.join(tongue)).unwrap();
        std::fs::write(at.join(tongue).join(cover), format!("# {tongue}\n")).unwrap();
        std::fs::write(at.join(tongue).join("rina.jpg"), [0xff, 0xd8, 0xff]).unwrap();
    }
}

fn signed_as(home: &Home, guide: &Path, author: &str) -> GuideSeal {
    let (key, certificate) = home.signer();
    let seal = sealed(&key, &certificate, author, guide_files(guide).unwrap()).unwrap();
    std::fs::write(guide.join(SEAL), serde_json::to_vec(&seal).unwrap()).unwrap();
    seal
}

#[test]
fn a_guide_signed_by_a_machine_of_the_root_answers_with_its_author() {
    let (_tmp, home) = signing_home();
    let guide = tempfile::tempdir().unwrap();
    a_guide(guide.path());
    signed_as(&home, guide.path(), AUTHOR);

    assert_eq!(
        author_of_guide(guide.path(), &root_of_this_home()).as_deref(),
        Some(AUTHOR)
    );
    assert_eq!(author_of_guide(guide.path(), MAKER_ROOT), None);
}

#[test]
fn a_word_or_a_file_changed_after_signing_leaves_the_guide_unsigned() {
    let (_tmp, home) = signing_home();
    let root = root_of_this_home();
    let guide = tempfile::tempdir().unwrap();
    a_guide(guide.path());
    signed_as(&home, guide.path(), AUTHOR);

    std::fs::write(guide.path().join("es/guia.md"), "# es, otra\n").unwrap();
    assert_eq!(author_of_guide(guide.path(), &root), None);

    std::fs::write(guide.path().join("es/guia.md"), "# es\n").unwrap();
    assert_eq!(
        author_of_guide(guide.path(), &root).as_deref(),
        Some(AUTHOR)
    );

    std::fs::write(guide.path().join("en/extra.png"), [1, 2, 3]).unwrap();
    assert_eq!(author_of_guide(guide.path(), &root), None);
}

#[test]
fn a_seal_whose_name_was_changed_or_that_names_tisty_holds_for_nobody() {
    let (_tmp, home) = signing_home();
    let root = root_of_this_home();
    let guide = tempfile::tempdir().unwrap();
    a_guide(guide.path());

    let mut seal = signed_as(&home, guide.path(), AUTHOR);
    seal.author = "otra persona".into();
    std::fs::write(guide.path().join(SEAL), serde_json::to_vec(&seal).unwrap()).unwrap();
    assert_eq!(author_of_guide(guide.path(), &root), None);

    signed_as(&home, guide.path(), "Tisty");
    assert_eq!(author_of_guide(guide.path(), &root), None);
}

#[test]
fn a_guide_with_no_seal_has_no_author() {
    let guide = tempfile::tempdir().unwrap();
    a_guide(guide.path());
    assert_eq!(author_of_guide(guide.path(), &root_of_this_home()), None);
}
