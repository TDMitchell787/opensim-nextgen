-- Region configuration and data
CREATE TABLE IF NOT EXISTS regions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    region_name VARCHAR(128) UNIQUE NOT NULL,
    location_x INTEGER NOT NULL,
    location_y INTEGER NOT NULL,
    size_x INTEGER DEFAULT 256,
    size_y INTEGER DEFAULT 256,
    internal_ip VARCHAR(45) NOT NULL,
    internal_port INTEGER NOT NULL,
    external_host_name VARCHAR(255) NOT NULL,
    master_avatar_id UUID,
    owner_id UUID,
    scope_id UUID DEFAULT '00000000-0000-0000-0000-000000000000',
    region_secret VARCHAR(255),
    token VARCHAR(255),
    flags INTEGER DEFAULT 0,
    last_seen TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    prim_count INTEGER DEFAULT 0,
    agent_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(location_x, location_y)
);

-- Region terrain data
CREATE TABLE IF NOT EXISTS region_terrain (
    region_id UUID PRIMARY KEY REFERENCES regions(id) ON DELETE CASCADE,
    terrain_data BYTEA, -- Binary heightfield data
    terrain_revision INTEGER DEFAULT 1,
    terrain_seed INTEGER DEFAULT 0,
    water_height REAL DEFAULT 20.0,
    terrain_raise_limit REAL DEFAULT 100.0,
    terrain_lower_limit REAL DEFAULT -100.0,
    use_estate_sun BOOLEAN DEFAULT TRUE,
    fixed_sun BOOLEAN DEFAULT FALSE,
    sun_position REAL DEFAULT 0.0,
    covenant UUID,
    covenant_timestamp TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Region environment settings
CREATE TABLE IF NOT EXISTS region_environment (
    region_id UUID PRIMARY KEY REFERENCES regions(id) ON DELETE CASCADE,
    sky_preset_name VARCHAR(128),
    water_preset_name VARCHAR(128),
    environment_version INTEGER DEFAULT 1,
    sky_settings TEXT, -- JSON serialized sky settings
    water_settings TEXT, -- JSON serialized water settings
    environment_settings TEXT, -- JSON serialized environment data
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Scene objects (prims) in regions
CREATE TABLE IF NOT EXISTS scene_objects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    region_id UUID NOT NULL REFERENCES regions(id) ON DELETE CASCADE,
    group_id UUID NOT NULL, -- Object group UUID
    part_id UUID NOT NULL, -- Individual prim UUID
    parent_id UUID, -- Parent prim for child prims
    owner_id UUID NOT NULL,
    creator_id UUID NOT NULL,
    last_owner_id UUID,
    group_owner_id UUID,
    object_name VARCHAR(255) DEFAULT 'Object',
    description VARCHAR(255) DEFAULT '',
    position_x REAL DEFAULT 128.0,
    position_y REAL DEFAULT 128.0,
    position_z REAL DEFAULT 25.0,
    rotation_x REAL DEFAULT 0.0,
    rotation_y REAL DEFAULT 0.0,
    rotation_z REAL DEFAULT 0.0,
    rotation_w REAL DEFAULT 1.0,
    velocity_x REAL DEFAULT 0.0,
    velocity_y REAL DEFAULT 0.0,
    velocity_z REAL DEFAULT 0.0,
    angular_velocity_x REAL DEFAULT 0.0,
    angular_velocity_y REAL DEFAULT 0.0,
    angular_velocity_z REAL DEFAULT 0.0,
    acceleration_x REAL DEFAULT 0.0,
    acceleration_y REAL DEFAULT 0.0,
    acceleration_z REAL DEFAULT 0.0,
    scale_x REAL DEFAULT 1.0,
    scale_y REAL DEFAULT 1.0,
    scale_z REAL DEFAULT 1.0,
    object_flags INTEGER DEFAULT 0,
    physics_type INTEGER DEFAULT 0,
    material INTEGER DEFAULT 3,
    click_action INTEGER DEFAULT 0,
    color_r REAL DEFAULT 0.0,
    color_g REAL DEFAULT 0.0,
    color_b REAL DEFAULT 0.0,
    color_a REAL DEFAULT 0.0,
    texture_entry BYTEA, -- Binary texture data
    extra_physics_data BYTEA, -- Binary physics data
    shape_data TEXT, -- JSON serialized shape data
    script_state BYTEA, -- Binary script state data
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Region estate settings
CREATE TABLE IF NOT EXISTS region_estates (
    id SERIAL PRIMARY KEY,
    estate_name VARCHAR(128) UNIQUE NOT NULL,
    estate_owner UUID NOT NULL,
    parent_estate_id INTEGER,
    flags INTEGER DEFAULT 0,
    bill_cycle INTEGER DEFAULT 30,
    price_per_meter INTEGER DEFAULT 1,
    redirect_grid_x INTEGER DEFAULT 0,
    redirect_grid_y INTEGER DEFAULT 0,
    force_landing BOOLEAN DEFAULT FALSE,
    reset_home_on_teleport BOOLEAN DEFAULT FALSE,
    deny_anonymous BOOLEAN DEFAULT FALSE,
    deny_identified BOOLEAN DEFAULT FALSE,
    deny_transacted BOOLEAN DEFAULT FALSE,
    abuse_email VARCHAR(255) DEFAULT '',
    estate_owner_email VARCHAR(255) DEFAULT '',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Link regions to estates
CREATE TABLE IF NOT EXISTS region_estate_map (
    region_id UUID NOT NULL REFERENCES regions(id) ON DELETE CASCADE,
    estate_id INTEGER NOT NULL REFERENCES region_estates(id) ON DELETE CASCADE,
    PRIMARY KEY(region_id, estate_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_regions_location ON regions(location_x, location_y);
CREATE INDEX IF NOT EXISTS idx_regions_name ON regions(region_name);
CREATE INDEX IF NOT EXISTS idx_scene_objects_region ON scene_objects(region_id);
CREATE INDEX IF NOT EXISTS idx_scene_objects_group ON scene_objects(group_id);
CREATE INDEX IF NOT EXISTS idx_scene_objects_owner ON scene_objects(owner_id);
CREATE INDEX IF NOT EXISTS idx_region_estates_owner ON region_estates(estate_owner);