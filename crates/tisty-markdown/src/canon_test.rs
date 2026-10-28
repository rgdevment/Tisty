use super::canonical;

// One case per line: the column's type, what was typed, then what the file gets.
const CASES: &str = "
number          | 6490             => 6490
number          | 1,5              => 1.5
number          |  12.25           => 12.25
number          | 6,490            => 6,490
number          | 1 500            => 1500
number          | mucho            => mucho
money CLP       | $6.490           => 6490
money CLP       | 6490             => 6490
money CLP       | CLP 1.234.567    => 1234567
money CLP       | 6.49             => 6.49
money USD       | US$6,490.50      => 6490.50
money USD       | 12.5             => 12.5
money EUR       | 6.490,50 €       => 6490.50
money EUR       | 3,5              => 3.5
money CHF       | CHF 1'234.00     => 1234.00
money           | 1,5              => 1.5
money           | $10              => 10
percent         | 45%              => 45
percent         | 12,5 %           => 12.5
duration        | 1 h 30 min       => 1h30m
duration        | 1h30             => 1h30m
duration        | 90 min           => 1h30m
duration        | 1:30             => 1h30m
duration        | 2h               => 2h
duration        | 45               => 45m
duration        | un rato          => un rato
rating          | 4                => 4
rating          | ★★★☆☆            => 3
rating          | 9                => 9
check           | sí               => [x]
check           | X                => [x]
check           | no               => [ ]
check           | quizás           => quizás
url             | spotify.com      => https://spotify.com
url             | http://a.cl      => http://a.cl
tags            | web,docs, Web    => web, docs
date            | 2026-11-14       => 2026-11-14
text            |   Hola           => Hola
select          | Música           => Música
";

#[test]
fn what_is_typed_lands_in_the_form_its_column_writes() {
    for line in CASES.lines().filter(|one| !one.trim().is_empty()) {
        let (asked, expected) = line.split_once("=>").expect(line);
        let (words, typed) = asked.split_once('|').expect(line);
        assert_eq!(canonical(words.trim(), typed), expected.trim(), "{line}");
    }
}

#[test]
fn a_column_without_a_type_keeps_what_was_typed() {
    assert_eq!(canonical("", "  6.490  "), "6.490");
    assert_eq!(canonical("nonsense", "x"), "x");
}
