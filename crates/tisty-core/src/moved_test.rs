use super::*;

fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn dev(id: &str) -> DeviceId {
    DeviceId(id.to_string())
}

fn confirmed_here(data: &Path, who: &str, key: &SigningKey) {
    std::fs::create_dir_all(data).unwrap();
    crate::vouched::confirm(data, &dev(who), &crate::signing::shown(key));
}

#[test]
fn a_word_signed_by_a_machine_confirmed_here_is_heeded() {
    let data = tempfile::tempdir().unwrap();
    let theirs = key(1);
    confirmed_here(data.path(), "dev_b", &theirs);

    let left = said(
        &theirs,
        &dev("dev_b"),
        "Google Drive · ana@example.com",
        1_760_000_000,
    );
    let heard = heeded(Some(&left), data.path(), &dev("dev_a")).unwrap();

    assert_eq!(heard.by, "dev_b");
    assert_eq!(heard.to, "Google Drive · ana@example.com");
}

#[test]
fn a_word_nobody_here_can_vouch_for_stops_nothing() {
    let data = tempfile::tempdir().unwrap();
    let theirs = key(1);
    let stranger = key(2);
    confirmed_here(data.path(), "dev_b", &theirs);

    let forged = said(&stranger, &dev("dev_b"), "elsewhere", 1);
    let unknown = said(&stranger, &dev("dev_c"), "elsewhere", 1);
    let mut altered = read(&said(&theirs, &dev("dev_b"), "here", 1)).unwrap();
    altered.to = "elsewhere".into();
    let altered = serde_json::to_string(&altered).unwrap();

    for one in [forged, unknown, altered, "no es un aviso".to_string()] {
        assert_eq!(
            heeded(Some(&one), data.path(), &dev("dev_a")),
            None,
            "{one}"
        );
    }
    assert_eq!(heeded(None, data.path(), &dev("dev_a")), None);
}

#[test]
fn a_machine_never_heeds_the_word_it_left_itself() {
    let data = tempfile::tempdir().unwrap();
    let mine = key(1);
    confirmed_here(data.path(), "dev_a", &mine);

    let left = said(&mine, &dev("dev_a"), "elsewhere", 1);

    assert_eq!(heeded(Some(&left), data.path(), &dev("dev_a")), None);
}

#[test]
fn a_word_past_what_one_can_weigh_is_not_read() {
    let huge = format!(
        r#"{{"by":"dev_b","to":"{}","at":1,"sig":""}}"#,
        "x".repeat(8 * 1024)
    );

    assert_eq!(read(&huge), None);
}
