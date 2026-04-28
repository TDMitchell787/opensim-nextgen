//! IAR/OAR Archive Support for OpenSim Next
//!
//! This module provides full support for:
//! - IAR (Inventory Archive) - User inventory backup/restore
//! - OAR (OpenSim Archive) - Region backup/restore
//!
//! Compatible with OpenSim master archive formats.

pub mod api;
pub mod common;
pub mod iar;
pub mod job_manager;
pub mod oar;
pub mod tar_handler;

#[cfg(test)]
mod tests;

pub use api::{create_archive_api_router, ArchiveApiState};
pub use common::*;
pub use iar::{IarLoadOptions, IarLoadResult, IarReader, IarSaveOptions, IarSaveResult, IarWriter};
pub use job_manager::{ArchiveJob, ArchiveJobManager, JobStatus, JobType};
pub use oar::{OarLoadOptions, OarLoadResult, OarReader, OarSaveOptions, OarSaveResult, OarWriter};
