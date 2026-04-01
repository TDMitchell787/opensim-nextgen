-- PostgreSQL initial schema for OpenSim Next
-- Optimized for PostgreSQL-specific features

-- Users and authentication
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR(64) NOT NULL,
    last_name VARCHAR(64) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    salt VARCHAR(255) NOT NULL,
    user_level INTEGER DEFAULT 0,
    user_flags INTEGER DEFAULT 0,
    user_title VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login TIMESTAMP WITH TIME ZONE,
    home_region UUID,
    home_location_x REAL DEFAULT 128.0,
    home_location_y REAL DEFAULT 128.0,
    home_location_z REAL DEFAULT 21.0,
    home_look_at_x REAL DEFAULT 1.0,
    home_look_at_y REAL DEFAULT 0.0,
    home_look_at_z REAL DEFAULT 0.0,
    profile_about TEXT,
    profile_first_text TEXT,
    profile_image UUID,
    profile_partner UUID,
    profile_url VARCHAR(255),
    profile_wants_to_mask INTEGER DEFAULT 0,
    profile_wants_to_text VARCHAR(255),
    profile_skills_mask INTEGER DEFAULT 0,
    profile_skills_text VARCHAR(255),
    profile_languages VARCHAR(255),
    active BOOLEAN DEFAULT TRUE
);

-- Regions
CREATE TABLE IF NOT EXISTS regions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    region_name VARCHAR(128) NOT NULL UNIQUE,
    location_x INTEGER NOT NULL,
    location_y INTEGER NOT NULL,
    size_x INTEGER DEFAULT 256,
    size_y INTEGER DEFAULT 256,
    external_host_name VARCHAR(255),
    external_port INTEGER,
    internal_host_name VARCHAR(255),
    internal_port INTEGER,
    map_texture UUID,
    terrain_texture_1 UUID,
    terrain_texture_2 UUID,
    terrain_texture_3 UUID,
    terrain_texture_4 UUID,
    elevation_1_nw REAL DEFAULT 10.0,
    elevation_2_ne REAL DEFAULT 10.0,
    elevation_1_sw REAL DEFAULT 10.0,
    elevation_2_se REAL DEFAULT 10.0,
    water_height REAL DEFAULT 20.0,
    terrain_raise_limit REAL DEFAULT 100.0,
    terrain_lower_limit REAL DEFAULT -100.0,
    use_estate_sun BOOLEAN DEFAULT TRUE,
    sun_position REAL DEFAULT 0.0,
    covenant UUID,
    sandbox BOOLEAN DEFAULT FALSE,
    public_access BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_seen TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    physics_engine VARCHAR(32) DEFAULT 'ODE',
    max_prims INTEGER DEFAULT 100000,
    max_agents INTEGER DEFAULT 100
);

-- Assets
CREATE TABLE IF NOT EXISTS assets (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    asset_type INTEGER NOT NULL,
    local BOOLEAN DEFAULT FALSE,
    temporary BOOLEAN DEFAULT FALSE,
    data BYTEA,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    creator_id UUID,
    asset_flags INTEGER DEFAULT 0,
    content_type VARCHAR(255),
    size_bytes BIGINT DEFAULT 0
);

-- Inventory
CREATE TABLE IF NOT EXISTS inventory_folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL,
    parent_folder_id UUID,
    folder_name VARCHAR(255) NOT NULL,
    folder_type INTEGER DEFAULT 0,
    version INTEGER DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS inventory_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL,
    asset_type INTEGER NOT NULL,
    folder_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    creator_id UUID NOT NULL,
    item_name VARCHAR(255) NOT NULL,
    item_description TEXT,
    next_permissions INTEGER DEFAULT 0,
    current_permissions INTEGER DEFAULT 0,
    base_permissions INTEGER DEFAULT 0,
    everyone_permissions INTEGER DEFAULT 0,
    group_permissions INTEGER DEFAULT 0,
    group_id UUID,
    group_owned BOOLEAN DEFAULT FALSE,
    sale_price INTEGER DEFAULT 0,
    sale_type INTEGER DEFAULT 0,
    flags INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    creation_date TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Estates
CREATE TABLE IF NOT EXISTS estates (
    id SERIAL PRIMARY KEY,
    estate_name VARCHAR(255) NOT NULL,
    estate_owner UUID NOT NULL,
    parent_estate_id INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Session management
CREATE TABLE IF NOT EXISTS sessions (
    session_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    region_id UUID,
    secure_session_id UUID NOT NULL,
    circuit_code INTEGER NOT NULL,
    login_time TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    logout_time TIMESTAMP WITH TIME ZONE,
    client_version VARCHAR(255),
    last_position_x REAL DEFAULT 128.0,
    last_position_y REAL DEFAULT 128.0,
    last_position_z REAL DEFAULT 21.0,
    last_look_at_x REAL DEFAULT 1.0,
    last_look_at_y REAL DEFAULT 0.0,
    last_look_at_z REAL DEFAULT 0.0,
    active BOOLEAN DEFAULT TRUE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_name ON users(first_name, last_name);
CREATE INDEX IF NOT EXISTS idx_regions_location ON regions(location_x, location_y);
CREATE INDEX IF NOT EXISTS idx_assets_type ON assets(asset_type);
CREATE INDEX IF NOT EXISTS idx_inventory_folders_agent ON inventory_folders(agent_id);
CREATE INDEX IF NOT EXISTS idx_inventory_folders_parent ON inventory_folders(parent_folder_id);
CREATE INDEX IF NOT EXISTS idx_inventory_items_folder ON inventory_items(folder_id);
CREATE INDEX IF NOT EXISTS idx_inventory_items_owner ON inventory_items(owner_id);
CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_active ON sessions(active) WHERE active = TRUE;