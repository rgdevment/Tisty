use crate::{Holds, Kind, read, vocab};

// One head per line: as written, then the type read from it (or `-`), then the words left in the head.
const HEADS: &str = "
Precio {money CLP sum}  => money CLP sum => Precio
Precio {money clp}      => money CLP     => Precio
Precio {money sum}      => money sum     => Precio
Precio {money}          => money         => Precio
Activa {check}          => check         => Activa
Activa {check count}    => check count   => Activa
{check}                 => check         =>
Fecha {date max}        => date max      => Fecha
Horas {duration avg}    => duration avg  => Horas
Nota {rating avg}       => rating avg    => Nota
Etiquetas {tags count}  => tags count    => Etiquetas
Notas {text}            => text          => Notas
Notas {importante}      => -             => Notas {importante}
Precio{money}           => -             => Precio{money}
Fecha {date sum}        => -             => Fecha {date sum}
Activa {check sum}      => -             => Activa {check sum}
Precio {money CLP USD}  => -             => Precio {money CLP USD}
Precio {money sum CLP}  => -             => Precio {money sum CLP}
Precio {number CLP}     => -             => Precio {number CLP}
Precio {money CL}       => -             => Precio {money CL}
Total {sum avg}         => -             => Total {sum avg}
Precio {}               => -             => Precio {}
Precio {Money}          => -             => Precio {Money}
";

fn typed(holds: &Holds, total: Option<crate::Total>) -> String {
    let currency = match holds {
        Holds::Money {
            currency: Some(code),
        } => format!(" {code}"),
        _ => String::new(),
    };
    let total = total
        .map(|one| format!(" {}", one.word()))
        .unwrap_or_default();
    format!("{}{currency}{total}", holds.word())
}

#[test]
fn a_head_says_its_type_in_braces_only_when_every_word_is_known() {
    for line in HEADS.lines().filter(|one| !one.trim().is_empty()) {
        let parts: Vec<&str> = line.split("=>").map(str::trim).collect();
        let [head, expected, left] = parts[..] else {
            panic!("a case is three parts: {line}");
        };
        let text = format!("| {head} | b |\n| --- | --- |\n| 1 | 2 |\n");
        let doc = read(&text);
        let table = &doc.blocks()[0];
        let Kind::Table { columns } = &table.kind else {
            panic!("not a table: {line}");
        };
        let column = &columns[0];
        let read_as = match column.typed {
            Some(_) => typed(&column.holds, column.total),
            None => "-".to_string(),
        };
        assert_eq!(read_as, expected, "{line}");
        let cell = &table.inner[0].inner[0];
        let shown: String = cell.spans.iter().map(|one| &text[one.at.clone()]).collect();
        assert_eq!(shown.trim(), left, "{line}");
    }
}

#[test]
fn the_type_is_cut_from_the_head_and_nothing_else_moves() {
    let text = "| Servicio | Precio {money CLP sum} |\n| --- | --: |\n| Spotify | 6490 |\n";
    let doc = read(text);
    let Kind::Table { columns } = &doc.blocks()[0].kind else {
        panic!("not a table");
    };
    let at = columns[1].typed.clone().expect("typed");
    assert_eq!(&text[at], " {money CLP sum}");
    assert_eq!(columns[0].typed, None);
    assert_eq!(doc.text(), text);
}

#[test]
fn braces_in_code_or_after_a_pen_are_not_a_type() {
    for head in ["`{money}`", "==Precio=={green}", "**Precio {money}**"] {
        let text = format!("| {head} |\n| --- |\n| 1 |\n");
        let doc = read(&text);
        let Kind::Table { columns } = &doc.blocks()[0].kind else {
            panic!("not a table");
        };
        assert_eq!(columns[0].typed, None, "{head}");
    }
}

#[test]
fn a_table_past_its_size_is_read_as_words() {
    let mut text = String::from("| Precio {money} |\n| --- |\n");
    for _ in 0..2001 {
        text.push_str("| 1 |\n");
    }
    let doc = read(&text);
    let Kind::Table { columns } = &doc.blocks()[0].kind else {
        panic!("not a table");
    };
    assert_eq!(columns[0].typed, None);
}

#[test]
fn every_word_in_the_vocabulary_is_a_type_or_a_total_tisty_reads() {
    for word in vocab::HOLDS {
        let (holds, _) = super::understood(word).expect(word);
        assert_eq!(holds.word(), *word);
    }
    for word in vocab::TOTALS {
        let (_, total) = super::understood(&format!("number {word}")).expect(word);
        assert_eq!(total.map(crate::Total::word), Some(*word));
    }
}
