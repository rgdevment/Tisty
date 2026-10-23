use mermaid_rs_renderer::{RenderOptions, compute_layout, parse_mermaid_strict, render_svg};

// Laying out a graph grows faster than its text: past this a fence is drawn as code instead.
const AT_MOST: usize = 32 * 1024;
// Text alone does not bound the layout: a short fence can still name thousands of links.
const MOST_PARTS: usize = 1_000;
// Hues that still read once a dark page turns the drawing.
const SLICES: [&str; 12] = [
    "#58aef0", "#ff9d4d", "#6fc44a", "#f07ac0", "#d9b02e", "#b98cf0", "#3fbfa6", "#ff7a8a",
    "#8b95ff", "#b39c86", "#a0a0a8", "#9ad1ff",
];

// Drawn once on a light sheet; a dark page turns it with a filter instead of carrying a second drawing.
pub fn drawn(source: &str) -> Option<String> {
    if source.trim().is_empty() || source.len() > AT_MOST {
        return None;
    }
    crate::guarded(|| {
        let mut options = RenderOptions::modern();
        options.theme.pie_colors = SLICES.map(String::from);
        options.theme.pie_opacity = 1.0;
        let parsed = parse_mermaid_strict(source).ok()?;
        if parsed.graph.nodes.len() + parsed.graph.edges.len() > MOST_PARTS {
            return None;
        }
        let laid = compute_layout(&parsed.graph, &options.theme, &options.layout);
        Some(unbacked(render_svg(&laid, &options.theme, &options.layout)))
    })
}

// The sheet is already behind it; a filled backdrop would turn darker than a dark sheet.
fn unbacked(svg: String) -> String {
    let Some(open) = svg.find('>').map(|at| at + 1) else {
        return svg;
    };
    if !svg[open..].starts_with("<rect ") {
        return svg;
    }
    match svg[open..].find("/>") {
        Some(end) => format!("{}{}", &svg[..open], &svg[open + end + 2..]),
        None => svg,
    }
}
