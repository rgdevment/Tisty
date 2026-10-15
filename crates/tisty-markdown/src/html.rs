pub fn attribute(tag: &str, name: &str) -> Option<String> {
    let open = format!("{name}=\"");
    let from = tag
        .match_indices(&open)
        .map(|(at, _)| at)
        .find(|at| tag[..*at].ends_with(char::is_whitespace))?
        + open.len();
    let to = from + tag[from..].find('"')?;
    Some(unquoted(&tag[from..to]))
}

pub fn unquoted(said: &str) -> String {
    said.replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}
