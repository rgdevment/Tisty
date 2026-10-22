use super::*;

fn v(text: &str) -> Version {
    Version::parse(text).unwrap()
}

fn plain(one: &str) -> String {
    one.to_string()
}

#[test]
fn a_copy_whose_version_cannot_be_read_leaves_the_path() {
    assert_eq!(due(true, true, None, None), Due::Out);
}

#[test]
fn a_developer_build_never_joins_the_path() {
    assert_eq!(due(false, true, Some(&v("0.1.0")), None), Due::Untouched);
}

#[test]
fn even_a_developer_build_leaves_when_the_door_shuts() {
    assert_eq!(due(false, false, Some(&v("0.1.0")), None), Due::Out);
}

#[test]
fn with_the_door_shut_the_folder_leaves() {
    assert_eq!(due(true, false, Some(&v("1.32.0")), None), Due::Out);
}

#[test]
fn a_candidate_installed_by_hand_is_never_the_one_called() {
    assert_eq!(due(true, true, Some(&v("1.33.0-rc.1")), None), Due::Out);
    assert_eq!(due(false, true, Some(&v("1.33.0-rc.1")), None), Due::Out);
}

#[test]
fn a_newer_store_copy_wins_and_the_older_one_leaves() {
    assert_eq!(
        due(true, true, Some(&v("1.32.0")), Some(&v("1.33.0"))),
        Due::Out
    );
}

#[test]
fn a_newer_copy_installed_by_hand_answers_before_the_store() {
    assert_eq!(
        due(true, true, Some(&v("1.33.0")), Some(&v("1.32.0"))),
        Due::In {
            ahead_of_the_store: true
        }
    );
}

#[test]
fn the_same_version_twice_keeps_the_person_s_order() {
    assert_eq!(
        due(true, true, Some(&v("1.32.0")), Some(&v("1.32.0"))),
        Due::In {
            ahead_of_the_store: false
        }
    );
}

#[test]
fn alone_on_the_machine_it_joins_the_path() {
    assert_eq!(
        due(true, true, Some(&v("1.32.0")), None),
        Due::In {
            ahead_of_the_store: false
        }
    );
}

#[test]
fn a_test_build_is_never_steady() {
    assert!(!steady());
}

#[test]
fn an_empty_path_is_not_a_missing_one() {
    assert_eq!(without("", r"C:\Tisty", &plain), None);
}

#[test]
fn a_long_path_survives_the_round_trip() {
    let path = (0..22)
        .map(|i| format!(r"C:\Users\Someone\A rather long program directory number {i}"))
        .collect::<Vec<_>>()
        .join(";");
    assert!(path.len() > 1024, "the case only bites past 1024");
    let added = within(&path, r"C:\Programs\Tisty", None, &plain).unwrap();
    assert_eq!(without(&added, r"C:\Programs\Tisty", &plain).unwrap(), path);
}

#[test]
fn an_unexpanded_entry_is_the_same_folder() {
    let expand = |one: &str| one.replace("%LOCALAPPDATA%", r"C:\Users\A\AppData\Local");
    let path = r"C:\one;%LOCALAPPDATA%\Programs\Tisty";
    assert_eq!(
        without(path, r"C:\Users\A\AppData\Local\Programs\Tisty", &expand).unwrap(),
        r"C:\one"
    );
}

#[test]
fn it_goes_at_the_end_so_what_was_there_first_still_wins() {
    let path = r"C:\one;%USERPROFILE%\.cargo\bin";
    assert_eq!(
        within(path, r"C:\Programs\Tisty", None, &plain).unwrap(),
        r"C:\one;%USERPROFILE%\.cargo\bin;C:\Programs\Tisty"
    );
}

#[test]
fn already_there_nothing_is_written() {
    let path = r"C:\one;c:\programs\TISTY\";
    assert_eq!(within(path, r"C:\Programs\Tisty", None, &plain), None);
}

#[test]
fn ahead_of_an_older_store_copy_it_moves_in_front_of_the_alias() {
    let expand = |one: &str| one.replace("%LOCALAPPDATA%", r"C:\Users\A\AppData\Local");
    let path = r"C:\one;%LOCALAPPDATA%\Microsoft\WindowsApps;C:\Programs\Tisty";
    assert_eq!(
        within(
            path,
            r"C:\Programs\Tisty",
            Some(r"C:\Users\A\AppData\Local\Microsoft\WindowsApps"),
            &expand
        )
        .unwrap(),
        r"C:\one;C:\Programs\Tisty;%LOCALAPPDATA%\Microsoft\WindowsApps"
    );
}

#[test]
fn already_ahead_of_the_alias_nothing_moves() {
    let path = r"C:\Programs\Tisty;C:\Apps\WindowsApps";
    assert_eq!(
        within(
            path,
            r"C:\Programs\Tisty",
            Some(r"C:\Apps\WindowsApps"),
            &plain
        ),
        None
    );
}

#[test]
fn an_empty_path_gets_just_the_folder() {
    assert_eq!(
        within("", r"C:\Programs\Tisty", None, &plain).unwrap(),
        r"C:\Programs\Tisty"
    );
}

#[test]
fn leaving_keeps_the_rest_in_order_and_unexpanded() {
    let path = r"%USERPROFILE%\bin;C:\Programs\Tisty;C:\Git\cmd";
    assert_eq!(
        without(path, r"C:\Programs\Tisty", &plain).unwrap(),
        r"%USERPROFILE%\bin;C:\Git\cmd"
    );
}

#[test]
fn leaving_what_was_never_there_writes_nothing() {
    assert_eq!(without(r"C:\one", r"C:\Programs\Tisty", &plain), None);
}
