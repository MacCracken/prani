//! Error types for the prani crate.

use alloc::string::String;
use serde::{Deserialize, Serialize};

/// Errors that can occur during creature vocal synthesis.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[non_exhaustive]
pub enum PraniError {
    /// A vocal tract parameter is out of valid range.
    #[error("invalid tract parameter: {0}")]
    InvalidTract(String),

    /// A vocalization parameter is invalid.
    #[error("invalid vocalization: {0}")]
    InvalidVocalization(String),

    /// The species configuration is invalid.
    #[error("invalid species: {0}")]
    InvalidSpecies(String),

    /// A synthesis step failed.
    #[error("synthesis failed: {0}")]
    SynthesisFailed(String),

    /// An underlying svara operation failed.
    #[error("svara error: {0}")]
    Svara(String),
}

impl From<svara::error::SvaraError> for PraniError {
    fn from(e: svara::error::SvaraError) -> Self {
        Self::Svara(alloc::format!("{e}"))
    }
}

/// Convenience type alias for prani results.
pub type Result<T> = core::result::Result<T, PraniError>;
