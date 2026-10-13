use super::*;

const PLACE: &str = "cloud:google:0123";
const ONE: &str = "01J0000000000000000000000A";
const OTHER: &str = "01J0000000000000000000000B";

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

fn a_store(data: &Path, named: &str) {
    let store = data.join("store");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::write(store.join(crate::store::MARKER), named).unwrap();
}

fn left(key: &SigningKey, by: &str, to: &str, store: &str, place: &str) -> String {
    said(
        key,
        Leaving {
            by: &dev(by),
            to,
            at: 1_760_000_000,
            store,
            place,
        },
    )
}

#[test]
fn a_word_signed_by_a_machine_confirmed_here_is_heeded() {
    let data = tempfile::tempdir().unwrap();
    let theirs = key(1);
    confirmed_here(data.path(), "dev_b", &theirs);
    a_store(data.path(), ONE);

    let word = left(
        &theirs,
        "dev_b",
        "Google Drive · ana@example.com",
        ONE,
        PLACE,
    );
    let heard = heeded(Some(&word), data.path(), &dev("dev_a"), PLACE).unwrap();

    assert_eq!(heard.by, "dev_b");
    assert_eq!(heard.to, "Google Drive · ana@example.com");
}

#[test]
fn a_word_nobody_here_can_vouch_for_stops_nothing() {
    let data = tempfile::tempdir().unwrap();
    let theirs = key(1);
    let stranger = key(2);
    confirmed_here(data.path(), "dev_b", &theirs);
    a_store(data.path(), ONE);

    let forged = left(&stranger, "dev_b", "elsewhere", ONE, PLACE);
    let unknown = left(&stranger, "dev_c", "elsewhere", ONE, PLACE);
    let mut altered = read(&left(&theirs, "dev_b", "here", ONE, PLACE)).unwrap();
    altered.to = "elsewhere".into();
    let altered = serde_json::to_string(&altered).unwrap();

    for one in [forged, unknown, altered, "no es un aviso".to_string()] {
        assert_eq!(
            heeded(Some(&one), data.path(), &dev("dev_a"), PLACE),
            None,
            "{one}"
        );
    }
    assert_eq!(heeded(None, data.path(), &dev("dev_a"), PLACE), None);
}

#[test]
fn a_word_carried_to_another_store_or_another_place_stops_nothing() {
    let data = tempfile::tempdir().unwrap();
    let theirs = key(1);
    confirmed_here(data.path(), "dev_b", &theirs);
    a_store(data.path(), ONE);

    let other_store = left(&theirs, "dev_b", "elsewhere", OTHER, PLACE);
    let other_place = left(&theirs, "dev_b", "elsewhere", ONE, "cloud:google:9999");
    let a_folder = left(&theirs, "dev_b", "elsewhere", ONE, "folder");

    for one in [other_store, other_place, a_folder] {
        assert_eq!(
            heeded(Some(&one), data.path(), &dev("dev_a"), PLACE),
            None,
            "{one}"
        );
    }
}

#[test]
fn a_word_signed_before_it_named_its_store_and_place_stops_nothing() {
    let data = tempfile::tempdir().unwrap();
    let theirs = key(1);
    confirmed_here(data.path(), "dev_b", &theirs);
    let sig = theirs.sign(b"tisty.moved\0dev_b\0elsewhere\x001");
    let older = format!(
        r#"{{"by":"dev_b","to":"elsewhere","at":1,"sig":"{}"}}"#,
        hexed(&sig.to_bytes())
    );

    assert_eq!(
        heeded(Some(&older), data.path(), &dev("dev_a"), "folder"),
        None
    );
}

#[test]
fn a_machine_never_heeds_the_word_it_left_itself() {
    let data = tempfile::tempdir().unwrap();
    let mine = key(1);
    confirmed_here(data.path(), "dev_a", &mine);

    let word = left(&mine, "dev_a", "elsewhere", "", "folder");

    assert_eq!(
        heeded(Some(&word), data.path(), &dev("dev_a"), "folder"),
        None
    );
}

#[test]
fn every_machine_names_a_place_alike() {
    let cloud = |email: &str| {
        Sync::Cloud(crate::config::CloudAt {
            provider: "google".into(),
            account: "0123".into(),
            email: email.into(),
        })
    };

    assert_eq!(place_of(&cloud("ana@example.com")), PLACE);
    assert_eq!(
        place_of(&cloud("ana.nueva@example.com")),
        place_of(&cloud("ana@example.com"))
    );
    assert_eq!(
        place_of(&Sync::folder("D:/Drive/Tisty")),
        place_of(&Sync::folder("/Users/ana/Drive/Tisty"))
    );
}

#[test]
fn a_word_past_what_one_can_weigh_is_not_read() {
    let huge = format!(
        r#"{{"by":"dev_b","to":"{}","at":1,"sig":""}}"#,
        "x".repeat(8 * 1024)
    );

    assert_eq!(read(&huge), None);
}
