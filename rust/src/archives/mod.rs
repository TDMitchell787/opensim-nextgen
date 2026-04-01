//! IAR/OAR Archive Support for OpenSim Next
//!
//! This module provides full support for:
//! - IAR (Inventory Archive) - User inventory backup/restore
//! - OAR (OpenSim Archive) - Region backup/restore
//!
//! Compatible with OpenSim master archive formats.

pub mod common;
pub mod tar_handler;
pub mod iar;
pub mod oar;
pub mod job_manager;
pub mod api;

#[cfg(test)]
mod tests;

pub use common::*;
pub use iar::{IarReader, IarWriter, IarLoadResult, IarSaveResult, IarLoadOptions, IarSaveOptions};
pub use oar::{OarReader, OarWriter, OarLoadResult, OarSaveResult, OarLoadOptions, OarSaveOptions};
pub use job_manager::{ArchiveJobManager, ArchiveJob, JobStatus, JobType};
pub use api::{ArchiveApiState, create_archive_api_router};
