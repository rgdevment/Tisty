use super::*;

#[test]
fn a_token_is_kept_sealed_read_back_replaced_and_forgotten() {
    let home = tempfile::tempdir().unwrap();
    let kept = Sealed::under(home.path(), "0123/ana");
    let token = "1//refresh-token-que-nadie-debe-leer";

    assert_eq!(kept.read("google").unwrap(), None);
    kept.write("google", token).unwrap();
    assert_eq!(kept.read("google").unwrap().as_deref(), Some(token));
    kept.write("google", "otro").unwrap();
    assert_eq!(kept.read("google").unwrap().as_deref(), Some("otro"));
    kept.forget("google").unwrap();
    kept.forget("google").unwrap();
    assert_eq!(kept.read("google").unwrap(), None);
}

#[test]
fn the_file_never_holds_the_token_as_it_reads() {
    let home = tempfile::tempdir().unwrap();
    let kept = Sealed::under(home.path(), "0123");
    let token = "1//refresh-token-que-nadie-debe-leer";

    kept.write("google", token).unwrap();

    let on_disk = std::fs::read(home.path().join("0123").join("google")).unwrap();
    assert!(
        !on_disk
            .windows(token.len())
            .any(|one| one == token.as_bytes()),
        "the token reached the disk as it reads"
    );
}

#[test]
fn a_scope_cannot_name_its_way_out_of_the_secrets_folder() {
    let home = tempfile::tempdir().unwrap();
    let kept = Sealed::under(home.path(), "../../fuera");

    kept.write("../google", "x").unwrap();

    assert!(home.path().join("______fuera").join("___google").is_file());
}

#[test]
fn bytes_that_were_never_sealed_here_read_as_a_lost_token() {
    let home = tempfile::tempdir().unwrap();
    let kept = Sealed::under(home.path(), "0123");
    std::fs::create_dir_all(home.path().join("0123")).unwrap();
    std::fs::write(home.path().join("0123").join("google"), b"no sellado").unwrap();

    assert_eq!(kept.read("google"), Err(Hitch::Lost));
}
