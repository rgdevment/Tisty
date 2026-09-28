use super::*;

#[test]
fn a_bare_folder_serialises_to_the_minimum() {
    let json = serde_json::to_string(&Folder::new(Ulid::generate(), "trabajo", "a0")).unwrap();

    assert!(!json.contains("parent"), "{json}");
    assert!(!json.contains("icon"), "{json}");
}

#[test]
fn a_document_with_no_folder_is_unfiled_rather_than_absent() {
    let kept = Kept {
        born_by: None,
        guest: false,
        title: None,
        bytes: None,
        wrote: None,
        made: None,
        made_by: None,
        wrote_by: None,
        by: None,
        tags: Vec::new(),
        id: Ulid::generate(),
        file: "a3f1-0001".into(),
        order: "a0".into(),
        folder: None,
        page_of: None,
        archived: false,
        locked: false,
        edited_by: None,
        flagged: None,
        folder_was: None,
    };
    let json = serde_json::to_string(&kept).unwrap();

    assert!(!json.contains("folder"), "{json}");
    assert!(!json.contains("archived"), "{json}");
    assert_eq!(serde_json::from_str::<Kept>(&json).unwrap(), kept);
}
