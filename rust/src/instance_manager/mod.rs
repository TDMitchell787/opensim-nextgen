//! Instance Manager Module
//!
//! Provides multi-instance command and control functionality for managing
//! multiple OpenSim-next server instances from a centralized dashboard.
//!
//! Architecture:
//! - ConfigLoader: Parses instances.toml configuration
//! - Registry: Tracks connected instances and their status
//! - Controller: Executes commands on instances
//! - Health: Monitors instance health and connectivity
//! - WebSocketHandler: Routes WebSocket messages for instance control

pub mod access_control;
pub mod announcement;
pub mod config_loader;
pub mod controller;
pub mod discovery;
pub mod health;
pub mod process_manager;
pub mod registry;
pub mod types;

pub use config_loader::{load_instances_config, InstanceConfig, InstancesConfig};
pub use controller::InstanceController;
pub use discovery::{
    find_available_controller_port, remove_discovery_file, scan_all_running_instances,
    write_discovery_file, RunningInstanceInfo,
};
pub use health::HealthChecker;
pub use process_manager::ProcessManager;
pub use registry::InstanceRegistry;
pub use types::*;
