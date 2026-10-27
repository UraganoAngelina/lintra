#[path = "../build_support/fonts.rs"]
mod fonts;

#[test]
fn supplied_faces_are_required_and_valid_truetype_assets() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/fonts");
    fonts::verify(&root).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    for name in &fonts::FONT_FILES[..4] {
        std::fs::copy(root.join(name), scratch.path().join(name)).unwrap();
    }
    assert_eq!(
        fonts::verify(scratch.path()).unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    // Presence alone is insufficient: an empty substitute is not an embedded font.
    std::fs::write(scratch.path().join(fonts::FONT_FILES[4]), []).unwrap();
    assert_eq!(
        fonts::verify(scratch.path()).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
}
