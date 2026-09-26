use super::*;

fn said(body: &str) -> Vec<String> {
    tags_in(body)
        .into_iter()
        .map(|one| one.as_str().to_string())
        .collect()
}

#[test]
fn a_hash_against_a_word_is_a_tag() {
    assert_eq!(said("esto es #legal antes que nada"), ["legal"]);
}

#[test]
fn a_heading_keeps_its_space_and_is_left_alone() {
    assert_eq!(
        said("# Alquiler del local\n\n## Lo que falta"),
        Vec::<String>::new()
    );
}

#[test]
fn the_fragment_of_an_address_belongs_to_the_address() {
    assert_eq!(
        said("mira https://ejemplo.com/pagina#seccion"),
        Vec::<String>::new()
    );
}

#[test]
fn a_colour_written_inline_is_not_a_tag_either() {
    assert_eq!(said("el fondo es `#ff0000` y ya"), Vec::<String>::new());
    assert_eq!(said("border: 1px solid #hair"), ["hair"]);
}

#[test]
fn a_fenced_block_is_stepped_over_whole() {
    assert_eq!(
        said("antes #uno\n\n```css\ncolor: #rojo;\n```\n\ndespués #dos"),
        ["uno", "dos"]
    );
}

#[test]
fn the_same_tag_twice_is_kept_once_and_in_the_order_it_was_written() {
    assert_eq!(
        said("#dinero y luego #legal y otra vez #dinero"),
        ["dinero", "legal"]
    );
}

#[test]
fn it_is_normalised_the_way_a_task_normalises_its_own() {
    assert_eq!(said("#Contrato #CONTRATO #contrato"), ["contrato"]);
    assert_eq!(said("#pago_mensual"), ["pago-mensual"]);
}

#[test]
fn a_hash_on_its_own_holds_nothing() {
    assert_eq!(said("un # suelto y un #- también"), Vec::<String>::new());
}

#[test]
fn punctuation_around_it_does_not_travel_with_it() {
    assert_eq!(said("queda (#legal), sí."), ["legal"]);
}

#[test]
fn several_on_one_line_are_all_read() {
    assert_eq!(said("#uno #dos #tres"), ["uno", "dos", "tres"]);
}

#[test]
fn a_word_pasted_from_a_mac_is_the_same_word() {
    assert_eq!(said("revisar #disen\u{303}o hoy"), ["diseno"]);
    assert_eq!(said("#diseño"), said("revisar #disen\u{303}o hoy"));
}

#[test]
fn a_backtick_on_its_own_is_prose_and_swallows_nothing() {
    assert_eq!(
        said("el operador ` marca código y esto es #legal"),
        ["legal"]
    );
    assert_eq!(said("`#rojo` no, pero #legal sí"), ["legal"]);
    assert_eq!(said("#uno `#dos` #tres"), ["uno", "tres"]);
}

#[test]
fn a_hash_needs_a_letter_against_it() {
    assert_eq!(said("#_borrador y #-legal"), Vec::<String>::new());
    assert_eq!(said("#borrador y #legal"), ["borrador", "legal"]);
}

#[test]
fn a_number_behind_the_hash_is_a_reference_somebody_wrote_down() {
    assert_eq!(
        said("cierra el ticket #1234 antes del viernes"),
        Vec::<String>::new()
    );
    assert_eq!(said("el punto #1 y luego el #2"), Vec::<String>::new());
    assert_eq!(said("#2026 tampoco"), Vec::<String>::new());
    assert_eq!(said("#pepe32 y #b2b si"), ["pepe32", "b2b"]);
}

#[test]
fn one_letter_labels_nothing() {
    assert_eq!(said("#a y #x"), Vec::<String>::new());
    assert_eq!(said("#ia y #ux"), ["ia", "ux"]);
}

#[test]
fn a_body_that_is_not_tagging_stops_at_the_cap() {
    let css: String = (0..2000)
        .map(|n| {
            format!(
                ".c{n} {{ color: #a{n:04x}; }}
"
            )
        })
        .collect();

    assert_eq!(
        tags_in(&css).len(),
        AT_MOST,
        "ni una linea de miles en el registro"
    );
}

#[test]
fn what_a_document_says_of_itself_carries_them() {
    let said = crate::event::Said::of("# Alquiler\n\nesto es #legal y #dinero");

    assert_eq!(said.title, "Alquiler");
    assert_eq!(
        said.tags
            .unwrap()
            .iter()
            .map(|one| one.as_str())
            .collect::<Vec<_>>(),
        ["legal", "dinero"]
    );
}

#[test]
fn a_tag_that_changed_is_news_even_where_the_title_did_not() {
    let kept = crate::model::Kept {
        born_by: None,
        guest: false,
        made: None,
        made_by: None,
        wrote_by: None,
        by: None,
        id: ulid::Ulid::generate(),
        file: "a3f1-0001".into(),
        order: "a0".into(),
        title: Some("Alquiler".into()),
        bytes: Some(31),
        wrote: None,
        folder: None,
        page_of: None,
        archived: false,
        locked: false,
        tags: vec![Tag::new("legal").unwrap()],
        edited_by: None,
        flagged: None,
        folder_was: None,
    };

    let same = crate::event::Said {
        title: "Alquiler".into(),
        bytes: Some(31),
        tags: Some(vec![Tag::new("legal").unwrap()]),
        by: None,
    };
    assert!(!same.news_for(&kept));

    let fresh = crate::event::Said {
        tags: Some(vec![
            Tag::new("legal").unwrap(),
            Tag::new("dinero").unwrap(),
        ]),
        ..same.clone()
    };
    let older = crate::event::Said { tags: None, ..same };
    assert!(
        fresh.news_for(&kept),
        "una etiqueta nueva es algo que contar"
    );
    assert!(
        !older.news_for(&kept),
        "y una nota de una version que no las leia no dice nada de ellas"
    );
}
