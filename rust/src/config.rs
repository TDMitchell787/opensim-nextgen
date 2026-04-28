//! Configuration for the OpenSim server.

use serde::Deserialize;

pub mod login;

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkConfig {
    pub max_connections: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
        }
    }
}
