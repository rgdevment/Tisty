use sha2::{Digest, Sha256};

use super::*;

fn in_the_tree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FOLDER)
}

#[test]
fn the_table_in_the_tree_is_the_one_derived_and_named() {
    let table = std::fs::read(in_the_tree().join("table.bin")).unwrap();
    let vocab = std::fs::read(in_the_tree().join("vocab.txt")).unwrap();
    let hex = |bytes: &[u8]| {
        Sha256::digest(bytes)
            .iter()
            .map(|one| format!("{one:02x}"))
            .collect::<String>()
    };
    assert_eq!(
        hex(&table),
        "f9178b5849f4ab6c2aef29c568abb34b8db8cec8b878a2553e944ebff192f3af",
        "a new table needs a new VERSION"
    );
    assert_eq!(
        hex(&vocab),
        "20e7ec04f4dc4c49f5d0962cf91aa460303f18e7172f8754e7006fceab4e089a"
    );
}

#[test]
fn the_shipped_model_reads_a_text_and_rests_until_asked_again() {
    let mut shipped = Shipped::at(Some(in_the_tree()));
    let meant = shipped.meaning("Renovar la licencia de conducir").unwrap();
    assert_eq!(meant.len(), 256);
    assert!(shipped.loaded.is_some());
    shipped.rest();
    assert!(shipped.loaded.is_none());
    assert!(!shipped.refused());
    assert_eq!(
        shipped.version(),
        "static-similarity-mrl-multilingual-v1@b68f412:256:i8"
    );
}

#[test]
fn a_missing_model_is_refused_for_a_while_and_tried_again_after() {
    let empty = tempfile::tempdir().unwrap();
    let mut shipped = Shipped::at(Some(empty.path().to_path_buf()));
    assert!(shipped.meaning("hola").is_none());
    assert!(shipped.refused());
    for file in ["table.bin", "vocab.txt"] {
        std::fs::copy(in_the_tree().join(file), empty.path().join(file)).unwrap();
    }
    assert!(shipped.meaning("hola").is_none(), "a refusal holds a while");

    shipped.refused_at = Instant::now().checked_sub(TRIES_AGAIN_AFTER + Duration::from_secs(1));
    assert!(!shipped.refused());
    assert_eq!(shipped.meaning("hola").map(|one| one.len()), Some(256));
}

#[test]
fn a_table_that_does_not_read_is_refused() {
    let broken = tempfile::tempdir().unwrap();
    std::fs::write(broken.path().join("table.bin"), [1, 2, 3]).unwrap();
    std::fs::write(broken.path().join("vocab.txt"), "[UNK]").unwrap();
    let mut shipped = Shipped::at(Some(broken.path().to_path_buf()));
    assert!(shipped.meaning("hola").is_none());
    assert!(shipped.refused());
}

#[test]
fn no_folder_at_all_is_refused() {
    let mut shipped = Shipped::at(None);
    assert!(shipped.meaning("hola").is_none());
    assert!(shipped.refused());
}

#[test]
fn the_table_is_looked_for_where_the_bundler_puts_resources() {
    let exe = Path::new("install").join("Tisty.exe");
    let found = beside(&exe);
    assert!(found.ends_with(FOLDER));
    assert!(found.starts_with("install"));
    assert!(Shipped::beside_the_app().at.is_some());
}
