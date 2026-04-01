-- RegionStore Migration Version 67: Add Rez Start String Parameter
-- This migration adds support for rez start string parameter

-- Add StartStr column to prims table (SQLite compatible)
ALTER TABLE prims ADD COLUMN StartStr TEXT DEFAULT NULL;