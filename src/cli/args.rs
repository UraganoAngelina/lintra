//! Parse CLI syntax and map it to the public application request, without I/O.
use super::paths::CliDirs;
use clap::{Arg, ArgMatches, Command, builder::NonEmptyStringValueParser, value_parser};
use lintra::*;
use std::{ffi::OsString, path::PathBuf};

/// Parsed adapter configuration, with real request and explicit output paths.
/// Reference data directories describe bundle selection, not provider availability.
pub struct ValidateArgs {
    pub dirs: CliDirs,
    pub request: ValidationRequest,
    pub encoding: RawResolvedEncoding,
    pub errors: PathBuf,
    pub summary: PathBuf,
}
/// Define only the current Trace/LocalOnly command; later tasks add execution
/// and schema commands. Required encoding prevents locale-based assumptions.
pub fn command() -> Command {
    let text = |name: &'static str| {
        Arg::new(name)
            .long(name)
            .required(true)
            .value_parser(NonEmptyStringValueParser::new())
    };
    let path = |name: &'static str| {
        Arg::new(name)
            .long(name)
            .value_parser(value_parser!(PathBuf))
    };
    Command::new("lintra")
        .about("Validazione fixed-width: perimetro LocalOnly")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("validate")
            .about("Prepara una validazione Trace/LocalOnly; esecuzione disponibile nella task CLI dedicata")
            .arg(text("flow"))
            .arg(text("schema-version"))
            .arg(text("trace"))
            .arg(path("input").required(true))
            .arg(path("errors").visible_alias("output").required(true))
            .arg(path("summary").required(true))
            .arg(Arg::new("encoding").long("encoding").required(true).value_parser(["utf8"]))
            .arg(path("schema-dir"))
            .arg(path("reference-data-dir")))
}
/// Clap's required, typed parsers guarantee the values exist on this path.
/// All file paths stay native PathBuf, including spaces/non-UTF-8 where supported.
fn required<T: Clone + Send + Sync + 'static>(matches: &ArgMatches, name: &str) -> T {
    matches
        .get_one::<T>(name)
        .expect("clap validated required argument")
        .clone()
}
/// Accept native argument sequences for deterministic tests and production argv.
/// No file is read, schema compiled, sink opened or validity inferred here.
pub fn parse<I, T>(arguments: I) -> Result<ValidateArgs, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = command().try_get_matches_from(arguments)?;
    let (_, args) = matches
        .subcommand()
        .expect("clap requires validate subcommand");
    Ok(ValidateArgs {
        dirs: CliDirs {
            schema_dir: args.get_one::<PathBuf>("schema-dir").cloned(),
            reference_data_dir: args.get_one::<PathBuf>("reference-data-dir").cloned(),
        },
        request: ValidationRequest {
            schema_key: SchemaKey {
                flow_id: FlowId(required(args, "flow")),
                version: SchemaVersion(required(args, "schema-version")),
            },
            scope: ValidationScope::Trace(TraceValidationInput {
                trace_id: TraceId(required(args, "trace")),
                source: InputSource::Path(required(args, "input")),
            }),
            mode: ValidationMode::LocalOnly,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: None,
            },
        },
        // The parser accepts only utf8; never infer or replace another profile.
        encoding: RawResolvedEncoding::Utf8,
        errors: required(args, "errors"),
        summary: required(args, "summary"),
    })
}
