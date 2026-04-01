-- OpenSim Compatibility Tables
-- Phase 26.1.3: Missing Tables Implementation
-- Adds critical tables for full OpenSim database compatibility

-- Object shape and geometry details (equivalent to OpenSim's primshapes)
CREATE TABLE IF NOT EXISTS prim_shapes (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prim_id UUID NOT NULL REFERENCES scene_objects(id) ON DELETE CASCADE,
    shape_type INTEGER DEFAULT 1,
    path_begin INTEGER DEFAULT 0,
    path_end INTEGER DEFAULT 0,
    path_scale_x INTEGER DEFAULT 100,
    path_scale_y INTEGER DEFAULT 100,
    path_shear_x INTEGER DEFAULT 0,
    path_shear_y INTEGER DEFAULT 0,
    path_skew INTEGER DEFAULT 0,
    path_curve INTEGER DEFAULT 16,
    path_radius_offset INTEGER DEFAULT 0,
    path_revolutions INTEGER DEFAULT 1,
    path_taper_x INTEGER DEFAULT 0,
    path_taper_y INTEGER DEFAULT 0,
    path_twist INTEGER DEFAULT 0,
    path_twist_begin INTEGER DEFAULT 0,
    profile_begin INTEGER DEFAULT 0,
    profile_end INTEGER DEFAULT 0,
    profile_curve INTEGER DEFAULT 1,
    profile_hollow INTEGER DEFAULT 0,
    texture_entry BYTEA,
    extra_params BYTEA,
    state INTEGER DEFAULT 0,
    last_attach_point INTEGER DEFAULT 0,
    media TEXT, -- JSON serialized media data
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Land parcel management system (equivalent to OpenSim's land table)
CREATE TABLE IF NOT EXISTS land_parcels (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    region_id UUID NOT NULL REFERENCES regions(id) ON DELETE CASCADE,
    local_land_id INTEGER NOT NULL,
    bitmap BYTEA, -- Land ownership bitmap
    name VARCHAR(255) DEFAULT 'Your Parcel',
    description TEXT DEFAULT '',
    owner_id UUID NOT NULL,
    group_id UUID,
    is_group_owned BOOLEAN DEFAULT FALSE,
    area INTEGER DEFAULT 0,
    auction_id INTEGER DEFAULT 0,
    category INTEGER DEFAULT 0, -- Parcel category (residential, commercial, etc.)
    claim_date INTEGER DEFAULT 0,
    claim_price INTEGER DEFAULT 0,
    status INTEGER DEFAULT 0,
    landing_type INTEGER DEFAULT 0,
    landing_x REAL DEFAULT 128.0,
    landing_y REAL DEFAULT 128.0,
    landing_z REAL DEFAULT 25.0,
    landing_look_x REAL DEFAULT 1.0,
    landing_look_y REAL DEFAULT 0.0,
    landing_look_z REAL DEFAULT 0.0,
    user_location_x REAL DEFAULT 128.0,
    user_location_y REAL DEFAULT 128.0,
    user_location_z REAL DEFAULT 25.0,
    user_look_at_x REAL DEFAULT 1.0,
    user_look_at_y REAL DEFAULT 0.0,
    user_look_at_z REAL DEFAULT 0.0,
    auth_buyer_id UUID,
    snapshot_id UUID,
    other_clean_time INTEGER DEFAULT 0,
    dwell REAL DEFAULT 0.0,
    media_auto_scale INTEGER DEFAULT 0,
    media_loop_set BOOLEAN DEFAULT FALSE,
    media_texture_id UUID,
    media_url VARCHAR(255) DEFAULT '',
    music_url VARCHAR(255) DEFAULT '',
    pass_hours REAL DEFAULT 0.0,
    pass_price INTEGER DEFAULT 0,
    sale_price INTEGER DEFAULT 0,
    media_type VARCHAR(32) DEFAULT 'none',
    media_description VARCHAR(255) DEFAULT '',
    media_size_x INTEGER DEFAULT 0,
    media_size_y INTEGER DEFAULT 0,
    media_loop BOOLEAN DEFAULT FALSE,
    obscure_media BOOLEAN DEFAULT FALSE,
    obscure_music BOOLEAN DEFAULT FALSE,
    see_avatar_distance REAL DEFAULT 20.0,
    any_avatar_sounds BOOLEAN DEFAULT TRUE,
    group_avatar_sounds BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(region_id, local_land_id)
);

-- Land access control list (equivalent to OpenSim's landaccesslist)
CREATE TABLE IF NOT EXISTS land_access_list (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    land_uuid UUID NOT NULL REFERENCES land_parcels(uuid) ON DELETE CASCADE,
    access_uuid UUID NOT NULL, -- User or group ID being granted/denied access
    flags INTEGER DEFAULT 0, -- Access type: 0=Access, 1=Ban
    expires TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Region-wide settings (equivalent to OpenSim's regionsettings)
CREATE TABLE IF NOT EXISTS region_settings (
    region_id UUID PRIMARY KEY REFERENCES regions(id) ON DELETE CASCADE,
    block_terraform BOOLEAN DEFAULT FALSE,
    block_fly BOOLEAN DEFAULT FALSE,
    allow_damage BOOLEAN DEFAULT FALSE,
    restrict_pushing BOOLEAN DEFAULT FALSE,
    allow_land_resell BOOLEAN DEFAULT TRUE,
    allow_land_join_divide BOOLEAN DEFAULT TRUE,
    block_show_in_search BOOLEAN DEFAULT FALSE,
    agent_limit INTEGER DEFAULT 40,
    object_bonus REAL DEFAULT 1.0,
    maturity INTEGER DEFAULT 1, -- 0=PG, 1=Mature, 2=Adult
    disable_scripts BOOLEAN DEFAULT FALSE,
    disable_collisions BOOLEAN DEFAULT FALSE,
    disable_physics BOOLEAN DEFAULT FALSE,
    terrain_texture_1 UUID,
    terrain_texture_2 UUID,
    terrain_texture_3 UUID,
    terrain_texture_4 UUID,
    elevation_1_nw REAL DEFAULT 10.0,
    elevation_2_nw REAL DEFAULT 10.0,
    elevation_1_ne REAL DEFAULT 10.0,
    elevation_2_ne REAL DEFAULT 10.0,
    elevation_1_se REAL DEFAULT 10.0,
    elevation_2_se REAL DEFAULT 10.0,
    elevation_1_sw REAL DEFAULT 10.0,
    elevation_2_sw REAL DEFAULT 10.0,
    water_height REAL DEFAULT 20.0,
    terrain_raise_limit REAL DEFAULT 100.0,
    terrain_lower_limit REAL DEFAULT -100.0,
    use_estate_sun BOOLEAN DEFAULT TRUE,
    fixed_sun BOOLEAN DEFAULT FALSE,
    sun_position REAL DEFAULT 0.0,
    covenant UUID,
    covenant_datetime INTEGER DEFAULT 0,
    sandbox BOOLEAN DEFAULT FALSE,
    sunvectorx REAL DEFAULT 1.0,
    sunvectory REAL DEFAULT 0.0,
    sunvectorz REAL DEFAULT 0.3,
    loaded_creation_id VARCHAR(64) DEFAULT '',
    loaded_creation_datetime INTEGER DEFAULT 0,
    map_tile_id UUID,
    telehub_id UUID,
    spawn_point_routing INTEGER DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Spawn points for region teleport/login (equivalent to OpenSim's spawn_points)
CREATE TABLE IF NOT EXISTS region_spawn_points (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    region_id UUID NOT NULL REFERENCES regions(id) ON DELETE CASCADE,
    spawn_point_x REAL NOT NULL,
    spawn_point_y REAL NOT NULL,
    spawn_point_z REAL NOT NULL,
    spawn_point_look_x REAL DEFAULT 1.0,
    spawn_point_look_y REAL DEFAULT 0.0,
    spawn_point_look_z REAL DEFAULT 0.0,
    name VARCHAR(255) DEFAULT 'Spawn Point',
    description TEXT DEFAULT '',
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Environment settings per region (WindLight/EEP support)
CREATE TABLE IF NOT EXISTS region_windlight (
    region_id UUID PRIMARY KEY REFERENCES regions(id) ON DELETE CASCADE,
    water_color_r REAL DEFAULT 0.0,
    water_color_g REAL DEFAULT 0.0,
    water_color_b REAL DEFAULT 0.0,
    water_fog_density_exponent REAL DEFAULT 4.0,
    underwater_fog_modifier REAL DEFAULT 0.25,
    reflection_wavelet_scale_1 REAL DEFAULT 2.0,
    reflection_wavelet_scale_2 REAL DEFAULT 2.0,
    reflection_wavelet_scale_3 REAL DEFAULT 2.0,
    fresnel_scale REAL DEFAULT 0.40,
    fresnel_offset REAL DEFAULT 0.50,
    refract_scale_above REAL DEFAULT 0.03,
    refract_scale_below REAL DEFAULT 0.20,
    blur_multiplier REAL DEFAULT 0.040,
    big_wave_direction_x REAL DEFAULT 1.05,
    big_wave_direction_y REAL DEFAULT -0.42,
    little_wave_direction_x REAL DEFAULT 1.11,
    little_wave_direction_y REAL DEFAULT -1.16,
    normal_map_texture UUID,
    horizon_r REAL DEFAULT 0.25,
    horizon_g REAL DEFAULT 0.25,
    horizon_b REAL DEFAULT 0.32,
    horizon_i REAL DEFAULT 0.32,
    haze_horizon REAL DEFAULT 0.19,
    blue_density_r REAL DEFAULT 0.12,
    blue_density_g REAL DEFAULT 0.22,
    blue_density_b REAL DEFAULT 0.38,
    blue_density_i REAL DEFAULT 0.38,
    haze_density REAL DEFAULT 0.70,
    density_multiplier REAL DEFAULT 0.18,
    distance_multiplier REAL DEFAULT 0.8,
    max_altitude INTEGER DEFAULT 1605,
    sun_moon_color_r REAL DEFAULT 0.24,
    sun_moon_color_g REAL DEFAULT 0.26,
    sun_moon_color_b REAL DEFAULT 0.30,
    sun_moon_color_i REAL DEFAULT 0.30,
    sun_moon_position REAL DEFAULT 0.317,
    ambient_r REAL DEFAULT 0.35,
    ambient_g REAL DEFAULT 0.35,
    ambient_b REAL DEFAULT 0.35,
    ambient_i REAL DEFAULT 0.35,
    east_angle REAL DEFAULT 0.0,
    sun_glow_focus REAL DEFAULT 0.10,
    sun_glow_size REAL DEFAULT 1.75,
    scene_gamma REAL DEFAULT 1.0,
    star_brightness REAL DEFAULT 0.0,
    cloud_color_r REAL DEFAULT 0.41,
    cloud_color_g REAL DEFAULT 0.41,
    cloud_color_b REAL DEFAULT 0.41,
    cloud_color_i REAL DEFAULT 0.41,
    cloud_x REAL DEFAULT 1.0,
    cloud_y REAL DEFAULT 0.53,
    cloud_density REAL DEFAULT 1.0,
    cloud_coverage REAL DEFAULT 0.27,
    cloud_scale REAL DEFAULT 0.42,
    cloud_detail_x REAL DEFAULT 1.0,
    cloud_detail_y REAL DEFAULT 0.53,
    cloud_detail_density REAL DEFAULT 0.12,
    cloud_scroll_x REAL DEFAULT 0.20,
    cloud_scroll_x_lock BOOLEAN DEFAULT FALSE,
    cloud_scroll_y REAL DEFAULT 0.01,
    cloud_scroll_y_lock BOOLEAN DEFAULT FALSE,
    draw_classic_clouds BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Performance indexes for OpenSim compatibility
CREATE INDEX IF NOT EXISTS idx_prim_shapes_prim ON prim_shapes(prim_id);
CREATE INDEX IF NOT EXISTS idx_prim_shapes_type ON prim_shapes(shape_type);

CREATE INDEX IF NOT EXISTS idx_land_parcels_region ON land_parcels(region_id);
CREATE INDEX IF NOT EXISTS idx_land_parcels_owner ON land_parcels(owner_id);
CREATE INDEX IF NOT EXISTS idx_land_parcels_group ON land_parcels(group_id);
CREATE INDEX IF NOT EXISTS idx_land_parcels_local_id ON land_parcels(region_id, local_land_id);

CREATE INDEX IF NOT EXISTS idx_land_access_land ON land_access_list(land_uuid);
CREATE INDEX IF NOT EXISTS idx_land_access_uuid ON land_access_list(access_uuid);
CREATE INDEX IF NOT EXISTS idx_land_access_flags ON land_access_list(flags);

CREATE INDEX IF NOT EXISTS idx_region_spawn_points_region ON region_spawn_points(region_id);
CREATE INDEX IF NOT EXISTS idx_region_spawn_points_default ON region_spawn_points(region_id, is_default);

-- Enhanced indexes for spatial queries and performance
CREATE INDEX IF NOT EXISTS idx_scene_objects_region_position ON scene_objects(region_id, position_x, position_y);
CREATE INDEX IF NOT EXISTS idx_scene_objects_parent ON scene_objects(parent_id);
CREATE INDEX IF NOT EXISTS idx_scene_objects_group_owner ON scene_objects(group_id, owner_id);

-- Land parcel spatial queries
CREATE INDEX IF NOT EXISTS idx_land_parcels_area ON land_parcels(region_id, area);
CREATE INDEX IF NOT EXISTS idx_land_parcels_sale ON land_parcels(region_id, sale_price) WHERE sale_price > 0;