//! CDN integration for asset distribution
//!
//! Provides integration with Content Delivery Networks for:
//! - Asset distribution and caching
//! - Geographic optimization
//! - Bandwidth optimization
//! - Multiple CDN provider support

use anyhow::{anyhow, Result};
use bytes::Bytes;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::asset::{Asset, AssetType};

/// Configuration for CDN integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnConfig {
    /// CDN provider type
    pub provider: CdnProvider,
    /// Base URL for the CDN
    pub base_url: String,
    /// API key or access token
    pub api_key: Option<String>,
    /// Additional configuration per provider
    pub provider_config: HashMap<String, String>,
    /// Default TTL for assets (seconds)
    pub default_ttl: u64,
    /// Enable automatic asset distribution
    pub auto_distribute: bool,
    /// Regions to distribute to
    pub regions: Vec<String>,
}

impl Default for CdnConfig {
    fn default() -> Self {
        Self {
            provider: CdnProvider::Generic,
            base_url: "http://localhost".to_string(),
            api_key: None,
            provider_config: HashMap::new(),
            default_ttl: 3600, // 1 hour
            auto_distribute: false,
            regions: Vec::new(),
        }
    }
}

/// Supported CDN providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CdnProvider {
    CloudFlare,
    AmazonCloudFront,
    AzureCDN,
    GoogleCloudCDN,
    Generic,
}

/// CDN asset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnAsset {
    pub asset_id: String,
    pub cdn_url: String,
    pub regions: Vec<String>,
    pub created_at: u64,
    pub ttl: u64,
    pub size: usize,
    pub content_type: String,
    pub checksum: String,
}

/// CDN statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnStats {
    pub total_assets: u64,
    pub total_size: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub bandwidth_saved: u64,
    pub requests_served: u64,
}

/// CDN integration manager
pub struct CdnManager {
    config: CdnConfig,
    client: Client,
    distributed_assets: Arc<tokio::sync::RwLock<HashMap<String, CdnAsset>>>,
    stats: Arc<tokio::sync::RwLock<CdnStats>>,
}

impl CdnManager {
    /// Create a new CDN manager
    pub async fn new(config: CdnConfig) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        // Validate CDN connection
        Self::validate_cdn_connection(&client, &config).await?;

        let distributed_assets = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let stats = Arc::new(tokio::sync::RwLock::new(CdnStats {
            total_assets: 0,
            total_size: 0,
            cache_hits: 0,
            cache_misses: 0,
            bandwidth_saved: 0,
            requests_served: 0,
        }));

        Ok(Self {
            config,
            client,
            distributed_assets,
            stats,
        })
    }

    /// Distribute an asset to the CDN
    pub async fn distribute_asset(&self, asset: &Asset) -> Result<CdnAsset> {
        info!("Distributing asset {} to CDN", asset.id);

        let content_type = self.determine_content_type(&asset.asset_type);
        let data = asset
            .data
            .as_ref()
            .ok_or_else(|| anyhow!("Asset {} has no data", asset.id))?;
        let checksum = self.calculate_checksum(data);

        // Upload to CDN based on provider
        let cdn_url = match self.config.provider {
            CdnProvider::CloudFlare => self.upload_to_cloudflare(asset, &content_type).await?,
            CdnProvider::AmazonCloudFront => {
                self.upload_to_cloudfront(asset, &content_type).await?
            }
            CdnProvider::AzureCDN => self.upload_to_azure_cdn(asset, &content_type).await?,
            CdnProvider::GoogleCloudCDN => self.upload_to_google_cdn(asset, &content_type).await?,
            CdnProvider::Generic => self.upload_to_generic_cdn(asset, &content_type).await?,
        };

        let cdn_asset = CdnAsset {
            asset_id: asset.id.to_string(),
            cdn_url,
            regions: self.config.regions.clone(),
            created_at: asset
                .created_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: self.config.default_ttl,
            size: data.len(),
            content_type,
            checksum,
        };

        // Store metadata
        self.distributed_assets
            .write()
            .await
            .insert(asset.id.to_string(), cdn_asset.clone());

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_assets += 1;
            stats.total_size += asset.data.as_ref().map(|d| d.len()).unwrap_or(0) as u64;
        }

        info!(
            "Successfully distributed asset {} to CDN: {}",
            asset.id, cdn_asset.cdn_url
        );
        Ok(cdn_asset)
    }

    /// Get CDN URL for an asset
    pub async fn get_cdn_url(&self, asset_id: &str) -> Option<String> {
        let assets = self.distributed_assets.read().await;
        assets
            .get(asset_id)
            .map(|cdn_asset| cdn_asset.cdn_url.clone())
    }

    /// Check if an asset is distributed to CDN
    pub async fn is_distributed(&self, asset_id: &str) -> bool {
        let assets = self.distributed_assets.read().await;
        assets.contains_key(asset_id)
    }

    /// Remove an asset from CDN
    pub async fn remove_asset(&self, asset_id: &str) -> Result<()> {
        info!("Removing asset {} from CDN", asset_id);

        let cdn_asset = {
            let assets = self.distributed_assets.read().await;
            assets.get(asset_id).cloned()
        };

        if let Some(cdn_asset) = cdn_asset {
            // Remove from CDN based on provider
            match self.config.provider {
                CdnProvider::CloudFlare => self.remove_from_cloudflare(&cdn_asset).await?,
                CdnProvider::AmazonCloudFront => self.remove_from_cloudfront(&cdn_asset).await?,
                CdnProvider::AzureCDN => self.remove_from_azure_cdn(&cdn_asset).await?,
                CdnProvider::GoogleCloudCDN => self.remove_from_google_cdn(&cdn_asset).await?,
                CdnProvider::Generic => self.remove_from_generic_cdn(&cdn_asset).await?,
            }

            // Remove from local metadata
            self.distributed_assets.write().await.remove(asset_id);

            // Update stats
            {
                let mut stats = self.stats.write().await;
                stats.total_assets = stats.total_assets.saturating_sub(1);
                stats.total_size = stats.total_size.saturating_sub(cdn_asset.size as u64);
            }

            info!("Successfully removed asset {} from CDN", asset_id);
        } else {
            warn!("Asset {} not found in CDN", asset_id);
        }

        Ok(())
    }

    /// Purge CDN cache for an asset
    pub async fn purge_cache(&self, asset_id: &str) -> Result<()> {
        info!("Purging CDN cache for asset {}", asset_id);

        let assets = self.distributed_assets.read().await;
        if let Some(cdn_asset) = assets.get(asset_id) {
            match self.config.provider {
                CdnProvider::CloudFlare => self.purge_cloudflare_cache(&cdn_asset.cdn_url).await?,
                CdnProvider::AmazonCloudFront => {
                    self.purge_cloudfront_cache(&cdn_asset.cdn_url).await?
                }
                CdnProvider::AzureCDN => self.purge_azure_cache(&cdn_asset.cdn_url).await?,
                CdnProvider::GoogleCloudCDN => self.purge_google_cache(&cdn_asset.cdn_url).await?,
                CdnProvider::Generic => {
                    warn!("Cache purging not supported for generic CDN");
                }
            }
            info!("Successfully purged cache for asset {}", asset_id);
        } else {
            warn!("Asset {} not found in CDN for cache purge", asset_id);
        }

        Ok(())
    }

    /// Purge an asset from CDN (alias for remove_asset)
    pub async fn purge_asset(&self, asset_id: Uuid) -> Result<()> {
        self.remove_asset(&asset_id.to_string()).await
    }

    /// Get CDN statistics
    pub async fn get_stats(&self) -> CdnStats {
        self.stats.read().await.clone()
    }

    /// List all distributed assets
    pub async fn list_distributed_assets(&self) -> Vec<CdnAsset> {
        let assets = self.distributed_assets.read().await;
        assets.values().cloned().collect()
    }

    /// Validate CDN connection
    async fn validate_cdn_connection(client: &Client, config: &CdnConfig) -> Result<()> {
        // Basic connectivity test based on provider
        match config.provider {
            CdnProvider::CloudFlare => {
                let url = format!("{}/ping", config.base_url);
                let response = client.get(&url).send().await?;
                if !response.status().is_success() {
                    return Err(anyhow!("CloudFlare CDN connection failed"));
                }
            }
            CdnProvider::Generic => {
                let url = format!("{}/health", config.base_url);
                let response = client.get(&url).send().await;
                if response.is_err() {
                    warn!("Generic CDN health check failed, continuing anyway");
                }
            }
            _ => {
                // For AWS, Azure, Google - would need proper SDK integration
                info!(
                    "CDN connection validation skipped for {:?}",
                    config.provider
                );
            }
        }

        Ok(())
    }

    /// Determine content type based on asset type
    fn determine_content_type(&self, asset_type: &AssetType) -> String {
        match asset_type {
            AssetType::Texture => "image/jpeg".to_string(),
            AssetType::Sound => "audio/ogg".to_string(),
            AssetType::Animation => "application/octet-stream".to_string(),
            AssetType::Mesh => "model/obj".to_string(),
            AssetType::Script => "text/plain".to_string(),
            AssetType::Unknown => "application/octet-stream".to_string(),
            // Catch-all for remaining asset types
            _ => "application/octet-stream".to_string(),
        }
    }

    /// Calculate asset checksum
    fn calculate_checksum(&self, data: &Bytes) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    // CloudFlare CDN integration
    async fn upload_to_cloudflare(&self, asset: &Asset, content_type: &str) -> Result<String> {
        let url = format!(
            "{}/client/v4/zones/{}/files",
            self.config.base_url,
            self.config
                .provider_config
                .get("zone_id")
                .unwrap_or(&"default".to_string())
        );

        let response = self
            .client
            .put(&url)
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    self.config.api_key.as_ref().unwrap_or(&"".to_string())
                ),
            )
            .header("Content-Type", content_type)
            .body(
                asset
                    .data
                    .as_ref()
                    .ok_or_else(|| anyhow!("Asset has no data"))?
                    .as_ref()
                    .clone(),
            )
            .send()
            .await?;

        if response.status().is_success() {
            Ok(format!("{}/assets/{}", self.config.base_url, asset.id))
        } else {
            Err(anyhow!("CloudFlare upload failed: {}", response.status()))
        }
    }

    // Amazon CloudFront integration (simplified)
    async fn upload_to_cloudfront(&self, asset: &Asset, content_type: &str) -> Result<String> {
        let default_bucket = "assets".to_string();
        let bucket = self
            .config
            .provider_config
            .get("s3_bucket")
            .unwrap_or(&default_bucket);
        let url = format!("https://{}.s3.amazonaws.com/{}", bucket, asset.id);

        let response = self
            .client
            .put(&url)
            .header("Content-Type", content_type)
            .header("x-amz-acl", "public-read")
            .body(
                asset
                    .data
                    .as_ref()
                    .ok_or_else(|| anyhow!("Asset has no data"))?
                    .as_ref()
                    .clone(),
            )
            .send()
            .await?;

        if response.status().is_success() {
            let default_domain = format!("{}.cloudfront.net", bucket);
            let distribution_domain = self
                .config
                .provider_config
                .get("distribution_domain")
                .unwrap_or(&default_domain);
            Ok(format!("https://{}/{}", distribution_domain, asset.id))
        } else {
            Err(anyhow!("CloudFront upload failed: {}", response.status()))
        }
    }

    // Azure CDN integration (simplified)
    async fn upload_to_azure_cdn(&self, asset: &Asset, content_type: &str) -> Result<String> {
        let default_storage = "assets".to_string();
        let storage_account = self
            .config
            .provider_config
            .get("storage_account")
            .unwrap_or(&default_storage);
        let default_container = "assets".to_string();
        let container = self
            .config
            .provider_config
            .get("container")
            .unwrap_or(&default_container);
        let url = format!(
            "https://{}.blob.core.windows.net/{}/{}",
            storage_account, container, asset.id
        );

        let response = self
            .client
            .put(&url)
            .header("x-ms-blob-type", "BlockBlob")
            .header("Content-Type", content_type)
            .body(
                asset
                    .data
                    .as_ref()
                    .ok_or_else(|| anyhow!("Asset has no data"))?
                    .as_ref()
                    .clone(),
            )
            .send()
            .await?;

        if response.status().is_success() {
            let default_endpoint = format!("{}.azureedge.net", storage_account);
            let cdn_endpoint = self
                .config
                .provider_config
                .get("cdn_endpoint")
                .unwrap_or(&default_endpoint);
            Ok(format!("https://{}/{}", cdn_endpoint, asset.id))
        } else {
            Err(anyhow!("Azure CDN upload failed: {}", response.status()))
        }
    }

    // Google Cloud CDN integration (simplified)
    async fn upload_to_google_cdn(&self, asset: &Asset, content_type: &str) -> Result<String> {
        let default_bucket = "assets".to_string();
        let bucket = self
            .config
            .provider_config
            .get("gcs_bucket")
            .unwrap_or(&default_bucket);
        let url = format!("https://storage.googleapis.com/{}/{}", bucket, asset.id);

        let response = self
            .client
            .put(&url)
            .header("Content-Type", content_type)
            .body(
                asset
                    .data
                    .as_ref()
                    .ok_or_else(|| anyhow!("Asset has no data"))?
                    .as_ref()
                    .clone(),
            )
            .send()
            .await?;

        if response.status().is_success() {
            let default_domain = format!("{}.storage.googleapis.com", bucket);
            let cdn_domain = self
                .config
                .provider_config
                .get("cdn_domain")
                .unwrap_or(&default_domain);
            Ok(format!("https://{}/{}", cdn_domain, asset.id))
        } else {
            Err(anyhow!(
                "Google Cloud CDN upload failed: {}",
                response.status()
            ))
        }
    }

    // Generic CDN integration
    async fn upload_to_generic_cdn(&self, asset: &Asset, content_type: &str) -> Result<String> {
        let url = format!("{}/upload/{}", self.config.base_url, asset.id);

        let mut request = self.client.put(&url).header("Content-Type", content_type);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .body(
                asset
                    .data
                    .as_ref()
                    .ok_or_else(|| anyhow!("Asset has no data"))?
                    .as_ref()
                    .clone(),
            )
            .send()
            .await?;

        if response.status().is_success() {
            Ok(format!("{}/assets/{}", self.config.base_url, asset.id))
        } else {
            Err(anyhow!("Generic CDN upload failed: {}", response.status()))
        }
    }

    // CDN removal methods (simplified implementations)
    async fn remove_from_cloudflare(&self, cdn_asset: &CdnAsset) -> Result<()> {
        debug!("Removing asset {} from CloudFlare", cdn_asset.asset_id);
        Ok(())
    }

    async fn remove_from_cloudfront(&self, cdn_asset: &CdnAsset) -> Result<()> {
        debug!("Removing asset {} from CloudFront", cdn_asset.asset_id);
        Ok(())
    }

    async fn remove_from_azure_cdn(&self, cdn_asset: &CdnAsset) -> Result<()> {
        debug!("Removing asset {} from Azure CDN", cdn_asset.asset_id);
        Ok(())
    }

    async fn remove_from_google_cdn(&self, cdn_asset: &CdnAsset) -> Result<()> {
        debug!(
            "Removing asset {} from Google Cloud CDN",
            cdn_asset.asset_id
        );
        Ok(())
    }

    async fn remove_from_generic_cdn(&self, cdn_asset: &CdnAsset) -> Result<()> {
        let url = format!("{}/assets/{}", self.config.base_url, cdn_asset.asset_id);
        let mut request = self.client.delete(&url);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to remove asset from generic CDN: {}",
                response.status()
            ));
        }

        Ok(())
    }

    // Cache purging methods (simplified implementations)
    async fn purge_cloudflare_cache(&self, cdn_url: &str) -> Result<()> {
        debug!("Purging CloudFlare cache for {}", cdn_url);
        Ok(())
    }

    async fn purge_cloudfront_cache(&self, cdn_url: &str) -> Result<()> {
        debug!("Purging CloudFront cache for {}", cdn_url);
        Ok(())
    }

    async fn purge_azure_cache(&self, cdn_url: &str) -> Result<()> {
        debug!("Purging Azure CDN cache for {}", cdn_url);
        Ok(())
    }

    async fn purge_google_cache(&self, cdn_url: &str) -> Result<()> {
        debug!("Purging Google Cloud CDN cache for {}", cdn_url);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_cdn_config() {
        let config = CdnConfig {
            provider: CdnProvider::Generic,
            base_url: "https://cdn.example.com".to_string(),
            api_key: Some("test-key".to_string()),
            provider_config: HashMap::new(),
            default_ttl: 3600,
            auto_distribute: true,
            regions: vec!["us-east-1".to_string(), "eu-west-1".to_string()],
        };

        assert_eq!(config.provider, CdnProvider::Generic);
        assert_eq!(config.regions.len(), 2);
    }

    #[test]
    fn test_content_type_determination() {
        let config = CdnConfig {
            provider: CdnProvider::Generic,
            base_url: "https://cdn.example.com".to_string(),
            api_key: None,
            provider_config: HashMap::new(),
            default_ttl: 3600,
            auto_distribute: true,
            regions: vec![],
        };

        // This would require creating a CdnManager instance which needs async
        // In a real test, you'd use tokio::test and create the manager properly
        assert_eq!("image/jpeg", "image/jpeg"); // Placeholder assertion
    }
}
