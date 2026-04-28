use super::models::*;
use std::collections::HashMap;

pub struct BuiltInTemplates;

impl BuiltInTemplates {
    pub fn get_all() -> Vec<SimulatorTemplate> {
        vec![
            Self::mainland(),
            Self::island(),
            Self::marina(),
            Self::sandbox(),
            Self::welcome(),
            Self::event(),
            Self::shopping(),
            Self::roleplay(),
            Self::residential(),
            Self::void_region(),
            Self::custom_terrain(),
        ]
    }

    pub fn get_by_type(template_type: SimulatorType) -> Option<SimulatorTemplate> {
        Self::get_all()
            .into_iter()
            .find(|t| t.template_type == template_type)
    }

    pub fn get_by_id(id: &str) -> Option<SimulatorTemplate> {
        Self::get_all().into_iter().find(|t| t.id == id)
    }

    fn mainland() -> SimulatorTemplate {
        SimulatorTemplate {
            id: "mainland".to_string(),
            name: "Mainland".to_string(),
            description:
                "Standard land region with moderate capacity, suitable for mixed-use development"
                    .to_string(),
            category: "builtin".to_string(),
            template_type: SimulatorType::Mainland,
            opensim_ini: OpenSimIniConfig {
                grid_name: "Mainland Grid".to_string(),
                welcome_message: "Welcome to Mainland!".to_string(),
                allow_anonymous_login: false,
                http_port: 9000,
                external_host_name: "localhost".to_string(),
                internal_ip: "0.0.0.0".to_string(),
                database_provider: DatabaseProvider::Sqlite,
                connection_string: "Data Source=opensim.db;Version=3".to_string(),
                physics_engine: PhysicsEngine::UbOde,
                enable_voice: false,
                enable_search: true,
                enable_currency: false,
                additional_sections: HashMap::new(),
            },
            region_ini: RegionIniConfig {
                region_name: "Mainland Region".to_string(),
                region_uuid: uuid::Uuid::new_v4().to_string(),
                location_x: 1000,
                location_y: 1000,
                size_x: 256,
                size_y: 256,
                internal_port: 9000,
                max_agents: 40,
                max_prims: 45000,
                estate_name: "Mainland Estate".to_string(),
                estate_owner: "Admin".to_string(),
            },
            ossl_config: OsslConfig {
                default_threat_level: OsslThreatLevel::Low,
                allowed_functions: vec![],
                blocked_functions: vec![],
                enable_npc: false,
                enable_teleport: true,
                enable_dynamic_textures: true,
                enable_json_store: true,
            },
            config_includes: HashMap::new(),
            system_requirements: SystemRequirements {
                min_memory_mb: 1024,
                recommended_memory_mb: 2048,
                min_cpu_cores: 2,
                recommended_cpu_cores: 2,
                network_bandwidth_mbps: 100,
                disk_space_gb: 10,
                notes: Some("Standard mainland configuration suitable for most uses".to_string()),
            },
            container_config: None,
            thumbnail_data: None,
        }
    }

    fn island() -> SimulatorTemplate {
        SimulatorTemplate {
            id: "island".to_string(),
            name: "Island".to_string(),
            description:
                "Isolated standalone region surrounded by water, perfect for private estates"
                    .to_string(),
            category: "builtin".to_string(),
            template_type: SimulatorType::Island,
            opensim_ini: OpenSimIniConfig {
                grid_name: "Island Paradise".to_string(),
                welcome_message: "Welcome to your private island!".to_string(),
                allow_anonymous_login: false,
                http_port: 9000,
                external_host_name: "localhost".to_string(),
                internal_ip: "0.0.0.0".to_string(),
                database_provider: DatabaseProvider::Sqlite,
                connection_string: "Data Source=opensim.db;Version=3".to_string(),
                physics_engine: PhysicsEngine::UbOde,
                enable_voice: false,
                enable_search: true,
                enable_currency: false,
                additional_sections: HashMap::new(),
            },
            region_ini: RegionIniConfig {
                region_name: "Private Island".to_string(),
                region_uuid: uuid::Uuid::new_v4().to_string(),
                location_x: 1000,
                location_y: 1000,
                size_x: 256,
                size_y: 256,
                internal_port: 9000,
                max_agents: 30,
                max_prims: 30000,
                estate_name: "Island Estate".to_string(),
                estate_owner: "Admin".to_string(),
            },
            ossl_config: OsslConfig {
                default_threat_level: OsslThreatLevel::Low,
                allowed_functions: vec![],
                blocked_functions: vec![],
                enable_npc: false,
                enable_teleport: true,
                enable_dynamic_textures: true,
                enable_json_store: true,
            },
            config_includes: HashMap::new(),
            system_requirements: SystemRequirements {
                min_memory_mb: 768,
                recommended_memory_mb: 1536,
                min_cpu_cores: 1,
                recommended_cpu_cores: 2,
                network_bandwidth_mbps: 50,
                disk_space_gb: 8,
                notes: Some("Lightweight island configuration for private use".to_string()),
            },
            container_config: None,
            thumbnail_data: None,
        }
    }

    fn marina() -> SimulatorTemplate {
        SimulatorTemplate {
            id: "marina".to_string(),
            name: "Marina".to_string(),
            description: "Water-focused region with boat physics and dock infrastructure"
                .to_string(),
            category: "builtin".to_string(),
            template_type: SimulatorType::Marina,
            opensim_ini: OpenSimIniConfig {
                grid_name: "Marina Bay".to_string(),
                welcome_message: "Welcome to Marina Bay!".to_string(),
                allow_anonymous_login: false,
                http_port: 9000,
                external_host_name: "localhost".to_string(),
                internal_ip: "0.0.0.0".to_string(),
                database_provider: DatabaseProvider::Sqlite,
                connection_string: "Data Source=opensim.db;Version=3".to_string(),
                physics_engine: PhysicsEngine::UbOde,
                enable_voice: true,
                enable_search: true,
                enable_currency: false,
                additional_sections: HashMap::new(),
            },
            region_ini: RegionIniConfig {
                region_name: "Marina Bay".to_string(),
                region_uuid: uuid::Uuid::new_v4().to_string(),
                location_x: 1000,
                location_y: 1000,
                size_x: 256,
                size_y: 256,
                internal_port: 9000,
                max_agents: 40,
                max_prims: 35000,
                estate_name: "Marina Estate".to_string(),
                estate_owner: "Admin".to_string(),
            },
            ossl_config: OsslConfig {
                default_threat_level: OsslThreatLevel::Moderate,
                allowed_functions: vec![],
                blocked_functions: vec![],
                enable_npc: false,
                enable_teleport: true,
                enable_dynamic_textures: true,
                enable_json_store: true,
            },
            config_includes: HashMap::new(),
            system_requirements: SystemRequirements {
                min_memory_mb: 1024,
                recommended_memory_mb: 2048,
                min_cpu_cores: 2,
                recommended_cpu_cores: 2,
                network_bandwidth_mbps: 75,
                disk_space_gb: 10,
                notes: Some("Optimized for water vehicles and boat physics".to_string()),
            },
            container_config: None,
            thumbnail_data: None,
        }
    }

    fn sandbox() -> SimulatorTemplate {
        SimulatorTemplate {
            id: "sandbox".to_string(),
            name: "Sandbox".to_string(),
            description:
                "Testing and building area with high prim limits and relaxed OSSL settings"
                    .to_string(),
            category: "builtin".to_string(),
            template_type: SimulatorType::Sandbox,
            opensim_ini: OpenSimIniConfig {
                grid_name: "Sandbox Grid".to_string(),
                welcome_message: "Welcome to the Sandbox! Build freely!".to_string(),
                allow_anonymous_login: false,
                http_port: 9000,
                external_host_name: "localhost".to_string(),
                internal_ip: "0.0.0.0".to_string(),
                database_provider: DatabaseProvider::Sqlite,
                connection_string: "Data Source=opensim.db;Version=3".to_string(),
                physics_engine: PhysicsEngine::UbOde,
                enable_voice: true,
                enable_search: true,
                enable_currency: false,
                additional_sections: HashMap::new(),
            },
            region_ini: RegionIniConfig {
                region_name: "Sandbox".to_string(),
                region_uuid: uuid::Uuid::new_v4().to_string(),
                location_x: 1000,
                location_y: 1000,
                size_x: 256,
                size_y: 256,
                internal_port: 9000,
                max_agents: 60,
                max_prims: 100000,
                estate_name: "Sandbox Estate".to_string(),
                estate_owner: "Admin".to_string(),
            },
            ossl_config: OsslConfig {
                default_threat_level: OsslThreatLevel::High,
                allowed_functions: vec![],
                blocked_functions: vec![],
                enable_npc: true,
                enable_teleport: true,
                enable_dynamic_textures: true,
                enable_json_store: true,
            },
            config_includes: HashMap::new(),
            system_requirements: SystemRequirements {
                min_memory_mb: 2048,
                recommended_memory_mb: 4096,
                min_cpu_cores: 2,
                recommended_cpu_cores: 4,
                network_bandwidth_mbps: 100,
                disk_space_gb: 20,
                notes: Some("High-capacity sandbox for testing and development".to_string()),
            },
            container_config: None,
            thumbnail_data: None,
        }
    }

    fn welcome() -> SimulatorTemplate {
        SimulatorTemplate {
            id: "welcome".to_string(),
            name: "Welcome Area".to_string(),
            description: "New user landing zone optimized for low lag and high capacity"
                .to_string(),
            category: "builtin".to_string(),
            template_type: SimulatorType::Welcome,
            opensim_ini: OpenSimIniConfig {
                grid_name: "Welcome Center".to_string(),
                welcome_message: "Welcome! This is your starting point.".to_string(),
                allow_anonymous_login: false,
                http_port: 9000,
                external_host_name: "localhost".to_string(),
                internal_ip: "0.0.0.0".to_string(),
                database_provider: DatabaseProvider::Sqlite,
                connection_string: "Data Source=opensim.db;Version=3".to_string(),
                physics_engine: PhysicsEngine::UbOde,
                enable_voice: true,
                enable_search: true,
                enable_currency: false,
                additional_sections: HashMap::new(),
            },
            region_ini: RegionIniConfig {
                region_name: "Welcome Center".to_string(),
                region_uuid: uuid::Uuid::new_v4().to_string(),
                location_x: 1000,
                location_y: 1000,
                size_x: 256,
                size_y: 256,
                internal_port: 9000,
                max_agents: 100,
                max_prims: 20000,
                estate_name: "Welcome Estate".to_string(),
                estate_owner: "Admin".to_string(),
            },
            ossl_config: OsslConfig {
                default_threat_level: OsslThreatLevel::VeryLow,
                allowed_functions: vec![],
                blocked_functions: vec![],
                enable_npc: true,
                enable_teleport: true,
                enable_dynamic_textures: false,
                enable_json_store: true,
            },
            config_includes: HashMap::new(),
            system_requirements: SystemRequirements {
                min_memory_mb: 2048,
                recommended_memory_mb: 3072,
                min_cpu_cores: 2,
                recommended_cpu_cores: 4,
                network_bandwidth_mbps: 200,
                disk_space_gb: 10,
                notes: Some("Optimized for high user traffic and low latency".to_string()),
            },
            container_config: None,
            thumbnail_data: None,
        }
    }

    fn event() -> SimulatorTemplate {
        SimulatorTemplate {
            id: "event".to_string(),
            name: "Event Venue".to_string(),
            description: "High-capacity region for events with voice and streaming enabled"
                .to_string(),
            category: "builtin".to_string(),
            template_type: SimulatorType::Event,
            opensim_ini: OpenSimIniConfig {
                grid_name: "Event Center".to_string(),
                welcome_message: "Welcome to the Event!".to_string(),
                allow_anonymous_login: false,
                http_port: 9000,
                external_host_name: "localhost".to_string(),
                internal_ip: "0.0.0.0".to_string(),
                database_provider: DatabaseProvider::Postgresql,
                connection_string:
                    "Host=localhost;Database=opensim;Username=opensim;Password=password".to_string(),
                physics_engine: PhysicsEngine::UbOde,
                enable_voice: true,
                enable_search: true,
                enable_currency: true,
                additional_sections: HashMap::new(),
            },
            region_ini: RegionIniConfig {
                region_name: "Event Venue".to_string(),
                region_uuid: uuid::Uuid::new_v4().to_string(),
                location_x: 1000,
                location_y: 1000,
                size_x: 512,
                size_y: 512,
                internal_port: 9000,
                max_agents: 200,
                max_prims: 30000,
                estate_name: "Event Estate".to_string(),
                estate_owner: "Admin".to_string(),
            },
            ossl_config: OsslConfig {
                default_threat_level: OsslThreatLevel::Moderate,
                allowed_functions: vec![],
                blocked_functions: vec![],
                enable_npc: true,
                enable_teleport: true,
                enable_dynamic_textures: true,
                enable_json_store: true,
            },
            config_includes: HashMap::new(),
            system_requirements: SystemRequirements {
                min_memory_mb: 4096,
                recommended_memory_mb: 8192,
                min_cpu_cores: 4,
                recommended_cpu_cores: 8,
                network_bandwidth_mbps: 500,
                disk_space_gb: 30,
                notes: Some(
                    "Enterprise-grade event configuration for large gatherings".to_string(),
                ),
            },
            container_config: None,
            thumbnail_data: None,
        }
    }

    fn shopping() -> SimulatorTemplate {
        SimulatorTemplate {
            id: "shopping".to_string(),
            name: "Shopping District".to_string(),
            description: "Commercial region with economy features and vendor script support"
                .to_string(),
            category: "builtin".to_string(),
            template_type: SimulatorType::Shopping,
            opensim_ini: OpenSimIniConfig {
                grid_name: "Shopping Center".to_string(),
                welcome_message: "Welcome to the Shopping District!".to_string(),
                allow_anonymous_login: false,
                http_port: 9000,
                external_host_name: "localhost".to_string(),
                internal_ip: "0.0.0.0".to_string(),
                database_provider: DatabaseProvider::Sqlite,
                connection_string: "Data Source=opensim.db;Version=3".to_string(),
                physics_engine: PhysicsEngine::UbOde,
                enable_voice: false,
                enable_search: true,
                enable_currency: true,
                additional_sections: HashMap::new(),
            },
            region_ini: RegionIniConfig {
                region_name: "Shopping District".to_string(),
                region_uuid: uuid::Uuid::new_v4().to_string(),
                location_x: 1000,
                location_y: 1000,
                size_x: 256,
                size_y: 256,
                internal_port: 9000,
                max_agents: 50,
                max_prims: 50000,
                estate_name: "Shopping Estate".to_string(),
                estate_owner: "Admin".to_string(),
            },
            ossl_config: OsslConfig {
                default_threat_level: OsslThreatLevel::Moderate,
                allowed_functions: vec![],
                blocked_functions: vec![],
                enable_npc: false,
                enable_teleport: true,
                enable_dynamic_textures: true,
                enable_json_store: true,
            },
            config_includes: HashMap::new(),
            system_requirements: SystemRequirements {
                min_memory_mb: 1024,
                recommended_memory_mb: 2048,
                min_cpu_cores: 2,
                recommended_cpu_cores: 2,
                network_bandwidth_mbps: 100,
                disk_space_gb: 15,
                notes: Some("Economy-enabled region for commercial activities".to_string()),
            },
            container_config: None,
            thumbnail_data: None,
        }
    }

    fn roleplay() -> SimulatorTemplate {
        SimulatorTemplate {
            id: "roleplay".to_string(),
            name: "Roleplay".to_string(),
            description: "RP-focused region with combat scripts, NPC support, and custom time"
                .to_string(),
            category: "builtin".to_string(),
            template_type: SimulatorType::Roleplay,
            opensim_ini: OpenSimIniConfig {
                grid_name: "Roleplay World".to_string(),
                welcome_message: "Welcome to the adventure!".to_string(),
                allow_anonymous_login: false,
                http_port: 9000,
                external_host_name: "localhost".to_string(),
                internal_ip: "0.0.0.0".to_string(),
                database_provider: DatabaseProvider::Sqlite,
                connection_string: "Data Source=opensim.db;Version=3".to_string(),
                physics_engine: PhysicsEngine::UbOde,
                enable_voice: true,
                enable_search: true,
                enable_currency: true,
                additional_sections: HashMap::new(),
            },
            region_ini: RegionIniConfig {
                region_name: "Roleplay Region".to_string(),
                region_uuid: uuid::Uuid::new_v4().to_string(),
                location_x: 1000,
                location_y: 1000,
                size_x: 256,
                size_y: 256,
                internal_port: 9000,
                max_agents: 40,
                max_prims: 40000,
                estate_name: "Roleplay Estate".to_string(),
                estate_owner: "Admin".to_string(),
            },
            ossl_config: OsslConfig {
                default_threat_level: OsslThreatLevel::Moderate,
                allowed_functions: vec![],
                blocked_functions: vec![],
                enable_npc: true,
                enable_teleport: true,
                enable_dynamic_textures: true,
                enable_json_store: true,
            },
            config_includes: HashMap::new(),
            system_requirements: SystemRequirements {
                min_memory_mb: 1536,
                recommended_memory_mb: 2560,
                min_cpu_cores: 2,
                recommended_cpu_cores: 2,
                network_bandwidth_mbps: 75,
                disk_space_gb: 12,
                notes: Some("NPC-enabled region for immersive roleplay experiences".to_string()),
            },
            container_config: None,
            thumbnail_data: None,
        }
    }

    fn residential() -> SimulatorTemplate {
        SimulatorTemplate {
            id: "residential".to_string(),
            name: "Residential".to_string(),
            description: "Private living spaces with strict permissions and low agent limits"
                .to_string(),
            category: "builtin".to_string(),
            template_type: SimulatorType::Residential,
            opensim_ini: OpenSimIniConfig {
                grid_name: "Residential Area".to_string(),
                welcome_message: "Welcome home!".to_string(),
                allow_anonymous_login: false,
                http_port: 9000,
                external_host_name: "localhost".to_string(),
                internal_ip: "0.0.0.0".to_string(),
                database_provider: DatabaseProvider::Sqlite,
                connection_string: "Data Source=opensim.db;Version=3".to_string(),
                physics_engine: PhysicsEngine::UbOde,
                enable_voice: false,
                enable_search: false,
                enable_currency: false,
                additional_sections: HashMap::new(),
            },
            region_ini: RegionIniConfig {
                region_name: "Residential Area".to_string(),
                region_uuid: uuid::Uuid::new_v4().to_string(),
                location_x: 1000,
                location_y: 1000,
                size_x: 256,
                size_y: 256,
                internal_port: 9000,
                max_agents: 20,
                max_prims: 25000,
                estate_name: "Residential Estate".to_string(),
                estate_owner: "Admin".to_string(),
            },
            ossl_config: OsslConfig {
                default_threat_level: OsslThreatLevel::VeryLow,
                allowed_functions: vec![],
                blocked_functions: vec![],
                enable_npc: false,
                enable_teleport: false,
                enable_dynamic_textures: true,
                enable_json_store: true,
            },
            config_includes: HashMap::new(),
            system_requirements: SystemRequirements {
                min_memory_mb: 512,
                recommended_memory_mb: 1024,
                min_cpu_cores: 1,
                recommended_cpu_cores: 1,
                network_bandwidth_mbps: 25,
                disk_space_gb: 5,
                notes: Some("Lightweight configuration for private residential use".to_string()),
            },
            container_config: None,
            thumbnail_data: None,
        }
    }

    fn void_region() -> SimulatorTemplate {
        SimulatorTemplate {
            id: "void".to_string(),
            name: "Void".to_string(),
            description: "Empty starter region with minimal settings and flat terrain".to_string(),
            category: "builtin".to_string(),
            template_type: SimulatorType::Void,
            opensim_ini: OpenSimIniConfig {
                grid_name: "Void Region".to_string(),
                welcome_message: "Empty canvas awaits...".to_string(),
                allow_anonymous_login: false,
                http_port: 9000,
                external_host_name: "localhost".to_string(),
                internal_ip: "0.0.0.0".to_string(),
                database_provider: DatabaseProvider::Sqlite,
                connection_string: "Data Source=opensim.db;Version=3".to_string(),
                physics_engine: PhysicsEngine::BasicPhysics,
                enable_voice: false,
                enable_search: false,
                enable_currency: false,
                additional_sections: HashMap::new(),
            },
            region_ini: RegionIniConfig {
                region_name: "Void".to_string(),
                region_uuid: uuid::Uuid::new_v4().to_string(),
                location_x: 1000,
                location_y: 1000,
                size_x: 256,
                size_y: 256,
                internal_port: 9000,
                max_agents: 10,
                max_prims: 5000,
                estate_name: "Void Estate".to_string(),
                estate_owner: "Admin".to_string(),
            },
            ossl_config: OsslConfig {
                default_threat_level: OsslThreatLevel::None,
                allowed_functions: vec![],
                blocked_functions: vec![],
                enable_npc: false,
                enable_teleport: false,
                enable_dynamic_textures: false,
                enable_json_store: false,
            },
            config_includes: HashMap::new(),
            system_requirements: SystemRequirements {
                min_memory_mb: 256,
                recommended_memory_mb: 512,
                min_cpu_cores: 1,
                recommended_cpu_cores: 1,
                network_bandwidth_mbps: 10,
                disk_space_gb: 2,
                notes: Some("Minimal resource region for basic needs".to_string()),
            },
            container_config: None,
            thumbnail_data: None,
        }
    }

    fn custom_terrain() -> SimulatorTemplate {
        SimulatorTemplate {
            id: "custom_terrain".to_string(),
            name: "Custom Terrain".to_string(),
            description: "Region with custom heightmap support and configurable water level"
                .to_string(),
            category: "builtin".to_string(),
            template_type: SimulatorType::CustomTerrain,
            opensim_ini: OpenSimIniConfig {
                grid_name: "Custom World".to_string(),
                welcome_message: "Welcome to a unique landscape!".to_string(),
                allow_anonymous_login: false,
                http_port: 9000,
                external_host_name: "localhost".to_string(),
                internal_ip: "0.0.0.0".to_string(),
                database_provider: DatabaseProvider::Sqlite,
                connection_string: "Data Source=opensim.db;Version=3".to_string(),
                physics_engine: PhysicsEngine::UbOde,
                enable_voice: false,
                enable_search: true,
                enable_currency: false,
                additional_sections: HashMap::new(),
            },
            region_ini: RegionIniConfig {
                region_name: "Custom Terrain".to_string(),
                region_uuid: uuid::Uuid::new_v4().to_string(),
                location_x: 1000,
                location_y: 1000,
                size_x: 256,
                size_y: 256,
                internal_port: 9000,
                max_agents: 40,
                max_prims: 45000,
                estate_name: "Custom Estate".to_string(),
                estate_owner: "Admin".to_string(),
            },
            ossl_config: OsslConfig {
                default_threat_level: OsslThreatLevel::Low,
                allowed_functions: vec![],
                blocked_functions: vec![],
                enable_npc: false,
                enable_teleport: true,
                enable_dynamic_textures: true,
                enable_json_store: true,
            },
            config_includes: HashMap::new(),
            system_requirements: SystemRequirements {
                min_memory_mb: 1024,
                recommended_memory_mb: 2048,
                min_cpu_cores: 2,
                recommended_cpu_cores: 2,
                network_bandwidth_mbps: 50,
                disk_space_gb: 10,
                notes: Some(
                    "Supports custom heightmap imports and terrain configuration".to_string(),
                ),
            },
            container_config: None,
            thumbnail_data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_templates() {
        let templates = BuiltInTemplates::get_all();
        assert_eq!(templates.len(), 11);
    }

    #[test]
    fn test_get_by_type() {
        let template = BuiltInTemplates::get_by_type(SimulatorType::Event);
        assert!(template.is_some());
        assert_eq!(template.unwrap().id, "event");
    }

    #[test]
    fn test_get_by_id() {
        let template = BuiltInTemplates::get_by_id("sandbox");
        assert!(template.is_some());
        assert_eq!(template.unwrap().template_type, SimulatorType::Sandbox);
    }

    #[test]
    fn test_template_system_requirements() {
        let event = BuiltInTemplates::get_by_id("event").unwrap();
        assert_eq!(event.system_requirements.recommended_memory_mb, 8192);
        assert_eq!(event.system_requirements.recommended_cpu_cores, 8);

        let void = BuiltInTemplates::get_by_id("void").unwrap();
        assert_eq!(void.system_requirements.recommended_memory_mb, 512);
    }
}
