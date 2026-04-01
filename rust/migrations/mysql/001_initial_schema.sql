-- MySQL/MariaDB initial schema for OpenSim Next
-- Compatible with both MySQL 8.0+ and MariaDB 10.5+

-- Users and authentication
CREATE TABLE IF NOT EXISTS users (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    first_name VARCHAR(64) NOT NULL,
    last_name VARCHAR(64) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    salt VARCHAR(255) NOT NULL,
    user_level INT DEFAULT 0,
    user_flags INT DEFAULT 0,
    user_title VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP NULL,
    home_region CHAR(36),
    home_location_x FLOAT DEFAULT 128.0,
    home_location_y FLOAT DEFAULT 128.0,
    home_location_z FLOAT DEFAULT 21.0,
    home_look_at_x FLOAT DEFAULT 1.0,
    home_look_at_y FLOAT DEFAULT 0.0,
    home_look_at_z FLOAT DEFAULT 0.0,
    profile_about TEXT,
    profile_first_text TEXT,
    profile_image CHAR(36),
    profile_partner CHAR(36),
    profile_url VARCHAR(255),
    profile_wants_to_mask INT DEFAULT 0,
    profile_wants_to_text VARCHAR(255),
    profile_skills_mask INT DEFAULT 0,
    profile_skills_text VARCHAR(255),
    profile_languages VARCHAR(255),
    active BOOLEAN DEFAULT TRUE,
    INDEX idx_users_email (email),
    INDEX idx_users_name (first_name, last_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Regions
CREATE TABLE IF NOT EXISTS regions (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    region_name VARCHAR(128) NOT NULL UNIQUE,
    location_x INT NOT NULL,
    location_y INT NOT NULL,
    size_x INT DEFAULT 256,
    size_y INT DEFAULT 256,
    external_host_name VARCHAR(255),
    external_port INT,
    internal_host_name VARCHAR(255),
    internal_port INT,
    map_texture CHAR(36),
    terrain_texture_1 CHAR(36),
    terrain_texture_2 CHAR(36),
    terrain_texture_3 CHAR(36),
    terrain_texture_4 CHAR(36),
    elevation_1_nw FLOAT DEFAULT 10.0,
    elevation_2_ne FLOAT DEFAULT 10.0,
    elevation_1_sw FLOAT DEFAULT 10.0,
    elevation_2_se FLOAT DEFAULT 10.0,
    water_height FLOAT DEFAULT 20.0,
    terrain_raise_limit FLOAT DEFAULT 100.0,
    terrain_lower_limit FLOAT DEFAULT -100.0,
    use_estate_sun BOOLEAN DEFAULT TRUE,
    sun_position FLOAT DEFAULT 0.0,
    covenant CHAR(36),
    sandbox BOOLEAN DEFAULT FALSE,
    public_access BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_seen TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    physics_engine VARCHAR(32) DEFAULT 'ODE',
    max_prims INT DEFAULT 100000,
    max_agents INT DEFAULT 100,
    INDEX idx_regions_location (location_x, location_y),
    INDEX idx_regions_name (region_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Assets  
CREATE TABLE IF NOT EXISTS assets (
    id CHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    asset_type INT NOT NULL,
    local BOOLEAN DEFAULT FALSE,
    temporary BOOLEAN DEFAULT FALSE,
    data LONGBLOB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    creator_id CHAR(36),
    asset_flags INT DEFAULT 0,
    content_type VARCHAR(255),
    size_bytes BIGINT DEFAULT 0,
    INDEX idx_assets_type (asset_type),
    INDEX idx_assets_creator (creator_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Inventory
CREATE TABLE IF NOT EXISTS inventory_folders (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    agent_id CHAR(36) NOT NULL,
    parent_folder_id CHAR(36),
    folder_name VARCHAR(255) NOT NULL,
    folder_type INT DEFAULT 0,
    version INT DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_inventory_folders_agent (agent_id),
    INDEX idx_inventory_folders_parent (parent_folder_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS inventory_items (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    asset_id CHAR(36) NOT NULL,
    asset_type INT NOT NULL,
    folder_id CHAR(36) NOT NULL,
    owner_id CHAR(36) NOT NULL,
    creator_id CHAR(36) NOT NULL,
    item_name VARCHAR(255) NOT NULL,
    item_description TEXT,
    next_permissions INT DEFAULT 0,
    current_permissions INT DEFAULT 0,
    base_permissions INT DEFAULT 0,
    everyone_permissions INT DEFAULT 0,
    group_permissions INT DEFAULT 0,
    group_id CHAR(36),
    group_owned BOOLEAN DEFAULT FALSE,
    sale_price INT DEFAULT 0,
    sale_type INT DEFAULT 0,
    flags INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    creation_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_inventory_items_folder (folder_id),
    INDEX idx_inventory_items_owner (owner_id),
    INDEX idx_inventory_items_asset (asset_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Estates
CREATE TABLE IF NOT EXISTS estates (
    id INT AUTO_INCREMENT PRIMARY KEY,
    estate_name VARCHAR(255) NOT NULL,
    estate_owner CHAR(36) NOT NULL,
    parent_estate_id INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_estates_owner (estate_owner)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Session management
CREATE TABLE IF NOT EXISTS sessions (
    session_id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    region_id CHAR(36),
    secure_session_id CHAR(36) NOT NULL,
    circuit_code INT NOT NULL,
    login_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    logout_time TIMESTAMP NULL,
    client_version VARCHAR(255),
    last_position_x FLOAT DEFAULT 128.0,
    last_position_y FLOAT DEFAULT 128.0,
    last_position_z FLOAT DEFAULT 21.0,
    last_look_at_x FLOAT DEFAULT 1.0,
    last_look_at_y FLOAT DEFAULT 0.0,
    last_look_at_z FLOAT DEFAULT 0.0,
    active BOOLEAN DEFAULT TRUE,
    INDEX idx_sessions_user (user_id),
    INDEX idx_sessions_active (active)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;