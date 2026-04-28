use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use rand;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tracing::{error, info, warn};

/// Safely get current timestamp, handling system time before Unix epoch
fn safe_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| {
            tracing::warn!("System time is before UNIX epoch, using fallback timestamp");
            Duration::from_secs(0)
        })
        .as_secs()
}
use axum::{
    extract::State, http::StatusCode, middleware, response::IntoResponse, routing::get, Json,
    Router,
};
use axum_server;
use headers::{authorization::Bearer, Authorization, HeaderMapExt};
use std::env;
use std::net::SocketAddr;

fn load_config() {
    if let Some(instance_dir) = env::var("OPENSIM_INSTANCE_DIR")
        .ok()
        .filter(|s| !s.is_empty())
    {
        let instance_env = std::path::Path::new(&instance_dir).join(".env");
        if instance_env.exists() {
            if let Ok(_) = dotenvy::from_path(&instance_env) {
                println!("Loaded instance config from {}", instance_env.display());
            }
        } else {
            println!(
                "Warning: OPENSIM_INSTANCE_DIR set but no .env found at {}",
                instance_env.display()
            );
        }
        if env::var("OPENSIM_REGIONS_DIR").is_err() {
            let regions = std::path::Path::new(&instance_dir).join("Regions");
            env::set_var("OPENSIM_REGIONS_DIR", regions.to_string_lossy().as_ref());
        }
        if env::var("OPENSIM_BIN_DIR").is_err() {
            let bin = std::path::Path::new(&instance_dir).join("bin");
            env::set_var("OPENSIM_BIN_DIR", bin.to_string_lossy().as_ref());
        }
    } else {
        if let Ok(_) = dotenvy::from_filename(".env") {
            println!("Loaded configuration from .env file");
        }
    }

    if env::var("DATABASE_URL").is_err() {
        env::set_var("DATABASE_URL", "postgresql://opensim@localhost/opensim_pg");
        println!("Set default DATABASE_URL: postgresql://opensim@localhost/opensim_pg");
    }

    let current_dir = env::current_dir().unwrap_or_default();
    println!("Working directory: {}", current_dir.display());
    println!(
        "Database URL: {}",
        env::var("DATABASE_URL").unwrap_or_default()
    );
    if let Ok(instance) = env::var("OPENSIM_INSTANCE_DIR") {
        println!("Instance directory: {}", instance);
    }
}

mod xmlrpc_login_response;

use opensim_next::{
    archives::api::{create_archive_api_router, ArchiveApiState},
    asset::jpeg2000::{check_opj_compress_available, J2KEncoder},
    asset::{
        cache::AssetCache,
        cdn::CdnManager,
        load_default_assets,
        storage::{FileSystemStorage, StorageBackend},
        AssetManager, AssetManagerConfig,
    },
    caps::{CapsHandlerState, CapsManager},
    database::{
        multi_backend::{DatabaseConnection, MultiDatabaseConfig},
        DatabaseAdmin, DatabaseInitializer, DatabaseManager,
    },
    ffi::physics::PhysicsBridge,
    login_service::LoginService,
    login_session::{CircuitCodeRegistry, LoginSession},
    login_stage_tracker::LoginStageTracker,
    monitoring::MonitoringSystem,
    network::{
        admin_api::{
            create_admin_api_router, create_security_api_router, AdminApiState, SecurityApiState,
        },
        ai_api::{create_ai_api_router, initialize_ai_api},
        console_api::{create_console_api_router, ConsoleApiState},
        handlers::login::LoginServer,
        hypergrid::{HypergridConfig, HypergridManager},
        loopback::{LoopbackConfig, LoopbackConnector},
        session::SessionManager,
        skill_api::create_skill_api_router,
        web_client::WebClientServer,
        NetworkManager,
    },
    opensim_compatibility::animations::init_global_animations,
    opensim_compatibility::avatar_data::{set_global_avatar_data_manager, AvatarDataManager},
    opensim_compatibility::library_assets::{set_global_library_manager, LibraryAssetManager},
    region::terrain_storage::TerrainStorage,
    region::{config_parser, simulation::SimulationEngine, RegionConfig, RegionId, RegionManager},
    services::AvatarService,
    session::SessionManager as LoginSessionManager,
    setup::{SetupPreset, SetupWizard},
    state::StateManager,
};

#[derive(Parser)]
#[command(name = "opensim-next")]
#[command(about = "OpenSim Next - High-performance virtual world server")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the interactive setup wizard
    Setup {
        /// Use a preset configuration
        #[arg(long)]
        preset: Option<String>,
        /// Run in non-interactive mode
        #[arg(long)]
        non_interactive: bool,
        /// Reconfigure existing installation
        #[arg(long)]
        reconfigure: bool,
    },
    /// Start the OpenSim Next server
    Start {
        /// Server mode: standalone (default), grid (region server), robust (grid services server)
        #[arg(long, default_value = "standalone")]
        mode: String,
    },
    /// Run preflight checks for an instance
    Preflight {
        /// Instance directory name (under Instances/)
        #[arg(long)]
        instance: String,
    },
    /// Migrate assets from DB blobs to FSAssets filesystem storage
    MigrateFsassets {
        /// FSAssets root directory (default: ./fsassets)
        #[arg(long, default_value = "./fsassets")]
        fsassets_root: String,
        /// Batch size for reading from assets table
        #[arg(long, default_value = "100")]
        batch_size: i64,
        /// Verify migration after completion
        #[arg(long)]
        verify: bool,
    },
}

#[derive(Clone)]
struct AppState {
    config: MonitoringHttpConfig,
    monitoring: Arc<MonitoringSystem>,
    session_manager: Arc<SessionManager>,
    circuit_codes: CircuitCodeRegistry,
}

#[derive(Clone)]
struct LoginState {
    admin: Arc<DatabaseAdmin>,
    circuit_codes: CircuitCodeRegistry,
    caps_manager: Arc<CapsManager>,
}

#[derive(Clone)]
struct MonitoringHttpConfig {
    api_key: String,
    metrics_port: u16,
    instance_id: String,
}

impl MonitoringHttpConfig {
    fn from_env() -> Self {
        Self {
            api_key: env::var("OPENSIM_API_KEY")
                .unwrap_or_else(|_| "default-key-change-me".to_string()),
            metrics_port: env::var("OPENSIM_METRICS_PORT")
                .unwrap_or_else(|_| "9100".to_string())
                .parse()
                .unwrap_or(9100),
            instance_id: env::var("OPENSIM_INSTANCE_ID")
                .unwrap_or_else(|_| format!("instance-{}", safe_timestamp())),
        }
    }
}

// Authentication middleware
async fn auth_middleware(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    let auth = req.headers().typed_get::<Authorization<Bearer>>();
    if let Some(Authorization(bearer)) = auth {
        if bearer.token() == state.config.api_key {
            return Ok(next.run(req).await);
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let result = state.monitoring.export_prometheus().await;
    match result {
        Ok(metrics) => {
            let instance_metrics = format!(
                "# HELP opensim_instance_id Instance identifier\n\
                 # TYPE opensim_instance_id gauge\n\
                 opensim_instance_id{{instance=\"{}\"}} 1\n\n{}",
                state.config.instance_id, metrics
            );
            (
                [
                    ("Content-Type", "text/plain; version=0.0.4"),
                    ("Access-Control-Allow-Origin", "*"),
                    ("Access-Control-Allow-Methods", "GET, OPTIONS"),
                    (
                        "Access-Control-Allow-Headers",
                        "Authorization, Content-Type",
                    ),
                ],
                instance_metrics,
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to collect metrics",
        )
            .into_response(),
    }
}

async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let result = state.monitoring.get_health_status().await;
    match result {
        Ok(health_status) => {
            let response = serde_json::json!({
                "instance_id": state.config.instance_id,
                "status": format!("{:?}", health_status),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });
            (
                [
                    ("Access-Control-Allow-Origin", "*"),
                    ("Access-Control-Allow-Methods", "GET, OPTIONS"),
                    (
                        "Access-Control-Allow-Headers",
                        "Authorization, Content-Type",
                    ),
                ],
                Json(response),
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to get health status",
        )
            .into_response(),
    }
}

async fn instance_info_handler(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.monitoring.get_stats().await.ok();
    let system_metrics = state.monitoring.get_system_metrics().await.ok();
    let response = serde_json::json!({
        "instance_id": state.config.instance_id,
        "metrics_port": state.config.metrics_port,
        "uptime": system_metrics.as_ref().map(|m| m.uptime.as_secs()),
        "active_connections": system_metrics.as_ref().map(|m| m.network_connections),
        "active_regions": system_metrics.as_ref().map(|m| m.active_regions),
        "cpu_usage": system_metrics.as_ref().map(|m| m.cpu_usage),
        "memory_usage": system_metrics.as_ref().map(|m| m.memory_usage),
        "monitoring_stats": stats.map(|s| {
            serde_json::json!({
                "metrics_count": s.metrics_count,
                "health_status": format!("{:?}", s.health_status),
                "profiling_enabled": s.profiling_enabled
            })
        })
    });
    (
        [
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Methods", "GET, OPTIONS"),
            (
                "Access-Control-Allow-Headers",
                "Authorization, Content-Type",
            ),
        ],
        Json(response),
    )
}

async fn run_setup_wizard(
    preset: Option<String>,
    non_interactive: bool,
    reconfigure: bool,
) -> Result<()> {
    info!("Starting OpenSim Next Setup Wizard");

    if reconfigure {
        info!("Reconfiguring existing OpenSim Next installation");
        if std::path::Path::new("./config/OpenSim.ini").exists() {
            info!("Existing configuration found, will be overwritten");
        }
    }

    let mut wizard = SetupWizard::new();

    // Apply preset if specified
    if let Some(preset_name) = preset {
        let preset = match preset_name.as_str() {
            "standalone" => SetupPreset::Standalone,
            "grid-region" => SetupPreset::GridRegion,
            "grid-robust" => SetupPreset::GridRobust,
            "development" => SetupPreset::Development,
            "production" => SetupPreset::Production,
            _ => {
                error!("Invalid preset: {}. Valid presets: standalone, grid-region, grid-robust, development, production", preset_name);
                return Ok(());
            }
        };
        wizard = SetupWizard::with_preset(preset);
        info!("Using preset: {}", preset_name);
    }

    // Set non-interactive mode if requested
    if non_interactive {
        wizard = wizard.non_interactive();
        info!("Running in non-interactive mode");
    }

    // Run the setup wizard
    match wizard.run().await {
        Ok(setup_result) => match setup_result {
            opensim_next::setup::SetupResult::Success(_config) => {
                info!("✅ Setup completed successfully!");
                info!("Configuration files generated in ./config/");
                info!("You can now start the server with: cargo run start");
            }
            opensim_next::setup::SetupResult::Cancelled => {
                info!("Setup cancelled by user");
                std::process::exit(0);
            }
            opensim_next::setup::SetupResult::Error(error) => {
                error!("❌ Setup failed: {}", error);
                std::process::exit(1);
            }
        },
        Err(e) => {
            error!("❌ Setup wizard error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Robust database connection with PostgreSQL → MariaDB fallback
async fn try_database_connection_with_fallback(
    primary_config: &MultiDatabaseConfig,
) -> Result<DatabaseConnection> {
    // Try primary database (PostgreSQL)
    info!(
        "Attempting connection to primary database: {:?}",
        primary_config.database_type
    );
    match DatabaseConnection::new(primary_config).await {
        Ok(conn) => {
            // Test the connection to ensure it's working
            match conn.test_connection().await {
                Ok(_) => {
                    info!(
                        "✅ Primary database connection successful: {:?}",
                        primary_config.database_type
                    );
                    return Ok(conn);
                }
                Err(e) => {
                    warn!(
                        "Primary database connection test failed: {}. Attempting fallback...",
                        e
                    );
                }
            }
        }
        Err(e) => {
            warn!(
                "Primary database connection failed: {}. Attempting fallback...",
                e
            );
        }
    }

    // Fallback to MariaDB
    let fallback_config = MultiDatabaseConfig::mysql(
        "localhost",
        3306,
        "opensim_mariadb",
        "opensim",
        "password123",
    );
    info!(
        "Attempting fallback database connection: {:?}",
        fallback_config.database_type
    );

    match DatabaseConnection::new(&fallback_config).await {
        Ok(conn) => match conn.test_connection().await {
            Ok(_) => {
                warn!(
                    "✅ Fallback database connection successful: {:?}",
                    fallback_config.database_type
                );
                Ok(conn)
            }
            Err(e) => {
                error!(
                    "❌ Fallback database connection test failed: {}. No database available!",
                    e
                );
                Err(anyhow!(
                    "Both primary and fallback database connections failed"
                ))
            }
        },
        Err(e) => {
            error!(
                "❌ Fallback database connection failed: {}. No database available!",
                e
            );
            Err(anyhow!(
                "Both primary and fallback database connections failed"
            ))
        }
    }
}

async fn generate_map_tile(
    heightmap: &[f32],
    tile_uuid: uuid::Uuid,
    db_conn: &Arc<DatabaseConnection>,
) -> Result<usize> {
    use image::{Rgba, RgbaImage};

    let mut img = RgbaImage::new(256, 256);

    for y in 0u32..256 {
        for x in 0u32..256 {
            let height = heightmap[(y * 256 + x) as usize];
            let (r, g, b) = if height < 20.0 {
                (72u8, 116u8, 166u8)
            } else if height < 25.0 {
                (210, 180, 140)
            } else if height < 40.0 {
                (34, 139, 34)
            } else if height < 60.0 {
                (107, 142, 35)
            } else {
                (139, 137, 137)
            };
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    let dynamic_img = image::DynamicImage::ImageRgba8(img);
    let j2c_data = J2KEncoder::new()
        .with_quality(75)
        .encode(&dynamic_img)
        .map_err(|e| anyhow!("J2K encode failed: {}", e))?;
    let j2c_len = j2c_data.len();

    let tile_uuid_str = tile_uuid.to_string();
    match &**db_conn {
        DatabaseConnection::PostgreSQL(pool) => {
            sqlx::query(
                r#"INSERT INTO assets (id, assettype, name, description, data, create_time, local, temporary)
                   VALUES ($1::uuid, 0, $2, 'Map tile', $3, EXTRACT(EPOCH FROM NOW())::bigint, 1, 0)
                   ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data, create_time = EXTRACT(EPOCH FROM NOW())::bigint"#
            )
            .bind(&tile_uuid_str)
            .bind(format!("Map Tile {}", tile_uuid_str))
            .bind(&j2c_data)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to store map tile asset: {}", e))?;
        }
        DatabaseConnection::MySQL(pool) => {
            sqlx::query(
                r#"INSERT INTO assets (id, assetType, name, description, data, create_time, local, temporary)
                   VALUES (?, 0, ?, 'Map tile', ?, UNIX_TIMESTAMP(), 1, 0)
                   ON DUPLICATE KEY UPDATE data = VALUES(data), create_time = UNIX_TIMESTAMP()"#
            )
            .bind(&tile_uuid_str)
            .bind(format!("Map Tile {}", tile_uuid_str))
            .bind(&j2c_data)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to store map tile asset: {}", e))?;
        }
    }

    Ok(j2c_len)
}

#[derive(Clone)]
struct MapTileState {
    region_registry: Arc<Vec<opensim_next::udp::RegionInfo>>,
    database_connection: Arc<opensim_next::database::DatabaseConnection>,
}

async fn handle_map_tile_request(
    State(state): State<MapTileState>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    let raw = path.strip_prefix("map-").unwrap_or(&path);
    let parts: Vec<&str> = raw.split('-').collect();
    if parts.len() < 3 {
        return (StatusCode::NOT_FOUND, Vec::new()).into_response();
    }

    let zoom: u32 = match parts[0].parse() {
        Ok(z) => z,
        Err(_) => return (StatusCode::NOT_FOUND, Vec::new()).into_response(),
    };
    let grid_x: u32 = match parts[1].parse() {
        Ok(x) => x,
        Err(_) => return (StatusCode::NOT_FOUND, Vec::new()).into_response(),
    };
    let grid_y: u32 = match parts[2].parse() {
        Ok(y) => y,
        Err(_) => return (StatusCode::NOT_FOUND, Vec::new()).into_response(),
    };

    if zoom != 1 {
        return (StatusCode::NOT_FOUND, Vec::new()).into_response();
    }

    let service_mode =
        env::var("OPENSIM_SERVICE_MODE").unwrap_or_else(|_| "standalone".to_string());
    let region = state
        .region_registry
        .iter()
        .find(|r| r.grid_x == grid_x && r.grid_y == grid_y && r.service_mode == service_mode);

    let map_image_id = match region {
        Some(r) if !r.map_image_id.is_nil() => r.map_image_id,
        _ => return (StatusCode::NOT_FOUND, Vec::new()).into_response(),
    };

    let tile_id_str = map_image_id.to_string();
    let j2k_data: Option<Vec<u8>> = match &*state.database_connection {
        opensim_next::database::DatabaseConnection::PostgreSQL(pool) => {
            sqlx::query_scalar::<_, Vec<u8>>("SELECT data FROM assets WHERE id = $1::uuid")
                .bind(&tile_id_str)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
        }
        opensim_next::database::DatabaseConnection::MySQL(pool) => {
            sqlx::query_scalar::<_, Vec<u8>>("SELECT data FROM assets WHERE id = ?")
                .bind(&tile_id_str)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
        }
    };

    let j2k_data = match j2k_data {
        Some(d) if !d.is_empty() => d,
        _ => return (StatusCode::NOT_FOUND, Vec::new()).into_response(),
    };

    let codec = opensim_next::asset::jpeg2000::J2KCodec::default();
    let rgba = match codec.decode_to_rgba(&j2k_data) {
        Ok(img) => img,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Vec::new()).into_response(),
    };

    let dynamic = image::DynamicImage::ImageRgba8(rgba);
    let mut jpeg_buf = std::io::Cursor::new(Vec::new());
    if dynamic
        .write_to(&mut jpeg_buf, image::ImageFormat::Jpeg)
        .is_err()
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, Vec::new()).into_response();
    }

    (
        StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, "image/jpeg"),
            (axum::http::header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        jpeg_buf.into_inner(),
    )
        .into_response()
}

#[derive(Clone)]
struct MapItemsState {
    avatar_states: Arc<
        parking_lot::RwLock<
            std::collections::HashMap<uuid::Uuid, opensim_next::udp::AvatarMovementState>,
        >,
    >,
    parcels: Arc<parking_lot::RwLock<Vec<opensim_next::modules::land::Parcel>>>,
    region_registry: Arc<Vec<opensim_next::udp::RegionInfo>>,
}

async fn handle_map_items_request(
    State(state): State<MapItemsState>,
    axum::extract::Path(handle_str): axum::extract::Path<String>,
) -> impl IntoResponse {
    let region_handle: u64 = match handle_str.parse() {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                [(axum::http::header::CONTENT_TYPE, "text/plain")],
                String::new(),
            )
                .into_response()
        }
    };

    let region_global_x = ((region_handle >> 32) & 0xFFFFFFFF) as u32;
    let region_global_y = (region_handle & 0xFFFFFFFF) as u32;

    let service_mode =
        env::var("OPENSIM_SERVICE_MODE").unwrap_or_else(|_| "standalone".to_string());
    let is_our_region = state.region_registry.iter().any(|r| {
        r.service_mode == service_mode
            && r.grid_x == region_global_x / 256
            && r.grid_y == region_global_y / 256
    });

    if !is_our_region {
        return (
            StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "text/plain")],
            String::new(),
        )
            .into_response();
    }

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<llsd>\n<map>\n");

    {
        let states = state.avatar_states.read();
        let count = states.len() as i32;
        xml.push_str("<key>6</key>\n<array>\n");
        if count > 0 {
            xml.push_str(&format!(
                "<map><key>X</key><integer>{}</integer><key>Y</key><integer>{}</integer><key>ID</key><uuid>00000000-0000-0000-0000-000000000000</uuid><key>Name</key><string></string><key>Extra</key><integer>{}</integer><key>Extra2</key><integer>0</integer></map>\n",
                region_global_x + 128, region_global_y + 128, count
            ));
        }
        xml.push_str("</array>\n");
    }

    {
        let parcels = state.parcels.read();
        xml.push_str("<key>7</key>\n<array>\n");
        for parcel in parcels.iter() {
            if parcel.sale_price > 0 && (parcel.flags & 0x4000) != 0 {
                let px = region_global_x + parcel.landing_point[0] as u32;
                let py = region_global_y + parcel.landing_point[1] as u32;
                let escaped_name = parcel
                    .name
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                xml.push_str(&format!(
                    "<map><key>X</key><integer>{}</integer><key>Y</key><integer>{}</integer><key>ID</key><uuid>{}</uuid><key>Name</key><string>{}</string><key>Extra</key><integer>{}</integer><key>Extra2</key><integer>{}</integer></map>\n",
                    px, py, parcel.uuid, escaped_name, parcel.area, parcel.sale_price
                ));
            }
        }
        xml.push_str("</array>\n");
    }

    xml.push_str("<key>1</key>\n<array>\n</array>\n");
    xml.push_str("</map>\n</llsd>");

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/xml+llsd")],
        xml,
    )
        .into_response()
}

async fn run_preflight(instance: &str) -> Result<()> {
    let instance_dir = format!("Instances/{}", instance);
    let instance_path = std::path::Path::new(&instance_dir);

    println!("=== Preflight Check: {} ===", instance);
    let mut pass = 0u32;
    let mut fail = 0u32;

    // 1. Instance directory exists
    if instance_path.exists() {
        println!("[PASS] Instance directory exists: {}", instance_dir);
        pass += 1;
    } else {
        println!("[FAIL] Instance directory not found: {}", instance_dir);
        fail += 1;
        println!("\n=== Results: {} pass, {} fail ===", pass, fail);
        return Ok(());
    }

    // 2. .env file exists
    let env_file = instance_path.join(".env");
    if env_file.exists() {
        println!("[PASS] .env file found");
        pass += 1;
        if let Ok(_) = dotenvy::from_path(&env_file) {
            println!("  Loaded env vars from {}", env_file.display());
        }
    } else {
        println!("[FAIL] .env file not found at {}", env_file.display());
        fail += 1;
    }

    // 3. Regions directory
    let regions_dir = instance_path.join("Regions");
    if regions_dir.exists() {
        let count = std::fs::read_dir(&regions_dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext.eq_ignore_ascii_case("ini"))
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0);
        println!("[PASS] Regions directory found ({} .ini files)", count);
        pass += 1;
    } else {
        println!("[FAIL] Regions directory not found");
        fail += 1;
    }

    // 4. Database connectivity
    let db_url = env::var("DATABASE_URL").unwrap_or_default();
    if !db_url.is_empty() {
        println!("[INFO] DATABASE_URL: {}", db_url);
        match opensim_next::database::multi_backend::MultiDatabaseConfig::from_env() {
            Ok(config) => match try_database_connection_with_fallback(&config).await {
                Ok(conn) => {
                    if conn.test_connection().await.is_ok() {
                        println!("[PASS] Database connection successful");
                        pass += 1;
                    } else {
                        println!("[FAIL] Database connection test failed");
                        fail += 1;
                    }
                }
                Err(e) => {
                    println!("[FAIL] Database connection failed: {}", e);
                    fail += 1;
                }
            },
            Err(e) => {
                println!("[FAIL] Database config error: {}", e);
                fail += 1;
            }
        }
    } else {
        println!("[FAIL] DATABASE_URL not set");
        fail += 1;
    }

    // 5. Port availability
    let ports_to_check: Vec<(u16, &str)> = vec![
        (
            env::var("OPENSIM_LOGIN_PORT")
                .unwrap_or_else(|_| "9000".to_string())
                .parse()
                .unwrap_or(9000),
            "Login/HTTP",
        ),
        (
            env::var("OPENSIM_ROBUST_PORT")
                .unwrap_or_else(|_| "8003".to_string())
                .parse()
                .unwrap_or(8003),
            "Robust",
        ),
        (
            env::var("OPENSIM_METRICS_PORT")
                .unwrap_or_else(|_| "9100".to_string())
                .parse()
                .unwrap_or(9100),
            "Metrics",
        ),
    ];

    for (port, name) in &ports_to_check {
        match std::net::TcpListener::bind(format!("0.0.0.0:{}", port)) {
            Ok(_) => {
                println!("[PASS] Port {} ({}) available", port, name);
                pass += 1;
            }
            Err(_) => {
                println!("[WARN] Port {} ({}) already in use", port, name);
            }
        }
    }

    // 6. Service mode
    let mode = env::var("OPENSIM_SERVICE_MODE").unwrap_or_else(|_| "standalone".to_string());
    println!("[INFO] Service mode: {}", mode);

    // 7. Hypergrid config
    let hg = env::var("OPENSIM_HYPERGRID_ENABLED").unwrap_or_default();
    if hg == "true" {
        println!("[INFO] Hypergrid enabled");
        if let Ok(home) = env::var("OPENSIM_HOME_URI") {
            println!("[INFO]   Home URI: {}", home);
        }
        if let Ok(gk) = env::var("OPENSIM_GATEKEEPER_URI") {
            println!("[INFO]   Gatekeeper URI: {}", gk);
        }
    }

    println!("\n=== Results: {} pass, {} fail ===", pass, fail);
    if fail == 0 {
        println!("All preflight checks passed! Ready to start.");
    } else {
        println!("Some checks failed. Review and fix before starting.");
    }
    Ok(())
}

async fn start_server_with_mode(mode: &str) -> Result<()> {
    if mode == "grid" {
        info!("Starting OpenSim Next Server in GRID mode (region server)...");
        let robust_url =
            env::var("OPENSIM_ROBUST_URL").unwrap_or_else(|_| "http://localhost:8003".to_string());
        info!("  Robust URL: {}", robust_url);
        info!("  Remote services will be used via Robust HTTP connections");
        env::set_var("OPENSIM_ROBUST_URL", &robust_url);
    } else {
        info!("Starting OpenSim Next Server in STANDALONE mode...");
    }
    start_server().await
}

async fn start_robust_mode() -> Result<()> {
    info!("Starting OpenSim Next in ROBUST mode (grid services server)...");
    info!("  No region/UDP services - HTTP service endpoints only");

    let db_config = match opensim_next::database::multi_backend::MultiDatabaseConfig::from_env() {
        Ok(config) => config,
        Err(e) => {
            warn!(
                "Failed to load database configuration: {}. Using defaults.",
                e
            );
            opensim_next::database::multi_backend::MultiDatabaseConfig::mysql(
                "localhost",
                3306,
                "opensim_mariadb",
                "opensim",
                "password123",
            )
        }
    };

    let conn = try_database_connection_with_fallback(&db_config).await?;
    let conn = Arc::new(conn);

    if let Err(e) = conn.test_connection().await {
        return Err(anyhow!("Database connection test failed: {}", e));
    }
    if let Err(e) = conn.migrate().await {
        warn!("Database migration warning: {}", e);
    }
    info!("Robust mode: Database connected and migrated");

    let service_config = opensim_next::services::traits::ServiceConfig {
        mode: opensim_next::services::traits::ServiceMode::Standalone,
        ..Default::default()
    };

    let container =
        opensim_next::services::factory::ServiceContainer::new(service_config, Some(conn.clone()))?;
    info!("Robust mode: ServiceContainer created with local services");

    let avatar_service: Arc<dyn opensim_next::services::traits::AvatarServiceTrait> = {
        let db_manager =
            opensim_next::database::DatabaseManager::with_connection(conn.clone()).await?;
        Arc::new(opensim_next::services::AvatarService::new(Arc::new(
            db_manager,
        )))
    };

    let hg_config = opensim_next::services::config_parser::build_hypergrid_config();
    let (gatekeeper_service, uas_service): (
        Option<Arc<dyn opensim_next::services::traits::GatekeeperServiceTrait>>,
        Option<Arc<dyn opensim_next::services::traits::UserAgentServiceTrait>>,
    ) = if hg_config.enabled {
        let local_uas: Arc<dyn opensim_next::services::traits::UserAgentServiceTrait> =
            Arc::new(opensim_next::services::local::LocalUserAgentService::new(
                container.grid_service.clone(),
                container.user_account_service.clone(),
                container.presence_service.clone(),
                conn.clone(),
                hg_config.home_uri.clone(),
                hg_config.external_uri.clone(),
                hg_config.external_robust_uri.clone(),
                hg_config.grid_name.clone(),
            ));
        let local_gk: Arc<dyn opensim_next::services::traits::GatekeeperServiceTrait> = Arc::new(
            opensim_next::services::local::LocalGatekeeperService::new(
                container.grid_service.clone(),
                container.presence_service.clone(),
                conn.clone(),
                hg_config.gatekeeper_uri.clone(),
            )
            .with_config(
                hg_config.allow_teleports_to_any_region,
                hg_config.foreign_agents_allowed,
            )
            .with_user_account_service(container.user_account_service.clone())
            .with_local_uas(local_uas.clone()),
        );
        info!(
            "Robust mode: Hypergrid services enabled (gk={}, uas={})",
            hg_config.gatekeeper_uri, hg_config.home_uri
        );
        (Some(local_gk), Some(local_uas))
    } else {
        info!("Robust mode: Hypergrid services disabled");
        (None, None)
    };

    let hg_inventory_service: Option<
        std::sync::Arc<dyn opensim_next::services::traits::InventoryServiceTrait>,
    > = if gatekeeper_service.is_some() {
        let suitcase_svc = opensim_next::services::hypergrid::HGSuitcaseInventoryService::new(
            container.inventory_service.clone(),
        );
        info!("[HG] Suitcase inventory service enabled for foreign grid requests");
        Some(std::sync::Arc::new(suitcase_svc))
    } else {
        None
    };

    let griduser_service: Arc<dyn opensim_next::services::traits::GridUserServiceTrait> = Arc::new(
        opensim_next::services::local::LocalGridUserService::new(conn.clone()),
    );
    let agentprefs_service: Arc<dyn opensim_next::services::traits::AgentPrefsServiceTrait> =
        Arc::new(opensim_next::services::local::LocalAgentPrefsService::new(
            conn.clone(),
        ));
    let hg_friends_service: Option<Arc<dyn opensim_next::services::traits::HGFriendsServiceTrait>> =
        if let Some(pg) = conn.postgres_pool() {
            Some(Arc::new(
                opensim_next::services::hypergrid::hg_friends::HGFriendsService::new(Arc::new(
                    pg.clone(),
                )),
            ))
        } else {
            None
        };

    let instance_dir = env::var("OPENSIM_INSTANCE_DIR").unwrap_or_else(|_| ".".to_string());
    let bakes_dir = format!("{}/bakes", instance_dir);

    let mutelist_service: Option<Arc<dyn opensim_next::services::traits::MuteListServiceTrait>> =
        Some(Arc::new(
            opensim_next::services::local::LocalMuteListService::new(conn.clone()),
        ));
    let estate_service: Option<Arc<dyn opensim_next::services::traits::EstateServiceTrait>> =
        Some(Arc::new(
            opensim_next::services::local::LocalEstateService::new(conn.clone()),
        ));
    let map_tiles_dir = format!("{}/maptiles", instance_dir);
    let map_service: Option<Arc<dyn opensim_next::services::traits::MapImageServiceTrait>> =
        Some(Arc::new(
            opensim_next::services::local::LocalMapImageService::new(map_tiles_dir),
        ));

    let authorization_service: Option<
        Arc<dyn opensim_next::services::traits::AuthorizationServiceTrait>,
    > = Some(Arc::new(
        opensim_next::services::local::LocalAuthorizationService::new(),
    ));
    let friends_service: Option<Arc<dyn opensim_next::services::traits::FriendsServiceTrait>> =
        Some(Arc::new(
            opensim_next::services::local::LocalFriendsService::new(conn.clone()),
        ));
    let land_service: Option<Arc<dyn opensim_next::services::traits::LandServiceTrait>> =
        Some(Arc::new(
            opensim_next::services::local::LocalLandService::new(conn.clone()),
        ));
    let offlineim_service: Option<Arc<dyn opensim_next::services::traits::OfflineIMServiceTrait>> =
        Some(Arc::new(
            opensim_next::services::local::LocalOfflineIMService::new(conn.clone()),
        ));
    let profiles_service: Option<Arc<dyn opensim_next::services::traits::ProfilesServiceTrait>> =
        Some(Arc::new(
            opensim_next::services::local::LocalProfilesService::new(conn.clone()),
        ));

    let robust_state = opensim_next::services::robust::RobustState {
        grid_service: container.grid_service.clone(),
        user_account_service: container.user_account_service.clone(),
        auth_service: container.authentication_service.clone(),
        asset_service: container.asset_service.clone(),
        inventory_service: container.inventory_service.clone(),
        presence_service: container.presence_service.clone(),
        avatar_service,
        gatekeeper_service,
        uas_service,
        hg_inventory_service,
        griduser_service: Some(griduser_service),
        agentprefs_service: Some(agentprefs_service),
        hg_friends_service,
        bakes_dir: Some(bakes_dir),
        mutelist_service,
        estate_service,
        map_service,
        authorization_service,
        friends_service,
        land_service,
        offlineim_service,
        profiles_service,
        db_pool: conn.postgres_pool().cloned(),
    };

    let robust_port: u16 = env::var("OPENSIM_ROBUST_PORT")
        .unwrap_or_else(|_| "8003".to_string())
        .parse()
        .unwrap_or(8003);

    let admin_port: u16 = env::var("OPENSIM_ADMIN_PORT")
        .unwrap_or_else(|_| "9200".to_string())
        .parse()
        .unwrap_or(9200);

    info!("Robust services:");
    info!("  Grid:        POST http://0.0.0.0:{}/grid", robust_port);
    info!(
        "  UserAccount: POST http://0.0.0.0:{}/accounts",
        robust_port
    );
    info!("  Auth:        POST http://0.0.0.0:{}/auth", robust_port);
    info!("  Asset:       POST http://0.0.0.0:{}/assets", robust_port);
    info!(
        "  Inventory:   POST http://0.0.0.0:{}/inventory",
        robust_port
    );
    info!(
        "  HG Inventory:POST http://0.0.0.0:{}/hg/xinventory (suitcase-restricted)",
        robust_port
    );
    info!(
        "  Presence:    POST http://0.0.0.0:{}/presence",
        robust_port
    );
    info!("  Avatar:      POST http://0.0.0.0:{}/avatar", robust_port);
    info!(
        "  GridUser:    POST http://0.0.0.0:{}/griduser",
        robust_port
    );
    info!(
        "  AgentPrefs:  POST http://0.0.0.0:{}/agentprefs",
        robust_port
    );
    info!(
        "  Bakes:       GET/POST http://0.0.0.0:{}/bakes/:id",
        robust_port
    );
    info!(
        "  MuteList:    POST http://0.0.0.0:{}/mutelist",
        robust_port
    );
    info!(
        "  Estates:     GET/POST http://0.0.0.0:{}/estates",
        robust_port
    );
    info!("  Map:         GET/POST http://0.0.0.0:{}/map", robust_port);
    info!(
        "  Authorization: POST http://0.0.0.0:{}/authorization",
        robust_port
    );
    info!("  Friends:     POST http://0.0.0.0:{}/friends", robust_port);
    info!("  Land:        POST http://0.0.0.0:{}/land", robust_port);
    info!(
        "  OfflineIM:   POST http://0.0.0.0:{}/offlineim",
        robust_port
    );
    info!(
        "  Neighbours:  POST http://0.0.0.0:{}/region/:id",
        robust_port
    );
    info!(
        "  Profiles:    POST http://0.0.0.0:{}/user_profile_rpc (JsonRpc)",
        robust_port
    );
    info!(
        "  Freeswitch:  POST http://0.0.0.0:{}/fsapi/* (stub)",
        robust_port
    );

    let robust_handle = tokio::spawn(async move {
        if let Err(e) =
            opensim_next::services::robust::start_robust_server(robust_port, robust_state).await
        {
            error!("Robust server error: {}", e);
        }
    });

    info!("OpenSim Next ROBUST server started on port {}", robust_port);
    info!("  Admin API: http://0.0.0.0:{}", admin_port);

    let robust_instance_dir = opensim_next::instance_manager::discovery::resolve_instance_dir();
    let robust_discovery_info = opensim_next::instance_manager::RunningInstanceInfo::new(
        env::var("OPENSIM_INSTANCE_ID").unwrap_or_else(|_| {
            robust_instance_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
                .replace(' ', "-")
        }),
        "robust".to_string(),
        0,
        0,
        robust_port,
    );
    if let Err(e) = opensim_next::instance_manager::write_discovery_file(
        &robust_instance_dir,
        "robust",
        &robust_discovery_info,
    ) {
        warn!("Failed to write robust discovery file: {}", e);
    }

    if let Ok(controller_url) = env::var("OPENSIM_CONTROLLER_URL") {
        let instance_id = env::var("OPENSIM_INSTANCE_ID").unwrap_or_else(|_| {
            env::var("OPENSIM_INSTANCE_DIR")
                .map(|d| {
                    std::path::Path::new(&d)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase()
                        .replace(' ', "-")
                })
                .unwrap_or_else(|_| "robust".to_string())
        });
        let announce_robust_port = robust_port;

        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let announcement = serde_json::json!({
                "instance_id": instance_id,
                "service_mode": "robust",
                "ports": { "login": announce_robust_port, "admin": announce_robust_port },
                "region_count": 0u32,
                "capabilities": ["grid", "auth", "asset", "inventory", "presence", "avatar"],
                "version": env!("CARGO_PKG_VERSION"),
                "host": "localhost",
            });

            for attempt in 0..3u32 {
                match client
                    .post(format!("{}/api/instance/announce", controller_url))
                    .json(&announcement)
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => {
                        info!("Announced to controller at {}", controller_url);
                        break;
                    }
                    Ok(resp) => warn!(
                        "Controller announce failed (attempt {}): HTTP {}",
                        attempt + 1,
                        resp.status()
                    ),
                    Err(e) => warn!(
                        "Controller announce failed (attempt {}): {}",
                        attempt + 1,
                        e
                    ),
                }
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
            }

            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                let heartbeat = serde_json::json!({
                    "instance_id": instance_id,
                    "status": "running",
                    "active_users": 0u32,
                    "active_regions": 0u32,
                    "uptime_seconds": 0u64,
                    "cpu_usage": 0.0f64,
                    "memory_usage_mb": 0u64,
                });
                let _ = client
                    .post(format!("{}/api/instance/heartbeat", controller_url))
                    .json(&heartbeat)
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await;
            }
        });
    }

    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");
    opensim_next::instance_manager::remove_discovery_file(&robust_instance_dir, "robust");
    robust_handle.abort();
    info!("OpenSim Next ROBUST server shutdown complete");
    Ok(())
}

async fn start_controller_mode() -> Result<()> {
    info!("Starting OpenSim Next in CONTROLLER mode (instance management)...");
    info!("  No region/login/UDP/database services — management plane only");

    let config = opensim_next::instance_manager::config_loader::load_default_instances_config()
        .unwrap_or_else(|e| {
            warn!("Failed to load instances.toml: {}. Using defaults.", e);
            opensim_next::instance_manager::config_loader::InstancesConfig {
                controller:
                    opensim_next::instance_manager::config_loader::ControllerConfig::default(),
                instances: Vec::new(),
            }
        });

    let controller_port = config.controller.controller_port;
    let instances_base_dir = std::path::PathBuf::from(&config.controller.instances_base_dir);
    let binary_path = if config.controller.binary_path.is_empty() {
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("opensim-next"))
    } else {
        std::path::PathBuf::from(&config.controller.binary_path)
    };

    let registry = Arc::new(opensim_next::instance_manager::InstanceRegistry::new(
        config,
    ));
    let process_manager = Arc::new(opensim_next::instance_manager::ProcessManager::new(
        binary_path,
        instances_base_dir.clone(),
    ));

    let discovered = process_manager.scan_directories();
    info!(
        "Discovered {} instance directories in {}",
        discovered.len(),
        instances_base_dir.display()
    );
    for d in &discovered {
        info!(
            "  - {} ({}) [mode={}, port={}]",
            d.name, d.id, d.service_mode, d.login_port
        );
        if !registry.has_instance(&d.id).await {
            let inst_config = opensim_next::instance_manager::InstanceConfig {
                id: d.id.clone(),
                name: d.name.clone(),
                description: format!("{} instance (discovered)", d.service_mode),
                host: "localhost".to_string(),
                websocket_port: 0,
                admin_port: 0,
                metrics_port: 0,
                http_port: d.login_port,
                udp_port: d.login_port,
                api_key: "discovered".to_string(),
                environment: opensim_next::instance_manager::Environment::Development,
                auto_connect: false,
                tags: vec![d.service_mode.clone(), "discovered".to_string()],
                authentication: Default::default(),
                tls: Default::default(),
            };
            if let Err(e) = registry.add_instance(inst_config).await {
                warn!("Failed to register discovered instance {}: {}", d.id, e);
            }
            if let Err(e) = registry
                .update_status(
                    &d.id,
                    opensim_next::instance_manager::InstanceStatus::Discovered,
                )
                .await
            {
                warn!("Failed to set Discovered status: {}", e);
            }
        }
    }

    let controller_state = Arc::new(
        opensim_next::instance_manager::announcement::ControllerState {
            registry: registry.clone(),
            process_manager: process_manager.clone(),
            controller_port,
        },
    );

    let pm_for_routes = process_manager.clone();
    let reg_for_routes = registry.clone();

    let app = axum::Router::new()
        .route("/health", axum::routing::get(opensim_next::instance_manager::announcement::handle_health))
        .route("/api/health", axum::routing::get(opensim_next::instance_manager::announcement::handle_api_health))
        .route("/api/info", axum::routing::get(opensim_next::instance_manager::announcement::handle_api_info))
        .route("/api/instances", axum::routing::get(opensim_next::instance_manager::announcement::handle_list_instances))
        .route("/api/instance-dirs", axum::routing::get(opensim_next::instance_manager::announcement::handle_list_instance_dirs))
        .route("/api/running", axum::routing::get(opensim_next::instance_manager::announcement::handle_list_running))
        .route("/api/instance/announce", axum::routing::post(opensim_next::instance_manager::announcement::handle_announce))
        .route("/api/instance/heartbeat", axum::routing::post(opensim_next::instance_manager::announcement::handle_heartbeat))
        .route("/api/instance/:id/start", axum::routing::post({
            let pm = pm_for_routes.clone();
            let reg = reg_for_routes.clone();
            let port = controller_port;
            move |axum::extract::Path(id): axum::extract::Path<String>| {
                let pm = pm.clone();
                let reg = reg.clone();
                async move {
                    let controller_url = format!("http://localhost:{}", port);
                    let instance_dir = {
                        let dirs = pm.scan_directories();
                        dirs.iter().find(|d| d.id == id).map(|d| std::path::PathBuf::from(&d.path))
                    };
                    match instance_dir {
                        Some(dir) => {
                            let _ = reg.update_status(&id, opensim_next::instance_manager::InstanceStatus::Starting).await;
                            match pm.spawn_instance(&id, &dir, &controller_url).await {
                                Ok(pid) => axum::Json(serde_json::json!({
                                    "success": true, "message": format!("Instance {} started (PID {})", id, pid), "pid": pid
                                })),
                                Err(e) => {
                                    let _ = reg.update_status(&id, opensim_next::instance_manager::InstanceStatus::Error).await;
                                    axum::Json(serde_json::json!({ "success": false, "error": e.to_string() }))
                                }
                            }
                        }
                        None => axum::Json(serde_json::json!({ "success": false, "error": format!("Instance directory not found for {}", id) })),
                    }
                }
            }
        }))
        .route("/api/instance/:id/stop", axum::routing::post({
            let pm = pm_for_routes.clone();
            let reg = reg_for_routes.clone();
            move |axum::extract::Path(id): axum::extract::Path<String>| {
                let pm = pm.clone();
                let reg = reg.clone();
                async move {
                    let _ = reg.update_status(&id, opensim_next::instance_manager::InstanceStatus::Stopping).await;
                    match pm.stop_instance(&id, true).await {
                        Ok(()) => {
                            let _ = reg.update_status(&id, opensim_next::instance_manager::InstanceStatus::Stopped).await;
                            axum::Json(serde_json::json!({ "success": true, "message": format!("Instance {} stopped", id) }))
                        }
                        Err(e) => axum::Json(serde_json::json!({ "success": false, "error": e.to_string() })),
                    }
                }
            }
        }))
        .route("/api/instance/:id/restart", axum::routing::post({
            let pm = pm_for_routes.clone();
            let reg = reg_for_routes.clone();
            let port = controller_port;
            move |axum::extract::Path(id): axum::extract::Path<String>| {
                let pm = pm.clone();
                let reg = reg.clone();
                async move {
                    let controller_url = format!("http://localhost:{}", port);
                    let instance_dir = {
                        let dirs = pm.scan_directories();
                        dirs.iter().find(|d| d.id == id).map(|d| std::path::PathBuf::from(&d.path))
                    };
                    match instance_dir {
                        Some(dir) => {
                            let _ = reg.update_status(&id, opensim_next::instance_manager::InstanceStatus::Starting).await;
                            match pm.restart_instance(&id, &dir, &controller_url).await {
                                Ok(pid) => axum::Json(serde_json::json!({
                                    "success": true, "message": format!("Instance {} restarted (PID {})", id, pid), "pid": pid
                                })),
                                Err(e) => axum::Json(serde_json::json!({ "success": false, "error": e.to_string() })),
                            }
                        }
                        None => axum::Json(serde_json::json!({ "success": false, "error": format!("Instance directory not found for {}", id) })),
                    }
                }
            }
        }))
        .route("/api/instance/:id/console", axum::routing::get({
            let pm = pm_for_routes.clone();
            move |axum::extract::Path(id): axum::extract::Path<String>| {
                let pm = pm.clone();
                async move {
                    let entries = pm.get_console(&id, 200).await;
                    axum::Json(serde_json::json!({ "entries": entries, "count": entries.len() }))
                }
            }
        }))
        .route("/api/admin/set-level", axum::routing::post(opensim_next::instance_manager::access_control::handle_set_level))
        .route("/api/admin/users", axum::routing::get(opensim_next::instance_manager::access_control::handle_list_users))
        .route("/ws", axum::routing::get({
            let reg = reg_for_routes.clone();
            let pm = process_manager.clone();
            move |ws: axum::extract::ws::WebSocketUpgrade| {
                let reg = reg.clone();
                let pm = pm.clone();
                async move {
                    ws.on_upgrade(move |socket| controller_ws_handler(socket, reg, pm))
                }
            }
        }))
        .with_state(controller_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], controller_port));
    info!("=== Controller Mode Endpoints ===");
    info!(
        "  Health:        GET  http://0.0.0.0:{}/health",
        controller_port
    );
    info!(
        "  Instances:     GET  http://0.0.0.0:{}/api/instances",
        controller_port
    );
    info!(
        "  Directories:   GET  http://0.0.0.0:{}/api/instance-dirs",
        controller_port
    );
    info!(
        "  Running:       GET  http://0.0.0.0:{}/api/running",
        controller_port
    );
    info!(
        "  Start:         POST http://0.0.0.0:{}/api/instance/:id/start",
        controller_port
    );
    info!(
        "  Stop:          POST http://0.0.0.0:{}/api/instance/:id/stop",
        controller_port
    );
    info!(
        "  Restart:       POST http://0.0.0.0:{}/api/instance/:id/restart",
        controller_port
    );
    info!(
        "  Console:       GET  http://0.0.0.0:{}/api/instance/:id/console",
        controller_port
    );
    info!(
        "  Announce:      POST http://0.0.0.0:{}/api/instance/announce",
        controller_port
    );
    info!(
        "  Heartbeat:     POST http://0.0.0.0:{}/api/instance/heartbeat",
        controller_port
    );
    info!("  WebSocket:     ws://0.0.0.0:{}/ws", controller_port);
    info!(
        "  Set Level:     POST http://0.0.0.0:{}/api/admin/set-level",
        controller_port
    );
    info!(
        "  List Users:    GET  http://0.0.0.0:{}/api/admin/users",
        controller_port
    );

    let mut process_rx = process_manager.subscribe();
    let reg_for_events = registry.clone();
    tokio::spawn(async move {
        while let Ok(event) = process_rx.recv().await {
            match &event {
                opensim_next::instance_manager::process_manager::ProcessEvent::Spawned {
                    id,
                    pid,
                } => {
                    info!("Process event: {} spawned (PID {})", id, pid);
                }
                opensim_next::instance_manager::process_manager::ProcessEvent::Exited {
                    id,
                    pid,
                    exit_code,
                } => {
                    info!(
                        "Process event: {} exited (PID {}, code={:?})",
                        id, pid, exit_code
                    );
                    let _ = reg_for_events
                        .update_status(id, opensim_next::instance_manager::InstanceStatus::Stopped)
                        .await;
                    let _ = reg_for_events.update_connected(id, false).await;
                }
                opensim_next::instance_manager::process_manager::ProcessEvent::StdoutLine {
                    id,
                    line,
                } => {
                    tracing::trace!("[{}] {}", id, line);
                }
                opensim_next::instance_manager::process_manager::ProcessEvent::StderrLine {
                    id,
                    line,
                } => {
                    tracing::trace!("[{}/err] {}", id, line);
                }
            }
        }
    });

    info!(
        "OpenSim Next CONTROLLER server starting on port {}",
        controller_port
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    info!("OpenSim Next CONTROLLER server shutdown complete");
    Ok(())
}

/// Spawn the controller management plane as a background task within an existing instance.
/// Default ON unless OPENSIM_DISABLE_CONTROLLER=true. Auto-selects port from 9300-9320 range.
/// The controller router binds to its own port alongside the instance's normal services.
async fn spawn_embedded_controller(controller_port: u16) {
    info!(
        "Starting EMBEDDED controller on port {} (same process)...",
        controller_port
    );

    let config = opensim_next::instance_manager::config_loader::load_default_instances_config()
        .unwrap_or_else(|e| {
            warn!("Failed to load instances.toml: {}. Using defaults.", e);
            opensim_next::instance_manager::config_loader::InstancesConfig {
                controller:
                    opensim_next::instance_manager::config_loader::ControllerConfig::default(),
                instances: Vec::new(),
            }
        });

    let instances_base_dir =
        opensim_next::instance_manager::discovery::resolve_instances_base_dir();
    let binary_path = if config.controller.binary_path.is_empty() {
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("opensim-next"))
    } else {
        std::path::PathBuf::from(&config.controller.binary_path)
    };

    let registry = Arc::new(opensim_next::instance_manager::InstanceRegistry::new(
        config,
    ));
    let process_manager = Arc::new(opensim_next::instance_manager::ProcessManager::new(
        binary_path,
        instances_base_dir.clone(),
    ));

    let discovered = process_manager.scan_directories();
    info!(
        "Embedded controller: instances_base_dir={}, discovered {} directories",
        instances_base_dir.display(),
        discovered.len()
    );
    for d in &discovered {
        info!(
            "  - {} ({}) [mode={}, port={}]",
            d.name, d.id, d.service_mode, d.login_port
        );
        if !registry.has_instance(&d.id).await {
            let inst_config = opensim_next::instance_manager::InstanceConfig {
                id: d.id.clone(),
                name: d.name.clone(),
                description: format!("{} instance (discovered)", d.service_mode),
                host: "localhost".to_string(),
                websocket_port: 0,
                admin_port: 0,
                metrics_port: 0,
                http_port: d.login_port,
                udp_port: d.login_port,
                api_key: "discovered".to_string(),
                environment: opensim_next::instance_manager::Environment::Development,
                auto_connect: false,
                tags: vec![d.service_mode.clone(), "discovered".to_string()],
                authentication: Default::default(),
                tls: Default::default(),
            };
            let _ = registry.add_instance(inst_config).await;
            let _ = registry
                .update_status(
                    &d.id,
                    opensim_next::instance_manager::InstanceStatus::Discovered,
                )
                .await;
        }
    }

    // Register THIS instance as Running (it's already started — we're inside it)
    let self_id = env::var("OPENSIM_INSTANCE_ID").unwrap_or_else(|_| {
        env::var("OPENSIM_INSTANCE_DIR")
            .map(|d| {
                std::path::Path::new(&d)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase()
                    .replace(' ', "-")
            })
            .unwrap_or_else(|_| "self".to_string())
    });
    let self_mode = env::var("OPENSIM_SERVICE_MODE").unwrap_or_else(|_| "standalone".to_string());
    let self_port = env::var("OPENSIM_LOGIN_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(9000);
    if !registry.has_instance(&self_id).await {
        let self_config = opensim_next::instance_manager::InstanceConfig {
            id: self_id.clone(),
            name: self_id.clone(),
            description: format!("{} (embedded host)", self_mode),
            host: "localhost".to_string(),
            websocket_port: 0,
            admin_port: 0,
            metrics_port: 0,
            http_port: self_port,
            udp_port: self_port,
            api_key: "embedded".to_string(),
            environment: opensim_next::instance_manager::Environment::Development,
            auto_connect: false,
            tags: vec![self_mode, "embedded-host".to_string()],
            authentication: Default::default(),
            tls: Default::default(),
        };
        let _ = registry.add_instance(self_config).await;
    }
    let _ = registry
        .update_status(
            &self_id,
            opensim_next::instance_manager::InstanceStatus::Running,
        )
        .await;
    let _ = registry.update_connected(&self_id, true).await;

    let controller_state = Arc::new(
        opensim_next::instance_manager::announcement::ControllerState {
            registry: registry.clone(),
            process_manager: process_manager.clone(),
            controller_port,
        },
    );

    let pm_for_routes = process_manager.clone();
    let reg_for_routes = registry.clone();

    let app = axum::Router::new()
        .route("/health", axum::routing::get(opensim_next::instance_manager::announcement::handle_health))
        .route("/api/health", axum::routing::get(opensim_next::instance_manager::announcement::handle_api_health))
        .route("/api/info", axum::routing::get(opensim_next::instance_manager::announcement::handle_api_info))
        .route("/api/instances", axum::routing::get(opensim_next::instance_manager::announcement::handle_list_instances))
        .route("/api/instance-dirs", axum::routing::get(opensim_next::instance_manager::announcement::handle_list_instance_dirs))
        .route("/api/running", axum::routing::get(opensim_next::instance_manager::announcement::handle_list_running))
        .route("/api/instance/announce", axum::routing::post(opensim_next::instance_manager::announcement::handle_announce))
        .route("/api/instance/heartbeat", axum::routing::post(opensim_next::instance_manager::announcement::handle_heartbeat))
        .route("/api/instance/:id/start", axum::routing::post({
            let pm = pm_for_routes.clone();
            let reg = reg_for_routes.clone();
            let port = controller_port;
            move |axum::extract::Path(id): axum::extract::Path<String>| {
                let pm = pm.clone();
                let reg = reg.clone();
                async move {
                    let controller_url = format!("http://localhost:{}", port);
                    let instance_dir = {
                        let dirs = pm.scan_directories();
                        dirs.iter().find(|d| d.id == id).map(|d| std::path::PathBuf::from(&d.path))
                    };
                    match instance_dir {
                        Some(dir) => {
                            let _ = reg.update_status(&id, opensim_next::instance_manager::InstanceStatus::Starting).await;
                            match pm.spawn_instance(&id, &dir, &controller_url).await {
                                Ok(pid) => axum::Json(serde_json::json!({
                                    "success": true, "message": format!("Instance {} started (PID {})", id, pid), "pid": pid
                                })),
                                Err(e) => {
                                    let _ = reg.update_status(&id, opensim_next::instance_manager::InstanceStatus::Error).await;
                                    axum::Json(serde_json::json!({ "success": false, "error": e.to_string() }))
                                }
                            }
                        }
                        None => axum::Json(serde_json::json!({ "success": false, "error": format!("Instance directory not found for {}", id) })),
                    }
                }
            }
        }))
        .route("/api/instance/:id/stop", axum::routing::post({
            let pm = pm_for_routes.clone();
            let reg = reg_for_routes.clone();
            move |axum::extract::Path(id): axum::extract::Path<String>| {
                let pm = pm.clone();
                let reg = reg.clone();
                async move {
                    let _ = reg.update_status(&id, opensim_next::instance_manager::InstanceStatus::Stopping).await;
                    match pm.stop_instance(&id, true).await {
                        Ok(()) => {
                            let _ = reg.update_status(&id, opensim_next::instance_manager::InstanceStatus::Stopped).await;
                            axum::Json(serde_json::json!({ "success": true, "message": format!("Instance {} stopped", id) }))
                        }
                        Err(e) => axum::Json(serde_json::json!({ "success": false, "error": e.to_string() })),
                    }
                }
            }
        }))
        .route("/api/instance/:id/restart", axum::routing::post({
            let pm = pm_for_routes.clone();
            let reg = reg_for_routes.clone();
            let port = controller_port;
            move |axum::extract::Path(id): axum::extract::Path<String>| {
                let pm = pm.clone();
                let reg = reg.clone();
                async move {
                    let controller_url = format!("http://localhost:{}", port);
                    let instance_dir = {
                        let dirs = pm.scan_directories();
                        dirs.iter().find(|d| d.id == id).map(|d| std::path::PathBuf::from(&d.path))
                    };
                    match instance_dir {
                        Some(dir) => {
                            let _ = reg.update_status(&id, opensim_next::instance_manager::InstanceStatus::Starting).await;
                            match pm.restart_instance(&id, &dir, &controller_url).await {
                                Ok(pid) => axum::Json(serde_json::json!({
                                    "success": true, "message": format!("Instance {} restarted (PID {})", id, pid), "pid": pid
                                })),
                                Err(e) => axum::Json(serde_json::json!({ "success": false, "error": e.to_string() })),
                            }
                        }
                        None => axum::Json(serde_json::json!({ "success": false, "error": format!("Instance directory not found for {}", id) })),
                    }
                }
            }
        }))
        .route("/api/instance/:id/console", axum::routing::get({
            let pm = pm_for_routes.clone();
            move |axum::extract::Path(id): axum::extract::Path<String>| {
                let pm = pm.clone();
                async move {
                    let entries = pm.get_console(&id, 200).await;
                    axum::Json(serde_json::json!({ "entries": entries, "count": entries.len() }))
                }
            }
        }))
        .route("/api/admin/set-level", axum::routing::post(opensim_next::instance_manager::access_control::handle_set_level))
        .route("/api/admin/users", axum::routing::get(opensim_next::instance_manager::access_control::handle_list_users))
        .route("/ws", axum::routing::get({
            let reg = reg_for_routes.clone();
            let pm = process_manager.clone();
            move |ws: axum::extract::ws::WebSocketUpgrade| {
                let reg = reg.clone();
                let pm = pm.clone();
                async move {
                    ws.on_upgrade(move |socket| controller_ws_handler(socket, reg, pm))
                }
            }
        }))
        .with_state(controller_state);

    info!(
        "=== Embedded Controller Endpoints (port {}) ===",
        controller_port
    );
    info!(
        "  Health:     GET  http://0.0.0.0:{}/health",
        controller_port
    );
    info!(
        "  Instances:  GET  http://0.0.0.0:{}/api/instances",
        controller_port
    );
    info!(
        "  Running:    GET  http://0.0.0.0:{}/api/running",
        controller_port
    );
    info!(
        "  Start:      POST http://0.0.0.0:{}/api/instance/:id/start",
        controller_port
    );
    info!(
        "  Stop:       POST http://0.0.0.0:{}/api/instance/:id/stop",
        controller_port
    );
    info!(
        "  Console:    GET  http://0.0.0.0:{}/api/instance/:id/console",
        controller_port
    );
    info!("  WebSocket:  ws://0.0.0.0:{}/ws", controller_port);

    let mut process_rx = process_manager.subscribe();
    let reg_for_events = registry.clone();
    tokio::spawn(async move {
        while let Ok(event) = process_rx.recv().await {
            match &event {
                opensim_next::instance_manager::process_manager::ProcessEvent::Spawned {
                    id,
                    pid,
                } => {
                    info!("Process event: {} spawned (PID {})", id, pid);
                }
                opensim_next::instance_manager::process_manager::ProcessEvent::Exited {
                    id,
                    pid,
                    exit_code,
                } => {
                    info!(
                        "Process event: {} exited (PID {}, code={:?})",
                        id, pid, exit_code
                    );
                    let _ = reg_for_events
                        .update_status(id, opensim_next::instance_manager::InstanceStatus::Stopped)
                        .await;
                    let _ = reg_for_events.update_connected(id, false).await;
                }
                _ => {}
            }
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], controller_port));
    tokio::spawn(async move {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                info!("Embedded controller listening on port {}", controller_port);
                if let Err(e) = axum::serve(
                    listener,
                    app.into_make_service_with_connect_info::<SocketAddr>(),
                )
                .await
                {
                    warn!("Embedded controller error: {}", e);
                }
            }
            Err(e) => {
                warn!(
                    "Failed to bind embedded controller on port {}: {}",
                    controller_port, e
                );
            }
        }
    });
}

async fn controller_ws_handler(
    mut socket: axum::extract::ws::WebSocket,
    registry: Arc<opensim_next::instance_manager::InstanceRegistry>,
    _process_manager: Arc<opensim_next::instance_manager::ProcessManager>,
) {
    use futures_util::{SinkExt, StreamExt};
    info!("Controller WebSocket client connected");

    let instances = registry.get_all_instances().await;
    let list_msg = serde_json::json!({
        "type": "InstanceList",
        "instances": instances,
    });
    if let Ok(text) = serde_json::to_string(&list_msg) {
        let _ = socket
            .send(axum::extract::ws::Message::Text(text.into()))
            .await;
    }

    while let Some(msg) = socket.next().await {
        match msg {
            Ok(axum::extract::ws::Message::Text(text)) => {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                    let msg_type = data.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    match msg_type {
                        "Heartbeat" => {
                            let pong = serde_json::json!({ "type": "Pong" });
                            if let Ok(text) = serde_json::to_string(&pong) {
                                let _ = socket
                                    .send(axum::extract::ws::Message::Text(text.into()))
                                    .await;
                            }
                        }
                        "Subscribe" => {
                            let ack = serde_json::json!({ "type": "SubscriptionConfirmed", "channels": ["all"] });
                            if let Ok(text) = serde_json::to_string(&ack) {
                                let _ = socket
                                    .send(axum::extract::ws::Message::Text(text.into()))
                                    .await;
                            }
                        }
                        _ => {
                            tracing::debug!("Controller WS received: {}", msg_type);
                        }
                    }
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }

    info!("Controller WebSocket client disconnected");
}

async fn start_server() -> Result<()> {
    info!("Starting OpenSim Next Server...");

    // Load monitoring HTTP config
    let http_config = MonitoringHttpConfig::from_env();
    info!("Instance ID: {}", http_config.instance_id);
    info!("Metrics endpoint: 0.0.0.0:{}", http_config.metrics_port);

    let login_port: u16 = env::var("OPENSIM_LOGIN_PORT")
        .unwrap_or_else(|_| "9000".to_string())
        .parse()
        .unwrap_or(9000);
    info!(
        "Unified port initialization - binding HTTP/UDP server on port {}...",
        login_port
    );
    let early_login_listener: Option<tokio::net::TcpListener> = None;

    let readiness_tracker = opensim_next::readiness::new_shared_tracker();

    // Initialize robust PostgreSQL → MariaDB fallback system (NO SQLite)
    let db_config = match MultiDatabaseConfig::from_env() {
        Ok(config) => {
            info!(
                "Database configuration loaded: {:?} backend",
                config.database_type
            );
            config
        }
        Err(e) => {
            warn!(
                "Failed to load database configuration: {}. Using MariaDB fallback.",
                e
            );
            MultiDatabaseConfig::mysql(
                "localhost",
                3306,
                "opensim_mariadb",
                "opensim",
                "password123",
            )
        }
    };

    // Robust PostgreSQL → MariaDB fallback connection system
    let (database_connection, database_manager, user_account_database) =
        match try_database_connection_with_fallback(&db_config).await {
            Ok(conn) => {
                let conn = Arc::new(conn);

                // Test connection and run migrations
                if let Err(e) = conn.test_connection().await {
                    warn!("Database connection test failed: {}", e);
                } else {
                    info!("Database connection test successful");
                    readiness_tracker.set_database_connected();

                    // Run database migrations (skip in grid mode — Robust handles migrations)
                    let skip_migrations =
                        env::var("OPENSIM_SERVICE_MODE").unwrap_or_default() == "grid";
                    let migration_ok = if skip_migrations {
                        info!("Grid mode: skipping migrations (Robust server handles them)");
                        true
                    } else {
                        match conn.migrate().await {
                            Ok(_) => {
                                info!("Database migrations completed successfully");
                                true
                            }
                            Err(e) => {
                                warn!("Database migration failed: {}", e);
                                false
                            }
                        }
                    };
                    if migration_ok {
                        readiness_tracker.set_migrations_complete();

                        // Seed default Ruth avatar assets
                        if let Err(e) = conn.seed_default_assets().await {
                            warn!("Failed to seed default assets: {}", e);
                        }

                        // PHASE 72: Load default assets from OpenSim XML asset sets
                        // This is CRITICAL - without these assets, the viewer cannot render properly
                        if let Some(pool) = conn.postgres_pool() {
                            let assets_path = std::path::PathBuf::from("../bin/assets");
                            if assets_path.exists() {
                                info!("🎨 Loading OpenSim default assets from XML asset sets...");
                                match load_default_assets(pool, &assets_path).await {
                                    Ok(()) => info!("✅ Default assets loaded successfully"),
                                    Err(e) => warn!("⚠️ Failed to load default assets: {}", e),
                                }
                            } else {
                                warn!("⚠️ Assets directory not found at {:?}", assets_path);
                            }
                        }
                    }
                }

                // ELEGANT SOLUTION: Use DATABASE_URL directly for PostgreSQL compatibility
                let postgres_url = std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| db_config.connection_string.clone());

                // Use our successful database connection directly instead of creating a new one
                info!(
                    "Creating database manager with existing {} connection",
                    conn.database_type()
                );

                match DatabaseManager::with_connection(conn.clone()).await {
                    Ok(db_manager) => {
                        let db_manager = Arc::new(db_manager);

                        // Database initialization handled by migrations - PostgreSQL/MariaDB ready to use
                        info!(
                            "✅ Database manager initialized with {} backend",
                            conn.database_type()
                        );

                        let user_db = db_manager.user_accounts();
                        info!(
                            "Database system initialized with {:?} backend",
                            db_config.database_type
                        );
                        (Some(conn), Some(db_manager), Some(user_db))
                    }
                    Err(e) => {
                        error!("Database manager initialization failed: {}. Continuing with limited functionality.", e);
                        error!("Database URL being used: {}", postgres_url);
                        error!("Database config type: {:?}", db_config.database_type);
                        (Some(conn), None, None)
                    }
                }
            }
            Err(e) => {
                error!(
                    "Database connection failed: {}. Continuing without database for testing.",
                    e
                );
                error!("Database config: {:?}", db_config);
                (None, None, None)
            }
        };

    // Initialize asset system components
    let cache_config = opensim_next::asset::cache::CacheConfig::default();
    let asset_cache = Arc::new(AssetCache::new(cache_config).await?);

    let cdn_config = opensim_next::asset::cdn::CdnConfig::default();
    let cdn_manager = Arc::new(CdnManager::new(cdn_config).await?);

    // Create assets directory if it doesn't exist
    std::fs::create_dir_all("./assets").unwrap_or_default();
    let storage: Arc<dyn StorageBackend> = Arc::new(FileSystemStorage::new("./assets".into())?);

    let asset_config = AssetManagerConfig::default();

    // For now, skip AssetManager if database is not available
    let asset_manager = if let Some(db_manager) = &database_manager {
        Some(Arc::new(
            AssetManager::new(
                db_manager.clone(),
                asset_cache,
                cdn_manager,
                storage,
                asset_config,
            )
            .await?,
        ))
    } else {
        warn!("Asset manager not initialized due to missing database");
        None
    };
    if asset_manager.is_some() {
        info!("Asset system initialized");
    }

    // Initialize library asset manager for default assets from bin/ directory
    // Look for bin/ directory relative to current working directory (rust/) or parent directory
    let bin_directory = if std::path::Path::new("../bin").exists() {
        std::path::PathBuf::from("../bin")
    } else if std::path::Path::new("bin").exists() {
        std::path::PathBuf::from("bin")
    } else {
        // Use environment variable or default
        std::path::PathBuf::from(
            env::var("OPENSIM_BIN_DIR").unwrap_or_else(|_| "../bin".to_string()),
        )
    };

    info!(
        "Initializing library asset manager from {}",
        bin_directory.display()
    );
    let library_asset_manager = match LibraryAssetManager::new(&bin_directory) {
        Ok(mut manager) => {
            // Initialize asynchronously to load all assets
            match manager.initialize().await {
                Ok(_) => {
                    info!("✅ Library asset manager initialized successfully");
                    let manager_arc = Arc::new(tokio::sync::RwLock::new(manager));
                    // Set global reference for use by login/inventory services
                    set_global_library_manager(manager_arc.clone());
                    Some(manager_arc)
                }
                Err(e) => {
                    warn!("⚠️  Library asset manager initialization failed: {}. Continuing without library assets.", e);
                    None
                }
            }
        }
        Err(e) => {
            warn!("⚠️  Failed to create library asset manager: {}. Continuing without library assets.", e);
            None
        }
    };

    // Initialize avatar data manager (openmetaverse_data files)
    info!(
        "Initializing avatar data manager from {}",
        bin_directory.display()
    );
    let _avatar_data_manager = match AvatarDataManager::new(&bin_directory) {
        Ok(mut manager) => match manager.initialize().await {
            Ok(_) => {
                info!("✅ Avatar data manager initialized successfully");
                let manager_arc = Arc::new(tokio::sync::RwLock::new(manager));
                set_global_avatar_data_manager(manager_arc.clone());
                Some(manager_arc)
            }
            Err(e) => {
                warn!("⚠️  Avatar data manager initialization failed: {}. Continuing without avatar data.", e);
                None
            }
        },
        Err(e) => {
            warn!(
                "⚠️  Failed to create avatar data manager: {}. Continuing without avatar data.",
                e
            );
            None
        }
    };

    // Initialize animation manager (avataranimations.xml)
    info!(
        "Initializing animation manager from {}",
        bin_directory.display()
    );
    match init_global_animations(&bin_directory) {
        Ok(_) => info!("✅ Animation manager initialized successfully"),
        Err(e) => warn!(
            "⚠️  Animation manager initialization failed: {}. Continuing without animations.",
            e
        ),
    }

    // Initialize core systems
    let state_manager = Arc::new(StateManager::new()?);

    // Phase 140: Create shared Physics instance — per-simulator from OpenSim.ini [Startup] physics
    let physics_engine_name = {
        let mut name = None;
        let instance_dir =
            std::env::var("OPENSIM_INSTANCE_DIR").unwrap_or_else(|_| ".".to_string());
        let ini_path = std::path::Path::new(&instance_dir).join("bin/OpenSim.ini");
        if let Ok(content) = std::fs::read_to_string(&ini_path) {
            let mut in_startup = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') {
                    in_startup = trimmed.eq_ignore_ascii_case("[startup]");
                } else if in_startup {
                    if let Some(eq) = trimmed.find('=') {
                        let key = trimmed[..eq].trim();
                        if key.eq_ignore_ascii_case("physics") {
                            let val = trimmed[eq + 1..].trim().trim_matches('"');
                            name = Some(val.to_string());
                            break;
                        }
                    }
                }
            }
            if let Some(ref n) = name {
                info!("[PHYSICS] Read physics='{}' from {}", n, ini_path.display());
            }
        }
        name.or_else(|| std::env::var("OPENSIM_PHYSICS").ok())
            .unwrap_or_else(|| "BulletSim".to_string())
    };
    let physics_engine_type = opensim_next::ffi::PhysicsEngineType::from_str(&physics_engine_name);
    info!(
        "Initializing physics engine: {:?} (from config: '{}')",
        physics_engine_type, physics_engine_name
    );
    let shared_physics: Option<Arc<std::sync::Mutex<opensim_next::ffi::Physics>>> =
        match opensim_next::ffi::Physics::with_engine(physics_engine_type) {
            Ok(physics) => {
                info!(
                    "✅ Physics engine enabled ({:?}) - shared instance for avatar movement",
                    physics_engine_type
                );
                Some(Arc::new(std::sync::Mutex::new(physics)))
            }
            Err(e) => {
                warn!("⚠️  Physics engine failed to initialize: {}. Avatar movement will use arithmetic fallback.", e);
                None
            }
        };
    // RegionManager uses PhysicsBridge stubs - keep it disabled to avoid double-init
    let physics_bridge = Arc::new(PhysicsBridge::new_disabled());
    let region_manager = Arc::new(RegionManager::new(
        physics_bridge.clone(),
        state_manager.clone(),
    ));

    // Phase 94.1: Initialize scripting system with real managers
    let _scripting_manager = if let Some(ref asset_mgr) = asset_manager {
        let script_engine = opensim_next::scripting::ScriptEngine::new_with_managers(
            region_manager.clone(),
            asset_mgr.clone(),
            None,
        );
        match opensim_next::scripting::ScriptingManager::new_with_engine(
            Arc::new(script_engine),
            region_manager.clone(),
            state_manager.clone(),
            asset_mgr.clone(),
        ) {
            Ok(mgr) => {
                let mgr = Arc::new(mgr);
                let mgr_clone = mgr.clone();
                tokio::spawn(async move {
                    if let Err(e) = mgr_clone.start().await {
                        error!("Scripting system start failed: {}", e);
                    }
                });
                info!("Scripting system started");
                Some(mgr)
            }
            Err(e) => {
                warn!("Scripting system initialization failed: {}", e);
                None
            }
        }
    } else {
        warn!("Scripting system not initialized - no asset manager");
        None
    };

    // Phase 103.1: Parse Regions.ini for dynamic region configuration
    let regions_dir_str =
        env::var("OPENSIM_REGIONS_DIR").unwrap_or_else(|_| "bin/Regions".to_string());
    let regions_dir = std::path::Path::new(&regions_dir_str);
    let region_configs = match config_parser::parse_regions_ini(regions_dir) {
        Ok(configs) => {
            info!(
                "[REGION] Loaded {} region(s) from Regions.ini",
                configs.len()
            );
            configs
        }
        Err(e) => {
            warn!(
                "[REGION] Failed to parse Regions.ini: {} - using defaults",
                e
            );
            vec![config_parser::RegionIniConfig {
                name: "Default Region".to_string(),
                uuid: uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
                grid_x: 1000,
                grid_y: 1000,
                size_x: 256,
                size_y: 256,
                internal_port: 9000,
                internal_address: "0.0.0.0".to_string(),
                external_host: "SYSTEMIP".to_string(),
                max_prims: 45000,
                max_agents: 100,
                scope_id: uuid::Uuid::nil(),
                region_type: "Mainland".to_string(),
                physics: physics_engine_name.clone(),
                meshing: "Meshmerizer".to_string(),
                water_height: 20.0,
            }]
        }
    };
    let active_region = region_configs[0].clone();
    info!(
        "[REGION] Active region: '{}' UUID={} grid=({},{}) port={} handle={}",
        active_region.name,
        active_region.uuid,
        active_region.grid_x,
        active_region.grid_y,
        active_region.internal_port,
        active_region.region_handle()
    );

    // Create both session managers: one for XMLRPC login sessions, one for network management
    let login_session_manager = Arc::new(LoginSessionManager::new_with_region_size(
        active_region.region_x_meters(),
        active_region.region_y_meters(),
        active_region.size_x,
        active_region.size_y,
    ));
    let network_session_manager =
        Arc::new(SessionManager::new(std::time::Duration::from_secs(60 * 10)));

    // Initialize loopback connectors for proper local networking (excluding web service to avoid port conflict)
    info!("Initializing loopback connectors...");
    let mut loopback_config = LoopbackConfig::default();
    loopback_config.enabled = false; // Disable to avoid port conflicts with WebClientServer
    let _loopback_connector = Arc::new(LoopbackConnector::new(loopback_config));

    // Note: Loopback connectors disabled to prevent port conflicts with dedicated WebClientServer

    // Initialize Hypergrid manager for inter-grid communication
    info!("Initializing Hypergrid...");
    let hg_config = opensim_next::services::config_parser::build_hypergrid_config();
    let mut grid_service_for_registration: Option<
        Arc<dyn opensim_next::services::traits::GridServiceTrait>,
    > = None;
    let hypergrid_manager: Option<Arc<HypergridManager>> = if hg_config.enabled {
        if let Some(ref db_conn) = database_connection {
            let svc_mode = env::var("OPENSIM_SERVICE_MODE").unwrap_or_default();
            let robust_url = env::var("OPENSIM_ROBUST_URL").ok();
            let svc_config = if svc_mode == "grid" {
                let url = robust_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:8003".to_string());
                opensim_next::services::traits::ServiceConfig {
                    mode: opensim_next::services::traits::ServiceMode::Grid,
                    grid_server_uri: Some(url.clone()),
                    asset_server_uri: Some(url.clone()),
                    inventory_server_uri: Some(url.clone()),
                    user_account_server_uri: Some(url.clone()),
                    presence_server_uri: Some(url.clone()),
                    avatar_server_uri: Some(url.clone()),
                    authentication_server_uri: Some(url),
                }
            } else {
                opensim_next::services::traits::ServiceConfig {
                    mode: opensim_next::services::traits::ServiceMode::Standalone,
                    ..Default::default()
                }
            };
            match opensim_next::services::factory::ServiceContainer::new(
                svc_config,
                Some(db_conn.clone()),
            ) {
                Ok(svc) => {
                    let local_uas: Arc<dyn opensim_next::services::traits::UserAgentServiceTrait> =
                        Arc::new(opensim_next::services::local::LocalUserAgentService::new(
                            svc.grid_service.clone(),
                            svc.user_account_service.clone(),
                            svc.presence_service.clone(),
                            db_conn.clone(),
                            hg_config.home_uri.clone(),
                            hg_config.external_uri.clone(),
                            hg_config.external_robust_uri.clone(),
                            hg_config.grid_name.clone(),
                        ));
                    let gk_external = if hg_config.external_uri.is_empty() {
                        hg_config.gatekeeper_uri.clone()
                    } else {
                        hg_config.external_uri.clone()
                    };
                    let local_gk: Arc<dyn opensim_next::services::traits::GatekeeperServiceTrait> =
                        Arc::new(
                            opensim_next::services::local::LocalGatekeeperService::new(
                                svc.grid_service.clone(),
                                svc.presence_service.clone(),
                                db_conn.clone(),
                                gk_external,
                            )
                            .with_config(
                                hg_config.allow_teleports_to_any_region,
                                hg_config.foreign_agents_allowed,
                            )
                            .with_user_account_service(svc.user_account_service.clone())
                            .with_local_uas(local_uas.clone()),
                        );
                    grid_service_for_registration = Some(svc.grid_service.clone());
                    let mgr = HypergridManager::new(local_gk, local_uas, hg_config.clone());
                    info!(
                        "Hypergrid manager created (home={}, gk={})",
                        hg_config.home_uri, hg_config.gatekeeper_uri
                    );
                    Some(Arc::new(mgr))
                }
                Err(e) => {
                    warn!("Hypergrid enabled but ServiceContainer creation failed: {}. Skipping HG init.", e);
                    None
                }
            }
        } else {
            warn!("Hypergrid enabled but no database connection — skipping HG init");
            None
        }
    } else {
        info!("Hypergrid disabled");
        None
    };

    // Monitoring system config (for logic)
    let mut monitoring_system_config = opensim_next::monitoring::MonitoringConfig::default();
    monitoring_system_config.enable_metrics = true;
    monitoring_system_config.enable_health_checks = true;
    monitoring_system_config.enable_profiling = env::var("OPENSIM_ENABLE_PROFILING")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);
    let monitoring = Arc::new(MonitoringSystem::new(monitoring_system_config)?);
    monitoring.start().await?;
    info!("Monitoring system started");

    // Force NetworkManager initialization for Phase 28.1 testing
    let _network_manager =
        if let (Some(asset_mgr), Some(user_db)) = (&asset_manager, &user_account_database) {
            let network_manager = Arc::new(
                NetworkManager::new(
                    monitoring.clone(),
                    network_session_manager.clone(),
                    region_manager.clone(),
                    state_manager.clone(),
                    asset_mgr.clone(),
                    user_db.clone(),
                )
                .await?,
            );
            info!("Network manager initialized with full dependencies");
            Some(network_manager)
        } else {
            warn!("Network manager dependencies missing - login server will be handled separately");
            None
        };

    // Create CAPS manager - UNIFIED PORT: Use port 9000 for all services
    let system_ip = opensim_next::config::login::resolve_system_ip();
    let caps_manager = Arc::new(
        CapsManager::new(format!("http://{}:{}", system_ip, login_port))
            .with_default_region_uuid(active_region.uuid.to_string()),
    );

    // Get database pool for CAPS handlers (texture serving)
    let caps_db_pool = if let Some(ref db_conn) = database_connection {
        if let Some(pool) = db_conn.postgres_pool() {
            Arc::new(pool.clone())
        } else {
            panic!("PostgreSQL pool required for CAPS texture serving");
        }
    } else {
        panic!("Database connection required for CAPS texture serving");
    };

    // Create login stage tracker for diagnostic visibility (created early so CAPS can use it)
    let login_stage_tracker = Arc::new(LoginStageTracker::new());
    info!("[LOGIN_STAGE] Created login stage tracker for diagnostic visibility");

    let mut caps_state = CapsHandlerState::new(
        caps_manager.clone(),
        caps_db_pool,
        login_stage_tracker.clone(),
    );

    // UNIFIED PORT: CAPS will be served through the unified HTTP server on port 9000
    // Separate CAPS server disabled in favor of unified architecture
    info!(
        "CAPS services will be handled through unified HTTP server on port {}",
        login_port
    );

    // Create circuit code registry with LoginSessionManager access
    let circuit_codes = CircuitCodeRegistry::with_session_manager(login_session_manager.clone());

    // Store early listener in an Arc for sharing with spawn
    let early_login_listener = Arc::new(tokio::sync::Mutex::new(early_login_listener));

    // Phase 106: Generate map tiles for each region, then build region registry
    let has_j2k_encoder = check_opj_compress_available();
    if !has_j2k_encoder {
        warn!("[MAP] opj_compress not found - map tiles will be blank. Install OpenJPEG for map tile support.");
    }

    let mut region_infos: Vec<opensim_next::udp::RegionInfo> =
        Vec::with_capacity(region_configs.len());
    for rc in &region_configs {
        let mut map_image_id = uuid::Uuid::nil();

        if has_j2k_encoder {
            if let Some(ref db_conn) = database_connection {
                let terrain_storage = TerrainStorage::new(Arc::clone(db_conn));
                match terrain_storage.load_terrain(rc.uuid).await {
                    Ok(Some(heightmap)) => {
                        let tile_uuid = uuid::Uuid::new_v5(&rc.uuid, b"maptile");
                        match generate_map_tile(&heightmap, tile_uuid, db_conn).await {
                            Ok(size) => {
                                map_image_id = tile_uuid;
                                info!(
                                    "[MAP] Generated map tile for '{}' ({}, {} bytes)",
                                    rc.name, tile_uuid, size
                                );
                            }
                            Err(e) => {
                                warn!("[MAP] Failed to generate map tile for '{}': {}", rc.name, e)
                            }
                        }
                    }
                    Ok(None) => info!(
                        "[MAP] No terrain data for '{}' - map tile will be blank",
                        rc.name
                    ),
                    Err(e) => warn!("[MAP] Failed to load terrain for '{}': {}", rc.name, e),
                }
            }
        }

        region_infos.push(opensim_next::udp::RegionInfo {
            name: rc.name.clone(),
            uuid: rc.uuid,
            grid_x: rc.grid_x,
            grid_y: rc.grid_y,
            size_x: rc.size_x,
            size_y: rc.size_y,
            map_image_id,
            internal_port: rc.internal_port,
            service_mode: env::var("OPENSIM_SERVICE_MODE")
                .unwrap_or_else(|_| "standalone".to_string()),
        });
    }

    let region_registry: Arc<Vec<opensim_next::udp::RegionInfo>> = Arc::new(region_infos);
    info!(
        "[REGION] Region registry: {} region(s) for map lookup",
        region_registry.len()
    );

    if let Some(ref grid_svc) = grid_service_for_registration {
        info!(
            "[GRID] Registering {} region(s) with Robust grid service...",
            region_configs.len()
        );
        for rc in &region_configs {
            let grid_region = opensim_next::services::traits::RegionInfo {
                region_id: rc.uuid,
                region_name: rc.name.clone(),
                region_loc_x: rc.grid_x,
                region_loc_y: rc.grid_y,
                region_size_x: rc.size_x,
                region_size_y: rc.size_y,
                server_ip: rc.internal_address.clone(),
                server_port: rc.internal_port,
                server_uri: rc.external_host.clone(),
                region_flags: 0x01,
                scope_id: rc.scope_id,
                owner_id: uuid::Uuid::nil(),
                estate_id: 1,
            };
            match grid_svc.register_region(&grid_region).await {
                Ok(true) => info!(
                    "[GRID] Registered '{}' ({}) at ({},{})",
                    rc.name, rc.uuid, rc.grid_x, rc.grid_y
                ),
                Ok(false) => warn!("[GRID] Failed to register '{}': returned false", rc.name),
                Err(e) => warn!("[GRID] Failed to register '{}': {}", rc.name, e),
            }
        }
        readiness_tracker.set_robust_registered();
    } else {
        readiness_tracker.set_robust_registered();
    }

    // Phase 109: Module system initialization
    let module_registry = {
        let mut registry = opensim_next::modules::ModuleRegistry::new();

        let chat_config = opensim_next::modules::ModuleConfig::new("Chat");
        registry.register_shared(
            Box::new(opensim_next::modules::ChatModule::new()),
            chat_config,
        );

        let perm_config = opensim_next::modules::ModuleConfig::new("Permissions");
        registry.register_shared(
            Box::new(opensim_next::modules::PermissionsModule::new()),
            perm_config,
        );

        let land_config = opensim_next::modules::ModuleConfig::new("Land");
        registry.register_shared(
            Box::new(opensim_next::modules::LandManagementModule::new()),
            land_config,
        );

        let dialog_config = opensim_next::modules::ModuleConfig::new("Dialog");
        registry.register_shared(
            Box::new(opensim_next::modules::DialogModule::new()),
            dialog_config,
        );

        let sound_config = opensim_next::modules::ModuleConfig::new("Sound");
        registry.register_shared(
            Box::new(opensim_next::modules::SoundModule::new()),
            sound_config,
        );

        let estate_config = opensim_next::modules::ModuleConfig::new("Estate");
        registry.register_shared(
            Box::new(opensim_next::modules::EstateManagementModule::new()),
            estate_config,
        );

        let user_mgmt_config = opensim_next::modules::ModuleConfig::new("UserManagement");
        registry.register_shared(
            Box::new(opensim_next::modules::UserManagementModule::new()),
            user_mgmt_config,
        );

        let xfer_config = opensim_next::modules::ModuleConfig::new("Xfer");
        registry.register_shared(
            Box::new(opensim_next::modules::XferModule::new()),
            xfer_config,
        );

        let terrain_config = opensim_next::modules::ModuleConfig::new("Terrain");
        registry.register_shared(
            Box::new(opensim_next::modules::TerrainModuleImpl::new()),
            terrain_config,
        );

        let env_config = opensim_next::modules::ModuleConfig::new("Environment");
        registry.register_shared(
            Box::new(opensim_next::modules::EnvironmentModule::new()),
            env_config,
        );

        let wind_config = opensim_next::modules::ModuleConfig::new("Wind");
        registry.register_shared(
            Box::new(opensim_next::modules::WindModule::new()),
            wind_config,
        );

        if let Err(e) = registry.initialize_all().await {
            warn!("[MODULES] Failed to initialize modules: {}", e);
        }

        Arc::new(parking_lot::RwLock::new(registry))
    };
    info!(
        "[MODULES] Module registry created with {} modules",
        module_registry.read().module_count()
    );

    let voice_module: Option<Arc<dyn opensim_next::modules::voice::VoiceHandler>> = {
        use opensim_next::modules::RegionModule;
        let vivox_enabled = std::env::var("OPENSIM_VIVOX_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let freeswitch_enabled = std::env::var("OPENSIM_FREESWITCH_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        if vivox_enabled {
            let mut vivox_config = opensim_next::modules::ModuleConfig::new("VivoxVoice");
            vivox_config
                .params
                .insert("enabled".to_string(), "true".to_string());
            if let Ok(v) = std::env::var("OPENSIM_VIVOX_SERVER") {
                vivox_config.params.insert("vivox_server".to_string(), v);
            }
            if let Ok(v) = std::env::var("OPENSIM_VIVOX_SIP_URI") {
                vivox_config.params.insert("vivox_sip_uri".to_string(), v);
            }
            if let Ok(v) = std::env::var("OPENSIM_VIVOX_ADMIN_USER") {
                vivox_config
                    .params
                    .insert("vivox_admin_user".to_string(), v);
            }
            if let Ok(v) = std::env::var("OPENSIM_VIVOX_ADMIN_PASSWORD") {
                vivox_config
                    .params
                    .insert("vivox_admin_password".to_string(), v);
            }
            if let Ok(v) = std::env::var("OPENSIM_VIVOX_CHANNEL_TYPE") {
                vivox_config
                    .params
                    .insert("vivox_channel_type".to_string(), v);
            }
            if let Ok(v) = std::env::var("OPENSIM_VIVOX_CHANNEL_MODE") {
                vivox_config
                    .params
                    .insert("vivox_channel_mode".to_string(), v);
            }
            if let Ok(v) = std::env::var("OPENSIM_VIVOX_CHANNEL_DISTANCE_MODEL") {
                vivox_config
                    .params
                    .insert("vivox_channel_distance_model".to_string(), v);
            }
            if let Ok(v) = std::env::var("OPENSIM_VIVOX_CHANNEL_ROLL_OFF") {
                vivox_config
                    .params
                    .insert("vivox_channel_roll_off".to_string(), v);
            }
            if let Ok(v) = std::env::var("OPENSIM_VIVOX_CHANNEL_MAX_RANGE") {
                vivox_config
                    .params
                    .insert("vivox_channel_max_range".to_string(), v);
            }
            if let Ok(v) = std::env::var("OPENSIM_VIVOX_CHANNEL_CLAMPING_DISTANCE") {
                vivox_config
                    .params
                    .insert("vivox_channel_clamping_distance".to_string(), v);
            }

            let mut module = opensim_next::modules::voice::VivoxVoiceModule::new();
            if let Err(e) = module.initialize(&vivox_config).await {
                warn!("[VOICE] Vivox initialization failed: {}", e);
                None
            } else if module.is_enabled() {
                info!("[VOICE] Vivox voice module enabled");
                Some(Arc::new(module) as Arc<dyn opensim_next::modules::voice::VoiceHandler>)
            } else {
                None
            }
        } else if freeswitch_enabled {
            let mut fs_config = opensim_next::modules::ModuleConfig::new("FreeSwitchVoice");
            fs_config
                .params
                .insert("Enabled".to_string(), "true".to_string());
            if let Ok(v) = std::env::var("OPENSIM_FREESWITCH_SERVER") {
                fs_config.params.insert("ServerAddress".to_string(), v);
            }
            if let Ok(v) = std::env::var("OPENSIM_FREESWITCH_REALM") {
                fs_config.params.insert("Realm".to_string(), v);
            }
            if let Ok(v) = std::env::var("OPENSIM_FREESWITCH_SIP_PROXY") {
                fs_config.params.insert("SIPProxy".to_string(), v);
            }
            if let Ok(v) = std::env::var("OPENSIM_FREESWITCH_ECHO_SERVER") {
                fs_config.params.insert("EchoServer".to_string(), v);
            }
            if let Ok(v) = std::env::var("OPENSIM_FREESWITCH_ECHO_PORT") {
                fs_config.params.insert("EchoPort".to_string(), v);
            }

            let mut module = opensim_next::modules::voice::FreeSwitchVoiceModule::new();
            if let Err(e) = module.initialize(&fs_config).await {
                warn!("[VOICE] FreeSWITCH initialization failed: {}", e);
                None
            } else if module.is_enabled() {
                info!("[VOICE] FreeSWITCH voice module enabled");
                Some(Arc::new(module) as Arc<dyn opensim_next::modules::voice::VoiceHandler>)
            } else {
                None
            }
        } else {
            info!("[VOICE] No voice module enabled (set OPENSIM_VIVOX_ENABLED=true or OPENSIM_FREESWITCH_ENABLED=true)");
            None
        }
    };
    if let Some(vm) = &voice_module {
        caps_state.voice_module = Some(vm.clone());
    }

    let yengine = {
        let mut engine = opensim_next::scripting::yengine_module::YEngineModule::new();
        engine.initialize_default();
        Arc::new(parking_lot::RwLock::new(engine))
    };
    info!("[YENGINE] Script engine initialized (TreeWalk backend)");

    // Phase 103.2: Multi-region startup - one UdpServer per configured region
    info!("[REGION] Starting {} region(s)...", region_configs.len());
    let mut udp_scene_objects_opt = None;
    let mut map_items_avatar_states: Option<
        Arc<
            parking_lot::RwLock<
                std::collections::HashMap<uuid::Uuid, opensim_next::udp::AvatarMovementState>,
            >,
        >,
    > = None;
    let mut map_items_parcels: Option<
        Arc<parking_lot::RwLock<Vec<opensim_next::modules::land::Parcel>>>,
    > = None;
    let mut regions_started = 0u32;

    let udp_security_manager = Arc::new(
        opensim_next::network::security::SecurityManager::new()
            .expect("Failed to create SecurityManager"),
    );
    info!("[SECURITY] SecurityManager initialized for UDP firewall");

    let ziti_manager = Arc::new(opensim_next::network::ziti_manager::ZitiManager::from_env());
    if ziti_manager.is_enabled() {
        if let Err(e) = ziti_manager.start().await {
            warn!("[ZITI] Failed to start OpenZiti tunnel: {} — continuing without zero-trust overlay", e);
        }
    } else {
        info!("[ZITI] OpenZiti integration disabled (set OPENSIM_ZITI_ENABLED=true to enable)");
    }

    let shared_galadriel_brain: Arc<
        tokio::sync::Mutex<Option<opensim_next::ai::galadriel::GaladrielBrain>>,
    > = Arc::new(tokio::sync::Mutex::new(None));
    let shared_build_sessions = if let Some(ref db_conn) = database_connection {
        if let Some(pool) = db_conn.postgres_pool() {
            opensim_next::ai::build_session::BuildSessionStore::new(Some(pool.clone()))
        } else {
            opensim_next::ai::build_session::BuildSessionStore::new(None)
        }
    } else {
        opensim_next::ai::build_session::BuildSessionStore::new(None)
    };
    let shared_npc_memory = if let Some(ref db_conn) = database_connection {
        if let Some(pool) = db_conn.postgres_pool() {
            opensim_next::ai::npc_memory::NPCMemoryStore::new(Some(pool.clone()))
        } else {
            opensim_next::ai::npc_memory::NPCMemoryStore::new(None)
        }
    } else {
        opensim_next::ai::npc_memory::NPCMemoryStore::new(None)
    };
    info!(
        "[GALADRIEL] Shared AI brain + stores created for {} region(s)",
        region_configs.len()
    );

    let shared_handshake_sent: Arc<tokio::sync::Mutex<std::collections::HashSet<u32>>> =
        Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new()));

    let shared_asset_fetcher: Arc<opensim_next::asset::AssetFetcher> = {
        let inst_dir = env::var("OPENSIM_INSTANCE_DIR").unwrap_or_else(|_| ".".to_string());
        let fsassets_base = format!("{}/fsassets/data", inst_dir);
        let fsassets_spool = format!("{}/fsassets/tmp", inst_dir);
        let fsassets_config = opensim_next::asset::FSAssetsConfig {
            base_directory: std::path::PathBuf::from(&fsassets_base),
            spool_directory: std::path::PathBuf::from(&fsassets_spool),
            use_osgrid_format: false,
            compression_level: 6,
        };

        let cache_config = opensim_next::asset::CacheConfig {
            memory_cache_size: 2000,
            memory_ttl_seconds: 300,
            redis_ttl_seconds: 3600,
            redis_url: env::var("REDIS_URL")
                .ok()
                .or_else(|| env::var("VALKEY_URL").ok()),
            enable_compression: true,
            compression_threshold: 4096,
        };
        let asset_cache = match opensim_next::asset::AssetCache::new(cache_config).await {
            Ok(cache) => {
                info!("[ASSET-CACHE] Memory LRU + Redis/Valkey cache initialized");
                Some(Arc::new(cache))
            }
            Err(e) => {
                warn!(
                    "[ASSET-CACHE] Failed to initialize: {} — continuing without cache",
                    e
                );
                None
            }
        };

        match opensim_next::asset::FSAssetsStorage::new(fsassets_config) {
            Ok(storage) => {
                let storage = Arc::new(storage);
                storage.start_background_writer();
                info!(
                    "[FSASSETS] Filesystem asset storage enabled: {}",
                    fsassets_base
                );
                let mut fetcher = opensim_next::asset::AssetFetcher::new(Some(storage), true);
                if let Some(cache) = asset_cache {
                    fetcher = fetcher.with_cache(cache);
                }
                Arc::new(fetcher)
            }
            Err(e) => {
                warn!("[FSASSETS] Failed to initialize filesystem storage: {} — using legacy DB-only mode", e);
                let mut fetcher = opensim_next::asset::AssetFetcher::new_legacy();
                if let Some(cache) = asset_cache {
                    fetcher = fetcher.with_cache(cache);
                }
                Arc::new(fetcher)
            }
        }
    };

    caps_state.asset_fetcher = Some(shared_asset_fetcher.clone());

    for (i, region_config) in region_configs.iter().enumerate() {
        let bind_addr = format!("0.0.0.0:{}", region_config.internal_port);
        info!(
            "[REGION {}/{}] Starting '{}' on {} (grid {},{} handle={})",
            i + 1,
            region_configs.len(),
            region_config.name,
            bind_addr,
            region_config.grid_x,
            region_config.grid_y,
            region_config.region_handle()
        );

        let udp_server_result = opensim_next::udp::UdpServer::new(
            &bind_addr,
            login_session_manager.clone(),
            login_stage_tracker.clone(),
            region_config.uuid,
            region_config.region_handle(),
            region_config.grid_x,
            region_config.grid_y,
            region_config.size_x,
            region_config.size_y,
            region_config.water_height,
            region_config.name.clone(),
        )
        .await;

        match udp_server_result {
            Ok(mut udp_server) => {
                if i == 0 {
                    readiness_tracker.set_udp_bound();
                }
                udp_server = udp_server
                    .with_caps_manager(caps_manager.clone())
                    .with_region_registry(region_registry.clone())
                    .with_module_registry(module_registry.clone())
                    .with_yengine(yengine.clone())
                    .with_security_manager(udp_security_manager.clone())
                    .with_galadriel_brain(shared_galadriel_brain.clone())
                    .with_shared_ai_stores(shared_build_sessions.clone(), shared_npc_memory.clone())
                    .with_shared_handshake_sent(shared_handshake_sent.clone());

                if let Some(ref hg) = hypergrid_manager {
                    udp_server = udp_server.with_hypergrid_manager(hg.clone());
                }

                if let Some(ref physics) = shared_physics {
                    udp_server = udp_server.with_physics(Arc::clone(physics));
                }

                udp_server = udp_server.with_asset_fetcher(shared_asset_fetcher.clone());

                if let Some(ref db_conn) = database_connection {
                    udp_server = udp_server.with_database(db_conn.clone());

                    if let Err(e) = udp_server.initialize_terrain().await {
                        warn!(
                            "[REGION] '{}': Failed to init terrain: {}",
                            region_config.name, e
                        );
                    }
                    if i == 0 {
                        readiness_tracker.set_terrain_loaded();
                    }
                    if let Err(e) = udp_server.load_scene_objects_from_db().await {
                        warn!(
                            "[REGION] '{}': Failed to load objects: {}",
                            region_config.name, e
                        );
                    }
                    if i == 0 {
                        readiness_tracker.set_scene_loaded();
                    }
                    if let Err(e) = udp_server.load_scripts_from_db().await {
                        warn!(
                            "[REGION] '{}': Failed to load scripts: {}",
                            region_config.name, e
                        );
                    }
                    if i == 0 {
                        readiness_tracker.set_scripts_initialized();
                    }
                    if let Err(e) = udp_server.load_parcels().await {
                        warn!(
                            "[REGION] '{}': Failed to load parcels: {}",
                            region_config.name, e
                        );
                    }
                    if let Err(e) = udp_server.load_estate_info().await {
                        warn!(
                            "[REGION] '{}': Failed to load estate info: {}",
                            region_config.name, e
                        );
                    }
                } else if i == 0 {
                    readiness_tracker.set_terrain_loaded();
                    readiness_tracker.set_scene_loaded();
                    readiness_tracker.set_scripts_initialized();
                }

                if i == 0 {
                    udp_server.set_home_region(true);
                    caps_state.avatar_factory = Some(udp_server.avatar_factory());
                    caps_state.scene_objects = Some(udp_server.scene_objects());
                    caps_state.parcels = Some(udp_server.parcels());
                    caps_state.yengine = Some(yengine.clone());
                    map_items_avatar_states = Some(udp_server.avatar_states());
                    map_items_parcels = Some(udp_server.parcels());
                    udp_scene_objects_opt = Some((
                        udp_server.scene_objects(),
                        udp_server.udp_socket(),
                        udp_server.avatar_states(),
                        udp_server.next_prim_local_id(),
                        udp_server.reliability_manager(),
                    ));
                }

                // Phase 109: Run module lifecycle for this region
                {
                    let scene = udp_server.build_scene_context();
                    let mut reg = module_registry.write();
                    if let Err(e) = reg.add_region_all(&scene).await {
                        warn!(
                            "[MODULES] add_region failed for '{}': {}",
                            region_config.name, e
                        );
                    }
                    if let Err(e) = reg.post_initialize_all(&scene).await {
                        warn!(
                            "[MODULES] post_initialize failed for '{}': {}",
                            region_config.name, e
                        );
                    }
                    if let Err(e) = reg.region_loaded_all(&scene).await {
                        warn!(
                            "[MODULES] region_loaded failed for '{}': {}",
                            region_config.name, e
                        );
                    }
                }

                let region_name = region_config.name.clone();
                let region_port = region_config.internal_port;
                let udp_server = std::sync::Arc::new(udp_server);
                let udp_clone = udp_server.clone();
                tokio::spawn(async move {
                    if let Err(e) = udp_clone.run().await {
                        error!("[REGION] '{}' UDP server error: {}", region_name, e);
                    }
                });
                regions_started += 1;
                info!(
                    "[REGION] '{}' started on port {}",
                    region_config.name, region_port
                );
            }
            Err(e) => {
                error!(
                    "[REGION] Failed to start '{}' on port {}: {}",
                    region_config.name, region_config.internal_port, e
                );
                if i == 0 {
                    return Err(anyhow!(
                        "Home region '{}' failed to start: {}",
                        region_config.name,
                        e
                    ));
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    info!(
        "[REGION] {} of {} region(s) started successfully",
        regions_started,
        region_configs.len()
    );

    // REMOVED: 600+ lines of manual UDP packet handling code replaced with UdpServer
    // The UdpServer now handles:
    // - UseCircuitCode
    // - CompleteAgentMovement (with automatic terrain transmission)
    // - StartPingCheck / CompletePingCheck
    // - RegionHandshakeReply
    // - AgentUpdate
    // - Packet acknowledgments
    // - Reliability management

    // Spawn HTTP server for metrics and health with authentication
    let app_state = AppState {
        config: http_config.clone(),
        monitoring: monitoring.clone(),
        session_manager: network_session_manager.clone(),
        circuit_codes: circuit_codes.clone(),
    };
    let metrics_port = http_config.metrics_port;

    // Spawn Admin API server for Robust-style database commands
    let admin_port = env::var("OPENSIM_ADMIN_PORT")
        .unwrap_or_else(|_| "9200".to_string())
        .parse()
        .unwrap_or(9200);

    if let Some(database_manager) = database_manager.as_ref() {
        let admin_api_state = AdminApiState {
            admin: database_manager.admin(),
        };

        let archive_api_state = match database_manager.legacy_pool() {
            Ok(pool) => {
                let mut state = ArchiveApiState::new(pool.clone());
                if let Some(ref tuple) = udp_scene_objects_opt {
                    state = state
                        .with_scene_objects(tuple.0.clone())
                        .with_udp_socket(tuple.1.clone())
                        .with_avatar_states(tuple.2.clone())
                        .with_next_prim_local_id(tuple.3.clone())
                        .with_reliability_manager(tuple.4.clone());
                }
                Some(state)
            }
            Err(e) => {
                warn!("Could not initialize archive API: {}", e);
                None
            }
        };

        let admin_api_clone = admin_api_state.clone();
        let archive_api_clone = archive_api_state.clone();
        let console_api_state = ConsoleApiState {
            db_admin: database_manager.admin(),
            db_connection: Some(database_manager.connection()),
        };
        let console_api_clone = console_api_state.clone();
        let security_api_state = SecurityApiState {
            security_manager: udp_security_manager.clone(),
            ziti_manager: ziti_manager.clone(),
        };
        let security_api_clone = security_api_state.clone();
        tokio::spawn(async move {
            let admin_router = create_admin_api_router().with_state(admin_api_clone);
            let console_router = create_console_api_router().with_state(console_api_clone);
            let security_router = create_security_api_router().with_state(security_api_clone);
            let skill_router = create_skill_api_router();
            let combined_router = if let Some(archive_state) = archive_api_clone {
                let archive_router = create_archive_api_router().with_state(archive_state);
                admin_router
                    .merge(archive_router)
                    .merge(console_router)
                    .merge(security_router)
                    .merge(skill_router)
            } else {
                admin_router
                    .merge(console_router)
                    .merge(security_router)
                    .merge(skill_router)
            };

            let addr = SocketAddr::from(([0, 0, 0, 0], admin_port));
            info!("Starting admin API server on {}", addr);
            info!("Admin API endpoints:");
            info!("  POST /admin/users - Create user");
            info!("  GET /admin/users - List users");
            info!("  GET /admin/users/account - Show user account");
            info!("  PUT /admin/users/password - Reset password");
            info!("  PUT /admin/users/email - Reset email");
            info!("  PUT /admin/users/level - Set user level");
            info!("  DELETE /admin/users/delete - Delete user");
            info!("  GET /admin/database/stats - Database statistics");
            info!("  GET /admin/health - Admin API health");
            info!("Archive API endpoints:");
            info!("  POST /admin/archives/iar/load - Load IAR file");
            info!("  POST /admin/archives/iar/save - Save IAR file");
            info!("  POST /admin/archives/oar/load - Load OAR file");
            info!("  POST /admin/archives/oar/save - Save OAR file");
            info!("  GET /admin/archives/jobs - List archive jobs");
            info!("  GET /admin/archives/jobs/:id - Get job status");
            info!("  POST /admin/archives/jobs/:id/cancel - Cancel job");
            info!("Console API endpoints:");
            info!("  GET /console/commands - List all commands");
            info!("  POST /console/execute - Execute command");
            info!("  GET /console/info - Server info");
            info!("  GET /console/regions - List regions");
            info!("  GET /console/regions/:name - Show region details");
            info!("  POST /console/regions/:name/restart - Restart region");
            info!("  GET /console/terrain/stats - Terrain statistics");
            info!("  POST /console/terrain/load - Load terrain");
            info!("  POST /console/terrain/save - Save terrain");
            info!("  POST /console/terrain/fill - Fill terrain");
            info!("  POST /console/shutdown - Shutdown server (disabled)");
            info!("  POST /console/users/kick - Kick user");
            info!("  POST /console/login/level - Set login level");
            info!("  POST /console/login/reset - Reset login");
            info!("  POST /console/login/text - Set login text");
            info!("  GET /console/connections - Show connections");
            info!("  GET /console/circuits - Show circuits");
            info!("  POST /console/objects/show - Show objects");
            info!("  POST /console/objects/delete - Delete objects");
            info!("  POST /console/objects/backup - Backup region");
            info!("  POST /console/scene/rotate - Rotate scene");
            info!("  POST /console/scene/scale - Scale scene");
            info!("  POST /console/scene/translate - Translate scene");
            info!("  POST /console/scene/force-update - Force update");
            info!("  POST /console/estates/create - Create estate");
            info!("  POST /console/estates/set-owner - Set estate owner");
            info!("  POST /console/estates/set-name - Rename estate");
            info!("  POST /console/estates/link-region - Link region to estate");
            info!("  POST /console/hypergrid/link - Create hypergrid link");
            info!("  POST /console/hypergrid/unlink - Remove hypergrid link");
            info!("  GET /console/hypergrid/links - Show hypergrid links");
            info!("  POST /console/hypergrid/mapping - Set link mapping");
            info!("  POST /console/assets/show - Show asset info");
            info!("  POST /console/assets/dump - Dump asset to file");
            info!("  POST /console/assets/delete - Delete asset");
            info!("  GET /console/fcache/status - Flotsam cache status");
            info!("  POST /console/fcache/clear - Clear flotsam cache");
            info!("  GET /console/fcache/assets - List cached assets");
            info!("  POST /console/fcache/expire - Expire cached assets");
            info!("  POST /console/xml/load - Load XML archive");
            info!("  POST /console/xml/save - Save XML archive");
            info!("  POST /console/regions/create - Create region");
            info!("  POST /console/regions/delete - Delete region");
            info!("  GET /console/regions/ratings - Show region ratings");
            info!("  GET /console/regions/neighbours - Show neighbours");
            info!("  GET /console/regions/inview - Show regions in view");
            info!("  POST /console/regions/change - Change region");
            info!("  POST /console/terrain/load-tile - Load terrain tile");
            info!("  POST /console/terrain/save-tile - Save terrain tile");
            info!("  POST /console/terrain/elevate - Elevate terrain");
            info!("  POST /console/terrain/lower - Lower terrain");
            info!("  POST /console/terrain/multiply - Multiply terrain");
            info!("  POST /console/terrain/bake - Bake terrain");
            info!("  POST /console/terrain/revert - Revert terrain");
            info!("  GET /console/terrain/show - Show terrain info");
            info!("  POST /console/terrain/effect - Apply terrain effect");
            info!("  POST /console/terrain/flip - Flip terrain");
            info!("  POST /console/terrain/rescale - Rescale terrain");
            info!("  POST /console/terrain/min - Set terrain min height");
            info!("  POST /console/terrain/max - Set terrain max height");
            info!("  POST /console/terrain/modify - Modify terrain");
            info!("  POST /console/general/quit - Quit server");
            info!("  GET /console/general/modules - Show modules");
            info!("  POST /console/general/command-script - Run script");
            info!("  GET /console/config/show - Show config");
            info!("  POST /console/config/get - Get config value");
            info!("  POST /console/config/set - Set config value");
            info!("  POST /console/log/level - Set log level");
            info!("  POST /console/general/force-gc - Force GC");
            info!("  POST /console/users/grid-user - Show grid user");
            info!("  GET /console/users/grid-users-online - Grid users online");
            info!("  POST /console/fcache/clearnegatives - Clear negatives");
            info!("  POST /console/fcache/cachedefaultassets - Cache defaults");
            info!("  POST /console/fcache/deletedefaultassets - Delete defaults");
            info!("  POST /console/parts/show - Show part by id/name/pos");
            info!("  POST /console/objects/dump - Dump object to XML");
            info!("  POST /console/objects/edit-scale - Edit prim scale");
            info!("  GET /console/comms/pending-objects - Show pending objects");
            info!("Security API endpoints:");
            info!("  GET /api/security/stats - Security statistics");
            info!("  GET /api/security/threats - Detected threats");
            info!("  GET /api/security/lockouts - Locked out IPs");
            info!("  POST /api/security/blacklist/:ip - Block IP");
            info!("  DELETE /api/security/blacklist/:ip - Unblock IP");
            info!("  GET /api/security/ziti/status - OpenZiti status");
            info!("Skill API endpoints:");
            info!("  GET /skills - List all skills by domain");
            info!("  GET /skills/dashboard - Maturity dashboard");
            info!("  GET /skills/search?q=<query> - Search skills");
            info!("  GET /skills/{{domain}} - List domain skills");
            info!("  GET /skills/{{domain}}/{{skill_id}} - Skill detail");

            match axum_server::bind(addr)
                .serve(combined_router.into_make_service())
                .await
            {
                Ok(_) => info!("Admin API server shut down gracefully"),
                Err(e) => error!("Admin API server failed: {}", e),
            }
        });
    } else {
        // NO SQLITE FALLBACK: Force database manager initialization to succeed
        error!("❌ CRITICAL: Database manager failed to initialize - PostgreSQL/MariaDB connections must be fixed");
        error!("❌ EADS VIOLATION: SQLite fallback has been eliminated from OpenSim Next");
        error!("❌ Admin API will not start without proper database backend");
        panic!("Database initialization failed - PostgreSQL/MariaDB required for EADS compliance");
    }
    tokio::spawn(async move {
        // CORS preflight handler
        async fn cors_handler() -> impl IntoResponse {
            (
                StatusCode::OK,
                [
                    ("Access-Control-Allow-Origin", "*"),
                    ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
                    (
                        "Access-Control-Allow-Headers",
                        "Authorization, Content-Type",
                    ),
                    ("Access-Control-Max-Age", "86400"),
                ],
            )
        }

        // Simple health check without authentication for browser testing
        async fn simple_health() -> impl IntoResponse {
            (
                StatusCode::OK,
                [
                    ("Content-Type", "application/json"),
                    ("Access-Control-Allow-Origin", "*"),
                    ("Access-Control-Allow-Methods", "GET, OPTIONS"),
                    (
                        "Access-Control-Allow-Headers",
                        "Authorization, Content-Type",
                    ),
                ],
                r#"{"status":"ok","message":"OpenSim Next server is running"}"#,
            )
        }

        let app = Router::new()
            .route("/metrics", get(metrics_handler).options(cors_handler))
            .route("/health", get(health_handler).options(cors_handler))
            .route("/info", get(instance_info_handler).options(cors_handler))
            .route("/ping", get(simple_health)) // No auth required
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            ))
            .with_state(app_state.clone())
            // Add fallback route without auth for OPTIONS
            .fallback(cors_handler);
        let addr = SocketAddr::from(([0, 0, 0, 0], metrics_port));
        info!("Starting monitoring HTTP server on {}", addr);
        info!("Available endpoints:");
        info!("  GET /metrics - Prometheus metrics (requires API key)");
        info!("  GET /health  - Health status (requires API key)");
        info!("  GET /info    - Instance information (requires API key)");

        match axum_server::bind(addr).serve(app.into_make_service()).await {
            Ok(_) => info!("Monitoring HTTP server shut down gracefully"),
            Err(e) => error!("Monitoring HTTP server failed: {}", e),
        }
    });

    // Create simulation engine
    let simulation_engine = Arc::new(SimulationEngine::new(
        region_manager.clone(),
        state_manager.clone(),
    ));

    // Create default region
    let region_id = RegionId(1);
    let region_config = RegionConfig {
        name: "Default Region".to_string(),
        size: (256, 256),
        location: (1000, 1000),
        terrain: opensim_next::region::terrain::TerrainConfig::default(),
        physics: opensim_next::ffi::physics::PhysicsConfig::default(),
        max_entities: 1000,
    };
    let _region = region_manager
        .create_region(region_id, region_config)
        .await?;
    info!("Created default region: {}", region_id);

    // Start simulation engine - TEMPORARILY DISABLED FOR WEB INTERFACE TESTING
    // simulation_engine.start().await?;
    info!("Simulation engine disabled for web interface testing");

    // Start web client server
    let web_client_port = env::var("OPENSIM_WEB_CLIENT_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    let mut web_client_server = WebClientServer::new(web_client_port);

    if let Some(ref db) = database_manager {
        match initialize_ai_api(db.clone(), None, None).await {
            Ok(ai_state) => {
                let ai_router = create_ai_api_router(ai_state);
                web_client_server = web_client_server.with_ai_router(ai_router);
                info!("AI API initialized and mounted on web client server");
            }
            Err(e) => {
                warn!("AI API initialization failed (non-fatal): {}", e);
            }
        }
    }

    tokio::spawn(async move {
        if let Err(e) = web_client_server.start().await {
            error!("Web client server error: {}", e);
        }
    });
    info!("Web client server started on 0.0.0.0:{}", web_client_port);

    // Start Firestorm-compatible login service for Second Life viewer compatibility
    // This is the primary login endpoint that handles Second Life viewer connections
    if user_account_database.is_some() {
        // Enable login server when database is available
        use opensim_next::config::login::LoginConfigManager;

        // Create login configuration manager (reads OPENSIM_LOGIN_PORT env var)
        let config_manager =
            LoginConfigManager::load_from_env().unwrap_or_else(|_| LoginConfigManager::new());

        // CRITICAL FIX: Reuse the SAME login_session_manager that UdpServer uses (created on line 453)
        // DO NOT create a new instance here - that creates a separate HashMap storage!

        // Create AvatarService for avatar appearance management
        // Matches: OpenSim LLLoginService.cs:117, 229-230
        let avatar_service = Arc::new(AvatarService::new(
            database_manager.as_ref().unwrap().clone(),
        ));
        info!("[AVATAR SERVICE]: Avatar service initialized for login");

        // Create the new LoginService with all required components
        let mut login_service = LoginService::new(
            config_manager,
            login_session_manager.clone(),
            user_account_database.as_ref().unwrap().clone(),
            avatar_service,
        );

        // CRITICAL FIX: Connect CircuitCodeRegistry to LoginService for XMLRPC->UDP coordination
        login_service.set_circuit_registry(Arc::new(circuit_codes.clone()));
        info!("🔗 Connected CircuitCodeRegistry to LoginService for XMLRPC->UDP coordination");

        // Connect the CapsManager to LoginService for session coordination
        login_service.set_caps_manager(caps_manager.clone());
        info!("Connected CapsManager to LoginService for session coordination");

        // Phase 83.2: Connect PostgreSQL pool to LoginService for inventory queries
        if let Some(ref db_manager) = database_manager {
            if let Ok(pool) = db_manager.legacy_pool() {
                login_service.set_db_pool(Arc::new(pool.clone()));
                info!("📦 Phase 83.2: Connected PostgreSQL pool to LoginService for inventory queries");
            } else {
                warn!("📦 Phase 83.2: No PostgreSQL pool available, login will use generated inventory");
            }
        }

        if let Some(ref hg) = hypergrid_manager {
            let mut urls = std::collections::HashMap::new();
            urls.insert("SRV_HomeURI".to_string(), hg.config().home_uri.clone());
            urls.insert(
                "SRV_GatekeeperURI".to_string(),
                hg.config().gatekeeper_uri.clone(),
            );
            let base = &hg.config().home_uri;
            urls.insert("SRV_InventoryServerURI".to_string(), format!("{}/hg", base));
            urls.insert("SRV_AssetServerURI".to_string(), base.clone());
            urls.insert("SRV_ProfileServerURI".to_string(), base.clone());
            urls.insert("SRV_FriendsServerURI".to_string(), base.clone());
            urls.insert("SRV_IMServerURI".to_string(), base.clone());
            login_service.set_hg_service_urls(urls);
            login_service.set_uas_service(hg.uas().clone());
            login_service.set_gatekeeper_service(hg.gatekeeper().clone());
            info!(
                "[HG] Service URLs + UAS + Gatekeeper wired into LoginService (home={})",
                base
            );
        }

        login_service.set_security_manager(udp_security_manager.clone());
        login_service.set_readiness_tracker(readiness_tracker.clone());

        // UDP server for circuit handling is already started above in main spawn task
        // Skipping duplicate UDP server startup to prevent port conflict
        info!(
            "UDP server already running on port {} - skipping duplicate startup",
            login_port
        );

        match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", login_port)).await {
            Ok(test_listener) => {
                // Port is available, close test listener and start real server
                drop(test_listener);

                // Create HTTP server for XMLRPC login requests and capabilities
                let login_service_arc = Arc::new(login_service);

                // Import the caps router
                use opensim_next::caps::router::create_caps_router;

                // Create login routes with legacy viewer compatibility
                // Cool Viewer, Singularity, and other legacy viewers use various paths
                let login_router = axum::Router::new()
                    .route(
                        "/",
                        axum::routing::post(opensim_next::login_service::handle_login_post),
                    ) // XMLRPC login at root
                    .route(
                        "/login",
                        axum::routing::post(opensim_next::login_service::handle_login_post),
                    )
                    .route(
                        "/login",
                        axum::routing::get(opensim_next::login_service::handle_login_page),
                    )
                    // Legacy viewer path aliases (Cool Viewer, Singularity, etc.)
                    .route(
                        "/xmlrpc.php",
                        axum::routing::post(opensim_next::login_service::handle_login_post),
                    )
                    .route(
                        "/login.cgi",
                        axum::routing::post(opensim_next::login_service::handle_login_post),
                    )
                    .route(
                        "/cgi-bin/login.cgi",
                        axum::routing::post(opensim_next::login_service::handle_login_post),
                    )
                    .route(
                        "/xmlrpc",
                        axum::routing::post(opensim_next::login_service::handle_login_post),
                    )
                    .route(
                        "/get_grid_info",
                        axum::routing::get(opensim_next::login_service::handle_grid_info),
                    )
                    .route(
                        "/json_grid_info",
                        axum::routing::get(opensim_next::login_service::handle_grid_info_json),
                    )
                    .route(
                        "/welcome",
                        axum::routing::get(opensim_next::login_service::handle_splash_page),
                    )
                    .route(
                        "/splash.png",
                        axum::routing::get(opensim_next::login_service::handle_splash_image),
                    )
                    .with_state(login_service_arc);

                let currency_state = opensim_next::login_service::CurrencyState {
                    database_manager: database_manager.clone(),
                };
                let currency_router = axum::Router::new()
                    .route(
                        "/currency.php",
                        axum::routing::post(opensim_next::login_service::handle_currency_php),
                    )
                    .route(
                        "/landtool.php",
                        axum::routing::post(opensim_next::login_service::handle_landtool_php),
                    )
                    .with_state(currency_state);

                let helo_router = axum::Router::new()
                    .route(
                        "/helo",
                        axum::routing::get(
                            opensim_next::services::robust::helo_handler::handle_helo_get,
                        ),
                    )
                    .route(
                        "/helo",
                        axum::routing::head(
                            opensim_next::services::robust::helo_handler::handle_helo_head,
                        ),
                    )
                    .route(
                        "/helo/",
                        axum::routing::get(
                            opensim_next::services::robust::helo_handler::handle_helo_get,
                        ),
                    )
                    .route(
                        "/helo/",
                        axum::routing::head(
                            opensim_next::services::robust::helo_handler::handle_helo_head,
                        ),
                    );

                let mut app = login_router
                    .merge(create_caps_router().with_state(caps_state))
                    .merge(helo_router)
                    .merge(currency_router);

                if let Some(ref db_conn) = database_connection {
                    let map_tile_state = MapTileState {
                        region_registry: region_registry.clone(),
                        database_connection: db_conn.clone(),
                    };
                    let map_router = axum::Router::new()
                        .route("/t/:path", axum::routing::get(handle_map_tile_request))
                        .with_state(map_tile_state);
                    app = app.merge(map_router);
                    info!("[MAP] World map tile endpoint enabled at /t/map-<zoom>-<x>-<y>-objects.jpg");
                }

                if let (Some(av_states), Some(parcels_ref)) =
                    (map_items_avatar_states.clone(), map_items_parcels.clone())
                {
                    let map_items_state = MapItemsState {
                        avatar_states: av_states,
                        parcels: parcels_ref,
                        region_registry: region_registry.clone(),
                    };
                    let map_items_router = axum::Router::new()
                        .route(
                            "/MAP/MapItems/:regionhandle",
                            axum::routing::get(handle_map_items_request),
                        )
                        .with_state(map_items_state);
                    app = app.merge(map_items_router);
                    info!("[MAP] MapItems endpoint enabled at /MAP/MapItems/{{regionhandle}}");
                }

                if let Some(ref hg) = hypergrid_manager {
                    let uas_state = opensim_next::services::robust::UasState {
                        uas_service: hg.uas().clone(),
                    };
                    let uas_router = opensim_next::services::robust::create_uas_router(uas_state);
                    app = app.merge(uas_router);
                    info!("[HG] UAS callback endpoints enabled: POST /useragent, POST /homeagent/{{agent_id}}");

                    let gk_state = opensim_next::services::robust::GatekeeperState {
                        gatekeeper_service: hg.gatekeeper().clone(),
                        circuit_code_registry: Some(circuit_codes.clone()),
                        session_manager: Some(login_session_manager.clone()),
                        caps_manager: Some(caps_manager.clone()),
                    };
                    let gk_router =
                        opensim_next::services::robust::create_gatekeeper_router(gk_state);
                    app = app.merge(gk_router);
                    info!("[HG] Gatekeeper endpoints enabled on port {}: POST /gatekeeper, POST /foreignagent/{{agent_id}}", login_port);
                }

                {
                    let tracker = readiness_tracker.clone();
                    let ready_router = axum::Router::new().route(
                        "/ready",
                        axum::routing::get(move || {
                            let tracker = tracker.clone();
                            async move {
                                if tracker.is_login_ready() {
                                    (
                                        axum::http::StatusCode::OK,
                                        axum::Json(serde_json::json!({
                                            "ready": true,
                                            "service": "region",
                                            "uptime_secs": tracker.uptime_secs()
                                        })),
                                    )
                                } else {
                                    let pending: Vec<&str> = tracker
                                        .status_breakdown()
                                        .iter()
                                        .filter(|(_, ready)| !ready)
                                        .map(|(name, _)| *name)
                                        .collect();
                                    (
                                        axum::http::StatusCode::SERVICE_UNAVAILABLE,
                                        axum::Json(serde_json::json!({
                                            "ready": false,
                                            "service": "region",
                                            "pending": pending,
                                            "uptime_secs": tracker.uptime_secs()
                                        })),
                                    )
                                }
                            }
                        }),
                    );
                    app = app.merge(ready_router);
                }

                app = app.fallback(|req: axum::extract::Request| async move {
                    let method = req.method().clone();
                    let uri = req.uri().clone();
                    let headers = req.headers().clone();
                    let content_len = headers
                        .get("content-length")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("unknown");
                    let content_type = headers
                        .get("content-type")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("unknown");
                    warn!(
                        "[FALLBACK] Unmatched request: {} {} (content-type={}, content-length={})",
                        method, uri, content_type, content_len
                    );
                    (axum::http::StatusCode::NOT_FOUND, "Not found")
                });

                let bind_port = login_port;
                tokio::spawn(async move {
                    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], bind_port));
                    match axum_server::bind(addr).serve(app.into_make_service()).await {
                        Ok(_) => info!("Login service shut down gracefully"),
                        Err(e) => error!("Login service error: {}", e),
                    }
                });

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                info!("Login server started on 0.0.0.0:{}", login_port);
            }
            Err(e) => {
                error!("Cannot bind to port {} for login server: {}", login_port, e);
                error!("Make sure no other service is using port {}", login_port);
                return Err(anyhow!("Login server port binding failed: {}", e));
            }
        }
    } else {
        error!("❌ No user account database available! Cannot start login server.");
        warn!("Server will continue running without login capabilities.");
    }
    // Wait a moment for services to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    info!("🎉 OpenSim Next server components started successfully!");
    info!("🌟 Revolutionary Multi-Database Virtual World Server Ready!");
    info!("");
    info!("📡 Available services:");
    info!(
        "  🌐 Frontend Dashboard: http://0.0.0.0:{}",
        web_client_port
    );
    info!(
        "  🔗 API Endpoints: http://0.0.0.0:{} (requires API key)",
        http_config.metrics_port
    );
    info!("  🔧 Admin API (Robust Commands): http://0.0.0.0:9200 (requires API key)");
    info!("  📡 WebSocket Server: ws://0.0.0.0:9001");
    info!("  Second Life Viewers: opensim://0.0.0.0:{}", login_port);
    info!("  🌍 Hypergrid Protocol: http://0.0.0.0:8002");
    info!("  ⚙️  Physics Engines: Multiple engines available");
    info!("  🗄️ Database: Multi-backend support (PostgreSQL, MySQL, SQLite)");
    info!("");
    info!("🔧 Server Configuration:");
    info!("  Instance ID: {}", http_config.instance_id);
    info!(
        "  API Key: {} (change via OPENSIM_API_KEY)",
        if http_config.api_key == "default-key-change-me" {
            "default-key-change-me (⚠️  CHANGE FOR PRODUCTION)"
        } else {
            "configured"
        }
    );
    info!(
        "  Database: {}",
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "SQLite (default)".to_string())
    );
    info!("");
    info!("🚀 Access your revolutionary virtual world:");
    info!("  👉 Open browser: http://localhost:{}", web_client_port);
    info!("  Connect viewer: opensim://localhost:{}", login_port);
    info!(
        "  👉 Test API: curl -H 'X-API-Key: {}' http://localhost:{}/health",
        http_config.api_key, http_config.metrics_port
    );

    let instance_dir = opensim_next::instance_manager::discovery::resolve_instance_dir();
    let instances_base = opensim_next::instance_manager::discovery::resolve_instances_base_dir();
    let svc_mode_for_discovery =
        env::var("OPENSIM_SERVICE_MODE").unwrap_or_else(|_| "standalone".to_string());

    if env::var("OPENSIM_DISABLE_CONTROLLER").unwrap_or_default() != "true" {
        let controller_port = env::var("OPENSIM_CONTROLLER_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or_else(|| {
                opensim_next::instance_manager::find_available_controller_port(
                    &instances_base,
                    9300,
                    9320,
                )
            });
        spawn_embedded_controller(controller_port).await;

        let discovery_info = opensim_next::instance_manager::RunningInstanceInfo::new(
            env::var("OPENSIM_INSTANCE_ID").unwrap_or_else(|_| {
                instance_dir
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase()
                    .replace(' ', "-")
            }),
            svc_mode_for_discovery.clone(),
            controller_port,
            login_port,
            0,
        );
        if let Err(e) = opensim_next::instance_manager::write_discovery_file(
            &instance_dir,
            &svc_mode_for_discovery,
            &discovery_info,
        ) {
            warn!("Failed to write discovery file: {}", e);
        }
    } else if let Ok(controller_url) = env::var("OPENSIM_CONTROLLER_URL") {
        // External controller: announce to it
        let instance_id = env::var("OPENSIM_INSTANCE_ID").unwrap_or_else(|_| {
            env::var("OPENSIM_INSTANCE_DIR")
                .map(|d| {
                    std::path::Path::new(&d)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase()
                        .replace(' ', "-")
                })
                .unwrap_or_else(|_| "standalone".to_string())
        });
        let svc_mode =
            env::var("OPENSIM_SERVICE_MODE").unwrap_or_else(|_| "standalone".to_string());
        let announce_login_port = login_port;

        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let announcement = serde_json::json!({
                "instance_id": instance_id,
                "service_mode": svc_mode,
                "ports": {
                    "login": announce_login_port,
                    "admin": 9200u16,
                    "metrics": 9100u16,
                },
                "region_count": 0u32,
                "capabilities": ["lludp", "http", "caps"],
                "version": env!("CARGO_PKG_VERSION"),
                "host": "localhost",
            });

            for attempt in 0..3u32 {
                match client
                    .post(format!("{}/api/instance/announce", controller_url))
                    .json(&announcement)
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => {
                        info!("Announced to controller at {}", controller_url);
                        break;
                    }
                    Ok(resp) => {
                        warn!(
                            "Controller announce failed (attempt {}): HTTP {}",
                            attempt + 1,
                            resp.status()
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Controller announce failed (attempt {}): {}",
                            attempt + 1,
                            e
                        );
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
            }

            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                let heartbeat = serde_json::json!({
                    "instance_id": instance_id,
                    "status": "running",
                    "active_users": 0u32,
                    "active_regions": 0u32,
                    "uptime_seconds": 0u64,
                    "cpu_usage": 0.0f64,
                    "memory_usage_mb": 0u64,
                });
                let _ = client
                    .post(format!("{}/api/instance/heartbeat", controller_url))
                    .json(&heartbeat)
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await;
            }
        });
    }

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");

    opensim_next::instance_manager::remove_discovery_file(&instance_dir, &svc_mode_for_discovery);

    // Graceful shutdown
    info!("Shutting down OpenSim Next Server...");
    if ziti_manager.is_running() {
        ziti_manager.stop().await.ok();
    }
    simulation_engine.shutdown().await?;
    monitoring.stop().await?;

    info!("OpenSim Next Server shutdown complete");
    Ok(())
}

async fn run_fsassets_migration(fsassets_root: &str, batch_size: i64, verify: bool) -> Result<()> {
    use opensim_next::asset::fsassets::{FSAssetsConfig, FSAssetsStorage};
    use opensim_next::asset::fsassets_migrate::FSAssetsMigrator;
    use opensim_next::database::multi_backend::DatabaseConnection;
    use std::path::PathBuf;

    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://opensim@localhost/gaiagrid".to_string());

    println!("=== FSAssets Migration Tool ===");
    println!("Database:      {}", db_url);
    println!("FSAssets root: {}", fsassets_root);
    println!("Batch size:    {}", batch_size);
    println!();

    println!("Connecting to database...");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    println!("Creating fsassets table if not exists...");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS fsassets (
            id UUID PRIMARY KEY,
            hash CHAR(64) NOT NULL,
            type INTEGER NOT NULL DEFAULT 0,
            create_time INTEGER NOT NULL DEFAULT 0,
            access_time INTEGER NOT NULL DEFAULT 0,
            asset_flags INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_fsassets_hash ON fsassets(hash)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_fsassets_access_time ON fsassets(access_time)")
        .execute(&pool)
        .await?;

    let (asset_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM assets")
        .fetch_one(&pool)
        .await?;
    let (fs_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fsassets")
        .fetch_one(&pool)
        .await?;

    println!("Legacy assets:    {}", asset_count);
    println!("FSAssets entries: {}", fs_count);

    if fs_count >= asset_count && fs_count > 0 {
        println!(
            "Migration appears complete ({} >= {}). Nothing to do.",
            fs_count, asset_count
        );
        if !verify {
            return Ok(());
        }
    }

    let config = FSAssetsConfig {
        base_directory: PathBuf::from(fsassets_root).join("data"),
        spool_directory: PathBuf::from(fsassets_root).join("tmp"),
        use_osgrid_format: false,
        compression_level: 6,
    };

    println!("Initializing FSAssets storage at {}...", fsassets_root);
    let storage = Arc::new(FSAssetsStorage::new(config)?);
    let _bg_writer = storage.start_background_writer();

    let connection = Arc::new(DatabaseConnection::PostgreSQL(pool.clone()));
    let migrator = FSAssetsMigrator::new(storage.clone(), connection);

    if fs_count < asset_count {
        println!();
        println!(
            "Starting migration of {} assets (batch_size={})...",
            asset_count, batch_size
        );
        println!("This will take several minutes for large databases.");
        println!();

        let stats = migrator.migrate_all(batch_size).await?;

        println!();
        println!("=== Migration Complete ===");
        println!("Total assets:     {}", stats.total_assets);
        println!("Migrated:         {}", stats.migrated);
        println!("Skipped (exists): {}", stats.skipped_existing);
        println!("Skipped (null):   {}", stats.skipped_null_data);
        println!("Errors:           {}", stats.errors);
        println!(
            "Data processed:   {:.2} GB",
            stats.bytes_processed as f64 / (1024.0 * 1024.0 * 1024.0)
        );
        println!("Elapsed:          {:.1}s", stats.elapsed_secs);
        if stats.elapsed_secs > 0.0 {
            println!(
                "Throughput:       {:.0} assets/sec, {:.1} MB/sec",
                stats.total_assets as f64 / stats.elapsed_secs,
                stats.bytes_processed as f64 / (1024.0 * 1024.0) / stats.elapsed_secs
            );
        }
    }

    println!();
    println!("Draining spool (compressing remaining files)...");
    loop {
        storage.notify_spool();
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let spool_dir = PathBuf::from(fsassets_root).join("tmp");
        let count = std::fs::read_dir(&spool_dir)
            .map(|d| {
                d.filter(|e| {
                    e.as_ref()
                        .map(|e| e.path().extension().map(|x| x == "asset").unwrap_or(false))
                        .unwrap_or(false)
                })
                .count()
            })
            .unwrap_or(0);
        if count == 0 {
            println!("Spool drained.");
            break;
        }
        println!("  {} files remaining in spool...", count);
    }

    if verify {
        println!();
        println!("=== Verification ===");
        let (ac, fc, issues) = migrator.verify_migration().await?;
        println!("Legacy assets: {}", ac);
        println!("FSAssets:      {}", fc);
        if issues.is_empty() {
            println!("Status:        OK — no issues found");
        } else {
            println!("Issues ({}):", issues.len());
            for issue in &issues {
                println!("  - {}", issue);
            }
        }

        let (unique,): (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT hash) FROM fsassets")
            .fetch_one(&pool)
            .await?;
        let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fsassets")
            .fetch_one(&pool)
            .await?;
        if total > 0 {
            let dedup_pct = (1.0 - unique as f64 / total as f64) * 100.0;
            println!(
                "Deduplication: {} unique hashes / {} total = {:.1}% savings",
                unique, total, dedup_pct
            );
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration first
    load_config();

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    let mode_env = env::var("OPENSIM_SERVICE_MODE").unwrap_or_default();

    match cli.command {
        Some(Commands::Setup {
            preset,
            non_interactive,
            reconfigure,
        }) => run_setup_wizard(preset, non_interactive, reconfigure).await,
        Some(Commands::Preflight { instance }) => run_preflight(&instance).await,
        Some(Commands::MigrateFsassets {
            fsassets_root,
            batch_size,
            verify,
        }) => run_fsassets_migration(&fsassets_root, batch_size, verify).await,
        Some(Commands::Start { mode }) => {
            let effective_mode = if !mode_env.is_empty() { mode_env } else { mode };
            match effective_mode.as_str() {
                "robust" => start_robust_mode().await,
                "grid" => start_server_with_mode("grid").await,
                "controller" => start_controller_mode().await,
                _ => start_server_with_mode("standalone").await,
            }
        }
        None => {
            if !mode_env.is_empty() {
                match mode_env.as_str() {
                    "robust" => return start_robust_mode().await,
                    "grid" => return start_server_with_mode("grid").await,
                    "controller" => return start_controller_mode().await,
                    _ => {}
                }
            }
            if std::path::Path::new("./config/OpenSim.ini").exists() {
                info!("Existing configuration found, starting server...");
                start_server_with_mode("standalone").await
            } else {
                info!("No configuration found, running setup wizard...");
                println!("Welcome to OpenSim Next!");
                println!("No configuration detected. Running initial setup...");
                println!();
                run_setup_wizard(None, false, false).await?;
                println!();
                println!("Setup complete! Starting server...");
                start_server_with_mode("standalone").await
            }
        }
    }
} // Force rebuild Sat Jan 17 05:40:46 CST 2026
