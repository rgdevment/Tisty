use mermaid_rs_renderer::{RenderOptions, Theme, render_with_options};

// Laying out a graph grows faster than its text: past this a fence is drawn as code instead.
const AT_MOST: usize = 32 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drawn {
    pub light: String,
    pub dark: String,
}

/// The diagram as SVG for each theme, or nothing when the fence cannot be read.
pub fn drawn(source: &str) -> Option<Drawn> {
    if source.trim().is_empty() || source.len() > AT_MOST {
        return None;
    }
    let light = crate::guarded(|| render_with_options(source, RenderOptions::modern()).ok())?;
    let dark = crate::guarded(|| {
        let options = RenderOptions {
            theme: Theme::dark(),
            ..RenderOptions::modern()
        };
        render_with_options(source, options).ok()
    })?;
    Some(Drawn { light, dark })
}
