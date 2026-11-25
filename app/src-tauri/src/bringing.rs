use std::io::Read;

// The hosts Tisty reads its help from never redirect, so a redirect means someone else is answering.
pub(crate) fn client(patience: std::time::Duration) -> Option<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(patience)
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("tisty/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()
}

/// A server may leave out the length or say less than it sends, so the cap also holds on what is read.
pub(crate) fn brought(
    client: &reqwest::blocking::Client,
    url: &str,
    most: u64,
) -> Result<Vec<u8>, String> {
    let answer = client
        .get(url)
        .send()
        .map_err(|why| why.without_url().to_string())?;
    if !answer.status().is_success() {
        return Err(answer.status().to_string());
    }
    if answer.content_length().is_some_and(|said| said > most) {
        return Err(format!("larger than {most} bytes"));
    }
    let mut held = Vec::new();
    answer
        .take(most + 1)
        .read_to_end(&mut held)
        .map_err(|why| why.to_string())?;
    if held.len() as u64 > most {
        return Err(format!("larger than {most} bytes"));
    }
    Ok(held)
}

/// A picture is told by its first bytes, never by the name or the type the server gives it.
pub(crate) fn is_picture(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        || bytes.starts_with(b"\xff\xd8\xff")
        || bytes.starts_with(b"GIF87a")
        || bytes.starts_with(b"GIF89a")
        || (bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP")
}

#[cfg(test)]
#[path = "bringing_test.rs"]
mod tests;
