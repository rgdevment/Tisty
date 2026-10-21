use std::process::Command;

#[test]
fn the_help_names_mcp_on_stdout_the_way_a_launcher_looks_for_it() {
    let out = Command::new(env!("CARGO_BIN_EXE_tisty"))
        .arg("--help")
        .env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .output()
        .unwrap();

    assert!(out.status.success());
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(
        said.lines()
            .any(|line| line.starts_with("  mcp ") || line.starts_with("\tmcp ")),
        "a launcher greps the help for an indented `mcp`: {said:?}"
    );
}
