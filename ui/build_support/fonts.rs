//! Shared with the negative build-contract test. Every shipped face is required:
//! no compile-time success followed by a machine-dependent system-font fallback.
use std::{fs, io, path::Path};

pub const FONT_FILES: [&str; 5] = [
    "IBMPlexSans-Regular.ttf",
    "IBMPlexSans-SemiBold.ttf",
    "IBMPlexSans-Bold.ttf",
    "IBMPlexMono-Regular.ttf",
    "IBMPlexMono-SemiBold.ttf",
];

pub fn verify(root: &Path) -> io::Result<()> {
    for name in FONT_FILES {
        let bytes = fs::read(root.join(name))?;
        // The approved assets are TrueType. A stub/empty file must not satisfy
        // the presence contract; Slint performs the remaining font validation.
        if !bytes.starts_with(&[0, 1, 0, 0]) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid IBM Plex font asset",
            ));
        }
    }
    Ok(())
}
