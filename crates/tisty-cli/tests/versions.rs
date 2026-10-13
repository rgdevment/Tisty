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

fn named(cli: &Cli) -> String {
    let at = cli.ok(&["config", "path"]);
    let body = std::fs::read_to_string(at.trim()).unwrap();
    body.lines()
        .find_map(|line| line.strip_prefix("device_id = "))
        .unwrap()
        .trim_matches('"')
        .to_string()
}

fn earlier_schema() -> u32 {
    std::env::var("TISTY_PREVIOUS_SCHEMA")
        .expect("TISTY_PREVIOUS_SCHEMA says which schema the earlier release writes")
        .parse()
        .expect("a schema number")
}

/// 0 for a release from before places said their shape: it reads none and stops only on the schema.
fn earlier_shape() -> u32 {
    std::env::var("TISTY_PREVIOUS_SHAPE")
        .expect("TISTY_PREVIOUS_SHAPE says which shape of place the earlier release arranges")
        .parse()
        .expect("a shape number")
}

fn earlier_stops() -> bool {
    SCHEMA_VERSION > earlier_schema()
        || (earlier_shape() > 0 && tisty_carrier::SHAPE > earlier_shape())
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
fn an_earlier_release_and_this_build_share_a_folder_as_far_as_their_schemas_allow() {
    let shared = tempfile::tempdir().unwrap();
    let (earlier, current) = written_in_turn(shared.path());
    current.ok(&["written by this build"]);
    current.ok(&["sync"]);
    let before = snapshot(shared.path());

    let run = earlier.run(&["sync"]);

    if earlier_stops() {
        assert_ne!(
            run.code, 0,
            "it carried on beside a newer build: {}",
            run.out
        );
        assert!(run.err.contains("newer"), "{}", run.err);
        assert_eq!(
            snapshot(shared.path()),
            before,
            "an earlier release changed a folder a newer build had written in"
        );
    } else {
        assert_eq!(run.code, 0, "{}", run.err);
        earlier.ok(&["sync", "--confirm", &named(&current), "--force"]);
        earlier.ok(&["sync"]);
        let taken = earlier.ok(&["ls", "all"]);
        assert!(taken.contains("written by this build"), "{taken}");
    }
    let kept = earlier.ok(&["ls", "all"]);
    assert!(kept.contains("written by the earlier release"), "{kept}");
    earlier.ok(&["still writing here"]);
}

#[test]
#[ignore = "needs an earlier release: .github/workflows/versions.yml provides one"]
fn an_earlier_release_stops_at_a_place_this_build_arranged_and_leaves_it_as_it_was() {
    let shared = tempfile::tempdir().unwrap();
    let (earlier, current) = written_in_turn(shared.path());
    current.ok(&["written by this build"]);
    current.ok(&["sync"]);
    let said = std::fs::read_to_string(shared.path().join("tisty.toml")).unwrap();
    assert!(
        said.contains(&format!("shape = {}", tisty_carrier::SHAPE)),
        "{said}"
    );
    assert!(
        said.contains(&format!("needs = \"{}\"", tisty_carrier::NEEDS)),
        "{said}"
    );
    let before = snapshot(shared.path());

    let run = earlier.run(&["sync"]);

    assert!(earlier_stops(), "an earlier release would still write here");
    assert_ne!(run.code, 0, "it synced beside this build: {}", run.out);
    assert!(run.err.contains("newer"), "{}", run.err);
    assert_eq!(
        snapshot(shared.path()),
        before,
        "an earlier release changed a place this build had arranged"
    );
}

#[test]
#[ignore = "needs an earlier release: .github/workflows/versions.yml provides one"]
fn an_earlier_release_stops_at_a_place_a_machine_left_with_a_word_and_leaves_it_as_it_was() {
    let shared = tempfile::tempdir().unwrap();
    let (earlier, current) = written_in_turn(shared.path());
    current.ok(&["written by this build before it left"]);
    current.ok(&["sync"]);
    let data = current.home.path().join("data");
    let key = tisty_core::signing::SigningKey::from_bytes(&[7; 32]);
    let word = tisty_core::moved::said(
        &key,
        tisty_core::moved::Leaving {
            by: &tisty_core::DeviceId(named(&current)),
            to: "folder:elsewhere",
            at: 1_760_000_000,
            store: &tisty_core::moved::store_of(&data),
            place: "folder",
        },
    );
    std::fs::write(shared.path().join(tisty_core::moved::NAMED), word).unwrap();
    let said = std::fs::read_to_string(shared.path().join("tisty.toml")).unwrap();
    assert!(
        said.contains(&format!("shape = {}", tisty_carrier::SHAPE)),
        "{said}"
    );
    let before = snapshot(shared.path());

    let run = earlier.run(&["sync"]);

    assert!(earlier_stops(), "an earlier release would still write here");
    assert_ne!(
        run.code, 0,
        "it synced beside a word it cannot read: {}",
        run.out
    );
    assert!(run.err.contains("newer"), "{}", run.err);
    assert_eq!(
        snapshot(shared.path()),
        before,
        "an earlier release changed a place a machine had left a word in"
    );
    let kept = earlier.ok(&["ls", "all"]);
    assert!(kept.contains("written by the earlier release"), "{kept}");
}
