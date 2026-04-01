-- RegionStore Migration Version 57: Add Physics Inertia Data
-- This migration adds support for physics inertia calculation data

-- Add PhysInertia column to prims table (SQLite compatible)
ALTER TABLE prims ADD COLUMN PhysInertia TEXT DEFAULT NULL;