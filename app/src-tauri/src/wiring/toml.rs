pub fn reads(text: &str, key: &str, name: &str) -> Option<String> {
    let doc: ::toml::Table = text.parse().ok()?;
    doc.get(key)?
        .get(name)?
        .get("command")?
        .as_str()
        .map(str::to_string)
}

pub fn set(text: &str, key: &str, name: &str, entry: &str) -> Option<String> {
    if !text.trim().is_empty() && text.parse::<::toml::Table>().is_err() {
        return None;
    }
    let head = format!("[{key}.{name}]");
    if let Some((from, to)) = span(text, key, name) {
        return Some(format!("{}{head}\n{entry}\n{}", &text[..from], &text[to..]));
    }
    // Written some other way — inline, or a name we would not recognise — and rewriting it would
    // mean guessing at somebody else's formatting.
    if reads(text, key, name).is_some() {
        return None;
    }
    let mut out = text.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.trim().is_empty() {
        out.push('\n');
    }
    out.push_str(&head);
    out.push('\n');
    out.push_str(entry);
    out.push('\n');
    Some(out)
}

pub fn unset(text: &str, key: &str, name: &str) -> Option<String> {
    let (from, to) = span(text, key, name)?;
    let kept = format!("{}{}", &text[..from], &text[to..]);
    Some(match kept.trim().is_empty() {
        true => String::new(),
        false => kept,
    })
}

fn span(text: &str, key: &str, name: &str) -> Option<(usize, usize)> {
    let want = [key, name];
    let mut ours: Option<usize> = None;
    let mut at = 0usize;
    for line in text.split_inclusive('\n') {
        let head = line.trim_start().starts_with('[');
        if head {
            if let Some(from) = ours {
                return Some((from, at));
            }
            if same(line, &want) {
                ours = Some(at);
            }
        }
        at += line.len();
    }
    ours.map(|from| (from, text.len()))
}

fn same(line: &str, want: &[&str]) -> bool {
    let Some(inner) = line
        .trim()
        .strip_prefix('[')
        .and_then(|it| it.strip_suffix(']'))
    else {
        return false;
    };
    if inner.starts_with('[') {
        return false;
    }
    let parts: Vec<&str> = inner
        .split('.')
        .map(|part| part.trim().trim_matches(['"', '\'']))
        .collect();
    parts == want
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTRY: &str = "command = \"C:\\\\Tisty\\\\tisty.exe\"\nargs = [\"mcp\"]";

    fn value(text: &str) -> ::toml::Table {
        text.parse().unwrap()
    }

    #[test]
    fn a_table_lands_at_the_end_of_a_file_that_had_none() {
        let was = "model = \"gpt\"\n\n[windows]\nsandbox = \"elevated\"\n";

        let now = set(was, "mcp_servers", "tisty", ENTRY).unwrap();

        let read = value(&now);
        assert_eq!(
            read["mcp_servers"]["tisty"]["args"][0].as_str(),
            Some("mcp")
        );
        assert_eq!(read["windows"]["sandbox"].as_str(), Some("elevated"));
        assert_eq!(read["model"].as_str(), Some("gpt"));
    }

    #[test]
    fn the_table_already_there_is_repointed_and_the_rest_kept_verbatim() {
        let was = "model = \"gpt\"\n\n[projects.'d:\\code\\thing']\ntrust_level = \"trusted\"\n\n[mcp_servers.tisty]\ncommand = \"C:\\\\Old\\\\tisty.exe\"\nargs = [\"mcp\"]\n";

        let now = set(was, "mcp_servers", "tisty", ENTRY).unwrap();

        assert!(now.contains("[projects.'d:\\code\\thing']"), "{now}");
        assert_eq!(now.match_indices("[mcp_servers.tisty]").count(), 1);
        assert_eq!(
            value(&now)["mcp_servers"]["tisty"]["command"].as_str(),
            Some("C:\\Tisty\\tisty.exe")
        );
    }

    #[test]
    fn a_table_in_the_middle_does_not_swallow_what_follows_it() {
        let was =
            "[mcp_servers.tisty]\ncommand = \"old\"\n\n[mcp_servers.sereno]\ncommand = \"s\"\n";

        let now = set(was, "mcp_servers", "tisty", ENTRY).unwrap();

        let read = value(&now);
        assert_eq!(read["mcp_servers"]["sereno"]["command"].as_str(), Some("s"));
        assert_eq!(
            read["mcp_servers"]["tisty"]["command"].as_str(),
            Some("C:\\Tisty\\tisty.exe")
        );
    }

    #[test]
    fn a_name_in_quotes_is_the_same_table() {
        let was = "[mcp_servers.\"tisty\"]\ncommand = \"old\"\n";

        let now = set(was, "mcp_servers", "tisty", ENTRY).unwrap();

        assert_eq!(now.match_indices("mcp_servers").count(), 1, "{now}");
    }

    #[test]
    fn an_empty_file_gets_the_table_alone() {
        let now = set("", "mcp_servers", "tisty", ENTRY).unwrap();

        assert_eq!(
            value(&now)["mcp_servers"]["tisty"]["args"][0].as_str(),
            Some("mcp")
        );
    }

    #[test]
    fn a_file_that_does_not_parse_is_left_alone() {
        assert_eq!(set("model = \n[[[", "mcp_servers", "tisty", ENTRY), None);
    }

    #[test]
    fn a_server_written_inline_is_refused_rather_than_doubled() {
        let was = "mcp_servers = { tisty = { command = \"x\" } }\n";

        assert_eq!(set(was, "mcp_servers", "tisty", ENTRY), None);
    }

    #[test]
    fn the_command_is_read_back() {
        let was = "[mcp_servers.tisty]\ncommand = \"C:\\\\Tisty\\\\tisty.exe\"\nargs = [\"mcp\"]\n";

        assert_eq!(
            reads(was, "mcp_servers", "tisty").unwrap(),
            "C:\\Tisty\\tisty.exe"
        );
        assert_eq!(reads(was, "mcp_servers", "sereno"), None);
    }

    #[test]
    fn taking_it_out_leaves_the_neighbours_alone() {
        let was = "model = \"gpt\"\n\n[mcp_servers.tisty]\ncommand = \"t\"\n\n[mcp_servers.sereno]\ncommand = \"s\"\n";

        let now = unset(was, "mcp_servers", "tisty").unwrap();

        let read = value(&now);
        assert!(read["mcp_servers"].get("tisty").is_none());
        assert_eq!(read["mcp_servers"]["sereno"]["command"].as_str(), Some("s"));
        assert_eq!(read["model"].as_str(), Some("gpt"));
    }

    #[test]
    fn taking_out_what_was_never_there_says_so() {
        assert_eq!(unset("model = \"gpt\"\n", "mcp_servers", "tisty"), None);
    }
}
