use super::*;
use crate::fakes::Fake;

#[test]
fn two_meters_spending_at_once_both_end_up_counted() {
    let home = tempfile::tempdir().unwrap();
    let fake = Fake::drive();
    let round = Metered::over(&fake, home.path(), Class::History);
    let glance = Metered::over(&fake, home.path(), Class::History);

    round.list("").unwrap();
    glance.list("").unwrap();
    glance.list("").unwrap();
    drop(glance);
    drop(round);

    let per_list = fake.limits().costs.list;
    assert_eq!(
        Spent::load(home.path()).units,
        3 * per_list,
        "a meter that saved last wrote over what the other had spent"
    );
}
