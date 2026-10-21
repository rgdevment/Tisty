use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

// The text an agent reads is a contract, so every wording change shows up in review.
fn ask(requests: &[Value]) -> Vec<Value> {
    let home = TempDir::new().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_tisty"))
        .arg("mcp")
        .env("TISTY_DATA", home.path().join("data"))
        .env("TISTY_CONFIG", home.path().join("config"))
        .env("TISTY_CACHE", home.path().join("cache"))
        .env("NO_COLOR", "1")
        .env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut pipe = child.stdin.take().unwrap();
        for one in requests {
            writeln!(pipe, "{one}").unwrap();
        }
    }
    let out = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect(line))
        .collect()
}

fn request(method: &str, params: &Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params })
}

fn rules(topic: Option<&str>) -> String {
    let arguments = topic.map_or_else(|| json!({}), |one| json!({ "topic": one }));
    let said = ask(&[request(
        "tools/call",
        &json!({ "name": "rules", "arguments": arguments }),
    )]);
    said[0]["result"]["structuredContent"]["rules"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn the_instructions_an_agent_reads_when_it_connects() {
    let said = ask(&[request("server/discover", &json!({}))]);
    let text = said[0]["result"]["instructions"].as_str().unwrap();
    // The first line names today, which changes every day.
    let (_, kept) = text.split_once("\n\n").unwrap();
    insta::assert_snapshot!(kept);
}

#[test]
fn the_general_rules_an_agent_is_handed() {
    insta::assert_snapshot!(rules(None));
}

#[test]
fn the_tidying_rules_an_agent_is_handed() {
    insta::assert_snapshot!(rules(Some("tidying")));
}

#[test]
fn the_names_of_the_tools_an_agent_is_offered() {
    let said = ask(&[request("tools/list", &json!({}))]);
    let names: Vec<&str> = said[0]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    insta::assert_json_snapshot!(names);
}

#[test]
fn every_tool_says_its_title_description_and_arguments() {
    let said = ask(&[request("tools/list", &json!({}))]);
    for tool in said[0]["result"]["tools"].as_array().unwrap() {
        let name = tool["name"].as_str().unwrap();
        insta::assert_json_snapshot!(name, tool);
    }
}
