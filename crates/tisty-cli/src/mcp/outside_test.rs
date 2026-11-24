use super::*;

fn named(said: &str) -> Vec<&str> {
    said.split('`').skip(1).step_by(2).collect()
}

#[test]
fn every_call_it_names_is_one_tisty_answers() {
    for said in [FILE_LEFT_OUTSIDE, COMMAND_LEAVES_OUTSIDE, CARD_LEFT_OUTSIDE] {
        for tool in named(said).into_iter().filter(|it| *it != "state") {
            assert!(crate::mcp::is_tool(tool), "{tool}");
        }
        assert!(
            said.contains("make the same call again: it goes through"),
            "{said}"
        );
        assert!(!said.contains("  "), "{said}");
    }
}

#[test]
fn a_command_is_told_that_none_of_it_ran() {
    assert!(COMMAND_LEAVES_OUTSIDE.contains("none of it ran"));
    assert!(!FILE_LEFT_OUTSIDE.contains("command"));
}
