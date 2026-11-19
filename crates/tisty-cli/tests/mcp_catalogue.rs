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
    said[0]["result"]["content"][0]["text"]
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

const DOOR_AT_MOST: usize = 40_830;
const DOOR_SLACK: usize = 1_000;

#[test]
fn what_an_agent_pays_to_have_tisty_connected_only_shrinks() {
    let said = ask(&[
        request("server/discover", &json!({})),
        request("tools/list", &json!({})),
    ]);
    let instructions = said[0]["result"]["instructions"].as_str().unwrap().len();
    let tools = serde_json::to_string(&said[1]["result"]["tools"])
        .unwrap()
        .len();
    let weight = instructions + tools;

    assert!(
        weight <= DOOR_AT_MOST,
        "connecting Tisty costs an agent {weight} bytes ({instructions} of instructions, {tools} of tools), past the {DOOR_AT_MOST} it may: say it in fewer words, or move the how-to into `rules`"
    );
    assert!(
        weight + DOOR_SLACK > DOOR_AT_MOST,
        "connecting Tisty now costs {weight} bytes: lower DOOR_AT_MOST to just above that, so the saving stays"
    );
}

#[test]
fn brain_read_and_sum_up_stay_within_the_weight_each_was_given() {
    let said = ask(&[request("tools/list", &json!({}))]);
    for (name, at_most) in [("brain", 3_800), ("read", 2_140), ("sum_up", 900)] {
        let tool = said[0]["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|tool| tool["name"] == name)
            .unwrap();
        let weight = serde_json::to_string(tool).unwrap().len();

        assert!(
            weight <= at_most,
            "`{name}` weighs {weight} bytes in the list, past the {at_most} it was given: say it in fewer words, or move the how-to into `rules`"
        );
    }
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
fn the_idea_rules_an_agent_is_handed() {
    insta::assert_snapshot!(rules(Some("ideas")));
}

#[test]
fn the_task_rules_an_agent_is_handed() {
    insta::assert_snapshot!(rules(Some("tasks")));
}

#[test]
fn the_document_rules_an_agent_is_handed() {
    insta::assert_snapshot!(rules(Some("documents")));
}

#[test]
fn the_guides_weigh_what_the_rules_tool_says_they_do() {
    for topic in ["tasks", "documents", "tidying", "ideas"] {
        let size = rules(Some(topic)).chars().count();
        assert!(
            (3_000..=11_200).contains(&size),
            "«{topic}» is {size} characters, and `rules` says 3,000 to 11,200: say the new size there"
        );
    }
    let all = rules(Some("all")).chars().count();
    assert!(
        all <= 34_100,
        "`all` is {all} characters, and `rules` says 34,100: say the new size there"
    );
}

#[test]
fn nothing_an_agent_is_handed_names_a_tool_that_moved() {
    let call = |name: &str, arguments: Value| {
        request(
            "tools/call",
            &json!({ "name": name, "arguments": arguments }),
        )
    };
    let prompts = ask(&[request("prompts/list", &json!({}))]);
    let mut asked = vec![
        request("server/discover", &json!({})),
        request("tools/list", &json!({})),
        request("prompts/list", &json!({})),
        call("rules", json!({})),
        call("language", json!({})),
        call("language", json!({ "topic": "all" })),
        call("help", json!({ "lang": "en" })),
        call("help", json!({ "lang": "es" })),
    ];
    for topic in ["tasks", "documents", "tidying", "ideas", "all"] {
        asked.push(call("rules", json!({ "topic": topic })));
    }
    for prompt in prompts[0]["result"]["prompts"].as_array().unwrap() {
        asked.push(request(
            "prompts/get",
            &json!({ "name": prompt["name"], "arguments": {} }),
        ));
    }

    let answers = ask(&asked);

    assert_eq!(answers.len(), asked.len());
    for one in &answers {
        assert!(
            one.get("error").is_none() && one["result"]["isError"] != true,
            "every text is read, none refused: {one}"
        );
    }
    let said = serde_json::to_string(&answers).unwrap();
    for read in [
        "How Tisty itself works",
        "Pick up where I left off",
        "# Tisty Markdown",
    ] {
        assert!(said.contains(read), "{read} was not handed");
    }
    // `docs`, `lists` and `tags` moved too, but they are also the names of fields and answer keys.
    let named: Vec<&str> = ["find", "outline_doc", "read_doc"]
        .into_iter()
        .filter(|name| said.contains(&format!("`{name}`")))
        .collect();
    assert!(
        named.is_empty(),
        "an agent is still pointed at {named:?}, which only answer that they moved: name `brain` or `read` instead"
    );
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
