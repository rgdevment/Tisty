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
    repeat: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    certainty: Option<String>,
    offer: Option<String>,
    offer_title: Option<String>,
    #[allow(dead_code)]
    why: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Cases {
    case: Vec<Case>,
}

fn said(repeat: tisty_core::model::Repeat) -> String {
    use tisty_core::model::{From, Unit};
    let (from, c) = match repeat.from {
        From::Due => ("due", repeat.each),
        From::Done => ("done", repeat.each),
    };
    let unit = match c.unit {
        Unit::Day => "d",
        Unit::Week => "w",
        Unit::Month => "m",
        Unit::Year => "y",
    };
    match repeat.until {
        Some(last) => format!("{from}:{}{unit} until {last}", c.every),
        None => format!("{from}:{}{unit}", c.every),
    }
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

fn now() -> jiff::Zoned {
    "2026-08-05T09:00:00[America/Santiago]".parse().unwrap()
}

#[derive(Default)]
struct Report {
    passed: usize,
    failures: Vec<String>,
}

#[test]
fn the_parser_matches_every_case() {
    let mut report = Report::default();

    for (locale, case) in every_case() {
        let got = tisty_nl::parse(&case.input, &now(), locale);
        let mut wrong = Vec::new();

        if got.title != case.title {
            wrong.push(format!("title: {:?} ≠ {:?}", got.title, case.title));
        }

        let expect_date = case
            .date
            .as_ref()
            .map(|d| d.parse::<jiff::civil::Date>().unwrap());
        let got_date = got.date.as_ref().map(|d| d.date());
        if got_date != expect_date {
            wrong.push(format!("date: {got_date:?} ≠ {expect_date:?}"));
        }

        let expect_deadline = case
            .deadline
            .as_ref()
            .map(|d| d.parse::<jiff::civil::Date>().unwrap());
        let got_deadline = got.deadline.as_ref().map(|d| d.date());
        if got_deadline != expect_deadline {
            wrong.push(format!("deadline: {got_deadline:?} ≠ {expect_deadline:?}"));
        }

        let expect_time = case
            .time
            .as_ref()
            .map(|t| t.parse::<jiff::civil::Time>().unwrap());
        let got_time = got
            .date
            .as_ref()
            .filter(|d| d.has_time)
            .map(|d| d.at.time());
        if got_time != expect_time {
            wrong.push(format!("time: {got_time:?} ≠ {expect_time:?}"));
        }

        let expect_repeat = case.repeat.as_deref();
        let got_repeat = got.repeat.map(said);
        assert_eq!(
            got_repeat.as_deref(),
            expect_repeat,
            "«{}» repeat
  {}",
            case.input,
            case.why.as_deref().unwrap_or("")
        );

        let expect_priority = case
            .priority
            .map(|p| u8::from(tisty_core::Priority::try_from(p).unwrap()));
        let got_priority = got.priority.map(u8::from);
        if got_priority != expect_priority {
            wrong.push(format!("priority: {got_priority:?} ≠ {expect_priority:?}"));
        }

        let got_tags: Vec<String> = got.tags.iter().map(|t| t.to_string()).collect();
        if got_tags != case.tags {
            wrong.push(format!("tags: {got_tags:?} ≠ {:?}", case.tags));
        }

        let expect_certainty = (case.date.is_some() || case.deadline.is_some())
            .then(|| case.certainty.as_deref().unwrap_or("sure"));
        let got_certainty = got
            .spans
            .iter()
            .find(|s| {
                matches!(s.mark, tisty_nl::Mark::Date | tisty_nl::Mark::Deadline)
                    || (matches!(s.mark, tisty_nl::Mark::Repeat) && got.date.is_some())
            })
            .map(|s| match s.certainty {
                tisty_nl::Certainty::Sure => "sure",
                tisty_nl::Certainty::Assumed => "assumed",
            });
        let expect_offer = case
            .offer
            .as_ref()
            .map(|d| d.parse::<jiff::civil::Date>().unwrap());
        let got_offer = got.offers.first().map(|o| o.date.date());
        if got_offer != expect_offer {
            wrong.push(format!("offer: {got_offer:?} ≠ {expect_offer:?}"));
        }

        if let Some(title) = &case.offer_title {
            let got_title = got.offers.first().map(|o| o.title.as_str());
            if got_title != Some(title.as_str()) {
                wrong.push(format!("offer title: {got_title:?} ≠ {title:?}"));
            }
        }

        if got_certainty != expect_certainty {
            wrong.push(format!(
                "certainty: {got_certainty:?} ≠ {expect_certainty:?}"
            ));
        }

        if wrong.is_empty() {
            report.passed += 1;
        } else {
            report.failures.push(format!(
                "  {locale}  «{}»\n       {}",
                case.input,
                wrong.join("\n       ")
            ));
        }
    }

    let total = report.passed + report.failures.len();
    if !report.failures.is_empty() {
        panic!(
            "{}/{} cases pass\n\n{}\n",
            report.passed,
            total,
            report.failures.join("\n")
        );
    }
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
fn assumptions_explain_themselves() {
    for (locale, case) in every_case() {
        let Some(said) = &case.certainty else {
            continue;
        };
        assert!(
            said == "sure" || said == "assumed",
            "{locale}: «{}» has an unknown certainty: {said}",
            case.input
        );
        assert!(
            case.date.is_some() || case.deadline.is_some(),
            "{locale}: «{}» states a certainty about nothing",
            case.input
        );
        assert!(
            said == "sure" || case.why.is_some(),
            "{locale}: «{}» guesses a date without explaining why",
            case.input
        );
    }
}

#[test]
fn both_locales_cover_the_same_ground() {
    type Kind = (&'static str, fn(&Case) -> bool);
    let kinds: [Kind; 8] = [
        ("a date", |c| c.date.is_some()),
        ("a deadline", |c| c.deadline.is_some()),
        ("a clock", |c| c.time.is_some()),
        ("a priority", |c| c.priority.is_some()),
        ("tags", |c| !c.tags.is_empty()),
        ("an offer", |c| c.offer.is_some()),
        ("a guess", |c| c.certainty.as_deref() == Some("assumed")),
        ("nothing taken", |c| {
            c.date.is_none() && c.deadline.is_none() && c.priority.is_none() && c.tags.is_empty()
        }),
    ];

    for locale in ["es", "en"] {
        let cases = load(locale);
        for (what, holds) in kinds {
            assert!(cases.iter().any(holds), "{locale} has no case with {what}");
        }
    }
}

#[test]
fn offers_only_exist_where_nothing_was_taken() {
    for (locale, case) in every_case() {
        if case.offer.is_some() {
            assert!(
                case.date.is_none() && case.deadline.is_none(),
                "{locale}: «{}» offers a date it already took",
                case.input
            );
        }
        assert!(
            case.offer_title.is_none() || case.offer.is_some(),
            "{locale}: «{}» names an offer title without an offer",
            case.input
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

#[test]
fn every_span_lands_on_a_letter_boundary() {
    for locale in ["es", "en"] {
        for case in load(locale) {
            let read = tisty_nl::parse(&case.input, &now(), locale);
            let letters = case.input.chars().count();
            for span in &read.spans {
                assert!(
                    span.from <= span.to && span.to <= letters,
                    "{locale} «{}»: {}..{} is outside {letters} letters",
                    case.input,
                    span.from,
                    span.to
                );
            }
        }
    }
}

#[test]
fn no_two_spans_ever_overlap() {
    for locale in ["es", "en"] {
        for case in load(locale) {
            let read = tisty_nl::parse(&case.input, &now(), locale);
            let mut ranges: Vec<_> = read.spans.iter().map(|s| (s.from, s.to)).collect();
            ranges.sort_unstable();
            for pair in ranges.windows(2) {
                assert!(
                    pair[0].1 <= pair[1].0,
                    "{locale} «{}»: {:?} overlaps {:?}",
                    case.input,
                    pair[0],
                    pair[1]
                );
            }
        }
    }
}
