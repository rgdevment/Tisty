pub fn blocks(body: &str) -> Vec<String> {
    let body = body.strip_prefix('\u{feff}').unwrap_or(body);
    let mut out: Vec<String> = Vec::new();
    let mut held: Vec<&str> = Vec::new();
    let mut fenced: Option<String> = None;

    for line in body.lines() {
        let bare = line.trim_start();
        match &fenced {
            Some(mark) => {
                held.push(line);
                if bare.starts_with(mark.as_str()) {
                    fenced = None;
                }
                continue;
            }
            None => {
                if let Some(mark) = opening(bare) {
                    fenced = Some(mark);
                    held.push(line);
                    continue;
                }
            }
        }
        if line.trim().is_empty() {
            if !held.is_empty() {
                out.push(held.join("\n"));
                held.clear();
            }
            continue;
        }
        held.push(line);
    }
    if !held.is_empty() {
        out.push(held.join("\n"));
    }
    out
}

fn opening(bare: &str) -> Option<String> {
    for mark in ["```", "~~~"] {
        if bare.starts_with(mark) {
            return Some(mark.to_string());
        }
    }
    None
}

pub fn front_matter(body: &str) -> bool {
    let body = body.strip_prefix('\u{feff}').unwrap_or(body);
    let mut lines = body.lines().map(str::trim).filter(|one| !one.is_empty());
    lines.next() == Some("---") && lines.any(|one| one == "---")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Edit {
    from: usize,
    upto: usize,
    said: Vec<String>,
}

pub const CELLS_AT_MOST: usize = 16_000_000;

fn shared(base: &[String], other: &[String]) -> Vec<(usize, usize)> {
    let wide = other.len() + 1;
    let mut grid = vec![0u32; (base.len() + 1) * wide];
    for (a, one) in base.iter().enumerate() {
        for (b, two) in other.iter().enumerate() {
            grid[(a + 1) * wide + b + 1] = if one == two {
                grid[a * wide + b] + 1
            } else {
                grid[a * wide + b + 1].max(grid[(a + 1) * wide + b])
            };
        }
    }

    let mut pairs = Vec::new();
    let (mut a, mut b) = (base.len(), other.len());
    while a > 0 && b > 0 {
        if base[a - 1] == other[b - 1] {
            pairs.push((a - 1, b - 1));
            a -= 1;
            b -= 1;
        } else if grid[(a - 1) * wide + b] >= grid[a * wide + b - 1] {
            a -= 1;
        } else {
            b -= 1;
        }
    }
    pairs.reverse();
    pairs
}

fn edits(base: &[String], other: &[String]) -> Vec<Edit> {
    let pairs = shared(base, other);
    let mut out = Vec::new();
    let (mut a, mut b) = (0usize, 0usize);

    for (at, to) in pairs
        .iter()
        .copied()
        .chain(std::iter::once((base.len(), other.len())))
    {
        if at > a || to > b {
            out.push(Edit {
                from: a,
                upto: at,
                said: other[b..to].to_vec(),
            });
        }
        a = at + 1;
        b = to + 1;
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rift {
    pub was: Vec<String>,
    pub mine: Vec<String>,
    pub theirs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    Mine,
    Theirs,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    Kept(Vec<String>),
    One(Vec<String>),
    Torn(Rift),
}

fn plan(base: &str, mine: &str, theirs: &str) -> Option<Vec<Step>> {
    if front_matter(base) || front_matter(mine) || front_matter(theirs) {
        return None;
    }
    let base = blocks(base);
    let mine = blocks(mine);
    let theirs = blocks(theirs);
    if base.len().saturating_mul(mine.len()) > CELLS_AT_MOST
        || base.len().saturating_mul(theirs.len()) > CELLS_AT_MOST
    {
        return None;
    }
    let ours = edits(&base, &mine);
    let yours = edits(&base, &theirs);

    let mut all: Vec<(bool, Edit)> = ours
        .into_iter()
        .map(|one| (true, one))
        .chain(yours.into_iter().map(|one| (false, one)))
        .collect();
    all.sort_by_key(|(ours, one)| (one.from, one.upto, !*ours));

    let mut out = Vec::new();
    let mut at = 0usize;
    let mut seen = 0usize;

    while seen < all.len() {
        let mut upto = seen + 1;
        let mut ends = all[seen].1.upto;
        while upto < all.len() && touching(&all[seen..upto], &all[upto].1) {
            ends = ends.max(all[upto].1.upto);
            upto += 1;
        }
        let group = &all[seen..upto];
        let from = group.iter().map(|(_, one)| one.from).min()?;
        if from < at {
            return None;
        }
        if at < from {
            out.push(Step::Kept(base[at..from].to_vec()));
        }

        let ours = rebuilt(&base, from, ends, true, group);
        let yours = rebuilt(&base, from, ends, false, group);
        let both = group.iter().any(|(ours, _)| *ours) && group.iter().any(|(ours, _)| !*ours);

        if !both || ours == yours {
            let touched = group.iter().any(|(ours, _)| *ours);
            out.push(Step::One(if touched { ours } else { yours }));
        } else {
            out.push(Step::Torn(Rift {
                was: base[from..ends].to_vec(),
                mine: ours,
                theirs: yours,
            }));
        }
        at = ends;
        seen = upto;
    }
    out.push(Step::Kept(base[at..].to_vec()));
    Some(out)
}

fn rebuilt(
    base: &[String],
    from: usize,
    ends: usize,
    mine: bool,
    group: &[(bool, Edit)],
) -> Vec<String> {
    let mut out = Vec::new();
    let mut at = from;
    for (_, one) in group.iter().filter(|(ours, _)| *ours == mine) {
        if one.from > at {
            out.extend(base[at..one.from].iter().cloned());
        }
        out.extend(one.said.iter().cloned());
        at = at.max(one.upto);
    }
    if at < ends {
        out.extend(base[at..ends].iter().cloned());
    }
    out
}

fn touching(group: &[(bool, Edit)], one: &Edit) -> bool {
    group.iter().any(|(_, held)| {
        if held.from == held.upto && one.from == one.upto {
            held.from == one.from
        } else {
            held.from < one.upto && one.from < held.upto
        }
    })
}

fn listing(block: &str) -> bool {
    let first = block.lines().next().unwrap_or("").trim_start();
    let bullet = first.starts_with("- ") || first.starts_with("* ") || first.starts_with("+ ");
    let numbered = first.split_once(['.', ')']).is_some_and(|(head, tail)| {
        !head.is_empty() && head.bytes().all(|c| c.is_ascii_digit()) && tail.starts_with(' ')
    });
    bullet || numbered
}

fn shaped(said: &[String], seams: &[usize]) -> Option<String> {
    for at in seams {
        if *at > 0 && *at < said.len() && listing(&said[at - 1]) && listing(&said[*at]) {
            return None;
        }
    }
    let whole = format!("{}\n", said.join("\n\n"));
    (blocks(&whole) == said).then_some(whole)
}

fn woven(steps: &[Step], picks: &[Pick]) -> Option<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seams: Vec<usize> = Vec::new();
    let mut torn = 0usize;
    for step in steps {
        seams.push(out.len());
        match step {
            Step::Kept(said) | Step::One(said) => out.extend(said.iter().cloned()),
            Step::Torn(rift) => {
                match picks.get(torn).copied().unwrap_or(Pick::Both) {
                    Pick::Mine => out.extend(rift.mine.iter().cloned()),
                    Pick::Theirs => out.extend(rift.theirs.iter().cloned()),
                    Pick::Both => {
                        out.extend(rift.mine.iter().cloned());
                        seams.push(out.len());
                        out.extend(rift.theirs.iter().cloned());
                    }
                }
                torn += 1;
            }
        }
    }
    shaped(&out, &seams)
}

fn tally(said: &[String]) -> std::collections::HashMap<&str, usize> {
    let mut many = std::collections::HashMap::new();
    for one in said {
        *many.entry(one.as_str()).or_default() += 1;
    }
    many
}

fn sound(out: &str, mine: &str, theirs: &str) -> bool {
    let (out, mine, theirs) = (blocks(out), blocks(mine), blocks(theirs));
    let (out, mine, theirs) = (tally(&out), tally(&mine), tally(&theirs));

    for (block, many) in &out {
        let most = mine
            .get(block)
            .copied()
            .unwrap_or(0)
            .max(theirs.get(block).copied().unwrap_or(0));
        if *many > most {
            return false;
        }
    }
    for (block, here) in &mine {
        let Some(there) = theirs.get(block) else {
            continue;
        };
        if out.get(block).copied().unwrap_or(0) < *here.min(there) {
            return false;
        }
    }
    true
}

pub fn merged(base: &str, mine: &str, theirs: &str) -> Option<String> {
    let steps = plan(base, mine, theirs)?;
    if steps.iter().any(|one| matches!(one, Step::Torn(_))) {
        return None;
    }
    let whole = woven(&steps, &[])?;
    sound(&whole, mine, theirs).then_some(whole)
}

pub fn rifts(base: &str, mine: &str, theirs: &str) -> Vec<Rift> {
    let Some(steps) = plan(base, mine, theirs) else {
        return Vec::new();
    };
    steps
        .into_iter()
        .filter_map(|one| match one {
            Step::Torn(rift) => Some(rift),
            _ => None,
        })
        .collect()
}

pub fn woven_with(base: &str, mine: &str, theirs: &str, picks: &[Pick]) -> Option<String> {
    let steps = plan(base, mine, theirs)?;
    let torn = steps
        .iter()
        .filter(|one| matches!(one, Step::Torn(_)))
        .count();
    if picks.len() != torn {
        return None;
    }
    let whole = woven(&steps, picks)?;
    sound(&whole, mine, theirs).then_some(whole)
}

#[cfg(test)]
#[path = "merge_test.rs"]
mod tests;
