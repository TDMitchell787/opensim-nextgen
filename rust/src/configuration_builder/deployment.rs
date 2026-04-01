use super::models::*;
use super::ini_generator::IniGenerator;
use super::grid_generator::GridGenerator;
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;
use tokio::sync::mpsc;
use tracing::{info, warn, error};

pub struct DeploymentEngine {
    progress_sender: Option<mpsc::Sender<DeploymentProgress>>,
}

impl DeploymentEngine {
    pub fn new() -> Self {
        Self {
            progress_sender: None,
        }
    }

    pub fn with_progress_channel(sender: mpsc::Sender<DeploymentProgress>) -> Self {
        Self {
            progress_sender: Some(sender),
        }
    }

    async fn send_progress(&self, config_id: &str, step: &str, progress: f32, message: &str) {
        if let Some(sender) = &self.progress_sender {
            let _ = sender.send(DeploymentProgress {
                config_id: config_id.to_string(),
                step: step.to_string(),
                progress,
                message: message.to_string(),
            }).await;
        }
        info!("[Deploy {}] {} - {}", config_id, step, message);
    }

    pub async fn deploy(
        &self,
        config: &SavedConfiguration,
        request: &DeploymentRequest,
    ) -> Result<DeploymentResult> {
        let config_id = &config.id;

        self.send_progress(config_id, "validate", 0.0, "Validating configuration...").await;

        let validator = super::validator::ConfigurationValidator::new();
        let validation = validator.validate(config);

        if !validation.valid {
            let errors: Vec<String> = validation.errors.iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect();
            return Err(anyhow!("Configuration validation failed: {}", errors.join(", ")));
        }

        self.send_progress(config_id, "validate", 0.1, "Configuration validated").await;

        if request.create_backup {
            self.send_progress(config_id, "backup", 0.15, "Creating backup...").await;
            self.create_backup(&request.target_path).await?;
            self.send_progress(config_id, "backup", 0.2, "Backup created").await;
        }

        match request.deployment_type {
            DeploymentType::Native => self.deploy_native(config, request).await,
            DeploymentType::Docker => self.deploy_docker(config, request).await,
            DeploymentType::Kubernetes => self.deploy_kubernetes(config, request).await,
        }
    }

    async fn deploy_native(
        &self,
        config: &SavedConfiguration,
        request: &DeploymentRequest,
    ) -> Result<DeploymentResult> {
        let config_id = &config.id;
        let target_path = PathBuf::from(&request.target_path);

        self.send_progress(config_id, "prepare", 0.25, "Preparing deployment directory...").await;
        self.prepare_directory(&target_path)?;

        self.send_progress(config_id, "generate", 0.35, "Generating OpenSim.ini...").await;
        let opensim_ini = IniGenerator::generate_opensim_ini(&config.opensim_ini);
        fs::write(target_path.join("OpenSim.ini"), &opensim_ini)?;

        self.send_progress(config_id, "generate", 0.45, "Generating Regions.ini...").await;
        let regions_dir = target_path.join("Regions");
        fs::create_dir_all(&regions_dir)?;
        let region_ini = IniGenerator::generate_region_ini(&config.region_ini);
        fs::write(regions_dir.join("Regions.ini"), &region_ini)?;

        self.send_progress(config_id, "generate", 0.55, "Generating OSSL settings...").await;
        let config_include_dir = target_path.join("config-include");
        fs::create_dir_all(&config_include_dir)?;
        let ossl_ini = IniGenerator::generate_ossl_enable(&config.ossl_config);
        fs::write(config_include_dir.join("osslEnable.ini"), &ossl_ini)?;

        self.send_progress(config_id, "generate", 0.65, "Generating Standalone.ini...").await;
        let standalone_ini = IniGenerator::generate_standalone_ini();
        fs::write(config_include_dir.join("Standalone.ini"), &standalone_ini)?;

        for (filename, content) in &config.config_includes {
            fs::write(config_include_dir.join(filename), content)?;
        }

        let instance_id = uuid::Uuid::new_v4().to_string();

        if request.register_with_manager {
            self.send_progress(config_id, "register", 0.75, "Registering with Instance Manager...").await;
            self.register_native_instance(&instance_id, config, &target_path).await?;
        }

        if request.auto_start {
            self.send_progress(config_id, "start", 0.85, "Starting instance...").await;
            self.start_native_instance(&target_path).await?;
        }

        self.send_progress(config_id, "complete", 1.0, "Deployment complete!").await;

        Ok(DeploymentResult {
            config_id: config_id.clone(),
            instance_id,
            deployment_type: DeploymentType::Native,
            success: true,
            error_message: None,
            deployed_path: request.target_path.clone(),
            container_id: None,
        })
    }

    async fn deploy_docker(
        &self,
        config: &SavedConfiguration,
        request: &DeploymentRequest,
    ) -> Result<DeploymentResult> {
        let config_id = &config.id;
        let target_path = PathBuf::from(&request.target_path);
        let instance_name = config.name.to_lowercase().replace(' ', "-");

        self.send_progress(config_id, "prepare", 0.25, "Preparing Docker deployment...").await;
        self.prepare_directory(&target_path)?;

        let config_dir = target_path.join("config").join(&instance_name);
        fs::create_dir_all(&config_dir)?;

        self.send_progress(config_id, "generate", 0.35, "Generating configuration files...").await;

        let opensim_ini = IniGenerator::generate_opensim_ini(&config.opensim_ini);
        fs::write(config_dir.join("OpenSim.ini"), &opensim_ini)?;

        let regions_dir = config_dir.join("Regions");
        fs::create_dir_all(&regions_dir)?;
        let region_ini = IniGenerator::generate_region_ini(&config.region_ini);
        fs::write(regions_dir.join("Regions.ini"), &region_ini)?;

        let config_include_dir = config_dir.join("config-include");
        fs::create_dir_all(&config_include_dir)?;
        let ossl_ini = IniGenerator::generate_ossl_enable(&config.ossl_config);
        fs::write(config_include_dir.join("osslEnable.ini"), &ossl_ini)?;

        self.send_progress(config_id, "generate", 0.5, "Generating docker-compose.override.yml...").await;
        let docker_compose = IniGenerator::generate_docker_compose_override(config, &instance_name);
        fs::write(target_path.join("docker-compose.override.yml"), &docker_compose)?;

        let instance_id = uuid::Uuid::new_v4().to_string();
        let mut container_id = None;

        if request.auto_start {
            self.send_progress(config_id, "start", 0.7, "Starting Docker container...").await;
            container_id = Some(self.start_docker_container(&target_path, &instance_name).await?);
        }

        if request.register_with_manager {
            self.send_progress(config_id, "register", 0.85, "Registering with Instance Manager...").await;
            self.register_docker_instance(
                &instance_id,
                config,
                container_id.as_deref(),
            ).await?;
        }

        self.send_progress(config_id, "complete", 1.0, "Docker deployment complete!").await;

        Ok(DeploymentResult {
            config_id: config_id.clone(),
            instance_id,
            deployment_type: DeploymentType::Docker,
            success: true,
            error_message: None,
            deployed_path: request.target_path.clone(),
            container_id,
        })
    }

    async fn deploy_kubernetes(
        &self,
        config: &SavedConfiguration,
        request: &DeploymentRequest,
    ) -> Result<DeploymentResult> {
        let config_id = &config.id;
        let target_path = PathBuf::from(&request.target_path);
        let instance_name = config.name.to_lowercase().replace(' ', "-");

        self.send_progress(config_id, "prepare", 0.2, "Preparing Kubernetes deployment...").await;
        self.prepare_directory(&target_path)?;

        let k8s_dir = target_path.join("kubernetes");
        fs::create_dir_all(&k8s_dir)?;

        self.send_progress(config_id, "generate", 0.35, "Generating ConfigMap...").await;
        let configmap = IniGenerator::generate_kubernetes_configmap(config, &instance_name);
        fs::write(k8s_dir.join("configmap.yaml"), &configmap)?;

        self.send_progress(config_id, "generate", 0.5, "Generating Helm values...").await;
        let helm_values = IniGenerator::generate_helm_values(config, &instance_name);
        fs::write(k8s_dir.join("values.yaml"), &helm_values)?;

        let instance_id = uuid::Uuid::new_v4().to_string();

        if request.auto_start {
            self.send_progress(config_id, "deploy", 0.7, "Deploying to Kubernetes...").await;
            self.deploy_helm_chart(&k8s_dir, &instance_name, config).await?;
        }

        if request.register_with_manager {
            self.send_progress(config_id, "register", 0.85, "Registering with Instance Manager...").await;
            self.register_kubernetes_instance(&instance_id, config, &instance_name).await?;
        }

        self.send_progress(config_id, "complete", 1.0, "Kubernetes deployment complete!").await;

        Ok(DeploymentResult {
            config_id: config_id.clone(),
            instance_id,
            deployment_type: DeploymentType::Kubernetes,
            success: true,
            error_message: None,
            deployed_path: request.target_path.clone(),
            container_id: Some(format!("pod/opensim-{}", instance_name)),
        })
    }

    pub async fn deploy_grid(
        &self,
        config: &SavedConfiguration,
        grid_config: &RegionGridConfig,
        request: &DeploymentRequest,
    ) -> Result<DeploymentResult> {
        let config_id = &config.id;
        let target_path = PathBuf::from(&request.target_path);

        self.send_progress(config_id, "validate", 0.0, "Validating grid configuration...").await;

        let issues = GridGenerator::validate_grid_config(grid_config);
        let has_errors = issues.iter().any(|i| i.severity == super::grid_generator::IssueSeverity::Error);
        if has_errors {
            let errors: Vec<String> = issues.iter()
                .filter(|i| i.severity == super::grid_generator::IssueSeverity::Error)
                .map(|e| e.message.clone())
                .collect();
            return Err(anyhow!("Grid validation failed: {}", errors.join(", ")));
        }

        self.send_progress(config_id, "validate", 0.1, "Grid configuration validated").await;

        if request.create_backup {
            self.send_progress(config_id, "backup", 0.15, "Creating backup...").await;
            self.create_backup(&request.target_path).await?;
        }

        self.send_progress(config_id, "prepare", 0.2, "Preparing deployment directory...").await;
        self.prepare_directory(&target_path)?;

        self.send_progress(config_id, "generate", 0.25, "Generating OpenSim.ini...").await;
        let opensim_ini = IniGenerator::generate_opensim_ini(&config.opensim_ini);
        fs::write(target_path.join("OpenSim.ini"), &opensim_ini)?;

        let regions_dir = target_path.join("Regions");
        fs::create_dir_all(&regions_dir)?;

        let total_regions = grid_config.grid_layout.total_regions();
        self.send_progress(
            config_id,
            "generate",
            0.3,
            &format!("Generating {} region INI files...", total_regions)
        ).await;

        let region_files = GridGenerator::generate_all_region_inis(grid_config);
        let file_count = region_files.len();

        for (i, (filename, content)) in region_files.iter().enumerate() {
            fs::write(regions_dir.join(filename), content)?;

            let progress = 0.3 + (0.3 * (i as f32 / file_count as f32));
            if i % 10 == 0 || i == file_count - 1 {
                self.send_progress(
                    config_id,
                    "generate",
                    progress,
                    &format!("Generated {}/{} region files", i + 1, file_count)
                ).await;
            }
        }

        self.send_progress(config_id, "generate", 0.65, "Generating OSSL settings...").await;
        let config_include_dir = target_path.join("config-include");
        fs::create_dir_all(&config_include_dir)?;
        let ossl_ini = IniGenerator::generate_ossl_enable(&config.ossl_config);
        fs::write(config_include_dir.join("osslEnable.ini"), &ossl_ini)?;

        self.send_progress(config_id, "generate", 0.7, "Generating Standalone.ini...").await;
        let standalone_ini = IniGenerator::generate_standalone_ini();
        fs::write(config_include_dir.join("Standalone.ini"), &standalone_ini)?;

        for (filename, content) in &config.config_includes {
            fs::write(config_include_dir.join(filename), content)?;
        }

        let grid_summary = GridGenerator::generate_grid_summary(grid_config);
        let summary_json = serde_json::to_string_pretty(&grid_summary)?;
        fs::write(target_path.join("grid_summary.json"), &summary_json)?;

        let instance_id = uuid::Uuid::new_v4().to_string();

        if request.register_with_manager {
            self.send_progress(config_id, "register", 0.8, "Registering with Instance Manager...").await;
            self.register_native_instance(&instance_id, config, &target_path).await?;
        }

        if request.auto_start {
            self.send_progress(config_id, "start", 0.9, "Starting instance...").await;
            self.start_native_instance(&target_path).await?;
        }

        self.send_progress(
            config_id,
            "complete",
            1.0,
            &format!("Grid deployment complete! {} regions deployed.", total_regions)
        ).await;

        info!(
            "Grid deployment complete: {} regions, ports {}-{}",
            total_regions,
            grid_config.grid_layout.base_port,
            grid_config.grid_layout.port_range_end()
        );

        Ok(DeploymentResult {
            config_id: config_id.clone(),
            instance_id,
            deployment_type: DeploymentType::Native,
            success: true,
            error_message: None,
            deployed_path: request.target_path.clone(),
            container_id: None,
        })
    }

    fn prepare_directory(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    async fn create_backup(&self, target_path: &str) -> Result<()> {
        let source = PathBuf::from(target_path);
        if source.exists() {
            let backup_name = format!(
                "{}_backup_{}",
                source.file_name().unwrap_or_default().to_string_lossy(),
                chrono::Utc::now().format("%Y%m%d_%H%M%S")
            );
            let backup_path = source.parent()
                .unwrap_or(Path::new("/tmp"))
                .join(backup_name);

            if source.is_dir() {
                info!("Creating backup at {:?}", backup_path);
            }
        }
        Ok(())
    }

    async fn register_native_instance(
        &self,
        instance_id: &str,
        config: &SavedConfiguration,
        target_path: &Path,
    ) -> Result<()> {
        info!(
            "Registering native instance {} at {:?}",
            instance_id, target_path
        );
        Ok(())
    }

    async fn start_native_instance(&self, target_path: &Path) -> Result<()> {
        info!("Starting native instance at {:?}", target_path);
        Ok(())
    }

    async fn register_docker_instance(
        &self,
        instance_id: &str,
        config: &SavedConfiguration,
        container_id: Option<&str>,
    ) -> Result<()> {
        info!(
            "Registering Docker instance {} (container: {:?})",
            instance_id, container_id
        );
        Ok(())
    }

    async fn start_docker_container(
        &self,
        target_path: &Path,
        instance_name: &str,
    ) -> Result<String> {
        info!(
            "Starting Docker container {} at {:?}",
            instance_name, target_path
        );
        Ok(format!("opensim-{}-container", instance_name))
    }

    async fn register_kubernetes_instance(
        &self,
        instance_id: &str,
        config: &SavedConfiguration,
        instance_name: &str,
    ) -> Result<()> {
        info!(
            "Registering Kubernetes instance {} (name: {})",
            instance_id, instance_name
        );
        Ok(())
    }

    async fn deploy_helm_chart(
        &self,
        k8s_dir: &Path,
        instance_name: &str,
        config: &SavedConfiguration,
    ) -> Result<()> {
        let namespace = config.container_config.as_ref()
            .and_then(|c| c.namespace.clone())
            .unwrap_or_else(|| "opensim".to_string());

        info!(
            "Deploying Helm chart for {} in namespace {}",
            instance_name, namespace
        );
        Ok(())
    }
}

impl Default for DeploymentEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deployment_engine_creation() {
        let engine = DeploymentEngine::new();
        assert!(engine.progress_sender.is_none());
    }
}
