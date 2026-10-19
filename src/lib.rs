//! Generic fixed-width validation engine.
//!
//! The library can be linked by an external consumer:
//!
//! ```
//! use lintra as _;
//! ```
//!
//! Engine internals are private (DETAILED_DESIGN §94):
//!
//! ```compile_fail,E0603
//! use lintra::engine;
//! ```
//!
//! Core persistence internals are private as well:
//!
//! ```compile_fail,E0603
//! use lintra::persistence;
//! ```
//!
//! Domain identifiers cannot be substituted for one another:
//!
//! ```compile_fail,E0308
//! use lintra::{FlowId, TraceId};
//! let flow: FlowId = TraceId("EXAMPLE".to_owned());
//! ```
//!
//! Byte ranges use zero-based byte offsets, without decoding or copying:
//!
//! ```
//! use lintra::ByteRange;
//! let range = ByteRange::new(1, 2).unwrap();
//! assert_eq!(range.slice(b"ABCD"), Some(&b"BC"[..]));
//! ```

mod application;
mod context;
mod domain;
mod engine;
mod input;
mod persistence;
mod reference_data;
mod report;
mod schema;
mod validators;

// Explicit domain primitives; module internals remain private.
pub use domain::{
    ByteRange, ByteRangeError, DatasetId, DatasetVersion, FieldId, FlowId, JoinId, SchemaVersion,
    TraceId,
};

pub use domain::{InvalidValidationCode, ValidationCode};

pub use domain::{
    RecordIdentity, RunMetadata, RunStatus, SchemaKey, ScopeSummary, ValidationRunOutcome,
    ValidationSummary,
};
