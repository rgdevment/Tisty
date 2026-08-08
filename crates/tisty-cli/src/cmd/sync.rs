use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

use crate::app::App;
use crate::i18n::Lang;
use crate::style::{self, GREEN};

const ATTRIBUTES: &str = "*.tisty -text\n*.md text eol=lf\nattachments/** binary\n";

pub fn sync(
    app: &App,
    setup: Option<String>,
    status: bool,
    lang: Lang,
) -> anyhow::Result<ExitCode> {
    let root = app.paths.data();
    available(lang)?;

    if let Some(remote) = setup {
        return start(root, &remote, lang);
    }
    if !root.join(".git").try_exists()? {
        anyhow::bail!("{}", lang.get("not-a-repo"));
    }
    if status {
        return report(root, lang);
    }
    run(root, lang)
}

fn available(lang: Lang) -> anyhow::Result<()> {
    let found = Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());

    if !found {
        anyhow::bail!("{}", lang.get("git-missing"));
    }
    Ok(())
}

fn start(root: &Path, remote: &str, lang: Lang) -> anyhow::Result<ExitCode> {
    if !root.join(".git").try_exists()? {
        git(root, &["init", "-b", "main"], lang)?;
    }
    std::fs::write(root.join(".gitattributes"), ATTRIBUTES)?;

    // Idempotent on purpose: running setup twice repoints the remote.
    let _ = git(root, &["remote", "remove", "origin"], lang);
    git(root, &["remote", "add", "origin", remote], lang)?;

    println!("  {} {remote}", style::paint(GREEN, "✓"));
    println!("  {}", style::dim(lang.get("setup-done")));
    Ok(ExitCode::SUCCESS)
}

fn report(root: &Path, lang: Lang) -> anyhow::Result<ExitCode> {
    let pending = git(root, &["status", "--short"], lang)?;
    let remote = git(root, &["remote", "get-url", "origin"], lang).unwrap_or_default();

    println!("\n  {}\n", style::bold(lang.get("sync")));
    println!(
        "    {:<12}{}",
        "remote",
        if remote.trim().is_empty() {
            style::dim(lang.get("no-remote"))
        } else {
            remote.trim().to_string()
        }
    );
    println!(
        "    {:<12}{}",
        "pending",
        if pending.trim().is_empty() {
            style::dim(lang.get("nothing-to-send"))
        } else {
            pending.lines().count().to_string()
        }
    );
    println!();
    Ok(ExitCode::SUCCESS)
}

fn run(root: &Path, lang: Lang) -> anyhow::Result<ExitCode> {
    let has_remote = git(root, &["remote", "get-url", "origin"], lang).is_ok();

    git(root, &["add", "-A"], lang)?;
    let staged = git(root, &["diff", "--cached", "--name-only"], lang)?;

    if staged.trim().is_empty() {
        println!("  {}", style::dim(lang.get("nothing-to-send")));
    } else {
        let message = format!("tisty: {} changes", staged.lines().count());
        git(root, &["commit", "-m", &message], lang)?;
    }

    if !has_remote {
        println!("  {}", style::dim(lang.get("no-remote")));
        return Ok(ExitCode::SUCCESS);
    }

    // Commit before pulling — rebase needs a commit to land on, or git refuses to overwrite setup's files.
    let _ = git(root, &["fetch", "origin"], lang);
    if git(root, &["rev-parse", "--verify", "origin/main"], lang).is_ok()
        && let Err(err) = git(root, &["pull", "--rebase", "origin", "main"], lang)
    {
        let _ = git(root, &["rebase", "--abort"], lang);
        return Err(err);
    }

    git(root, &["push", "origin", "HEAD:main"], lang)?;
    println!("  {} {}", style::paint(GREEN, "✓"), lang.get("synced"));
    Ok(ExitCode::SUCCESS)
}

fn git(root: &Path, args: &[&str], lang: Lang) -> anyhow::Result<String> {
    let out = Command::new("git").current_dir(root).args(args).output()?;
    if !out.status.success() {
        anyhow::bail!(
            "{}: {}",
            lang.fill("git-failed", &[("command", &args.join(" "))]),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
