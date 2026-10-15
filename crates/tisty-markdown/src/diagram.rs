use mermaid_rs_renderer::{RenderOptions, Theme, compute_layout, parse_mermaid_strict, render_svg};

// Laying out a graph grows faster than its text: past this a fence is drawn as code instead.
const AT_MOST: usize = 32 * 1024;
// Text alone does not bound the layout: a short fence can still name thousands of links.
const MOST_PARTS: usize = 1_000;
// A slice takes its colour when the chart is laid out, before a theme is picked, so both themes
// share hues that read on a light sheet and on a dark one.
const SLICES: [&str; 12] = [
    "#58aef0", "#ff9d4d", "#6fc44a", "#f07ac0", "#d9b02e", "#b98cf0", "#3fbfa6", "#ff7a8a",
    "#8b95ff", "#b39c86", "#a0a0a8", "#9ad1ff",
];

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
        let mut light = RenderOptions::modern();
        light.theme.pie_colors = SLICES.map(String::from);
        light.theme.pie_opacity = 1.0;
        // Both themes share one typeface, so one layout serves both and only the colours change.
        let dark = Theme {
            font_family: light.theme.font_family.clone(),
            font_size: light.theme.font_size,
            pie_colors: light.theme.pie_colors.clone(),
            pie_opacity: 1.0,
            pie_section_text_color: "#1c1c1e".to_string(),
            pie_outer_stroke_color: "#232326".to_string(),
            ..Theme::dark()
        };
        let parsed = parse_mermaid_strict(source).ok()?;
        if parsed.graph.nodes.len() + parsed.graph.edges.len() > MOST_PARTS {
            return None;
        }
        let laid = compute_layout(&parsed.graph, &light.theme, &light.layout);
        Some(Drawn {
            light: render_svg(&laid, &light.theme, &light.layout),
            dark: render_svg(&laid, &dark, &light.layout),
        })
    })
}
