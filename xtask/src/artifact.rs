//! Release checks only: parse bytes, never execute or resolve an imported library.
use crate::{Failure, Product, invalid};
use goblin::{elf, pe};
use std::{collections::BTreeSet, env, fs, path::Path, process::Command};

const GNU_LIBRARIES: &[&str] = &[
    "libc.so.6",
    "libm.so.6",
    "ld-linux-x86-64.so.2",
    "libgcc_s.so.1",
];
// OS libraries, not whatever happens to be installed on the build host.
const WINDOWS_LIBRARIES: &[&str] = &[
    "advapi32.dll",
    "bcrypt.dll",
    "bcryptprimitives.dll",
    "combase.dll",
    "crypt32.dll",
    "dbgcore.dll",
    "dbghelp.dll",
    "gdi32.dll",
    "iphlpapi.dll",
    "kernel32.dll",
    "kernelbase.dll",
    "mswsock.dll",
    "ncrypt.dll",
    "normaliz.dll",
    "ntdll.dll",
    "ole32.dll",
    "oleaut32.dll",
    "psapi.dll",
    "rpcrt4.dll",
    "secur32.dll",
    "shell32.dll",
    "shlwapi.dll",
    "synchronization.dll",
    "user32.dll",
    "userenv.dll",
    "version.dll",
    "winmm.dll",
    "ws2_32.dll",
    "wsock32.dll",
];

pub fn supported(target: &str) -> bool {
    matches!(
        target,
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" | "x86_64-pc-windows-msvc"
    )
}

fn windows_library(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    // A library name must be a basename, never a path or an arbitrary API path.
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        return false;
    }
    if name.starts_with("libgcc")
        || name.starts_with("libwinpthread")
        || name.starts_with("vcruntime")
        || name.starts_with("msvcp")
        || name.starts_with("api-ms-win-crt-")
    {
        return false;
    }
    WINDOWS_LIBRARIES.contains(&name.as_str())
        || ((name.starts_with("api-ms-win-") || name.starts_with("ext-ms-win-"))
            && name.ends_with(".dll"))
}

#[cfg(test)]
fn check_libraries(target: &str, libraries: &[&str]) -> Result<(), Failure> {
    check_product_libraries(target, libraries, Product::Cli)
}
/// Desktop policy is separate from CLI: no wildcard or arbitrary build-host library.
const UI_WINDOWS_LIBRARIES: &[&str] = &[
    "comctl32.dll",
    "dwmapi.dll",
    "imm32.dll",
    "uxtheme.dll",
    "dwrite.dll",
    "shcore.dll",
    "msimg32.dll",
];
const UI_GNU_LIBRARIES: &[&str] = &["libfontconfig.so.1"];
fn check_product_libraries(
    target: &str,
    libraries: &[&str],
    product: Product,
) -> Result<(), Failure> {
    for name in libraries.iter().copied().collect::<BTreeSet<_>>() {
        let allowed = match target {
            "x86_64-unknown-linux-gnu" => {
                GNU_LIBRARIES.contains(&name)
                    || (product == Product::Ui && UI_GNU_LIBRARIES.contains(&name))
            }
            "x86_64-unknown-linux-musl" => false,
            "x86_64-pc-windows-msvc" => {
                windows_library(name)
                    || (product == Product::Ui
                        && UI_WINDOWS_LIBRARIES.contains(&name.to_ascii_lowercase().as_str()))
            }
            _ => return Err(invalid("unsupported verification target").into()),
        };
        if !allowed {
            return Err(invalid(&format!("forbidden imported library: {name}")).into());
        }
    }
    Ok(())
}

/// Explicit build roots plus conservative, documented host-path signatures.
/// Include aliases and UTF-16 later; never query libraries installed on the host.
pub fn host_paths(repo: &Path) -> Result<Vec<String>, Failure> {
    let home = env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .ok_or_else(|| invalid("home unavailable for artifact verification"))?;
    let home = std::path::PathBuf::from(home);
    let mut paths = vec![
        "/home/",
        "/users/",
        "/root/",
        "/tmp/",
        "/private/tmp/",
        "/__w/",
        ":/a/",
        ":/hostedtoolcache/",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    for path in [
        repo.to_owned(),
        home.clone(),
        env::var_os("CARGO_HOME")
            .map(Into::into)
            .unwrap_or_else(|| home.join(".cargo")),
        env::var_os("RUSTUP_HOME")
            .map(Into::into)
            .unwrap_or_else(|| home.join(".rustup")),
    ] {
        for form in [path.clone(), fs::canonicalize(&path)?] {
            let text = form
                .to_str()
                .ok_or_else(|| invalid("non-Unicode build root"))?;
            if text.len() <= 1 {
                return Err(invalid("build root too broad for path verification").into());
            }
            paths.push(text.to_owned());
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn normalized(bytes: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .map(|b| {
            if *b == b'\\' {
                b'/'
            } else {
                b.to_ascii_lowercase()
            }
        })
        .collect()
}

#[cfg(test)]
fn check_paths(bytes: &[u8], paths: &[String]) -> Result<(), Failure> {
    check_product_paths(bytes, paths, Product::Cli, "x86_64-unknown-linux-gnu")
}

/// x11rb-protocol0.13.2 formats the OS socket from this fixed prefix. It is
/// not a build-host directory. Only the bounded UTF-8 literal is admissible;
/// explicit build roots, other /tmp paths and appended path text still fail.
fn x11_socket_literal(bytes: &[u8], offset: usize) -> bool {
    let prefix = b"/tmp/.x11-unix/x";
    bytes.get(offset..offset + prefix.len()) == Some(prefix)
        && matches!(bytes.get(offset + prefix.len()), None | Some(0 | 0xc0))
}
fn check_product_paths(
    bytes: &[u8],
    paths: &[String],
    product: Product,
    target: &str,
) -> Result<(), Failure> {
    let bytes = normalized(bytes);
    for path in paths {
        let path = path.replace('\\', "/").to_ascii_lowercase();
        let wide: Vec<u16> = path.encode_utf16().collect();
        for needle in [
            path.as_bytes().to_vec(),
            wide.iter().flat_map(|v| v.to_le_bytes()).collect(),
            wide.iter().flat_map(|v| v.to_be_bytes()).collect(),
        ] {
            let needle = normalized(&needle);
            if needle.is_empty() {
                continue;
            }
            for (offset, window) in bytes.windows(needle.len()).enumerate() {
                if window != needle {
                    continue;
                }
                if product == Product::Ui
                    && target == "x86_64-unknown-linux-gnu"
                    && path == "/tmp/"
                    && needle == b"/tmp/"
                    && x11_socket_literal(&bytes, offset)
                {
                    continue;
                }
                return Err(invalid(&format!(
                    "host path detected in artifact: pattern {path:?}, offset {offset:#x}"
                ))
                .into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
pub fn inspect(bytes: &[u8], target: &str, paths: &[String]) -> Result<Vec<String>, Failure> {
    inspect_product(bytes, target, paths, Product::Cli)
}
fn inspect_product(
    bytes: &[u8],
    target: &str,
    paths: &[String],
    product: Product,
) -> Result<Vec<String>, Failure> {
    if !supported(target) || (product == Product::Ui && target.ends_with("musl")) {
        return Err(invalid("unsupported verification target for selected product").into());
    }
    let libraries = if target.ends_with("msvc") {
        let image = pe::PE::parse(bytes)?;
        if !image.is_64
            || image.header.coff_header.machine != 0x8664
            || image.is_lib
            || image.entry == 0
        {
            return Err(invalid("expected x86_64 PE executable").into());
        }
        let optional = image
            .header
            .optional_header
            .as_ref()
            .ok_or_else(|| invalid("missing PE optional header"))?;
        // Goblin exposes ordinary imports. Do not silently certify unexamined
        // delay imports: those images fail explicitly until supported.
        if optional
            .data_directories
            .get_delay_import_descriptor()
            .is_some()
        {
            return Err(invalid("PE delay imports are not supported by this verifier").into());
        }
        image.libraries
    } else {
        let image = elf::Elf::parse(bytes)?;
        if !image.is_64
            || !image.little_endian
            || image.header.e_machine != elf::header::EM_X86_64
            || !matches!(
                image.header.e_type,
                elf::header::ET_EXEC | elf::header::ET_DYN
            )
            || image.entry == 0
        {
            return Err(invalid("expected x86_64 ELF executable").into());
        }
        if target.ends_with("musl") && image.interpreter.is_some() {
            return Err(invalid("musl artifact must be static (no interpreter)").into());
        }
        if target.ends_with("gnu")
            && let Some(interpreter) = image.interpreter
            && !matches!(
                interpreter,
                "/lib64/ld-linux-x86-64.so.2"
                    | "/lib/ld-linux-x86-64.so.2"
                    | "/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2"
            )
        {
            return Err(invalid("unexpected GNU ELF interpreter").into());
        }
        image.libraries
    };
    check_product_libraries(target, &libraries, product)?;
    check_product_paths(bytes, paths, product, target)?;
    Ok(libraries
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect())
}

fn check_tree(text: &str) -> Result<(), Failure> {
    if text
        .lines()
        .any(|line| line.split_whitespace().next() == Some("openssl-sys"))
    {
        return Err(invalid("openssl-sys is present in the dependency tree").into());
    }
    Ok(())
}

pub fn verify_product(
    repo: &Path,
    artifact: &Path,
    target: &str,
    offline: bool,
    product: Product,
) -> Result<(), Failure> {
    let bytes = fs::read(artifact)?;
    let libraries = inspect_product(&bytes, target, &host_paths(repo)?, product)?;
    // Enumerate with a fixed unprefixed format first: cargo tree -i ABSENT
    // returns an error, which must not be confused with an I/O/lock failure.
    let mut cargo = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    cargo.current_dir(repo).args([
        "tree",
        "--locked",
        "--all-features",
        "--target",
        target,
        "--prefix",
        "none",
        "--format",
        "{p}",
    ]);
    cargo.args(["-p", product.package()]);
    if offline {
        cargo.arg("--offline");
    }
    let output = cargo.output()?;
    if !output.status.success() {
        return Err(invalid("cargo tree failed; dependency absence is unverified").into());
    }
    check_tree(&String::from_utf8(output.stdout)?)?;
    println!("artifact_verified=true target={target} libraries={libraries:?} openssl_sys=absent");
    Ok(())
}

pub fn cli(repo: &Path, args: impl IntoIterator<Item = String>) -> Result<(), Failure> {
    let mut args = args.into_iter();
    if args.next().as_deref() != Some("verify-artifact") {
        return Err(invalid("expected verify-artifact").into());
    }
    let (mut target, mut artifact, mut offline) = (None, None, false);
    let mut product = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--product" if product.is_none() => {
                product = Some(Product::parse(
                    &args.next().ok_or_else(|| invalid("missing product"))?,
                )?)
            }
            "--target" if target.is_none() => {
                target = Some(args.next().ok_or_else(|| invalid("missing target"))?)
            }
            "--artifact" if artifact.is_none() => {
                artifact = Some(args.next().ok_or_else(|| invalid("missing artifact"))?)
            }
            "--offline" if !offline => offline = true,
            _ => return Err(invalid("unknown or duplicate verification option").into()),
        }
    }
    let target = target.ok_or_else(|| invalid("explicit verification target required"))?;
    if !supported(&target) {
        return Err(invalid("unsupported verification target").into());
    }
    let artifact = artifact.ok_or_else(|| invalid("explicit artifact required"))?;
    verify_product(
        repo,
        Path::new(&artifact),
        &target,
        offline,
        product.unwrap_or(Product::Cli),
    )
}

/// Test the actual command's failure on a corrupted copy of a valid artifact.
/// Retain both files for diagnosis; do not modify the original executable.
pub fn negative_probe(
    artifact: &Path,
    target: &str,
    offline: bool,
    product: Product,
) -> Result<(), Failure> {
    let mut bytes = fs::read(artifact)?;
    bytes.extend_from_slice(b"\0/home/lintra-verifier-negative-probe/source.rs\0");
    let bad = artifact.with_file_name("deliberately-invalid-artifact.bin");
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&bad)?;
    std::io::Write::write_all(&mut output, &bytes)?;
    drop(output);
    let mut probe = Command::new(env::current_exe()?);
    probe
        .arg("verify-artifact")
        .arg("--product")
        .arg(if product == Product::Ui { "ui" } else { "cli" })
        .arg("--target")
        .arg(target)
        .arg("--artifact")
        .arg(&bad);
    if offline {
        probe.arg("--offline");
    }
    let result = probe.output()?;
    if result.status.code() != Some(1)
        || !String::from_utf8(result.stderr)?.contains("host path detected")
    {
        return Err(invalid("negative artifact probe did not fail for the expected reason").into());
    }
    println!("negative_artifact_probe=rejected exit=1");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_executable_import_tables_are_parsed_and_modified_imports_rejected() {
        let target = if cfg!(windows) {
            "x86_64-pc-windows-msvc"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        let bytes = fs::read(env::current_exe().unwrap()).unwrap();
        // Ignore debug build roots here: this test exercises the real native
        // import table. The release gate supplies actual forbidden host paths.
        let libraries = inspect(&bytes, target, &[]).unwrap();
        assert!(!libraries.is_empty());
        // Keep PT_INTERP intact: this assertion targets the DT_NEEDED/import
        // gate; interpreter rejection has its own independent boundary test.
        let name = libraries
            .iter()
            .find(|name| name.as_str() != "ld-linux-x86-64.so.2")
            .unwrap()
            .as_bytes();
        let mut corrupted = bytes.clone();
        let offsets = bytes
            .windows(name.len())
            .enumerate()
            .filter_map(|(i, value)| (value == name).then_some(i))
            .collect::<Vec<_>>();
        assert!(!offsets.is_empty());
        for offset in offsets {
            corrupted[offset] = b'z';
        }
        assert!(
            inspect(&corrupted, target, &[])
                .unwrap_err()
                .to_string()
                .contains("forbidden imported library")
        );
        let wrong = if cfg!(windows) {
            "x86_64-unknown-linux-gnu"
        } else {
            "x86_64-pc-windows-msvc"
        };
        assert!(inspect(&bytes, wrong, &[]).is_err());
    }
    #[test]
    fn synthetic_static_elf_and_pe_boundaries() {
        // Declared synthetic, minimal ELF64 with no program/section tables.
        let mut linux = vec![0; 64];
        linux[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        linux[16..18].copy_from_slice(&2u16.to_le_bytes());
        linux[18..20].copy_from_slice(&62u16.to_le_bytes());
        linux[20..24].copy_from_slice(&1u32.to_le_bytes());
        linux[24..32].copy_from_slice(&0x401000u64.to_le_bytes());
        linux[52..54].copy_from_slice(&64u16.to_le_bytes());
        linux[54..56].copy_from_slice(&56u16.to_le_bytes());
        linux[58..60].copy_from_slice(&64u16.to_le_bytes());
        assert!(inspect(&linux, "x86_64-unknown-linux-musl", &[]).is_ok());
        let loader = b"/lib64/ld-linux-x86-64.so.2\0";
        linux.resize(120 + loader.len(), 0);
        linux[32..40].copy_from_slice(&64u64.to_le_bytes());
        linux[56..58].copy_from_slice(&1u16.to_le_bytes());
        linux[64..68].copy_from_slice(&3u32.to_le_bytes()); // PT_INTERP
        linux[72..80].copy_from_slice(&120u64.to_le_bytes());
        linux[96..104].copy_from_slice(&(loader.len() as u64).to_le_bytes());
        linux[120..].copy_from_slice(loader);
        assert!(inspect(&linux, "x86_64-unknown-linux-gnu", &[]).is_ok());
        assert!(
            inspect(&linux, "x86_64-unknown-linux-musl", &[])
                .unwrap_err()
                .to_string()
                .contains("no interpreter")
        );
        linux[120] = b'x';
        assert!(
            inspect(&linux, "x86_64-unknown-linux-gnu", &[])
                .unwrap_err()
                .to_string()
                .contains("interpreter")
        );
        linux[18..20].copy_from_slice(&183u16.to_le_bytes());
        assert!(inspect(&linux, "x86_64-unknown-linux-gnu", &[]).is_err());
        // Minimal PE32+ header; no imports. Tests parser/architecture/delay gate.
        let mut windows = vec![0; 512];
        windows[..2].copy_from_slice(b"MZ");
        windows[60..64].copy_from_slice(&128u32.to_le_bytes());
        windows[128..132].copy_from_slice(b"PE\0\0");
        windows[132..134].copy_from_slice(&0x8664u16.to_le_bytes());
        windows[148..150].copy_from_slice(&240u16.to_le_bytes());
        windows[150..152].copy_from_slice(&2u16.to_le_bytes());
        windows[152..154].copy_from_slice(&0x20bu16.to_le_bytes());
        windows[152 + 16..152 + 20].copy_from_slice(&4096u32.to_le_bytes());
        windows[152 + 32..152 + 36].copy_from_slice(&4096u32.to_le_bytes());
        windows[152 + 36..152 + 40].copy_from_slice(&512u32.to_le_bytes());
        windows[152 + 108..152 + 112].copy_from_slice(&16u32.to_le_bytes());
        assert!(inspect(&windows, "x86_64-pc-windows-msvc", &[]).is_ok());
        let delay = 152 + 112 + 13 * 8;
        windows[delay..delay + 4].copy_from_slice(&4096u32.to_le_bytes());
        windows[delay + 4..delay + 8].copy_from_slice(&32u32.to_le_bytes());
        assert!(
            inspect(&windows, "x86_64-pc-windows-msvc", &[])
                .unwrap_err()
                .to_string()
                .contains("delay imports")
        );
    }
    #[test]
    fn strict_library_boundaries_and_case_insensitive_windows_names() {
        assert!(check_libraries("x86_64-unknown-linux-gnu", GNU_LIBRARIES).is_ok());
        assert!(check_libraries("x86_64-unknown-linux-musl", &[]).is_ok());
        for bad in [
            "libgcc_s.so.2",
            "libgcc_s.so.1/extra",
            "libssl.so.3",
            "libpthread.so.0",
            "LIBC.SO.6",
        ] {
            assert!(check_libraries("x86_64-unknown-linux-gnu", &[bad]).is_err());
        }
        assert!(check_libraries("x86_64-unknown-linux-musl", &["libc.so.6"]).is_err());
        for good in [
            "KERNEL32.dll",
            "ntdll.dll",
            "api-ms-win-core-synch-l1-2-0.dll",
        ] {
            assert!(windows_library(good));
        }
        for bad in [
            "libgcc_s_seh-1.dll",
            "libwinpthread-1.dll",
            "VCRUNTIME140.dll",
            "MSVCP140.dll",
            "ucrtbase.dll",
            "api-ms-win-crt-runtime-l1-1-0.dll",
            "custom.dll",
            "C:\\Windows\\kernel32.dll",
            "../kernel32.dll",
        ] {
            assert!(!windows_library(bad));
        }
    }
    #[test]
    fn ascii_and_wide_host_paths_are_detected_without_rejecting_virtual_paths() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut paths = host_paths(repo).unwrap();
        paths.push("C:\\Users\\builder".into());
        for text in ["/home/builder/src/main.rs", "c:/users/BUILDER/src/main.rs"] {
            assert!(check_paths(text.as_bytes(), &paths).is_err());
            let wide = text.encode_utf16().collect::<Vec<_>>();
            for bytes in [
                wide.iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect::<Vec<_>>(),
                wide.iter().flat_map(|v| v.to_be_bytes()).collect(),
            ] {
                assert!(check_paths(&bytes, &paths).is_err());
            }
        }
        assert!(
            check_paths(
                b"/lintra/src/main.rs\0/cargo/registry/src/serde/src/private/de.rs",
                &paths
            )
            .is_ok()
        );
    }
    #[test]
    fn malformed_wrong_format_and_empty_inputs_fail() {
        for data in [b"".as_slice(), b"MZ", b"\x7fELF", b"not an executable"] {
            for target in ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"] {
                assert!(inspect(data, target, &[]).is_err());
            }
        }
        assert!(inspect(&[], "unknown", &[]).is_err());
    }
    #[test]
    fn openssl_detection_matches_package_identity_not_substrings() {
        assert!(check_tree("lintra v0.1.0\nopenssl-sys v0.9.0\n").is_err());
        assert!(check_tree("lintra v0.1.0\nnot-openssl-sys v1\n").is_ok());
    }
    #[test]
    fn verification_cli_requires_explicit_unique_selectors() {
        for args in [
            vec!["verify-artifact"],
            vec!["verify-artifact", "--target", "unknown", "--artifact", "a"],
            vec!["verify-artifact", "--target", "x86_64-pc-windows-msvc"],
            vec!["verify-artifact", "--artifact", "a", "--artifact", "b"],
            vec!["verify-artifact", "--offline", "--offline"],
        ] {
            assert!(cli(Path::new("."), args.into_iter().map(str::to_owned)).is_err());
        }
    }
}

#[cfg(test)]
mod desktop_policy_tests {
    use super::*;
    #[test]
    fn desktop_libraries_do_not_relax_cli_or_allow_non_system_dependencies() {
        assert!(
            check_product_libraries("x86_64-pc-windows-msvc", &["comctl32.dll"], Product::Cli)
                .is_err()
        );
        assert!(
            check_product_libraries(
                "x86_64-unknown-linux-gnu",
                &["libfontconfig.so.1"],
                Product::Ui
            )
            .is_ok()
        );
        assert!(
            check_product_libraries(
                "x86_64-unknown-linux-gnu",
                &["libfontconfig.so.1"],
                Product::Cli
            )
            .is_err()
        );
        for product in [Product::Cli, Product::Ui] {
            for library in [
                "libsqlite3.so.0",
                "libssl.so.3",
                "/tmp/libc.so.6",
                "libmadeup.so.1",
            ] {
                assert!(
                    check_product_libraries("x86_64-unknown-linux-gnu", &[library], product)
                        .is_err()
                );
            }
            assert!(
                check_product_libraries("x86_64-pc-windows-msvc", &["vcruntime140.dll"], product)
                    .is_err()
            );
        }
        assert!(
            check_product_libraries("x86_64-pc-windows-msvc", &["DWRITE.dll"], Product::Ui).is_ok()
        );
    }
}

#[cfg(test)]
mod x11_runtime_path_tests {
    use super::*;
    #[test]
    fn only_the_bounded_x11_protocol_literal_is_allowed_and_all_matches_are_scanned() {
        let paths = vec!["/tmp/".into()];
        let literal = b"/tmp/.X11-unix/X\xc0\0";
        assert!(
            check_product_paths(literal, &paths, Product::Ui, "x86_64-unknown-linux-gnu").is_ok()
        );
        assert!(check_paths(literal, &paths).is_err());
        for invalid in [
            b"/tmp/.X11-unix/X/source.rs\0".as_slice(),
            b"/tmp/.X11-unix/Xmalicious\0",
            b"/tmp/build/project.rs\0",
            b"/tmp/.X11-unix/X\0/tmp/build/source.rs\0",
        ] {
            assert!(
                check_product_paths(invalid, &paths, Product::Ui, "x86_64-unknown-linux-gnu")
                    .is_err()
            );
        }
        assert!(
            check_product_paths(
                literal,
                &["/tmp/.X11-unix/X".into()],
                Product::Ui,
                "x86_64-unknown-linux-gnu"
            )
            .is_err()
        );
    }
}
