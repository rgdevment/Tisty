use crate::answers::agents::hands;
use tisty_core::{DeviceId, Event, Op};

fn wrote(device: &str, via: Option<&str>, at: i64, op: Op) -> Event {
    let mut event = Event::new(
        DeviceId(device.into()),
        jiff::Timestamp::from_second(at).unwrap(),
        op,
    );
    event.via = via.map(str::to_string);
    event
}

fn task(title: &str) -> Op {
    Op::TaskAdd {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::TaskAdd::new(title, "a0"),
    }
}

#[test]
fn a_client_is_one_row_whatever_it_called_itself_and_a_join_is_not_writing() {
    let agent = DeviceId("dev_agent".into());
    let events = vec![
        wrote(
            "dev_agent",
            None,
            1,
            Op::DeviceJoin {
                d: agent.clone(),
                k: Some(tisty_core::event::DeviceKind::Agent),
            },
        ),
        wrote(
            "dev_agent",
            None,
            1,
            Op::DeviceHost {
                d: agent.clone(),
                of: DeviceId("dev_laptop".into()),
            },
        ),
        wrote("dev_agent", Some("codex-mcp-client"), 10, task("one")),
        wrote("dev_agent", Some("Codex"), 20, task("two")),
        wrote("dev_agent", Some("claude-code"), 30, task("three")),
        wrote("dev_laptop", None, 40, task("the person's own")),
    ];
    let seen = vec![super::wiring::Seen {
        id: "codex",
        name: "Codex",
        at: "~/.codex/config.toml".into(),
        wired: true,
        astray: false,
        points: None,
    }];

    let rows = hands(&events, &[agent].into_iter().collect(), &seen);

    let said: Vec<String> = rows
        .iter()
        .map(|row| {
            format!(
                "{}={} wired:{:?} filed:{} wrote:{}",
                row.via.as_deref().unwrap_or("-"),
                row.named,
                row.wired,
                row.filed,
                row.wrote
            )
        })
        .collect();
    assert_eq!(
        said,
        vec![
            "codex=Codex wired:Some(true) filed:2 wrote:2",
            "claude-code=Claude Code wired:None filed:1 wrote:1",
        ],
        "{rows:?}"
    );
    assert_eq!(rows[0].last.as_deref(), Some("1970-01-01T00:00:20Z"));
}
