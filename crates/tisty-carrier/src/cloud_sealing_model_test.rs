use std::collections::BTreeSet;

use proptest::prelude::*;
use tisty_share_proto::cloud::CloudKey;
use tisty_share_proto::seed::Seed;
use tisty_sync::{Sealing, Turn};

use super::*;
use crate::{Stepped, Turning};

const CLOUD: &str = "store_one";
const SETTLING: usize = 40;

#[derive(Debug, Clone)]
enum Step {
    Wrote(usize),
    Round(usize),
    Turned,
    Cut(usize),
    Crashed,
    GaveUp,
}

fn steps() -> impl Strategy<Value = Vec<Step>> {
    let step = prop_oneof![
        3 => (0usize..2).prop_map(Step::Wrote),
        3 => (0usize..2).prop_map(Step::Round),
        4 => Just(Step::Turned),
        1 => (0usize..8).prop_map(Step::Cut),
        1 => Just(Step::Crashed),
        1 => Just(Step::GaveUp),
    ];
    proptest::collection::vec(step, 1..28)
}

fn key() -> CloudKey {
    CloudKey::of(&Seed::from_bytes(&[7; 32]))
}

struct World {
    setting: Setting,
    fake: Arc<Fake>,
    devices: [String; 2],
    turning: Option<Turning>,
}

impl World {
    fn new() -> Self {
        let (setting, fake) = each_kind(|fake| fake).remove(0);
        let devices = ["dev_a".to_string(), "dev_b".to_string()];
        let (one, two) = (desk(&setting, &devices[0]), desk(&setting, &devices[1]));
        one.vouches_for(&two);
        two.vouches_for(&one);
        Self {
            setting,
            fake,
            devices,
            turning: None,
        }
    }

    fn plain(&self) -> Arc<dyn Remote> {
        self.fake.clone()
    }

    // The test reads the word past any cut it planted, so a cut only ever lands on the machines.
    fn word(&self) -> Sealing {
        for _ in 0..8 {
            let mut said = Vec::new();
            if self.fake.fetch(tisty_sync::NAMED, 0, &mut said).is_ok() {
                return tisty_sync::sealing_in(&String::from_utf8_lossy(&said));
            }
        }
        Sealing::Unsaid
    }

    // A machine reads the cloud the way its word says, as the app's `seal_the_cloud` does.
    fn desk(&self, at: usize) -> Desk {
        let base = desk(&self.setting, &self.devices[at]);
        let remote: Arc<dyn Remote> = match self.word() {
            Sealing::Sealed(_) | Sealing::Copying(Turn::Off) => {
                Arc::new(crate::Sealed::new(self.fake.clone(), key(), CLOUD))
            }
            _ => self.fake.clone(),
        };
        Desk {
            here: base.here,
            carrier: Arc::new(Cloud::over(
                remote,
                home_of(&self.setting, &self.devices[at]),
            )),
        }
    }

    fn home(&self) -> PathBuf {
        home_of(&self.setting, &self.devices[0])
    }

    fn turned(&mut self) {
        let mut turning = match self.turning.take() {
            Some(turning) => turning,
            None => match Turning::resume(self.plain(), &self.home(), key()) {
                Some(turning) => turning,
                None => {
                    let way = match self.word() {
                        Sealing::Sealed(_) => Turn::Off,
                        _ => Turn::On,
                    };
                    match Turning::begin(
                        self.plain(),
                        &self.home(),
                        key(),
                        CLOUD,
                        &self.devices[0],
                        way,
                    ) {
                        Ok(turning) => turning,
                        Err(_) => return,
                    }
                }
            },
        };
        match turning.step() {
            Ok(Stepped::Done) => {}
            _ => self.turning = Some(turning),
        }
    }

    fn gave_up(&mut self) {
        let turning = self
            .turning
            .take()
            .or_else(|| Turning::resume(self.plain(), &self.home(), key()));
        if let Some(turning) = turning {
            let _ = turning.abort();
        }
    }

    fn settled(&mut self) -> bool {
        for _ in 0..SETTLING {
            let left = self.turning.is_some()
                || Turning::resume(self.plain(), &self.home(), key()).is_some();
            if !left {
                return true;
            }
            self.turned();
            if self.turning.is_none()
                && Turning::resume(self.plain(), &self.home(), key()).is_none()
            {
                return true;
            }
        }
        false
    }
}

fn holds(steps: &[Step]) -> Result<(), TestCaseError> {
    let mut world = World::new();
    let mut written = BTreeSet::from(["lo que abrio el almacen".to_string()]);
    let first = world.desk(0);
    first.wrote("lo que abrio el almacen");
    prop_assert!(first.carry(Way::Both, Holds::Everywhere).is_ok());
    prop_assert!(world.desk(1).carry(Way::Both, Holds::Everywhere).is_ok());
    for (at, step) in steps.iter().enumerate() {
        match step {
            Step::Wrote(on) => {
                let title = format!("lo escrito en el paso {at}");
                world.desk(*on).wrote(&title);
                written.insert(title);
            }
            Step::Round(on) => {
                let _ = world.desk(*on).carry(Way::Both, Holds::Everywhere);
            }
            Step::Turned => world.turned(),
            Step::Cut(after) => world
                .fake
                .fail_after(*after, Hitch::Unreachable(None, "sin red".into())),
            Step::Crashed => world.turning = None,
            Step::GaveUp => world.gave_up(),
        }
    }
    prop_assert!(world.settled(), "a turning never ended: {:?}", world.word());
    prop_assert!(
        matches!(
            world.word(),
            Sealing::Plain | Sealing::Unsaid | Sealing::Sealed(_)
        ),
        "the place was left mid turn: {:?}",
        world.word()
    );
    for _ in 0..4 {
        for at in 0..2 {
            let done = world.desk(at).carry(Way::Both, Holds::Everywhere);
            prop_assert!(done.is_ok(), "a round failed once settled: {done:?}");
        }
    }
    let written: Vec<String> = written.into_iter().collect();
    for at in 0..2 {
        prop_assert_eq!(
            &world.desk(at).titles(),
            &written,
            "{} lost or invented a task",
            world.devices[at]
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
    fn writing_rounds_turns_cuts_crashes_and_giving_up_never_lose_or_invent_a_task(steps in steps()) {
        holds(&steps)?;
    }
}
