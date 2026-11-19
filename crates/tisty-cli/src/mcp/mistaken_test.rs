use super::as_the_tools_read;
use serde_json::json;

const ADDRESS: &str = "tisty://doc/personal/proyectos/validacion-de-datos?id=4nnctnme-0039";

#[test]
fn an_address_stands_for_its_id_wherever_a_tool_names_a_document() {
    let asked = json!({
        "doc": ADDRESS,
        "page_of": "tisty://doc/actas?id=4nnctnme-0001",
        "translation_of": "tisty://doc?id=4nnctnme-0002",
        "of": " tisty://doc/?id=4nnctnme-0003 ",
        "after": "tisty://doc/marzo?id=4nnctnme-0004",
        "before": "tisty://doc/mayo?id=4nnctnme-0005",
        "instead": "tisty://doc/la-nueva?id=4nnctnme-0006",
    });

    assert_eq!(
        as_the_tools_read(asked),
        json!({
            "doc": "4nnctnme-0039",
            "page_of": "4nnctnme-0001",
            "translation_of": "4nnctnme-0002",
            "of": "4nnctnme-0003",
            "after": "4nnctnme-0004",
            "before": "4nnctnme-0005",
            "instead": "4nnctnme-0006",
        })
    );
}

#[test]
fn each_address_in_a_list_of_documents_stands_for_its_own_id() {
    let asked = json!({ "doc": [ADDRESS, "4nnctnme-0040", "Actas de marzo"], "folder": "Trabajo" });

    assert_eq!(
        as_the_tools_read(asked),
        json!({ "doc": ["4nnctnme-0039", "4nnctnme-0040", "Actas de marzo"], "folder": "Trabajo" })
    );
}

#[test]
fn the_pages_put_in_order_may_each_be_named_by_its_address() {
    let asked = json!({
        "page_of": "4nnctnme-0001",
        "order": ["tisty://doc/actas/mayo?id=4nnctnme-0005", "4nnctnme-0004"],
    });

    assert_eq!(
        as_the_tools_read(asked),
        json!({ "page_of": "4nnctnme-0001", "order": ["4nnctnme-0005", "4nnctnme-0004"] })
    );
}

#[test]
fn an_address_written_anywhere_else_stays_as_the_words_it_is() {
    let asked = json!({
        "doc": "4nnctnme-0040",
        "after": 20,
        "body": format!("See [the plan]({ADDRESS})."),
        "old": ADDRESS,
        "new": ADDRESS,
        "query": ADDRESS,
        "task": ADDRESS,
        "folder": ADDRESS,
    });

    assert_eq!(as_the_tools_read(asked.clone()), asked);
}

#[test]
fn what_is_no_address_of_a_document_is_left_for_the_tool_to_refuse() {
    let asked = json!({
        "doc": "tisty://doc/personal/proyectos",
        "page_of": "tisty://task?id=4nnctnme-0039",
        "of": "tisty://doc?id=4nnctnme-0039&folder=x",
    });

    assert_eq!(as_the_tools_read(asked.clone()), asked);
    assert_eq!(as_the_tools_read(json!("doc")), json!("doc"));
}

#[test]
fn a_name_reaches_the_door_and_the_tool_without_the_blanks_around_it() {
    let asked = json!({
        "doc": " 4nnctnme-0040\n",
        "page_of": "\t4nnctnme-0001",
        "order": [" 4nnctnme-0005 ", 7],
        "folder": " AGENTS / Claude\n",
        "inside": "\tTrabajo ",
        "after": 20,
        "of": "\u{2066}4nnctnme-0003",
    });

    assert_eq!(
        as_the_tools_read(asked),
        json!({
            "doc": "4nnctnme-0040",
            "page_of": "4nnctnme-0001",
            "order": ["4nnctnme-0005", 7],
            "folder": "AGENTS / Claude",
            "inside": "Trabajo",
            "after": 20,
            "of": "\u{2066}4nnctnme-0003",
        })
    );
}
