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
