//! Parse CLI syntax and map it to the public application request, without I/O.
use super::paths::CliDirs;
use clap::{Arg, ArgMatches, Command, builder::NonEmptyStringValueParser, value_parser};
use lintra::*;
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    path::PathBuf,
};

/// Parsed adapter configuration, with real request and explicit output paths.
/// Reference data directories describe bundle selection, not provider availability.
pub struct ValidateArgs {
    pub dirs: CliDirs,
    pub request: ValidationRequest,
    pub encoding: RawResolvedEncoding,
    pub reference_policy: DatasetPreparationPolicy,
    pub errors: Option<PathBuf>,
    pub xlsx: Option<PathBuf>,
    pub summary: PathBuf,
    pub parameters: BTreeMap<String, String>,
}
/// Binary commands carry only the configuration needed by their operation.
pub enum CliCommand {
    Validate(Box<ValidateArgs>),
    SchemaLint(LintArgs),
    SchemaList(PathBuf),
    ReferenceList(ReferenceListArgs),
    SchemaImport(ImportArgs),
}
pub struct ReferenceListArgs {
    pub schema_root: PathBuf,
    pub reference_root: PathBuf,
    pub key: SchemaKey,
}
pub struct ImportArgs {
    pub input: PathBuf,
    pub blueprint: PathBuf,
    pub profile: PathBuf,
    pub output: PathBuf,
    pub aliases: Option<PathBuf>,
}
/// Lint checks an explicitly selected version using the production registry.
/// A schema-only operation does not require a reference-data bundle or input.
pub struct LintArgs {
    pub schema_root: PathBuf,
    pub key: SchemaKey,
    pub encoding: RawResolvedEncoding,
}
/// Required encoding prevents locale-based assumptions for both commands.
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
    let validation_command = |name: &'static str, full: bool| {
        let common = Command::new(name)
            .about(if full {
                "Valida un bundle Full con correlazioni fra tracciati"
            } else {
                "Valida un tracciato LocalOnly"
            })
            .arg(text("flow"))
            .arg(text("schema-version"))
            .arg(
                path("errors")
                    .visible_alias("output")
                    .required_unless_present("xlsx"),
            )
            .arg(path("xlsx"))
            .arg(path("summary").required(true))
            .arg(
                Arg::new("encoding")
                    .long("encoding")
                    .required(true)
                    .value_parser(["utf8"]),
            )
            .arg(path("schema-dir"))
            .arg(path("reference-data-dir"))
            .arg(
                Arg::new("reference-policy")
                    .long("reference-policy")
                    .value_parser(["continue", "strict"])
                    .default_value("continue"),
            );
        if full {
            common
                .arg(
                    Arg::new("trace-input")
                        .long("trace-input")
                        .required(true)
                        .action(clap::ArgAction::Append)
                        .value_parser(value_parser!(OsString))
                        .value_name("ID=PATH"),
                )
                .arg(
                    Arg::new("parameter")
                        .long("parameter")
                        .action(clap::ArgAction::Append)
                        .value_parser(NonEmptyStringValueParser::new())
                        .value_name("ID=VALUE"),
                )
        } else {
            common.arg(text("trace")).arg(path("input").required(true))
        }
    };
    Command::new("lintra")
        .about("Validazione fixed-width LocalOnly e Full")
        .version(concat!(
            env!("CARGO_PKG_VERSION"),
            " (commit ",
            env!("LINTRA_BUILD_COMMIT"),
            ")"
        ))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("schema")
                .about("Strumenti per gli schemi versionati")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("list")
                        .about("Elenca schemi acquisibili, senza certificarli")
                        .arg(path("schema-dir").required(true)),
                )
                .subcommand(
                    Command::new("import")
                        .about("Importa un draft XLSX con blueprint e profilo revisionati")
                        .arg(path("input").required(true))
                        .arg(path("blueprint").required(true))
                        .arg(path("profile").required(true))
                        .arg(path("output").required(true))
                        .arg(path("type-aliases")),
                )
                .subcommand(
                    Command::new("lint")
                        .about("Verifica lo schema con il compiler attualmente supportato")
                        .arg(text("flow"))
                        .arg(text("schema-version"))
                        .arg(path("schema-dir").required(true))
                        .arg(
                            Arg::new("encoding")
                                .long("encoding")
                                .required(true)
                                .value_parser(["utf8"]),
                        ),
                ),
        )
        .subcommand(
            Command::new("reference-data")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("list")
                        .about("Inventario diagnostico delle dipendenze dello schema")
                        .arg(path("schema-dir").required(true))
                        .arg(path("reference-data-dir").required(true))
                        .arg(text("flow"))
                        .arg(text("schema-version")),
                ),
        )
        .subcommand(validation_command("validate", false))
        .subcommand(validation_command("validate-flow", true))
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
pub fn parse<I, T>(arguments: I) -> Result<CliCommand, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = command().try_get_matches_from(arguments)?;
    let (name, args) = matches.subcommand().expect("clap requires a subcommand");
    if name == "reference-data" {
        let (_, list) = args
            .subcommand()
            .expect("clap requires reference-data subcommand");
        return Ok(CliCommand::ReferenceList(ReferenceListArgs {
            schema_root: required(list, "schema-dir"),
            reference_root: required(list, "reference-data-dir"),
            key: SchemaKey {
                flow_id: FlowId(required(list, "flow")),
                version: SchemaVersion(required(list, "schema-version")),
            },
        }));
    }
    if name == "schema" {
        let (operation, lint) = args.subcommand().expect("clap requires schema subcommand");
        if operation == "list" {
            return Ok(CliCommand::SchemaList(required(lint, "schema-dir")));
        }
        if operation == "import" {
            return Ok(CliCommand::SchemaImport(ImportArgs {
                input: required(lint, "input"),
                blueprint: required(lint, "blueprint"),
                profile: required(lint, "profile"),
                output: required(lint, "output"),
                aliases: lint.get_one::<PathBuf>("type-aliases").cloned(),
            }));
        }
        return Ok(CliCommand::SchemaLint(LintArgs {
            schema_root: required(lint, "schema-dir"),
            key: SchemaKey {
                flow_id: FlowId(required(lint, "flow")),
                version: SchemaVersion(required(lint, "schema-version")),
            },
            encoding: RawResolvedEncoding::Utf8,
        }));
    }
    let (scope, mode, parameters) = if name == "validate-flow" {
        let traces = args
            .get_many::<OsString>("trace-input")
            .expect("clap requires trace inputs")
            .map(|value| {
                split_trace_input(value).map_err(|message| {
                    clap::Error::raw(clap::error::ErrorKind::InvalidValue, message)
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut parameters = BTreeMap::new();
        for parameter in args.get_many::<String>("parameter").into_iter().flatten() {
            let Some((id, value)) = parameter.split_once('=') else {
                return Err(clap::Error::raw(
                    clap::error::ErrorKind::InvalidValue,
                    "parameter requires ID=VALUE",
                ));
            };
            if id.is_empty() || parameters.insert(id.into(), value.into()).is_some() {
                return Err(clap::Error::raw(
                    clap::error::ErrorKind::InvalidValue,
                    "parameter ID is empty or duplicated",
                ));
            }
        }
        (
            ValidationScope::Flow(FlowValidationInput { traces }),
            ValidationMode::Full,
            parameters,
        )
    } else {
        (
            ValidationScope::Trace(TraceValidationInput {
                trace_id: TraceId(required(args, "trace")),
                source: InputSource::Path(required(args, "input")),
            }),
            ValidationMode::LocalOnly,
            BTreeMap::new(),
        )
    };
    Ok(CliCommand::Validate(Box::new(ValidateArgs {
        dirs: CliDirs {
            schema_dir: args.get_one::<PathBuf>("schema-dir").cloned(),
            reference_data_dir: args.get_one::<PathBuf>("reference-data-dir").cloned(),
        },
        request: ValidationRequest {
            schema_key: SchemaKey {
                flow_id: FlowId(required(args, "flow")),
                version: SchemaVersion(required(args, "schema-version")),
            },
            scope,
            mode,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: None,
            },
        },
        // The parser accepts only utf8; never infer or replace another profile.
        encoding: RawResolvedEncoding::Utf8,
        reference_policy: match required::<String>(args, "reference-policy").as_str() {
            "strict" => DatasetPreparationPolicy::Strict,
            _ => DatasetPreparationPolicy::ContinueWithAvailableEvidence,
        },
        errors: args.get_one::<PathBuf>("errors").cloned(),
        xlsx: args.get_one::<PathBuf>("xlsx").cloned(),
        summary: required(args, "summary"),
        parameters,
    })))
}

#[cfg(unix)]
fn split_trace_input(value: &OsStr) -> Result<TraceValidationInput, &'static str> {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let bytes = value.as_bytes();
    let index = bytes
        .iter()
        .position(|byte| *byte == b'=')
        .ok_or("trace input requires ID=PATH")?;
    let id = std::str::from_utf8(&bytes[..index]).map_err(|_| "trace ID must be UTF-8")?;
    if id.is_empty() || index + 1 == bytes.len() {
        return Err("trace input ID and PATH must be nonempty");
    }
    Ok(TraceValidationInput {
        trace_id: TraceId(id.into()),
        source: InputSource::Path(PathBuf::from(OsString::from_vec(
            bytes[index + 1..].to_vec(),
        ))),
    })
}
#[cfg(windows)]
fn split_trace_input(value: &OsStr) -> Result<TraceValidationInput, &'static str> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    let units = value.encode_wide().collect::<Vec<_>>();
    let index = units
        .iter()
        .position(|unit| *unit == u16::from(b'='))
        .ok_or("trace input requires ID=PATH")?;
    let id = String::from_utf16(&units[..index]).map_err(|_| "trace ID must be Unicode")?;
    if id.is_empty() || index + 1 == units.len() {
        return Err("trace input ID and PATH must be nonempty");
    }
    Ok(TraceValidationInput {
        trace_id: TraceId(id),
        source: InputSource::Path(PathBuf::from(OsString::from_wide(&units[index + 1..]))),
    })
}
