use super::*;
use crate::refusing::{Behind, RELEASES, behind_words};

const STRANGER: &str = "dev_f0ztyvwj";

#[test]
fn what_a_machine_left_behind_is_told_says_what_to_do_not_what_broke() {
    for spanish in [true, false] {
        for itself in [true, false] {
            for written_by in ["", STRANGER] {
                let (said, yes, no) = behind_words(Behind {
                    spanish,
                    itself,
                    written_by,
                    from_the_store: false,
                });
                let plain = said.replace(STRANGER, "");
                assert!(!said.contains("schema"), "{said}");
                assert!(!plain.chars().any(|one| one.is_ascii_digit()), "{said}");
                assert!(!yes.is_empty() && !no.is_empty());
                assert_eq!(
                    itself,
                    !said.contains("instaló") && !said.contains("installed"),
                    "a copy something else updates has to be told so: {said}"
                );
            }
        }
    }
    assert!(
        update::ours(RELEASES),
        "the offer has to lead where our releases live"
    );
}

#[test]
fn the_machine_that_wrote_ahead_is_named() {
    for spanish in [true, false] {
        let (said, ..) = behind_words(Behind {
            spanish,
            itself: true,
            written_by: STRANGER,
            from_the_store: false,
        });
        assert!(said.contains(&format!("«{STRANGER}»")), "{said}");
        assert!(!said.contains("Store"), "{said}");
    }
}

#[test]
fn a_store_copy_is_told_another_install_on_this_computer_may_have_written_ahead() {
    for spanish in [true, false] {
        let (said, ..) = behind_words(Behind {
            spanish,
            itself: false,
            written_by: STRANGER,
            from_the_store: true,
        });
        assert!(said.contains("Microsoft Store"), "{said}");
        assert!(said.contains(&format!("«{STRANGER}»")), "{said}");

        let (unnamed, ..) = behind_words(Behind {
            spanish,
            itself: false,
            written_by: "",
            from_the_store: true,
        });
        assert!(
            !unnamed.contains("Microsoft Store"),
            "without a machine to name there is nothing to point at: {unnamed}"
        );
    }
}
