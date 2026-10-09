use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tempfile::TempDir;
use tisty_core::event::SCHEMA_VERSION;

struct Cli {
    home: TempDir,
    binary: PathBuf,
}

struct Run {
    out: String,
    err: String,
    code: i32,
}

impl Cli {
    fn current() -> Self {
        Self::of(PathBuf::from(env!("CARGO_BIN_EXE_tisty")))
    }

    fn earlier() -> Self {
        let named = std::env::var("TISTY_PREVIOUS")
            .expect("TISTY_PREVIOUS names the earlier release's tisty: .github/workflows/versions.yml sets it");
        Self::of(PathBuf::from(named))
    }

    fn of(binary: PathBuf) -> Self {
        Self {
            home: tempfile::tempdir().unwrap(),
            binary,
        }
    }

    fn run(&self, args: &[&str]) -> Run {
        let root = self.home.path();
        let output = Command::new(&self.binary)
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("TZ", "UTC")
            .env("NO_COLOR", "1")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL")
            .env_remove("LC_MESSAGES")
            .args(args)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        Run {
            out: String::from_utf8_lossy(&output.stdout).into_owned(),
            err: String::from_utf8_lossy(&output.stderr).into_owned(),
            code: output.status.code().unwrap_or(-1),
        }
    }

    fn ok(&self, args: &[&str]) -> String {
        let run = self.run(args);
        assert_eq!(run.code, 0, "`{}` failed: {}", args.join(" "), run.err);
        run.out
    }
}

fn earlier_schema() -> u32 {
    std::env::var("TISTY_PREVIOUS_SCHEMA")
        .expect("TISTY_PREVIOUS_SCHEMA says which schema the earlier release writes")
        .parse()
        .expect("a schema number")
}

fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(here) = pending.pop() {
        for entry in std::fs::read_dir(&here).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                pending.push(entry.path());
            } else {
                let name = entry.path().strip_prefix(dir).unwrap().to_path_buf();
                found.insert(name, std::fs::read(entry.path()).unwrap());
            }
        }
    }
    found
}

fn written_in_turn(shared: &Path) -> (Cli, Cli) {
    let met = shared.display().to_string();

    let earlier = Cli::earlier();
    earlier.ok(&["config", "set", "remote", &met]);
    earlier.ok(&["written by the earlier release"]);
    earlier.ok(&["sync"]);

    let current = Cli::current();
    current.ok(&["config", "set", "remote", &met]);
    current.ok(&["sync"]);
    current.ok(&["sync"]);
    (earlier, current)
}

#[test]
#[ignore = "needs an earlier release: .github/workflows/versions.yml provides one"]
fn this_build_takes_in_what_an_earlier_release_synced() {
    let shared = tempfile::tempdir().unwrap();

    let (_earlier, current) = written_in_turn(shared.path());

    let seen = current.ok(&["ls", "all"]);
    assert!(seen.contains("written by the earlier release"), "{seen}");
}

#[test]
#[ignore = "needs an earlier release: .github/workflows/versions.yml provides one"]
fn an_earlier_release_stops_before_it_writes_once_a_newer_build_has() {
    let shared = tempfile::tempdir().unwrap();
    let (earlier, current) = written_in_turn(shared.path());
    current.ok(&["written by this build"]);
    current.ok(&["sync"]);
    let before = snapshot(shared.path());

    let run = earlier.run(&["sync"]);

    if SCHEMA_VERSION > earlier_schema() {
        assert_ne!(
            run.code, 0,
            "it carried on beside a newer build: {}",
            run.out
        );
        assert!(run.err.contains("newer Tisty"), "{}", run.err);
        assert_eq!(
            snapshot(shared.path()),
            before,
            "an earlier release changed a folder a newer build had written in"
        );
    } else {
        assert_eq!(run.code, 0, "{}", run.err);
    }
    let kept = earlier.ok(&["ls", "all"]);
    assert!(kept.contains("written by the earlier release"), "{kept}");
    earlier.ok(&["still writing here"]);
}
