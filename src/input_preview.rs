//! Shared bounded native input probe, without decoding or retaining record bytes.
use crate::*;
use std::{
    error::Error,
    fmt, fs,
    io::{self, Read},
    path::Path,
};

#[derive(Debug)]
#[non_exhaustive]
pub enum ConfigurationError {
    Missing(&'static str),
    InvalidSelection,
    Input(io::Error),
    Bom(InputError),
    Schema(SchemaResolutionError),
}
impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(field) => write!(f, "Seleziona {field}."),
            Self::InvalidSelection => {
                f.write_str("La selezione non appartiene al catalogo corrente.")
            }
            Self::Input(_) => f.write_str("Il file non è accessibile o non è un file regolare."),
            Self::Bom(InputError::BomRejected(_)) => {
                f.write_str("BOM non supportato: il file non è stato modificato.")
            }
            Self::Bom(_) => {
                f.write_str("Lettura del file non riuscita durante il controllo preliminare.")
            }
            Self::Schema(error) => write!(f, "Schema non eseguibile: {error}"),
        }
    }
}
impl Error for ConfigurationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Input(e) => Some(e),
            Self::Bom(e) => Some(e),
            Self::Schema(e) => Some(e),
            _ => None,
        }
    }
}
/// Only counters escape this probe. Raw patient bytes are neither retained,
/// decoded, normalized, logged nor treated as proof of normative validity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputPreview {
    pub file_size: u64,
    pub sampled_bytes: usize,
    pub lf: u64,
    pub crlf: u64,
    pub lone_cr: u64,
    pub reached_eof: bool,
}
impl InputPreview {
    pub fn inspect(path: &Path) -> Result<Self, ConfigurationError> {
        let initial = fs::metadata(path).map_err(ConfigurationError::Input)?;
        if !initial.is_file() {
            return Err(ConfigurationError::Input(io::Error::new(
                io::ErrorKind::InvalidInput,
                "input must be a regular file",
            )));
        }
        let file = fs::File::open(path).map_err(ConfigurationError::Input)?;
        let metadata = file.metadata().map_err(ConfigurationError::Input)?;
        if !metadata.is_file() {
            return Err(ConfigurationError::Input(io::Error::new(
                io::ErrorKind::InvalidInput,
                "input must be a regular file",
            )));
        }
        let mut reader = preflight_bom(file, BomPolicy::Reject).map_err(ConfigurationError::Bom)?;
        let mut result = Self {
            file_size: metadata.len(),
            sampled_bytes: 0,
            lf: 0,
            crlf: 0,
            lone_cr: 0,
            reached_eof: false,
        };
        let mut previous_cr = false;
        let mut buffer = [0u8; 4096];
        while result.sampled_bytes < 65536 {
            let limit = buffer.len().min(65536 - result.sampled_bytes);
            let count = match reader.read(&mut buffer[..limit]) {
                Ok(n) => n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(ConfigurationError::Input(e)),
            };
            if count == 0 {
                result.reached_eof = true;
                break;
            }
            for &byte in &buffer[..count] {
                if byte == b'\n' {
                    if previous_cr {
                        result.crlf += 1;
                    } else {
                        result.lf += 1;
                    }
                } else if previous_cr {
                    result.lone_cr += 1;
                }
                previous_cr = byte == b'\r';
            }
            result.sampled_bytes += count;
        }
        if previous_cr && result.reached_eof {
            result.lone_cr += 1;
        }
        Ok(result)
    }
    pub fn label(&self) -> String {
        let endings = match (self.lf > 0, self.crlf > 0) {
            (true, true) => "LF e CRLF",
            (true, false) => "LF",
            (false, true) => "CRLF",
            _ => "nessun LF nel campione",
        };
        let cr = if self.lone_cr > 0 {
            " · CR isolati rilevati"
        } else {
            ""
        };
        format!(
            "Campione {} byte: {endings} · nessun BOM{cr}",
            self.sampled_bytes
        )
    }
}
