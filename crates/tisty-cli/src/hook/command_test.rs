use super::*;

fn paths(command: &str) -> Vec<String> {
    written(command).into_iter().map(|one| one.path).collect()
}

fn moved(command: &str) -> Vec<bool> {
    written(command).into_iter().map(|one| one.moved).collect()
}

#[test]
fn a_shell_redirect_names_the_file_it_writes() {
    for (command, expected) in [
        ("echo x > a.md", vec!["a.md"]),
        ("printf x >> a.md", vec!["a.md"]),
        ("echo x>a.md", vec!["a.md"]),
        ("echo x 1>a.md", vec!["a.md"]),
        ("echo x >|a.md", vec!["a.md"]),
        ("echo x > \"a b.md\"", vec!["a b.md"]),
        ("echo x > 'a b.md'", vec!["a b.md"]),
        ("echo x &> a.md", vec!["a.md"]),
        ("echo x > ~/a.md", vec!["~/a.md"]),
        ("echo x > $HOME/a.md", vec!["$HOME/a.md"]),
    ] {
        assert_eq!(paths(command), expected, "{command}");
    }
}

#[test]
fn a_redirect_is_found_after_any_separator_or_inside_a_group() {
    for command in [
        "ls && echo x > a.md",
        "ls ; echo x > a.md",
        "ls\necho x > a.md",
        "ls || echo x > a.md",
        "ls | cat > a.md",
        "(echo x > a.md)",
        "( echo x > a.md )",
        "{ echo x > a.md; }",
        "echo x \\\n  > a.md",
        "echo x > \\\na.md",
    ] {
        assert_eq!(paths(command), ["a.md"], "{command:?}");
    }
}

#[test]
fn a_here_document_is_never_read_for_what_it_holds() {
    for command in [
        "cat > a.md <<'EOF'\nsee > b.md\nEOF",
        "cat > a.md <<EOF\nsee > b.md\nEOF\n",
        "cat <<EOF > a.md\nsee > b.md\nEOF",
        "cat <<'EOF' > a.md\nsee > b.md\nEOF",
        "cat <<-EOF > a.md\n\tsee > b.md\n\tEOF",
        "cat <<\"END_1\" > a.md\nsee > b.md\nEND_1",
        "cat <<\\EOF > a.md\nsee > b.md\nEOF",
    ] {
        assert_eq!(paths(command), ["a.md"], "{command:?}");
    }
}

#[test]
fn what_follows_a_here_document_is_read_again() {
    assert_eq!(
        paths("cat > a.md <<EOF\nx > b.md\nEOF\necho y > c.md"),
        ["a.md", "c.md"]
    );
}

#[test]
fn tee_names_every_file_that_is_not_an_option() {
    for (command, expected) in [
        ("echo x | tee a.md", vec!["a.md"]),
        ("echo x | tee -a a.md", vec!["a.md"]),
        ("echo x | tee a.md b.md", vec!["a.md", "b.md"]),
        ("echo x | /usr/bin/tee a.md", vec!["a.md"]),
        ("echo x | Tee-Object a.md", vec!["a.md"]),
    ] {
        assert_eq!(paths(command), expected, "{command}");
    }
}

#[test]
fn powershell_cmdlets_name_the_file_they_write() {
    for (command, expected) in [
        ("Set-Content -Path a.md -Value 'x > y.md'", "a.md"),
        ("Set-Content a.md x", "a.md"),
        ("Set-Content a.md -Value x", "a.md"),
        ("Set-Content -Value x a.md", "a.md"),
        (
            "'x' | Out-File -FilePath \"C:\\A B\\a.md\"",
            "C:\\A B\\a.md",
        ),
        ("Add-Content -LiteralPath a.md x", "a.md"),
        ("Add-Content -PSPath a.md x", "a.md"),
        ("New-Item a.md", "a.md"),
        ("New-Item -Path a.md -ItemType File", "a.md"),
        ("sc a.md x", "a.md"),
        ("ac a.md x", "a.md"),
        ("ni a.md", "a.md"),
        ("set-content a.md x", "a.md"),
    ] {
        assert_eq!(paths(command), [expected], "{command}");
    }
}

#[test]
fn a_powershell_here_string_is_never_read_for_what_it_holds() {
    for command in [
        "@'\nx > b.md\n'@ | Set-Content a.md",
        "@\"\nx > b.md\n\"@ | Set-Content a.md",
        "@'\nx > b.md\n'@ | Out-File -FilePath a.md",
        "@'\nx > b.md\n'@ > a.md",
    ] {
        assert_eq!(paths(command), ["a.md"], "{command:?}");
    }
}

#[test]
fn a_program_that_writes_for_itself_names_nothing() {
    for command in [
        "python x.py > a.md",
        "npm test > a.txt",
        "git log > a.md",
        "cargo run >> a.md",
        "echo x 2> a.md",
        "echo x 2>> a.md",
        "echo x >&2",
        "echo x 1>&2",
        "echo \"a > b.md\"",
        "echo 'a > b.md'",
        "grep x < a.md",
        "cat < a.md",
        "# echo > a.md",
        "ls # echo > a.md",
        "git commit -m \"echo > a.md\"",
        "ls a.md",
        "",
        "   \n  ",
    ] {
        assert!(
            paths(command).is_empty(),
            "{command:?}: {:?}",
            paths(command)
        );
    }
}

#[test]
fn what_is_left_open_never_panics() {
    for command in [
        "echo x > \"a.md",
        "echo 'x > a.md",
        "cat > a.md <<EOF\nnever closed > b.md",
        "@'\nnever closed > b.md",
        "echo $(echo > a.md",
        "echo ${x",
        ">",
        ">>",
        "<<",
        "<<-",
        "@'",
        "\\",
        "`",
        "echo x >",
        "&",
        "$",
    ] {
        let _ = written(command);
    }
}

#[test]
fn a_move_before_the_write_marks_it_and_one_after_does_not() {
    for (command, expected) in [
        ("cd /x && echo y > a.md", true),
        ("echo y > a.md && cd /x", false),
        ("pushd /x; echo y > a.md", true),
        ("chdir /x\necho y > a.md", true),
        ("Set-Location C:\\x; 'y' | Out-File a.md", true),
        ("sl C:\\x; Set-Content a.md y", true),
        ("Set-Content a.md y; Set-Location C:\\x", false),
        ("ls && echo y > a.md", false),
        ("cd /x && ls && echo y > a.md", true),
    ] {
        assert_eq!(moved(command), [expected], "{command}");
    }
}

#[test]
fn only_the_writes_after_a_move_are_marked() {
    assert_eq!(
        moved("echo a > one.md && cd /x && echo b > two.md"),
        [false, true]
    );
}
