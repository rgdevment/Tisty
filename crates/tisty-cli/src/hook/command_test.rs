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

#[test]
fn a_push_a_pull_request_opened_or_merged_publishes() {
    for command in [
        "git push",
        "git push origin main",
        "git push -u origin feat/x",
        "git push --force-with-lease",
        "gh pr create --title x --body y",
        "gh pr create",
        "gh pr merge 12 --squash",
        "gh pr merge",
        "/usr/bin/git push",
        "git.exe push",
    ] {
        assert!(publishes(command), "{command}");
    }
}

#[test]
fn options_before_the_subcommand_do_not_hide_a_push() {
    for command in [
        "git -C ../repo push",
        "git -C ../repo push origin main",
        "git -c user.name=x push",
        "git -c core.editor=true -C repo push",
        "git --git-dir x push",
        "git --work-tree y push",
        "git --no-pager push",
        "git --git-dir=x push",
        "gh -R owner/repo pr merge 3",
        "gh --repo owner/repo pr create",
        "gh -R owner/repo pr create --fill",
    ] {
        assert!(publishes(command), "{command}");
    }
}

#[test]
fn a_push_chained_with_other_commands_still_publishes() {
    for command in [
        "git add . && git commit -m x && git push",
        "git status; git push",
        "git push && echo done",
        "git push || echo failed",
        "git push\necho done",
        "echo start; gh pr create --fill; echo end",
        "cargo test && gh pr merge 4 --squash",
        "(git push)",
        "{ git push; }",
    ] {
        assert!(publishes(command), "{command}");
    }
}

#[test]
fn what_only_looks_at_a_repository_or_a_pull_request_does_not_publish() {
    for command in [
        "git status",
        "git pull",
        "git fetch",
        "git commit -m x",
        "git stash push",
        "git log --oneline",
        "git diff",
        "git branch push",
        "git remote add push url",
        "gh pr view",
        "gh pr view 12 --json state",
        "gh pr list",
        "gh pr checks",
        "gh pr status",
        "gh issue create",
        "gh -R owner/repo pr view 3",
        "ls",
        "",
    ] {
        assert!(!publishes(command), "{command}");
    }
}

#[test]
fn a_quoted_push_is_text_and_not_a_command() {
    for command in [
        "echo \"git push\"",
        "echo 'git push'",
        "echo \"gh pr merge 3\"",
        "printf 'run git push later'",
        "grep \"git push\" notes.md",
    ] {
        assert!(!publishes(command), "{command}");
    }
}

#[test]
fn a_push_written_inside_a_here_document_is_not_run() {
    for command in [
        "cat <<EOF\ngit push\nEOF",
        "cat <<'EOF' > notes.md\ngit push origin main\ngh pr merge 3\nEOF",
        "cat > run.sh <<EOF\ngit push\nEOF\nls",
        "cat <<EOF\ngh pr create\nEOF\necho done",
    ] {
        assert!(!publishes(command), "{command}");
    }
}

#[test]
fn a_push_after_a_here_document_is_still_run() {
    let command = "cat <<EOF > notes.md\nthe plan\nEOF\ngit push";
    assert!(publishes(command));
}

#[test]
fn a_push_written_into_a_here_string_is_not_run() {
    assert!(!publishes("cat <<< \"git push\""));
}
