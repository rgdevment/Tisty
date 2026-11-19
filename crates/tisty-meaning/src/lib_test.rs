use super::*;

const SMALL_VOCAB: &str = "[PAD]\n[UNK]\nsol\nluna";

fn table(rows: &[(f32, &[i8])]) -> Vec<u8> {
    let dims = rows.first().map_or(0, |(_, values)| values.len());
    let mut bytes = Vec::new();
    bytes.extend(u32::try_from(rows.len()).unwrap().to_le_bytes());
    bytes.extend(u32::try_from(dims).unwrap().to_le_bytes());
    for (scale, _) in rows {
        bytes.extend(scale.to_le_bytes());
    }
    for (_, values) in rows {
        bytes.extend(values.iter().map(|one| one.to_le_bytes()[0]));
    }
    bytes
}

fn small() -> Model {
    Model::read(
        table(&[
            (1.0, &[0, 0]),
            (1.0, &[0, 0]),
            (0.5, &[2, 0]),
            (1.0 / 3.0, &[0, -3]),
        ]),
        SMALL_VOCAB,
    )
    .unwrap()
}

#[test]
fn the_shipped_table_means_what_the_published_model_means() {
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../app/src-tauri/resources/brain");
    let model = Model::read(
        std::fs::read(at.join("table.bin")).unwrap(),
        &std::fs::read_to_string(at.join("vocab.txt")).unwrap(),
    )
    .unwrap();
    assert_eq!(model.dims(), 256);
    let mut read = 0;
    for line in include_str!("../fixtures/meanings.jsonl").lines() {
        let said: serde_json::Value = serde_json::from_str(line).unwrap();
        let text = said["text"].as_str().unwrap();
        let wanted: Vec<f32> = serde_json::from_value(said["meaning"].clone()).unwrap();
        let meant = model.meaning(text);
        let worst = meant
            .iter()
            .zip(&wanted)
            .map(|(a, b)| (a - b).abs())
            .fold(0f32, f32::max);
        assert!(worst < 1e-5, "«{text}» is {worst} away");
        read += 1;
    }
    assert!(read >= 20);
}

#[test]
fn a_text_means_the_mean_of_its_pieces_at_unit_length() {
    let model = small();
    assert_eq!(model.meaning("sol"), vec![1.0, 0.0]);
    assert_eq!(model.meaning("luna"), vec![0.0, -1.0]);
    let both = model.meaning("sol luna");
    assert!((both[0] - 1.0 / 2f32.sqrt()).abs() < 1e-6);
    assert!((both[1] + 1.0 / 2f32.sqrt()).abs() < 1e-6);
}

#[test]
fn a_text_with_no_pieces_means_nothing() {
    assert_eq!(small().meaning(""), vec![0.0, 0.0]);
    assert_eq!(small().meaning("estrella"), vec![0.0, 0.0]);
    assert_eq!(small().pieces("sol estrella"), vec![2, 1]);
}

#[test]
fn a_table_shorter_than_its_head_is_refused() {
    assert!(Model::read(vec![1, 0, 0], SMALL_VOCAB).is_err());
}

#[test]
fn a_table_whose_length_disagrees_with_its_head_is_refused() {
    let mut bytes = table(&[(1.0, &[0]), (1.0, &[0]), (1.0, &[1]), (1.0, &[2])]);
    bytes.pop();
    let refused = Model::read(bytes, SMALL_VOCAB).err().unwrap();
    assert_eq!(
        refused.to_string(),
        "the table is not as long as its head says"
    );
}

#[test]
fn a_table_without_dimensions_is_refused() {
    let mut bytes = Vec::new();
    bytes.extend(4u32.to_le_bytes());
    bytes.extend(0u32.to_le_bytes());
    bytes.extend([0u8; 16]);
    assert!(Model::read(bytes, SMALL_VOCAB).is_err());
}

#[test]
fn a_table_naming_more_rows_than_fit_is_refused() {
    let mut bytes = Vec::new();
    bytes.extend(u32::MAX.to_le_bytes());
    bytes.extend(u32::MAX.to_le_bytes());
    assert!(Model::read(bytes, SMALL_VOCAB).is_err());
}

#[test]
fn a_vocabulary_of_another_length_is_refused() {
    let bytes = table(&[(1.0, &[0]), (1.0, &[0])]);
    let refused = Model::read(bytes, SMALL_VOCAB).err().unwrap();
    assert_eq!(
        refused.to_string(),
        "the vocabulary and the table differ in length"
    );
}

#[test]
fn a_vocabulary_without_its_unknown_piece_is_refused() {
    let bytes = table(&[(1.0, &[0]), (1.0, &[0])]);
    assert!(Model::read(bytes, "sol\nluna").is_err());
}
