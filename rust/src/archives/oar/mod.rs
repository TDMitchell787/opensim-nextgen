//! OAR (OpenSim Archive) support
//!
//! Handles loading and saving region archives in OpenSim-compatible format.

pub mod reader;
pub mod writer;
pub mod xml_schemas;

pub use reader::{OarReader, OarLoadResult, OarLoadOptions};
pub use writer::{OarWriter, OarSaveResult, OarSaveOptions};
