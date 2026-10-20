use super::catalog::Shape;

fn quoted(said: &str) -> String {
    serde_json::Value::from(said).to_string()
}

/// The fields ours in a JSON entry, each with whether it is set over what is there.
pub fn fields(shape: Shape, command: &str) -> Vec<(&'static str, String, bool)> {
    let args = ("args", "[\"mcp\"]".to_string(), true);
    let command_alone = ("command", quoted(command), true);
    match shape {
        Shape::Plain => vec![command_alone, args],
        Shape::Stdio => vec![("type", quoted("stdio"), true), command_alone, args],
        Shape::Copilot => vec![
            ("type", quoted("local"), true),
            command_alone,
            args,
            ("tools", "[\"*\"]".to_string(), false),
        ],
        Shape::Opencode => vec![
            ("type", quoted("local"), true),
            ("command", format!("[{}, \"mcp\"]", quoted(command)), true),
            ("enabled", "true".to_string(), false),
        ],
        Shape::Toml | Shape::Manual => Vec::new(),
    }
}

pub fn toml_entry(command: &str) -> String {
    format!(
        "command = {}\nargs = [\"mcp\"]",
        ::toml::Value::from(command)
    )
}

/// What a person pastes by hand: the entry in the shape the assistant reads, under its key.
pub fn snippet(shape: Shape, key: &str, command: &str) -> String {
    match shape {
        Shape::Toml => format!("[{key}.tisty]\n{}", toml_entry(command)),
        Shape::Manual if key == "extensions" => format!(
            "extensions:\n  tisty:\n    type: stdio\n    name: tisty\n    enabled: true\n    cmd: {}\n    args: [mcp]",
            quoted(command)
        ),
        Shape::Manual => format!(
            "mcpServers:\n  - name: tisty\n    type: stdio\n    command: {}\n    args: [mcp]",
            quoted(command)
        ),
        _ => {
            let inner: Vec<String> = fields(shape, command)
                .into_iter()
                .map(|(field, value, _)| format!("    \"{field}\": {value}"))
                .collect();
            format!(
                "\"{key}\": {{\n  \"tisty\": {{\n{}\n  }}\n}}",
                inner.join(",\n")
            )
        }
    }
}

#[cfg(test)]
#[path = "shape_test.rs"]
mod tests;
