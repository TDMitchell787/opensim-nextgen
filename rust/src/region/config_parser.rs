use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;
use anyhow::{Result, anyhow};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct RegionIniConfig {
    pub name: String,
    pub uuid: Uuid,
    pub grid_x: u32,
    pub grid_y: u32,
    pub size_x: u32,
    pub size_y: u32,
    pub internal_port: u16,
    pub internal_address: String,
    pub external_host: String,
    pub max_prims: u32,
    pub max_agents: u32,
    pub scope_id: Uuid,
    pub region_type: String,
    pub physics: String,
    pub meshing: String,
    pub water_height: f32,
}

impl RegionIniConfig {
    pub fn region_handle(&self) -> u64 {
        let world_x = (self.grid_x as u64) * 256;
        let world_y = (self.grid_y as u64) * 256;
        (world_x << 32) | world_y
    }

    pub fn region_x_meters(&self) -> u32 {
        self.grid_x * 256
    }

    pub fn region_y_meters(&self) -> u32 {
        self.grid_y * 256
    }

    pub fn is_varregion(&self) -> bool {
        self.size_x > 256 || self.size_y > 256
    }

    pub fn patches_per_edge_x(&self) -> u32 {
        self.size_x / 16
    }

    pub fn patches_per_edge_y(&self) -> u32 {
        self.size_y / 16
    }
}

pub fn parse_regions_ini(config_dir: &Path) -> Result<Vec<RegionIniConfig>> {
    let mut regions = Vec::new();

    if !config_dir.exists() {
        return Err(anyhow!("Regions config directory does not exist: {:?}", config_dir));
    }

    let entries = std::fs::read_dir(config_dir)?;
    let mut ini_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .map(|ext| ext.eq_ignore_ascii_case("ini"))
                .unwrap_or(false)
        })
        .collect();

    ini_files.sort_by_key(|e| e.path());

    if ini_files.is_empty() {
        return Err(anyhow!("No .ini files found in {:?}", config_dir));
    }

    for entry in &ini_files {
        let path = entry.path();
        let content = std::fs::read_to_string(&path)?;
        let mut file_regions = parse_ini_content(&content, &path)?;
        regions.append(&mut file_regions);
    }

    validate_regions(&regions)?;

    info!("Parsed {} region(s) from {:?}", regions.len(), config_dir);
    for region in &regions {
        info!(
            "  Region '{}' UUID={} grid=({},{}) port={} handle={}",
            region.name, region.uuid, region.grid_x, region.grid_y,
            region.internal_port, region.region_handle()
        );
    }

    Ok(regions)
}

fn parse_ini_content(content: &str, path: &Path) -> Result<Vec<RegionIniConfig>> {
    let mut regions = Vec::new();
    let mut current_section: Option<String> = None;
    let mut current_props: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            if let Some(section_name) = current_section.take() {
                if let Some(region) = build_region(&section_name, &current_props, path)? {
                    regions.push(region);
                }
            }
            current_section = Some(line[1..line.len() - 1].trim().to_string());
            current_props.clear();
            continue;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let mut value = line[eq_pos + 1..].trim().to_string();

            if let Some(comment_pos) = value.find(';') {
                value = value[..comment_pos].trim().to_string();
            }
            value = value.trim_matches('"').to_string();

            current_props.insert(key, value);
        }
    }

    if let Some(section_name) = current_section.take() {
        if let Some(region) = build_region(&section_name, &current_props, path)? {
            regions.push(region);
        }
    }

    Ok(regions)
}

fn build_region(
    section_name: &str,
    props: &HashMap<String, String>,
    path: &Path,
) -> Result<Option<RegionIniConfig>> {
    let uuid_str = match props.get("RegionUUID") {
        Some(s) => s,
        None => {
            warn!("Section [{}] in {:?} missing RegionUUID, skipping", section_name, path);
            return Ok(None);
        }
    };

    let uuid = Uuid::parse_str(uuid_str)
        .map_err(|e| anyhow!("Invalid UUID '{}' in section [{}]: {}", uuid_str, section_name, e))?;

    let (grid_x, grid_y) = parse_location(
        props.get("Location").map(|s| s.as_str()).unwrap_or("1000,1000"),
        section_name,
    )?;

    let internal_port: u16 = props.get("InternalPort")
        .map(|s| s.parse().unwrap_or(9000))
        .unwrap_or(9000);

    let name = props.get("RegionName")
        .cloned()
        .unwrap_or_else(|| section_name.to_string());

    let internal_address = props.get("InternalAddress")
        .cloned()
        .unwrap_or_else(|| "0.0.0.0".to_string());

    let external_host = props.get("ExternalHostName")
        .cloned()
        .unwrap_or_else(|| "SYSTEMIP".to_string());

    let max_prims: u32 = props.get("MaxPrims")
        .map(|s| s.parse().unwrap_or(45000))
        .unwrap_or(45000);

    let max_agents: u32 = props.get("MaxAgents")
        .map(|s| s.parse().unwrap_or(100))
        .unwrap_or(100);

    let scope_id = props.get("ScopeID")
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());

    let region_type = props.get("RegionType")
        .cloned()
        .unwrap_or_else(|| "Mainland".to_string());

    let physics = props.get("Physics")
        .cloned()
        .unwrap_or_else(|| "BulletSim".to_string());

    let meshing = props.get("Meshing")
        .cloned()
        .unwrap_or_else(|| "Meshmerizer".to_string());

    let size_x: u32 = props.get("SizeX")
        .map(|s| s.parse().unwrap_or(256))
        .unwrap_or(256);

    let size_y: u32 = props.get("SizeY")
        .map(|s| s.parse().unwrap_or(256))
        .unwrap_or(256);

    let water_height: f32 = props.get("WaterHeight")
        .map(|s| s.parse().unwrap_or(20.0))
        .unwrap_or(20.0);

    if size_x % 256 != 0 || size_y % 256 != 0 {
        return Err(anyhow!("Region '{}' SizeX/SizeY must be multiples of 256 (got {}x{})", section_name, size_x, size_y));
    }

    Ok(Some(RegionIniConfig {
        name,
        uuid,
        grid_x,
        grid_y,
        size_x,
        size_y,
        internal_port,
        internal_address,
        external_host,
        max_prims,
        max_agents,
        scope_id,
        region_type,
        physics,
        meshing,
        water_height,
    }))
}

fn parse_location(location: &str, section: &str) -> Result<(u32, u32)> {
    let parts: Vec<&str> = location.split(',').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid Location '{}' in section [{}]", location, section));
    }
    let x: u32 = parts[0].trim().parse()
        .map_err(|_| anyhow!("Invalid X coordinate in Location for section [{}]", section))?;
    let y: u32 = parts[1].trim().parse()
        .map_err(|_| anyhow!("Invalid Y coordinate in Location for section [{}]", section))?;
    Ok((x, y))
}

fn validate_regions(regions: &[RegionIniConfig]) -> Result<()> {
    let mut ports: HashMap<u16, &str> = HashMap::new();
    let mut positions: HashMap<(u32, u32), &str> = HashMap::new();

    for region in regions {
        if let Some(existing) = ports.get(&region.internal_port) {
            return Err(anyhow!(
                "Duplicate port {} used by regions '{}' and '{}'",
                region.internal_port, existing, region.name
            ));
        }
        ports.insert(region.internal_port, &region.name);

        let pos = (region.grid_x, region.grid_y);
        if let Some(existing) = positions.get(&pos) {
            return Err(anyhow!(
                "Duplicate grid position ({},{}) used by regions '{}' and '{}'",
                pos.0, pos.1, existing, region.name
            ));
        }
        positions.insert(pos, &region.name);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_ini() {
        let content = r#"
[TestRegion]
RegionUUID = 11111111-1111-1111-1111-111111111111
Location = 1000,1000
InternalPort = 9000
RegionName = TestRegion
ExternalHostName = SYSTEMIP
MaxPrims = 45000
MaxAgents = 200
"#;
        let regions = parse_ini_content(content, Path::new("test.ini")).unwrap();
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].name, "TestRegion");
        assert_eq!(regions[0].grid_x, 1000);
        assert_eq!(regions[0].grid_y, 1000);
        assert_eq!(regions[0].internal_port, 9000);
        assert_eq!(regions[0].region_x_meters(), 256000);
        assert_eq!(regions[0].region_y_meters(), 256000);
        assert_eq!(regions[0].region_handle(), (256000_u64 << 32) | 256000_u64);
        assert_eq!(regions[0].size_x, 256);
        assert_eq!(regions[0].size_y, 256);
        assert!(!regions[0].is_varregion());
        assert_eq!(regions[0].water_height, 20.0);
    }

    #[test]
    fn test_parse_varregion() {
        let content = r#"
[BigRegion]
RegionUUID = 33333333-3333-3333-3333-333333333333
Location = 2000,2000
InternalPort = 9516
SizeX = 8192
SizeY = 8192
WaterHeight = 0.0
"#;
        let regions = parse_ini_content(content, Path::new("test.ini")).unwrap();
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].size_x, 8192);
        assert_eq!(regions[0].size_y, 8192);
        assert!(regions[0].is_varregion());
        assert_eq!(regions[0].patches_per_edge_x(), 512);
        assert_eq!(regions[0].patches_per_edge_y(), 512);
        assert_eq!(regions[0].water_height, 0.0);
    }

    #[test]
    fn test_parse_multiple_regions() {
        let content = r#"
[Region1]
RegionUUID = 11111111-1111-1111-1111-111111111111
Location = 1000,1000
InternalPort = 9000

[Region2]
RegionUUID = 22222222-2222-2222-2222-222222222222
Location = 1001,1000
InternalPort = 9001
"#;
        let regions = parse_ini_content(content, Path::new("test.ini")).unwrap();
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].name, "Region1");
        assert_eq!(regions[1].name, "Region2");
        assert_eq!(regions[1].grid_x, 1001);
        assert_eq!(regions[1].internal_port, 9001);
    }

    #[test]
    fn test_duplicate_port_detection() {
        let regions = vec![
            RegionIniConfig {
                name: "R1".to_string(),
                uuid: Uuid::new_v4(),
                grid_x: 1000, grid_y: 1000,
                size_x: 256, size_y: 256,
                internal_port: 9000,
                internal_address: "0.0.0.0".to_string(),
                external_host: "SYSTEMIP".to_string(),
                max_prims: 45000, max_agents: 100,
                scope_id: Uuid::nil(),
                region_type: "Mainland".to_string(),
                physics: "BulletSim".to_string(),
                meshing: "Meshmerizer".to_string(),
                water_height: 20.0,
            },
            RegionIniConfig {
                name: "R2".to_string(),
                uuid: Uuid::new_v4(),
                grid_x: 1001, grid_y: 1000,
                size_x: 256, size_y: 256,
                internal_port: 9000,
                internal_address: "0.0.0.0".to_string(),
                external_host: "SYSTEMIP".to_string(),
                max_prims: 45000, max_agents: 100,
                scope_id: Uuid::nil(),
                region_type: "Mainland".to_string(),
                physics: "BulletSim".to_string(),
                meshing: "Meshmerizer".to_string(),
                water_height: 20.0,
            },
        ];
        assert!(validate_regions(&regions).is_err());
    }

    #[test]
    fn test_region_handle_for_grid_2000() {
        let config = RegionIniConfig {
            name: "Test".to_string(),
            uuid: Uuid::new_v4(),
            grid_x: 2000, grid_y: 2000,
            size_x: 256, size_y: 256,
            internal_port: 9000,
            internal_address: "0.0.0.0".to_string(),
            external_host: "SYSTEMIP".to_string(),
            max_prims: 45000, max_agents: 100,
            scope_id: Uuid::nil(),
            region_type: "Mainland".to_string(),
            physics: "BulletSim".to_string(),
            meshing: "Meshmerizer".to_string(),
            water_height: 20.0,
        };
        assert_eq!(config.region_x_meters(), 512000);
        assert_eq!(config.region_y_meters(), 512000);
        assert_eq!(config.region_handle(), (512000_u64 << 32) | 512000_u64);
    }

    #[test]
    fn test_comments_and_blank_lines() {
        let content = r#"
; This is a comment
# So is this

[TestRegion]
; Comment in section
RegionUUID = 11111111-1111-1111-1111-111111111111
Location = 1000,1000
InternalPort = 9000

"#;
        let regions = parse_ini_content(content, Path::new("test.ini")).unwrap();
        assert_eq!(regions.len(), 1);
    }
}
