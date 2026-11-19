use std::collections::HashMap;

use super::likeness::{Caps, Found, Item, Near, Pairs, Spread, Tongue, floors, mutual};
use super::meant::Meant;

#[derive(Default, Clone, Copy)]
struct Standing {
    sum: f64,
    squares: f64,
    count: u32,
}

impl Standing {
    fn add(&mut self, said: f32) {
        self.sum += f64::from(said);
        self.squares += f64::from(said) * f64::from(said);
        self.count += 1;
    }

    // How far a score stands above what this node scores with anything; a node alike everything alike nothing.
    fn standout(&self, said: f32) -> f64 {
        if self.count < 2 {
            return 0.0;
        }
        let n = f64::from(self.count);
        let mean = self.sum / n;
        let spread = (self.squares / n - mean * mean).max(0.0).sqrt();
        match spread > f64::EPSILON {
            true => (f64::from(said) - mean) / spread,
            false => 0.0,
        }
    }
}

// Past the budget each node meets only the next `reach` nodes around the base, so the pass stays bounded.
pub(crate) fn by_meaning(
    items: &[Item],
    apart: &impl Fn(usize, usize) -> bool,
    caps: &Caps,
) -> (Vec<Found>, u64) {
    let meant: Vec<(usize, &Meant)> = items
        .iter()
        .enumerate()
        .filter_map(|(at, item)| Some((at, item.meant.as_ref()?)))
        .filter(|(_, meant)| meant.says_something())
        .collect();
    let m = meant.len();
    if m < 2 {
        return (Vec::new(), 0);
    }
    let spoken = spoken(items, &meant);
    let dims = meant[0].1.values().len();
    let wide: Vec<i16> = meant
        .iter()
        .flat_map(|(_, one)| (0..dims).map(|at| one.values().get(at).copied().map_or(0, i16::from)))
        .collect();
    let scales: Vec<f32> = meant.iter().map(|(_, one)| one.scale()).collect();
    let row = |at: usize| &wide[at * dims..(at + 1) * dims];
    let rooms = [caps.closest, caps.abroad];
    let mut ranked: Vec<Near> = vec![[Vec::new(), Vec::new()]; m];
    let mut standing = vec![Standing::default(); m];
    let mut spread = [Spread::default(), Spread::default()];
    let mut compared = 0;
    let mut weigh = |one: usize, other: usize| {
        let said = dot(row(one), row(other)) as f32 * scales[one] * scales[other];
        let tongue = match (spoken[one], spoken[other]) {
            (Some(mine), Some(theirs)) if mine != theirs => Tongue::Other,
            _ => Tongue::Same,
        } as usize;
        spread[tongue].add_times(said, 2);
        standing[one].add(said);
        standing[other].add(said);
        compared += 1;
        let room = rooms[tongue];
        let here = admits(&ranked[one][tongue], room, other, said);
        let there = admits(&ranked[other][tongue], room, one, said);
        if (here || there) && !apart(meant[one].0, meant[other].0) {
            if here {
                offer(&mut ranked[one][tongue], room, other, said);
            }
            if there {
                offer(&mut ranked[other][tongue], room, one, said);
            }
        }
    };
    let whole = (m as u64) * (m as u64 - 1) / 2;
    if whole <= caps.pairs || m < 3 {
        for one in 0..m {
            for other in one + 1..m {
                weigh(one, other);
            }
        }
    } else {
        let reach = usize::try_from(caps.pairs / m as u64)
            .unwrap_or(usize::MAX)
            .clamp(1, (m - 1) / 2);
        for one in 0..m {
            for step in 1..=reach {
                weigh(one, (one + step) % m);
            }
        }
    }
    let langs: Vec<Option<&str>> = meant
        .iter()
        .map(|(at, _)| items[*at].lang.as_deref())
        .collect();
    // Within one language the base is crowded with near twins, so a pair must stand out for both ends instead.
    let [_, abroad] = floors(&langs, &spread, Pairs::Compared, caps);
    let found = mutual(&ranked, &[f32::MIN, abroad], |one| one)
        .into_iter()
        .filter(|(tongue, one, other, said)| {
            *tongue == Tongue::Other
                || standing[*one]
                    .standout(*said)
                    .min(standing[*other].standout(*said))
                    >= caps.standout
        })
        .map(|(tongue, one, other, said)| (tongue, meant[one].0, meant[other].0, said))
        .collect();
    (found, compared)
}

fn dot(one: &[i16], other: &[i16]) -> i32 {
    one.iter()
        .zip(other)
        .map(|(mine, theirs)| i32::from(*mine) * i32::from(*theirs))
        .sum()
}

fn spoken(items: &[Item], meant: &[(usize, &Meant)]) -> Vec<Option<usize>> {
    let mut named: HashMap<&str, usize> = HashMap::new();
    meant
        .iter()
        .map(|(at, _)| {
            let lang = items[*at].lang.as_deref()?;
            let next = named.len();
            Some(*named.entry(lang).or_insert(next))
        })
        .collect()
}

fn admits(list: &[(usize, f32)], room: usize, by: usize, said: f32) -> bool {
    match list.last() {
        _ if list.len() < room => true,
        Some((last, lowest)) => said > *lowest || (said == *lowest && by < *last),
        None => false,
    }
}

fn offer(list: &mut Vec<(usize, f32)>, room: usize, by: usize, said: f32) {
    let at = list
        .iter()
        .position(|(other, theirs)| said > *theirs || (said == *theirs && by < *other))
        .unwrap_or(list.len());
    list.insert(at, (by, said));
    list.truncate(room);
}

#[cfg(test)]
#[path = "nearness_test.rs"]
mod tests;
