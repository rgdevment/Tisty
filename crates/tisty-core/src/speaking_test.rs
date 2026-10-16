use super::*;

fn put(at: &Path, body: &[u8]) {
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, body).unwrap();
}

#[test]
fn every_attachment_leaves_its_shelf_for_the_one_flat_folder() {
    let dir = tempfile::tempdir().unwrap();
    let shed = dir.path().join("attachments");
    put(&shed.join("23").join("plano-0006bf09.png"), b"plano");
    put(&shed.join("ab").join("acta-1234abcd.pdf"), b"acta");

    assert_eq!(flattened(&shed), (2, 0));
    assert!(shed.join("plano-0006bf09.png").is_file());
    assert!(shed.join("acta-1234abcd.pdf").is_file());
    assert!(!shed.join("23").exists());
}

#[test]
fn a_twin_already_flat_is_the_same_bytes_and_the_shelved_copy_goes() {
    let dir = tempfile::tempdir().unwrap();
    let shed = dir.path().join("attachments");
    put(&shed.join("plano-0006bf09.png"), b"plano");
    put(&shed.join("23").join("plano-0006bf09.png"), b"plano");

    assert_eq!(flattened(&shed), (1, 0));
    assert_eq!(
        std::fs::read(shed.join("plano-0006bf09.png")).unwrap(),
        b"plano"
    );
}

#[test]
fn something_else_in_the_way_stays_where_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let shed = dir.path().join("attachments");
    put(&shed.join("plano-0006bf09.png"), b"otra cosa");
    put(&shed.join("23").join("plano-0006bf09.png"), b"plano");

    assert_eq!(flattened(&shed), (0, 1));
    assert!(shed.join("23").join("plano-0006bf09.png").is_file());
}

#[test]
fn a_folder_that_is_not_a_shelf_is_left_alone() {
    let dir = tempfile::tempdir().unwrap();
    let shed = dir.path().join("attachments");
    put(&shed.join("mios").join("plano-0006bf09.png"), b"plano");

    assert_eq!(flattened(&shed), (0, 0));
    assert!(shed.join("mios").join("plano-0006bf09.png").is_file());
}

#[test]
fn a_store_speaks_once_and_its_documents_say_it_in_tisty_words() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path();
    let docs = data.join("docs");
    let device = crate::event::DeviceId("dev_a".to_string());
    put(
        &data
            .join("attachments")
            .join("23")
            .join("plano-0006bf09.png"),
        b"plano",
    );
    let doc = crate::docs::create(
        &docs,
        &device,
        "# Casa\n\n<mark data-pen=\"green\">ojo</mark> ![p](<attachments/23/plano-0006bf09.png>)\n",
    )
    .unwrap();
    let plain = crate::docs::create(&docs, &device, "# Nada\n\nsolo texto\n").unwrap();

    assert!(!spoken(data));
    let spoke = speak(data, &docs).unwrap();

    assert!(spoken(data));
    assert_eq!(spoke.moved, 1);
    assert_eq!(spoke.docs.len(), 1);
    assert_eq!(spoke.docs[0].0, doc.id);
    assert_eq!(
        crate::docs::read(&docs, &doc.id).unwrap(),
        "# Casa\n\n==ojo=={green} ![p](plano-0006bf09.png)\n"
    );
    assert_eq!(
        crate::docs::read(&docs, &plain.id).unwrap(),
        "# Nada\n\nsolo texto\n"
    );
}
