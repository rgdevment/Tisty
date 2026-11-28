use serde_json::{Value, json};
use tisty_core::Paths;
use tisty_core::event::{DocAdd, FolderAdd, Said};
use tisty_core::model::FolderId;
use ulid::Ulid;

use super::super::{Refused, Takes, answered, written_or_read};

struct Home {
    _room: tempfile::TempDir,
    paths: Paths,
    ideas: String,
    ideas_page: String,
    diary: String,
    day: String,
    rule: String,
    plain: String,
    notes: String,
    work: String,
    claude: String,
    picture: String,
}

fn persons(paths: &Paths, ops: Vec<tisty_core::Op>) {
    let config = tisty_core::Config::load_or_init(paths).unwrap();
    tisty_core::Store::open(paths.store(), config.device_id.clone())
        .unwrap()
        .signing_with(tisty_core::signing::mine(paths, &config.device_id))
        .append_batch_tagged(ops, false, false)
        .unwrap();
}

fn folder_op(
    id: FolderId,
    name: &str,
    parent: Option<FolderId>,
    icon: Option<&str>,
) -> tisty_core::Op {
    tisty_core::Op::FolderAdd {
        id,
        d: FolderAdd {
            name: name.into(),
            order: "a0".into(),
            parent,
            icon: icon.map(str::to_string),
            color: None,
            computer: None,
        },
    }
}

fn written(
    paths: &Paths,
    body: &str,
    folder: Option<FolderId>,
    page_of: Option<Ulid>,
    mark: impl FnOnce(&mut Said),
) -> (String, Ulid) {
    let config = tisty_core::Config::load_or_init(paths).unwrap();
    let made = tisty_core::docs::create(&paths.docs(), &config.device_id, body).unwrap();
    let id = Ulid::generate();
    let mut said = Said::of(body).agreed();
    mark(&mut said);
    persons(
        paths,
        vec![tisty_core::Op::DocAdd {
            id,
            d: DocAdd {
                file: made.id.clone(),
                order: "a0".into(),
                said: Some(said),
                folder,
                page_of,
                ..Default::default()
            },
        }],
    );
    (made.id, id)
}

fn home() -> Home {
    let room = tempfile::tempdir().unwrap();
    let paths = Paths::new(room.path().join("data"), room.path().join("config"));
    persons(&paths, tisty_core::model::sown("en"));
    tisty_core::agent::register(&paths).unwrap();
    let (root, claude, notes, work) = (
        Ulid::generate(),
        Ulid::generate(),
        Ulid::generate(),
        Ulid::generate(),
    );
    persons(
        &paths,
        vec![
            folder_op(
                root,
                tisty_core::agents::FOLDER,
                None,
                Some(tisty_core::agents::ICON),
            ),
            folder_op(claude, "Claude", Some(root), None),
            folder_op(notes, "Notes", None, None),
            folder_op(work, "Work", None, None),
        ],
    );
    let none = |_: &mut Said| {};
    let (ideas, ideas_id) = written(
        &paths,
        "---\nlang: en\n---\n# Ideas\n",
        Some(notes),
        None,
        |one| one.brainstorm = true,
    );
    let (ideas_page, _) = written(&paths, "# Page of ideas\n", None, Some(ideas_id), none);
    let (diary, diary_id) = written(
        &paths,
        "---\ndiary: on\nlang: en\n---\n\n# 2026\n",
        Some(notes),
        None,
        |one| one.diary = true,
    );
    let (day, _) = written(&paths, "# November\n", None, Some(diary_id), none);
    let (rule, _) = written(
        &paths,
        "---\nagent: claude/rule/CLAUDE.md\ntitle: claude · CLAUDE.md\n---\n# Claude\nTests first.\n",
        Some(claude),
        None,
        none,
    );
    let (plain, _) = written(
        &paths,
        "---\nlang: es\n---\n# Plain\n\nMine.\n",
        Some(notes),
        None,
        none,
    );
    let picture = room.path().join("idea.md");
    std::fs::write(&picture, "# Brought\n\nfrom outside\n").unwrap();
    Home {
        picture: picture.to_string_lossy().into_owned(),
        _room: room,
        paths,
        ideas,
        ideas_page,
        diary,
        day,
        rule,
        plain,
        notes: "Notes".into(),
        work: "Work".into(),
        claude: "AGENTS / Claude".into(),
    }
}

fn open_diary(home: &Home) {
    persons(
        &home.paths,
        vec![tisty_core::Op::DiaryAgents { open: true }],
    );
}

fn ops(paths: &Paths) -> usize {
    tisty_core::store::read_all(paths.store()).unwrap().len()
}

fn bodies(paths: &Paths) -> Vec<(String, String)> {
    let mut all: Vec<_> = tisty_core::docs::all(&paths.docs())
        .into_iter()
        .map(|one| {
            let body = tisty_core::docs::read(&paths.docs(), &one.id).unwrap();
            (one.id, body)
        })
        .collect();
    all.sort();
    all
}

fn refuses(home: &Home, name: &str, args: Value) {
    refused_by(home, name, args, written_or_read);
}

fn refuses_past_the_door(home: &Home, name: &str, args: Value) {
    refused_by(home, name, args, answered);
}

fn refused_by(
    home: &Home,
    name: &str,
    args: Value,
    call: fn(&Paths, &str, Value, Option<&Takes>) -> Result<Value, Refused>,
) {
    let (log, words) = (ops(&home.paths), bodies(&home.paths));
    let said = call(&home.paths, name, args.clone(), None);
    let wrote = ops(&home.paths) != log || bodies(&home.paths) != words;
    match said {
        Err(Refused::Tool(why)) if !wrote => {
            assert!(!why.contains("moved into"), "{name} {args}: {why}");
        }
        Err(Refused::Tool(why)) => panic!("{name} {args}: refused but wrote ({why})"),
        Err(_) => panic!("{name} {args}: refused as a protocol error"),
        Ok(said) => panic!("{name} {args} went through: {said}"),
    }
}

#[test]
fn file_doc_ideas_into_a_folder() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "file", "docs": [home.ideas], "folder": home.work }),
    );
}

#[test]
fn file_doc_ideas_to_the_root() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "file", "docs": [home.ideas] }),
    );
}

#[test]
fn file_doc_diary_into_a_folder_with_the_diary_open() {
    let home = home();
    open_diary(&home);
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "file", "docs": [home.diary], "folder": home.work }),
    );
}

#[test]
fn file_doc_diary_to_the_root_with_the_diary_open() {
    let home = home();
    open_diary(&home);
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "file", "docs": [home.diary] }),
    );
}

#[test]
fn file_doc_diary_into_a_folder_with_the_diary_closed() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "file", "docs": [home.diary], "folder": home.work }),
    );
}

#[test]
fn file_doc_diary_to_the_root_with_the_diary_closed() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "file", "docs": [home.diary] }),
    );
}

#[test]
fn file_doc_diary_day_to_the_root_with_the_diary_open() {
    let home = home();
    open_diary(&home);
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "file", "docs": [home.day] }),
    );
}

#[test]
fn file_doc_a_plain_doc_into_an_agents_folder() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "file", "docs": [home.plain], "folder": home.claude }),
    );
}

#[test]
fn file_doc_an_agents_doc_out_to_a_plain_folder() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "file", "docs": [home.rule], "folder": home.notes }),
    );
}

#[test]
fn page_doc_ideas_becomes_a_page() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "add_page", "docs": [home.ideas], "parent": home.plain }),
    );
}

#[test]
fn page_doc_a_page_of_ideas_is_let_go() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "remove_page", "docs": [home.ideas_page] }),
    );
}

#[test]
fn page_doc_a_doc_is_hung_under_ideas() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "add_page", "docs": [home.plain], "parent": home.ideas }),
    );
}

#[test]
fn page_doc_the_diary_becomes_a_page_with_the_diary_open() {
    let home = home();
    open_diary(&home);
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "add_page", "docs": [home.diary], "parent": home.plain }),
    );
}

#[test]
fn page_doc_a_diary_day_is_let_go_with_the_diary_open() {
    let home = home();
    open_diary(&home);
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "remove_page", "docs": [home.day] }),
    );
}

#[test]
fn page_doc_a_doc_is_hung_under_the_diary_with_the_diary_open() {
    let home = home();
    open_diary(&home);
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "add_page", "docs": [home.plain], "parent": home.diary }),
    );
}

#[test]
fn page_doc_a_diary_day_is_let_go_with_the_diary_closed() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "remove_page", "docs": [home.day] }),
    );
}

#[test]
fn page_doc_a_doc_is_hung_under_the_diary_with_the_diary_closed() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "add_page", "docs": [home.plain], "parent": home.diary }),
    );
}

#[test]
fn write_doc_new_page_of_ideas() {
    let home = home();
    refuses(
        &home,
        "doc_create",
        json!({ "body": "# New\n", "page_of": home.ideas }),
    );
}

#[test]
fn write_doc_new_page_of_the_diary_with_the_diary_open() {
    let home = home();
    open_diary(&home);
    refuses(
        &home,
        "doc_create",
        json!({ "body": "# New\n", "page_of": home.diary }),
    );
}

#[test]
fn write_doc_new_page_of_the_diary_with_the_diary_closed() {
    let home = home();
    refuses(
        &home,
        "doc_create",
        json!({ "body": "# New\n", "page_of": home.diary }),
    );
}

#[test]
fn write_doc_translation_of_ideas() {
    let home = home();
    refuses(
        &home,
        "doc_create",
        json!({ "body": "# Ideas\n", "translation_of": home.ideas, "lang": "es" }),
    );
}

#[test]
fn write_doc_translation_of_the_diary_with_the_diary_open() {
    let home = home();
    open_diary(&home);
    refuses(
        &home,
        "doc_create",
        json!({ "body": "# 2026\n", "translation_of": home.diary, "lang": "es" }),
    );
}

#[test]
fn write_doc_translation_of_the_diary_with_the_diary_closed() {
    let home = home();
    refuses(
        &home,
        "doc_create",
        json!({ "body": "# 2026\n", "translation_of": home.diary, "lang": "es" }),
    );
}

#[test]
fn write_doc_a_plain_doc_into_an_agents_folder() {
    let home = home();
    refuses(
        &home,
        "doc_create",
        json!({ "body": "# Mine\n", "folder": home.claude }),
    );
}

#[test]
fn tongue_doc_ideas_joins_another_doc() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "link_translation", "docs": [home.ideas], "parent": home.plain, "lang": "en" }),
    );
}

#[test]
fn tongue_doc_a_doc_joins_ideas() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "link_translation", "docs": [home.plain], "parent": home.ideas }),
    );
}

#[test]
fn tongue_doc_the_diary_joins_another_doc_with_the_diary_open() {
    let home = home();
    open_diary(&home);
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "link_translation", "docs": [home.diary], "parent": home.plain, "lang": "en" }),
    );
}

#[test]
fn tongue_doc_a_doc_joins_the_diary_with_the_diary_open() {
    let home = home();
    open_diary(&home);
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "link_translation", "docs": [home.plain], "parent": home.diary }),
    );
}

#[test]
fn tongue_doc_the_diary_joins_another_doc_with_the_diary_closed() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "link_translation", "docs": [home.diary], "parent": home.plain, "lang": "en" }),
    );
}

#[test]
fn tongue_doc_a_doc_joins_the_diary_with_the_diary_closed() {
    let home = home();
    refuses(
        &home,
        "doc_organize",
        json!({ "action": "link_translation", "docs": [home.plain], "parent": home.diary }),
    );
}

#[test]
fn import_doc_as_a_page_of_ideas() {
    let home = home();
    refuses(
        &home,
        "file_transfer",
        json!({ "action": "import_doc", "path": home.picture, "page_of": home.ideas }),
    );
}

#[test]
fn import_doc_as_a_page_of_the_diary_with_the_diary_open() {
    let home = home();
    open_diary(&home);
    refuses(
        &home,
        "file_transfer",
        json!({ "action": "import_doc", "path": home.picture, "page_of": home.diary }),
    );
}

#[test]
fn import_doc_as_a_page_of_the_diary_with_the_diary_closed() {
    let home = home();
    refuses(
        &home,
        "file_transfer",
        json!({ "action": "import_doc", "path": home.picture, "page_of": home.diary }),
    );
}

#[test]
fn import_doc_into_an_agents_folder() {
    let home = home();
    refuses(
        &home,
        "file_transfer",
        json!({ "action": "import_doc", "path": home.picture, "folder": home.claude }),
    );
}

#[test]
fn past_the_door_file_doc_diary_to_the_root() {
    let home = home();
    open_diary(&home);
    refuses_past_the_door(
        &home,
        "doc_organize",
        json!({ "action": "file", "docs": [home.diary] }),
    );
}

#[test]
fn past_the_door_file_doc_diary_into_a_folder() {
    let home = home();
    open_diary(&home);
    refuses_past_the_door(
        &home,
        "doc_organize",
        json!({ "action": "file", "docs": [home.diary], "folder": home.work }),
    );
}

#[test]
fn past_the_door_page_doc_diary_becomes_a_page() {
    let home = home();
    open_diary(&home);
    refuses_past_the_door(
        &home,
        "doc_organize",
        json!({ "action": "add_page", "docs": [home.diary], "parent": home.plain }),
    );
}

#[test]
fn past_the_door_page_doc_a_diary_day_is_let_go() {
    let home = home();
    open_diary(&home);
    refuses_past_the_door(
        &home,
        "doc_organize",
        json!({ "action": "remove_page", "docs": [home.day] }),
    );
}

#[test]
fn past_the_door_page_doc_a_doc_is_hung_under_the_diary() {
    let home = home();
    open_diary(&home);
    refuses_past_the_door(
        &home,
        "doc_organize",
        json!({ "action": "add_page", "docs": [home.plain], "parent": home.diary }),
    );
}

#[test]
fn past_the_door_write_doc_new_page_of_the_diary() {
    let home = home();
    open_diary(&home);
    refuses_past_the_door(
        &home,
        "doc_create",
        json!({ "body": "# New\n", "page_of": home.diary }),
    );
}

#[test]
fn past_the_door_write_doc_translation_of_the_diary() {
    let home = home();
    open_diary(&home);
    refuses_past_the_door(
        &home,
        "doc_create",
        json!({ "body": "# 2026\n", "translation_of": home.diary, "lang": "es" }),
    );
}

#[test]
fn past_the_door_tongue_doc_the_diary_joins_another_doc() {
    let home = home();
    open_diary(&home);
    refuses_past_the_door(
        &home,
        "doc_organize",
        json!({ "action": "link_translation", "docs": [home.diary], "parent": home.plain, "lang": "en" }),
    );
}

#[test]
fn past_the_door_tongue_doc_a_doc_joins_the_diary() {
    let home = home();
    open_diary(&home);
    refuses_past_the_door(
        &home,
        "doc_organize",
        json!({ "action": "link_translation", "docs": [home.plain], "parent": home.diary }),
    );
}

#[test]
fn past_the_door_import_doc_as_a_page_of_the_diary() {
    let home = home();
    open_diary(&home);
    refuses_past_the_door(
        &home,
        "file_transfer",
        json!({ "action": "import_doc", "path": home.picture, "page_of": home.diary }),
    );
}
