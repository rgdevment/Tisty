const NBSP: &str = "&nbsp;";

/// Spaces Tisty wrote out as `&nbsp;` for another reader come back as plain spaces; a line of one alone is a blank line and stays.
pub fn spaced_back(text: &str) -> String {
    let mut fence: Option<char> = None;
    text.split('\n')
        .map(|line| {
            let bare = line.trim_start();
            let mark = bare.chars().next().filter(|one| *one == '`' || *one == '~');
            if let Some(one) = mark.filter(|one| bare.starts_with(&one.to_string().repeat(3))) {
                fence = match fence {
                    Some(open) if open == one => None,
                    None => Some(one),
                    still => still,
                };
                return line.to_string();
            }
            if fence.is_some() || line.trim() == NBSP {
                return line.to_string();
            }
            let words = leading(line);
            format!(
                "{}{}",
                &line[..line.len() - words.len()],
                outside_code(words)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// Four spaces at a line's start would make code, so what opens the line stays as written.
fn leading(line: &str) -> &str {
    let mut rest = line;
    loop {
        let next = rest.trim_start_matches(' ');
        match next.strip_prefix(NBSP) {
            Some(after) => rest = after,
            None => return next,
        }
    }
}

fn outside_code(words: &str) -> String {
    words
        .split('`')
        .enumerate()
        .map(|(at, part)| match at % 2 {
            0 => part.replace(NBSP, " "),
            _ => part.to_string(),
        })
        .collect::<Vec<_>>()
        .join("`")
}
