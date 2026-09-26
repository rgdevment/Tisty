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

pub fn slotted(keys: &[&str], stop: Option<usize>) -> String {
    match stop {
        None => last_of(keys.iter().copied()),
        Some(at) => between(
            at.checked_sub(1).map(|one| keys[one]),
            keys.get(at).copied(),
        ),
    }
}

pub fn dealt(keys: &[&str], stop: Option<usize>) -> (String, Vec<Option<String>>) {
    let mine = slotted(keys, stop);
    if mine.len() <= LONG {
        return (mine, vec![None; keys.len()]);
    }
    let at = stop.unwrap_or(keys.len());
    let mut run: Vec<&str> = keys.to_vec();
    run.insert(at, mine.as_str());
    let mut fresh = afresh(&run);
    let taken = fresh.remove(at).unwrap_or(mine);
    (taken, fresh)
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
#[path = "order_test.rs"]
mod tests;
