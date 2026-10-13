use super::*;
use crate::fakes::{Counting, Fake};

#[test]
fn meters_spending_at_the_same_time_are_all_counted() {
    let home = tempfile::tempdir().unwrap();
    let fake = Fake::drive();

    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                let meter = Metered::over(&fake, home.path(), Class::History);
                for _ in 0..25 {
                    meter.list("").unwrap();
                }
            });
        }
    });

    assert_eq!(
        Spent::load(home.path()).units,
        100 * fake.limits().costs.list,
        "a meter wrote over what another had spent"
    );
}

#[test]
fn spending_is_counted_as_each_request_goes_out() {
    let home = tempfile::tempdir().unwrap();
    let fake = Fake::drive();
    let meter = Metered::over(&fake, home.path(), Class::History);

    meter.list("").unwrap();

    assert_eq!(
        Spent::load(home.path()).units,
        fake.limits().costs.list,
        "a round cut short here would have spent without saying so"
    );
    let other = Metered::over(&fake, home.path(), Class::History);
    other.list("").unwrap();
    assert_eq!(Spent::load(home.path()).units, 2 * fake.limits().costs.list);
}

#[test]
fn a_charge_still_counts_when_another_process_holds_the_budget_too_long() {
    let home = tempfile::tempdir().unwrap();
    let fake = Fake::drive();
    let _held = Held::take(home.path(), BUDGETING).unwrap().unwrap();
    let meter = Metered::over(&fake, home.path(), Class::History);

    meter.list("").unwrap();

    assert_eq!(
        Spent::load(home.path()).units,
        fake.limits().costs.list,
        "what went out was never counted, so a loop could spend past the day"
    );
}

#[test]
fn a_cloud_paused_here_or_from_afar_asks_nothing_not_even_what_the_person_asked_for() {
    let home = tempfile::tempdir().unwrap();
    let fake = Fake::drive();

    policy::pause_here(home.path(), true).unwrap();
    let held = Metered::over(&fake, home.path(), Class::Asked).list("");
    policy::pause_here(home.path(), false).unwrap();
    let freed = Metered::over(&fake, home.path(), Class::Asked).list("");

    assert_eq!(held, Err(Hitch::Paused));
    assert!(freed.is_ok());
    assert_eq!(
        crate::fakes::Counting::counts(&fake).lists,
        1,
        "the paused one was asked after all"
    );
}

#[test]
fn the_limits_of_a_signed_word_are_the_ones_the_meter_keeps() {
    let home = tempfile::tempdir().unwrap();
    let fake = Fake::drive();
    std::fs::write(
        home.path().join("cloud-policy.json"),
        br#"{"serial":1,"units_a_day":1}"#,
    )
    .unwrap();
    let meter = Metered::over(&fake, home.path(), Class::History);

    meter.list("").unwrap();

    assert_eq!(
        meter.list(""),
        Err(Hitch::Spent),
        "the lowered day was not kept"
    );
}

#[test]
fn a_paused_cloud_does_not_listen_either() {
    let home = tempfile::tempdir().unwrap();
    let fake = Fake::dropbox();

    let open = Metered::over(&fake, home.path(), Class::History).hears("0");
    policy::pause_here(home.path(), true).unwrap();
    let held = Metered::over(&fake, home.path(), Class::History).hears("0");

    assert!(open.is_some(), "a provider that can say so was not heard");
    assert!(
        held.is_none(),
        "a paused cloud kept listening to the provider"
    );
}

#[test]
fn a_listen_already_open_is_charged_each_ask_and_stops_when_the_cloud_pauses() {
    let home = tempfile::tempdir().unwrap();
    let fake = Fake::dropbox();
    let mut heard = Metered::over(&fake, home.path(), Class::History)
        .hears("0")
        .unwrap();

    heard.told(std::time::Duration::ZERO).unwrap();
    let after_one = Spent::load(home.path()).units;
    policy::pause_here(home.path(), true).unwrap();

    assert_eq!(
        after_one,
        fake.limits().costs.changes,
        "the ask went uncounted"
    );
    assert_eq!(heard.told(std::time::Duration::ZERO), Err(Hitch::Paused));
    assert_eq!(
        Spent::load(home.path()).units,
        after_one,
        "a paused listen still asked"
    );
}

#[test]
fn the_look_a_provider_takes_before_writing_is_counted_with_the_write() {
    let home = tempfile::tempdir().unwrap();
    let fake = Fake::drive();
    let meter = Metered::over(&fake, home.path(), Class::History);
    let source = home.path().join("bytes");
    std::fs::write(&source, b"unos bytes").unwrap();

    meter
        .put("store/dev_a.log", &source, crate::Expect::Absent)
        .unwrap();
    meter.delete("store/dev_a.log", None).unwrap();

    let costs = fake.limits().costs;
    assert_eq!(
        Spent::load(home.path()).units,
        costs.put + costs.delete + 2 * costs.checks,
        "the budget sat below what Google counts"
    );
}

struct SpendingMore(Fake, std::sync::atomic::AtomicU64);

impl Remote for SpendingMore {
    fn limits(&self) -> Limits {
        self.0.limits()
    }
    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        self.1.store(300, std::sync::atomic::Ordering::SeqCst);
        self.0.list(under)
    }
    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        self.0.fetch(name, from, into)
    }
    fn put(&self, name: &str, from: &Path, expect: crate::Expect) -> Result<Seen, Hitch> {
        self.0.put(name, from, expect)
    }
    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        self.0.delete(name, expect)
    }
    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        self.0.hash_of(local)
    }
    fn spent_beyond(&self) -> u64 {
        self.1.swap(0, std::sync::atomic::Ordering::SeqCst)
    }
}

#[test]
fn what_a_provider_spent_beyond_a_verb_is_charged_with_it_and_only_once() {
    let home = tempfile::tempdir().unwrap();
    let more = SpendingMore(Fake::drive(), std::sync::atomic::AtomicU64::new(0));
    let meter = Metered::over(&more, home.path(), Class::History);

    meter.list("").unwrap();
    meter.list("").unwrap();

    assert_eq!(
        Spent::load(home.path()).units,
        2 * more.limits().costs.list + 600,
    );
}

#[test]
fn a_request_that_never_left_this_machine_costs_nothing() {
    use crate::fakes::Counting;
    let home = tempfile::tempdir().unwrap();
    let fake = Fake::drive();
    let meter = Metered::over(&fake, home.path(), Class::History);
    fake.fail_next(Hitch::Unreachable("sin red".into()));

    assert!(meter.list("").is_err());

    assert_eq!(Spent::load(home.path()).units, 0);
}

#[test]
fn a_budget_that_does_not_read_starts_from_nothing() {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(home.path().join(FILE), b"{ no es json").unwrap();

    assert_eq!(Spent::load(home.path()).units, 0);
    assert_eq!(Spent::load(home.path()).units, 0);
}

#[test]
fn a_round_that_keeps_asking_with_nothing_landing_stops_and_anything_landing_starts_the_count_again()
 {
    let home = tempfile::tempdir().unwrap();
    let local = home.path().join("local.md");
    std::fs::write(&local, b"hola").unwrap();
    let fake = Fake::drive();
    fake.refusing("never/", Hitch::Broke("not taken".into()));
    let meter = Metered::over(&fake, home.path(), Class::Attachments);
    let each = fake.limits().costs.put + fake.limits().costs.checks;

    let mut asked = 0;
    let stopped = loop {
        match meter.put(&format!("never/{asked}.md"), &local, Expect::Absent) {
            Err(Hitch::Troubled(why)) => break why,
            _ => asked += 1,
        }
        assert!(
            asked < 1_000,
            "a round that landed nothing was never stopped"
        );
    };

    assert_eq!(asked, STALLED_UNITS.div_ceil(each), "{stopped}");
    assert_eq!(
        meter.list(""),
        Err(Hitch::Troubled(stopped)),
        "the stalled round went on asking"
    );
    meter.moved_on();
    assert!(meter.put("landed.md", &local, Expect::Absent).is_ok());
    for at in 0..asked {
        assert_eq!(
            meter.put(&format!("never/again-{at}.md"), &local, Expect::Absent),
            Err(Hitch::Broke("not taken".into())),
            "a landing did not start the count again"
        );
    }
}

#[test]
fn many_cheap_asks_with_nothing_landing_stop_a_round_too() {
    let home = tempfile::tempdir().unwrap();
    let fake = Fake::dropbox();
    let meter = Metered::over(&fake, home.path(), Class::History);

    let mut asked = 0;
    loop {
        fake.fail_after(0, Hitch::Broke("an answer that did not read".into()));
        match meter.about("nada/aqui.md") {
            Err(Hitch::Troubled(_)) => break,
            _ => asked += 1,
        }
        assert!(asked < 1_000, "a round of cheap asks was never stopped");
    }

    assert_eq!(asked, STALLED_ASKS);
}

#[test]
fn answers_that_say_how_things_stand_never_take_a_round_for_a_loop() {
    let home = tempfile::tempdir().unwrap();
    let fake = Fake::drive();
    let meter = Metered::over(&fake, home.path(), Class::Attachments);
    let local = home.path().join("local.md");
    std::fs::write(&local, b"hola").unwrap();
    meter.put("aqui.md", &local, Expect::Absent).unwrap();

    for at in 0..(2 * STALLED_ASKS) {
        let mut sink = Vec::new();
        assert!(matches!(
            meter.fetch(&format!("gone/{at}.md"), 0, &mut sink),
            Err(Hitch::Missing(_))
        ));
        assert!(matches!(
            meter.put("aqui.md", &local, Expect::Absent),
            Err(Hitch::Changed(_))
        ));
        assert!(meter.about("nada/aqui.md").is_ok());
    }
}

#[test]
fn a_write_refused_on_the_look_before_it_is_charged_the_look_and_not_the_upload() {
    let home = tempfile::tempdir().unwrap();
    let fake = Fake::drive().with_chunk(4);
    let meter = Metered::over(&fake, home.path(), Class::History);
    let source = home.path().join("bytes");
    std::fs::write(&source, b"many more bytes than one chunk holds").unwrap();
    meter
        .put("attachments/ab/x-ab12cd34.png", &source, Expect::Absent)
        .unwrap();
    let after_landing = Spent::load(home.path()).units;

    assert!(matches!(
        meter.put("attachments/ab/x-ab12cd34.png", &source, Expect::Absent),
        Err(Hitch::Changed(_))
    ));
    assert!(matches!(
        meter.put(
            "attachments/cd/y-cd12ab34.png",
            &source,
            Expect::Revision("gone.1".into())
        ),
        Err(Hitch::Missing(_))
    ));

    let costs = fake.limits().costs;
    assert_eq!(
        Spent::load(home.path()).units - after_landing,
        2 * costs.checks,
        "a write that never sent a byte was charged as a whole upload"
    );
}
