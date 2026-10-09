use super::*;

fn line(v: u32, by: &str, op: &str) -> String {
    format!("{{\"v\":{v},\"ts\":\"2026-10-01T12:00:00Z\",\"by\":\"{by}\",{op}}}\n")
}

fn a_key(seed: u8) -> String {
    crate::signing::shown(&ed25519_dalek::SigningKey::from_bytes(&[seed; 32]))
}

fn key(by: &str, p: &str) -> String {
    line(
        16,
        by,
        &format!("\"op\":\"device.key\",\"d\":\"{by}\",\"p\":\"{p}\""),
    )
}

fn old(by: &str) -> String {
    line(
        15,
        by,
        "\"op\":\"list.add\",\"id\":\"01M47QTGVYVRCY846RM2DHCPYK\",\"d\":{\"name\":\"Casa\",\"order\":\"a\"}",
    )
}

fn segment(dir: &Path, name: &str, body: &str) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join(name), body).unwrap();
}

#[test]
fn the_key_said_after_the_history_we_held_is_the_one_taken() {
    let room = tempfile::tempdir().unwrap();
    let (mine, theirs) = (room.path().join("mine"), room.path().join("theirs"));
    let before = old("dev_b");
    segment(&mine, "active.tisty", &before);
    segment(
        &theirs,
        "active.tisty",
        &format!("{before}{}", key("dev_b", &a_key(1))),
    );

    assert_eq!(
        first_key_past(&mine, &theirs, &DeviceId("dev_b".into())).as_deref(),
        Some(a_key(1).as_str())
    );
}

#[test]
fn a_segment_that_rotated_since_we_read_it_still_lines_up() {
    let room = tempfile::tempdir().unwrap();
    let (mine, theirs) = (room.path().join("mine"), room.path().join("theirs"));
    let before = old("dev_b");
    segment(&mine, "active.tisty", &before);
    segment(
        &theirs,
        "000001.tisty",
        &format!("{before}{}", old("dev_b")),
    );
    segment(&theirs, "active.tisty", &key("dev_b", &a_key(1)));

    assert_eq!(
        first_key_past(&mine, &theirs, &DeviceId("dev_b".into())).as_deref(),
        Some(a_key(1).as_str())
    );
}

#[test]
fn a_history_that_does_not_begin_with_ours_says_nothing() {
    let room = tempfile::tempdir().unwrap();
    let (mine, theirs) = (room.path().join("mine"), room.path().join("theirs"));
    segment(&mine, "active.tisty", &old("dev_b"));
    segment(&theirs, "000000.tisty", &key("dev_b", &a_key(3)));
    segment(&theirs, "active.tisty", &old("dev_b"));

    assert_eq!(
        first_key_past(&mine, &theirs, &DeviceId("dev_b".into())),
        None
    );
}

#[test]
fn a_history_that_always_signed_is_not_one_from_before() {
    let room = tempfile::tempdir().unwrap();
    let (mine, theirs) = (room.path().join("mine"), room.path().join("theirs"));
    let signed = key("dev_b", &a_key(1));
    segment(&mine, "active.tisty", &signed);
    segment(
        &theirs,
        "active.tisty",
        &format!("{signed}{}", key("dev_b", &a_key(2))),
    );

    assert_eq!(
        first_key_past(&mine, &theirs, &DeviceId("dev_b".into())),
        None
    );
}

#[test]
fn a_key_nothing_can_read_is_not_the_first_key_said() {
    let room = tempfile::tempdir().unwrap();
    let (mine, theirs) = (room.path().join("mine"), room.path().join("theirs"));
    let before = old("dev_b");
    segment(&mine, "active.tisty", &before);
    segment(
        &theirs,
        "active.tisty",
        &format!(
            "{before}{}{}",
            key("dev_b", "not a key"),
            key("dev_b", &a_key(1))
        ),
    );

    assert_eq!(
        first_key_past(&mine, &theirs, &DeviceId("dev_b".into())).as_deref(),
        Some(a_key(1).as_str())
    );
}
