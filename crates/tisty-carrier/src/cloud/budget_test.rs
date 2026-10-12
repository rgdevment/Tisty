use super::*;
use crate::fakes::Fake;

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
