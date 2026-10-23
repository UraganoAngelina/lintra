//! Versioned byte-oriented datasets, loaded once before validation.
pub(crate) mod dataset;
pub(crate) mod evidence;
pub(crate) mod filesystem;
pub(crate) mod in_memory;
pub(crate) mod membership;
mod membership_format;
pub(crate) mod normalization;
pub(crate) mod provider;
