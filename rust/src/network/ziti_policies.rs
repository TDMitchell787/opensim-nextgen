use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiService {
    pub name: String,
    pub protocol: String,
    pub port: u16,
    pub port_end: Option<u16>,
    pub required_identity: IdentityType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IdentityType {
    RegionServer,
    GridAdmin,
    Monitoring,
    ViewerGateway,
}

impl std::fmt::Display for IdentityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RegionServer => write!(f, "region-server"),
            Self::GridAdmin => write!(f, "grid-admin"),
            Self::Monitoring => write!(f, "monitoring"),
            Self::ViewerGateway => write!(f, "viewer-gateway"),
        }
    }
}

pub fn default_services() -> Vec<ZitiService> {
    vec![
        ZitiService {
            name: "opensim-robust".to_string(),
            protocol: "tcp".to_string(),
            port: 8503,
            port_end: None,
            required_identity: IdentityType::RegionServer,
            description: "Robust grid services (asset, inventory, auth, grid)".to_string(),
        },
        ZitiService {
            name: "opensim-admin".to_string(),
            protocol: "tcp".to_string(),
            port: 9700,
            port_end: None,
            required_identity: IdentityType::GridAdmin,
            description: "Grid administration API".to_string(),
        },
        ZitiService {
            name: "opensim-metrics".to_string(),
            protocol: "tcp".to_string(),
            port: 9600,
            port_end: None,
            required_identity: IdentityType::Monitoring,
            description: "Prometheus metrics endpoint".to_string(),
        },
        ZitiService {
            name: "opensim-region".to_string(),
            protocol: "udp".to_string(),
            port: 9500,
            port_end: Some(9515),
            required_identity: IdentityType::ViewerGateway,
            description: "Region UDP ports for viewer connections (future)".to_string(),
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiAccessPolicy {
    pub name: String,
    pub service: String,
    pub allowed_identities: Vec<IdentityType>,
    pub dial: bool,
    pub bind: bool,
}

pub fn default_policies() -> Vec<ZitiAccessPolicy> {
    vec![
        ZitiAccessPolicy {
            name: "region-to-robust".to_string(),
            service: "opensim-robust".to_string(),
            allowed_identities: vec![IdentityType::RegionServer],
            dial: true,
            bind: false,
        },
        ZitiAccessPolicy {
            name: "admin-access".to_string(),
            service: "opensim-admin".to_string(),
            allowed_identities: vec![IdentityType::GridAdmin],
            dial: true,
            bind: false,
        },
        ZitiAccessPolicy {
            name: "metrics-access".to_string(),
            service: "opensim-metrics".to_string(),
            allowed_identities: vec![IdentityType::Monitoring, IdentityType::GridAdmin],
            dial: true,
            bind: false,
        },
    ]
}

pub fn generate_setup_script() -> String {
    let mut script = String::from("#!/bin/bash\n");
    script.push_str("# OpenSim OpenZiti Setup Script\n");
    script.push_str("# Run this on a machine with ziti CLI installed and controller access\n\n");
    script.push_str("set -e\n\n");
    script.push_str("CONTROLLER=${OPENSIM_ZITI_CONTROLLER:-\"https://localhost:1280\"}\n\n");

    for svc in default_services() {
        let port_range = if let Some(end) = svc.port_end {
            format!("{}-{}", svc.port, end)
        } else {
            svc.port.to_string()
        };
        script.push_str(&format!(
            "echo \"Creating service: {}\"\n\
             ziti edge create config {}-intercept intercept.v1 \\\n\
             '{{\"protocols\": [\"{}\"], \"addresses\": [\"{}.ziti\"], \"portRanges\": [{{\"low\": {}, \"high\": {}}}]}}'\n\n",
            svc.name, svc.name, svc.protocol, svc.name,
            svc.port, svc.port_end.unwrap_or(svc.port)
        ));
    }

    for identity_type in &[IdentityType::RegionServer, IdentityType::GridAdmin, IdentityType::Monitoring] {
        script.push_str(&format!(
            "echo \"Creating identity: {}\"\n\
             ziti edge create identity device {} -o {}.jwt\n\
             ziti edge enroll {}.jwt\n\n",
            identity_type, identity_type, identity_type, identity_type
        ));
    }

    script.push_str("echo \"OpenZiti setup complete\"\n");
    script
}
