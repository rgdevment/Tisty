use super::*;

#[test]
fn the_catalog_reads_and_names_each_assistant_once() {
    let all = all();
    assert!(all.len() >= 25, "{}", all.len());
    let mut ids: Vec<&str> = all.iter().map(|it| it.id.as_str()).collect();
    ids.sort_unstable();
    let before = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), before);
}

#[test]
fn every_assistant_has_a_file_on_each_system_it_runs_on() {
    let here = Place::here("tisty".to_string()).unwrap();
    for client in all() {
        let any = client.everywhere.len()
            + client.windows.len()
            + client.macos.len()
            + client.linux.len();
        assert!(any > 0, "{} has nowhere to write", client.id);
    }
    assert!(all().iter().any(|client| client.file(&here).is_some()));
}
