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
    pub allow_shared_export: bool,
    pub resource_budget: Option<PathBuf>,
    pub proceed_over_budget: bool,
    pub dirs: CliDirs,
    pub bundle_manifest: Option<PathBuf>,
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
    HistoryInit(ArchiveArgs),
    HistoryCheck(ArchiveArgs),
    Historical(Box<HistoricalArgs>),
    SchemaLint(LintArgs),
    SchemaList(PathBuf),
    ReferenceList(ReferenceListArgs),
    SchemaImport(ImportArgs),
}
/// Creating an archive is a separate command, never a fallback from opening.
pub struct ArchiveArgs {
    pub archive: PathBuf,
    pub key_id: String,
    pub key_file: PathBuf,
}
pub struct HistoricalArgs {
    pub validation: ValidateArgs,
    pub archive: ArchiveArgs,
    pub submission_id: String,
    pub audit: PathBuf,
    pub audit_summary: PathBuf,
    /// Enables a review prompt after preview; it does not grant approval.
    pub review: bool,
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
            .arg(Arg::new("allow-shared-export").long("allow-shared-export")
                .action(clap::ArgAction::SetTrue)
                .help("Conferma per questo comando il salvataggio su destinazioni remote o non determinabili; non ignora protezioni o errori tecnici"))
            .arg(
                Arg::new("encoding")
                    .long("encoding")
                    .required(true)
                    .value_parser(["utf8"]),
            )
            .arg(path("bundle-manifest"))
            .arg(path("schema-dir"))
            .arg(path("reference-data-dir"))
            .arg(
                Arg::new("reference-policy")
                    .long("reference-policy")
                    .value_parser(["continue", "strict"])
                    .default_value("continue"),
            );
        // Historical preview retains its independent audit/approval contract.
        // Normal LocalOnly/Full receive the new opt-in operational preflight.
        let common = if name == "preview" {
            common
        } else {
            common.arg(path("resource-budget").help("Configurazione JSON v1 di budget soft misurati"))
                .arg(Arg::new("proceed-over-budget").long("proceed-over-budget")
                    .requires("resource-budget").action(clap::ArgAction::SetTrue)
                    .help("Procedi comunque sul budget soft per questa richiesta; non ignora errori tecnici"))
        };
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
        .about("Validazione fixed-width LocalOnly, Full e audit storico locale")
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
        .subcommand(
            Command::new("history")
                .about("Archivio locale e preview storico audit-only con approvazione separata")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("init")
                        .about(
                            "Crea esplicitamente un nuovo archivio locale, senza generare chiavi",
                        )
                        .arg(path("archive").required(true))
                        .arg(text("key-id"))
                        .arg(path("key-file").required(true)),
                )
                .subcommand(
                    Command::new("check")
                        .about("Verifica chiave e archivio esistente senza approvare invii")
                        .arg(path("archive").required(true))
                        .arg(text("key-id"))
                        .arg(path("key-file").required(true)),
                )
                .subcommand(
                    validation_command("preview", true)
                        .about(
                            "Valida Full e confronta lo storico locale; nessun commit automatico",
                        )
                        .arg(path("archive").required(true))
                        .arg(text("key-id"))
                        .arg(path("key-file").required(true))
                        .arg(text("submission-id"))
                        .arg(path("audit").required(true))
                        .arg(path("audit-summary").required(true))
                        .arg(
                            Arg::new("review")
                                .long("review")
                                .action(clap::ArgAction::SetTrue)
                                .help("Dopo la preview richiede conferma esatta o ABORT su stdin"),
                        ),
                ),
        )
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
    let (name, args, historical) = if name == "history" {
        let (operation, args) = args.subcommand().expect("clap requires history operation");
        if operation == "init" {
            return Ok(CliCommand::HistoryInit(archive_args(args)));
        }
        if operation == "check" {
            return Ok(CliCommand::HistoryCheck(archive_args(args)));
        }
        ("validate-flow", args, true)
    } else {
        (name, args, false)
    };
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
    let validation = ValidateArgs {
        allow_shared_export: args.get_flag("allow-shared-export"),
        resource_budget: if historical {
            None
        } else {
            args.get_one::<PathBuf>("resource-budget").cloned()
        },
        proceed_over_budget: !historical && args.get_flag("proceed-over-budget"),
        bundle_manifest: args.get_one::<PathBuf>("bundle-manifest").cloned(),
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
    };
    if historical {
        Ok(CliCommand::Historical(Box::new(HistoricalArgs {
            validation,
            archive: archive_args(args),
            submission_id: required(args, "submission-id"),
            audit: required(args, "audit"),
            audit_summary: required(args, "audit-summary"),
            review: args.get_flag("review"),
        })))
    } else {
        Ok(CliCommand::Validate(Box::new(validation)))
    }
}

fn archive_args(args: &ArgMatches) -> ArchiveArgs {
    ArchiveArgs {
        archive: required(args, "archive"),
        key_id: required(args, "key-id"),
        key_file: required(args, "key-file"),
    }
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
