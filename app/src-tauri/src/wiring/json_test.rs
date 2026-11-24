use super::*;

const ENTRY: &str = r#"{ "command": "C:\\Tisty\\tisty.exe", "args": ["mcp"] }"#;

fn value(text: &str) -> serde_json::Value {
    serde_json::from_str(text).unwrap()
}

#[test]
fn a_server_lands_beside_the_ones_already_there() {
    let was = "{\n  \"mcpServers\": {\n    \"sereno\": { \"command\": \"sereno\" }\n  }\n}\n";

    let now = set(was, "mcpServers", "tisty", ENTRY).unwrap();

    let read = value(&now);
    assert_eq!(read["mcpServers"]["tisty"]["args"][0], "mcp");
    assert_eq!(read["mcpServers"]["sereno"]["command"], "sereno");
}

#[test]
fn everything_else_in_the_file_is_left_where_it_was() {
    let was = "{\n  \"numStartups\": 53,\n  \"mcpServers\": {\n    \"sereno\": { \"command\": \"sereno\" }\n  },\n  \"projects\": { \"d:\\\\code\": { \"trust\": true } }\n}\n";

    let now = set(was, "mcpServers", "tisty", ENTRY).unwrap();

    assert!(now.starts_with("{\n  \"numStartups\": 53,"), "{now}");
    assert!(now.contains("\"projects\": { \"d:\\\\code\": { \"trust\": true } }"));
    assert_eq!(
        now.match_indices("\"sereno\":").count(),
        1,
        "the other server was written twice"
    );
}

#[test]
fn a_server_that_is_already_there_is_repointed_and_not_doubled() {
    let was = "{\n  \"mcpServers\": {\n    \"tisty\": { \"command\": \"C:\\\\Old\\\\tisty.exe\", \"args\": [\"mcp\"] },\n    \"sereno\": { \"command\": \"sereno\" }\n  }\n}\n";

    let now = set(was, "mcpServers", "tisty", ENTRY).unwrap();

    assert_eq!(now.match_indices("\"tisty\"").count(), 1);
    assert_eq!(
        value(&now)["mcpServers"]["tisty"]["command"],
        "C:\\Tisty\\tisty.exe"
    );
    assert!(value(&now)["mcpServers"]["sereno"].is_object());
}

#[test]
fn a_file_without_the_key_gets_it() {
    let was = "{\n  \"theme\": \"dark\"\n}\n";

    let now = set(was, "mcpServers", "tisty", ENTRY).unwrap();

    assert_eq!(value(&now)["theme"], "dark");
    assert_eq!(value(&now)["mcpServers"]["tisty"]["args"][0], "mcp");
}

#[test]
fn an_empty_object_is_filled_rather_than_broken() {
    let was = "{\n  \"servers\": {}\n}\n";

    let now = set(was, "servers", "tisty", ENTRY).unwrap();

    assert_eq!(value(&now)["servers"]["tisty"]["args"][0], "mcp");
}

#[test]
fn a_file_that_is_not_there_yet_is_written_whole() {
    let now = set("", "mcpServers", "tisty", ENTRY).unwrap();

    assert_eq!(value(&now)["mcpServers"]["tisty"]["args"][0], "mcp");
}

#[test]
fn comments_and_tabs_survive_the_visit() {
    let was = "{\n\t// what code puts here\n\t\"servers\": {\n\t\t\"Figma\": { \"url\": \"https://x\" }\n\t},\n\t\"inputs\": []\n}\n";

    let now = set(was, "servers", "tisty", ENTRY).unwrap();

    assert!(now.contains("// what code puts here"), "{now}");
    assert!(now.contains("\n\t\t\"tisty\": {"), "{now}");
    assert!(now.contains("\"Figma\""));
}

#[test]
fn a_key_that_is_not_an_object_is_refused_rather_than_replaced() {
    let was = "{\n  \"mcpServers\": \"none\"\n}\n";

    assert_eq!(set(was, "mcpServers", "tisty", ENTRY), None);
}

#[test]
fn what_is_not_json_at_all_is_refused() {
    assert_eq!(set("just words", "mcpServers", "tisty", ENTRY), None);
    assert_eq!(set("[1, 2]", "mcpServers", "tisty", ENTRY), None);
}

#[test]
fn the_command_is_read_back_even_through_comments() {
    let was = "{\n\t/* kept */\n\t\"servers\": {\n\t\t\"tisty\": { \"command\": \"C:\\\\Tisty\\\\tisty.exe\", \"args\": [\"mcp\"] }\n\t}\n}\n";

    assert_eq!(
        reads(was, "servers", "tisty").unwrap(),
        "C:\\Tisty\\tisty.exe"
    );
    assert_eq!(reads(was, "servers", "sereno"), None);
}

#[test]
fn taking_it_out_leaves_the_neighbours_alone() {
    let was = "{\n  \"mcpServers\": {\n    \"tisty\": { \"command\": \"t\" },\n    \"sereno\": { \"command\": \"s\" }\n  },\n  \"theme\": \"dark\"\n}\n";

    let now = unset(was, "mcpServers", "tisty").unwrap();

    assert!(value(&now)["mcpServers"]["tisty"].is_null());
    assert_eq!(value(&now)["mcpServers"]["sereno"]["command"], "s");
    assert_eq!(value(&now)["theme"], "dark");
}

#[test]
fn taking_out_the_last_one_leaves_a_file_that_still_parses() {
    let was = "{\n  \"mcpServers\": {\n    \"sereno\": { \"command\": \"s\" },\n    \"tisty\": { \"command\": \"t\" }\n  }\n}\n";

    let now = unset(was, "mcpServers", "tisty").unwrap();

    assert_eq!(value(&now)["mcpServers"]["sereno"]["command"], "s");
    assert!(value(&now)["mcpServers"]["tisty"].is_null());
}

#[test]
fn taking_out_the_only_one_leaves_an_empty_room() {
    let was = "{\n  \"mcpServers\": {\n    \"tisty\": { \"command\": \"t\" }\n  }\n}\n";

    let now = unset(was, "mcpServers", "tisty").unwrap();

    assert!(value(&now)["mcpServers"].as_object().unwrap().is_empty());
}

#[test]
fn taking_out_what_was_never_there_says_so() {
    let was = "{\n  \"mcpServers\": {\n    \"sereno\": { \"command\": \"s\" }\n  }\n}\n";

    assert_eq!(unset(was, "mcpServers", "tisty"), None);
}

#[test]
fn a_byte_order_mark_does_not_hide_the_document() {
    let was = "\u{feff}{\n  \"mcpServers\": {}\n}\n";

    let now = set(was, "mcpServers", "tisty", ENTRY).unwrap();

    assert!(now.starts_with('\u{feff}'));
    assert_eq!(
        value(now.trim_start_matches('\u{feff}'))["mcpServers"]["tisty"]["args"][0],
        "mcp"
    );
}

fn list_at(text: &str, key: &str) -> usize {
    let raw = text.as_bytes();
    member(raw, opens(raw).unwrap(), key).unwrap().value.0
}

#[test]
fn an_element_lands_after_the_ones_already_in_a_list() {
    let was = "{\n  \"Stop\": [\n    { \"a\": 1 }\n  ]\n}\n";

    let now = push(was, list_at(was, "Stop"), "{ \"b\": 2 }").unwrap();

    assert_eq!(value(&now)["Stop"][0]["a"], 1);
    assert_eq!(value(&now)["Stop"][1]["b"], 2);
    assert_eq!(
        now,
        "{\n  \"Stop\": [\n    { \"a\": 1 },\n    { \"b\": 2 }\n  ]\n}\n"
    );
}

#[test]
fn an_empty_list_takes_its_first_element() {
    let was = "{\n  \"Stop\": []\n}\n";

    let now = push(was, list_at(was, "Stop"), "{ \"b\": 2 }").unwrap();

    assert_eq!(value(&now)["Stop"][0]["b"], 2);
}

#[test]
fn the_elements_of_a_list_are_read_as_written() {
    let was = "{ \"x\": [ 1, \"two\", { \"three\": [3] } ] }";
    let raw = was.as_bytes();

    let all = elements(raw, list_at(was, "x")).unwrap();

    let told: Vec<&str> = all.iter().map(|(a, b)| &was[*a..*b]).collect();
    assert_eq!(told, vec!["1", "\"two\"", "{ \"three\": [3] }"]);
    assert_eq!(elements(raw, 0), None);
}

#[test]
fn cutting_an_element_leaves_the_list_whole() {
    let was = "{\n  \"Stop\": [\n    { \"a\": 1 },\n    { \"b\": 2 },\n    { \"c\": 3 }\n  ]\n}\n";
    let at = list_at(was, "Stop");
    let all = elements(was.as_bytes(), at).unwrap();

    let first = cut(was, at, all[0]).unwrap();
    let middle = cut(was, at, all[1]).unwrap();
    let last = cut(was, at, all[2]).unwrap();

    assert_eq!(value(&first)["Stop"].as_array().unwrap().len(), 2);
    assert_eq!(value(&first)["Stop"][0]["b"], 2);
    assert_eq!(value(&middle)["Stop"][1]["c"], 3);
    assert_eq!(value(&last)["Stop"][1]["b"], 2);
    assert_eq!(
        last,
        "{\n  \"Stop\": [\n    { \"a\": 1 },\n    { \"b\": 2 }\n  ]\n}\n"
    );
}

#[test]
fn cutting_the_only_element_leaves_an_empty_list() {
    let was = "{\n  \"Stop\": [\n    { \"a\": 1 }\n  ]\n}\n";
    let at = list_at(was, "Stop");
    let all = elements(was.as_bytes(), at).unwrap();

    let now = cut(was, at, all[0]).unwrap();

    assert_eq!(value(&now)["Stop"], serde_json::json!([]));
}
