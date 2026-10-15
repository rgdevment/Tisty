use std::borrow::Cow;

/// What a body becomes when it is written: one kind of line ending and no byte-order mark.
pub fn settled(text: &str) -> Cow<'_, str> {
    let bare = text.strip_prefix('\u{feff}').unwrap_or(text);
    if bare.contains('\r') {
        Cow::Owned(bare.replace("\r\n", "\n"))
    } else {
        Cow::Borrowed(bare)
    }
}
