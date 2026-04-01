-- RegionStore Migration Version 56: Add RezzerID Field
-- This migration adds support for tracking which object/avatar rezzed a prim

-- Add RezzerID column to prims table (SQLite compatible)
ALTER TABLE prims ADD COLUMN RezzerID CHAR(36) DEFAULT NULL;