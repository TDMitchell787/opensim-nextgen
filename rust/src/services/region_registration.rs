use anyhow::Result;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::region::config_parser::RegionIniConfig;
use crate::services::factory::ServiceContainer;
use crate::services::traits::{GridServiceTrait, RegionInfo};

pub async fn register_regions(
    container: &ServiceContainer,
    regions: &[RegionIniConfig],
) -> Result<()> {
    let grid_service = container.grid();

    for region in regions {
        let region_info = RegionInfo {
            region_id: region.uuid,
            region_name: region.name.clone(),
            region_loc_x: region.grid_x,
            region_loc_y: region.grid_y,
            region_size_x: region.size_x,
            region_size_y: region.size_y,
            server_ip: if region.external_host == "SYSTEMIP" {
                crate::config::login::resolve_system_ip()
            } else {
                region.external_host.clone()
            },
            server_port: region.internal_port as u16,
            server_uri: format!(
                "http://{}:{}",
                if region.external_host == "SYSTEMIP" {
                    crate::config::login::resolve_system_ip()
                } else {
                    region.external_host.clone()
                },
                region.internal_port
            ),
            region_flags: 0,
            scope_id: region.scope_id,
            owner_id: Uuid::nil(),
            estate_id: 1,
        };

        match grid_service.register_region(&region_info).await {
            Ok(true) => info!(
                "[GRID] Registered region '{}' ({}) at ({},{})",
                region.name, region.uuid, region.grid_x, region.grid_y
            ),
            Ok(false) => warn!(
                "[GRID] Registration returned false for region '{}'",
                region.name
            ),
            Err(e) => error!("[GRID] Failed to register region '{}': {}", region.name, e),
        }
    }

    Ok(())
}

pub async fn deregister_regions(container: &ServiceContainer, region_ids: &[Uuid]) -> Result<()> {
    let grid_service = container.grid();

    for region_id in region_ids {
        match grid_service.deregister_region(*region_id).await {
            Ok(true) => info!("[GRID] Deregistered region {}", region_id),
            Ok(false) => warn!(
                "[GRID] Deregistration returned false for region {}",
                region_id
            ),
            Err(e) => error!("[GRID] Failed to deregister region {}: {}", region_id, e),
        }
    }

    Ok(())
}
