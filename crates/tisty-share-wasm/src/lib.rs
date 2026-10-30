//! What the page that reads a link runs in the browser: it opens the envelope with the key in the link's `#`
//! and draws the text as Tisty draws it, so the server never sees the key nor the words.

pub mod shown;

#[cfg(target_arch = "wasm32")]
mod browser;
