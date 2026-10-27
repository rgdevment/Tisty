use super::*;
use crate::{Kind, read};

// One cell per line: its column's type, what is written, then what the page shows (`-` for the words
// as written, `""` for nothing) and its sort key, or `amiss`.
const CELLS: &str = r#"
number      | 6490                      => -              | 6490
number      | 1.5                       => -              | 1.5
number      | -3.25                     => -              | -3.25
number      | 6.490                     => -              | 6.490
number      | 6,490                     => amiss
number      | 1e3                       => amiss
number      | .5                        => amiss
number      | 5.                        => amiss
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
select      | Música                    => -              | música
tags        | web, docs                 => -              | web, docs
text        | Hola                      => -              | hola
"#;

fn holds(words: &str) -> Holds {
    crate::typed::understood(words).expect(words).0
}

#[test]
fn each_cell_reads_as_its_column_says_and_never_is_refused() {
    for line in CELLS.lines().filter(|one| !one.trim().is_empty()) {
        let (cell, outcome) = line.split_once("=>").expect(line);
        let (words, said) = cell.split_once('|').expect(line);
        let seen = seen(&holds(words.trim()), said.trim());
        let outcome = outcome.trim();
        if outcome == "amiss" {
            assert_eq!(
                seen,
                Seen {
                    amiss: true,
                    ..Seen::default()
                },
                "{line}"
            );
            continue;
        }
        let (shown, key) = outcome.split_once('|').expect(line);
        let shown = match shown.trim() {
            "-" => None,
            r#""""# => Some(String::new()),
            one => Some(one.to_string()),
        };
        let expected = Seen {
            shown,
            key: Some(key.trim().to_string()),
            amiss: false,
        };
        assert_eq!(seen, expected, "{line}");
    }
}

#[test]
fn an_empty_cell_is_neither_a_value_nor_amiss() {
    assert_eq!(seen(&Holds::Number, ""), Seen::default());
}

// One column per line: its type and totals, its cells split by `;`, then what its foot says.
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
";

#[test]
fn a_column_adds_up_its_values_and_leaves_out_what_is_amiss() {
    for line in FEET.lines().filter(|one| !one.trim().is_empty()) {
        let (column, expected) = line.split_once("=>").expect(line);
        let (words, cells) = column.split_once('|').expect(line);
        let mut text = format!("| Name | Value {{{}}} |\n| --- | --- |\n", words.trim());
        for cell in cells.split(';') {
            text.push_str(&format!("| r | {} |\n", cell.trim()));
        }
        let doc = read(&text);
        let table = &doc.blocks()[0];
        let Kind::Table { columns } = &table.kind else {
            panic!("not a table: {line}");
        };
        let foot = footed(columns, table, &text).expect(line);
        assert_eq!(
            foot,
            vec!["Total".to_string(), expected.trim().to_string()],
            "{line}"
        );
    }
}

#[test]
fn a_table_that_adds_nothing_up_has_no_foot_and_a_first_column_that_does_keeps_its_own() {
    let plain = "| a {number} |\n| --- |\n| 1 |\n";
    let doc = read(plain);
    let Kind::Table { columns } = &doc.blocks()[0].kind else {
        panic!("not a table");
    };
    assert_eq!(footed(columns, &doc.blocks()[0], plain), None);

    let summed = "| a {number sum} | b |\n| --- | --- |\n| 1 | x |\n| 2 | y |\n";
    let doc = read(summed);
    let Kind::Table { columns } = &doc.blocks()[0].kind else {
        panic!("not a table");
    };
    assert_eq!(
        footed(columns, &doc.blocks()[0], summed),
        Some(vec!["3".to_string(), String::new()])
    );
}
