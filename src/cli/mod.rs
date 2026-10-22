//! Binary-only adapter. No CLI symbols are exported from the library.
mod args;
mod exit_code;
mod paths;
mod signal;

mod command;

/// Parsing stays separate from execution; every technical failure maps to 1.
pub fn run() -> u8 {
    let args = match args::parse(std::env::args_os()) {
        Ok(args) => args,
        Err(error) => {
            let code = exit_code::from_parse_error(&error);
            if error.print().is_err() {
                return exit_code::TECHNICAL;
            }
            return code;
        }
    };
    let result = match args {
        args::CliCommand::Validate(args) => command::validate(*args).map(exit_code::from_status),
        args::CliCommand::SchemaLint(args) => command::schema_lint(args),
    };
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("command failed: {error}");
            exit_code::TECHNICAL
        }
    }
}

#[cfg(test)]
mod tests;
