use std::path::{Path, PathBuf};

use tisty_core::brain::{Brain, Passed, Round, Weighing, kept_up};
use tisty_core::config::Holds;
use tisty_core::event::{DocAdd, Said, TaskAdd};
use tisty_core::{DeviceId, Op, Paths, Store, docs};
use tisty_sync::{Way, carry_telling};
use ulid::Ulid;

const DEVICE: &str = "uno";
const TOLD: &str = "≈";

fn signing(paths: &Paths) -> Store {
    let whose = DeviceId(DEVICE.into());
    let key = tisty_core::signing::mine(paths, &whose);
    if let Some(shown) = key.as_ref().map(tisty_core::signing::shown) {
        std::fs::create_dir_all(paths.data()).unwrap();
        tisty_core::vouched::confirm(paths.data(), &whose, &shown);
    }
    Store::open(paths.store(), whose).unwrap().signing_with(key)
}

fn wrote(paths: &Paths, store: &mut Store, body: &str) -> String {
    let made = docs::create(&paths.docs(), store.device(), body).unwrap();
    store
        .append(Op::DocAdd {
            id: Ulid::generate(),
            d: DocAdd {
                file: made.id.clone(),
                order: "a0".into(),
                said: Some(Said::of(body)),
                ..Default::default()
            },
        })
        .unwrap();
    made.id
}

// A Spanish document and its English twin, alike through the dictionary, beside four tasks that are not.
fn twins_guessed_alike(paths: &Paths) -> Vec<String> {
    let mut store = signing(paths);
    let spanish = wrote(
        paths,
        &mut store,
        "# Pagar la cuenta del agua\n\nPagar la cuenta del agua de la casa.\n",
    );
    let english = wrote(
        paths,
        &mut store,
        "# Pay the water bill\n\nPay the water bill of the house.\n",
    );
    for title in [
        "water the plants",
        "call the plumber",
        "book the dentist",
        "iron shirts",
    ] {
        store
            .append(Op::TaskAdd {
                id: Ulid::generate(),
                d: TaskAdd::new(title, "a0"),
            })
            .unwrap();
    }
    drop(store);
    kept_up(paths, Round::Starting).expect("the base is built");
    let at = paths.cache().join("dictionary.tsv");
    std::fs::write(&at, "agua\twater\ncasa\thouse\ncuenta\tbill\npagar\tpay\n").unwrap();
    let mut weighing = Weighing::default();
    let ran = (0..20)
        .map(|_| weighing.kept(paths, Some(&at)))
        .find(|passed| !matches!(passed, Some(Passed::Settling)));
    assert!(
        matches!(ran, Some(Some(Passed::Ran { translated: 1, .. }))),
        "{ran:?}"
    );
    let guessed = Brain::open(paths.cache())
        .unwrap()
        .alike(&spanish, |_| true)
        .unwrap();
    assert_eq!(guessed.len(), 1, "{guessed:?}");
    assert_eq!(guessed[0].method, "dictionary");
    vec![spanish, english]
}

fn every_file_under(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut waiting = vec![root.to_path_buf()];
    while let Some(at) = waiting.pop() {
        for one in std::fs::read_dir(&at).unwrap().map(Result::unwrap) {
            let path = one.path();
            if path.is_dir() {
                waiting.push(path);
            } else {
                found.push(path);
            }
        }
    }
    found
}

fn holds(bytes: &[u8], said: &str) -> bool {
    bytes
        .windows(said.len())
        .any(|window| window == said.as_bytes())
}

#[test]
fn a_push_carries_the_documents_and_nothing_tisty_guessed_though_the_cache_is_beside_it() {
    let home = tempfile::tempdir().unwrap();
    let paths = Paths::new(home.path().join("data"), home.path().join("config"));
    let alive = twins_guessed_alike(&paths);
    let shared = tempfile::tempdir().unwrap();

    let moved = carry_telling(
        paths.data(),
        Some(paths.cache()),
        DEVICE,
        shared.path(),
        Way::Push,
        &alive,
        Holds::Everywhere,
        &mut |_| {},
    )
    .unwrap();

    assert!(moved.sent > 0, "{moved:?}");
    let carried = every_file_under(shared.path());
    let read: Vec<(PathBuf, Vec<u8>)> = carried
        .iter()
        .map(|path| (path.clone(), std::fs::read(path).unwrap()))
        .collect();
    assert!(
        read.iter()
            .any(|(_, bytes)| holds(bytes, "Pay the water bill of the house")),
        "the documents themselves went: {carried:?}"
    );
    for (path, bytes) in &read {
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(!name.contains("brain") && !name.ends_with(".db"), "{name}");
        assert!(!holds(bytes, TOLD), "{} tells a guess", path.display());
        assert!(
            !holds(bytes, "SQLite format"),
            "{} is a base",
            path.display()
        );
    }
    assert!(paths.cache().join("brain.db").is_file());
}
