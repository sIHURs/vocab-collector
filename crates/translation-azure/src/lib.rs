//! Azure Translator implementation of the portable translation contract.

mod client;
mod error;
mod protocol;

pub use client::{AzureTranslationProvider, AzureTranslatorConfig};
