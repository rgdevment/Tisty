use super::*;

const COMMAND: &str = "C:\\Tisty\\tisty.exe";

fn value(text: &str) -> ::toml::Table {
    text.parse().unwrap()
}

#[test]
fn a_table_lands_at_the_end_of_a_file_that_had_none() {
    let was = "model = \"gpt\"\n\n[windows]\nsandbox = \"elevated\"\n";

    let now = set(was, "mcp_servers", "tisty", COMMAND).unwrap();

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

    let now = set(was, "mcp_servers", "tisty", COMMAND).unwrap();

    assert!(now.contains("[projects.'d:\\code\\thing']"), "{now}");
    assert_eq!(now.match_indices("[mcp_servers.tisty]").count(), 1);
    assert_eq!(
        value(&now)["mcp_servers"]["tisty"]["command"].as_str(),
        Some("C:\\Tisty\\tisty.exe")
    );
}

#[test]
fn a_table_in_the_middle_does_not_swallow_what_follows_it() {
    let was = "[mcp_servers.tisty]\ncommand = \"old\"\n\n[mcp_servers.sereno]\ncommand = \"s\"\n";

    let now = set(was, "mcp_servers", "tisty", COMMAND).unwrap();

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

    let now = set(was, "mcp_servers", "tisty", COMMAND).unwrap();

    assert_eq!(now.match_indices("mcp_servers").count(), 1, "{now}");
}

#[test]
fn an_empty_file_gets_the_table_alone() {
    let now = set("", "mcp_servers", "tisty", COMMAND).unwrap();

    assert_eq!(
        value(&now)["mcp_servers"]["tisty"]["args"][0].as_str(),
        Some("mcp")
    );
}

#[test]
fn a_file_that_does_not_parse_is_left_alone() {
    assert_eq!(set("model = \n[[[", "mcp_servers", "tisty", COMMAND), None);
}

#[test]
fn a_server_written_inline_is_repointed_where_it_is_and_one_beside_it_is_refused() {
    let was = "mcp_servers = { tisty = { command = \"x\" } }\n";
    let now = set(was, "mcp_servers", "tisty", COMMAND).unwrap();
    assert_eq!(now.match_indices("mcp_servers").count(), 1, "{now}");
    assert_eq!(
        reads(&now, "mcp_servers", "tisty").as_deref(),
        Some(COMMAND)
    );

    let other = "mcp_servers = { sereno = { command = \"x\" } }\n";
    assert_eq!(set(other, "mcp_servers", "tisty", COMMAND), None);
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

fn repointed(was: &str) -> String {
    set(was, "mcp_servers", "tisty", COMMAND).unwrap()
}

#[test]
fn a_value_over_several_lines_is_changed_whole_and_the_file_still_reads() {
    let was = "model = \"gpt\"\n\n[mcp_servers.tisty]\ncommand = 'C:\\Old\\tisty.exe'\nargs = [\n  \"mcp\",\n]\nstartup_timeout_sec = 20\n\n[mcp_servers.other]\ncommand = \"x\"\n";

    let now = repointed(was);

    let read = value(&now);
    assert_eq!(
        read["mcp_servers"]["tisty"]["command"].as_str(),
        Some("C:\\Tisty\\tisty.exe")
    );
    assert_eq!(
        read["mcp_servers"]["tisty"]["startup_timeout_sec"].as_integer(),
        Some(20)
    );
    assert_eq!(read["mcp_servers"]["other"]["command"].as_str(), Some("x"));
    assert_eq!(read["model"].as_str(), Some("gpt"));
}

#[test]
fn a_key_in_quotes_is_not_written_twice() {
    let was = "[mcp_servers.tisty]\n\"command\" = 'C:\\Old\\tisty.exe'\nargs = [\"mcp\"]\n";

    let read = value(&repointed(was));

    assert_eq!(
        read["mcp_servers"]["tisty"]["command"].as_str(),
        Some("C:\\Tisty\\tisty.exe")
    );
}

#[test]
fn a_line_inside_a_persons_text_is_not_taken_for_ours() {
    let was = "[mcp_servers.tisty]\ncommand = 'C:\\Old\\tisty.exe'\nargs = [\"mcp\"]\nnote = \"\"\"\nuse\nargs = none\n[mcp_servers.tisty]\n\"\"\"\n";

    let read = value(&repointed(was));

    assert_eq!(
        read["mcp_servers"]["tisty"]["note"].as_str(),
        Some("use\nargs = none\n[mcp_servers.tisty]\n")
    );
}

#[test]
fn every_way_of_writing_the_entry_is_repointed_and_the_rest_kept_to_the_byte() {
    for was in [
        "# mine\r\n[mcp_servers.tisty] # ours\r\ncommand = 'C:\\Old\\tisty.exe' # why\r\nargs = [\"mcp\"]\r\n\r\n[other]\r\na = 1\r\n",
        "mcp_servers.tisty.command = 'C:\\Old\\tisty.exe'\nmcp_servers.tisty.args = [\"mcp\"]\n",
        "[mcp_servers]\ntisty = { command = 'C:\\Old\\tisty.exe', args = [\"mcp\"] }\n",
        "[mcp_servers.\"tisty\"]\ncommand = 'C:\\Old\\tisty.exe'\nargs = [\n  \"mcp\", # only\n]\n",
    ] {
        let now = repoint(was, "mcp_servers", "tisty", "C:\\Tisty\\tisty.exe").unwrap();

        assert_eq!(
            reads(&now, "mcp_servers", "tisty").as_deref(),
            Some("C:\\Tisty\\tisty.exe"),
            "{now}"
        );
        assert_eq!(
            now.replace("\"C:\\\\Tisty\\\\tisty.exe\"", "@")
                .replace("'C:\\Tisty\\tisty.exe'", "@"),
            was.replace("'C:\\Old\\tisty.exe'", "@"),
            "only the program changes"
        );
    }
}

#[test]
fn taking_it_out_never_leaves_a_file_that_does_not_read() {
    let was = "[mcp_servers.tisty]\ncommand = 't'\nargs = [\n  [\"a\"],\n]\nnote = \"\"\"\n[x]\n\"\"\"\n\n[other]\na = 1\n";

    let now = unset(was, "mcp_servers", "tisty").unwrap();

    let read = value(&now);
    assert!(
        read.get("mcp_servers")
            .is_none_or(|it| it.get("tisty").is_none())
    );
    assert_eq!(read["other"]["a"].as_integer(), Some(1));
}

#[test]
fn taking_it_out_leaves_the_comments_around_it() {
    let was =
        "# mine\n[a]\nb = 1\n\n# my servers\n[mcp_servers.tisty]\ncommand = \"t\"\n\n[c]\nd = 2\n";

    let now = unset(was, "mcp_servers", "tisty").unwrap();

    assert_eq!(now, "# mine\n[a]\nb = 1\n\n# my servers\n[c]\nd = 2\n");
}
