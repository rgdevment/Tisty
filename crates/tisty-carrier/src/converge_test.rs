use std::collections::BTreeSet;

use proptest::prelude::*;

use super::*;

const SETTLING_ROUNDS: usize = 12;

#[derive(Debug, Clone)]
enum Step {
    Wrote(usize),
    Round(usize, Way),
    Cut(usize),
}

fn steps(machines: usize) -> impl Strategy<Value = Vec<Step>> {
    let way = prop_oneof![Just(Way::Both), Just(Way::Push), Just(Way::Pull)];
    let step = prop_oneof![
        3 => (0..machines).prop_map(Step::Wrote),
        4 => (0..machines, way).prop_map(|(at, way)| Step::Round(at, way)),
        1 => (0usize..6).prop_map(Step::Cut),
    ];
    proptest::collection::vec(step, 1..24)
}

fn provider(kind: usize, lag: u64, room: &tempfile::TempDir) -> Arc<dyn Counting> {
    match kind {
        0 => Arc::new(Fake::drive().lagging(lag)),
        1 => Arc::new(Fake::onedrive().lagging(lag)),
        2 => Arc::new(Fake::dropbox().lagging(lag)),
        _ => Arc::new(crate::folder_backed::FolderBacked::at(
            room.path().join("provider"),
        )),
    }
}

fn converges(kind: usize, lag: u64, machines: usize, steps: &[Step]) -> Result<(), TestCaseError> {
    let room = tempfile::tempdir().unwrap();
    let remote = provider(kind, lag, &room);
    let setting = Setting {
        name: format!("cloud over {}", remote.who()),
        room,
        place: Place::Cloud(remote.clone()),
    };
    let desks: Vec<Desk> = (0..machines)
        .map(|at| desk(&setting, &format!("dev_{}", (b'a' + at as u8) as char)))
        .collect();
    for one in &desks {
        for other in desks
            .iter()
            .filter(|other| other.here.device != one.here.device)
        {
            one.vouches_for(other);
        }
    }

    let mut written = BTreeSet::from(["lo que abrio el almacen".to_string()]);
    desks[0].wrote("lo que abrio el almacen");
    prop_assert!(desks[0].carry(Way::Both, Holds::Everywhere).is_ok());
    // Joining while the listing still hides the store is the race shape 2 settles; this measures what follows.
    for _ in 0..=lag {
        let _ = desks[0].carry(Way::Pull, Holds::Everywhere);
    }
    for one in &desks[1..] {
        prop_assert!(one.carry(Way::Both, Holds::Everywhere).is_ok());
    }
    for (at, step) in steps.iter().enumerate() {
        match step {
            Step::Wrote(on) => {
                let title = format!("lo escrito en el paso {at}");
                desks[*on].wrote(&title);
                written.insert(title);
            }
            Step::Round(on, way) => {
                let done = desks[*on].carry(*way, Holds::Everywhere);
                prop_assert!(done.is_ok(), "{}: a round failed: {done:?}", setting.name);
            }
            Step::Cut(after) => remote.fail_after(*after, Hitch::Unreachable("sin red".into())),
        }
    }
    for _ in 0..SETTLING_ROUNDS {
        for one in &desks {
            let done = one.carry(Way::Both, Holds::Everywhere);
            prop_assert!(done.is_ok(), "{}: a round failed: {done:?}", setting.name);
        }
    }

    let written: Vec<String> = written.into_iter().collect();
    for one in &desks {
        prop_assert_eq!(
            &one.titles(),
            &written,
            "{} lost or invented a task on {}",
            one.here.device,
            setting.name
        );
    }
    Ok(())
}

fn cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|cases| cases.parse().ok())
        .unwrap_or(8)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: cases(), ..ProptestConfig::default() })]

    #[test]
    fn two_or_three_machines_end_up_with_every_task_and_nothing_else(
        kind in 0usize..4,
        lag in 0u64..3,
        (machines, steps) in (2usize..=3).prop_flat_map(|machines| (Just(machines), steps(machines))),
    ) {
        converges(kind, lag, machines, &steps)?;
    }
}
