use super::*;

#[test]
fn every_backup_is_kept_newest_first_and_read_back_whole() {
    let data = tempfile::tempdir().unwrap();
    let at = Path::new("/home/rina/.claude/CLAUDE.md");
    keep(data.path(), at, b"one\n").unwrap();
    keep(data.path(), at, b"two\r\n").unwrap();
    let all = kept(data.path(), at);
    assert_eq!(all.iter().map(|one| one.n).collect::<Vec<_>>(), [2, 1]);
    assert_eq!(read_kept(data.path(), at, 2).unwrap(), b"two\r\n");
    assert_eq!(read_kept(data.path(), at, 1).unwrap(), b"one\n");
}

#[test]
fn the_first_backup_stays_and_only_the_last_twenty_others_do() {
    let data = tempfile::tempdir().unwrap();
    let at = Path::new("/home/rina/.codex/AGENTS.md");
    for n in 0..25 {
        keep(data.path(), at, format!("version {n}\n").as_bytes()).unwrap();
    }
    let all = kept(data.path(), at);
    assert_eq!(all.len(), KEPT_AT_MOST + 1);
    assert_eq!(all.first().unwrap().n, 25);
    assert_eq!(all[KEPT_AT_MOST - 1].n, 6);
    let original = all.last().unwrap();
    assert_eq!((original.n, original.bytes, original.pinned), (1, 10, true));
    assert_eq!(read_kept(data.path(), at, 1).unwrap(), b"version 0\n");
    assert_eq!(read_kept(data.path(), at, 5), Err(Refused::NoSuchBackup));
    assert!(all[..KEPT_AT_MOST].iter().all(|one| !one.pinned));
}

#[test]
fn each_file_has_its_own_backups_inside_tistys_store() {
    let data = tempfile::tempdir().unwrap();
    let claude = Path::new("/home/rina/.claude/CLAUDE.md");
    let codex = Path::new("/home/rina/.codex/AGENTS.md");
    keep(data.path(), claude, b"claude\n").unwrap();
    assert!(kept(data.path(), codex).is_empty());
    assert!(data.path().join("agents").join("kept").is_dir());
}

#[test]
fn keeping_the_same_bytes_again_adds_no_second_backup() {
    let data = tempfile::tempdir().unwrap();
    let at = Path::new("/home/rina/.gemini/GEMINI.md");
    assert_eq!(keep(data.path(), at, b"one\n").unwrap(), 1);
    assert_eq!(keep(data.path(), at, b"one\n").unwrap(), 1);
    assert_eq!(keep(data.path(), at, b"two\n").unwrap(), 2);
    assert_eq!(
        keep(data.path(), at, b"one\n").unwrap(),
        3,
        "only the newest backup is compared"
    );
    assert_eq!(kept(data.path(), at).len(), 3);
}

#[test]
fn each_backup_says_when_it_was_kept() {
    let data = tempfile::tempdir().unwrap();
    let at = Path::new("/home/rina/.claude/CLAUDE.md");
    let before = jiff::Timestamp::now() - jiff::SignedDuration::from_secs(60);
    keep(
        data.path(),
        at,
        b"one
",
    )
    .unwrap();
    let all = kept(data.path(), at);
    assert!(all[0].at.is_some_and(|when| when > before));
}
