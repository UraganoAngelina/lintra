#[path = "build_support/fonts.rs"]
mod fonts;

fn main() {
    let root = std::path::Path::new("ui/fonts");
    for name in fonts::FONT_FILES {
        println!("cargo:rerun-if-changed={}", root.join(name).display());
    }
    fonts::verify(root).expect("required embedded IBM Plex fonts are missing or invalid");
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-light".into())
        .embed_resources(slint_build::EmbedResourcesKind::EmbedFiles);
    slint_build::compile_with_config("ui/main.slint", config)
        .expect("Slint bootstrap compilation failed");
}
