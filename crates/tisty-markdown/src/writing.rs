/// The same body the core keeps: Windows line endings folded, no byte-order mark, a final newline.
pub fn on_disk(text: &str) -> String {
    let bare = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut body = if bare.contains("\r\n") {
        bare.replace("\r\n", "\n")
    } else {
        bare.to_string()
    };
    if !body.is_empty() && !body.ends_with('\n') {
        body.push('\n');
    }
    body
}
