const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const BASE: usize = 62;

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

const LONG: usize = 20;

/// Squeezing a key in before another lengthens it, and a run reordered from its text is
/// squeezed on every move. Past a length nothing legitimate reaches, the whole run is dealt
/// fresh keys instead: it costs one event per neighbour, once, rather than a key that grows
/// for ever and is carried in every event, every row and every replay after it.
pub fn resequenced(keys: &[&str]) -> Vec<Option<String>> {
    let fresh = squeezed(keys);
    let long = fresh.iter().flatten().any(|one| one.len() > LONG);
    match long || !rises(keys, &fresh) {
        true => afresh(keys),
        false => fresh,
    }
}

/// Between two keys that came from somewhere else there may be no key at all to hand out, and
/// a run that does not rise is not an order. Dealing it again is always an answer.
fn rises(keys: &[&str], fresh: &[Option<String>]) -> bool {
    let settled: Vec<&str> = keys
        .iter()
        .zip(fresh)
        .map(|(had, now)| now.as_deref().unwrap_or(had))
        .collect();
    settled.windows(2).all(|two| two[0] < two[1])
}

/// Every slot, even one already holding the key it is dealt: dealing the run again re-bases the
/// whole of it, and half a re-base merged with another machine's reads as neither of the two.
fn afresh(keys: &[&str]) -> Vec<Option<String>> {
    let mut last = String::new();
    keys.iter()
        .map(|_| {
            last = if last.is_empty() {
                first()
            } else {
                after(&last)
            };
            Some(last.clone())
        })
        .collect()
}

fn squeezed(keys: &[&str]) -> Vec<Option<String>> {
    let held = rising(keys);
    let mut fresh = vec![None; keys.len()];
    let mut last: Option<String> = None;
    for i in 0..keys.len() {
        if held[i] {
            last = Some(keys[i].to_string());
            continue;
        }
        let ceiling = keys[i + 1..]
            .iter()
            .zip(&held[i + 1..])
            .find(|(_, held)| **held)
            .map(|(key, _)| *key)
            // A key from elsewhere can sit below the one before it, and there is no between then.
            .filter(|up| last.as_deref().is_none_or(|low| low < *up));
        let key = between(last.as_deref(), ceiling);
        last = Some(key.clone());
        fresh[i] = Some(key);
    }
    fresh
}

fn rising(keys: &[&str]) -> Vec<bool> {
    let mut reach = vec![1usize; keys.len()];
    let mut from = vec![usize::MAX; keys.len()];
    let (mut longest, mut end) = (0, usize::MAX);
    for i in 0..keys.len() {
        for j in 0..i {
            if keys[j] < keys[i] && reach[j] + 1 > reach[i] {
                reach[i] = reach[j] + 1;
                from[i] = j;
            }
        }
        if reach[i] > longest {
            longest = reach[i];
            end = i;
        }
    }
    let mut held = vec![false; keys.len()];
    while end != usize::MAX {
        held[end] = true;
        end = from[end];
    }
    held
}

fn midpoint(a: &str, b: Option<&str>) -> String {
    let Some(b) = b else {
        return append_after(a);
    };

    let mut shared = b
        .bytes()
        .enumerate()
        .take_while(|(i, y)| a.as_bytes().get(*i).copied().unwrap_or(DIGITS[0]) == *y)
        .count();
    // Counted in bytes and cut as text: a key from elsewhere can share half a character. The
    // count runs past the end of `a` on purpose, and there is nothing to cut out there.
    while shared > 0
        && !((shared > a.len() || a.is_char_boundary(shared)) && b.is_char_boundary(shared))
    {
        shared -= 1;
    }
    if shared > 0 {
        return format!(
            "{}{}",
            &b[..shared],
            midpoint(tail(a, shared), Some(&b[shared..]))
        );
    }

    let low = a.bytes().next().map_or(0, index);
    let high = b.bytes().next().map_or(BASE, index);

    // A byte this alphabet does not know reads as nought, which can sit below `low`.
    if high > low + 1 {
        return digit((low + high) / 2);
    }
    let head = b.chars().next().map_or(1, char::len_utf8);
    if b.len() > head {
        return b[..head].to_string();
    }
    format!("{}{}", digit(low), midpoint(tail(a, head), None))
}

fn append_after(a: &str) -> String {
    // A byte outside the alphabet reads as nought, and raising it would sort the key downwards.
    let Some(i) = a
        .bytes()
        .all(|d| DIGITS.contains(&d))
        .then(|| a.bytes().rposition(|d| index(d) + 1 < BASE))
        .flatten()
    else {
        return format!("{a}{}", digit(BASE / 2));
    };
    format!("{}{}", &a[..i], digit(index(a.as_bytes()[i]) + 1))
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
    fn appending_does_not_grow_the_key_out_of_hand() {
        let mut key = first();
        for n in 0..5_000 {
            let next = after(&key);
            assert!(next > key, "{next} is not after {key} at {n}");
            key = next;
        }
        assert!(key.len() <= 200, "{} chars after 5000 appends", key.len());
    }

    fn settled(keys: &[&str]) -> Vec<String> {
        let fresh = resequenced(keys);
        keys.iter()
            .zip(fresh)
            .map(|(had, now)| now.unwrap_or_else(|| (*had).to_string()))
            .collect()
    }

    #[test]
    fn a_run_already_in_order_keeps_every_key() {
        assert!(resequenced(&["a", "b", "c"]).iter().all(Option::is_none));
    }

    #[test]
    fn moving_one_to_the_front_only_rekeys_that_one() {
        let fresh = resequenced(&["c", "a", "b"]);
        assert_eq!(fresh.iter().filter(|one| one.is_some()).count(), 1);
        let now = settled(&["c", "a", "b"]);
        assert!(now[0] < now[1] && now[1] < now[2], "{now:?}");
    }

    #[test]
    fn a_run_turned_around_still_reads_in_the_order_asked_for() {
        let now = settled(&["e", "d", "c", "b", "a"]);
        assert!(now.windows(2).all(|two| two[0] < two[1]), "{now:?}");
    }

    #[test]
    fn a_key_repeated_is_pushed_past_its_twin() {
        let now = settled(&["a", "a"]);
        assert!(now[0] < now[1], "{now:?}");
    }

    #[test]
    fn a_key_this_machine_never_wrote_is_ordered_rather_than_panicked_on() {
        for run in [
            vec!["é", "ê", "a"],
            vec!["é", "ê", "a", "ë"],
            vec!["ñ", "ña", "ñb", "b", "ñ"],
            vec!["日", "本", "a"],
        ] {
            let now = settled(&run);
            assert!(now.windows(2).all(|two| two[0] < two[1]), "{run:?} {now:?}");
        }

        assert!(after("ñ").as_str() > "ñ");
    }

    #[test]
    fn a_key_from_an_alphabet_this_one_does_not_know_is_dealt_around_rather_than_crashed_on() {
        for run in [
            vec!["z", "9", "é"],
            vec!["a0", "9", "ñ"],
            vec!["z", "a", "é"],
            vec!["V", "A", "é"],
            vec!["z", "9", "日"],
        ] {
            let now = settled(&run);
            assert!(now.windows(2).all(|two| two[0] < two[1]), "{run:?} {now:?}");
        }
    }

    #[test]
    fn what_follows_a_key_this_alphabet_does_not_know_still_follows_it() {
        for key in ["~", "_", ":", "a~", "z~", "\u{7f}", "é", " "] {
            let next = after(key);
            assert!(next.as_str() > key, "after({key:?}) = {next:?}");
        }
        assert!(last_of(["V", "W", "~"]).as_str() > "~");
    }

    #[test]
    fn a_run_whose_keys_do_not_rise_is_dealt_again_rather_than_asserted_at() {
        let now = settled(&["é", "_", "\u{1c}", "1"]);

        assert!(now.windows(2).all(|two| two[0] < two[1]), "{now:?}");
    }

    #[test]
    fn a_run_with_no_room_left_between_its_keys_is_dealt_again_rather_than_left_unordered() {
        let fresh = resequenced(&["é", "ê", "a", "ë"]);

        assert!(
            fresh.iter().all(Option::is_some),
            "nothing can be squeezed between those, so the whole run is dealt: {fresh:?}"
        );
        let now: Vec<&str> = fresh.iter().map(|one| one.as_deref().unwrap()).collect();
        assert!(now.windows(2).all(|two| two[0] < two[1]), "{now:?}");
    }

    #[test]
    fn a_run_moved_about_for_ever_does_not_grow_keys_without_end() {
        let mut keys: Vec<String> = vec![first()];
        for _ in 0..7 {
            keys.push(after(keys.last().unwrap()));
        }

        for _ in 0..20_000 {
            let mut wanted: Vec<&str> = keys.iter().map(String::as_str).collect();
            let last = wanted.pop().unwrap();
            wanted.insert(0, last);
            let fresh = resequenced(&wanted);
            keys = wanted
                .iter()
                .zip(fresh)
                .map(|(had, now)| now.unwrap_or_else(|| (*had).to_string()))
                .collect();
            assert!(keys.windows(2).all(|two| two[0] < two[1]), "{keys:?}");
        }

        let longest = keys.iter().map(String::len).max().unwrap();
        assert!(
            longest <= LONG + 1,
            "{longest} characters after 20000 moves"
        );
    }

    #[test]
    fn an_empty_run_asks_for_nothing() {
        assert!(resequenced(&[]).is_empty());
    }

    #[test]
    fn keys_that_did_not_come_from_here_still_work() {
        let mid = between(Some("a0"), Some("a1"));
        assert!("a0" < mid.as_str() && mid.as_str() < "a1", "{mid}");
        assert!(before("a0").as_str() < "a0");
    }
}
