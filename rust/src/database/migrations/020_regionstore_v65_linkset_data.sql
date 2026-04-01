-- RegionStore Migration Version 65: Add Linkset Data Binary Storage
-- This migration adds support for linkset binary data storage

-- Add lnkstBinData column to prims table (SQLite compatible)
ALTER TABLE prims ADD COLUMN lnkstBinData BLOB DEFAULT NULL;