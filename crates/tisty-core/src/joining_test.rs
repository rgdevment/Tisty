use super::*;

const READABLE: &str = "attachments/revision-diseno-a42d4603.mp4";
const REPEATED: &str = "attachments/ttkjw8qofb-a42d4603-a42d4603-a42d4603.mp4";

fn copy(at: &str, links: usize, shelved: bool, since: i64) -> Copy {
    Copy {
        at: at.to_string(),
        links,
        shelved,
        since,
    }
}

#[test]
fn the_copy_no_old_shelf_names_stays_before_anything_else() {
    let copies = [copy(READABLE, 3, true, 1), copy(REPEATED, 1, false, 9)];

    assert_eq!(keeper(&copies), 1);
}

#[test]
fn a_readable_name_stays_over_one_that_repeats_its_stamp() {
    let copies = [copy(REPEATED, 5, false, 1), copy(READABLE, 1, false, 9)];

    assert_eq!(keeper(&copies), 1);
}

#[test]
fn among_names_alike_the_most_linked_then_the_oldest_stays() {
    let one = "attachments/acta-a42d4603.pdf";
    let other = "attachments/minuta-a42d4603.pdf";

    assert_eq!(
        keeper(&[copy(one, 1, false, 1), copy(other, 2, false, 9)]),
        1
    );
    assert_eq!(
        keeper(&[copy(one, 2, false, 9), copy(other, 2, false, 1)]),
        1
    );
}

#[test]
fn every_form_of_a_link_is_pointed_at_the_copy_that_stays() {
    let from = "foto-a42d4603.png";
    let to = "playa-a42d4603.png";
    let body = format!(
        "![a]({from}) ![b](<{from}>) [c](attachments/{from}?x=1)\n\
         [d]: attachments/b7/{from}\n<img src=\"{from}\"> el archivo {from}."
    );

    let (now, moved) = relinked(&body, from, to).unwrap();

    assert_eq!(moved, 6);
    assert!(!now.contains(from), "{now}");
    assert_eq!(now.matches(to).count(), 6, "{now}");
    assert!(now.contains(&format!("attachments/b7/{to}")));
    assert!(now.contains(&format!("(<{to}>)")));
    assert!(now.contains(&format!("{to}?x=1")));
}

#[test]
fn a_longer_name_that_holds_the_one_that_goes_is_left_alone() {
    let from = "foto-a42d4603.png";
    let body = format!("![a](mi{from}) ![b]({from}.part) ![c](x-{from})");

    assert_eq!(relinked(&body, from, "playa-a42d4603.png"), None);
}

#[test]
fn a_link_through_an_old_shelf_is_told_from_a_flat_one() {
    let leaf = "foto-a42d4603.png";

    assert!(shelved(&format!("![a](attachments/b7/{leaf})"), leaf));
    assert!(!shelved(
        &format!("![a](attachments/{leaf}) ![b]({leaf})"),
        leaf
    ));
}

#[test]
fn several_copies_that_go_are_pointed_at_the_one_that_stays_and_counted() {
    let body = "![a](uno-a42d4603.png) ![b](dos-a42d4603.png) ![c](uno-a42d4603.png)";
    let going = [
        (
            "uno-a42d4603.png".to_string(),
            "playa-a42d4603.png".to_string(),
        ),
        (
            "dos-a42d4603.png".to_string(),
            "playa-a42d4603.png".to_string(),
        ),
    ];

    let (now, moved) = relinked_all(body, &going).unwrap();

    assert_eq!(moved, 3);
    assert_eq!(now.matches("playa-a42d4603.png").count(), 3);
}
