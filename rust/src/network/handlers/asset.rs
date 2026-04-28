//! Handles asset-related messages

use anyhow::{anyhow, Result};
use bytes::Bytes;
use futures::stream;
use std::{collections::HashMap, sync::Arc};
use tokio_stream::{Stream, StreamExt};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    asset::{AssetManager, AssetMetadata, AssetType},
    network::{llsd::LLSDValue, session::Session},
};
use tokio::sync::RwLock;

/// Handles asset-related messages
#[derive(Default)]
pub struct AssetHandler;

impl AssetHandler {
    /// Handles asset upload requests from clients
    pub async fn handle_asset_upload(
        &self,
        session: Arc<RwLock<Session>>,
        asset_manager: Arc<AssetManager>,
        asset_data: Bytes,
        asset_type: AssetType,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String> {
        let session_guard = session.read().await;
        let session_id = session_guard.session_id.clone();
        drop(session_guard);

        info!("Handling asset upload for session: {}", session_id);

        // Generate unique asset ID
        let asset_id = Uuid::new_v4().to_string();

        // Validate asset data
        if asset_data.is_empty() {
            return Err(anyhow!("Asset data cannot be empty"));
        }

        // Check asset size limits
        const MAX_ASSET_SIZE: usize = 10 * 1024 * 1024; // 10MB
        if asset_data.len() > MAX_ASSET_SIZE {
            return Err(anyhow!(
                "Asset too large: {} bytes (max: {} bytes)",
                asset_data.len(),
                MAX_ASSET_SIZE
            ));
        }

        // Validate asset type based on data
        if let Err(e) = self.validate_asset_data(&asset_data, &asset_type) {
            warn!("Asset validation failed for session {}: {}", session_id, e);
            return Err(e);
        }

        // Store asset
        let asset_metadata = AssetMetadata {
            name: metadata
                .as_ref()
                .and_then(|m| m.get("name"))
                .cloned()
                .unwrap_or_default(),
            description: metadata
                .as_ref()
                .and_then(|m| m.get("description"))
                .cloned()
                .unwrap_or_default(),
            creator_id: Uuid::nil(), // Will be set from session if available
            content_type: asset_type.content_type().to_string(),
            ..Default::default()
        };
        let asset_uuid = Uuid::parse_str(&asset_id).unwrap_or_else(|_| Uuid::new_v4());
        asset_manager
            .store_asset(asset_uuid, asset_type, asset_data, asset_metadata, None)
            .await?;

        info!(
            "Successfully uploaded asset {} for session {}",
            asset_id, session_id
        );
        Ok(asset_id)
    }

    /// Handles asset download requests from clients
    pub async fn handle_asset_download(
        &self,
        session: Arc<RwLock<Session>>,
        asset_manager: Arc<AssetManager>,
        asset_id: &str,
    ) -> Result<Option<Bytes>> {
        let session_guard = session.read().await;
        let session_id = session_guard.session_id.clone();
        drop(session_guard);

        info!(
            "Handling asset download request for asset {} from session {}",
            asset_id, session_id
        );

        // Validate asset ID format
        if asset_id.is_empty() {
            return Err(anyhow!("Asset ID cannot be empty"));
        }

        // Retrieve asset
        let asset_uuid = Uuid::parse_str(asset_id)
            .map_err(|_| anyhow::anyhow!("Invalid asset ID: {}", asset_id))?;
        match asset_manager.get_asset(asset_uuid).await? {
            Some(asset) => {
                info!("Asset {} downloaded by session {}", asset_id, session_id);
                Ok(asset.data.as_ref().map(|data| (**data).clone()))
            }
            None => {
                warn!("Asset {} not found for session {}", asset_id, session_id);
                Ok(None)
            }
        }
    }

    /// Handles asset metadata requests
    pub async fn handle_asset_metadata(
        &self,
        session: Arc<RwLock<Session>>,
        asset_manager: Arc<AssetManager>,
        asset_id: &str,
    ) -> Result<Option<LLSDValue>> {
        let session_guard = session.read().await;
        let session_id = session_guard.session_id.clone();
        drop(session_guard);

        info!(
            "Handling asset metadata request for asset {} from session {}",
            asset_id, session_id
        );

        let asset_uuid = Uuid::parse_str(asset_id)
            .map_err(|_| anyhow::anyhow!("Invalid asset ID: {}", asset_id))?;
        if let Some(metadata) = asset_manager.get_metadata(asset_uuid).await? {
            let mut llsd_map = HashMap::new();
            llsd_map.insert("name".to_string(), LLSDValue::String(metadata.name));
            llsd_map.insert(
                "description".to_string(),
                LLSDValue::String(metadata.description),
            );
            llsd_map.insert(
                "content_type".to_string(),
                LLSDValue::String(metadata.content_type),
            );
            llsd_map.insert(
                "creator_id".to_string(),
                LLSDValue::UUID(metadata.creator_id),
            );
            Ok(Some(LLSDValue::Map(llsd_map)))
        } else {
            Ok(None)
        }
    }

    /// Handles chunked asset upload for large files
    pub async fn handle_chunked_asset_upload(
        &self,
        session: Arc<RwLock<Session>>,
        asset_manager: Arc<AssetManager>,
        chunk_data: Bytes,
        chunk_index: u32,
        total_chunks: u32,
        asset_id: Option<String>,
        asset_type: AssetType,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String> {
        let session_guard = session.read().await;
        let session_id = session_guard.session_id.clone();
        drop(session_guard);

        info!(
            "Handling chunked asset upload for session: {} (chunk {}/{})",
            session_id,
            chunk_index + 1,
            total_chunks
        );

        // Generate or use provided asset ID
        let final_asset_id = asset_id.unwrap_or_else(|| Uuid::new_v4().to_string());

        // Validate chunk data
        if chunk_data.is_empty() {
            return Err(anyhow!("Chunk data cannot be empty"));
        }

        // Store chunk temporarily
        asset_manager
            .store_asset_chunk(&final_asset_id, chunk_index, chunk_data)
            .await?;

        // If this is the last chunk, assemble the complete asset
        if chunk_index == total_chunks - 1 {
            info!(
                "Assembling complete asset {} from {} chunks",
                final_asset_id, total_chunks
            );

            let complete_data = asset_manager
                .assemble_asset_chunks(&final_asset_id, total_chunks)
                .await?;

            // Validate complete asset
            if let Err(e) = self.validate_asset_data(&complete_data, &asset_type) {
                warn!("Asset validation failed for session {}: {}", session_id, e);
                // Clean up chunks on validation failure
                asset_manager.cleanup_asset_chunks(&final_asset_id).await?;
                return Err(e);
            }

            // Store final asset
            let final_asset_uuid =
                Uuid::parse_str(&final_asset_id).unwrap_or_else(|_| Uuid::new_v4());
            let final_metadata = if let Some(meta_map) = metadata {
                AssetMetadata {
                    name: meta_map.get("name").cloned().unwrap_or_default(),
                    description: meta_map.get("description").cloned().unwrap_or_default(),
                    creator_id: Uuid::nil(),
                    content_type: asset_type.content_type().to_string(),
                    ..Default::default()
                }
            } else {
                AssetMetadata::default()
            };
            asset_manager
                .store_asset(
                    final_asset_uuid,
                    asset_type,
                    complete_data,
                    final_metadata,
                    None,
                )
                .await?;

            // Clean up temporary chunks
            asset_manager.cleanup_asset_chunks(&final_asset_id).await?;

            info!(
                "Successfully assembled and uploaded asset {} for session {}",
                final_asset_id, session_id
            );
        }

        Ok(final_asset_id)
    }

    /// Handles streaming asset download for large files
    pub async fn handle_streaming_asset_download(
        &self,
        session: Arc<RwLock<Session>>,
        asset_manager: Arc<AssetManager>,
        asset_id: &str,
        chunk_size: Option<usize>,
    ) -> Result<impl Stream<Item = Result<Bytes>>> {
        let session_guard = session.read().await;
        let session_id = session_guard.session_id.clone();
        drop(session_guard);

        info!(
            "Handling streaming asset download for asset {} from session {}",
            asset_id, session_id
        );

        // Validate asset ID format
        if asset_id.is_empty() {
            return Err(anyhow!("Asset ID cannot be empty"));
        }

        // Get asset size first
        let asset_uuid = Uuid::parse_str(asset_id)
            .map_err(|_| anyhow::anyhow!("Invalid asset ID: {}", asset_id))?;
        let asset = match asset_manager.get_asset(asset_uuid).await? {
            Some(asset) => asset,
            None => {
                warn!("Asset {} not found for session {}", asset_id, session_id);
                return Err(anyhow!("Asset not found"));
            }
        };

        let data = asset
            .data
            .as_ref()
            .ok_or_else(|| anyhow!("Asset {} has no data", asset_id))?
            .as_ref();
        let chunk_size = chunk_size.unwrap_or(64 * 1024); // Default 64KB chunks

        info!(
            "Streaming asset {} ({} bytes) in chunks of {} bytes for session {}",
            asset_id,
            data.len(),
            chunk_size,
            session_id
        );

        // Create streaming chunks
        let chunks: Vec<Bytes> = data
            .chunks(chunk_size)
            .map(|chunk| Bytes::copy_from_slice(chunk))
            .collect();

        let stream = stream::iter(chunks.into_iter().map(Ok));
        Ok(stream)
    }

    /// Handles progressive asset download with range requests
    pub async fn handle_progressive_asset_download(
        &self,
        session: Arc<RwLock<Session>>,
        asset_manager: Arc<AssetManager>,
        asset_id: &str,
        range_start: Option<usize>,
        range_end: Option<usize>,
    ) -> Result<(Bytes, AssetRange)> {
        let session_guard = session.read().await;
        let session_id = session_guard.session_id.clone();
        drop(session_guard);

        info!(
            "Handling progressive asset download for asset {} from session {} (range: {:?}-{:?})",
            asset_id, session_id, range_start, range_end
        );

        // Validate asset ID format
        if asset_id.is_empty() {
            return Err(anyhow!("Asset ID cannot be empty"));
        }

        // Retrieve asset
        let asset_uuid = Uuid::parse_str(asset_id)
            .map_err(|_| anyhow::anyhow!("Invalid asset ID: {}", asset_id))?;
        let asset = match asset_manager.get_asset(asset_uuid).await? {
            Some(asset) => asset,
            None => {
                warn!("Asset {} not found for session {}", asset_id, session_id);
                return Err(anyhow!("Asset not found"));
            }
        };

        let data = asset
            .data
            .as_ref()
            .ok_or_else(|| anyhow!("Asset {} has no data", asset_id))?;
        let total_size = data.len();

        // Calculate range
        let start = range_start.unwrap_or(0);
        let end = range_end.unwrap_or(total_size - 1).min(total_size - 1);

        if start > end || start >= total_size {
            return Err(anyhow!(
                "Invalid range: {}-{} for asset of size {}",
                start,
                end,
                total_size
            ));
        }

        let range_data = data.slice(start..=end);
        let range_info = AssetRange {
            start,
            end,
            total_size,
            content_length: range_data.len(),
        };

        info!(
            "Serving asset {} range {}-{} ({} bytes) for session {}",
            asset_id,
            start,
            end,
            range_data.len(),
            session_id
        );

        Ok((range_data, range_info))
    }

    /// Get asset information without downloading the content
    pub async fn handle_asset_info(
        &self,
        session: Arc<RwLock<Session>>,
        asset_manager: Arc<AssetManager>,
        asset_id: &str,
    ) -> Result<Option<AssetInfo>> {
        let session_guard = session.read().await;
        let session_id = session_guard.session_id.clone();
        drop(session_guard);

        info!(
            "Handling asset info request for asset {} from session {}",
            asset_id, session_id
        );

        let asset_uuid = Uuid::parse_str(asset_id)
            .map_err(|_| anyhow::anyhow!("Invalid asset ID: {}", asset_id))?;
        if let Some(asset) = asset_manager.get_asset(asset_uuid).await? {
            let info = AssetInfo {
                id: asset_id.to_string(),
                asset_type: asset.asset_type.clone(),
                size: asset.data.as_ref().map(|d| d.len()).unwrap_or(0),
                created_at: asset
                    .created_at
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                checksum: asset
                    .data
                    .as_ref()
                    .map(|d| calculate_checksum(d))
                    .unwrap_or_else(|| "".to_string()),
            };
            Ok(Some(info))
        } else {
            Ok(None)
        }
    }

    /// Validates asset data based on its type
    fn validate_asset_data(&self, data: &Bytes, asset_type: &AssetType) -> Result<()> {
        match asset_type {
            AssetType::Texture => {
                // Basic image format validation
                if data.len() < 8 {
                    return Err(anyhow!("Invalid texture data: too short"));
                }

                // Check for common image headers
                let header = &data[..8.min(data.len())];
                if header.starts_with(&[0x89, 0x50, 0x4E, 0x47]) // PNG
                    || header.starts_with(&[0xFF, 0xD8, 0xFF]) // JPEG
                    || header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a")
                // GIF
                {
                    Ok(())
                } else {
                    Err(anyhow!("Invalid texture format"))
                }
            }
            AssetType::Sound => {
                // Basic audio format validation
                if data.len() < 4 {
                    return Err(anyhow!("Invalid sound data: too short"));
                }

                let header = &data[..4.min(data.len())];
                if header.starts_with(b"RIFF") || header.starts_with(b"OggS") {
                    Ok(())
                } else {
                    Err(anyhow!("Invalid sound format"))
                }
            }
            AssetType::Script => {
                // Basic script validation - check if it's valid UTF-8
                match std::str::from_utf8(data) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(anyhow!("Script must be valid UTF-8 text")),
                }
            }
            _ => Ok(()), // Accept other types without validation for now
        }
    }
}

/// Information about an asset range request
#[derive(Debug, Clone)]
pub struct AssetRange {
    pub start: usize,
    pub end: usize,
    pub total_size: usize,
    pub content_length: usize,
}

/// Basic asset information without content
#[derive(Debug, Clone)]
pub struct AssetInfo {
    pub id: String,
    pub asset_type: AssetType,
    pub size: usize,
    pub created_at: u64,
    pub checksum: String,
}

/// Calculate SHA-256 checksum of asset data
fn calculate_checksum(data: &Bytes) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
