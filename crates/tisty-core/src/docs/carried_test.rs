use super::*;

#[test]
fn a_reader_that_never_ends_is_refused_at_the_ceiling() {
    assert!(read_within(std::io::repeat(b'x'), 10).is_err());
}

#[test]
fn a_reader_right_up_to_the_ceiling_is_read_whole() {
    assert_eq!(
        read_within(&b"0123456789"[..], 10).unwrap(),
        b"0123456789".to_vec()
    );
}

#[test]
fn a_reader_one_byte_past_the_ceiling_is_refused() {
    assert!(read_within(&b"0123456789x"[..], 10).is_err());
}

#[test]
fn a_body_past_the_ceiling_is_refused_without_being_opened_to_be_read() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("huge.md");
    std::fs::write(&at, vec![b'x'; (BODY_AT_MOST + 1) as usize]).unwrap();
    crate::counting::from_now();

    assert!(whole_of(&at).is_err());

    assert_eq!(
        crate::counting::from_now(),
        0,
        "a body past the ceiling was read to find that out"
    );
}

#[test]
fn the_same_folder_come_back_with_another_mark_keeps_what_was_carried() {
    let data_room = tempfile::tempdir().unwrap();
    let shared = tempfile::tempdir().unwrap();
    let mut said = Carried {
        prints: [("dev_a-0001".to_string(), "huella".to_string())].into(),
        up_to: Some(format!("1:2\n{}", shared.path().display())),
        ..Carried::default()
    };

    assert!(!said.facing(data_room.path(), shared.path(), false, true));

    assert_eq!(said.of("dev_a-0001"), Some("huella"));
    assert_eq!(
        said.up_to.as_deref(),
        Some(crate::paths::told_of(shared.path()).as_str())
    );
}

#[test]
fn the_same_path_holding_another_store_forgets_what_was_carried() {
    let data_room = tempfile::tempdir().unwrap();
    let shared = tempfile::tempdir().unwrap();
    let mut said = Carried {
        prints: [("dev_a-0001".to_string(), "huella".to_string())].into(),
        up_to: Some(format!("1:2\n{}", shared.path().display())),
        ..Carried::default()
    };

    assert!(said.facing(data_room.path(), shared.path(), false, false));

    assert_eq!(said.of("dev_a-0001"), None);
}

#[test]
fn another_folder_forgets_what_was_carried() {
    let data_room = tempfile::tempdir().unwrap();
    let before = tempfile::tempdir().unwrap();
    let now = tempfile::tempdir().unwrap();
    let mut said = Carried {
        prints: [("dev_a-0001".to_string(), "huella".to_string())].into(),
        up_to: Some(crate::paths::told_of(before.path())),
        ..Carried::default()
    };

    assert!(said.facing(data_room.path(), now.path(), false, true));

    assert_eq!(said.of("dev_a-0001"), None);
}

fn carried_to(data: &Path, dest: &Path, id: &str, body: &str) -> Carried {
    let mut said = Carried::read(data);
    said.facing(data, dest, false, true);
    keep_carried(data, id, body).unwrap();
    said.keep(id, &crate::attach::printed(body.as_bytes()));
    said.save(data).unwrap();
    said
}

#[test]
fn carrying_to_the_folder_and_the_cloud_in_turns_keeps_the_base_of_each() {
    let room = tempfile::tempdir().unwrap();
    let data = room.path();
    let folder = tempfile::tempdir().unwrap();
    let cloud = tempfile::tempdir().unwrap();

    carried_to(
        data,
        folder.path(),
        "dev_a-0001",
        "# Carpeta
",
    );
    carried_to(
        data,
        cloud.path(),
        "dev_a-0001",
        "# Nube
",
    );
    for _ in 0..3 {
        let mut said = Carried::read(data);
        assert!(said.facing(data, folder.path(), false, true));
        assert_eq!(
            said.of("dev_a-0001"),
            Some(
                crate::attach::printed(
                    b"# Carpeta
"
                )
                .as_str()
            ),
            "the folder's print was lost while the cloud was carried to"
        );
        assert_eq!(
            read_carried(data, "dev_a-0001").as_deref(),
            Some(
                "# Carpeta
"
            ),
            "the folder's base body was deleted while the cloud was carried to"
        );
        said.save(data).unwrap();

        let mut said = Carried::read(data);
        assert!(said.facing(data, cloud.path(), false, true));
        assert_eq!(
            read_carried(data, "dev_a-0001").as_deref(),
            Some(
                "# Nube
"
            )
        );
        said.save(data).unwrap();
    }
}

#[test]
fn a_ledger_of_one_folder_reads_as_what_was_carried_to_that_folder() {
    let room = tempfile::tempdir().unwrap();
    let data = room.path();
    let folder = tempfile::tempdir().unwrap();
    let cloud = tempfile::tempdir().unwrap();
    std::fs::write(
        data.join("carried.json"),
        serde_json::json!({
            "prints": {"dev_a-0001": "huella"},
            "up_to": crate::paths::told_of(folder.path()),
        })
        .to_string(),
    )
    .unwrap();
    keep_carried(
        data,
        "dev_a-0001",
        "# Base
",
    )
    .unwrap();

    let mut said = Carried::read(data);
    assert!(!said.facing(data, folder.path(), false, true));
    assert_eq!(said.of("dev_a-0001"), Some("huella"));

    assert!(said.facing(data, cloud.path(), false, true));
    assert_eq!(said.of("dev_a-0001"), None);
    said.save(data).unwrap();

    let mut said = Carried::read(data);
    assert!(said.facing(data, folder.path(), false, true));
    assert_eq!(said.of("dev_a-0001"), Some("huella"));
    assert_eq!(
        read_carried(data, "dev_a-0001").as_deref(),
        Some(
            "# Base
"
        )
    );
}

#[test]
fn only_a_few_places_are_remembered_and_the_oldest_goes_with_its_bases() {
    let room = tempfile::tempdir().unwrap();
    let data = room.path();
    let places: Vec<_> = (0..=ELSEWHERE_AT_MOST + 1)
        .map(|_| tempfile::tempdir().unwrap())
        .collect();
    for (n, place) in places.iter().enumerate() {
        carried_to(
            data,
            place.path(),
            "dev_a-0001",
            &format!(
                "# {n}
"
            ),
        );
    }

    let mut said = Carried::read(data);
    assert!(said.facing(data, places[0].path(), false, true));
    assert_eq!(
        said.of("dev_a-0001"),
        None,
        "the oldest place was still kept"
    );
    assert_eq!(read_carried(data, "dev_a-0001"), None);
    said.save(data).unwrap();

    let mut said = Carried::read(data);
    assert!(said.facing(data, places[2].path(), false, true));
    assert_eq!(
        read_carried(data, "dev_a-0001").as_deref(),
        Some(
            "# 2
"
        )
    );
}

#[test]
fn forgetting_what_was_carried_forgets_it_for_every_place() {
    let room = tempfile::tempdir().unwrap();
    let data = room.path();
    let folder = tempfile::tempdir().unwrap();
    let cloud = tempfile::tempdir().unwrap();
    carried_to(
        data,
        folder.path(),
        "dev_a-0001",
        "# Carpeta
",
    );
    carried_to(
        data,
        cloud.path(),
        "dev_a-0001",
        "# Nube
",
    );

    forget_what_was_carried(data);

    let mut said = Carried::read(data);
    said.facing(data, folder.path(), false, true);
    assert_eq!(said.of("dev_a-0001"), None);
    assert_eq!(read_carried(data, "dev_a-0001"), None);
}
