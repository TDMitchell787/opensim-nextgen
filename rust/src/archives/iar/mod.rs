//! IAR (Inventory Archive) support
//!
//! Handles loading and saving user inventory archives in OpenSim-compatible format.

pub mod reader;
pub mod writer;
pub mod xml_schemas;

pub use reader::{IarLoadOptions, IarLoadResult, IarReader};
pub use writer::{IarSaveOptions, IarSaveResult, IarWriter};
