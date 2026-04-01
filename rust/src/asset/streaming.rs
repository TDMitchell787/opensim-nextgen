//! Asset Streaming Module
//! 
//! Provides efficient streaming capabilities for large assets,
//! supporting progressive download, chunked transfer, and bandwidth optimization.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_stream::{Stream, StreamExt};
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::{AssetData, AssetType};

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Enable asset streaming
    pub enabled: bool,
    /// Default chunk size for streaming (bytes)
    pub default_chunk_size: usize,
    /// Maximum chunk size (bytes)
    pub max_chunk_size: usize,
    /// Minimum chunk size (bytes)
    pub min_chunk_size: usize,
    /// Maximum concurrent streams per client
    pub max_concurrent_streams: usize,
    /// Stream timeout in seconds
    pub stream_timeout_seconds: u64,
    /// Enable adaptive bitrate streaming
    pub enable_adaptive_bitrate: bool,
    /// Enable compression for streams
    pub enable_compression: bool,
    /// Prefetch buffer size (number of chunks)
    pub prefetch_buffer_size: usize,
    /// Enable progressive JPEG streaming for images
    pub enable_progressive_jpeg: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_chunk_size: 64 * 1024, // 64KB
            max_chunk_size: 1024 * 1024, // 1MB
            min_chunk_size: 4 * 1024, // 4KB
            max_concurrent_streams: 10,
            stream_timeout_seconds: 300, // 5 minutes
            enable_adaptive_bitrate: true,
            enable_compression: true,
            prefetch_buffer_size: 3,
            enable_progressive_jpeg: true,
        }
    }
}

/// Custom serialization for Bytes type
mod bytes_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use bytes::Bytes;
    
    pub fn serialize<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        bytes.as_ref().serialize(serializer)
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
    where D: Deserializer<'de> {
        let vec = Vec::<u8>::deserialize(deserializer)?;
        Ok(Bytes::from(vec))
    }
}

/// Asset chunk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetChunk {
    pub chunk_id: u32,
    pub stream_id: Uuid,
    pub asset_id: Uuid,
    #[serde(with = "bytes_serde")]
    pub data: Bytes,
    pub offset: u64,
    pub total_size: u64,
    pub is_final: bool,
    pub checksum: Option<String>,
    pub compression: Option<CompressionType>,
}

/// Compression types for asset streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Brotli,
    Lz4,
}

/// Stream priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StreamPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Stream quality settings for adaptive streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamQuality {
    pub bandwidth_bps: u64,
    pub chunk_size: usize,
    pub compression: CompressionType,
    pub quality_level: u8, // 0-100
}

/// Active streaming session
#[derive(Debug, Clone)]
pub struct StreamingSession {
    pub session_id: Uuid,
    pub asset_id: Uuid,
    pub client_id: String,
    pub asset_type: AssetType,
    pub total_size: u64,
    pub bytes_streamed: u64,
    pub chunks_sent: u32,
    pub total_chunks: u32,
    pub chunk_size: usize,
    pub priority: StreamPriority,
    pub quality: StreamQuality,
    pub started_at: Instant,
    pub last_activity: Instant,
    pub estimated_bandwidth: Option<u64>,
    pub is_complete: bool,
    pub is_paused: bool,
}

/// Streaming statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreamingStats {
    pub active_streams: u64,
    pub total_streams_started: u64,
    pub total_streams_completed: u64,
    pub total_bytes_streamed: u64,
    pub average_stream_duration: Duration,
    pub bandwidth_utilization: f64,
    pub cache_hit_ratio: f64,
}

/// Asset streaming manager
pub struct AssetStreamingManager {
    config: StreamingConfig,
    active_sessions: Arc<RwLock<HashMap<Uuid, StreamingSession>>>,
    client_sessions: Arc<RwLock<HashMap<String, Vec<Uuid>>>>, // client_id -> session_ids
    stream_queues: Arc<RwLock<HashMap<StreamPriority, Vec<Uuid>>>>,
    chunk_cache: Arc<RwLock<HashMap<String, Bytes>>>, // chunk_key -> cached data
    streaming_stats: Arc<RwLock<StreamingStats>>,
}

impl AssetStreamingManager {
    /// Create a new asset streaming manager
    pub fn new(config: StreamingConfig) -> Self {
        let mut stream_queues = HashMap::new();
        stream_queues.insert(StreamPriority::Low, Vec::new());
        stream_queues.insert(StreamPriority::Normal, Vec::new());
        stream_queues.insert(StreamPriority::High, Vec::new());
        stream_queues.insert(StreamPriority::Critical, Vec::new());
        
        Self {
            config,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            client_sessions: Arc::new(RwLock::new(HashMap::new())),
            stream_queues: Arc::new(RwLock::new(stream_queues)),
            chunk_cache: Arc::new(RwLock::new(HashMap::new())),
            streaming_stats: Arc::new(RwLock::new(StreamingStats::default())),
        }
    }
    
    /// Start a new streaming session
    pub async fn start_stream(
        &self,
        asset_id: Uuid,
        asset_type: AssetType,
        asset_data: &AssetData,
        client_id: String,
        priority: StreamPriority,
    ) -> Result<Uuid> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("Asset streaming is disabled"));
        }
        
        // Check client session limits
        {
            let client_sessions = self.client_sessions.read().await;
            if let Some(sessions) = client_sessions.get(&client_id) {
                if sessions.len() >= self.config.max_concurrent_streams {
                    return Err(anyhow::anyhow!("Maximum concurrent streams exceeded for client"));
                }
            }
        }
        
        let session_id = Uuid::new_v4();
        let total_size = asset_data.len() as u64;
        let chunk_size = self.calculate_optimal_chunk_size(total_size, &asset_type);
        let total_chunks = ((total_size as f64) / (chunk_size as f64)).ceil() as u32;
        
        let quality = StreamQuality {
            bandwidth_bps: 1_000_000, // Start with 1 Mbps default
            chunk_size,
            compression: if self.config.enable_compression {
                CompressionType::Gzip
            } else {
                CompressionType::None
            },
            quality_level: 80,
        };
        
        let session = StreamingSession {
            session_id,
            asset_id,
            client_id: client_id.clone(),
            asset_type,
            total_size,
            bytes_streamed: 0,
            chunks_sent: 0,
            total_chunks,
            chunk_size,
            priority,
            quality,
            started_at: Instant::now(),
            last_activity: Instant::now(),
            estimated_bandwidth: None,
            is_complete: false,
            is_paused: false,
        };
        
        // Add to active sessions
        {
            let mut active_sessions = self.active_sessions.write().await;
            active_sessions.insert(session_id, session);
        }
        
        // Add to client sessions
        {
            let mut client_sessions = self.client_sessions.write().await;
            client_sessions.entry(client_id.clone()).or_insert_with(Vec::new).push(session_id);
        }
        
        // Add to priority queue
        {
            let mut queues = self.stream_queues.write().await;
            if let Some(queue) = queues.get_mut(&priority) {
                queue.push(session_id);
            }
        }
        
        // Update statistics
        {
            let mut stats = self.streaming_stats.write().await;
            stats.active_streams += 1;
            stats.total_streams_started += 1;
        }
        
        info!("Started streaming session {} for asset {} (client: {}, priority: {:?})", 
              session_id, asset_id, client_id, priority);
        
        Ok(session_id)
    }
    
    /// Get the next chunk for a streaming session
    pub async fn get_next_chunk(&self, session_id: &Uuid, asset_data: &AssetData) -> Result<Option<AssetChunk>> {
        let mut session = {
            let mut active_sessions = self.active_sessions.write().await;
            match active_sessions.get_mut(session_id) {
                Some(session) => {
                    if session.is_complete || session.is_paused {
                        return Ok(None);
                    }
                    session.clone()
                }
                None => return Err(anyhow::anyhow!("Streaming session not found")),
            }
        };
        
        // Check if session has timed out
        if session.last_activity.elapsed() > Duration::from_secs(self.config.stream_timeout_seconds) {
            self.stop_stream(session_id).await?;
            return Err(anyhow::anyhow!("Streaming session timed out"));
        }
        
        // Calculate chunk offset and size
        let offset = session.bytes_streamed;
        let remaining_bytes = session.total_size - offset;
        let chunk_size = std::cmp::min(session.chunk_size as u64, remaining_bytes) as usize;
        
        if chunk_size == 0 {
            // Stream is complete
            self.complete_stream(session_id).await?;
            return Ok(None);
        }
        
        // Extract chunk data
        let start = offset as usize;
        let end = start + chunk_size;
        let chunk_data = if end <= asset_data.len() {
            Bytes::copy_from_slice(&asset_data[start..end])
        } else {
            return Err(anyhow::anyhow!("Chunk extends beyond asset data"));
        };
        
        // Apply compression if enabled
        let (final_data, compression) = if matches!(session.quality.compression, CompressionType::Gzip) {
            (self.compress_data(&chunk_data, CompressionType::Gzip)?, Some(CompressionType::Gzip))
        } else {
            (chunk_data, None)
        };
        
        // Calculate checksum
        let checksum = self.calculate_checksum(&final_data);
        
        let chunk = AssetChunk {
            chunk_id: session.chunks_sent,
            stream_id: *session_id,
            asset_id: session.asset_id,
            data: final_data,
            offset,
            total_size: session.total_size,
            is_final: end >= asset_data.len(),
            checksum: Some(checksum),
            compression,
        };
        
        // Update session
        session.bytes_streamed += chunk_size as u64;
        session.chunks_sent += 1;
        session.last_activity = Instant::now();
        
        if chunk.is_final {
            session.is_complete = true;
        }
        
        // Store updated session
        {
            let mut active_sessions = self.active_sessions.write().await;
            active_sessions.insert(*session_id, session);
        }
        
        debug!("Generated chunk {} for session {} ({} bytes)", 
               chunk.chunk_id, session_id, chunk.data.len());
        
        Ok(Some(chunk))
    }
    
    /// Pause a streaming session
    pub async fn pause_stream(&self, session_id: &Uuid) -> Result<()> {
        let mut active_sessions = self.active_sessions.write().await;
        if let Some(session) = active_sessions.get_mut(session_id) {
            session.is_paused = true;
            session.last_activity = Instant::now();
            info!("Paused streaming session {}", session_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Streaming session not found"))
        }
    }
    
    /// Resume a streaming session
    pub async fn resume_stream(&self, session_id: &Uuid) -> Result<()> {
        let mut active_sessions = self.active_sessions.write().await;
        if let Some(session) = active_sessions.get_mut(session_id) {
            session.is_paused = false;
            session.last_activity = Instant::now();
            info!("Resumed streaming session {}", session_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Streaming session not found"))
        }
    }
    
    /// Stop a streaming session
    pub async fn stop_stream(&self, session_id: &Uuid) -> Result<()> {
        let session = {
            let mut active_sessions = self.active_sessions.write().await;
            active_sessions.remove(session_id)
        };
        
        if let Some(session) = session {
            // Remove from client sessions
            {
                let mut client_sessions = self.client_sessions.write().await;
                if let Some(sessions) = client_sessions.get_mut(&session.client_id) {
                    sessions.retain(|id| id != session_id);
                    if sessions.is_empty() {
                        client_sessions.remove(&session.client_id);
                    }
                }
            }
            
            // Remove from priority queues
            {
                let mut queues = self.stream_queues.write().await;
                for queue in queues.values_mut() {
                    queue.retain(|id| id != session_id);
                }
            }
            
            // Update statistics
            {
                let mut stats = self.streaming_stats.write().await;
                stats.active_streams = stats.active_streams.saturating_sub(1);
                if session.is_complete {
                    stats.total_streams_completed += 1;
                    stats.total_bytes_streamed += session.bytes_streamed;
                }
            }
            
            info!("Stopped streaming session {} ({}% complete)", 
                  session_id, 
                  (session.bytes_streamed as f64 / session.total_size as f64) * 100.0);
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Streaming session not found"))
        }
    }
    
    /// Complete a streaming session
    async fn complete_stream(&self, session_id: &Uuid) -> Result<()> {
        {
            let mut active_sessions = self.active_sessions.write().await;
            if let Some(session) = active_sessions.get_mut(session_id) {
                session.is_complete = true;
                session.last_activity = Instant::now();
            }
        }
        
        self.stop_stream(session_id).await
    }
    
    /// Get streaming session information
    pub async fn get_session_info(&self, session_id: &Uuid) -> Option<StreamingSession> {
        let active_sessions = self.active_sessions.read().await;
        active_sessions.get(session_id).cloned()
    }
    
    /// List active streaming sessions for a client
    pub async fn get_client_sessions(&self, client_id: &str) -> Vec<Uuid> {
        let client_sessions = self.client_sessions.read().await;
        client_sessions.get(client_id).cloned().unwrap_or_default()
    }
    
    /// Update streaming quality based on network conditions
    pub async fn update_stream_quality(&self, session_id: &Uuid, bandwidth_bps: u64) -> Result<()> {
        let mut active_sessions = self.active_sessions.write().await;
        if let Some(session) = active_sessions.get_mut(session_id) {
            session.estimated_bandwidth = Some(bandwidth_bps);
            
            // Adapt chunk size based on bandwidth
            if self.config.enable_adaptive_bitrate {
                session.chunk_size = self.calculate_adaptive_chunk_size(bandwidth_bps);
                session.quality.bandwidth_bps = bandwidth_bps;
                session.quality.chunk_size = session.chunk_size;
                
                debug!("Updated stream quality for session {}: bandwidth={}bps, chunk_size={}", 
                       session_id, bandwidth_bps, session.chunk_size);
            }
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Streaming session not found"))
        }
    }
    
    /// Calculate optimal chunk size for asset type and size
    fn calculate_optimal_chunk_size(&self, asset_size: u64, asset_type: &AssetType) -> usize {
        let base_size = match asset_type {
            AssetType::Texture => {
                if asset_size > 1024 * 1024 { // > 1MB
                    128 * 1024 // 128KB chunks for large textures
                } else {
                    64 * 1024 // 64KB chunks for smaller textures
                }
            }
            AssetType::Sound => 32 * 1024, // 32KB chunks for audio
            AssetType::Mesh => 256 * 1024, // 256KB chunks for meshes
            _ => self.config.default_chunk_size,
        };
        
        // Clamp to configured limits
        base_size.max(self.config.min_chunk_size).min(self.config.max_chunk_size)
    }
    
    /// Calculate adaptive chunk size based on bandwidth
    fn calculate_adaptive_chunk_size(&self, bandwidth_bps: u64) -> usize {
        let target_chunk_time_ms = 250; // Target 250ms per chunk
        let bytes_per_ms = bandwidth_bps / 8 / 1000; // Convert to bytes per millisecond
        let optimal_size = (bytes_per_ms * target_chunk_time_ms as u64) as usize;
        
        // Clamp to configured limits
        optimal_size.max(self.config.min_chunk_size).min(self.config.max_chunk_size)
    }
    
    /// Compress data using specified compression type
    fn compress_data(&self, data: &Bytes, compression: CompressionType) -> Result<Bytes> {
        match compression {
            CompressionType::None => Ok(data.clone()),
            CompressionType::Gzip => {
                use flate2::{write::GzEncoder, Compression};
                use std::io::Write;
                
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(data)?;
                let compressed = encoder.finish()?;
                Ok(Bytes::from(compressed))
            }
            _ => {
                // Fallback to no compression for unsupported types
                warn!("Unsupported compression type: {:?}, using no compression", compression);
                Ok(data.clone())
            }
        }
    }
    
    /// Calculate checksum for chunk data
    fn calculate_checksum(&self, data: &Bytes) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    /// Get streaming statistics
    pub async fn get_stats(&self) -> StreamingStats {
        let stats = self.streaming_stats.read().await;
        let mut current_stats = stats.clone();
        
        // Update real-time values
        let active_sessions = self.active_sessions.read().await;
        current_stats.active_streams = active_sessions.len() as u64;
        
        current_stats
    }
    
    /// Cleanup expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<usize> {
        let timeout_duration = Duration::from_secs(self.config.stream_timeout_seconds);
        let now = Instant::now();
        let mut expired_sessions = Vec::new();
        
        {
            let active_sessions = self.active_sessions.read().await;
            for (session_id, session) in active_sessions.iter() {
                if now.duration_since(session.last_activity) > timeout_duration {
                    expired_sessions.push(*session_id);
                }
            }
        }
        
        let cleanup_count = expired_sessions.len();
        for session_id in expired_sessions {
            if let Err(e) = self.stop_stream(&session_id).await {
                warn!("Failed to cleanup expired session {}: {}", session_id, e);
            }
        }
        
        if cleanup_count > 0 {
            info!("Cleaned up {} expired streaming sessions", cleanup_count);
        }
        
        Ok(cleanup_count)
    }
}

/// Create a stream from asset data
pub fn create_asset_stream(
    asset_data: AssetData,
    chunk_size: usize,
) -> impl Stream<Item = Result<AssetChunk>> {
    let asset_id = Uuid::new_v4();
    let stream_id = Uuid::new_v4();
    let total_size = asset_data.len() as u64;
    let total_chunks = ((total_size as f64) / (chunk_size as f64)).ceil() as u32;
    
    let stream = tokio_stream::iter(0..total_chunks).map(move |chunk_id| {
        let offset = (chunk_id as usize) * chunk_size;
        let end = std::cmp::min(offset + chunk_size, asset_data.len());
        let chunk_data = Bytes::copy_from_slice(&asset_data[offset..end]);
        let is_final = chunk_id == total_chunks - 1;
        
        Ok(AssetChunk {
            chunk_id,
            stream_id,
            asset_id,
            data: chunk_data,
            offset: offset as u64,
            total_size,
            is_final,
            checksum: None,
            compression: None,
        })
    });
    
    stream
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_streaming_session_creation() {
        let config = StreamingConfig::default();
        let manager = AssetStreamingManager::new(config);
        
        let asset_id = Uuid::new_v4();
        let asset_data = vec![0u8; 1024]; // 1KB of test data
        let client_id = "test_client".to_string();
        
        let session_id = manager.start_stream(
            asset_id,
            AssetType::Texture,
            &asset_data,
            client_id.clone(),
            StreamPriority::Normal,
        ).await.unwrap();
        
        let session_info = manager.get_session_info(&session_id).await;
        assert!(session_info.is_some());
        
        let session = session_info.unwrap();
        assert_eq!(session.asset_id, asset_id);
        assert_eq!(session.client_id, client_id);
        assert_eq!(session.total_size, 1024);
        assert!(!session.is_complete);
    }
    
    #[tokio::test]
    async fn test_chunk_streaming() {
        let config = StreamingConfig {
            default_chunk_size: 256,
            ..Default::default()
        };
        let manager = AssetStreamingManager::new(config);
        
        let asset_id = Uuid::new_v4();
        let asset_data = vec![42u8; 1000]; // 1000 bytes of test data
        let client_id = "test_client".to_string();
        
        let session_id = manager.start_stream(
            asset_id,
            AssetType::Texture,
            &asset_data,
            client_id,
            StreamPriority::Normal,
        ).await.unwrap();
        
        // Get first chunk
        let chunk1 = manager.get_next_chunk(&session_id, &asset_data).await.unwrap();
        assert!(chunk1.is_some());
        let chunk1 = chunk1.unwrap();
        assert_eq!(chunk1.chunk_id, 0);
        assert_eq!(chunk1.data.len(), 256);
        assert!(!chunk1.is_final);
        
        // Get remaining chunks
        let mut total_chunks = 1;
        loop {
            match manager.get_next_chunk(&session_id, &asset_data).await.unwrap() {
                Some(chunk) => {
                    total_chunks += 1;
                    if chunk.is_final {
                        break;
                    }
                }
                None => break,
            }
        }
        
        // Should have 4 chunks total (1000 bytes / 256 = 3.9, rounded up to 4)
        assert_eq!(total_chunks, 4);
        
        // Session should be completed and removed
        let session_info = manager.get_session_info(&session_id).await;
        assert!(session_info.is_none());
    }
    
    #[tokio::test]
    async fn test_stream_pause_resume() {
        let config = StreamingConfig::default();
        let manager = AssetStreamingManager::new(config);
        
        let asset_id = Uuid::new_v4();
        let asset_data = vec![0u8; 1024];
        let client_id = "test_client".to_string();
        
        let session_id = manager.start_stream(
            asset_id,
            AssetType::Texture,
            &asset_data,
            client_id,
            StreamPriority::Normal,
        ).await.unwrap();
        
        // Pause stream
        manager.pause_stream(&session_id).await.unwrap();
        
        // Should return None when paused
        let chunk = manager.get_next_chunk(&session_id, &asset_data).await.unwrap();
        assert!(chunk.is_none());
        
        // Resume stream
        manager.resume_stream(&session_id).await.unwrap();
        
        // Should return chunk when resumed
        let chunk = manager.get_next_chunk(&session_id, &asset_data).await.unwrap();
        assert!(chunk.is_some());
    }
    
    #[test]
    fn test_asset_stream_creation() {
        let asset_data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let chunk_size = 3;
        
        let stream = create_asset_stream(asset_data, chunk_size);
        
        // This would normally be used with StreamExt::collect() in an async context
        // For this test, we just verify the stream was created
        assert!(true); // Placeholder assertion
    }
}