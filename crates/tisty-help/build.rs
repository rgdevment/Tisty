use std::fmt::Write as _;
use std::path::{Path, PathBuf};

struct Listed {
    page: String,
    label: String,
    section: String,
}

fn linked(raw: &str) -> (String, String) {
    let (label, target) = raw.split_once('|').unwrap_or((raw, raw));
    let page = target
        .split('#')
        .next()
        .unwrap_or(target)
        .trim()
        .replace(' ', "-");
    (label.trim().to_string(), page)
}

fn sidebar(text: &str) -> ([String; 2], [Vec<Listed>; 2]) {
    let homes: Vec<String> = text
        .lines()
        .next()
        .unwrap_or_default()
        .split("[[")
        .skip(1)
        .filter_map(|one| one.split_once("]]"))
        .map(|(raw, _)| linked(raw).1)
        .collect();
    let english = homes.iter().find(|one| *one == "Home").cloned();
    let spanish = homes.iter().find(|one| *one != "Home").cloned();
    let mut lists: [Vec<Listed>; 2] = [Vec::new(), Vec::new()];
    for block in text.split("<summary>").skip(1) {
        let tongue = usize::from(block.lines().next().unwrap_or_default().contains("Español"));
        let mut section = String::new();
        for line in block.lines().skip(1) {
            let line = line.trim_end();
            if let Some(head) = line
                .strip_prefix("**")
                .and_then(|rest| rest.strip_suffix("**"))
            {
                section = head.to_string();
            } else if let Some(raw) = line
                .strip_prefix("- [[")
                .and_then(|rest| rest.split_once("]]"))
                .map(|(raw, _)| raw)
            {
                let (label, page) = linked(raw);
                lists[tongue].push(Listed {
                    page,
                    label,
                    section: section.clone(),
                });
            }
        }
    }
    let (Some(english), Some(spanish)) = (english, spanish) else {
        panic!("docs/help/_Sidebar.md names no English and Spanish home");
    };
    assert_eq!(
        lists[0].len(),
        lists[1].len(),
        "docs/help/_Sidebar.md lists a different number of pages in each language"
    );
    ([english, spanish], lists)
}

fn entry(
    out: &mut String,
    help: &Path,
    home: &str,
    one: Option<&Listed>,
    twin: &str,
    order: usize,
) {
    let page = one.map_or(home, |one| one.page.as_str());
    let file = help.join(format!("{page}.md"));
    assert!(
        file.is_file(),
        "docs/help/_Sidebar.md names {page}, which has no page"
    );
    let _ = writeln!(
        out,
        "    Page {{ page: {page:?}, label: {:?}, section: {:?}, order: {order}, twin: {twin:?}, body: include_str!({:?}) }},",
        one.map_or("", |one| one.label.as_str()),
        one.map_or("", |one| one.section.as_str()),
        file.display().to_string(),
    );
}

const OURS: &str = "https://raw.githubusercontent.com/rgdevment/Tisty/main/";

fn weighed(out: &mut String, help: &Path) {
    let root = help.join("../..");
    let mut found: Vec<(String, u64)> = Vec::new();
    for entry in std::fs::read_dir(help).into_iter().flatten().flatten() {
        let text = std::fs::read_to_string(entry.path()).unwrap_or_default();
        for piece in text.split(OURS).skip(1) {
            let path: String = piece
                .chars()
                .take_while(|one| !matches!(one, '"' | ')' | ' ' | '\''))
                .collect();
            let file = root.join(&path);
            println!("cargo::rerun-if-changed={}", file.display());
            if let Ok(meta) = std::fs::metadata(&file) {
                found.push((format!("{OURS}{path}"), meta.len()));
            }
        }
    }
    found.sort();
    found.dedup();
    let _ = writeln!(out, "pub static WEIGHS: &[(&str, u64)] = &[");
    for (url, bytes) in found {
        let _ = writeln!(out, "    ({url:?}, {bytes}),");
    }
    let _ = writeln!(out, "];");
}

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let help = manifest.join("../../docs/help");
    println!("cargo::rerun-if-changed={}", help.display());
    let text = std::fs::read_to_string(help.join("_Sidebar.md"))
        .unwrap_or_else(|why| panic!("docs/help/_Sidebar.md: {why}"));
    let (homes, lists) = sidebar(&text);
    let mut out = String::new();
    for (tongue, name) in [(0usize, "EN"), (1, "ES")] {
        let other = 1 - tongue;
        let _ = writeln!(out, "pub static {name}: &[Page] = &[");
        entry(&mut out, &help, &homes[tongue], None, &homes[other], 0);
        for (at, one) in lists[tongue].iter().enumerate() {
            entry(
                &mut out,
                &help,
                &homes[tongue],
                Some(one),
                &lists[other][at].page,
                at + 1,
            );
        }
        let _ = writeln!(out, "];");
    }
    weighed(&mut out, &help);
    let target = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default()).join("pages.rs");
    std::fs::write(&target, out).unwrap_or_else(|why| panic!("{}: {why}", target.display()));
}
