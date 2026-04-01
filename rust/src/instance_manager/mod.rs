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

pub mod config_loader;
pub mod registry;
pub mod controller;
pub mod health;
pub mod types;
pub mod process_manager;
pub mod announcement;
pub mod access_control;
pub mod discovery;

pub use config_loader::{InstanceConfig, InstancesConfig, load_instances_config};
pub use registry::InstanceRegistry;
pub use controller::InstanceController;
pub use health::HealthChecker;
pub use process_manager::ProcessManager;
pub use types::*;
pub use discovery::{RunningInstanceInfo, write_discovery_file, remove_discovery_file, scan_all_running_instances, find_available_controller_port};
