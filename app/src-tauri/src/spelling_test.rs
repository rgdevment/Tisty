use super::{spellable, tongue_chosen};

fn have(tags: &[&str]) -> Vec<String> {
    tags.iter().map(|one| (*one).to_owned()).collect()
}

#[test]
fn the_document_is_read_in_its_own_language_when_the_system_has_it() {
    let supported = have(&["en-US", "es-ES", "es-MX"]);
    assert_eq!(
        tongue_chosen(&supported, Some("es-MX"), Some("en-US")).as_deref(),
        Some("es-MX")
    );
}

#[test]
fn a_bare_language_takes_the_persons_own_variant_of_it_first() {
    let supported = have(&["en-US", "es-ES", "es-CL"]);
    assert_eq!(
        tongue_chosen(&supported, Some("es"), Some("es-CL")).as_deref(),
        Some("es-CL")
    );
    assert_eq!(
        tongue_chosen(&supported, Some("es"), Some("en-US")).as_deref(),
        Some("es-ES"),
        "sin variante propia, la primera de la familia"
    );
}

#[test]
fn a_language_the_system_lacks_is_never_read_with_another_ones_speller() {
    let supported = have(&["en-US", "fr-FR"]);
    assert_eq!(
        tongue_chosen(&supported, Some("de-DE"), Some("fr-CA")),
        None
    );
    assert_eq!(tongue_chosen(&supported, Some("es"), Some("en-US")), None);
    assert_eq!(tongue_chosen(&supported, None, None), None);
    assert_eq!(tongue_chosen(&[], Some("es"), Some("es-ES")), None);
}

#[test]
fn with_no_language_asked_the_persons_own_is_read() {
    let supported = have(&["en-US", "fr-FR"]);
    assert_eq!(
        tongue_chosen(&supported, None, Some("fr-CA")).as_deref(),
        Some("fr-FR")
    );
}

#[test]
fn only_one_word_of_reasonable_length_is_spelled() {
    assert_eq!(spellable("  prueva ").as_deref(), Some("prueva"));
    assert_eq!(spellable(""), None);
    assert_eq!(spellable("dos palabras"), None);
    assert_eq!(spellable(&"a".repeat(65)), None);
}
