#[test]
fn the_same_machine_is_called_the_same_thing_on_every_machine_that_asks() {
    for (id, called) in [
        ("dev_a657da33", "carrasco 75"),
        ("dev_jtntzhbx", "acacia 1"),
        ("dev_ej8mf31b", "roble 60"),
    ] {
        assert_eq!(
            nicknamed(id),
            called,
            "dos maquinas dejarian de llamar igual a la misma"
        );
    }
}

#[test]
fn the_whole_dictionary_is_used_and_not_a_corner_of_it() {
    let said: std::collections::BTreeSet<String> = (0..60_000)
        .map(|n| nicknamed(&format!("dev_{n:08x}")))
        .collect();

    assert!(
        said.len() > 6_000,
        "solo {} nombres de los 6400 posibles",
        said.len()
    );
}

#[test]
fn a_handful_of_machines_can_be_told_apart_by_name_alone() {
    let mut twice = 0;
    for round in 0..500 {
        let names: std::collections::BTreeSet<String> = (0..5)
            .map(|n| nicknamed(&format!("dev_{round:04x}{n:04x}")))
            .collect();
        if names.len() < 5 {
            twice += 1;
        }
    }

    assert!(twice < 15, "{twice} de 500 flotas con dos nombres iguales");
}

#[test]
fn a_nickname_is_a_word_and_a_number_that_can_be_said_out_loud() {
    for n in 0..500 {
        let said = nicknamed(&format!("dev_{n:08x}"));
        let (word, number) = said.split_once(' ').expect("una palabra y un numero");
        assert!(TREES.contains(&word), "{word} no esta en el diccionario");
        let number: u16 = number.parse().expect("un numero");
        assert!((1..=100).contains(&number), "{number} fuera de rango");
        assert!(word.chars().all(|c| c.is_ascii_lowercase()));
    }
}

#[test]
fn the_nickname_never_carries_anything_of_the_identifier_it_came_from() {
    let said = nicknamed("dev_a657da33");

    assert!(!said.contains("a657"));
    assert!(!said.contains("da33"));
    assert!(!said.contains("dev"));
}
use super::*;

fn paths(tmp: &tempfile::TempDir) -> Paths {
    Paths::new(tmp.path().join("data"), tmp.path().join("config"))
}

#[test]
fn the_device_id_is_generated_once_and_reused() {
    let tmp = tempfile::tempdir().unwrap();
    let p = paths(&tmp);

    let first = Config::load_or_init(&p).unwrap();
    let second = Config::load_or_init(&p).unwrap();

    assert_eq!(first.device_id, second.device_id);
}

#[test]
fn the_guide_it_wrote_is_remembered_so_a_second_one_is_never_written() {
    let tmp = tempfile::tempdir().unwrap();
    let p = paths(&tmp);

    let mut first = Config::load_or_init(&p).unwrap();
    assert_eq!(first.guide, None, "no hay guia antes de escribirla");
    first.guide = Some("mac0-0001".into());
    first.save(&p).unwrap();

    let again = Config::load_or_init(&p).unwrap();

    assert_eq!(again.guide.as_deref(), Some("mac0-0001"));
}

#[test]
fn a_settings_file_written_before_the_guide_existed_still_reads() {
    let tmp = tempfile::tempdir().unwrap();
    let p = paths(&tmp);
    std::fs::create_dir_all(p.config()).unwrap();
    std::fs::write(p.config_file(), "device_id = \"dev_a3f10000\"\n").unwrap();

    let kept = Config::load(&p.config_file()).unwrap().unwrap();

    assert_eq!(kept.guide, None);
}

#[test]
fn a_machine_named_where_no_directory_can_be_is_said_once_a_session() {
    let _alone = crate::witness::ALONE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let p = paths(&tmp);
    std::fs::create_dir_all(p.config()).unwrap();
    std::fs::write(
        p.config_file(),
        "device_id = \"Dev A\"
",
    )
    .unwrap();
    crate::witness::keeps(crate::witness::file(&p), false);

    for _ in 0..5 {
        Config::load(&p.config_file()).unwrap().unwrap();
    }
    let after_reads = std::fs::read_to_string(crate::witness::file(&p)).unwrap_or_default();

    for _ in 0..5 {
        Config::load_or_init(&p).unwrap();
    }
    let after_opening = std::fs::read_to_string(crate::witness::file(&p)).unwrap();
    crate::witness::stops();

    assert_eq!(
        after_reads.matches("travels nowhere").count(),
        0,
        "reading the settings is what a window refresh does, and it wrote a line each time"
    );
    assert_eq!(
        after_opening.matches("travels nowhere").count(),
        1,
        "the MCP server opens the settings once per request, and it said so each time"
    );
}

#[test]
fn two_installs_never_share_a_device_id() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();

    assert_ne!(
        Config::load_or_init(&paths(&a)).unwrap().device_id,
        Config::load_or_init(&paths(&b)).unwrap().device_id
    );
}

#[test]
fn the_config_file_is_written_outside_the_synced_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let p = paths(&tmp);
    Config::load_or_init(&p).unwrap();

    assert!(p.config_file().exists());
    assert!(!p.config_file().starts_with(p.data()));
}

#[test]
fn round_trips_through_toml() {
    let tmp = tempfile::tempdir().unwrap();
    let p = paths(&tmp);

    let mut config = Config::load_or_init(&p).unwrap();
    config.locale = Some("es".into());
    config.editor = Some("hx".into());
    config.save(&p).unwrap();

    assert_eq!(Config::load(&p.config_file()).unwrap().unwrap(), config);
}

#[test]
fn a_missing_config_is_not_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(
        Config::load(&tmp.path().join("absent.toml"))
            .unwrap()
            .is_none()
    );
}
#[test]
fn a_table_valued_field_does_not_swallow_what_follows_it() {
    let config = Config {
        device_id: DeviceId("dev_a".into()),
        inst: None,
        agent_id: None,
        agent_vouched: None,
        called: None,
        candidates: None,
        locale: Some("es".into()),
        editor: None,
        opened_by: Some("0.1.0".into()),
        on_close: Some(Closing::Hide),
        theme: Some(Theme::Dark),
        backed_up_at: None,
        restored_at: None,
        shared_was: None,
        shared_was_later: None,
        sync: Some(Sync::Folder("G:/Mi unidad/Tisty".into())),
        synced_at: None,
        heard_at: None,
        quiet: None,
        checked_at: None,
        found_version: None,
        found_in_the_shop: Some(true),
        attach_up_to: None,
        holds: None,
        guide: Some("mac0-0001".into()),
        sown: Some(true),
        here_since: Some(jiff::Timestamp::from_second(1_700_000_000).unwrap()),
        asked_for_a_star: Some(true),
        asked_to_wire: Some(true),
        homes: std::collections::BTreeMap::new(),
        rest: toml::Table::new(),
    };

    let written = toml::to_string_pretty(&config).unwrap();
    let read: Config = toml::from_str(&written).unwrap();
    assert_eq!(
        read, config,
        "round trip lost something:
{written}"
    );
}

/// A file from before the choice existed follows the computer, and the word in the file
/// is the one settings shows, so a hand edit reads back.
#[test]
fn the_look_follows_the_computer_until_one_is_chosen() {
    let before: Config = toml::from_str(r#"device_id = "dev_a""#).unwrap();
    assert_eq!(before.theme, None);

    let chosen: Config = toml::from_str("device_id = \"dev_a\"\ntheme = \"light\"").unwrap();
    assert_eq!(chosen.theme, Some(Theme::Light));
    assert_eq!("dark".parse(), Ok(Theme::Dark));
    assert_eq!("system".parse::<Theme>(), Err(()));

    let written = toml::to_string_pretty(&chosen).unwrap();
    assert!(written.contains("theme = \"light\""), "{written}");
}

fn bare() -> Config {
    Config {
        device_id: DeviceId(new_device_id()),
        inst: None,
        agent_id: None,
        agent_vouched: None,
        called: None,
        candidates: None,
        sown: None,
        locale: None,
        editor: None,
        quiet: None,
        checked_at: None,
        found_version: None,
        found_in_the_shop: None,
        attach_up_to: None,
        holds: None,
        opened_by: None,
        on_close: None,
        theme: None,
        backed_up_at: None,
        restored_at: None,
        shared_was: None,
        shared_was_later: None,
        sync: None,
        synced_at: None,
        heard_at: None,
        guide: None,
        here_since: None,
        asked_for_a_star: None,
        asked_to_wire: None,
        homes: std::collections::BTreeMap::new(),
        rest: toml::Table::new(),
    }
}

mod ceilings {
    use super::*;

    #[test]
    fn a_task_never_takes_a_file_past_what_the_product_promises() {
        let mut config = bare();
        config.attach_up_to = Some(200 * 1024 * 1024);

        assert_eq!(
            config.copies_up_to(),
            crate::attach::COPIED_UP_TO,
            "a setting talked the ceiling above what a task is meant to hold"
        );
    }

    #[test]
    fn a_store_that_never_chose_keeps_the_ceiling_it_always_had() {
        assert_eq!(bare().copies_up_to(), crate::attach::COPIED_UP_TO);
    }

    #[test]
    fn a_machine_starting_today_asks_for_less_until_it_says_otherwise() {
        let room = tempfile::tempdir().unwrap();
        let paths = Paths::new(room.path().join("data"), room.path().join("config"));

        let made = Config::load_or_init(&paths).unwrap();

        assert_eq!(made.copies_up_to(), crate::attach::COPIED_AT_FIRST);
        assert!(made.copies_up_to() < crate::attach::COPIED_UP_TO);
    }

    #[test]
    fn a_document_still_holds_its_own_ceiling_whatever_a_task_is_set_to() {
        let mut config = bare();
        config.attach_up_to = Some(crate::attach::COPIED_AT_FIRST);

        assert_eq!(config.copies_in_a_doc(), 750 * 1024 * 1024);
        assert!(config.copies_in_a_doc() > config.copies_up_to());
    }
}

mod sharing {
    use super::*;

    #[test]
    fn a_store_shares_only_where_a_folder_was_chosen() {
        let mut config = bare();
        assert!(!config.shares(), "nothing was chosen");

        config.sync = Some(Sync::Local);
        assert!(!config.shares(), "staying local is not sharing");

        config.sync = Some(Sync::Folder(std::path::PathBuf::from("G:/Drive/tisty")));
        assert!(config.shares());
    }
}

#[test]
fn one_run_of_an_older_build_does_not_erase_what_a_later_one_wrote() {
    let tmp = tempfile::tempdir().unwrap();
    let p = paths(&tmp);
    std::fs::create_dir_all(p.config()).unwrap();
    std::fs::write(
        p.config_file(),
        "device_id = \"dev_a\"
what_a_later_build_knows = 7
",
    )
    .unwrap();

    let mut read = Config::load(&p.config_file()).unwrap().unwrap();
    read.locale = Some("es".into());
    read.save(&p).unwrap();

    let again = std::fs::read_to_string(p.config_file()).unwrap();
    assert!(
        again.contains("what_a_later_build_knows = 7"),
        "opening an older build took a setting it did not know with it:
{again}"
    );
    assert!(again.contains("locale = \"es\""), "{again}");
}

#[test]
fn a_way_of_syncing_this_build_does_not_know_neither_stops_it_nor_is_thrown_away() {
    let tmp = tempfile::tempdir().unwrap();
    let p = paths(&tmp);
    std::fs::create_dir_all(p.config()).unwrap();
    std::fs::write(
        p.config_file(),
        "device_id = \"dev_a\"

[sync]
how = \"cloud\"
at = \"https://somewhere\"
",
    )
    .unwrap();

    let read = Config::load(&p.config_file())
        .expect("a way of syncing it does not know stopped it opening")
        .unwrap();

    assert!(!read.shares(), "it read an unknown way as a folder");
    read.save(&p).unwrap();

    let again = std::fs::read_to_string(p.config_file()).unwrap();
    assert!(
        again.contains("cloud") && again.contains("https://somewhere"),
        "the way this machine was set to sync was thrown away:
{again}"
    );
}

#[test]
fn a_way_of_syncing_it_does_know_and_cannot_make_sense_of_is_said_out_loud() {
    let said = ["device_id = \"dev_a\"", "[sync]", "how = \"folder\""].join("\n");
    let broken = toml::from_str::<Config>(&said);

    assert!(
        broken.is_err(),
        "a folder to sync with and no folder named read as a way this build never heard of, which turns syncing off without a word"
    );
}

#[test]
fn a_key_this_build_cannot_name_survives_a_sync_folder_being_set() {
    let tmp = tempfile::tempdir().unwrap();
    let p = paths(&tmp);
    std::fs::create_dir_all(p.config()).unwrap();
    let said = [
        "device_id = \"dev_a\"",
        "what_a_later_build_knows = 7",
        "and_a_word = \"kept\"",
        "[sync]",
        "how = \"folder\"",
        "at = \"G:/Mi unidad/Tisty\"",
    ]
    .join(
        "
",
    );
    std::fs::write(p.config_file(), &said).unwrap();

    let read = Config::load(&p.config_file()).unwrap().unwrap();
    assert!(
        read.shares(),
        "the folder it was set to sync with was lost on the way in"
    );
    read.save(&p).unwrap();

    let again = std::fs::read_to_string(p.config_file()).unwrap();
    let back = Config::load(&p.config_file())
        .expect("what it wrote itself it can no longer read")
        .unwrap();

    assert_eq!(
        back.rest
            .get("what_a_later_build_knows")
            .and_then(toml::Value::as_integer),
        Some(7),
        "a key of a later build ended up somewhere else:
{again}"
    );
    assert_eq!(
        back.rest.get("and_a_word").and_then(toml::Value::as_str),
        Some("kept"),
        "{again}"
    );
    assert!(
        back.shares(),
        "the folder to sync with did not survive:
{again}"
    );
}

fn later() -> toml::Value {
    toml::from_str("how = \"nube\"\nat = { cuenta = \"yo\" }").unwrap()
}

#[test]
fn a_folder_places_itself_and_a_way_nobody_here_reads_does_not() {
    let folder = Sync::folder("G:/Mi unidad/compartida");

    assert_eq!(folder.place(), Some(Path::new("G:/Mi unidad/compartida")));
    assert!(folder.shares());
    assert_eq!(folder.leaving(), Leaving::Place);
    for nobody in [Sync::alone(), Sync::Unknown(later())] {
        assert_eq!(nobody.place(), None);
        assert!(
            !nobody.shares(),
            "a way this build cannot read offered its settings"
        );
    }
    assert_eq!(Sync::alone().leaving(), Leaving::Free);
    assert_eq!(Sync::Unknown(later()).leaving(), Leaving::Later);
}

#[test]
fn only_a_folder_makes_the_holds_setting_count() {
    let mut config = Config::load_or_init(&paths(&tempfile::tempdir().unwrap())).unwrap();
    config.holds = Some(Holds::Mine);

    for sync in [None, Some(Sync::alone()), Some(Sync::Unknown(later()))] {
        config.sync = sync;
        assert_eq!(config.holds(), Holds::Everywhere);
        assert!(!config.shares());
    }
    config.sync = Some(Sync::folder("compartida"));
    assert_eq!(config.holds(), Holds::Mine);
    assert!(config.shares());
}

#[test]
fn a_way_put_away_by_a_restore_is_read_from_the_old_key_or_the_new_one() {
    let mut config = Config::load_or_init(&paths(&tempfile::tempdir().unwrap())).unwrap();
    assert_eq!(config.once_shared(), None);

    config.remember_shared(Some(Was::Folder("G:/compartida".into())));
    assert_eq!(
        config.once_shared(),
        Some(Was::Folder("G:/compartida".into()))
    );
    assert_eq!(config.shared_was_later, None);

    config.remember_shared(Some(Was::Later(later())));
    assert_eq!(config.once_shared(), Some(Was::Later(later())));
    assert_eq!(config.shared_was, None);

    config.remember_shared(None);
    assert_eq!(config.once_shared(), None);
}

#[test]
fn a_file_from_before_the_later_key_reads_and_writes_back_the_same() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths(&tmp);
    let mut config = Config::load_or_init(&paths).unwrap();
    config.remember_shared(Some(Was::Folder("G:/compartida".into())));
    config.save(&paths).unwrap();

    let said = std::fs::read_to_string(paths.config_file()).unwrap();
    assert!(said.contains("shared_was = "), "{said}");
    assert!(!said.contains("shared_was_later"), "{said}");

    let read = Config::load(&paths.config_file()).unwrap().unwrap();
    assert_eq!(
        read.once_shared(),
        Some(Was::Folder("G:/compartida".into()))
    );
}

#[test]
fn a_later_way_survives_a_trip_through_the_file_in_both_places() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths(&tmp);
    let mut config = Config::load_or_init(&paths).unwrap();
    config.sync = Some(Sync::Unknown(later()));
    config.remember_shared(Some(Was::Later(later())));
    config
        .rest
        .insert("a_scalar_of_a_newer_build".into(), 7.into());
    config
        .rest
        .insert("a_table_of_a_newer_build".into(), later());
    config.save(&paths).unwrap();

    let said = std::fs::read_to_string(paths.config_file()).unwrap();
    let read = Config::load(&paths.config_file()).unwrap().unwrap();

    assert_eq!(read.sync, Some(Sync::Unknown(later())), "{said}");
    assert_eq!(read.once_shared(), Some(Was::Later(later())), "{said}");
    assert_eq!(
        read.rest
            .get("a_scalar_of_a_newer_build")
            .and_then(toml::Value::as_integer),
        Some(7),
        "a key written after the tables was swallowed by one of them: {said}"
    );
    assert_eq!(
        read.rest.get("a_table_of_a_newer_build"),
        Some(&later()),
        "{said}"
    );
    assert!(
        said.contains("[shared_was_later]"),
        "an older build only keeps it if it is a key of its own: {said}"
    );
    assert!(
        !said.contains("shared_was = "),
        "what an older build reads as a path was written as something else: {said}"
    );
}

#[test]
fn a_file_written_by_hand_before_the_later_key_existed_still_reads() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths(&tmp);
    std::fs::create_dir_all(paths.config_file().parent().unwrap()).unwrap();
    std::fs::write(
        paths.config_file(),
        "device_id = \"dev_a\"\nshared_was = \"G:/compartida\"\n\n[sync]\nhow = \"folder\"\nat = \"G:/otra\"\n",
    )
    .unwrap();

    let read = Config::load(&paths.config_file()).unwrap().unwrap();

    assert_eq!(
        read.once_shared(),
        Some(Was::Folder("G:/compartida".into()))
    );
    assert_eq!(read.sync, Some(Sync::folder("G:/otra")));
    assert_eq!(read.shared_was_later, None);
}
