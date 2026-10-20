use super::*;

#[test]
fn every_write_rings_the_bell_anew() {
    let room = tempfile::tempdir().unwrap();
    let paths = Paths::new(room.path().join("data"), room.path().join("config"));
    assert_eq!(rung(&paths), None);

    stir(&paths);
    let first = rung(&paths).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2));
    stir(&paths);

    assert_ne!(rung(&paths).unwrap(), first);
    assert!(!paths.stirred_file().starts_with(paths.data()));
}
