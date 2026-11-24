const BRIDGE: &str = "@deepseek-ai/dsh-mcp-client";
const NAME: &str = "tisty";

fn set(text: &str, command: &str) -> Option<String> {
    super::set(text, BRIDGE, NAME, command)
}

fn reads(text: &str) -> Option<String> {
    super::reads(text, BRIDGE, NAME)
}

fn holds(text: &str) -> bool {
    super::holds(text, BRIDGE, NAME)
}

fn unset(text: &str) -> Option<String> {
    super::unset(text, BRIDGE, NAME)
}

fn repoint(text: &str, command: &str) -> Option<String> {
    super::repoint(text, BRIDGE, NAME, command)
}

fn same_but(was: &str, now: &str) -> bool {
    super::same_but(was, now, BRIDGE, NAME)
}

const ROW: &str = "    - id: tisty\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        serverName: tisty\n        transport: stdio\n        command: /opt/tisty\n        args: [mcp]\n";

#[test]
fn an_empty_file_is_written_with_tistys_row() {
    let now = set("", "/opt/tisty").unwrap();
    assert_eq!(now, format!("- insert:\n{ROW}"));
    assert_eq!(reads(&now).as_deref(), Some("/opt/tisty"));
    assert!(holds(&now));
}

#[test]
fn a_windows_command_stays_plain() {
    let now = set("", r"C:\Program Files\Tisty\tisty.exe").unwrap();
    assert!(now.contains(r"command: C:\Program Files\Tisty\tisty.exe"));
    assert_eq!(
        reads(&now).as_deref(),
        Some(r"C:\Program Files\Tisty\tisty.exe")
    );
}

#[test]
fn a_command_with_a_quote_is_quoted() {
    let now = set("", "/opt/it's tisty").unwrap();
    assert!(now.contains("command: '/opt/it''s tisty'"));
    assert_eq!(reads(&now).as_deref(), Some("/opt/it's tisty"));
}

#[test]
fn a_word_yaml_reads_as_another_kind_is_quoted() {
    let now = set("", "true").unwrap();
    assert!(now.contains("command: 'true'"));
    assert_eq!(reads(&now).as_deref(), Some("true"));
}

#[test]
fn the_row_joins_an_insert_entry_that_is_there() {
    let was = "- insert:\n    - id: mcp-github\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        serverName: github\n        transport: stdio\n        command: npx\n        args: ['-y', '@modelcontextprotocol/server-github']\n";
    let now = set(was, "/opt/tisty").unwrap();
    assert_eq!(now, format!("{was}{ROW}"));
}

#[test]
fn a_new_insert_entry_goes_after_what_the_person_wrote() {
    let was = "- id: agent-default-model\n  name: \"@deepseek-ai/dsh-agent-default-model\"\n  config:\n    model: deepseek-v4-pro\n";
    let now = set(was, "/opt/tisty").unwrap();
    assert_eq!(now, format!("{was}- insert:\n{ROW}"));
}

#[test]
fn what_the_person_added_to_the_row_stays() {
    let was = "- insert:\n    - id: tisty\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        serverName: tisty\n        transport: stdio\n        command: /old/tisty\n        args: [mcp]\n        toolCallTimeoutMs: 30000\n";
    let now = set(was, "/opt/tisty").unwrap();
    assert!(now.contains("command: /opt/tisty"));
    assert!(now.contains("toolCallTimeoutMs: 30000"));
    assert_eq!(reads(&now).as_deref(), Some("/opt/tisty"));
}

#[test]
fn a_row_missing_command_and_args_gets_them_under_its_config() {
    let was = "- insert:\n    - id: tisty\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        serverName: tisty\n";
    let now = set(was, "/opt/tisty").unwrap();
    assert_eq!(
        now,
        "- insert:\n    - id: tisty\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        serverName: tisty\n        command: /opt/tisty\n        args: [mcp]\n"
    );
}

#[test]
fn a_row_with_an_empty_config_gets_the_fields_inside_it() {
    let was =
        "- insert:\n    - id: tisty\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n";
    let now = set(was, "/opt/tisty").unwrap();
    assert_eq!(
        now,
        "- insert:\n    - id: tisty\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        command: /opt/tisty\n        args: [mcp]\n"
    );
    assert_eq!(reads(&now).as_deref(), Some("/opt/tisty"));
}

#[test]
fn a_row_without_any_config_gets_one() {
    let was = "- insert:\n    - id: tisty\n      name: '@deepseek-ai/dsh-mcp-client'\n";
    let now = set(was, "/opt/tisty").unwrap();
    assert_eq!(
        now,
        "- insert:\n    - id: tisty\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        command: /opt/tisty\n        args: [mcp]\n"
    );
    assert_eq!(reads(&now).as_deref(), Some("/opt/tisty"));
}

#[test]
fn a_row_by_name_and_server_is_known_even_without_its_id() {
    let was = "- insert:\n    - name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        serverName: tisty\n        transport: stdio\n        command: /old/tisty\n";
    assert!(holds(was));
    assert_eq!(reads(was).as_deref(), Some("/old/tisty"));
    let now = set(was, "/opt/tisty").unwrap();
    assert_eq!(reads(&now).as_deref(), Some("/opt/tisty"));
}

#[test]
fn a_row_named_tisty_with_another_server_is_read_by_its_command() {
    let was = "- insert:\n    - id: tisty\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        serverName: github\n        command: npx\n";
    assert!(holds(was));
    assert_eq!(reads(was).as_deref(), Some("npx"));
}

#[test]
fn a_nested_list_item_names_nothing() {
    let was = "- insert:\n    - id: mcp-other\n      name: '@other/bridge'\n      config:\n        extra:\n          - id: tisty\n          - serverName: tisty\n";
    assert!(!holds(was));
    let nested = "- insert:\n    - id: tisty\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        serverName: tisty\n        other:\n          - command: npx\n        command: /opt/tisty\n";
    assert_eq!(reads(nested).as_deref(), Some("/opt/tisty"));
}

#[test]
fn unwire_takes_the_row_and_its_emptied_entry_out() {
    let was = format!("- id: a\n  name: x\n- insert:\n{ROW}");
    assert_eq!(unset(&was).as_deref(), Some("- id: a\n  name: x\n"));
}

#[test]
fn unwire_leaves_the_other_rows_in_the_entry() {
    let was = "- insert:\n    - id: mcp-github\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        serverName: github\n".to_string()
        + ROW;
    assert_eq!(
        unset(&was).as_deref(),
        Some(
            "- insert:\n    - id: mcp-github\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        serverName: github\n"
        )
    );
}

#[test]
fn unwire_of_tistys_only_entry_leaves_an_empty_patch() {
    let was = set("", "/opt/tisty").unwrap();
    assert_eq!(unset(&was).as_deref(), Some("[]\n"));
    assert!(!holds("[]\n"));
}

#[test]
fn wiring_again_after_an_unwire_stays_one_document() {
    let was = set("", "/old/tisty").unwrap();
    let now = set(&unset(&was).unwrap(), "/opt/tisty").unwrap();
    assert_eq!(now, format!("- insert:\n{ROW}"));
    assert!(!now.contains("[]"));
}

#[test]
fn an_unwired_file_keeps_its_comments_when_wired_again() {
    let now = set("# the patch layer\n[]\n", "/opt/tisty").unwrap();
    assert_eq!(now, format!("# the patch layer\n- insert:\n{ROW}"));
}

#[test]
fn comments_around_stay() {
    let was = format!(
        "# the patch layer\n- insert:\n{}",
        ROW.replace("/opt/tisty", "/old/tisty")
    );
    let now = set(&was, "/opt/tisty").unwrap();
    assert!(now.starts_with("# the patch layer\n"));
    assert!(now.contains("command: /opt/tisty"));
    assert_eq!(unset(&now).as_deref(), Some("# the patch layer\n"));
}

#[test]
fn a_file_that_is_not_a_patch_sequence_is_never_written() {
    let was = "mcpServers:\n  tisty:\n    command: /opt/tisty\n";
    assert_eq!(set(was, "/opt/tisty"), None);
    assert!(!holds(was));
}

#[test]
fn crlf_files_keep_their_ends() {
    let was = "- id: a\r\n  name: x\r\n";
    let now = set(was, "/opt/tisty").unwrap();
    assert_eq!(
        now,
        "- id: a\r\n  name: x\r\n- insert:\r\n    - id: tisty\r\n      name: '@deepseek-ai/dsh-mcp-client'\r\n      config:\r\n        serverName: tisty\r\n        transport: stdio\r\n        command: /opt/tisty\r\n        args: [mcp]\r\n"
    );
    assert_eq!(unset(&now).as_deref(), Some("- id: a\r\n  name: x\r\n"));
}

#[test]
fn a_file_without_a_final_end_stays_readable() {
    let was = "- id: a\n  name: x";
    let now = set(was, "/opt/tisty").unwrap();
    assert_eq!(now, format!("- id: a\n  name: x\n- insert:\n{ROW}"));
    assert_eq!(reads(&now).as_deref(), Some("/opt/tisty"));
}

#[test]
fn a_command_on_the_last_line_without_an_end_is_read_and_written() {
    let was = "- insert:\n    - id: tisty\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        serverName: tisty\n        command: /old/tisty";
    assert_eq!(reads(was).as_deref(), Some("/old/tisty"));
    let now = set(was, "/opt/tisty").unwrap();
    assert!(now.contains("command: /opt/tisty"));
    assert!(now.contains("args: [mcp]"));
}

#[test]
fn unwire_keeps_another_row_on_the_last_line_without_an_end() {
    let was = "- insert:\n    - id: tisty\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        serverName: tisty\n    - id: a";
    assert_eq!(unset(was).as_deref(), Some("- insert:\n    - id: a"));
}

#[test]
fn blank_lines_inside_an_entry_do_not_hide_or_duplicate_the_row() {
    let was = "- insert:\n\n    - id: tisty\n\n      config:\n        serverName: tisty\n        command: /old/tisty\n";
    assert!(holds(was));
    let now = set(was, "/opt/tisty").unwrap();
    assert_eq!(now.matches("- id: tisty").count(), 1);
    assert_eq!(reads(&now).as_deref(), Some("/opt/tisty"));
}

#[test]
fn repoint_changes_only_the_command() {
    let was = set("", "/old/tisty").unwrap();
    let now = repoint(&was, "/opt/tisty").unwrap();
    assert_eq!(reads(&now).as_deref(), Some("/opt/tisty"));
    assert!(same_but(&was, &now));
    assert_eq!(repoint(&now, "/opt/tisty").as_deref(), Some(now.as_str()));
}

#[test]
fn set_and_unset_are_idempotent() {
    let was = set("", "/opt/tisty").unwrap();
    assert_eq!(set(&was, "/opt/tisty").as_deref(), Some(was.as_str()));
    assert_eq!(unset(&unset(&was).unwrap()), None);
}

#[test]
fn a_bom_is_kept_where_it_was() {
    let was = format!(
        "\u{feff}# the patch layer\n- insert:\n{}",
        ROW.replace("/opt/tisty", "/old/tisty")
    );
    let now = set(&was, "/opt/tisty").unwrap();
    assert!(now.starts_with('\u{feff}'));
    assert!(now.contains("command: /opt/tisty"));
    assert_eq!(reads(&was).as_deref(), Some("/old/tisty"));
}

#[test]
fn same_but_sees_only_tistys_row_change() {
    let was = set("", "/opt/tisty").unwrap();
    assert!(same_but(&was, &was));
    assert!(!same_but(&was, "- id: other\n  name: x\n"));
}

#[test]
fn tabs_on_structural_lines_are_kept() {
    let was = "- id: a\n\tname: x\n- insert:\n\t- id: other\n\t\tname: y\n";
    let now = set(was, "/opt/tisty").unwrap();
    assert_eq!(
        now,
        "- id: a\n\tname: x\n- insert:\n\t- id: other\n\t\tname: y\n\t- id: tisty\n\t\tname: '@deepseek-ai/dsh-mcp-client'\n\t\tconfig:\n\t\t\tserverName: tisty\n\t\t\ttransport: stdio\n\t\t\tcommand: /opt/tisty\n\t\t\targs: [mcp]\n"
    );
}

#[test]
fn a_tab_inside_a_scalar_does_not_turn_the_file_into_tabs() {
    let was = "- id: a\n  note: |\n\tnot indentation\n";
    let now = set(was, "/opt/tisty").unwrap();
    assert!(now.contains("    - id: tisty"));
}

#[test]
fn the_first_insert_entry_is_joined_when_there_are_more() {
    let was = "- insert:\n    - id: a\n      name: x\n- insert:\n    - id: b\n      name: y\n";
    let now = set(was, "/opt/tisty").unwrap();
    assert_eq!(
        now,
        "- insert:\n    - id: a\n      name: x\n".to_string()
            + ROW
            + "- insert:\n    - id: b\n      name: y\n"
    );
}

#[test]
fn another_harness_names_its_own_bridge() {
    let was = "- insert:\n    - name: '@some/bridge'\n      config:\n        serverName: tisty\n";
    assert!(super::holds(was, "@some/bridge", NAME));
    assert!(!super::holds(was, "another", NAME));
}

#[test]
fn a_row_without_tistys_id_is_known_only_by_its_bridge_name() {
    let now = super::set("", "@some/bridge", NAME, "/opt/tisty").unwrap();
    assert!(now.contains("name: '@some/bridge'"));
    assert_eq!(
        super::reads(&now, "@some/bridge", NAME).as_deref(),
        Some("/opt/tisty")
    );
}

#[test]
fn the_row_to_paste_quotes_what_it_holds() {
    let said = super::pasted(BRIDGE, NAME, "/opt/it's tisty");
    assert!(said.contains("- id: 'tisty'"));
    assert!(said.contains("name: '@deepseek-ai/dsh-mcp-client'"));
    assert!(said.contains("command: '/opt/it''s tisty'"));
    assert!(said.contains("args: [mcp]"));
}
