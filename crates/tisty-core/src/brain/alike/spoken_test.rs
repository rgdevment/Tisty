use super::*;

#[test]
fn the_little_words_tell_spanish_from_english() {
    assert_eq!(guessed(&["Pagar la cuenta del agua"]), Some(SPANISH));
    assert_eq!(guessed(&["Pay the water bill"]), Some(ENGLISH));
}

#[test]
fn every_text_of_a_task_counts() {
    assert_eq!(
        guessed(&["Router", "Call the company", "and ask for a new one"]),
        Some(ENGLISH)
    );
}

#[test]
fn a_tie_is_spanish_only_when_it_carries_a_spanish_mark() {
    assert_eq!(guessed(&["Revisión anual"]), Some(SPANISH));
    assert_eq!(guessed(&["¿Router?"]), Some(SPANISH));
    assert_eq!(guessed(&["Router"]), None);
    assert_eq!(guessed(&[]), None);
}

#[test]
fn capitals_do_not_hide_a_little_word() {
    assert_eq!(guessed(&["THE ROUTER"]), Some(ENGLISH));
    assert_eq!(guessed(&["EL ROUTER"]), Some(SPANISH));
}
