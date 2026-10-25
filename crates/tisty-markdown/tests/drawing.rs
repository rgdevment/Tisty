#![cfg(feature = "math")]

use tisty_markdown::math;

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
