-- RegionStore Migration Version 61: Add Pseudo CRC and Region Cache ID
-- This migration adds support for change detection and region caching

-- Add pseudocrc column to prims table (SQLite compatible)
ALTER TABLE prims ADD COLUMN pseudocrc INTEGER DEFAULT 0;

-- Add cacheID column to regionsettings table (SQLite compatible)
ALTER TABLE regionsettings ADD COLUMN cacheID CHAR(36) DEFAULT NULL;