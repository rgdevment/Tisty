use super::*;
use tisty_core::DeviceId;

fn by_the_agent(ms: i64, op: Op) -> Event {
    let mut said = Event::new(
        DeviceId("dev_agent".into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    );
    said.via = Some("claude-code".into());
    said
}

#[test]
fn a_note_an_agent_leaves_on_a_document_is_not_counted_as_something_it_wrote() {
    let id = ulid::Ulid::generate();
    let events = [
        by_the_agent(
            1,
            Op::TaskAdd {
                id,
                d: tisty_core::event::TaskAdd::new("ordenar la bodega", "a0"),
            },
        ),
        by_the_agent(
            2,
            Op::DocGist {
                id,
                d: tisty_core::event::Gist {
                    print: "print".into(),
                    summary: "de qué trata".into(),
                    notes: String::new(),
                },
            },
        ),
    ];
    let assistants = [DeviceId("dev_agent".into())].into();

    let all = hands(&events, &assistants, &[]);

    let claude = all
        .iter()
        .find(|one| one.via.as_deref() == Some("claude-code"))
        .expect("the hand that wrote is listed");
    assert_eq!(claude.wrote, 1, "{all:?}");
    assert_eq!(
        claude.last.as_deref(),
        Some("1970-01-01T00:00:00.001Z"),
        "{all:?}"
    );
}
