// Kept to the letter of the core's `settled` until the core reads its bodies through this crate.
pub fn on_disk(text: &str) -> String {
    if text.is_empty() || text.ends_with('\n') {
        return text.to_string();
    }
    format!("{text}\n")
}
