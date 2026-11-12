use super::*;
use tisty_core::event::TaskAdd;

#[test]
fn a_page_cut_between_events_stamped_alike_resumes_before_the_first_left_out() {
    let at: jiff::Timestamp = "2026-11-11T12:00:00Z".parse().unwrap();
    let phone = DeviceId("phone".into());
    let log: Vec<Event> = (0..=MOVED_SHOWN)
        .map(|n| {
            let add = TaskAdd::new(format!("chore {n}"), "a0");
            let mut one = Event::new(
                phone.clone(),
                at,
                Op::TaskAdd {
                    id: Ulid::generate(),
                    d: add,
                },
            );
            one.seq = n as u64;
            one
        })
        .collect();
    let state = State::replay(&log);
    let since = Since::At("2026-11-11T11:00:00Z".parse().unwrap());

    let moved = moved_since(
        &state,
        &log,
        Asked {
            since: &since,
            mine_too: true,
            seen: Seen::new(),
        },
        &DeviceId("laptop".into()),
        "2026-11-11".parse().unwrap(),
    );

    assert!(moved.cut());
    assert!(
        moved.resume.unwrap() < at,
        "the first left out shares its stamp with those shown, so the cursor stands before it"
    );
}
