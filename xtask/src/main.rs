//! Native release recipe and artifact checks. No normative input or application API.
mod artifact;
use std::{
    env,
    error::Error,
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
type Failure = Box<dyn Error>;

#[derive(Debug, PartialEq, Eq)]
struct Options {
    target: String,
    offline: bool,
    reproducible: bool,
    verify_artifact: bool,
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
/// Parse only build-tool options; duplicated/unknown selectors fail before cargo.
fn options(args: impl IntoIterator<Item = String>) -> Result<Options, Failure> {
    let mut args = args.into_iter();
    if args.next().as_deref() != Some("build-release") {
        return Err(invalid(
            "expected build-release --target TARGET [--offline] [--verify-reproducible] [--verify-artifact]",
        )
        .into());
    }
    let mut target = None;
    let mut offline = false;
    let mut reproducible = false;
    let mut verify_artifact = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--target" if target.is_none() => {
                target = Some(args.next().ok_or_else(|| invalid("missing target"))?)
            }
            "--offline" if !offline => offline = true,
            "--verify-reproducible" if !reproducible => reproducible = true,
            "--verify-artifact" if !verify_artifact => verify_artifact = true,
            _ => return Err(invalid("unknown or duplicate build option").into()),
        }
    }
    let target = target.ok_or_else(|| invalid("explicit native target required"))?;
    if !matches!(
        target.as_str(),
        "x86_64-unknown-linux-gnu" | "x86_64-pc-windows-msvc"
    ) {
        return Err(invalid("release target must be native Linux GNU or Windows MSVC").into());
    }
    Ok(Options {
        target,
        offline,
        reproducible,
        verify_artifact,
    })
}
/// Unit separators preserve paths with spaces. Include lexical and canonical
/// forms: Windows may expand 8.3 aliases and use extended-length path syntax.
fn add_remap(flags: &mut Vec<String>, source: &Path, replacement: &str) -> io::Result<()> {
    let mut paths = vec![source.to_owned()];
    if source.exists() {
        paths.push(fs::canonicalize(source)?);
    }
    for path in paths {
        let path = path
            .to_str()
            .ok_or_else(|| invalid("release remapping requires Unicode build paths"))?;
        let flag = format!("--remap-path-prefix={path}={replacement}");
        if !flags.contains(&flag) {
            flags.push(flag);
        }
    }
    Ok(())
}
fn command(mut command: Command) -> Result<(), Failure> {
    let status = command.status()?;
    if !status.success() {
        return Err(io::Error::other(format!("build command failed: {status}")).into());
    }
    Ok(())
}
/// Preserve Cargo's highest-priority flags before adding required release flags.
/// No environment variable of the current process is mutated (also safe in tests).
fn inherited_flags(target: &str) -> Result<Vec<String>, Failure> {
    if let Some(encoded) = env::var_os("CARGO_ENCODED_RUSTFLAGS") {
        return Ok(encoded
            .into_string()
            .map_err(|_| invalid("non-Unicode encoded flags"))?
            .split('\x1f')
            .filter(|v| !v.is_empty())
            .map(str::to_owned)
            .collect());
    }
    let name = format!(
        "CARGO_TARGET_{}_RUSTFLAGS",
        target.replace('-', "_").to_ascii_uppercase()
    );
    let text = env::var_os("RUSTFLAGS")
        .or_else(|| env::var_os(name))
        .unwrap_or_default()
        .into_string()
        .map_err(|_| invalid("non-Unicode Rust flags"))?;
    Ok(text.split_whitespace().map(str::to_owned).collect())
}
fn build(
    repo: &Path,
    directory: &Path,
    options: &Options,
    flags: &[String],
) -> Result<PathBuf, Failure> {
    let mut cargo = Command::new(env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo")));
    cargo.current_dir(repo).args([
        "build",
        "--release",
        "--locked",
        "--target",
        &options.target,
    ]);
    if options.offline {
        cargo.arg("--offline");
    }
    cargo
        .env("CARGO_TARGET_DIR", directory)
        .env("CARGO_ENCODED_RUSTFLAGS", flags.join("\x1f"));
    command(cargo)?;
    let binary = if options.target.ends_with("msvc") {
        "lintra.exe"
    } else {
        "lintra"
    };
    let artifact = directory.join(&options.target).join("release").join(binary);
    if !artifact.is_file() {
        return Err(invalid("cargo did not produce the requested executable").into());
    }
    Ok(artifact)
}
fn run(options: Options) -> Result<(), Failure> {
    // The compiled-in path belongs to this development tool, never the product.
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| invalid("tool needs repository parent"))?;
    let home = env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .map(PathBuf::from)
        .ok_or_else(|| invalid("toolchain home unavailable"))?;
    let cargo_home = env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".cargo"));
    let rustup_home = env::var_os("RUSTUP_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".rustup"));
    let base = repo.join("target/release-profile");
    fs::create_dir_all(&base)?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let first = base.join(format!("{}-{nonce}-first", std::process::id()));
    let second = base.join(format!("{}-{nonce}-second", std::process::id()));
    // create_dir, never remove/reuse another build's tree; failures remain in
    // ignored target/ for diagnosis. Both builds use exactly the same flags.
    fs::create_dir(&first)?;
    let mut flags = inherited_flags(&options.target)?;
    for (source, replacement) in [
        (repo, "/lintra"),
        (cargo_home.as_path(), "/cargo"),
        (rustup_home.as_path(), "/rustup"),
        (base.as_path(), "/lintra-build"),
    ] {
        add_remap(&mut flags, source, replacement)?;
    }
    if options.target.ends_with("msvc") {
        // link.exe otherwise embeds the wall-clock time in the PE header.
        // /Brepro requests a content-derived value; byte comparison stays strict.
        flags.extend([
            "-C".into(),
            "target-feature=+crt-static".into(),
            "-C".into(),
            "link-arg=/Brepro".into(),
        ]);
    }
    let artifact = build(repo, &first, &options, &flags)?;
    if options.reproducible {
        fs::create_dir(&second)?;
        let repeated = build(repo, &second, &options, &flags)?;
        let first_bytes = fs::read(&artifact)?;
        let second_bytes = fs::read(&repeated)?;
        if first_bytes != second_bytes {
            // Diagnostic offsets/sizes only, never discard differing bytes or
            // normalize headers to make an unsuccessful comparison pass.
            let offset = first_bytes
                .iter()
                .zip(&second_bytes)
                .position(|(left, right)| left != right)
                .unwrap_or_else(|| first_bytes.len().min(second_bytes.len()));
            return Err(io::Error::other(format!(
                "clean release builds differ; reproducibility gate failed: lengths {}/{}, first difference at {offset:#x}",
                first_bytes.len(), second_bytes.len()
            )).into());
        }
        println!("reproducible=true target={}", options.target);
    }
    println!("artifact={}", artifact.display());
    if options.verify_artifact {
        artifact::verify(repo, &artifact, &options.target, options.offline)?;
        artifact::negative_probe(&artifact, &options.target, options.offline)?;
    }
    Ok(())
}
fn main() -> Result<(), Failure> {
    let args = env::args_os()
        .skip(1)
        .map(|arg| {
            arg.into_string()
                .map_err(|_| invalid("non-Unicode tooling argument"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if args.first().map(String::as_str) == Some("verify-artifact") {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| invalid("tool needs repository parent"))?;
        artifact::cli(repo, args)
    } else {
        run(options(args)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_selector_required_and_no_silent_options() {
        for args in [
            vec!["build-release"],
            vec!["build-release", "--target", "unknown"],
            vec![
                "build-release",
                "--target",
                "x86_64-pc-windows-msvc",
                "--target",
                "x86_64-unknown-linux-gnu",
            ],
        ] {
            assert!(options(args.into_iter().map(str::to_owned)).is_err());
        }
        assert_eq!(
            options(
                [
                    "build-release",
                    "--target",
                    "x86_64-pc-windows-msvc",
                    "--verify-reproducible",
                    "--offline"
                ]
                .map(str::to_owned)
            )
            .unwrap(),
            Options {
                target: "x86_64-pc-windows-msvc".into(),
                offline: true,
                reproducible: true,
                verify_artifact: false,
            }
        );
    }
    #[test]
    fn remapping_preserves_space_containing_flags_as_one_argument() {
        let mut flags = Vec::new();
        add_remap(
            &mut flags,
            Path::new("absent build directory with spaces"),
            "/lintra",
        )
        .unwrap();
        assert_eq!(
            flags,
            ["--remap-path-prefix=absent build directory with spaces=/lintra"]
        );
        assert_eq!(flags.join("\x1f").split('\x1f').collect::<Vec<_>>(), flags);
    }
}
