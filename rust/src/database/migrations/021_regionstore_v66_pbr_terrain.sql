-- RegionStore Migration Version 66: Add PBR Terrain Storage
-- This migration adds support for PBR (Physically Based Rendering) terrain textures

-- Add PBR terrain texture columns to regionsettings table (SQLite compatible)
ALTER TABLE regionsettings ADD COLUMN TerrainPBR1 VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE regionsettings ADD COLUMN TerrainPBR2 VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE regionsettings ADD COLUMN TerrainPBR3 VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE regionsettings ADD COLUMN TerrainPBR4 VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';