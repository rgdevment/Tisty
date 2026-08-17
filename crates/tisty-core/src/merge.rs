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
mod tests {
    use super::*;

    const BASE: &str = "# Kit\n\nprimer parrafo\n\nsegundo parrafo\n\ntercer parrafo\n";

    fn told(said: &[&str]) -> String {
        format!("{}\n", said.join("\n\n"))
    }

    #[test]
    fn a_clash_says_what_each_side_holds_for_the_whole_stretch_not_only_what_it_replaced() {
        let base = told(&["b0", "b1", "b2", "b3", "b4", "b5"]);
        let mine = told(&["b0", "M0", "M1", "b5"]);
        let theirs = told(&["b0", "b1", "T0", "T1", "b4", "b5"]);

        let said = rifts(&base, &mine, &theirs);

        assert_eq!(said.len(), 1);
        assert_eq!(said[0].mine, vec!["M0", "M1"]);
        assert_eq!(
            said[0].theirs,
            vec!["b1", "T0", "T1", "b4"],
            "el dialogo enseñaria menos de lo que esa maquina tiene"
        );
    }

    #[test]
    fn keeping_one_side_of_a_wide_clash_gives_back_that_side_whole() {
        let base = told(&["b0", "b1", "b2", "b3", "b4", "b5"]);
        let mine = told(&["b0", "M0", "M1", "b5"]);
        let theirs = told(&["b0", "b1", "T0", "T1", "b4", "b5"]);

        assert_eq!(
            woven_with(&base, &mine, &theirs, &[Pick::Mine]).unwrap(),
            mine
        );
        assert_eq!(
            woven_with(&base, &mine, &theirs, &[Pick::Theirs]).unwrap(),
            theirs
        );
    }

    #[test]
    fn keeping_your_own_side_never_empties_a_document_that_had_blocks() {
        let base = told(&["b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7"]);
        let mine = told(&["b0", "b1", "b2"]);
        let theirs = told(&["T0", "b6", "b7"]);

        let said = woven_with(&base, &mine, &theirs, &[Pick::Mine]).unwrap();

        assert_eq!(
            said, mine,
            "quedarse con la propia dejo el documento en nada"
        );
    }

    #[test]
    fn a_side_that_kept_a_block_inside_a_clash_still_has_it_afterwards() {
        let base = told(&["uno", "dos", "tres", "cuatro"]);
        let mine = told(&["uno", "dos cambiado", "tres", "cuatro"]);
        let theirs = told(&["uno", "dos", "tres cambiado", "cuatro"]);

        let whole = merged(&base, &mine, &theirs).expect("no se solapan");

        assert!(whole.contains("dos cambiado"));
        assert!(whole.contains("tres cambiado"));
        assert_eq!(whole.matches("uno").count(), 1);
    }

    #[test]
    fn when_both_only_touched_the_torn_stretch_keeping_a_side_hands_it_back_word_for_word() {
        let mut seed: u64 = 0x2545_F491_4F6C_DD1D;
        let mut roll = move |upto: u64| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed % upto
        };
        let mut tried = 0;

        for _ in 0..20_000 {
            let many = 6 + roll(4) as usize;
            let base: Vec<String> = (0..many).map(|at| format!("b{at}")).collect();
            let from = roll(3) as usize + 1;
            let upto = (from + 2 + roll(3) as usize).min(many);
            let carve = |roll: &mut dyn FnMut(u64) -> u64, tag: &str| -> Vec<String> {
                base.iter()
                    .enumerate()
                    .filter_map(|(at, block)| {
                        if at < from || at >= upto {
                            return Some(block.clone());
                        }
                        match roll(3) {
                            0 => None,
                            1 => Some(format!("{tag}{at}")),
                            _ => Some(block.clone()),
                        }
                    })
                    .collect()
            };
            let mine = told(
                &carve(&mut roll, "m")
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            );
            let theirs = told(
                &carve(&mut roll, "t")
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            );
            let was = told(&base.iter().map(String::as_str).collect::<Vec<_>>());

            let said = rifts(&was, &mine, &theirs);
            if said.len() != 1 || said[0].was != base[from..upto] {
                continue;
            }
            tried += 1;

            assert_eq!(
                woven_with(&was, &mine, &theirs, &[Pick::Mine]).unwrap(),
                mine,
                "base {was:?} mia {mine:?} suya {theirs:?}"
            );
            assert_eq!(
                woven_with(&was, &mine, &theirs, &[Pick::Theirs]).unwrap(),
                theirs,
                "base {was:?} mia {mine:?} suya {theirs:?}"
            );
        }

        assert!(tried > 500, "el sorteo no llego a probar nada: {tried}");
    }

    #[test]
    fn a_paper_with_more_blocks_than_we_can_weave_is_handed_back_to_the_person() {
        let many = CELLS_AT_MOST.isqrt() + 1;
        let base: String = (0..many).map(|at| format!("b{at}\n\n")).collect();
        let mine = base.replace("b7\n", "cambiado\n");
        let theirs = base.replace("b9\n", "otro\n");

        let now = std::time::Instant::now();
        let said = merged(&base, &mine, &theirs);

        assert!(said.is_none(), "se puso a tejer algo que no puede");
        assert!(
            now.elapsed() < std::time::Duration::from_millis(200),
            "tardo {:?} en decir que no",
            now.elapsed()
        );
    }

    #[test]
    fn the_widest_paper_we_accept_is_woven_without_eating_the_machine() {
        let many = CELLS_AT_MOST.isqrt();
        let base: String = (0..many).map(|at| format!("b{at}\n\n")).collect();
        let mine = base.replace("b7\n", "cambiado\n");
        let theirs = base.replace("b9\n", "otro\n");

        let now = std::time::Instant::now();
        let said = merged(&base, &mine, &theirs).expect("cabe de sobra");

        assert!(said.contains("cambiado") && said.contains("otro"));
        assert!(
            now.elapsed() < std::time::Duration::from_secs(5),
            "tardo {:?}",
            now.elapsed()
        );
    }

    #[test]
    fn a_byte_order_mark_does_not_smuggle_front_matter_past_the_gate() {
        let base = "\u{feff}---\ntitle: uno\n---\n\ncuerpo\n";
        let mine = "\u{feff}---\ntitle: uno\n---\n\notro cuerpo\n";
        let theirs = "\u{feff}---\ntitle: dos\n---\n\ncuerpo\n";

        assert!(front_matter(base));
        assert!(merged(base, mine, theirs).is_none());
    }

    #[test]
    fn a_block_that_both_sides_moved_is_never_quietly_doubled() {
        let base = told(&["a", "b", "c"]);
        let mine = told(&["c", "b"]);
        let theirs = told(&["a", "c", "b"]);

        assert!(rifts(&base, &mine, &theirs).is_empty());
        assert_eq!(
            merged(&base, &mine, &theirs),
            None,
            "escribio el documento con un bloque repetido sin preguntar"
        );
    }

    #[test]
    fn keeping_a_side_never_hands_back_a_document_missing_what_both_sides_held() {
        let base = told(&["a", "b", "c"]);
        let mine = told(&["c"]);
        let theirs = told(&["c", "b"]);

        assert_eq!(rifts(&base, &mine, &theirs).len(), 1);
        assert_eq!(
            woven_with(&base, &mine, &theirs, &[Pick::Mine]),
            None,
            "devolvio un documento sin el bloque que las dos conservaban"
        );
    }

    #[test]
    fn keeping_a_side_never_hands_back_the_same_block_twice() {
        let base = told(&["a", "b", "c"]);
        let mine = told(&["b"]);
        let theirs = told(&["b", "a"]);

        for pick in [Pick::Mine, Pick::Both] {
            assert_eq!(
                woven_with(&base, &mine, &theirs, &[pick]),
                None,
                "devolvio el mismo bloque dos veces"
            );
        }
    }

    #[test]
    fn a_byte_order_mark_does_not_turn_the_same_heading_into_two_different_ones() {
        let base = told(&["# Kit", "el cuerpo"]);
        let mine = format!("\u{feff}{}", told(&["# Kit", "el cuerpo del mac"]));
        let theirs = told(&["# Kit", "el cuerpo", "algo de windows"]);

        assert_eq!(blocks(&mine).first().map(String::as_str), Some("# Kit"));
        assert!(
            rifts(&base, &mine, &theirs).is_empty(),
            "un encabezado identico a la vista salio como desacuerdo"
        );
        assert_eq!(
            merged(&base, &mine, &theirs),
            Some(told(&["# Kit", "el cuerpo del mac", "algo de windows"]))
        );
    }

    #[test]
    fn a_fence_left_open_never_swallows_the_paragraph_that_follows() {
        let base = "M\n\nmedio\n";
        let mine = "M\n\n```\nx\n";
        let theirs = "M\n\nT\n";

        assert_eq!(
            blocks(mine),
            vec!["M", "```\nx"],
            "el caso no es el que creo"
        );
        assert_eq!(rifts(base, mine, theirs).len(), 1);

        let said = woven_with(base, mine, theirs, &[Pick::Both]);

        assert_eq!(
            said, None,
            "el parrafo de la otra maquina quedo dentro de un bloque de codigo"
        );
    }

    #[test]
    fn whatever_is_woven_reads_back_as_the_very_blocks_it_was_made_of() {
        let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut roll = move |upto: u64| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed % upto
        };
        let bits = [
            "un parrafo",
            "# titulo",
            "- uno",
            "| a | b |",
            "texto   ",
            "1. uno",
            ">",
            "```\ncerrado\n```",
            "\u{feff}marca",
        ];
        let mut tried = 0;

        for _ in 0..5_000 {
            let pick = |roll: &mut dyn FnMut(u64) -> u64| {
                let many = 2 + roll(4) as usize;
                (0..many)
                    .map(|_| bits[roll(bits.len() as u64) as usize])
                    .collect::<Vec<_>>()
            };
            let ragged = |roll: &mut dyn FnMut(u64) -> u64| {
                let mut said = pick(roll);
                if roll(3) == 0 {
                    said.push("```\nsin cerrar");
                }
                told(&said)
            };
            let base = ragged(&mut roll);
            let mine = ragged(&mut roll);
            let theirs = ragged(&mut roll);

            for picks in [
                vec![],
                vec![Pick::Mine],
                vec![Pick::Theirs],
                vec![Pick::Both],
            ] {
                let Some(whole) = woven_with(&base, &mine, &theirs, &picks) else {
                    continue;
                };
                tried += 1;
                assert_eq!(
                    format!("{}\n", blocks(&whole).join("\n\n")),
                    whole,
                    "base {base:?} mia {mine:?} suya {theirs:?} elecciones {picks:?}"
                );
            }
        }

        assert!(tried > 200, "el sorteo no llego a tejer nada: {tried}");
    }

    #[test]
    fn the_weave_always_comes_out_with_one_kind_of_line_ending_so_two_machines_agree() {
        let base = "# Kit\r\n\r\nuno\r\n";
        let mine = "# Kit\r\n\r\nuno del mac\r\n";
        let theirs = "# Kit\r\n\r\nuno\r\n\r\ndos\r\n";

        let said = merged(base, mine, theirs).expect("se junta solo");

        assert!(
            !said.contains('\r'),
            "conservar el final de linea de cada maquina las dejaria en desacuerdo para siempre"
        );
        assert_eq!(said, "# Kit\n\nuno del mac\n\ndos\n");
    }

    #[test]
    fn the_same_body_written_two_ways_weaves_to_the_very_same_bytes() {
        let flat = |said: &str| said.replace("\r\n", "\n");
        let base = "# Kit\r\n\r\nuno\r\n";
        let mine = "# Kit\r\n\r\nuno del mac\r\n";
        let theirs = "# Kit\r\n\r\nuno\r\n\r\ndos\r\n";

        assert_eq!(
            merged(base, mine, theirs),
            merged(&flat(base), &flat(mine), &flat(theirs)),
            "windows y mac llegarian a bytes distintos desde el mismo texto"
        );
    }

    #[test]
    fn two_lists_the_weave_would_glue_together_are_handed_back_instead() {
        let base = told(&["intro", "- uno\n- dos", "cierre"]);
        let mine = told(&["intro", "- uno\n- dos", "- [ ] pendiente", "cierre"]);
        let theirs = told(&["intro", "- uno\n- dos", "1. otra", "cierre"]);

        let said = woven_with(&base, &mine, &theirs, &[Pick::Both]);

        assert_eq!(
            said, None,
            "markdown leeria las dos listas como una sola, con un item inventado"
        );
    }

    #[test]
    fn a_list_the_person_already_had_beside_another_is_not_refused() {
        let base = told(&["- uno\n- dos", "1. tres", "cierre"]);
        let mine = told(&["- uno\n- dos", "1. tres", "cierre del mac"]);
        let theirs = told(&["- uno\n- dos", "1. tres", "cierre", "algo mas"]);

        assert!(
            merged(&base, &mine, &theirs).is_some(),
            "se nego por una vecindad que ya estaba en el documento"
        );
    }

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
        assert!(
            rifts(base, mine, theirs).is_empty(),
            "sin desacuerdos que enseñar, la ventana pregunta de frente en vez de encallarse"
        );
        assert!(woven_with(base, mine, theirs, &[]).is_none());
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
