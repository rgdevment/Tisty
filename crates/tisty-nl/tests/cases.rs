//! Until a parser exists this guards the contract itself: a malformed or
//! duplicated case is a silent hole in the suite it will be measured against.

use std::collections::BTreeSet;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Case {
    input: String,
    title: String,
    date: Option<String>,
    time: Option<String>,
    deadline: Option<String>,
    priority: Option<u8>,
    #[serde(default)]
    tags: Vec<String>,
    #[allow(dead_code)]
    why: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Cases {
    case: Vec<Case>,
}

fn load(locale: &str) -> Vec<Case> {
    let path = format!("{}/tests/cases/{locale}.toml", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    toml::from_str::<Cases>(&text)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .case
}

fn every_case() -> Vec<(&'static str, Case)> {
    ["es", "en"]
        .into_iter()
        .flat_map(|l| load(l).into_iter().map(move |c| (l, c)))
        .collect()
}

#[test]
fn both_locales_have_cases() {
    for locale in ["es", "en"] {
        assert!(load(locale).len() >= 20, "{locale} needs more cases");
    }
}

#[test]
fn no_case_is_written_twice() {
    for locale in ["es", "en"] {
        let cases = load(locale);
        let unique: BTreeSet<_> = cases.iter().map(|c| &c.input).collect();
        assert_eq!(unique.len(), cases.len(), "{locale} has a duplicated input");
    }
}

#[test]
fn dates_and_times_are_well_formed() {
    for (locale, case) in every_case() {
        for (field, value) in [("date", &case.date), ("deadline", &case.deadline)] {
            if let Some(v) = value {
                assert!(
                    v.parse::<jiff::civil::Date>().is_ok(),
                    "{locale}: «{}» has an invalid {field}: {v}",
                    case.input
                );
            }
        }
        if let Some(t) = &case.time {
            assert!(
                t.parse::<jiff::civil::Time>().is_ok(),
                "{locale}: «{}» has an invalid time: {t}",
                case.input
            );
        }
    }
}

#[test]
fn priorities_are_in_range() {
    for (locale, case) in every_case() {
        if let Some(p) = case.priority {
            assert!(
                (1..=4).contains(&p),
                "{locale}: «{}» has priority {p}",
                case.input
            );
        }
    }
}

#[test]
fn tags_are_already_normalised() {
    for (locale, case) in every_case() {
        for tag in &case.tags {
            assert_eq!(
                tag,
                &tisty_core::Tag::new(tag).unwrap().to_string(),
                "{locale}: «{}» expects a tag that is not normalised",
                case.input
            );
        }
    }
}

#[test]
fn no_title_is_longer_than_its_input() {
    for (locale, case) in every_case() {
        assert!(
            case.title.chars().count() <= case.input.chars().count(),
            "{locale}: «{}» expects a longer title than the input",
            case.input
        );
    }
}

#[test]
fn ambiguous_cases_are_represented() {
    for locale in ["es", "en"] {
        let untouched = load(locale)
            .iter()
            .filter(|c| {
                c.title == c.input.trim()
                    && c.date.is_none()
                    && c.deadline.is_none()
                    && c.priority.is_none()
                    && c.tags.is_empty()
            })
            .count();
        assert!(
            untouched >= 2,
            "{locale} needs cases where nothing is consumed"
        );
    }
}

#[test]
fn deadline_cases_explain_themselves() {
    for (locale, case) in every_case() {
        if case.deadline.is_some() {
            assert!(
                case.why.is_some(),
                "{locale}: «{}» sets a deadline without explaining why",
                case.input
            );
        }
    }
}
