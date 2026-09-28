use super::bundled;

#[test]
fn a_report_is_one_zip_that_carries_what_was_ticked() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join("tisty-report.zip");
    let log = (
        "tisty.log".to_string(),
        b"WARN sync folder unreachable
"
        .to_vec(),
    );

    bundled(
        &at,
        "# report
version 0.1.0
",
        std::slice::from_ref(&log),
    )
    .unwrap();

    let mut zip = zip::ZipArchive::new(std::fs::File::open(&at).unwrap()).unwrap();
    let named: Vec<String> = zip.file_names().map(str::to_owned).collect();
    assert!(named.contains(&"report.txt".to_string()), "{named:?}");
    assert!(named.contains(&"tisty.log".to_string()), "{named:?}");

    use std::io::Read;
    let mut said = String::new();
    zip.by_name("report.txt")
        .unwrap()
        .read_to_string(&mut said)
        .unwrap();
    assert!(said.contains("version 0.1.0"), "{said}");
}

#[test]
fn a_report_without_the_log_carries_only_itself() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join("tisty-report.zip");

    bundled(&at, "# report", &[]).unwrap();

    let zip = zip::ZipArchive::new(std::fs::File::open(&at).unwrap()).unwrap();
    assert_eq!(zip.file_names().count(), 1);
}
