//! Binary-only adapter. No CLI symbols are exported from the library.
mod args;
mod exit_code;
mod paths;

/// T-P8-01 defines syntax/mapping/resolution; command execution belongs to P8-03.
/// A parsed request must never appear to have been validated by this skeleton.
pub fn run() -> u8 {
    let args = match args::parse(std::env::args_os()) {
        Ok(args) => args,
        Err(error) => {
            let code = exit_code::from_parse_error(&error);
            let _ = error.print();
            return code;
        }
    };
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(error) => {
            eprintln!("cannot locate executable: {error}");
            return exit_code::TECHNICAL;
        }
    };
    if let Err(error) = paths::resolve_bundle_source(&args.dirs, &paths::SystemEnv, &exe) {
        eprintln!("{error}");
        return exit_code::TECHNICAL;
    }
    // Destructure to make the prepared adapter state explicit. These values are
    // not sent to a validator until the command task wires real output handling.
    let args::ValidateArgs {
        request,
        encoding,
        errors,
        summary,
        ..
    } = args;
    let _prepared = (request, encoding, errors, summary);
    eprintln!("validate execution is not available in this CLI skeleton");
    exit_code::TECHNICAL
}

#[cfg(test)]
mod tests;
