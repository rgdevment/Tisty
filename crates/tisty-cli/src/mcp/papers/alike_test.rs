use super::near;

#[test]
fn titles_sharing_most_of_their_words_are_alike() {
    assert!(near(
        "tisty nube por api",
        "nube por api el camino al 10 10 2026 otra version"
    ));
    assert!(near("login", "login"));
    assert!(!near("nube por api", "recetas de pan"));
    assert!(!near("tisty arquitectura", "tisty nube por api"));
}
