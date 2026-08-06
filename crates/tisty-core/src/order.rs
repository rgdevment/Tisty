//! Fractional indexing: positions are strings, so an insert never renumbers
//! the rest and two devices can insert at the same spot without conflict.

const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const BASE: usize = 62;

/// Keys never end in the lowest digit, or there is no room to insert before them.
pub fn between(before: Option<&str>, after: Option<&str>) -> String {
    let (a, b) = (before.unwrap_or(""), after.filter(|s| !s.is_empty()));
    debug_assert!(b.is_none_or(|b| a < b), "{a} is not before {b:?}");
    midpoint(a, b)
}

pub fn first() -> String {
    between(None, None)
}

pub fn after(key: &str) -> String {
    between(Some(key), None)
}

pub fn before(key: &str) -> String {
    between(None, Some(key))
}

pub fn last_of<'a>(keys: impl IntoIterator<Item = &'a str>) -> String {
    match keys.into_iter().max() {
        Some(k) => after(k),
        None => first(),
    }
}

fn midpoint(a: &str, b: Option<&str>) -> String {
    if let Some(b) = b {
        // Past the end of `a`, compare against the lowest digit, not nothing.
        let shared = b
            .bytes()
            .enumerate()
            .take_while(|(i, y)| a.as_bytes().get(*i).copied().unwrap_or(DIGITS[0]) == *y)
            .count();
        if shared > 0 {
            return format!(
                "{}{}",
                &b[..shared],
                midpoint(tail(a, shared), Some(&b[shared..]))
            );
        }
    }

    let low = a.bytes().next().map_or(0, index);
    let high = b.and_then(|b| b.bytes().next()).map_or(BASE, index);

    if high - low > 1 {
        return digit((low + high) / 2);
    }
    match b {
        Some(b) if b.len() > 1 => b[..1].to_string(),
        _ => format!("{}{}", digit(low), midpoint(tail(a, 1), None)),
    }
}

fn tail(s: &str, from: usize) -> &str {
    s.get(from..).unwrap_or("")
}

fn index(byte: u8) -> usize {
    DIGITS.iter().position(|d| *d == byte).unwrap_or(0)
}

fn digit(i: usize) -> String {
    (DIGITS[i.min(BASE - 1)] as char).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_lands_between_its_neighbours() {
        let a = first();
        let b = after(&a);
        let mid = between(Some(&a), Some(&b));

        assert!(a < mid, "{a} < {mid}");
        assert!(mid < b, "{mid} < {b}");
    }

    #[test]
    fn appending_keeps_growing() {
        let mut keys = vec![first()];
        for _ in 0..100 {
            keys.push(after(keys.last().unwrap()));
        }
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
    }

    #[test]
    fn splitting_the_same_gap_stays_ordered() {
        let (low, high) = (first(), after(&first()));
        let mut upper = high.clone();

        for _ in 0..100 {
            let mid = between(Some(&low), Some(&upper));
            assert!(low < mid && mid < upper, "{low} < {mid} < {upper}");
            upper = mid;
        }
    }

    #[test]
    fn prepending_keeps_shrinking() {
        let mut key = first();
        for _ in 0..50 {
            let earlier = before(&key);
            assert!(earlier < key, "{earlier} < {key}");
            key = earlier;
        }
    }

    #[test]
    fn a_shared_prefix_is_preserved() {
        let a = "aV";
        let b = "aW";
        let mid = between(Some(a), Some(b));
        assert!(a < mid.as_str() && mid.as_str() < b, "{mid}");
    }

    #[test]
    fn the_end_of_a_run_is_found_regardless_of_input_order() {
        let keys = ["V", "l", "b"];
        let next = last_of(keys);
        assert!(keys.iter().all(|k| *k < next.as_str()), "{next}");
    }

    #[test]
    fn an_empty_run_starts_somewhere_insertable() {
        let start = last_of(std::iter::empty());
        assert!(before(&start) < start);
        assert!(after(&start) > start);
    }

    #[test]
    fn keys_that_did_not_come_from_here_still_work() {
        let mid = between(Some("a0"), Some("a1"));
        assert!("a0" < mid.as_str() && mid.as_str() < "a1", "{mid}");
        assert!(before("a0").as_str() < "a0");
    }
}
