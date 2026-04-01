-- RegionStore Migration Version 53: Add Rotation Axis Locks
-- This migration adds support for rotation axis locking in prims

-- Add rotation axis locks column to prims table (SQLite compatible)
ALTER TABLE prims ADD COLUMN RotationAxisLocks INTEGER NOT NULL DEFAULT 0;