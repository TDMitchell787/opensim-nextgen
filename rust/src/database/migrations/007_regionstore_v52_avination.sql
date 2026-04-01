-- RegionStore Migration Version 52: Add Avination Fields
-- This migration adds support for collision passing, vehicle data, and parcel settings

-- Add new columns to prims table (SQLite compatible)
ALTER TABLE prims ADD COLUMN PassCollisions INTEGER NOT NULL DEFAULT 0;
ALTER TABLE prims ADD COLUMN Vehicle TEXT DEFAULT NULL;

-- Create regionsettings table if it doesn't exist (SQLite compatible)
CREATE TABLE IF NOT EXISTS regionsettings (
    regionUUID CHAR(36) NOT NULL PRIMARY KEY,
    block_terraform INTEGER NOT NULL DEFAULT 1,
    block_fly INTEGER NOT NULL DEFAULT 0,
    allow_damage INTEGER NOT NULL DEFAULT 0,
    restrict_pushing INTEGER NOT NULL DEFAULT 0,
    allow_land_resell INTEGER NOT NULL DEFAULT 1,
    allow_land_join_divide INTEGER NOT NULL DEFAULT 1,
    block_show_in_search INTEGER NOT NULL DEFAULT 0,
    agent_limit INTEGER NOT NULL DEFAULT 10,
    object_bonus REAL NOT NULL DEFAULT 1.0,
    maturity INTEGER NOT NULL DEFAULT 1,
    disable_scripts INTEGER NOT NULL DEFAULT 0,
    disable_collisions INTEGER NOT NULL DEFAULT 0,
    disable_physics INTEGER NOT NULL DEFAULT 0,
    terrain_texture_1 CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    terrain_texture_2 CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    terrain_texture_3 CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    terrain_texture_4 CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    elevation_1_nw REAL NOT NULL DEFAULT 10.0,
    elevation_2_ne REAL NOT NULL DEFAULT 10.0,
    elevation_1_se REAL NOT NULL DEFAULT 10.0,
    elevation_2_sw REAL NOT NULL DEFAULT 10.0,
    water_height REAL NOT NULL DEFAULT 20.0,
    terrain_raise_limit REAL NOT NULL DEFAULT 100.0,
    terrain_lower_limit REAL NOT NULL DEFAULT -100.0,
    use_estate_sun INTEGER NOT NULL DEFAULT 1,
    fixed_sun INTEGER NOT NULL DEFAULT 0,
    sun_position REAL NOT NULL DEFAULT 0.0,
    covenant CHAR(36) DEFAULT NULL,
    covenant_datetime INTEGER NOT NULL DEFAULT 0,
    Sandbox INTEGER NOT NULL DEFAULT 0,
    sunvectorx REAL NOT NULL DEFAULT 0.0,
    sunvectory REAL NOT NULL DEFAULT 0.0,
    sunvectorz REAL NOT NULL DEFAULT 0.0,
    loaded_creation_datetime INTEGER NOT NULL DEFAULT 0,
    loaded_creation_id VARCHAR(64) NOT NULL DEFAULT '',
    map_tile_ID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    minimum_age INTEGER NOT NULL DEFAULT 0,
    block_search INTEGER NOT NULL DEFAULT 0,
    casino INTEGER NOT NULL DEFAULT 0
);

-- Add new columns to land table (SQLite compatible - these fields already exist)
-- ALTER TABLE land ADD COLUMN SeeAVs INTEGER NOT NULL DEFAULT 1;
-- ALTER TABLE land ADD COLUMN AnyAVSounds INTEGER NOT NULL DEFAULT 1;
-- ALTER TABLE land ADD COLUMN GroupAVSounds INTEGER NOT NULL DEFAULT 1;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS prims_regionuuid ON prims(RegionUUID);
CREATE INDEX IF NOT EXISTS regionsettings_regionuuid ON regionsettings(regionUUID);
CREATE INDEX IF NOT EXISTS land_regionuuid ON land(RegionUUID);