//! CLI adapter entrypoint; the library has no dependency on argument parsing.
mod cli;

/// Return the adapter's code unchanged; parsing failures cannot masquerade as
/// completed normative invalidity through clap's default process exit code.
fn main() -> std::process::ExitCode {
    std::process::ExitCode::from(cli::run())
}
