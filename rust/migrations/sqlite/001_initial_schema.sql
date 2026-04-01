-- SQLite initial schema for OpenSim Next
-- Optimized for SQLite limitations and features

-- Users and authentication
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6)))),
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    salt TEXT NOT NULL,
    user_level INTEGER DEFAULT 0,
    user_flags INTEGER DEFAULT 0,
    user_title TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    last_login TEXT,
    home_region TEXT,
    home_location_x REAL DEFAULT 128.0,
    home_location_y REAL DEFAULT 128.0,
    home_location_z REAL DEFAULT 21.0,
    home_look_at_x REAL DEFAULT 1.0,
    home_look_at_y REAL DEFAULT 0.0,
    home_look_at_z REAL DEFAULT 0.0,
    profile_about TEXT,
    profile_first_text TEXT,
    profile_image TEXT,
    profile_partner TEXT,
    profile_url TEXT,
    profile_wants_to_mask INTEGER DEFAULT 0,
    profile_wants_to_text TEXT,
    profile_skills_mask INTEGER DEFAULT 0,
    profile_skills_text TEXT,
    profile_languages TEXT,
    active INTEGER DEFAULT 1
);

-- Regions
CREATE TABLE IF NOT EXISTS regions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6)))),
    region_name TEXT NOT NULL UNIQUE,
    location_x INTEGER NOT NULL,
    location_y INTEGER NOT NULL,
    size_x INTEGER DEFAULT 256,
    size_y INTEGER DEFAULT 256,
    external_host_name TEXT,
    external_port INTEGER,
    internal_host_name TEXT,
    internal_port INTEGER,
    map_texture TEXT,
    terrain_texture_1 TEXT,
    terrain_texture_2 TEXT,
    terrain_texture_3 TEXT,
    terrain_texture_4 TEXT,
    elevation_1_nw REAL DEFAULT 10.0,
    elevation_2_ne REAL DEFAULT 10.0,
    elevation_1_sw REAL DEFAULT 10.0,
    elevation_2_se REAL DEFAULT 10.0,
    water_height REAL DEFAULT 20.0,
    terrain_raise_limit REAL DEFAULT 100.0,
    terrain_lower_limit REAL DEFAULT -100.0,
    use_estate_sun INTEGER DEFAULT 1,
    sun_position REAL DEFAULT 0.0,
    covenant TEXT,
    sandbox INTEGER DEFAULT 0,
    public_access INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now')),
    last_seen TEXT DEFAULT (datetime('now')),
    physics_engine TEXT DEFAULT 'ODE',
    max_prims INTEGER DEFAULT 100000,
    max_agents INTEGER DEFAULT 100
);

-- Assets
CREATE TABLE IF NOT EXISTS assets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    asset_type INTEGER NOT NULL,
    local INTEGER DEFAULT 0,
    temporary INTEGER DEFAULT 0,
    data BLOB,
    created_at TEXT DEFAULT (datetime('now')),
    creator_id TEXT,
    asset_flags INTEGER DEFAULT 0,
    content_type TEXT,
    size_bytes INTEGER DEFAULT 0
);

-- Inventory
CREATE TABLE IF NOT EXISTS inventory_folders (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6)))),
    agent_id TEXT NOT NULL,
    parent_folder_id TEXT,
    folder_name TEXT NOT NULL,
    folder_type INTEGER DEFAULT 0,
    version INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS inventory_items (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6)))),
    asset_id TEXT NOT NULL,
    asset_type INTEGER NOT NULL,
    folder_id TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    creator_id TEXT NOT NULL,
    item_name TEXT NOT NULL,
    item_description TEXT,
    next_permissions INTEGER DEFAULT 0,
    current_permissions INTEGER DEFAULT 0,
    base_permissions INTEGER DEFAULT 0,
    everyone_permissions INTEGER DEFAULT 0,
    group_permissions INTEGER DEFAULT 0,
    group_id TEXT,
    group_owned INTEGER DEFAULT 0,
    sale_price INTEGER DEFAULT 0,
    sale_type INTEGER DEFAULT 0,
    flags INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now')),
    creation_date TEXT DEFAULT (datetime('now'))
);

-- Estates
CREATE TABLE IF NOT EXISTS estates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    estate_name TEXT NOT NULL,
    estate_owner TEXT NOT NULL,
    parent_estate_id INTEGER,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Session management
CREATE TABLE IF NOT EXISTS sessions (
    session_id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6)))),
    user_id TEXT NOT NULL,
    region_id TEXT,
    secure_session_id TEXT NOT NULL,
    circuit_code INTEGER NOT NULL,
    login_time TEXT DEFAULT (datetime('now')),
    logout_time TEXT,
    client_version TEXT,
    last_position_x REAL DEFAULT 128.0,
    last_position_y REAL DEFAULT 128.0,
    last_position_z REAL DEFAULT 21.0,
    last_look_at_x REAL DEFAULT 1.0,
    last_look_at_y REAL DEFAULT 0.0,
    last_look_at_z REAL DEFAULT 0.0,
    active INTEGER DEFAULT 1
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
CREATE INDEX IF NOT EXISTS idx_sessions_active ON sessions(active) WHERE active = 1;

-- Enable foreign key constraints
PRAGMA foreign_keys = ON;