use std::ops::Range;

const PROFILE_AT_MOST: usize = 64 * 1024;
const HEADER: usize = 128;
const CURVE_POINTS_AT_MOST: usize = 4096;
// Its version, class and spaces, its signature, and its intent and illuminant. The rest of a header says who made it, when and on what.
const HEADER_KEPT: [Range<usize>; 3] = [8..24, 36..40, 64..80];
// What a reader needs to turn its pixels into colours. What else a profile holds, names and descriptions, can be anything.
const NEEDED: [&[u8; 4]; 6] = [b"rXYZ", b"gXYZ", b"bXYZ", b"rTRC", b"gTRC", b"bTRC"];
const HELPFUL: [&[u8; 4]; 2] = [b"wtpt", b"chad"];

fn number_at(said: &[u8], at: usize) -> Option<usize> {
    let bytes = said.get(at..at + 4)?;
    Some(u32::from_be_bytes(bytes.try_into().ok()?) as usize)
}

// Only as long as its kind says, so nothing can ride on a tag past its values.
fn values_of(data: &[u8]) -> Option<&[u8]> {
    let length = match data.get(..4)? {
        b"XYZ " => 20,
        b"sf32" => 44,
        b"curv" => {
            let points = number_at(data, 8)?;
            if points > CURVE_POINTS_AT_MOST {
                return None;
            }
            12 + 2 * points
        }
        b"para" => {
            let kind = u16::from_be_bytes(data.get(8..10)?.try_into().ok()?);
            12 + 4 * [1, 3, 4, 5, 7].get(usize::from(kind))?
        }
        _ => return None,
    };
    data.get(..length)
}

/// Of a profile for three channels, only what says how its pixels look, written anew: a profile can hold anything else, and only pixels travel.
pub fn colour_only(said: &[u8]) -> Option<Vec<u8>> {
    if said.len() > PROFILE_AT_MOST
        || said.get(36..40) != Some(b"acsp".as_slice())
        || said.get(16..20) != Some(b"RGB ".as_slice())
    {
        return None;
    }
    let table = said.get(HEADER + 4..)?;
    let entries = number_at(said, HEADER)?;
    let mut tags: Vec<(&[u8], &[u8])> = Vec::new();
    for entry in table.as_chunks::<12>().0.iter().take(entries) {
        let (name, place) = entry.split_at(4);
        if NEEDED.iter().chain(&HELPFUL).any(|one| *one == name) {
            let (at, length) = (number_at(place, 0)?, number_at(place, 4)?);
            tags.push((name, values_of(said.get(at..at.checked_add(length)?)?)?));
        }
    }
    if !NEEDED
        .iter()
        .all(|one| tags.iter().any(|(name, _)| one == name))
    {
        return None;
    }
    let mut out = vec![0u8; HEADER];
    for kept in HEADER_KEPT {
        out[kept.clone()].copy_from_slice(&said[kept]);
    }
    out.extend_from_slice(&u32::try_from(tags.len()).ok()?.to_be_bytes());
    let mut at = HEADER + 4 + 12 * tags.len();
    // The three curves are most often one, kept once as it came.
    let mut held: Vec<(&[u8], usize)> = Vec::new();
    for (name, values) in &tags {
        let found = held.iter().find(|(one, _)| one == values);
        let place = found.map_or(at, |(_, place)| *place);
        if found.is_none() {
            held.push((values, at));
            at += values.len().next_multiple_of(4);
        }
        out.extend_from_slice(name);
        out.extend_from_slice(&u32::try_from(place).ok()?.to_be_bytes());
        out.extend_from_slice(&u32::try_from(values.len()).ok()?.to_be_bytes());
    }
    for (values, _) in &held {
        out.extend_from_slice(values);
        out.resize(out.len().next_multiple_of(4), 0);
    }
    let size = u32::try_from(out.len()).ok()?.to_be_bytes();
    out[..4].copy_from_slice(&size);
    Some(out)
}

#[cfg(test)]
#[path = "colour_test.rs"]
mod tests;
