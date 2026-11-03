use std::path::{Path, PathBuf};

use tisty_core::event::TaskAdd;
use tisty_core::seal::{self, Line};
use tisty_core::signing::{self, SigningKey};
use tisty_core::store::{check_device, read_all};
use tisty_core::{DeviceId, Op, Paths, State, Store, backup};
use ulid::Ulid;

const LINKS: [&str; 5] = [
    r#"{"v":18,"ts":"2026-11-01T18:59:16.5467216Z","by":"dev_a","opt":true,"tz":"America/Santiago","op":"share.doc","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{"server":"https://tisty.cl","link":"k7Qm2xPa","key":"tWj3cJFtRiwp6ms2QS6DKXwHL2aGnEB6HcDUhX9r99E","edit":"edit-token","admin":"admin-token","editable":true,"gated":true,"version":1,"still":true,"print":"uno","signs":"alias","holds":["01M14RFT9ECC2B6E4CX4P59XPH"],"cites":true}}"#,
    r#"{"v":18,"ts":"2026-11-01T18:59:16.5526586Z","by":"dev_a","opt":true,"tz":"America/Santiago","op":"share.version","id":"01M14RFT9ECC2B6E4CX4P59XPH","link":"k7Qm2xPa","version":2,"print":"dos"}"#,
    r#"{"v":18,"ts":"2026-11-01T18:59:16.5550000Z","by":"dev_a","opt":true,"op":"share.version","id":"01M14RFT9ECC2B6E4CX4P59XPH","version":3}"#,
    r#"{"v":18,"ts":"2026-11-01T18:59:16.5574072Z","by":"dev_a","opt":true,"tz":"America/Santiago","op":"share.stop","id":"01M14RFT9ECC2B6E4CX4P59XPH","link":"k7Qm2xPa"}"#,
    r#"{"v":18,"ts":"2026-11-01T18:59:16.5600000Z","by":"dev_a","opt":true,"op":"share.stop","id":"01M14RFT9ECC2B6E4CX4P59XPH"}"#,
];

fn key() -> SigningKey {
    SigningKey::from_bytes(&[7; 32])
}

fn task(title: &str) -> Op {
    Op::TaskAdd {
        id: Ulid::generate(),
        d: TaskAdd::new(title, "a0"),
    }
}

fn signed(root: &Path) -> Store {
    Store::open(root, DeviceId("dev_a".into()))
        .unwrap()
        .signing_with(Some(key()))
}

fn with_links(root: &Path) -> (PathBuf, Vec<u8>) {
    signed(root).append(task("before")).unwrap();
    let active = root.join("dev_a").join("active.tisty");
    let mut said = std::fs::read(&active).unwrap();
    for line in LINKS {
        said.extend_from_slice(line.as_bytes());
        said.push(b'\n');
    }
    (active, said)
}

fn holding_links(root: &Path) -> PathBuf {
    let (active, said) = with_links(root);
    std::fs::write(&active, said).unwrap();
    signed(root).append(task("after")).unwrap();
    active
}

fn ending_in_links(root: &Path) -> PathBuf {
    let (active, mut said) = with_links(root);
    let (before, _) = last_seal(&std::fs::read(&active).unwrap());
    let closing = seal::Seal {
        seg: before.seal.seg,
        at: said.len() as u64,
        tip: signing::tip_of(signing::NOTHING_BEFORE, &said),
        n: 1 + LINKS.len() as u64,
        closed: false,
    };
    said.extend_from_slice(seal::line(&key(), "dev_a", &closing).as_bytes());
    std::fs::write(&active, said).unwrap();
    active
}

fn last_seal(said: &[u8]) -> (Box<seal::Read>, usize) {
    let cut = said[..said.len() - 1]
        .iter()
        .rposition(|one| *one == b'\n')
        .unwrap()
        + 1;
    let Line::Seal(read) = seal::read(&said[cut..]) else {
        panic!("the last write ends in its seal");
    };
    (read, cut)
}

fn answers_for_every_byte(active: &Path, events: usize) {
    let said = std::fs::read(active).unwrap();
    let (read, cut) = last_seal(&said);
    assert!(seal::holds(&key().verifying_key(), "dev_a", &read));
    assert_eq!(read.seal.at, cut as u64);
    assert_eq!(
        read.seal.tip,
        signing::tip_of(signing::NOTHING_BEFORE, &said[..cut])
    );
    assert_eq!(read.seal.n, events as u64);
}

fn titles(root: &Path) -> Vec<String> {
    let mut found: Vec<String> = State::replay(&read_all(root).unwrap())
        .tasks
        .into_values()
        .map(|one| one.title)
        .collect();
    found.sort();
    found
}

#[test]
fn every_link_a_development_build_kept_is_still_a_name_this_build_never_reads() {
    for line in LINKS {
        assert!(
            serde_json::from_str::<tisty_core::Event>(line).is_err(),
            "{line}"
        );
        let op = serde_json::from_str::<serde_json::Value>(line).unwrap()["op"].clone();
        let op = op.as_str().unwrap();
        assert!(!tisty_core::event::KNOWN_OPS.contains(&op), "{op}");
        assert!(tisty_core::event::RETIRED_OPS.contains(&op), "{op}");
    }
}

#[test]
fn a_log_holding_those_links_replays_and_its_seal_still_answers_for_every_byte() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let active = holding_links(&root);

    assert_eq!(titles(&root), ["after", "before"]);
    assert_eq!(check_device(&root.join("dev_a")).unwrap(), 2);
    answers_for_every_byte(&active, 2 + LINKS.len());
}

#[test]
fn a_store_whose_last_lines_are_those_links_projects_opens_and_writes_on() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let active = ending_in_links(&root);
    let kept = std::fs::read(&active).unwrap();
    answers_for_every_byte(&active, 1 + LINKS.len());
    assert_eq!(check_device(&root.join("dev_a")).unwrap(), 1);

    let state = tisty_core::cache::project(&root, &room.path().join("cache")).unwrap();
    assert_eq!(state.tasks.len(), 1);
    signed(&root).append(task("after")).unwrap();

    assert!(std::fs::read(&active).unwrap().starts_with(&kept));
    assert_eq!(check_device(&root.join("dev_a")).unwrap(), 2);
    assert_eq!(titles(&root), ["after", "before"]);
    answers_for_every_byte(&active, 2 + LINKS.len());
}

#[test]
fn a_segment_closed_over_those_links_counts_them_and_checks_whole() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let active = holding_links(&root);
    let kept = std::fs::read(&active).unwrap();

    signed(&root)
        .append_batch((0..5_000).map(|n| task(&format!("filler {n}"))).collect())
        .unwrap();

    let closed = root.join("dev_a").join("000001.tisty");
    assert!(std::fs::read(&closed).unwrap().starts_with(&kept));
    assert_eq!(check_device(&root.join("dev_a")).unwrap(), 5_002);
    assert_eq!(read_all(&root).unwrap().len(), 5_002);
}

#[test]
fn another_machine_beside_those_links_merges_with_them_untouched() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let active = holding_links(&root);
    let kept = std::fs::read(&active).unwrap();

    Store::open(&root, DeviceId("dev_b".into()))
        .unwrap()
        .append(task("from the other machine"))
        .unwrap();

    assert_eq!(titles(&root), ["after", "before", "from the other machine"]);
    assert_eq!(std::fs::read(&active).unwrap(), kept);
}

#[test]
fn a_backup_holding_those_links_restores_them_as_they_were() {
    let room = tempfile::tempdir().unwrap();
    let data = room.path().join("data");
    let active = holding_links(&data.join("store"));
    let kept = std::fs::read(&active).unwrap();
    let file = room.path().join("tisty.zip");
    backup::write(&data, &file, &room.path().join("aside"), None).unwrap();

    let fresh = tempfile::tempdir().unwrap();
    let paths = Paths::new(fresh.path().join("data"), fresh.path().join("config"));
    backup::read(&paths, &file).unwrap();

    assert_eq!(titles(&paths.store()), ["after", "before"]);
    assert_eq!(
        std::fs::read(paths.store().join("dev_a").join("active.tisty")).unwrap(),
        kept
    );
}
