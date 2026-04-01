use super::models::*;
use regex::Regex;

pub struct ConfigurationValidator;

impl ConfigurationValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, config: &SavedConfiguration) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        self.validate_opensim_ini(&config.opensim_ini, &mut errors, &mut warnings);
        self.validate_region_ini(&config.region_ini, &mut errors, &mut warnings);
        self.validate_ossl_config(&config.ossl_config, &mut errors, &mut warnings);
        self.validate_system_requirements(&config.system_requirements, &mut warnings);

        if let Some(container_config) = &config.container_config {
            self.validate_container_config(container_config, &config.deployment_type, &mut errors, &mut warnings);
        }

        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    fn validate_opensim_ini(
        &self,
        config: &OpenSimIniConfig,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        if config.grid_name.trim().is_empty() {
            errors.push(ValidationError {
                field: "opensim_ini.grid_name".to_string(),
                message: "Grid name cannot be empty".to_string(),
                code: "REQUIRED_FIELD".to_string(),
            });
        }

        if config.grid_name.len() > 64 {
            warnings.push(ValidationWarning {
                field: "opensim_ini.grid_name".to_string(),
                message: "Grid name is very long and may be truncated in viewers".to_string(),
                suggestion: Some("Consider using a shorter grid name (max 64 characters)".to_string()),
            });
        }

        if config.http_port < 1024 {
            warnings.push(ValidationWarning {
                field: "opensim_ini.http_port".to_string(),
                message: "Ports below 1024 require elevated privileges".to_string(),
                suggestion: Some("Consider using a port above 1024".to_string()),
            });
        }

        if config.http_port > 65535 {
            errors.push(ValidationError {
                field: "opensim_ini.http_port".to_string(),
                message: "Port number must be between 1 and 65535".to_string(),
                code: "INVALID_PORT".to_string(),
            });
        }

        if config.external_host_name.trim().is_empty() {
            errors.push(ValidationError {
                field: "opensim_ini.external_host_name".to_string(),
                message: "External hostname cannot be empty".to_string(),
                code: "REQUIRED_FIELD".to_string(),
            });
        }

        if config.external_host_name == "localhost" || config.external_host_name == "127.0.0.1" {
            warnings.push(ValidationWarning {
                field: "opensim_ini.external_host_name".to_string(),
                message: "Using localhost will prevent external connections".to_string(),
                suggestion: Some("Use your public IP or hostname for external access".to_string()),
            });
        }

        if config.connection_string.trim().is_empty() {
            errors.push(ValidationError {
                field: "opensim_ini.connection_string".to_string(),
                message: "Database connection string cannot be empty".to_string(),
                code: "REQUIRED_FIELD".to_string(),
            });
        }

        self.validate_connection_string(
            &config.connection_string,
            &config.database_provider,
            errors,
            warnings,
        );
    }

    fn validate_connection_string(
        &self,
        connection_string: &str,
        provider: &DatabaseProvider,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        match provider {
            DatabaseProvider::Sqlite => {
                if !connection_string.contains("Data Source=") && !connection_string.contains("Filename=") {
                    errors.push(ValidationError {
                        field: "opensim_ini.connection_string".to_string(),
                        message: "SQLite connection string must include 'Data Source=' or 'Filename='".to_string(),
                        code: "INVALID_CONNECTION_STRING".to_string(),
                    });
                }
            }
            DatabaseProvider::Postgresql => {
                if !connection_string.contains("Host=") {
                    errors.push(ValidationError {
                        field: "opensim_ini.connection_string".to_string(),
                        message: "PostgreSQL connection string must include 'Host='".to_string(),
                        code: "INVALID_CONNECTION_STRING".to_string(),
                    });
                }
                if !connection_string.contains("Database=") {
                    warnings.push(ValidationWarning {
                        field: "opensim_ini.connection_string".to_string(),
                        message: "PostgreSQL connection string should include 'Database='".to_string(),
                        suggestion: Some("Add 'Database=opensim' to connection string".to_string()),
                    });
                }
            }
            DatabaseProvider::Mysql | DatabaseProvider::Mariadb => {
                if !connection_string.contains("Server=") && !connection_string.contains("Host=") {
                    errors.push(ValidationError {
                        field: "opensim_ini.connection_string".to_string(),
                        message: "MySQL/MariaDB connection string must include 'Server=' or 'Host='".to_string(),
                        code: "INVALID_CONNECTION_STRING".to_string(),
                    });
                }
            }
        }

        if connection_string.contains("password=password") || connection_string.contains("Pwd=password") {
            warnings.push(ValidationWarning {
                field: "opensim_ini.connection_string".to_string(),
                message: "Default password detected in connection string".to_string(),
                suggestion: Some("Change the database password before deployment".to_string()),
            });
        }
    }

    fn validate_region_ini(
        &self,
        config: &RegionIniConfig,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        if config.region_name.trim().is_empty() {
            errors.push(ValidationError {
                field: "region_ini.region_name".to_string(),
                message: "Region name cannot be empty".to_string(),
                code: "REQUIRED_FIELD".to_string(),
            });
        }

        if config.region_name.len() > 64 {
            errors.push(ValidationError {
                field: "region_ini.region_name".to_string(),
                message: "Region name cannot exceed 64 characters".to_string(),
                code: "MAX_LENGTH_EXCEEDED".to_string(),
            });
        }

        let uuid_regex = Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$").unwrap();
        if !uuid_regex.is_match(&config.region_uuid) {
            errors.push(ValidationError {
                field: "region_ini.region_uuid".to_string(),
                message: "Region UUID must be a valid UUID format".to_string(),
                code: "INVALID_UUID".to_string(),
            });
        }

        if config.location_x < 0 || config.location_y < 0 {
            errors.push(ValidationError {
                field: "region_ini.location".to_string(),
                message: "Region location coordinates must be non-negative".to_string(),
                code: "INVALID_LOCATION".to_string(),
            });
        }

        let valid_sizes = [256, 512, 768, 1024];
        if !valid_sizes.contains(&config.size_x) {
            warnings.push(ValidationWarning {
                field: "region_ini.size_x".to_string(),
                message: format!("Non-standard region size {}. Standard sizes are: 256, 512, 768, 1024", config.size_x),
                suggestion: Some("Use standard sizes for best compatibility".to_string()),
            });
        }

        if !valid_sizes.contains(&config.size_y) {
            warnings.push(ValidationWarning {
                field: "region_ini.size_y".to_string(),
                message: format!("Non-standard region size {}. Standard sizes are: 256, 512, 768, 1024", config.size_y),
                suggestion: Some("Use standard sizes for best compatibility".to_string()),
            });
        }

        if config.size_x > 1024 || config.size_y > 1024 {
            warnings.push(ValidationWarning {
                field: "region_ini.size".to_string(),
                message: "Very large regions may have performance issues".to_string(),
                suggestion: Some("Consider using smaller regions or a var-region approach".to_string()),
            });
        }

        if config.internal_port < 1024 {
            warnings.push(ValidationWarning {
                field: "region_ini.internal_port".to_string(),
                message: "Ports below 1024 require elevated privileges".to_string(),
                suggestion: Some("Consider using a port above 1024".to_string()),
            });
        }

        if config.max_agents < 1 {
            errors.push(ValidationError {
                field: "region_ini.max_agents".to_string(),
                message: "Max agents must be at least 1".to_string(),
                code: "INVALID_VALUE".to_string(),
            });
        }

        if config.max_agents > 1000 {
            warnings.push(ValidationWarning {
                field: "region_ini.max_agents".to_string(),
                message: "Very high max agents may cause performance issues".to_string(),
                suggestion: Some("Consider limiting to 200 agents or less for stability".to_string()),
            });
        }

        if config.max_prims < 100 {
            warnings.push(ValidationWarning {
                field: "region_ini.max_prims".to_string(),
                message: "Very low prim limit may restrict content creation".to_string(),
                suggestion: Some("Consider allowing at least 5000 prims".to_string()),
            });
        }

        if config.max_prims > 500000 {
            warnings.push(ValidationWarning {
                field: "region_ini.max_prims".to_string(),
                message: "Very high prim limit may cause memory issues".to_string(),
                suggestion: Some("Consider limiting to 100000 prims or less".to_string()),
            });
        }
    }

    fn validate_ossl_config(
        &self,
        config: &OsslConfig,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        if config.default_threat_level == OsslThreatLevel::Severe {
            warnings.push(ValidationWarning {
                field: "ossl_config.default_threat_level".to_string(),
                message: "Severe threat level enables potentially dangerous functions".to_string(),
                suggestion: Some("Only use for trusted development environments".to_string()),
            });
        }

        if config.default_threat_level == OsslThreatLevel::VeryHigh {
            warnings.push(ValidationWarning {
                field: "ossl_config.default_threat_level".to_string(),
                message: "Very High threat level enables powerful functions".to_string(),
                suggestion: Some("Ensure only trusted users have script permissions".to_string()),
            });
        }

        for func in &config.allowed_functions {
            if config.blocked_functions.contains(func) {
                errors.push(ValidationError {
                    field: "ossl_config.functions".to_string(),
                    message: format!("Function '{}' cannot be both allowed and blocked", func),
                    code: "CONFLICTING_PERMISSIONS".to_string(),
                });
            }
        }
    }

    fn validate_system_requirements(
        &self,
        requirements: &SystemRequirements,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        if requirements.recommended_memory_mb < requirements.min_memory_mb {
            warnings.push(ValidationWarning {
                field: "system_requirements.memory".to_string(),
                message: "Recommended memory is less than minimum memory".to_string(),
                suggestion: Some("Adjust memory requirements to be consistent".to_string()),
            });
        }

        if requirements.recommended_cpu_cores < requirements.min_cpu_cores {
            warnings.push(ValidationWarning {
                field: "system_requirements.cpu".to_string(),
                message: "Recommended CPU cores is less than minimum".to_string(),
                suggestion: Some("Adjust CPU requirements to be consistent".to_string()),
            });
        }
    }

    fn validate_container_config(
        &self,
        config: &ContainerConfig,
        deployment_type: &DeploymentType,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        if *deployment_type == DeploymentType::Docker || *deployment_type == DeploymentType::Kubernetes {
            if config.memory_limit_mb < 256 {
                errors.push(ValidationError {
                    field: "container_config.memory_limit_mb".to_string(),
                    message: "Container memory limit too low. Minimum 256MB required".to_string(),
                    code: "INSUFFICIENT_RESOURCES".to_string(),
                });
            }

            if config.cpu_limit < 0.25 {
                errors.push(ValidationError {
                    field: "container_config.cpu_limit".to_string(),
                    message: "Container CPU limit too low. Minimum 0.25 cores required".to_string(),
                    code: "INSUFFICIENT_RESOURCES".to_string(),
                });
            }
        }

        if *deployment_type == DeploymentType::Kubernetes {
            if config.namespace.is_none() || config.namespace.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
                warnings.push(ValidationWarning {
                    field: "container_config.namespace".to_string(),
                    message: "No namespace specified, will use 'opensim' default".to_string(),
                    suggestion: Some("Consider specifying a namespace for better organization".to_string()),
                });
            }

            if config.enable_hpa && config.max_replicas <= config.min_replicas {
                errors.push(ValidationError {
                    field: "container_config.replicas".to_string(),
                    message: "HPA max replicas must be greater than min replicas".to_string(),
                    code: "INVALID_HPA_CONFIG".to_string(),
                });
            }

            if config.replicas > 10 {
                warnings.push(ValidationWarning {
                    field: "container_config.replicas".to_string(),
                    message: "High replica count may cause resource contention".to_string(),
                    suggestion: Some("OpenSim regions typically run better with fewer replicas".to_string()),
                });
            }
        }
    }
}

impl Default for ConfigurationValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> SavedConfiguration {
        SavedConfiguration {
            id: "test".to_string(),
            name: "Test Config".to_string(),
            description: None,
            based_on_template: None,
            opensim_ini: OpenSimIniConfig::default(),
            region_ini: RegionIniConfig::default(),
            ossl_config: OsslConfig::default(),
            config_includes: std::collections::HashMap::new(),
            system_requirements: SystemRequirements::default(),
            deployment_type: DeploymentType::Native,
            container_config: None,
            deployment_status: DeploymentStatus::Draft,
            deployed_instance_id: None,
            deployed_path: None,
            tags: vec![],
            created_at: None,
            updated_at: None,
            last_deployed_at: None,
        }
    }

    #[test]
    fn test_validate_valid_config() {
        let validator = ConfigurationValidator::new();
        let config = create_test_config();
        let result = validator.validate(&config);

        assert!(result.valid || result.warnings.len() > 0);
    }

    #[test]
    fn test_validate_empty_grid_name() {
        let validator = ConfigurationValidator::new();
        let mut config = create_test_config();
        config.opensim_ini.grid_name = "".to_string();

        let result = validator.validate(&config);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.field.contains("grid_name")));
    }

    #[test]
    fn test_validate_invalid_uuid() {
        let validator = ConfigurationValidator::new();
        let mut config = create_test_config();
        config.region_ini.region_uuid = "not-a-uuid".to_string();

        let result = validator.validate(&config);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.field.contains("region_uuid")));
    }

    #[test]
    fn test_validate_conflicting_ossl_functions() {
        let validator = ConfigurationValidator::new();
        let mut config = create_test_config();
        config.ossl_config.allowed_functions = vec!["osTest".to_string()];
        config.ossl_config.blocked_functions = vec!["osTest".to_string()];

        let result = validator.validate(&config);
        assert!(!result.valid);
    }
}
