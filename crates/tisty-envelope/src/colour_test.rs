use super::*;

const SECRET: &[u8] = b"taken at home, Calle Larga 12";

fn xyz(value: u8) -> Vec<u8> {
    let mut out = b"XYZ \0\0\0\0".to_vec();
    out.extend_from_slice(&[value; 12]);
    out
}

fn gamma() -> Vec<u8> {
    let mut out = b"curv\0\0\0\0\0\0\0\x01".to_vec();
    out.extend_from_slice(&[2, 51]);
    out
}

fn a_profile(extra: &[(&[u8; 4], Vec<u8>)]) -> Vec<u8> {
    let mut tags: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"rXYZ", xyz(1)),
        (b"gXYZ", xyz(2)),
        (b"bXYZ", xyz(3)),
        (b"rTRC", gamma()),
        (b"gTRC", gamma()),
        (b"bTRC", gamma()),
    ];
    tags.extend_from_slice(extra);
    let mut out = vec![0u8; HEADER];
    out[8..12].copy_from_slice(&[2, 64, 0, 0]);
    out[12..16].copy_from_slice(b"mntr");
    out[16..20].copy_from_slice(b"RGB ");
    out[20..24].copy_from_slice(b"XYZ ");
    out[24..36].copy_from_slice(&[7; 12]);
    out[36..40].copy_from_slice(b"acsp");
    out[48..56].copy_from_slice(b"maker123");
    out[80..84].copy_from_slice(b"mine");
    out.extend_from_slice(&u32::try_from(tags.len()).unwrap().to_be_bytes());
    let mut at = HEADER + 4 + 12 * tags.len();
    for (name, values) in &tags {
        out.extend_from_slice(*name);
        out.extend_from_slice(&u32::try_from(at).unwrap().to_be_bytes());
        out.extend_from_slice(&u32::try_from(values.len()).unwrap().to_be_bytes());
        at += values.len();
    }
    for (_, values) in &tags {
        out.extend_from_slice(values);
    }
    out
}

fn holds(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|one| one == needle)
}

fn tag_of<'a>(profile: &'a [u8], name: &[u8; 4]) -> Option<&'a [u8]> {
    let entries = number_at(profile, HEADER)?;
    profile[HEADER + 4..]
        .as_chunks::<12>()
        .0
        .iter()
        .take(entries)
        .find(|entry| &entry[..4] == name)
        .and_then(|entry| {
            let (at, length) = (number_at(entry, 4)?, number_at(entry, 8)?);
            profile.get(at..at + length)
        })
}

#[test]
fn its_colours_stay_and_they_read_as_before() {
    let done = colour_only(&a_profile(&[])).unwrap();
    assert_eq!(number_at(&done, 0), Some(done.len()));
    assert_eq!(&done[36..40], b"acsp");
    assert_eq!(&done[16..20], b"RGB ");
    assert_eq!(tag_of(&done, b"gXYZ"), Some(xyz(2).as_slice()));
    assert_eq!(tag_of(&done, b"bTRC"), Some(gamma().as_slice()));
}

#[test]
fn nothing_else_a_profile_held_travels() {
    let mut desc = b"desc\0\0\0\0".to_vec();
    desc.extend_from_slice(SECRET);
    let mut riding = xyz(9);
    riding.extend_from_slice(SECRET);
    let mut said = a_profile(&[(b"desc", desc), (b"wtpt", riding)]);
    said.extend_from_slice(SECRET);
    assert!(holds(&said, SECRET));
    let done = colour_only(&said).unwrap();
    assert!(!holds(&done, SECRET));
    assert!(!holds(&done, b"maker123"));
    assert!(!holds(&done, b"mine"));
    assert!(tag_of(&done, b"desc").is_none());
    assert_eq!(tag_of(&done, b"wtpt"), Some(xyz(9).as_slice()));
    assert!(done[24..36].iter().all(|one| *one == 0));
}

#[test]
fn what_says_no_colours_this_way_goes_without_one() {
    let real = a_profile(&[]);
    let mut gray = real.clone();
    gray[16..20].copy_from_slice(b"GRAY");
    assert_eq!(colour_only(&gray), None);
    let mut cmyk = real.clone();
    cmyk[16..20].copy_from_slice(b"CMYK");
    assert_eq!(colour_only(&cmyk), None);
    assert_eq!(colour_only(b"a text about where this was taken"), None);
    let mut large = real.clone();
    large.resize(PROFILE_AT_MOST + 1, 0);
    assert_eq!(colour_only(&large), None);
    let mut no_curves = real;
    let at = HEADER + 4 + 12 * 3;
    no_curves[at..at + 4].copy_from_slice(b"none");
    assert_eq!(colour_only(&no_curves), None);
}

#[test]
fn a_table_that_lies_opens_nothing() {
    let mut past_its_end = a_profile(&[]);
    let at = HEADER + 4 + 4;
    past_its_end[at..at + 4].copy_from_slice(&u32::MAX.to_be_bytes());
    assert_eq!(colour_only(&past_its_end), None);
    let mut other_kind = a_profile(&[]);
    let first = number_at(&other_kind, at).unwrap();
    other_kind[first..first + 4].copy_from_slice(b"text");
    assert_eq!(colour_only(&other_kind), None);
}
