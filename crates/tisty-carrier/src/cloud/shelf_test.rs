use super::*;

#[test]
fn a_full_drive_stops_the_attachments_without_stopping_the_round() {
    for fake in crate::fakes::every() {
        let remote: &dyn Remote = &*fake;
        let mut index = Index::default();
        let mut shelf = Shelf::over(remote, &mut index);

        let said = shelf.refuse(Hitch::Full, "attachments/ab/foto.jpg", Vec::new());

        assert!(said.is_ok(), "{}", fake.who());
        assert!(
            matches!(shelf.full(), Some(Trouble::CloudFull { .. })),
            "{}: {:?}",
            fake.who(),
            shelf.full()
        );
        assert!(shelf.paused().is_none(), "{}", fake.who());
    }
}
