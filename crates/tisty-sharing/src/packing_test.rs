use super::own_name;

#[test]
fn a_file_is_named_without_the_stamp_of_its_bytes() {
    assert_eq!(own_name("cartilla-a1b2c3d4.pdf"), "cartilla.pdf");
    assert_eq!(own_name("attachments/mi-gata-a1b2c3d4.png"), "mi-gata.png");
    assert_eq!(
        own_name("attachments/a1/cartilla-a1b2c3d4.pdf"),
        "cartilla.pdf"
    );
}

#[test]
fn a_file_kept_under_its_stamp_alone_says_nothing_of_it() {
    assert_eq!(own_name("a1b2c3d4.pdf"), "file.pdf");
}
