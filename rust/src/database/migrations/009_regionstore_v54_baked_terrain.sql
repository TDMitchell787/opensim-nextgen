-- RegionStore Migration Version 54: Add Baked Terrain Store
-- This migration adds support for baked terrain storage

-- Create bakedterrain table for storing baked terrain data (SQLite compatible)
CREATE TABLE IF NOT EXISTS bakedterrain (
    RegionUUID VARCHAR(255) DEFAULT NULL,
    Revision INTEGER DEFAULT NULL,
    Heightfield BLOB,
    PRIMARY KEY (RegionUUID, Revision)
);

-- Create index for performance
CREATE INDEX IF NOT EXISTS bakedterrain_regionuuid ON bakedterrain(RegionUUID);