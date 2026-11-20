use super::*;

fn words(text: &str) -> Vec<String> {
    counted(&[text])
        .into_iter()
        .map(|(key, _, _)| key)
        .collect()
}

fn word(key: &str, written: &str, count: u32) -> Word {
    (key.to_string(), written.to_string(), count)
}

#[test]
fn function_words_and_short_ones_are_not_counted() {
    assert_eq!(
        words("el presupuesto de la casa y the budget"),
        ["presupuesto", "budget", "casa"]
    );
}

#[test]
fn words_for_how_a_text_is_laid_out_are_not_counted_in_either_language() {
    assert_eq!(
        words("La **negrita** y la cursiva, un Título de nivel 2, el número de la lista: Rina"),
        vec!["rina"]
    );
    assert_eq!(
        words("Bold and italic, a heading level, numbered lists, a table and a link: Roger"),
        vec!["roger"]
    );
    assert_eq!(
        words("cita veterinario quote"),
        vec!["veterinario", "quote", "cita"]
    );
}

#[test]
fn a_word_counts_by_its_letters_whatever_its_accents_and_case_and_keeps_its_first_spelling() {
    assert_eq!(
        counted(&["Método nuevo", "el metodo"]),
        [word("metodo", "método", 2), word("nuevo", "nuevo", 1)]
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
        ["sha256", "utf8", "mp3"]
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
    assert_eq!(all.len(), KEPT_AT_MOST);
    assert_eq!(all[0], word("often", "often", 3));
}

#[test]
fn what_is_counted_reads_back_the_same_and_its_spelling_is_kept_only_when_it_differs() {
    let all = counted(&["Presupuesto familiar presupuesto Método"]);
    let stored = said(&all);
    assert_eq!(stored, "presupuesto:2 familiar:1 metodo:1:método");
    assert_eq!(read(&stored), all);
    assert_eq!(read("broken anything:x fine:2"), [word("fine", "fine", 2)]);
}

#[test]
fn among_words_said_as_often_the_longer_ones_outlast_the_cut() {
    let short: Vec<String> = (0..KEPT_AT_MOST)
        .map(|mut n| {
            let mut word = String::from("a");
            for _ in 0..3 {
                word.push(char::from(b'a' + (n % 26) as u8));
                n /= 26;
            }
            word
        })
        .collect();
    let text = format!("{} veterinaria", short.join(" "));
    assert!(words(&text).contains(&"veterinaria".to_string()));
}

fn kept(text: &str) -> Vec<String> {
    let mut all = words(text);
    all.sort();
    all
}

#[test]
fn a_link_or_an_address_says_nothing() {
    assert_eq!(kept("Ver https://tisty.app/guia/roger hoy"), ["hoy", "ver"]);
    assert_eq!(kept("Ver www.tisty.app hoy"), ["hoy", "ver"]);
    assert_eq!(kept("Escribe a (roger@rina.cl) hoy"), ["escribe", "hoy"]);
}

#[test]
fn a_path_or_a_file_says_nothing() {
    assert_eq!(kept("Abre wizard/template.sh ahora"), ["abre", "ahora"]);
    assert_eq!(kept("Abre src/brain/alike ahora"), ["abre", "ahora"]);
    assert_eq!(kept("Abre C:\\Users\\rina ahora"), ["abre", "ahora"]);
    assert_eq!(kept("Abre ~/notas ahora"), ["abre", "ahora"]);
    assert_eq!(kept("Abre /etc ahora"), ["abre", "ahora"]);
    assert_eq!(kept("Abre openai.yaml ahora."), ["abre", "ahora"]);
}

#[test]
fn a_word_beside_a_slash_is_still_a_word() {
    assert_eq!(kept("entrada/salida"), ["entrada", "salida"]);
    assert_eq!(kept("Fin del paseo."), ["fin", "paseo"]);
    assert_eq!(kept("¿Paseo, correa?"), ["correa", "paseo"]);
}

#[test]
fn a_tag_an_emoji_or_an_entity_says_nothing() {
    assert_eq!(
        kept("Paseo #roger :dog: &nbsp; diario"),
        ["diario", "paseo"]
    );
}

#[test]
fn a_name_or_a_mark_from_code_says_nothing() {
    for code in [
        "kept_up",
        "TISTY_PROFILE",
        "--force",
        "a=b",
        "<div>",
        "{money}",
        "`span`",
        "$HOME",
        "a|b",
    ] {
        assert_eq!(
            kept(&format!("Usa {code} siempre")),
            ["siempre", "usa"],
            "{code}"
        );
    }
}
