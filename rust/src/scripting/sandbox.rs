//! Secure sandboxing environment for LSL script execution

use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use super::{LSLValue, ScriptContext};

/// Resource limits for script execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory: usize,
    /// Maximum execution time per event in milliseconds
    pub max_execution_time_ms: u64,
    /// Maximum number of function calls per event
    pub max_function_calls: u32,
    /// Maximum script execution frequency (events per second)
    pub max_event_frequency: f32,
    /// Maximum number of HTTP requests per minute
    pub max_http_requests_per_minute: u32,
    /// Maximum number of email sends per hour
    pub max_emails_per_hour: u32,
    /// Maximum number of sensors
    pub max_sensors: u32,
    /// Maximum number of listeners
    pub max_listeners: u32,
    /// Maximum number of timers
    pub max_timers: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 64 * 1024, // 64KB
            max_execution_time_ms: 100, // 100ms
            max_function_calls: 1000,
            max_event_frequency: 100.0, // 100 events per second
            max_http_requests_per_minute: 60,
            max_emails_per_hour: 10,
            max_sensors: 16,
            max_listeners: 64,
            max_timers: 10,
        }
    }
}

/// Script sandbox security levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityLevel {
    /// Minimal restrictions for trusted scripts
    Trusted,
    /// Standard restrictions for user scripts
    Standard,
    /// High restrictions for untrusted scripts
    Restricted,
    /// Maximum restrictions for potentially dangerous scripts
    Quarantined,
}

impl SecurityLevel {
    /// Get resource limits for this security level
    pub fn get_limits(&self) -> ResourceLimits {
        match self {
            SecurityLevel::Trusted => ResourceLimits {
                max_memory: 1024 * 1024, // 1MB
                max_execution_time_ms: 1000, // 1 second
                max_function_calls: 10000,
                max_event_frequency: 1000.0,
                max_http_requests_per_minute: 600,
                max_emails_per_hour: 100,
                max_sensors: 64,
                max_listeners: 256,
                max_timers: 100,
            },
            SecurityLevel::Standard => ResourceLimits::default(),
            SecurityLevel::Restricted => ResourceLimits {
                max_memory: 32 * 1024, // 32KB
                max_execution_time_ms: 50, // 50ms
                max_function_calls: 500,
                max_event_frequency: 50.0,
                max_http_requests_per_minute: 30,
                max_emails_per_hour: 5,
                max_sensors: 8,
                max_listeners: 32,
                max_timers: 5,
            },
            SecurityLevel::Quarantined => ResourceLimits {
                max_memory: 16 * 1024, // 16KB
                max_execution_time_ms: 25, // 25ms
                max_function_calls: 100,
                max_event_frequency: 10.0,
                max_http_requests_per_minute: 5,
                max_emails_per_hour: 1,
                max_sensors: 2,
                max_listeners: 8,
                max_timers: 2,
            },
        }
    }
}

/// Sandbox violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxViolation {
    /// Exceeded memory limit
    MemoryLimit {
        used: usize,
        limit: usize,
    },
    /// Exceeded execution time limit
    ExecutionTimeLimit {
        time_ms: u64,
        limit_ms: u64,
    },
    /// Exceeded function call limit
    FunctionCallLimit {
        calls: u32,
        limit: u32,
    },
    /// Exceeded event frequency limit
    EventFrequencyLimit {
        frequency: f32,
        limit: f32,
    },
    /// Attempted unauthorized operation
    UnauthorizedOperation {
        operation: String,
        reason: String,
    },
    /// Attempted to access forbidden resource
    ForbiddenResource {
        resource: String,
        reason: String,
    },
    /// Script appears to be in infinite loop
    InfiniteLoop {
        duration_ms: u64,
    },
    /// Too many HTTP requests
    HttpRateLimit {
        requests: u32,
        limit: u32,
    },
    /// Too many email sends
    EmailRateLimit {
        emails: u32,
        limit: u32,
    },
}

/// Script execution statistics
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    pub memory_used: usize,
    pub execution_time_ms: u64,
    pub function_calls: u32,
    pub events_processed: u64,
    pub http_requests: u32,
    pub emails_sent: u32,
    pub violations: Vec<SandboxViolation>,
    pub last_event_time: Option<Instant>,
    pub event_frequency: f32,
}

/// Secure sandbox for script execution
pub struct ScriptSandbox {
    /// Script ID
    script_id: Uuid,
    /// Security level
    security_level: SecurityLevel,
    /// Resource limits
    limits: ResourceLimits,
    /// Current execution statistics
    stats: ExecutionStats,
    /// Allowed function list
    allowed_functions: HashMap<String, bool>,
    /// Blocked resources
    blocked_resources: Vec<String>,
    /// Last resource usage check
    last_check: Instant,
}

impl ScriptSandbox {
    /// Create a new sandbox for a script
    pub fn new(script_id: Uuid, security_level: SecurityLevel) -> Self {
        let limits = security_level.get_limits();
        let allowed_functions = Self::create_function_whitelist(&security_level);
        let blocked_resources = Self::create_resource_blacklist(&security_level);

        Self {
            script_id,
            security_level,
            limits,
            stats: ExecutionStats::default(),
            allowed_functions,
            blocked_resources,
            last_check: Instant::now(),
        }
    }

    /// Check if a function call is allowed
    pub fn check_function_call(&mut self, function_name: &str) -> Result<()> {
        // Check if function is allowed
        if !self.allowed_functions.get(function_name).unwrap_or(&true) {
            let violation = SandboxViolation::UnauthorizedOperation {
                operation: format!("function call: {}", function_name),
                reason: "Function not allowed at this security level".to_string(),
            };
            self.stats.violations.push(violation.clone());
            return Err(anyhow!("Unauthorized function call: {}", function_name));
        }

        // Increment function call counter
        self.stats.function_calls += 1;

        // Check function call limit
        if self.stats.function_calls > self.limits.max_function_calls {
            let violation = SandboxViolation::FunctionCallLimit {
                calls: self.stats.function_calls,
                limit: self.limits.max_function_calls,
            };
            self.stats.violations.push(violation.clone());
            return Err(anyhow!("Exceeded function call limit: {}/{}", 
                self.stats.function_calls, self.limits.max_function_calls));
        }

        debug!("Function call allowed: {} ({}/{})", 
            function_name, self.stats.function_calls, self.limits.max_function_calls);
        Ok(())
    }

    /// Check if a resource access is allowed
    pub fn check_resource_access(&mut self, resource: &str) -> Result<()> {
        // Check if resource is blocked
        for blocked in &self.blocked_resources {
            if resource.contains(blocked) {
                let violation = SandboxViolation::ForbiddenResource {
                    resource: resource.to_string(),
                    reason: format!("Resource matches blocked pattern: {}", blocked),
                };
                self.stats.violations.push(violation.clone());
                return Err(anyhow!("Access to resource '{}' is forbidden", resource));
            }
        }

        debug!("Resource access allowed: {}", resource);
        Ok(())
    }

    /// Start execution timing
    pub fn start_execution(&mut self) -> ExecutionTimer {
        let now = Instant::now();
        
        // Update event frequency
        if let Some(last_time) = self.stats.last_event_time {
            let elapsed = now.duration_since(last_time).as_secs_f32();
            if elapsed > 0.0 {
                self.stats.event_frequency = 1.0 / elapsed;
            }
        }
        self.stats.last_event_time = Some(now);

        // Check event frequency limit
        if self.stats.event_frequency > self.limits.max_event_frequency {
            let violation = SandboxViolation::EventFrequencyLimit {
                frequency: self.stats.event_frequency,
                limit: self.limits.max_event_frequency,
            };
            self.stats.violations.push(violation);
            warn!("Event frequency limit exceeded: {:.2}/{:.2} events/sec",
                self.stats.event_frequency, self.limits.max_event_frequency);
        }

        ExecutionTimer::new(now, self.limits.max_execution_time_ms)
    }

    /// End execution timing and check limits
    pub fn end_execution(&mut self, timer: ExecutionTimer) -> Result<()> {
        let execution_time = timer.elapsed_ms();
        self.stats.execution_time_ms += execution_time;

        // Check execution time limit
        if execution_time > self.limits.max_execution_time_ms {
            let violation = SandboxViolation::ExecutionTimeLimit {
                time_ms: execution_time,
                limit_ms: self.limits.max_execution_time_ms,
            };
            self.stats.violations.push(violation.clone());
            return Err(anyhow!("Execution time limit exceeded: {}ms/{}ms",
                execution_time, self.limits.max_execution_time_ms));
        }

        debug!("Execution completed in {}ms", execution_time);
        Ok(())
    }

    /// Check memory usage
    pub fn check_memory_usage(&mut self, used_memory: usize) -> Result<()> {
        self.stats.memory_used = used_memory;

        if used_memory > self.limits.max_memory {
            let violation = SandboxViolation::MemoryLimit {
                used: used_memory,
                limit: self.limits.max_memory,
            };
            self.stats.violations.push(violation.clone());
            return Err(anyhow!("Memory limit exceeded: {}/{} bytes",
                used_memory, self.limits.max_memory));
        }

        Ok(())
    }

    /// Record HTTP request
    pub fn record_http_request(&mut self) -> Result<()> {
        self.stats.http_requests += 1;

        // Check rate limit (simplified - per execution rather than per minute)
        if self.stats.http_requests > self.limits.max_http_requests_per_minute {
            let violation = SandboxViolation::HttpRateLimit {
                requests: self.stats.http_requests,
                limit: self.limits.max_http_requests_per_minute,
            };
            self.stats.violations.push(violation.clone());
            return Err(anyhow!("HTTP request rate limit exceeded"));
        }

        Ok(())
    }

    /// Record email send
    pub fn record_email_send(&mut self) -> Result<()> {
        self.stats.emails_sent += 1;

        // Check rate limit (simplified - per execution rather than per hour)
        if self.stats.emails_sent > self.limits.max_emails_per_hour {
            let violation = SandboxViolation::EmailRateLimit {
                emails: self.stats.emails_sent,
                limit: self.limits.max_emails_per_hour,
            };
            self.stats.violations.push(violation.clone());
            return Err(anyhow!("Email rate limit exceeded"));
        }

        Ok(())
    }

    /// Get current execution statistics
    pub fn get_stats(&self) -> &ExecutionStats {
        &self.stats
    }

    /// Get sandbox violations
    pub fn get_violations(&self) -> &[SandboxViolation] {
        &self.stats.violations
    }

    /// Reset statistics (called at the start of each event)
    pub fn reset_event_stats(&mut self) {
        self.stats.function_calls = 0;
        self.stats.execution_time_ms = 0;
        // Keep cumulative stats like events_processed, http_requests, etc.
    }

    /// Create function whitelist based on security level
    fn create_function_whitelist(security_level: &SecurityLevel) -> HashMap<String, bool> {
        let mut whitelist = HashMap::new();

        // Basic functions allowed at all levels
        let basic_functions = vec![
            "llSay", "llWhisper", "llShout", "llOwnerSay",
            "llGetPos", "llSetPos", "llGetRot", "llSetRot",
            "llGetObjectName", "llSetObjectName", "llGetObjectDesc", "llSetObjectDesc",
            "llGetKey", "llGetOwner", "llGetRegionName",
            // Math functions
            "llAbs", "llFabs", "llSqrt", "llPow", "llSin", "llCos", "llTan",
            "llAsin", "llAcos", "llAtan2", "llFloor", "llCeil", "llRound",
            "llFrand", "llLog", "llLog10",
            // Vector/rotation functions
            "llVecDist", "llVecMag", "llVecNorm", "llEuler2Rot", "llRot2Euler",
            // String functions
            "llStringLength", "llGetSubString", "llToLower", "llToUpper",
            // List functions
            "llListLength", "llList2String", "llList2Integer", "llList2Float",
            // Time functions
            "llGetTimestamp", "llGetUnixTime",
        ];

        for func in basic_functions {
            whitelist.insert(func.to_string(), true);
        }

        // Additional functions for higher security levels
        match security_level {
            SecurityLevel::Trusted => {
                // Trusted scripts can use all functions
                let advanced_functions = vec![
                    "llHTTPRequest", "llEmail", "llRequestPermissions",
                    "llTakeControls", "llReleaseControls", "llAttachToAvatar",
                    "llDetachFromAvatar", "llTeleportAgent", "llMapDestination",
                    "llGiveInventory", "llGiveMoney", "llTransferLindenDollars",
                    "llSetScriptState", "llResetOtherScript", "llMessageLinked",
                    "llCreateLink", "llBreakLink", "llBreakAllLinks",
                    "llSetLinkPrimitiveParams", "llModifyLand", "llSetParcelMusicURL",
                    "llSetParcelMediaURL", "llParcelMediaCommandList",
                ];
                for func in advanced_functions {
                    whitelist.insert(func.to_string(), true);
                }
            }
            SecurityLevel::Standard => {
                // Standard scripts can use most functions except dangerous ones
                let standard_functions = vec![
                    "llHTTPRequest", "llListen", "llSetTimerEvent", "llSensor",
                    "llSetText", "llSetVelocity", "llApplyImpulse", "llSetStatus",
                    "llRequestPermissions", "llGiveInventory", "llMessageLinked",
                    "llSetLinkPrimitiveParams",
                ];
                for func in standard_functions {
                    whitelist.insert(func.to_string(), true);
                }
            }
            SecurityLevel::Restricted => {
                // Restricted scripts can only use basic functions
                let restricted_functions = vec![
                    "llListen", "llSetTimerEvent", "llSetText",
                ];
                for func in restricted_functions {
                    whitelist.insert(func.to_string(), true);
                }
            }
            SecurityLevel::Quarantined => {
                // Quarantined scripts have very limited functionality
                // Only basic functions are allowed (already added above)
            }
        }

        whitelist
    }

    /// Create resource blacklist based on security level
    fn create_resource_blacklist(security_level: &SecurityLevel) -> Vec<String> {
        match security_level {
            SecurityLevel::Trusted => vec![],
            SecurityLevel::Standard => vec![
                "file://".to_string(),
                "ftp://".to_string(),
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "0.0.0.0".to_string(),
            ],
            SecurityLevel::Restricted => vec![
                "file://".to_string(),
                "ftp://".to_string(),
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "0.0.0.0".to_string(),
                "192.168.".to_string(),
                "10.".to_string(),
                "172.16.".to_string(),
                "172.17.".to_string(),
                "172.18.".to_string(),
                "172.19.".to_string(),
                "172.20.".to_string(),
                "172.21.".to_string(),
                "172.22.".to_string(),
                "172.23.".to_string(),
                "172.24.".to_string(),
                "172.25.".to_string(),
                "172.26.".to_string(),
                "172.27.".to_string(),
                "172.28.".to_string(),
                "172.29.".to_string(),
                "172.30.".to_string(),
                "172.31.".to_string(),
            ],
            SecurityLevel::Quarantined => vec![
                "http://".to_string(),
                "https://".to_string(),
                "file://".to_string(),
                "ftp://".to_string(),
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "0.0.0.0".to_string(),
                "192.168.".to_string(),
                "10.".to_string(),
                "172.".to_string(),
            ],
        }
    }
}

/// Execution timer for tracking script execution time
pub struct ExecutionTimer {
    start_time: Instant,
    max_time_ms: u64,
}

impl ExecutionTimer {
    fn new(start_time: Instant, max_time_ms: u64) -> Self {
        Self {
            start_time,
            max_time_ms,
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    /// Check if execution time limit has been exceeded
    pub fn is_time_exceeded(&self) -> bool {
        self.elapsed_ms() > self.max_time_ms
    }

    /// Get remaining time in milliseconds
    pub fn remaining_ms(&self) -> u64 {
        self.max_time_ms.saturating_sub(self.elapsed_ms())
    }
}

/// Sandbox manager for multiple scripts
pub struct SandboxManager {
    sandboxes: tokio::sync::RwLock<HashMap<Uuid, Arc<tokio::sync::RwLock<ScriptSandbox>>>>,
    global_limits: ResourceLimits,
}

impl SandboxManager {
    /// Create a new sandbox manager
    pub fn new() -> Self {
        Self {
            sandboxes: tokio::sync::RwLock::new(HashMap::new()),
            global_limits: ResourceLimits::default(),
        }
    }

    /// Create a sandbox for a script
    pub async fn create_sandbox(&self, script_id: Uuid, security_level: SecurityLevel) -> Arc<tokio::sync::RwLock<ScriptSandbox>> {
        let sandbox = Arc::new(tokio::sync::RwLock::new(ScriptSandbox::new(script_id, security_level)));
        self.sandboxes.write().await.insert(script_id, sandbox.clone());
        
        info!("Created sandbox for script {} with security level {:?}", script_id, sandbox.read().await.security_level);
        sandbox
    }

    /// Get sandbox for a script
    pub async fn get_sandbox(&self, script_id: Uuid) -> Option<Arc<tokio::sync::RwLock<ScriptSandbox>>> {
        self.sandboxes.read().await.get(&script_id).cloned()
    }

    /// Remove sandbox for a script
    pub async fn remove_sandbox(&self, script_id: Uuid) -> bool {
        let removed = self.sandboxes.write().await.remove(&script_id).is_some();
        if removed {
            info!("Removed sandbox for script {}", script_id);
        }
        removed
    }

    /// Get global sandbox statistics
    pub async fn get_global_stats(&self) -> HashMap<String, u64> {
        let sandboxes = self.sandboxes.read().await;
        let mut total_memory = 0;
        let mut total_violations = 0;
        let mut total_events = 0;
        let mut total_http_requests = 0;
        let mut total_emails = 0;

        for sandbox_arc in sandboxes.values() {
            let sandbox = sandbox_arc.read().await;
            let stats = sandbox.get_stats();
            
            total_memory += stats.memory_used;
            total_violations += stats.violations.len() as u64;
            total_events += stats.events_processed;
            total_http_requests += stats.http_requests as u64;
            total_emails += stats.emails_sent as u64;
        }

        let mut global_stats = HashMap::new();
        global_stats.insert("active_sandboxes".to_string(), sandboxes.len() as u64);
        global_stats.insert("total_memory_used".to_string(), total_memory as u64);
        global_stats.insert("total_violations".to_string(), total_violations);
        global_stats.insert("total_events_processed".to_string(), total_events);
        global_stats.insert("total_http_requests".to_string(), total_http_requests);
        global_stats.insert("total_emails_sent".to_string(), total_emails);

        global_stats
    }

    /// Cleanup expired sandboxes
    pub async fn cleanup_expired_sandboxes(&self) {
        let mut sandboxes = self.sandboxes.write().await;
        let current_time = Instant::now();
        
        sandboxes.retain(|script_id, sandbox_arc| {
            let sandbox = sandbox_arc.try_read();
            match sandbox {
                Ok(sandbox) => {
                    // Keep sandbox if it was used recently (within 10 minutes)
                    if let Some(last_event) = sandbox.stats.last_event_time {
                        let elapsed = current_time.duration_since(last_event);
                        if elapsed > Duration::from_secs(600) {
                            info!("Cleaning up expired sandbox for script {}", script_id);
                            return false;
                        }
                    }
                    true
                }
                Err(_) => {
                    // If we can't read the sandbox, keep it for now
                    true
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let script_id = Uuid::new_v4();
        let sandbox = ScriptSandbox::new(script_id, SecurityLevel::Standard);
        
        assert_eq!(sandbox.script_id, script_id);
        assert_eq!(sandbox.security_level, SecurityLevel::Standard);
        assert!(sandbox.allowed_functions.contains_key("llSay"));
    }

    #[test]
    fn test_function_call_checking() {
        let script_id = Uuid::new_v4();
        let mut sandbox = ScriptSandbox::new(script_id, SecurityLevel::Restricted);
        
        // Should allow basic functions
        assert!(sandbox.check_function_call("llSay").is_ok());
        
        // Should block advanced functions at restricted level
        assert!(sandbox.check_function_call("llHTTPRequest").is_err());
    }

    #[test]
    fn test_execution_timer() {
        let timer = ExecutionTimer::new(Instant::now(), 100);
        
        // Should not be exceeded immediately
        assert!(!timer.is_time_exceeded());
        
        // Should have close to full time remaining
        assert!(timer.remaining_ms() >= 99);
    }

    #[test]
    fn test_security_levels() {
        let trusted_limits = SecurityLevel::Trusted.get_limits();
        let quarantined_limits = SecurityLevel::Quarantined.get_limits();
        
        // Trusted should have higher limits than quarantined
        assert!(trusted_limits.max_memory > quarantined_limits.max_memory);
        assert!(trusted_limits.max_execution_time_ms > quarantined_limits.max_execution_time_ms);
        assert!(trusted_limits.max_function_calls > quarantined_limits.max_function_calls);
    }

    #[tokio::test]
    async fn test_sandbox_manager() {
        let manager = SandboxManager::new();
        let script_id = Uuid::new_v4();
        
        // Create sandbox
        let sandbox = manager.create_sandbox(script_id, SecurityLevel::Standard).await;
        assert!(manager.get_sandbox(script_id).await.is_some());
        
        // Remove sandbox
        assert!(manager.remove_sandbox(script_id).await);
        assert!(manager.get_sandbox(script_id).await.is_none());
    }
}