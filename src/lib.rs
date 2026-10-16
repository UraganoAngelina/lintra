//! Generic fixed-width validation engine: initial module boundaries.
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
//! Public application types will be introduced by their authorized tasks.

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
