use super::*;

fn root() -> Root {
    Root::from_bytes(&[7; 32])
}

#[test]
fn the_phrase_has_twenty_four_words_and_brings_back_the_same_person() {
    for words in [Words::English, Words::Spanish] {
        let said = phrase(&root(), words);

        assert_eq!(said.split(' ').count(), WORDS);
        let back = root_from(&said).unwrap();
        assert_eq!(back.person(), root().person());
    }
}

#[test]
fn the_two_languages_say_different_words_for_one_person() {
    assert_ne!(
        *phrase(&root(), Words::English),
        *phrase(&root(), Words::Spanish)
    );
}

#[test]
fn case_spacing_and_accents_do_not_matter() {
    let said = phrase(&root(), Words::Spanish);
    let copied = format!(
        "  {}  ",
        unmarked(&said.nfkd().collect::<String>()).to_uppercase()
    )
    .replace(' ', "   ");

    assert_eq!(root_from(&copied).unwrap().person(), root().person());
}

#[test]
fn every_spanish_word_stays_unique_without_its_accents() {
    let mut bare: Vec<String> = Language::Spanish
        .word_list()
        .iter()
        .map(|one| unmarked(&lowered(one)))
        .collect();
    bare.sort();
    bare.dedup();

    assert_eq!(bare.len(), 2048);
}

#[test]
fn a_short_phrase_says_how_many_words_it_has() {
    let said = phrase(&root(), Words::English);
    let short: Vec<&str> = said.split(' ').take(12).collect();

    assert_eq!(root_from(&short.join(" ")).err(), Some(Unread::Count(12)));
}

#[test]
fn an_unknown_word_says_where_it_is() {
    let said = phrase(&root(), Words::English);
    let mut words: Vec<&str> = said.split(' ').collect();
    words[4] = "tisty";

    assert_eq!(root_from(&words.join(" ")).err(), Some(Unread::Word(5)));
}

/// One swap in 256 adds up by chance; none of them may bring back the same person.
#[test]
fn two_words_swapped_never_bring_back_the_person() {
    let said = phrase(&root(), Words::Spanish);
    let words: Vec<&str> = said.split(' ').collect();
    let mut caught = 0;
    for other in (1..WORDS).filter(|&at| words[at] != words[0]) {
        let mut swapped = words.clone();
        swapped.swap(0, other);
        match root_from(&swapped.join(" ")) {
            Err(unread) => {
                assert_eq!(unread, Unread::Checksum);
                caught += 1;
            }
            Ok(back) => assert_ne!(back.person(), root().person()),
        }
    }

    assert!(caught > 0, "no swap was caught by the checksum");
}
