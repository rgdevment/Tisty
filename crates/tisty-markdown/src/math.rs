use math_core::{LatexToMathML, MathCoreConfig, MathDisplay};

const AT_MOST: usize = 8 * 1024;

pub fn mathml(latex: &str, block: bool) -> Option<String> {
    if latex.trim().is_empty() || latex.len() > AT_MOST {
        return None;
    }
    let config = MathCoreConfig {
        ignore_unknown_commands: true,
        ..Default::default()
    };
    let display = if block {
        MathDisplay::Block
    } else {
        MathDisplay::Inline
    };
    crate::guarded(|| {
        let converter = LatexToMathML::new(config, Vec::new()).ok()?;
        converter
            .convert_with_local_state(latex, display)
            .ok()
            .map(|done| done.mathml)
    })
}
