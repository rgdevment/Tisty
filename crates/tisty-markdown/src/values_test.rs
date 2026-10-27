use super::*;
use crate::{Kind, read};

// One cell per line, `-` for words drawn as written and `""` for nothing drawn, so a new case is a new line.
const CELLS: &str = r#"
number      | 6490                      => -              | 6490
number      | 1.5                       => -              | 1.5
number      | -3.25                     => -              | -3.25
number      | 6.490                     => -              | 6.490
number      | 6,490                     => amiss
number      | 1e3                       => amiss
number      | .5                        => amiss
number      | 5.                        => amiss
number      | 1234567890123456789       => amiss
number      | 0.1234567890              => amiss
money CLP   | 6490                      => $6.490         | 6490
money CLP   | 1234567                   => $1.234.567     | 1234567
money CLP   | -1500                     => -$1.500        | -1500
money CLP   | 6.490                     => amiss
money USD   | 6490                      => US$6,490.00    | 6490
money USD   | 12.5                      => US$12.50       | 12.5
money EUR   | 1234.5                    => 1.234,50 €     | 1234.5
money CHF   | 1234567                   => CHF 1'234'567.00 | 1234567
money JPY   | 300                       => ¥300           | 300
money XYZ   | 10                        => XYZ 10.00      | 10
money       | 10                        => 10.00          | 10
money       | 1234.5                    => 1234.50        | 1234.5
money       | 1.234                     => amiss
percent     | 45                        => 45%            | 45
percent     | 12.5                      => 12.5%          | 12.5
percent     | 45%                       => amiss
date        | 2026-11-14                => -              | 2026-11-14
date        | 2024-02-29                => -              | 2024-02-29
date        | 2026-02-29                => amiss
date        | 2026-13-01                => amiss
date        | 14/11/2026                => amiss
date        | 2026-1-14                 => amiss
duration    | 1h30m                     => 1 h 30 min     | 90
duration    | 2h                        => 2 h            | 120
duration    | 45m                       => 45 min         | 45
duration    | 90m                       => 1 h 30 min     | 90
duration    | 0m                        => 0 min          | 0
duration    | 1h30                      => amiss
duration    | h                         => amiss
duration    | 1 h                       => amiss
check       | [x]                       => ✓              | 1
check       | [X]                       => ✓              | 1
check       | [ ]                       => ""             | 0
check       | sí                        => amiss
rating      | 4                         => ★★★★☆          | 4
rating      | 0                         => ☆☆☆☆☆          | 0
rating      | 6                         => amiss
rating      | 4.5                       => amiss
url         | https://spotify.com       => -              | https://spotify.com
url         | spotify.com               => amiss
url         | https://a b               => amiss
email       | ana@correo.cl             => -              | ana@correo.cl
email       | ana@                      => amiss
email       | ana@correo                => amiss
phone       | +56 9 1234 5678           => -              | 56912345678
phone       | llamar                    => amiss
link        | [[Contrato de arriendo]]  => -              | contrato de arriendo
link        | Contrato                  => amiss
link        | ![Contrato](tisty:doc/01JC4M) => -       | contrato
link        | [Contrato](tisty:doc/01JC4M)  => -       | contrato
link        | [Contrato](https://x.cl)      => amiss
link        | [Contrato](tisty:doc/)        => amiss
select      | Música                    => -              | música
tags        | web, docs                 => -              | web, docs
text        | Hola                      => -              | hola
"#;

fn holds(words: &str) -> Holds {
    crate::typed::understood(words).expect(words).0
}

fn parts(seen: &Seen) -> (Option<&str>, Option<&str>, bool) {
    (seen.shown.as_deref(), seen.key.as_deref(), seen.amiss)
}

#[test]
fn each_cell_reads_as_its_column_says_and_never_is_refused() {
    for line in CELLS.lines().filter(|one| !one.trim().is_empty()) {
        let (cell, outcome) = line.split_once("=>").expect(line);
        let (words, said) = cell.split_once('|').expect(line);
        let seen = seen(&holds(words.trim()), said.trim());
        let outcome = outcome.trim();
        if outcome == "amiss" {
            assert_eq!(parts(&seen), (None, None, true), "{line}");
            continue;
        }
        let (shown, key) = outcome.split_once('|').expect(line);
        let shown = match shown.trim() {
            "-" => None,
            r#""""# => Some(""),
            one => Some(one),
        };
        assert_eq!(parts(&seen), (shown, Some(key.trim()), false), "{line}");
    }
}

#[test]
fn an_empty_cell_is_neither_a_value_nor_amiss() {
    assert_eq!(seen(&Holds::Number, ""), Seen::default());
}

fn table_of(text: &str) -> Read {
    let doc = read(text);
    let table = &doc.blocks()[0];
    let Kind::Table { columns } = &table.kind else {
        panic!("not a table: {text}");
    };
    super::read(columns, table, text)
}

// One column per line, its cells split by `;`, so a new case is a new line.
const FEET: &str = "
money CLP sum   | 6490; 9990; 7990         => $24.470
money CLP avg   | 6490; 9990; 7990         => $8.157
money CLP min   | 6490; 9990; 7990         => $6.490
money USD avg   | 1; 2                     => US$1.50
number sum      | 1.5; 2; ; x              => 3.5
number avg      | 1; 2                     => 1.5
number avg      | 1; 2; 2                  => 1.67
number avg      | 2; 4                     => 3
number min      | 3; -1; 2                 => -1
number max      | 3; -1; 2.5               => 3
number count    | 1; ; x                   => 2
number sum      | ; x                      =>
percent avg     | 40; 50                   => 45%
duration sum    | 1h30m; 45m               => 2 h 15 min
duration avg    | 1h; 2h                   => 1 h 30 min
duration max    | 1h; 2h                   => 2 h
date min        | 2026-11-14; 2026-03-01   => 2026-03-01
date max        | 2026-11-14; 2026-03-01   => 2026-11-14
check count     | [x]; [ ]; [x]            => 2
rating avg      | 5; 4                     => 4.5
rating max      | 3; 4                     => ★★★★☆
rating sum      | 3; 4                     => 7
text count      | a; ; b                   => 2
number sum      | 999999999999999999.999999999; 999999999999999999.999999999 => 1999999999999999999.999999998
number sum      | 999999999999999999; 0.000000001 => 999999999999999999.000000001
number max      | 999999999999999999; 0.000000001 => 999999999999999999
";

#[test]
fn a_column_adds_up_its_values_and_leaves_out_what_is_amiss() {
    for line in FEET.lines().filter(|one| !one.trim().is_empty()) {
        let (column, expected) = line.split_once("=>").expect(line);
        let (words, cells) = column.split_once('|').expect(line);
        let mut text = format!(
            "| Name | Value {{{}}} |
| --- | --- |
",
            words.trim()
        );
        for cell in cells.split(';') {
            text.push_str(&format!(
                "| r | {} |
",
                cell.trim()
            ));
        }
        assert_eq!(
            table_of(&text).foot,
            Some(vec!["Total".to_string(), expected.trim().to_string()]),
            "{line}"
        );
    }
}

#[test]
fn a_table_that_adds_nothing_up_has_no_foot_and_a_first_column_that_does_keeps_its_own() {
    assert_eq!(
        table_of(
            "| a {number} |
| --- |
| 1 |
"
        )
        .foot,
        None
    );
    assert_eq!(
        table_of(
            "| a {number sum} | b |
| --- | --- |
| 1 | x |
| 2 | y |
"
        )
        .foot,
        Some(vec!["3".to_string(), String::new()])
    );
}

#[test]
fn a_value_is_read_as_a_reader_sees_it_whatever_marks_dress_it() {
    let read = table_of(
        "| Precio {money CLP sum} | Nombre {select} |
| --- | --- |
| **6490** | **Zeta** |
| [10](https://x.cl) | [Alpha](https://a.cl) |
| ==7== | `beta` |
",
    );
    let keys: Vec<Option<&str>> = read
        .rows
        .iter()
        .flatten()
        .map(|one| one.key.as_deref())
        .collect();
    assert_eq!(
        keys,
        vec![
            Some("6490"),
            Some("zeta"),
            Some("10"),
            Some("alpha"),
            Some("7"),
            Some("beta")
        ]
    );
    assert!(read.rows.iter().flatten().all(|one| !one.amiss));
    assert_eq!(read.foot, Some(vec!["$6.507".to_string(), String::new()]));
}
