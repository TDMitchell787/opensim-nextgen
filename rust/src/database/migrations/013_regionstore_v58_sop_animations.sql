-- RegionStore Migration Version 58: Add SOP Animations
-- This migration adds support for Scene Object Part animation data

-- Add sopanims column to prims table (SQLite compatible)
ALTER TABLE prims ADD COLUMN sopanims BLOB DEFAULT NULL;