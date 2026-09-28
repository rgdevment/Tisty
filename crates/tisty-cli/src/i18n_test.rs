use super::*;

#[test]
fn every_locale_parses() {
    for (name, _) in LOCALES {
        assert!(catalog(name).is_some(), "{name} failed to parse");
    }
}

#[test]
fn every_locale_defines_the_same_keys() {
    let reference = catalog(FALLBACK).unwrap();

    for (name, _) in LOCALES {
        let c = catalog(name).unwrap();
        for key in reference.messages.keys() {
            assert!(c.messages.contains_key(key), "{name} is missing «{key}»");
        }
        for key in reference.weekday.keys() {
            assert!(
                c.weekday.contains_key(key),
                "{name} is missing weekday «{key}»"
            );
        }
        for key in reference.month.keys() {
            assert!(c.month.contains_key(key), "{name} is missing month «{key}»");
        }
        for key in reference.plural.keys() {
            assert!(
                c.plural.contains_key(key),
                "{name} is missing plural «{key}»"
            );
        }
        for key in c.messages.keys() {
            assert!(
                reference.messages.contains_key(key),
                "{name} defines «{key}», which {FALLBACK} does not"
            );
        }
    }
}

#[test]
fn placeholders_match_the_reference() {
    let reference = catalog(FALLBACK).unwrap();

    for (name, _) in LOCALES {
        let c = catalog(name).unwrap();
        for (key, text) in &reference.messages {
            let expected = placeholders(text);
            let actual = placeholders(&c.messages[key]);
            assert_eq!(expected, actual, "{name}: «{key}» has wrong placeholders");
        }
    }
}

fn placeholders(text: &str) -> Vec<String> {
    let mut found: Vec<String> = text
        .split('{')
        .skip(1)
        .filter_map(|s| s.split('}').next())
        .map(str::to_string)
        .collect();
    found.sort();
    found
}

#[test]
fn english_is_the_fallback() {
    assert_eq!(Lang::choose(None, None).code(), "en");
    assert_eq!(Lang::choose(None, Some("fr_FR.UTF-8")).code(), "en");
    assert_eq!(Lang::from_code("fr_FR.UTF-8").code(), "en");
}

#[test]
fn a_locale_is_detected_from_any_variant() {
    for code in ["es", "es_CL.UTF-8", "ES_ES", "es-419"] {
        assert_eq!(Lang::from_code(code).code(), "es", "{code}");
    }
}

#[test]
fn an_unsupported_first_choice_falls_to_the_next_one_tisty_speaks() {
    let codes = |list: &[&str]| list.iter().map(|s| s.to_string()).collect();

    assert_eq!(
        first_spoken(codes(&["fr-FR", "es-ES", "en-US"])),
        Some("es-ES".into())
    );
    assert_eq!(
        first_spoken(codes(&["en-GB", "es-CL"])),
        Some("en-GB".into())
    );
    assert_eq!(first_spoken(codes(&["fr-FR", "de-DE"])), None);
    assert_eq!(first_spoken(Vec::new()), None);
}

#[test]
fn configured_locale_wins_over_the_environment() {
    assert_eq!(Lang::choose(Some("es"), Some("en_US.UTF-8")).code(), "es");
    assert_eq!(Lang::choose(Some("en"), Some("es_CL.UTF-8")).code(), "en");
    assert_eq!(Lang::choose(None, Some("es_CL.UTF-8")).code(), "es");
}

#[test]
fn plurals_agree_in_every_locale() {
    assert_eq!(Lang::from_code("en").plural("tasks", 1), "1 task");
    assert_eq!(Lang::from_code("en").plural("tasks", 3), "3 tasks");
    assert_eq!(Lang::from_code("es").plural("tasks", 1), "1 tarea");
    assert_eq!(Lang::from_code("es").plural("tasks", 3), "3 tareas");
}

#[test]
fn day_and_month_names_are_localised() {
    assert_eq!(Lang::from_code("en").weekday(1), "mon");
    assert_eq!(Lang::from_code("es").weekday(1), "lun");
    assert_eq!(Lang::from_code("en").month(8), "aug");
    assert_eq!(Lang::from_code("es").month(8), "ago");
}

#[test]
fn a_missing_key_is_visible_not_fatal() {
    assert_eq!(Lang::from_code("en").get("no-such-key"), "⟨?⟩");
}

#[test]
fn arguments_are_substituted() {
    let out = Lang::from_code("en").fill("not-found", &[("selector", "abc")]);
    assert!(out.contains("abc"), "{out}");
    assert!(!out.contains('{'), "{out}");
}

#[test]
fn filters_are_accepted_in_any_language() {
    assert_eq!(canonical_filter("hoy"), canonical_filter("today"));
    assert_eq!(canonical_filter("hechas"), canonical_filter("done"));
    assert_eq!(canonical_filter("TODAS"), Some("all"));
    assert_eq!(canonical_filter("nonsense"), None);
}
