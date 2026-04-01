//! Zig FFI bindings for OpenZiti C SDK
//!
//! Provides safe Rust bindings to the OpenZiti C library through Zig FFI layer.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void, c_uint};
use std::ptr;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

/// OpenZiti context handle
pub type ZitiContextHandle = *mut c_void;

/// OpenZiti connection handle
pub type ZitiConnectionHandle = *mut c_void;

/// OpenZiti service handle
pub type ZitiServiceHandle = *mut c_void;

/// Callback function type for connection events
pub type ZitiConnectionCallback = extern "C" fn(connection: ZitiConnectionHandle, status: c_int, data: *mut c_void);

/// Callback function type for data events
pub type ZitiDataCallback = extern "C" fn(connection: ZitiConnectionHandle, data: *const c_char, len: c_uint, user_data: *mut c_void);

/// Callback function type for service events
pub type ZitiServiceCallback = extern "C" fn(service: ZitiServiceHandle, status: c_int, data: *mut c_void);

/// OpenZiti initialization parameters
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ZitiInitParams {
    pub controller_url: *const c_char,
    pub identity_file: *const c_char,
    pub log_level: c_int,
    pub config_dir: *const c_char,
}

/// OpenZiti connection parameters
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ZitiConnectParams {
    pub service_name: *const c_char,
    pub session_type: c_int,
    pub timeout_ms: c_uint,
    pub user_data: *mut c_void,
}

/// OpenZiti service configuration
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ZitiServiceConfig {
    pub name: *const c_char,
    pub endpoint_address: *const c_char,
    pub port: c_uint,
    pub protocol: c_int,
    pub encrypt: c_int,
}

/// OpenZiti network statistics
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ZitiNetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections_active: c_uint,
    pub connections_total: c_uint,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors: c_uint,
}

/// External functions from Zig FFI layer
extern "C" {
    /// Initialize OpenZiti context
    fn ziti_init(params: *const ZitiInitParams) -> ZitiContextHandle;
    
    /// Shutdown OpenZiti context
    fn ziti_shutdown(context: ZitiContextHandle) -> c_int;
    
    /// Connect to OpenZiti service
    fn ziti_connect(
        context: ZitiContextHandle,
        params: *const ZitiConnectParams,
        callback: ZitiConnectionCallback
    ) -> c_int;
    
    /// Disconnect from OpenZiti service
    fn ziti_disconnect(connection: ZitiConnectionHandle) -> c_int;
    
    /// Send data through OpenZiti connection
    fn ziti_send(
        connection: ZitiConnectionHandle,
        data: *const c_char,
        len: c_uint,
        callback: ZitiDataCallback,
        user_data: *mut c_void
    ) -> c_int;
    
    /// Receive data from OpenZiti connection
    fn ziti_receive(
        connection: ZitiConnectionHandle,
        buffer: *mut c_char,
        buffer_len: c_uint,
        callback: ZitiDataCallback,
        user_data: *mut c_void
    ) -> c_int;
    
    /// Create OpenZiti service
    fn ziti_create_service(
        context: ZitiContextHandle,
        config: *const ZitiServiceConfig,
        callback: ZitiServiceCallback
    ) -> c_int;
    
    /// Delete OpenZiti service
    fn ziti_delete_service(
        context: ZitiContextHandle,
        service_name: *const c_char
    ) -> c_int;
    
    /// Get network statistics
    fn ziti_get_stats(
        context: ZitiContextHandle,
        stats: *mut ZitiNetworkStats
    ) -> c_int;
    
    /// Set log level
    fn ziti_set_log_level(level: c_int) -> c_int;
    
    /// Get version information
    fn ziti_get_version() -> *const c_char;
    
    /// Check if context is ready
    fn ziti_is_ready(context: ZitiContextHandle) -> c_int;
    
    /// Get last error message
    fn ziti_get_last_error() -> *const c_char;
    
    /// Enable/disable encryption
    fn ziti_set_encryption(context: ZitiContextHandle, enabled: c_int) -> c_int;
    
    /// Set connection timeout
    fn ziti_set_timeout(context: ZitiContextHandle, timeout_ms: c_uint) -> c_int;
    
    /// Add trusted certificate
    fn ziti_add_trusted_cert(
        context: ZitiContextHandle,
        cert_path: *const c_char
    ) -> c_int;
    
    /// Enable service hosting
    fn ziti_host_service(
        context: ZitiContextHandle,
        service_name: *const c_char,
        address: *const c_char,
        port: c_uint,
        callback: ZitiServiceCallback
    ) -> c_int;
    
    /// Process events (non-blocking)
    fn ziti_process_events(context: ZitiContextHandle, timeout_ms: c_uint) -> c_int;
}

/// Safe Rust wrapper for OpenZiti FFI
pub struct ZitiFFI {
    context: ZitiContextHandle,
    is_initialized: bool,
}

impl ZitiFFI {
    /// Create a new OpenZiti FFI instance
    pub fn new() -> Self {
        Self {
            context: ptr::null_mut(),
            is_initialized: false,
        }
    }

    /// Initialize OpenZiti with configuration
    pub fn initialize(&mut self, controller_url: &str, identity_file: &str, log_level: ZitiLogLevel) -> Result<()> {
        if self.is_initialized {
            return Ok(());
        }

        let controller_url_c = CString::new(controller_url)
            .map_err(|e| anyhow!(format!("Invalid controller URL: {}", e)))?;
        
        let identity_file_c = CString::new(identity_file)
            .map_err(|e| anyhow!(format!("Invalid identity file path: {}", e)))?;
        
        let config_dir_c = CString::new("/tmp/ziti")
            .map_err(|e| anyhow!(format!("Invalid config directory: {}", e)))?;

        let params = ZitiInitParams {
            controller_url: controller_url_c.as_ptr(),
            identity_file: identity_file_c.as_ptr(),
            log_level: log_level as c_int,
            config_dir: config_dir_c.as_ptr(),
        };

        unsafe {
            self.context = ziti_init(&params);
            if self.context.is_null() {
                let error_msg = self.get_last_error();
                return Err(anyhow!(
                    format!("Failed to initialize OpenZiti: {}", error_msg)
                ));
            }
        }

        self.is_initialized = true;
        tracing::info!("OpenZiti FFI initialized successfully");
        Ok(())
    }

    /// Shutdown OpenZiti
    pub fn shutdown(&mut self) -> Result<()> {
        if !self.is_initialized {
            return Ok(());
        }

        unsafe {
            let result = ziti_shutdown(self.context);
            if result != 0 {
                let error_msg = self.get_last_error();
                return Err(anyhow!(
                    format!("Failed to shutdown OpenZiti: {}", error_msg)
                ));
            }
        }

        self.context = ptr::null_mut();
        self.is_initialized = false;
        tracing::info!("OpenZiti FFI shutdown successfully");
        Ok(())
    }

    /// Connect to a service
    pub fn connect_to_service(&self, service_name: &str, timeout_ms: u32) -> Result<ZitiConnectionWrapper> {
        if !self.is_initialized {
            return Err(anyhow!(
                "OpenZiti not initialized".to_string()
            ));
        }

        let service_name_c = CString::new(service_name)
            .map_err(|e| anyhow!(format!("Invalid service name: {}", e)))?;

        let params = ZitiConnectParams {
            service_name: service_name_c.as_ptr(),
            session_type: 0, // Default session type
            timeout_ms,
            user_data: ptr::null_mut(),
        };

        let mut connection_handle = ptr::null_mut();
        
        unsafe {
            let result = ziti_connect(
                self.context,
                &params,
                Self::connection_callback
            );
            
            if result != 0 {
                let error_msg = self.get_last_error();
                return Err(anyhow!(
                    format!("Failed to connect to service {}: {}", service_name, error_msg)
                ));
            }
        }

        Ok(ZitiConnectionWrapper::new(connection_handle))
    }

    /// Create a service
    pub fn create_service(&self, name: &str, address: &str, port: u16, protocol: ZitiProtocolType) -> Result<()> {
        if !self.is_initialized {
            return Err(anyhow!(
                "OpenZiti not initialized".to_string()
            ));
        }

        let name_c = CString::new(name)
            .map_err(|e| anyhow!(format!("Invalid service name: {}", e)))?;
        
        let address_c = CString::new(address)
            .map_err(|e| anyhow!(format!("Invalid address: {}", e)))?;

        let config = ZitiServiceConfig {
            name: name_c.as_ptr(),
            endpoint_address: address_c.as_ptr(),
            port: port as c_uint,
            protocol: protocol as c_int,
            encrypt: 1, // Always enable encryption
        };

        unsafe {
            let result = ziti_create_service(
                self.context,
                &config,
                Self::service_callback
            );
            
            if result != 0 {
                let error_msg = self.get_last_error();
                return Err(anyhow!(
                    format!("Failed to create service {}: {}", name, error_msg)
                ));
            }
        }

        tracing::info!("Created OpenZiti service: {}", name);
        Ok(())
    }

    /// Get network statistics
    pub fn get_network_stats(&self) -> Result<ZitiNetworkStats> {
        if !self.is_initialized {
            return Err(anyhow!(
                "OpenZiti not initialized".to_string()
            ));
        }

        let mut stats = ZitiNetworkStats {
            bytes_sent: 0,
            bytes_received: 0,
            connections_active: 0,
            connections_total: 0,
            packets_sent: 0,
            packets_received: 0,
            errors: 0,
        };

        unsafe {
            let result = ziti_get_stats(self.context, &mut stats);
            if result != 0 {
                let error_msg = self.get_last_error();
                return Err(anyhow!(
                    format!("Failed to get network stats: {}", error_msg)
                ));
            }
        }

        Ok(stats)
    }

    /// Check if OpenZiti is ready
    pub fn is_ready(&self) -> bool {
        if !self.is_initialized {
            return false;
        }

        unsafe {
            ziti_is_ready(self.context) == 1
        }
    }

    /// Get OpenZiti version
    pub fn get_version(&self) -> String {
        unsafe {
            let version_ptr = ziti_get_version();
            if version_ptr.is_null() {
                return "unknown".to_string();
            }
            
            CStr::from_ptr(version_ptr)
                .to_string_lossy()
                .to_string()
        }
    }

    /// Get last error message
    pub fn get_last_error(&self) -> String {
        unsafe {
            let error_ptr = ziti_get_last_error();
            if error_ptr.is_null() {
                return "No error".to_string();
            }
            
            CStr::from_ptr(error_ptr)
                .to_string_lossy()
                .to_string()
        }
    }

    /// Process network events
    pub fn process_events(&self, timeout_ms: u32) -> Result<()> {
        if !self.is_initialized {
            return Err(anyhow!(
                "OpenZiti not initialized".to_string()
            ));
        }

        unsafe {
            let result = ziti_process_events(self.context, timeout_ms);
            if result < 0 {
                let error_msg = self.get_last_error();
                return Err(anyhow!(
                    format!("Failed to process events: {}", error_msg)
                ));
            }
        }

        Ok(())
    }

    /// Connection callback handler
    extern "C" fn connection_callback(connection: ZitiConnectionHandle, status: c_int, _data: *mut c_void) {
        match status {
            0 => tracing::info!("Connection established successfully"),
            _ => tracing::error!("Connection failed with status: {}", status),
        }
    }

    /// Service callback handler
    extern "C" fn service_callback(service: ZitiServiceHandle, status: c_int, _data: *mut c_void) {
        match status {
            0 => tracing::info!("Service operation completed successfully"),
            _ => tracing::error!("Service operation failed with status: {}", status),
        }
    }
}

impl Drop for ZitiFFI {
    fn drop(&mut self) {
        if self.is_initialized {
            let _ = self.shutdown();
        }
    }
}

/// Safe wrapper for OpenZiti connections
pub struct ZitiConnectionWrapper {
    handle: ZitiConnectionHandle,
}

impl ZitiConnectionWrapper {
    pub fn new(handle: ZitiConnectionHandle) -> Self {
        Self { handle }
    }

    /// Send data through the connection
    pub fn send_data(&self, data: &[u8]) -> Result<()> {
        if self.handle.is_null() {
            return Err(anyhow!(
                "Invalid connection handle".to_string()
            ));
        }

        unsafe {
            let result = ziti_send(
                self.handle,
                data.as_ptr() as *const c_char,
                data.len() as c_uint,
                Self::data_callback,
                ptr::null_mut()
            );
            
            if result != 0 {
                return Err(anyhow!(
                    format!("Failed to send data: error code {}", result)
                ));
            }
        }

        Ok(())
    }

    /// Receive data from the connection
    pub fn receive_data(&self, buffer: &mut [u8]) -> Result<usize> {
        if self.handle.is_null() {
            return Err(anyhow!(
                "Invalid connection handle".to_string()
            ));
        }

        unsafe {
            let result = ziti_receive(
                self.handle,
                buffer.as_mut_ptr() as *mut c_char,
                buffer.len() as c_uint,
                Self::data_callback,
                ptr::null_mut()
            );
            
            if result < 0 {
                return Err(anyhow!(
                    format!("Failed to receive data: error code {}", result)
                ));
            }
            
            Ok(result as usize)
        }
    }

    /// Data callback handler
    extern "C" fn data_callback(_connection: ZitiConnectionHandle, _data: *const c_char, len: c_uint, _user_data: *mut c_void) {
        tracing::debug!("Data transfer completed: {} bytes", len);
    }

    /// Disconnect the connection
    pub fn disconnect(&self) -> Result<()> {
        if self.handle.is_null() {
            return Ok(());
        }

        unsafe {
            let result = ziti_disconnect(self.handle);
            if result != 0 {
                return Err(anyhow!(
                    format!("Failed to disconnect: error code {}", result)
                ));
            }
        }

        Ok(())
    }
}

impl Drop for ZitiConnectionWrapper {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}

/// OpenZiti log levels
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum ZitiLogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

/// OpenZiti protocol types
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum ZitiProtocolType {
    Tcp = 0,
    Udp = 1,
    Http = 2,
    Https = 3,
    WebSocket = 4,
    Custom = 5,
}

unsafe impl Send for ZitiFFI {}
unsafe impl Sync for ZitiFFI {}
unsafe impl Send for ZitiConnectionWrapper {}
unsafe impl Sync for ZitiConnectionWrapper {}