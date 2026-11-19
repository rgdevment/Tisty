use super::*;

fn words(text: &str) -> Vec<String> {
    counted(&[text]).into_iter().map(|(word, _)| word).collect()
}

#[test]
fn function_words_and_short_ones_are_not_counted() {
    assert_eq!(
        words("el presupuesto de la casa y the budget"),
        ["budget", "casa", "presupuesto"]
    );
}

#[test]
fn a_word_counts_by_its_letters_whatever_its_accents_and_case() {
    assert_eq!(
        counted(&["Método nuevo", "el metodo"]),
        [("método".to_string(), 2), ("nuevo".to_string(), 1)]
    );
}

#[test]
fn a_word_is_three_letters_to_thirty_two() {
    let longest = "a".repeat(32);
    let too_long = "b".repeat(33);
    assert_eq!(
        words(&format!("ab abc {longest} {too_long}")),
        [longest, "abc".to_string()]
    );
}

#[test]
fn numbers_and_ids_are_not_words_but_short_names_with_digits_are() {
    assert_eq!(
        words("2026 4nnctnme 01m7n1bnd01pqamywacmfj5p0g mp3 sha256 utf8"),
        ["mp3", "sha256", "utf8"]
    );
}

#[test]
fn the_most_frequent_words_are_kept_when_there_are_too_many() {
    let lettered = |mut n: usize| {
        let mut word = String::from("w");
        for _ in 0..3 {
            word.push(char::from(b'a' + (n % 26) as u8));
            n /= 26;
        }
        word
    };
    let many: Vec<String> = (0..KEPT_AT_MOST + 10).map(lettered).collect();
    let mut text = many.join(" ");
    text.push_str(" often often often");
    let all = counted(&[&text]);
    assert!(all.len() <= KEPT_AT_MOST);
    assert_eq!(all[0], ("often".to_string(), 3));
}

#[test]
fn what_is_counted_reads_back_the_same() {
    let all = counted(&["presupuesto familiar presupuesto"]);
    assert_eq!(read(&said(&all)), all);
    assert_eq!(read("broken anything:x fine:2"), [("fine".to_string(), 2)]);
}
