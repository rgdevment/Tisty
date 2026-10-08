use super::{App, Edited};
use tisty_core::Paths;

fn desk() -> (tempfile::TempDir, Paths) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    (tmp, paths)
}

#[test]
fn a_settings_file_this_version_cannot_read_is_not_written_over() {
    let (_tmp, paths) = desk();
    let mut app = App::at(paths.clone()).unwrap();
    let said = std::fs::read_to_string(paths.config_file()).unwrap();
    let unreadable = format!("holds = \"cloudonly\"\n{said}");
    std::fs::write(paths.config_file(), &unreadable).unwrap();

    let edited = app.edit_config_if(|c| {
        c.locale = Some("es".into());
        true
    });

    assert!(matches!(edited, Ok(Edited::Unreadable)));
    assert_eq!(
        std::fs::read_to_string(paths.config_file()).unwrap(),
        unreadable
    );
    assert!(app.edit_config(|c| c.editor = Some("vi".into())).is_err());
    assert_eq!(
        std::fs::read_to_string(paths.config_file()).unwrap(),
        unreadable
    );
}

#[test]
fn a_refused_change_leaves_the_file_exactly_as_it_was() {
    let (_tmp, paths) = desk();
    let mut app = App::at(paths.clone()).unwrap();
    let before = std::fs::read(paths.config_file()).unwrap();

    let edited = app.edit_config_if(|_| false);

    assert!(matches!(edited, Ok(Edited::Refused)));
    assert_eq!(std::fs::read(paths.config_file()).unwrap(), before);
}

#[test]
fn a_change_that_is_made_is_written() {
    let (_tmp, paths) = desk();
    let mut app = App::at(paths.clone()).unwrap();

    let edited = app.edit_config_if(|c| {
        c.locale = Some("es".into());
        true
    });

    assert!(matches!(edited, Ok(Edited::Saved)));
    let said = std::fs::read_to_string(paths.config_file()).unwrap();
    assert!(said.contains("locale = \"es\""), "{said}");
}
