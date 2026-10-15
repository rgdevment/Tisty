#![cfg(all(feature = "math", feature = "mermaid"))]

use tisty_markdown::{diagram, math};

#[test]
fn a_formula_from_the_guide_becomes_mathml_the_system_draws() {
    let block = math::mathml(r"\sum_{i=1}^{n} i = \frac{n(n+1)}{2}", true).expect("a formula");
    assert!(block.starts_with("<math"));
    assert!(block.contains("display=\"block\""));
    let inline = math::mathml(r"x^2", false).expect("a formula");
    assert!(!inline.contains("display=\"block\""));
}

#[test]
fn a_formula_that_cannot_be_read_draws_nothing() {
    assert_eq!(math::mathml(r"\frac{1}{", true), None);
    assert_eq!(math::mathml("   ", true), None);
    assert_eq!(math::mathml(&"x".repeat(9 * 1024), true), None);
}

#[test]
fn words_inside_a_formula_never_become_markup() {
    let said = math::mathml(r"\text{<script>alert(1)</script>}", true).expect("a formula");
    assert!(!said.contains("<script"));
}

#[test]
fn a_diagram_from_the_guide_is_drawn_for_both_themes() {
    let drawn = diagram::drawn("flowchart TD\n  A[Rina] --> B{Out?}\n  B -- No --> C[Walk]")
        .expect("a diagram");
    assert!(drawn.light.contains("<svg"));
    assert!(drawn.dark.contains("<svg"));
    assert_ne!(drawn.light, drawn.dark);
}

#[test]
fn a_pie_shows_the_same_hues_in_its_slices_and_its_legend_in_both_themes() {
    let drawn =
        diagram::drawn("pie title Walks\n  \"Morning\" : 7\n  \"Evening\" : 5").expect("a pie");
    for svg in [&drawn.light, &drawn.dark] {
        assert!(svg.contains("fill=\"#58aef0\""), "{svg}");
        assert!(svg.contains("fill=\"#ff9d4d\""), "{svg}");
        assert!(!svg.contains("opacity=\"0.700\""), "{svg}");
    }
}

#[test]
fn a_diagram_that_cannot_be_read_draws_nothing() {
    assert_eq!(diagram::drawn("flowchart TD\n  A --> "), None);
    assert_eq!(diagram::drawn(""), None);
    assert_eq!(
        diagram::drawn(&format!("graph TD\n{}", "  A --> B\n".repeat(4000))),
        None
    );
}

#[test]
fn labels_inside_a_diagram_never_become_markup() {
    let drawn = diagram::drawn(
        "flowchart LR\n  A[\"<script>alert(2)</script>\"] --> B[\"<img src=x onerror=alert(1)>\"]",
    )
    .expect("a diagram");
    for svg in [&drawn.light, &drawn.dark] {
        assert!(!svg.contains("<script"));
        assert!(!svg.contains("<img"));
    }
}
