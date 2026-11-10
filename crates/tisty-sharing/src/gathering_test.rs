use super::Carried;

#[test]
fn a_field_that_does_not_read_leaves_only_itself_out() {
    let said = r#"{"lang":"es","languages":[{"lang":"en","document":"e"}],"editors":[{"name":"Ana"}],"reviews":[{"seq":"uno"}]}"#;

    let carried: Carried = serde_json::from_str(said).unwrap();

    assert_eq!(carried.lang.as_deref(), Some("es"));
    assert_eq!(carried.languages.unwrap()[0].document, "e");
    assert!(carried.editors.is_none());
    assert!(carried.reviews.is_none());
}
