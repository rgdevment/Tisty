use super::*;
use crate::config::new_device_id;

fn configured(inst: Option<&str>) -> Config {
    let mut config: Config = toml::from_str(&format!(
        "device_id = \"{}\"\nagent_id = \"dev_agent01\"\nsynced_at = \"2026-10-01T00:00:00Z\"\n",
        new_device_id()
    ))
    .unwrap();
    config.inst = inst.map(str::to_string);
    config
}

#[test]
fn a_configuration_from_before_takes_this_computer_as_its_own() {
    let mut config = configured(None);
    let was = config.device_id.clone();

    assert!(settled(&mut config, Some("aaaa")));

    assert_eq!(config.inst.as_deref(), Some("aaaa"));
    assert_eq!(
        config.device_id, was,
        "nothing to tell it from the one it came from"
    );
    assert!(config.agent_id.is_some());
}

#[test]
fn the_same_computer_changes_nothing() {
    let mut config = configured(Some("aaaa"));
    let was = config.clone();

    assert!(!settled(&mut config, Some("aaaa")));
    assert_eq!(config, was);
}

#[test]
fn a_configuration_copied_to_another_computer_does_not_write_as_the_same_machine() {
    let mut config = configured(Some("aaaa"));
    let was = config.device_id.clone();

    assert!(settled(&mut config, Some("bbbb")));

    assert_ne!(config.device_id, was);
    assert!(crate::store::is_device_name(&config.device_id.0));
    assert_eq!(config.inst.as_deref(), Some("bbbb"));
    assert!(
        config.agent_id.is_none(),
        "turning the agent on again is the person's"
    );
    assert!(
        config.synced_at.is_none(),
        "the new machine has not synced yet"
    );
}

#[test]
fn a_profile_that_roams_back_and_forth_keeps_each_computer_its_own_name_and_agent() {
    let mut config = configured(Some("aaaa"));
    let first = config.device_id.clone();
    let first_agent = config.agent_id.clone();

    settled(&mut config, Some("bbbb"));
    let second = config.device_id.clone();
    config.agent_id = Some(DeviceId("dev_agentb1".into()));
    settled(&mut config, Some("aaaa"));

    assert_eq!(config.device_id, first);
    assert_eq!(config.agent_id, first_agent);

    settled(&mut config, Some("bbbb"));
    assert_eq!(config.device_id, second);
    assert_eq!(config.agent_id, Some(DeviceId("dev_agentb1".into())));
    assert_eq!(
        config.homes.len(),
        1,
        "only the other computer is remembered"
    );
}

#[test]
fn two_processes_waking_on_the_new_computer_agree_on_its_name() {
    let mut window = configured(Some("aaaa"));
    let mut terminal = window.clone();

    settled(&mut window, Some("bbbb"));
    settled(&mut terminal, Some("bbbb"));

    assert_eq!(window.device_id, terminal.device_id);
}

#[test]
fn a_computer_that_will_not_say_what_it_is_changes_nothing() {
    let mut config = configured(Some("aaaa"));
    let was = config.clone();

    assert!(!settled(&mut config, None));
    assert_eq!(config, was);
}

#[test]
fn what_is_remembered_survives_being_written_and_read_back() {
    let mut config = configured(Some("aaaa"));
    settled(&mut config, Some("bbbb"));

    let back: Config = toml::from_str(&toml::to_string(&config).unwrap()).unwrap();

    assert_eq!(back.homes, config.homes);
    assert_eq!(back.inst, config.inst);
}

#[test]
fn the_inst_is_a_digest_and_never_the_raw_identifier() {
    let inst = inst_of(" 1b2c-raw-guid \n").unwrap();

    assert_eq!(inst.len(), 32);
    assert!(!inst.contains("raw"));
    assert_eq!(Some(inst), inst_of("1b2c-raw-guid"));
    assert_eq!(inst_of("  "), None);
}

#[test]
fn this_computer_answers_the_same_twice() {
    assert_eq!(here(), here());
}

#[test]
fn what_a_later_build_wrote_stays_at_the_top_beside_the_remembered_computers() {
    let mut config = configured(Some("aaaa"));
    config
        .rest
        .insert("from_later".into(), toml::Value::Boolean(true));
    settled(&mut config, Some("bbbb"));

    let written = toml::to_string(&config).unwrap();
    let back: Config = toml::from_str(&written).unwrap();

    assert_eq!(
        back.rest.get("from_later"),
        Some(&toml::Value::Boolean(true)),
        "{written}"
    );
    assert_eq!(back.homes, config.homes, "{written}");
}

#[test]
fn reading_a_moved_configuration_writes_the_switch_down_once() {
    let Some(this) = here() else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("config.toml");
    let config = configured(Some("another-computer"));
    std::fs::write(&file, toml::to_string(&config).unwrap()).unwrap();

    let first = Config::load(&file).unwrap().unwrap();
    let second = Config::load(&file).unwrap().unwrap();

    assert_eq!(first.inst.as_deref(), Some(this.as_str()));
    assert_ne!(first.device_id, config.device_id);
    assert_eq!(second, first, "the second read finds it already settled");
}
