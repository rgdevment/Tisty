use mermaid_rs_renderer::{RenderOptions, Theme, compute_layout, parse_mermaid_strict, render_svg};

// Laying out a graph grows faster than its text: past this a fence is drawn as code instead.
const AT_MOST: usize = 32 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drawn {
    pub light: String,
    pub dark: String,
}

pub fn drawn(source: &str) -> Option<Drawn> {
    if source.trim().is_empty() || source.len() > AT_MOST {
        return None;
    }
    crate::guarded(|| {
        let light = RenderOptions::modern();
        // Both themes share one typeface, so one layout serves both and only the colours change.
        let dark = Theme {
            font_family: light.theme.font_family.clone(),
            font_size: light.theme.font_size,
            ..Theme::dark()
        };
        let parsed = parse_mermaid_strict(source).ok()?;
        let laid = compute_layout(&parsed.graph, &light.theme, &light.layout);
        Some(Drawn {
            light: render_svg(&laid, &light.theme, &light.layout),
            dark: render_svg(&laid, &dark, &light.layout),
        })
    })
}
