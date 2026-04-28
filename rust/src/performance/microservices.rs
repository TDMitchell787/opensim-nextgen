//! Microservices architecture support for distributed OpenSim deployment

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Custom error type for circuit breaker compatibility
/// Implements std::error::Error while providing semantic error categories
#[derive(Debug)]
pub enum ServiceError {
    Request(String),
    Network(String),
    Internal(String),
    CircuitBreaker(String),
    Authentication(String),
    Timeout(String),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::Request(msg) => write!(f, "Request error: {}", msg),
            ServiceError::Network(msg) => write!(f, "Network error: {}", msg),
            ServiceError::Internal(msg) => write!(f, "Internal error: {}", msg),
            ServiceError::CircuitBreaker(msg) => write!(f, "Circuit breaker error: {}", msg),
            ServiceError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            ServiceError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}

impl From<anyhow::Error> for ServiceError {
    fn from(err: anyhow::Error) -> Self {
        // Try to categorize the error based on its message for better circuit breaker logic
        let error_msg = err.to_string().to_lowercase();

        if error_msg.contains("timeout") || error_msg.contains("deadline") {
            ServiceError::Timeout(err.to_string())
        } else if error_msg.contains("network") || error_msg.contains("connection") {
            ServiceError::Network(err.to_string())
        } else if error_msg.contains("auth") || error_msg.contains("unauthorized") {
            ServiceError::Authentication(err.to_string())
        } else if error_msg.contains("request") || error_msg.contains("invalid") {
            ServiceError::Request(err.to_string())
        } else {
            ServiceError::Internal(err.to_string())
        }
    }
}

/// Service types in the OpenSim microservices architecture
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub enum ServiceType {
    Gateway,           // API Gateway and load balancer
    RegionServer,      // Region simulation service
    AssetService,      // Asset storage and management
    UserService,       // User accounts and authentication
    InventoryService,  // User inventory management
    GridService,       // Grid-wide coordination
    MessagingService,  // Chat and messaging
    DatabaseService,   // Database access layer
    CacheService,      // Distributed caching
    MonitoringService, // Metrics and monitoring
}

impl fmt::Display for ServiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceType::Gateway => write!(f, "Gateway"),
            ServiceType::RegionServer => write!(f, "RegionServer"),
            ServiceType::AssetService => write!(f, "AssetService"),
            ServiceType::UserService => write!(f, "UserService"),
            ServiceType::InventoryService => write!(f, "InventoryService"),
            ServiceType::GridService => write!(f, "GridService"),
            ServiceType::MessagingService => write!(f, "MessagingService"),
            ServiceType::DatabaseService => write!(f, "DatabaseService"),
            ServiceType::CacheService => write!(f, "CacheService"),
            ServiceType::MonitoringService => write!(f, "MonitoringService"),
        }
    }
}

/// Service instance health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Starting,
    Stopping,
    Unknown,
}

/// Service instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub id: String,
    pub service_type: ServiceType,
    pub version: String,
    pub address: String,
    pub port: u16,
    pub health_status: ServiceHealth,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub dependencies: Vec<ServiceType>,
    pub resource_requirements: ResourceRequirements,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

/// Resource requirements for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: f32,
    pub memory_mb: u32,
    pub disk_mb: u32,
    pub network_mbps: u32,
}

/// Service discovery interface
#[async_trait::async_trait]
pub trait ServiceDiscovery: Send + Sync {
    async fn register_service(&self, instance: ServiceInstance) -> Result<()>;
    async fn unregister_service(&self, service_id: &str) -> Result<()>;
    async fn discover_services(&self, service_type: ServiceType) -> Result<Vec<ServiceInstance>>;
    async fn get_service(&self, service_id: &str) -> Result<Option<ServiceInstance>>;
    async fn update_health(&self, service_id: &str, health: ServiceHealth) -> Result<()>;
    async fn get_healthy_services(&self, service_type: ServiceType)
        -> Result<Vec<ServiceInstance>>;
}

/// In-memory service registry implementation
pub struct InMemoryServiceRegistry {
    services: Arc<RwLock<HashMap<String, ServiceInstance>>>,
    service_types: Arc<RwLock<HashMap<ServiceType, Vec<String>>>>,
}

impl InMemoryServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            service_types: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start health monitoring for registered services
    pub async fn start_health_monitoring(&self) -> Result<()> {
        let services = self.services.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                let mut services_guard = services.write().await;
                let now = chrono::Utc::now();

                for (service_id, service) in services_guard.iter_mut() {
                    let last_heartbeat_age = now.signed_duration_since(service.last_heartbeat);

                    if last_heartbeat_age > chrono::Duration::seconds(60) {
                        if service.health_status == ServiceHealth::Healthy {
                            service.health_status = ServiceHealth::Degraded;
                            warn!(
                                "Service {} marked as degraded due to missing heartbeat",
                                service_id
                            );
                        }
                    }

                    if last_heartbeat_age > chrono::Duration::seconds(120) {
                        service.health_status = ServiceHealth::Unhealthy;
                        warn!(
                            "Service {} marked as unhealthy due to missing heartbeat",
                            service_id
                        );
                    }
                }
            }
        });

        Ok(())
    }

    /// Record a heartbeat for a service
    pub async fn heartbeat(&self, service_id: &str) -> Result<()> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            service.last_heartbeat = chrono::Utc::now();
            if service.health_status == ServiceHealth::Degraded {
                service.health_status = ServiceHealth::Healthy;
                info!("Service {} restored to healthy status", service_id);
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl ServiceDiscovery for InMemoryServiceRegistry {
    async fn register_service(&self, instance: ServiceInstance) -> Result<()> {
        info!(
            "Registering service: {} ({})",
            instance.id, instance.service_type
        );

        let service_id = instance.id.clone();
        let service_type = instance.service_type.clone();

        // Add to services map
        self.services
            .write()
            .await
            .insert(service_id.clone(), instance);

        // Add to service type index
        let mut service_types = self.service_types.write().await;
        service_types
            .entry(service_type)
            .or_default()
            .push(service_id);

        Ok(())
    }

    async fn unregister_service(&self, service_id: &str) -> Result<()> {
        info!("Unregistering service: {}", service_id);

        let mut services = self.services.write().await;
        let mut service_types = self.service_types.write().await;

        if let Some(service) = services.remove(service_id) {
            // Remove from service type index
            if let Some(type_list) = service_types.get_mut(&service.service_type) {
                type_list.retain(|id| id != service_id);
            }
        }

        Ok(())
    }

    async fn discover_services(&self, service_type: ServiceType) -> Result<Vec<ServiceInstance>> {
        let services = self.services.read().await;
        let service_types = self.service_types.read().await;

        let service_ids = service_types
            .get(&service_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let mut instances = Vec::new();

        for service_id in service_ids {
            if let Some(instance) = services.get(service_id) {
                instances.push(instance.clone());
            }
        }

        Ok(instances)
    }

    async fn get_service(&self, service_id: &str) -> Result<Option<ServiceInstance>> {
        let services = self.services.read().await;
        Ok(services.get(service_id).cloned())
    }

    async fn update_health(&self, service_id: &str, health: ServiceHealth) -> Result<()> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            service.health_status = health;
            service.last_heartbeat = chrono::Utc::now();
        }
        Ok(())
    }

    async fn get_healthy_services(
        &self,
        service_type: ServiceType,
    ) -> Result<Vec<ServiceInstance>> {
        let all_services = self.discover_services(service_type).await?;
        Ok(all_services
            .into_iter()
            .filter(|s| s.health_status == ServiceHealth::Healthy)
            .collect())
    }
}

/// Service communication client for inter-service calls
pub struct ServiceClient {
    service_discovery: Arc<dyn ServiceDiscovery>,
    client: reqwest::Client,
    timeout: Duration,
}

impl ServiceClient {
    pub fn new(service_discovery: Arc<dyn ServiceDiscovery>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            service_discovery,
            client,
            timeout: Duration::from_secs(30),
        }
    }

    /// Make a GET request to a service
    pub async fn get<T>(&self, service_type: ServiceType, path: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let service = self.find_healthy_service(service_type).await?;
        let url = format!("http://{}:{}{}", service.address, service.port, path);

        debug!("Making GET request to: {}", url);

        let response = self
            .client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

        if response.status().is_success() {
            let data = response
                .json::<T>()
                .await
                .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
            Ok(data)
        } else {
            Err(anyhow!(
                "HTTP request failed with status: {}",
                response.status()
            ))
        }
    }

    /// Make a POST request to a service
    pub async fn post<T, U>(&self, service_type: ServiceType, path: &str, body: &T) -> Result<U>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        let service = self.find_healthy_service(service_type).await?;
        let url = format!("http://{}:{}{}", service.address, service.port, path);

        debug!("Making POST request to: {}", url);

        let response = self
            .client
            .post(&url)
            .json(body)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

        if response.status().is_success() {
            let data = response
                .json::<U>()
                .await
                .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
            Ok(data)
        } else {
            Err(anyhow!(
                "HTTP request failed with status: {}",
                response.status()
            ))
        }
    }

    async fn find_healthy_service(&self, service_type: ServiceType) -> Result<ServiceInstance> {
        let services = self
            .service_discovery
            .get_healthy_services(service_type.clone())
            .await?;

        if services.is_empty() {
            return Err(anyhow!(
                "No healthy services found for type: {:?}",
                service_type
            ));
        }

        // Simple round-robin selection
        let index = rand::random::<usize>() % services.len();
        Ok(services[index].clone())
    }
}

/// Circuit breaker for service calls
#[derive(Clone)]
pub struct CircuitBreaker {
    failure_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: Arc<RwLock<CircuitBreakerState>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Failing, reject calls
    HalfOpen, // Testing recovery
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            failure_threshold,
            recovery_timeout,
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
        }
    }

    /// Execute a function with circuit breaker protection
    /// Execute a future with circuit breaker protection
    /// This version is specifically designed for ServiceError
    pub async fn execute<F, T>(&self, f: F) -> Result<T, ServiceError>
    where
        F: futures::Future<Output = Result<T, ServiceError>>,
    {
        // Check if circuit breaker should allow the call
        if !self.should_allow_call().await {
            return Err(ServiceError::CircuitBreaker(
                "Circuit breaker is open".to_string(),
            ));
        }

        match f.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(error)
            }
        }
    }

    /// Generic execute method for backwards compatibility
    /// Note: This is less elegant but needed for generic error types
    pub async fn execute_generic<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: futures::Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static + From<ServiceError>,
    {
        // Check if circuit breaker should allow the call
        if !self.should_allow_call().await {
            return Err(E::from(ServiceError::CircuitBreaker(
                "Circuit breaker is open".to_string(),
            )));
        }

        match f.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(error)
            }
        }
    }

    async fn should_allow_call(&self) -> bool {
        let state = self.state.read().await.clone();

        match state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() >= self.recovery_timeout {
                        *self.state.write().await = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    async fn on_success(&self) {
        let mut state = self.state.write().await;

        match *state {
            CircuitBreakerState::HalfOpen => {
                *state = CircuitBreakerState::Closed;
                *self.failure_count.write().await = 0;
                info!("Circuit breaker closed after successful call");
            }
            _ => {}
        }
    }

    async fn on_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        let mut state = self.state.write().await;

        *failure_count += 1;
        *self.last_failure_time.write().await = Some(Instant::now());

        if *failure_count >= self.failure_threshold {
            *state = CircuitBreakerState::Open;
            warn!("Circuit breaker opened after {} failures", failure_count);
        }
    }
}

/// Service mesh coordinator for managing microservices
pub struct ServiceMesh {
    service_discovery: Arc<dyn ServiceDiscovery>,
    service_client: ServiceClient,
    circuit_breakers: Arc<RwLock<HashMap<ServiceType, CircuitBreaker>>>,
    load_balancers: Arc<RwLock<HashMap<ServiceType, Box<dyn LoadBalancer>>>>,
}

impl ServiceMesh {
    pub fn new(service_discovery: Arc<dyn ServiceDiscovery>) -> Self {
        let service_client = ServiceClient::new(service_discovery.clone());

        Self {
            service_discovery,
            service_client,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            load_balancers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new service instance
    pub async fn register_service(&self, instance: ServiceInstance) -> Result<()> {
        self.service_discovery.register_service(instance).await
    }

    /// Call a service with circuit breaker protection
    pub async fn call_service<T, U>(
        &self,
        service_type: ServiceType,
        method: &str,
        path: &str,
        body: Option<&T>,
    ) -> Result<U>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        let circuit_breaker = self
            .get_or_create_circuit_breaker(service_type.clone())
            .await;

        // Execute with circuit breaker using ServiceError for std::error::Error compatibility
        let result: Result<U, ServiceError> = circuit_breaker
            .execute(async {
                let service_result: Result<U, ServiceError> = match method.to_uppercase().as_str() {
                    "GET" => self
                        .service_client
                        .get(service_type.clone(), path)
                        .await
                        .map_err(ServiceError::from),
                    "POST" => {
                        if let Some(body) = body {
                            self.service_client
                                .post(service_type.clone(), path, body)
                                .await
                                .map_err(ServiceError::from)
                        } else {
                            Err(ServiceError::Request(
                                "POST request requires body".to_string(),
                            ))
                        }
                    }
                    _ => Err(ServiceError::Request(format!(
                        "Unsupported HTTP method: {}",
                        method
                    ))),
                };
                service_result
            })
            .await;

        // Convert back to anyhow::Error for consistent API
        result.map_err(anyhow::Error::from)
    }

    /// Get service health status
    pub async fn get_service_health(
        &self,
        service_type: ServiceType,
    ) -> Result<ServiceHealthSummary> {
        let all_services = self
            .service_discovery
            .discover_services(service_type)
            .await?;
        let healthy_services = all_services
            .iter()
            .filter(|s| s.health_status == ServiceHealth::Healthy)
            .count();

        Ok(ServiceHealthSummary {
            total_instances: all_services.len(),
            healthy_instances: healthy_services,
            unhealthy_instances: all_services.len() - healthy_services,
            overall_health: if healthy_services == 0 {
                ServiceHealth::Unhealthy
            } else if healthy_services == all_services.len() {
                ServiceHealth::Healthy
            } else {
                ServiceHealth::Degraded
            },
        })
    }

    /// Deploy a new service
    pub async fn deploy_service(&self, deployment: ServiceDeployment) -> Result<String> {
        info!(
            "Deploying service: {} version {}",
            deployment.service_type, deployment.version
        );

        let service_id = format!(
            "{}_{}_{}",
            format!("{:?}", deployment.service_type).to_lowercase(),
            deployment.version.replace('.', "_"),
            Uuid::new_v4()
        );

        let instance = ServiceInstance {
            id: service_id.clone(),
            service_type: deployment.service_type,
            version: deployment.version,
            address: deployment.address,
            port: deployment.port,
            health_status: ServiceHealth::Starting,
            capabilities: deployment.capabilities,
            metadata: deployment.metadata,
            dependencies: deployment.dependencies,
            resource_requirements: deployment.resource_requirements,
            started_at: chrono::Utc::now(),
            last_heartbeat: chrono::Utc::now(),
        };

        self.service_discovery.register_service(instance).await?;

        // In a real implementation, this would start the actual service process

        info!("Service deployed successfully: {}", service_id);
        Ok(service_id)
    }

    async fn get_or_create_circuit_breaker(&self, service_type: ServiceType) -> CircuitBreaker {
        let mut circuit_breakers = self.circuit_breakers.write().await;

        if !circuit_breakers.contains_key(&service_type) {
            let circuit_breaker = CircuitBreaker::new(5, Duration::from_secs(60));
            circuit_breakers.insert(service_type.clone(), circuit_breaker);
        }

        circuit_breakers.get(&service_type).unwrap().clone()
    }
}

/// Load balancer trait for service selection
#[async_trait::async_trait]
pub trait LoadBalancer: Send + Sync {
    async fn select_service(&self, services: &[ServiceInstance]) -> Option<ServiceInstance>;
}

/// Round-robin load balancer
pub struct RoundRobinLoadBalancer {
    counter: Arc<RwLock<usize>>,
}

impl RoundRobinLoadBalancer {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(RwLock::new(0)),
        }
    }
}

#[async_trait::async_trait]
impl LoadBalancer for RoundRobinLoadBalancer {
    async fn select_service(&self, services: &[ServiceInstance]) -> Option<ServiceInstance> {
        if services.is_empty() {
            return None;
        }

        let mut counter = self.counter.write().await;
        let index = *counter % services.len();
        *counter = (*counter + 1) % services.len();

        Some(services[index].clone())
    }
}

/// Service deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDeployment {
    pub service_type: ServiceType,
    pub version: String,
    pub address: String,
    pub port: u16,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub dependencies: Vec<ServiceType>,
    pub resource_requirements: ResourceRequirements,
}

/// Service health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthSummary {
    pub total_instances: usize,
    pub healthy_instances: usize,
    pub unhealthy_instances: usize,
    pub overall_health: ServiceHealth,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_registry() -> Result<()> {
        let registry = InMemoryServiceRegistry::new();

        let instance = ServiceInstance {
            id: "test-service-1".to_string(),
            service_type: ServiceType::RegionServer,
            version: "1.0.0".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            health_status: ServiceHealth::Healthy,
            capabilities: vec!["regions".to_string()],
            metadata: HashMap::new(),
            dependencies: vec![],
            resource_requirements: ResourceRequirements {
                cpu_cores: 2.0,
                memory_mb: 1024,
                disk_mb: 5000,
                network_mbps: 100,
            },
            started_at: chrono::Utc::now(),
            last_heartbeat: chrono::Utc::now(),
        };

        registry.register_service(instance.clone()).await?;

        let discovered = registry
            .discover_services(ServiceType::RegionServer)
            .await?;
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].id, "test-service-1");

        Ok(())
    }

    #[tokio::test]
    async fn test_circuit_breaker() -> Result<()> {
        let circuit_breaker = CircuitBreaker::new(2, Duration::from_millis(100));

        // Test successful calls
        let result = circuit_breaker
            .execute(async { Ok::<_, ServiceError>(42) })
            .await;
        assert!(result.is_ok());

        // Test failing calls
        for _ in 0..2 {
            let _ = circuit_breaker
                .execute(async { Err::<i32, _>(ServiceError::Internal("Test error".to_string())) })
                .await;
        }

        // Circuit should be open now
        let state = circuit_breaker.state.read().await.clone();
        assert_eq!(state, CircuitBreakerState::Open);

        Ok(())
    }

    #[tokio::test]
    async fn test_service_mesh() -> Result<()> {
        let registry = Arc::new(InMemoryServiceRegistry::new());
        let mesh = ServiceMesh::new(registry.clone());

        let instance = ServiceInstance {
            id: "test-service-1".to_string(),
            service_type: ServiceType::AssetService,
            version: "1.0.0".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            health_status: ServiceHealth::Healthy,
            capabilities: vec!["assets".to_string()],
            metadata: HashMap::new(),
            dependencies: vec![],
            resource_requirements: ResourceRequirements {
                cpu_cores: 1.0,
                memory_mb: 512,
                disk_mb: 1000,
                network_mbps: 50,
            },
            started_at: chrono::Utc::now(),
            last_heartbeat: chrono::Utc::now(),
        };

        mesh.register_service(instance).await?;

        let health = mesh.get_service_health(ServiceType::AssetService).await?;
        assert_eq!(health.total_instances, 1);
        assert_eq!(health.healthy_instances, 1);

        Ok(())
    }
}
