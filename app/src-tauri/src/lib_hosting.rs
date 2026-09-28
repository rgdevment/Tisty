use super::Session;
use tisty_core::{Config, DeviceId, Op, Paths, Store};

/// An agent that joined before `device.host` existed gets its host written by the window,
/// once, as the machine — and a machine with no agent writes nothing.
#[test]
fn the_window_says_where_an_older_agent_lives_once() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let lines = || {
        tisty_core::store::read_all(paths.store())
            .unwrap()
            .into_iter()
            .filter(|one| matches!(one.op, Op::DeviceHost { .. }))
            .count()
    };

    Session::at(paths.clone()).unwrap();
    assert_eq!(lines(), 0, "no agent, nothing to say");

    let mut config = Config::load_or_init(&paths).unwrap();
    let agent = DeviceId("dev_agent".into());
    config.agent_id = Some(agent.clone());
    config.save(&paths).unwrap();
    Store::open(paths.store(), agent.clone())
        .unwrap()
        .append(Op::DeviceJoin {
            d: agent.clone(),
            k: Some(tisty_core::event::DeviceKind::Agent),
        })
        .unwrap();

    let session = Session::at(paths.clone()).unwrap();
    assert_eq!(session.state.hosts.get(&agent), Some(&config.device_id));
    assert_eq!(lines(), 1);
    let said = tisty_core::store::read_all(paths.store())
        .unwrap()
        .into_iter()
        .find(|one| matches!(one.op, Op::DeviceHost { .. }))
        .unwrap();
    assert_eq!(said.device, config.device_id, "the machine's own word");
    assert!(said.optional, "an older build walks past it");

    Session::at(paths.clone()).unwrap();
    assert_eq!(lines(), 1, "said once");
}
