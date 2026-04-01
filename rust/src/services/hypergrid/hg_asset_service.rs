use anyhow::Result;
use uuid::Uuid;
use tracing::{info, warn, debug};
use crate::services::traits::{AssetServiceTrait, AssetBase};

pub fn is_foreign_asset(asset_id: &str) -> bool {
    asset_id.contains('|') && (asset_id.starts_with("http://") || asset_id.starts_with("https://"))
}

pub fn parse_foreign_asset(asset_id: &str) -> Option<(String, Uuid)> {
    let pipe_pos = asset_id.find('|')?;
    let grid_uri = &asset_id[..pipe_pos];
    let local_id_str = &asset_id[pipe_pos + 1..];
    let local_id = Uuid::parse_str(local_id_str).ok()?;
    Some((grid_uri.to_string(), local_id))
}

pub async fn fetch_foreign_asset(grid_uri: &str, asset_id: Uuid) -> Result<AssetBase> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let url = format!("{}/assets/{}", grid_uri.trim_end_matches('/'), asset_id);
    debug!("Fetching foreign asset from {}", url);

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Foreign asset fetch failed: HTTP {}",
            response.status()
        ));
    }

    let data = response.bytes().await?.to_vec();

    info!("Fetched foreign asset {} ({} bytes) from {}", asset_id, data.len(), grid_uri);

    Ok(AssetBase {
        id: asset_id.to_string(),
        name: format!("Foreign asset {}", asset_id),
        description: format!("Fetched from {}", grid_uri),
        asset_type: -1,
        local: false,
        temporary: false,
        data,
        creator_id: Uuid::nil().to_string(),
        flags: 0,
    })
}

pub async fn get_asset_with_foreign_fallback(
    asset_id: &str,
    local_service: &dyn AssetServiceTrait,
) -> Result<Option<AssetBase>> {
    if is_foreign_asset(asset_id) {
        if let Some((grid_uri, local_id)) = parse_foreign_asset(asset_id) {
            if let Ok(Some(cached)) = local_service.get(&local_id.to_string()).await {
                debug!("Foreign asset {} found in local cache", local_id);
                return Ok(Some(cached));
            }

            match fetch_foreign_asset(&grid_uri, local_id).await {
                Ok(asset) => {
                    if let Err(e) = local_service.store(&asset).await {
                        warn!("Failed to cache foreign asset {}: {}", local_id, e);
                    }
                    return Ok(Some(asset));
                }
                Err(e) => {
                    warn!("Failed to fetch foreign asset {} from {}: {}", local_id, grid_uri, e);
                    return Ok(None);
                }
            }
        }
        return Ok(None);
    }

    local_service.get(asset_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_foreign_asset() {
        assert!(is_foreign_asset("http://grid.example.com:8002|00000000-0000-0000-0000-000000000001"));
        assert!(is_foreign_asset("https://secure.grid.com|abcdef01-2345-6789-abcd-ef0123456789"));
        assert!(!is_foreign_asset("00000000-0000-0000-0000-000000000001"));
        assert!(!is_foreign_asset("not-a-foreign-asset"));
    }

    #[test]
    fn test_parse_foreign_asset() {
        let (uri, id) = parse_foreign_asset("http://grid.example.com:8002|00000000-0000-0000-0000-000000000001").unwrap();
        assert_eq!(uri, "http://grid.example.com:8002");
        assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000001");

        assert!(parse_foreign_asset("not-foreign").is_none());
        assert!(parse_foreign_asset("http://grid.com|not-a-uuid").is_none());
    }
}
