use super::*;

#[test]
fn what_a_machine_left_behind_is_told_says_what_to_do_not_what_broke() {
    for spanish in [true, false] {
        for itself in [true, false] {
            let (said, yes, no) = behind_words(spanish, itself);
            assert!(!said.contains("schema"), "{said}");
            assert!(!said.chars().any(|one| one.is_ascii_digit()), "{said}");
            assert!(!yes.is_empty() && !no.is_empty());
            assert_eq!(
                itself,
                !said.contains("instaló") && !said.contains("installed"),
                "a copy something else updates has to be told so: {said}"
            );
        }
    }
    assert!(
        update::ours(RELEASES),
        "the offer has to lead where our releases live"
    );
}
