use super::*;

// Only Windows runners hold a keychain that answers without a person unlocking it.
#[cfg(windows)]
#[test]
fn a_token_written_is_read_back_whole_and_forgetting_it_twice_is_no_error() {
    let keychain = Keychain::native().unwrap();
    let key = format!("test-{}", std::process::id());
    let long = "1//0".to_string() + &"a".repeat(2000);

    assert_eq!(keychain.read(&key).unwrap(), None);
    keychain.write(&key, &long).unwrap();
    assert_eq!(keychain.read(&key).unwrap().as_deref(), Some(long.as_str()));
    keychain.write(&key, "otro").unwrap();
    assert_eq!(keychain.read(&key).unwrap().as_deref(), Some("otro"));

    keychain.forget(&key).unwrap();
    keychain.forget(&key).unwrap();
    assert_eq!(keychain.read(&key).unwrap(), None);
}
