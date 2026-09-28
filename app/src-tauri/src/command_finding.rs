use super::*;

#[test]
fn a_window_that_answers_to_the_cli_name_has_none_beside_it() {
    let tmp = tempfile::tempdir().unwrap();
    let window = tmp.path().join(CLI);
    std::fs::write(&window, b"the window itself").unwrap();

    assert_eq!(with_cli(&window), None);
}

#[test]
fn the_cli_next_door_is_found() {
    let tmp = tempfile::tempdir().unwrap();
    let window = tmp.path().join(if cfg!(windows) {
        "tisty-gui.exe"
    } else {
        "tisty-gui"
    });
    std::fs::write(&window, b"the window").unwrap();
    std::fs::write(tmp.path().join(CLI), b"the command").unwrap();

    assert_eq!(with_cli(&window).unwrap(), tmp.path());
}

#[test]
fn a_window_on_its_own_has_none_beside_it() {
    let tmp = tempfile::tempdir().unwrap();
    let window = tmp.path().join(if cfg!(windows) {
        "tisty-gui.exe"
    } else {
        "tisty-gui"
    });
    std::fs::write(&window, b"the window").unwrap();

    assert_eq!(with_cli(&window), None);
}

#[test]
fn without_one_beside_it_the_name_alone_is_what_others_run() {
    assert_eq!(called(None), "tisty");
}

#[cfg(windows)]
#[test]
fn the_packaged_window_is_not_taken_for_the_command() {
    let tmp = tempfile::tempdir().unwrap();
    let window = tmp.path().join("Tisty.exe");
    std::fs::write(&window, b"the window the Store installs").unwrap();

    assert_eq!(with_cli(&window), None);
}
