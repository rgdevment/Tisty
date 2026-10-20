use super::*;

#[test]
fn copilot_is_told_its_type_and_every_tool() {
    let named: Vec<&str> = fields(Shape::Copilot, "x")
        .iter()
        .map(|(field, _, _)| *field)
        .collect();
    assert_eq!(named, ["type", "command", "args", "tools"]);
}

#[test]
fn opencode_gets_the_command_as_one_list() {
    let all = fields(Shape::Opencode, "C:\\T\\tisty.exe");
    let command = &all
        .iter()
        .find(|(field, _, _)| *field == "command")
        .unwrap()
        .1;
    let read: serde_json::Value = serde_json::from_str(command).unwrap();
    assert_eq!(read, serde_json::json!(["C:\\T\\tisty.exe", "mcp"]));
}

#[test]
fn a_snippet_by_hand_parses_once_wrapped() {
    let said = snippet(Shape::Stdio, "servers", "/usr/bin/tisty");
    let read: serde_json::Value = serde_json::from_str(&format!("{{{said}}}")).unwrap();
    assert_eq!(read["servers"]["tisty"]["type"], "stdio");
}
