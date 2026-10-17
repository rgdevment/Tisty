use crate::Paths;

#[derive(Default)]
pub struct Carried {
    pub kept: usize,
    pub missed: Vec<String>,
    pub papers: Vec<String>,
    pub told: Vec<crate::Op>,
}

pub fn unescaped(target: &str) -> String {
    let bytes = target.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'%'
            && at + 2 < bytes.len()
            && let Ok(one) = u8::from_str_radix(&target[at + 1..at + 3], 16)
        {
            out.push(one);
            at += 3;
            continue;
        }
        out.push(bytes[at]);
        at += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| target.to_string())
}

fn opening(body: &str, at: usize) -> Option<usize> {
    let bytes = body.as_bytes();
    let line = body[..at].rfind('\n').map(|one| one + 1).unwrap_or(0);
    let mut deep = 0;
    let mut walk = at;
    while walk > line {
        walk -= 1;
        match bytes[walk] {
            b']' => deep += 1,
            b'[' if deep == 0 => return Some(walk),
            b'[' => deep -= 1,
            _ => {}
        }
    }
    None
}

pub fn written_as_code(body: &str) -> Vec<bool> {
    let mut out = vec![false; body.len()];
    let mut fence: Option<String> = None;
    let mut at = 0;
    for line in body.split_inclusive('\n') {
        let bare = line.trim_end_matches('\n');
        let opens = bare
            .trim_start()
            .chars()
            .take_while(|one| *one == '`' || *one == '~')
            .count();
        let mark = bare.trim_start().chars().next().unwrap_or(' ');
        match &fence {
            Some(open) => {
                for one in out.iter_mut().skip(at).take(line.len()) {
                    *one = true;
                }
                if opens >= open.len() && bare.trim_start().starts_with(open.as_str()) {
                    fence = None;
                }
            }
            None if opens >= 3 => {
                fence = Some(mark.to_string().repeat(opens));
                for one in out.iter_mut().skip(at).take(line.len()) {
                    *one = true;
                }
            }
            None => {
                let mut walk = 0;
                for (span, part) in crate::arriving::spans(bare) {
                    if span {
                        for one in out.iter_mut().skip(at + walk).take(part.len()) {
                            *one = true;
                        }
                    }
                    walk += part.len();
                }
            }
        }
        at += line.len();
    }
    out
}

pub fn retargeted(
    body: &str,
    with: &mut impl FnMut(&str, &str, &str) -> Option<(String, String)>,
) -> String {
    let coded = written_as_code(body);
    let bytes = body.as_bytes();
    let mut out = String::with_capacity(body.len());
    let mut at = 0;
    let mut from = 0;
    while at < bytes.len() {
        if bytes[at] != b']' || bytes.get(at + 1) != Some(&b'(') || coded[at] {
            at += 1;
            continue;
        }
        let Some(open) = opening(body, at) else {
            at += 1;
            continue;
        };
        let mut walk = at + 2;
        let mut deep = 1;
        while walk < bytes.len() && deep > 0 {
            match bytes[walk] {
                b'(' => deep += 1,
                b')' => deep -= 1,
                _ => {}
            }
            walk += 1;
        }
        if deep > 0 {
            at += 1;
            continue;
        }
        let inner = &body[at + 2..walk - 1];
        let (target, title) = split_target(inner);
        let label = &body[open + 1..at];
        let pictured = open > 0 && bytes[open - 1] == b'!';
        let cut = match pictured {
            true => open - 1,
            false => open,
        };
        out.push_str(&body[from..cut]);
        match with(label, &target, &title) {
            Some((now, title)) => {
                let shown = match title.is_empty() {
                    true => format!("<{now}>"),
                    false => format!("<{now}> {title}"),
                };
                let mark = if pictured { "!" } else { "" };
                out.push_str(&format!("{mark}[{label}]({shown})"));
            }
            None => out.push_str(label),
        }
        from = walk;
        at = walk;
    }
    out.push_str(&body[from..]);
    out
}

fn split_target(inner: &str) -> (String, String) {
    let said = inner.trim();
    if let Some(rest) = said.strip_prefix('<')
        && let Some(shut) = rest.find('>')
    {
        return (
            rest[..shut].to_string(),
            rest[shut + 1..].trim().to_string(),
        );
    }
    match said.find(char::is_whitespace) {
        Some(gap) => {
            let rest = said[gap..].trim();
            match rest.starts_with(['"', '\'', '(']) {
                true => (said[..gap].to_string(), rest.to_string()),
                false => (said.to_string(), String::new()),
            }
        }
        None => (said.to_string(), String::new()),
    }
}

pub fn brought(
    paths: &Paths,
    from: &std::path::Path,
    body: &str,
    reach: &mut dyn FnMut(&std::path::Path) -> Result<std::path::PathBuf, String>,
) -> (String, Carried) {
    let mut done = Carried::default();
    let here = from
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    let here = here.canonicalize().unwrap_or(here);
    let mut kept: std::collections::BTreeMap<String, Option<String>> = Default::default();

    let out = retargeted(body, &mut |_, target: &str, title: &str| {
        if let Some(said) = kept.get(target) {
            return said.clone().map(|at| (at, title.to_string()));
        }
        let landed = brought_in(paths, &here, target, &mut done, reach);
        kept.insert(target.to_string(), landed.clone());
        landed.map(|at| (at, title.to_string()))
    });
    (out, done)
}

fn brought_in(
    paths: &Paths,
    here: &std::path::Path,
    target: &str,
    done: &mut Carried,
    reach: &mut dyn FnMut(&std::path::Path) -> Result<std::path::PathBuf, String>,
) -> Option<String> {
    if target.starts_with('#')
        || target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
        || target.starts_with(crate::refs::DOC)
        || schemed(target)
    {
        return Some(target.to_string());
    }
    if let Some(kept) = target.strip_prefix("attachments/") {
        if stays_beside(&unescaped(kept)) {
            return Some(target.to_string());
        }
        done.missed.push(format!(
            "{target} — it reads as a file Tisty already keeps, but it climbs out of the shelf \
             those sit on, and nothing there can be named that way"
        ));
        return None;
    }
    if target.starts_with('/')
        || target.starts_with('\\')
        || target.contains("://")
        || target.chars().nth(1) == Some(':')
    {
        done.missed.push(format!(
            "{target} — it names a place on this machine rather than something beside the document, and a document that leans on a path outside Tisty breaks the day it moves"
        ));
        return None;
    }
    let plain = unescaped(target);
    let mut cannot = |why: String| {
        done.missed.push(format!("{target} — {why}"));
        None::<String>
    };
    if !stays_beside(&plain) {
        return cannot(OUTSIDE.into());
    }
    let at = here.join(&plain);
    let there = at.exists();

    if !there {
        return cannot("no file is there".into());
    }
    let at = match reach(&at) {
        Ok(at) => at,
        Err(why) => return cannot(why),
    };
    if !at.starts_with(here) {
        return cannot(OUTSIDE.into());
    }
    if crate::agent::fit_to_keep(&at).is_err() {
        return cannot(
            "its bytes are not the kind of file its name says it is, so what came out of Tisty later would not open"
                .into(),
        );
    }
    let heavy = std::fs::metadata(&at).map(|one| one.len()).unwrap_or(0);
    if heavy > crate::attach::COPIED_IN_DOC {
        return cannot(format!(
            "it is {heavy} bytes, past the {} a document holds; make it smaller and import it again",
            crate::attach::COPIED_IN_DOC
        ));
    }
    if done.kept >= crate::attach::KEPT_IN_A_DOC {
        return cannot(format!(
            "a document holds {} files at most, and this one is already full",
            crate::attach::KEPT_IN_A_DOC
        ));
    }
    match crate::attach::keep(&at, paths.data(), crate::attach::COPIED_IN_DOC) {
        Err(why) => cannot(why.to_string()),
        Ok(one) => {
            done.kept += 1;
            done.told.push(one.told());
            if plain.to_lowercase().ends_with(".md") || plain.to_lowercase().ends_with(".markdown")
            {
                done.papers.push(plain);
            }
            Some(
                crate::attach::leaf_of(&one.at)
                    .unwrap_or(&one.at)
                    .to_string(),
            )
        }
    }
}

pub const OUTSIDE: &str = "it sits outside the folder the document came from, and an import \
takes only what is kept beside it. Put a copy in that folder and import again";
fn stays_beside(plain: &str) -> bool {
    let mut depth = 0i32;
    for part in std::path::Path::new(plain).components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(_) => depth += 1,
            std::path::Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => return false,
        }
    }
    true
}

// A link with a scheme of its own, like tel: or another app's, names nothing on disk; a drive letter and file: do.
fn schemed(target: &str) -> bool {
    let Some((scheme, _)) = target.split_once(':') else {
        return false;
    };
    scheme.len() > 1
        && !scheme.eq_ignore_ascii_case("file")
        && scheme.starts_with(|one: char| one.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|one| one.is_ascii_alphanumeric() || matches!(one, '+' | '-' | '.'))
}
