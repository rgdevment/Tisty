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
    let ours = edits(&base, &blocks(mine));
    let yours = edits(&base, &blocks(theirs));

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

        let ours: Vec<String> = group
            .iter()
            .filter(|(ours, _)| *ours)
            .flat_map(|(_, one)| one.said.clone())
            .collect();
        let yours: Vec<String> = group
            .iter()
            .filter(|(ours, _)| !*ours)
            .flat_map(|(_, one)| one.said.clone())
            .collect();
        let both = group.iter().any(|(ours, _)| *ours) && group.iter().any(|(ours, _)| !*ours);

        if !both || ours == yours {
            out.push(Step::One(if ours.is_empty() { yours } else { ours }));
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

fn touching(group: &[(bool, Edit)], one: &Edit) -> bool {
    group.iter().any(|(_, held)| {
        if held.from == held.upto && one.from == one.upto {
            held.from == one.from
        } else {
            held.from < one.upto && one.from < held.upto
        }
    })
}

fn woven(steps: &[Step], picks: &[Pick]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut torn = 0usize;
    for step in steps {
        match step {
            Step::Kept(said) | Step::One(said) => out.extend(said.iter().cloned()),
            Step::Torn(rift) => {
                match picks.get(torn).copied().unwrap_or(Pick::Both) {
                    Pick::Mine => out.extend(rift.mine.iter().cloned()),
                    Pick::Theirs => out.extend(rift.theirs.iter().cloned()),
                    Pick::Both => {
                        out.extend(rift.mine.iter().cloned());
                        out.extend(rift.theirs.iter().cloned());
                    }
                }
                torn += 1;
            }
        }
    }
    format!("{}\n", out.join("\n\n"))
}

pub fn merged(base: &str, mine: &str, theirs: &str) -> Option<String> {
    let steps = plan(base, mine, theirs)?;
    if steps.iter().any(|one| matches!(one, Step::Torn(_))) {
        return None;
    }
    Some(woven(&steps, &[]))
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
    Some(woven(&steps, picks))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "# Kit\n\nprimer parrafo\n\nsegundo parrafo\n\ntercer parrafo\n";

    #[test]
    fn a_clash_says_what_each_side_wrote_and_what_was_there_before() {
        let mine = "# Kit\n\nprimer parrafo del mac\n\nsegundo parrafo\n\ntercer parrafo\n";
        let theirs = "# Kit\n\nprimer parrafo de windows\n\nsegundo parrafo\n\ntercer parrafo\n";

        let said = rifts(BASE, mine, theirs);

        assert_eq!(said.len(), 1);
        assert_eq!(said[0].was, vec!["primer parrafo"]);
        assert_eq!(said[0].mine, vec!["primer parrafo del mac"]);
        assert_eq!(said[0].theirs, vec!["primer parrafo de windows"]);
    }

    #[test]
    fn what_merged_on_its_own_is_never_offered_as_something_to_decide() {
        let mine = "# Kit\n\nprimer parrafo del mac\n\nsegundo parrafo\n\ntercer parrafo\n";
        let theirs = "# Kit\n\nprimer parrafo\n\nsegundo parrafo\n\ntercer parrafo de windows\n";

        assert!(rifts(BASE, mine, theirs).is_empty());
    }

    #[test]
    fn two_clashes_far_apart_come_back_as_two_and_in_order() {
        let mine = "# Kit del mac\n\nprimer parrafo\n\nsegundo parrafo\n\ntercero del mac\n";
        let theirs =
            "# Kit de windows\n\nprimer parrafo\n\nsegundo parrafo\n\ntercero de windows\n";

        let said = rifts(BASE, mine, theirs);

        assert_eq!(said.len(), 2);
        assert_eq!(said[0].mine, vec!["# Kit del mac"]);
        assert_eq!(said[1].theirs, vec!["tercero de windows"]);
    }

    #[test]
    fn keeping_one_side_of_a_clash_leaves_the_rest_of_the_document_alone() {
        let mine = "# Kit\n\nprimer parrafo del mac\n\nsegundo parrafo\n\ntercer parrafo\n";
        let theirs = "# Kit\n\nprimer parrafo de windows\n\nsegundo parrafo\n\ntercer parrafo\n";

        let said = woven_with(BASE, mine, theirs, &[Pick::Mine]).unwrap();

        assert_eq!(said, mine);
    }

    #[test]
    fn keeping_the_other_side_gives_exactly_what_the_other_side_wrote() {
        let mine = "# Kit\n\nprimer parrafo del mac\n\nsegundo parrafo\n\ntercer parrafo\n";
        let theirs = "# Kit\n\nprimer parrafo de windows\n\nsegundo parrafo\n\ntercer parrafo\n";

        assert_eq!(
            woven_with(BASE, mine, theirs, &[Pick::Theirs]).unwrap(),
            theirs
        );
    }

    #[test]
    fn keeping_both_puts_them_one_after_the_other_and_loses_neither() {
        let mine = "# Kit\n\nprimer parrafo del mac\n\nsegundo parrafo\n\ntercer parrafo\n";
        let theirs = "# Kit\n\nprimer parrafo de windows\n\nsegundo parrafo\n\ntercer parrafo\n";

        let said = woven_with(BASE, mine, theirs, &[Pick::Both]).unwrap();

        assert!(said.contains("del mac"));
        assert!(said.contains("de windows"));
        assert_eq!(said.matches("segundo parrafo").count(), 1);
    }

    #[test]
    fn deciding_with_the_wrong_number_of_answers_is_refused_instead_of_guessed() {
        let mine = "# Kit\n\nprimer parrafo del mac\n\nsegundo parrafo\n\ntercer parrafo\n";
        let theirs = "# Kit\n\nprimer parrafo de windows\n\nsegundo parrafo\n\ntercer parrafo\n";

        assert!(woven_with(BASE, mine, theirs, &[]).is_none());
        assert!(woven_with(BASE, mine, theirs, &[Pick::Mine, Pick::Mine]).is_none());
    }

    #[test]
    fn a_document_that_cannot_be_read_as_blocks_offers_nothing_to_decide() {
        let base = "---\ntitle: x\n---\n\n# Kit\n\nuno\n";

        assert!(rifts(base, base, base).is_empty());
        assert!(woven_with(base, base, base, &[]).is_none());
    }

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
