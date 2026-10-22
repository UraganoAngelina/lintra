//! Binary-only adapter. No CLI symbols are exported from the library.
mod args;
mod exit_code;
mod paths;

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
    match command::validate(args) {
        Ok(status) => exit_code::from_status(status),
        Err(error) => {
            eprintln!("validate failed: {error}");
            exit_code::TECHNICAL
        }
    }
}

#[cfg(test)]
mod tests;
