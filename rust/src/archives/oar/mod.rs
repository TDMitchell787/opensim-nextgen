//! OAR (OpenSim Archive) support
//!
//! Handles loading and saving region archives in OpenSim-compatible format.

pub mod reader;
pub mod writer;
pub mod xml_schemas;

pub use reader::{OarLoadOptions, OarLoadResult, OarReader};
pub use writer::{OarSaveOptions, OarSaveResult, OarWriter};
