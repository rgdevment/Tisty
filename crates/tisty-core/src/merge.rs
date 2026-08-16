pub fn blocks(body: &str) -> Vec<String> {
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
    let mut lines = body.lines().map(str::trim).filter(|one| !one.is_empty());
    lines.next() == Some("---") && lines.any(|one| one == "---")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Edit {
    from: usize,
    upto: usize,
    said: Vec<String>,
}

fn shared(base: &[String], other: &[String]) -> Vec<(usize, usize)> {
    let mut grid = vec![vec![0usize; other.len() + 1]; base.len() + 1];
    for (a, one) in base.iter().enumerate() {
        for (b, two) in other.iter().enumerate() {
            grid[a + 1][b + 1] = if one == two {
                grid[a][b] + 1
            } else {
                grid[a][b + 1].max(grid[a + 1][b])
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
        } else if grid[a - 1][b] >= grid[a][b - 1] {
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

fn clashes(one: &Edit, two: &Edit) -> bool {
    if one.said == two.said {
        return false;
    }
    if one.from == one.upto && two.from == two.upto {
        return one.from == two.from;
    }
    one.from < two.upto && two.from < one.upto
}

pub fn merged(base: &str, mine: &str, theirs: &str) -> Option<String> {
    if front_matter(base) || front_matter(mine) || front_matter(theirs) {
        return None;
    }
    let base = blocks(base);
    let ours = edits(&base, &blocks(mine));
    let yours = edits(&base, &blocks(theirs));

    for one in &ours {
        for two in &yours {
            if clashes(one, two) {
                return None;
            }
        }
    }

    let mut all: Vec<&Edit> = ours.iter().chain(yours.iter()).collect();
    all.sort_by_key(|one| (one.from, one.upto));
    all.dedup_by(|a, b| a == b);

    let mut out: Vec<String> = Vec::new();
    let mut at = 0usize;
    for one in all {
        if one.from < at {
            return None;
        }
        out.extend(base[at..one.from].iter().cloned());
        out.extend(one.said.iter().cloned());
        at = one.upto;
    }
    out.extend(base[at..].iter().cloned());

    Some(format!("{}\n", out.join("\n\n")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "# Kit\n\nprimer parrafo\n\nsegundo parrafo\n\ntercer parrafo\n";

    #[test]
    fn a_body_is_split_where_a_blank_line_says_so() {
        assert_eq!(
            blocks(BASE),
            vec![
                "# Kit",
                "primer parrafo",
                "segundo parrafo",
                "tercer parrafo"
            ]
        );
    }

    #[test]
    fn a_fenced_block_keeps_its_blank_lines_instead_of_being_torn_apart() {
        let said = blocks("uno\n\n```js\nconst a = 1;\n\nconst b = 2;\n```\n\ndos");

        assert_eq!(said.len(), 3);
        assert!(said[1].contains("const b"), "{said:?}");
    }

    #[test]
    fn a_table_is_one_block_because_a_column_rewrites_all_of_it() {
        let said = blocks("texto\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n\nmas texto");

        assert_eq!(said.len(), 3);
        assert!(said[1].lines().count() == 3);
    }

    #[test]
    fn changes_far_apart_are_taken_from_both_sides_without_asking() {
        let mine =
            "# Kit\n\nprimer parrafo cambiado en el mac\n\nsegundo parrafo\n\ntercer parrafo\n";
        let theirs = "# Kit\n\nprimer parrafo\n\nsegundo parrafo\n\ntercer parrafo de windows\n";

        let said = merged(BASE, mine, theirs).expect("tenia que fusionar");

        assert!(said.contains("cambiado en el mac"));
        assert!(said.contains("de windows"));
        assert!(said.contains("segundo parrafo"));
    }

    #[test]
    fn a_section_added_at_the_end_lands_beside_a_paragraph_changed_at_the_top() {
        let mine =
            "# Kit\n\nprimer parrafo\n\nsegundo parrafo\n\ntercer parrafo\n\n## Nueva\n\ncuerpo\n";
        let theirs = "# Kit corregido\n\nprimer parrafo\n\nsegundo parrafo\n\ntercer parrafo\n";

        let said = merged(BASE, mine, theirs).expect("tenia que fusionar");

        assert!(said.contains("# Kit corregido"));
        assert!(said.contains("## Nueva"));
    }

    #[test]
    fn the_same_block_written_two_ways_is_never_guessed() {
        let mine = "# Kit\n\nprimer parrafo del mac\n\nsegundo parrafo\n\ntercer parrafo\n";
        let theirs = "# Kit\n\nprimer parrafo de windows\n\nsegundo parrafo\n\ntercer parrafo\n";

        assert!(merged(BASE, mine, theirs).is_none());
    }

    #[test]
    fn the_same_block_written_the_same_way_twice_is_not_a_clash() {
        let same = "# Kit\n\nprimer parrafo igual\n\nsegundo parrafo\n\ntercer parrafo\n";

        let said = merged(BASE, same, same).expect("no habia nada que discutir");

        assert!(said.contains("primer parrafo igual"));
        assert_eq!(blocks(&said).len(), 4);
    }

    #[test]
    fn one_side_deleting_what_the_other_rewrote_is_never_guessed() {
        let mine = "# Kit\n\nsegundo parrafo\n\ntercer parrafo\n";
        let theirs = "# Kit\n\nprimer parrafo reescrito\n\nsegundo parrafo\n\ntercer parrafo\n";

        assert!(merged(BASE, mine, theirs).is_none());
    }

    #[test]
    fn two_sides_adding_at_the_very_same_point_is_never_guessed() {
        let mine = "# Kit\n\nprimer parrafo\n\ndel mac\n\nsegundo parrafo\n\ntercer parrafo\n";
        let theirs = "# Kit\n\nprimer parrafo\n\nde windows\n\nsegundo parrafo\n\ntercer parrafo\n";

        assert!(merged(BASE, mine, theirs).is_none());
    }

    #[test]
    fn a_document_with_front_matter_is_left_alone_because_it_does_not_settle() {
        let base = "---\ntitle: x\n---\n\n# Kit\n\nuno\n";
        let mine = "---\ntitle: x\n---\n\n# Kit\n\ndos\n";
        let theirs = "---\ntitle: x\n---\n\n# Kit\n\nuno\n\ntres\n";

        assert!(merged(base, mine, theirs).is_none());
    }

    #[test]
    fn what_neither_side_touched_comes_out_once_and_in_its_place() {
        let mine = "# Kit\n\nprimer parrafo\n\nsegundo parrafo\n\ntercer parrafo\n\ncuarto\n";
        let theirs = "# Kit del mac\n\nprimer parrafo\n\nsegundo parrafo\n\ntercer parrafo\n";

        let said = merged(BASE, mine, theirs).unwrap();

        assert_eq!(said.matches("segundo parrafo").count(), 1);
        assert_eq!(
            blocks(&said),
            vec![
                "# Kit del mac",
                "primer parrafo",
                "segundo parrafo",
                "tercer parrafo",
                "cuarto"
            ]
        );
    }

    #[test]
    fn a_side_that_changed_nothing_leaves_the_other_side_whole() {
        let mine = "# Kit\n\nprimer parrafo\n\nsegundo parrafo\n\ntercer parrafo\n\nañadido\n";

        let said = merged(BASE, mine, BASE).unwrap();

        assert_eq!(said, mine);
    }
}
